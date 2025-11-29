use crate::domain::services::llm_service::{ChatStreamChunk, LLMError, FinishReason, TokenUsage};
use futures::{Stream, StreamExt};
use serde_json::Value;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Stream wrapper for handling LLM streaming responses
pub struct LLMStream {
    inner: Pin<Box<dyn Stream<Item = Result<String, LLMError>> + Send>>,
    buffer: String,
    finished: bool,
}

// Implement Unpin for LLMStream so it can be used in Box instead of Pin<Box>
impl Unpin for LLMStream {}

impl LLMStream {
    pub fn new(stream: Pin<Box<dyn Stream<Item = Result<String, LLMError>> + Send>>) -> Self {
        Self {
            inner: stream,
            buffer: String::new(),
            finished: false,
        }
    }

    /// Parse Server-Sent Events (SSE) format commonly used by LLM APIs
    pub fn parse_sse_chunk(data: &str) -> Result<Option<ChatStreamChunk>, LLMError> {
        if data.trim().is_empty() || data.starts_with(": ") {
            return Ok(None);
        }

        let mut event_data = None;
        
        for line in data.lines() {
            if let Some(data_content) = line.strip_prefix("data: ") {
                if data_content.trim() == "[DONE]" {
                    return Ok(Some(ChatStreamChunk {
                        content: None,
                        reasoning_content: None,
                        finish_reason: Some(FinishReason::Stop),
                        usage: None,
                    }));
                }
                event_data = Some(data_content);
                break;
            }
        }

        if let Some(json_str) = event_data {
            match serde_json::from_str::<Value>(json_str) {
                Ok(json) => Self::parse_streaming_response(json),
                Err(e) => Err(LLMError::SerializationError(format!("Failed to parse JSON: {}", e))),
            }
        } else {
            Ok(None)
        }
    }

    /// Parse different provider streaming response formats
    fn parse_streaming_response(json: Value) -> Result<Option<ChatStreamChunk>, LLMError> {
        // OpenAI format
        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
            if let Some(choice) = choices.first() {
                let content = choice
                    .get("delta")
                    .and_then(|d| d.get("content"))
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());

                let reasoning_content = choice
                    .get("delta")
                    .and_then(|d| d.get("reasoning_content"))
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());

                let finish_reason = choice
                    .get("finish_reason")
                    .and_then(|r| r.as_str())
                    .and_then(|r| match r {
                        "stop" => Some(FinishReason::Stop),
                        "length" => Some(FinishReason::Length),
                        "content_filter" => Some(FinishReason::ContentFilter),
                        "tool_calls" => Some(FinishReason::ToolCalls),
                        _ => None,
                    });

                let usage = json.get("usage").and_then(|u| {
                    let prompt_tokens = u.get("prompt_tokens")?.as_u64()? as u32;
                    let completion_tokens = u.get("completion_tokens")?.as_u64()? as u32;
                    Some(TokenUsage {
                        prompt_tokens,
                        completion_tokens,
                        total_tokens: prompt_tokens + completion_tokens,
                    })
                });

                return Ok(Some(ChatStreamChunk {
                    content,
                    reasoning_content,
                    finish_reason,
                    usage,
                }));
            } else {
                let usage = json.get("usage").and_then(|u| {
                    let prompt_tokens = u.get("prompt_tokens")?.as_u64()? as u32;
                    let completion_tokens = u.get("completion_tokens")?.as_u64()? as u32;
                    Some(TokenUsage {
                        prompt_tokens,
                        completion_tokens,
                        total_tokens: prompt_tokens + completion_tokens,
                    })
                });

                if usage.is_some() {
                    return Ok(Some(ChatStreamChunk {
                        content: None,
                        reasoning_content: None,
                        finish_reason: None,
                        usage,
                    }));
                }
            }
        }

        // Claude format
        if let Some(event_type) = json.get("type").and_then(|t| t.as_str()) {
            match event_type {
                "content_block_delta" => {
                    let content = json
                        .get("delta")
                        .and_then(|d| d.get("text"))
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string());

                    return Ok(Some(ChatStreamChunk {
                        content,
                        reasoning_content: None,
                        finish_reason: None,
                        usage: None,
                    }));
                }
                "message_stop" => {
                    return Ok(Some(ChatStreamChunk {
                        content: None,
                        reasoning_content: None,
                        finish_reason: Some(FinishReason::Stop),
                        usage: None,
                    }));
                }
                _ => return Ok(None),
            }
        }

        // Generic format fallback
        if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
            return Ok(Some(ChatStreamChunk {
                content: Some(content.to_string()),
                reasoning_content: None,
                finish_reason: None,
                usage: None,
            }));
        }

        Ok(None)
    }
}

impl Stream for LLMStream {
    type Item = Result<ChatStreamChunk, LLMError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.finished {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(data))) => {
                self.buffer.push_str(&data);
                
                // Try to parse complete SSE events from buffer
                if let Some(double_newline_pos) = self.buffer.find("\n\n") {
                    let event_data = self.buffer[..double_newline_pos].to_string();
                    self.buffer.drain(..double_newline_pos + 2);
                    
                    match Self::parse_sse_chunk(&event_data) {
                        Ok(Some(chunk)) => {
                            // To collect usage so checking finish_reason is not enough.
                            // if chunk.finish_reason.is_some() {
                            if chunk.usage.is_some() {
                                self.finished = true;
                            }
                            Poll::Ready(Some(Ok(chunk)))
                        }
                        Ok(None) => {
                            // Continue polling for next chunk
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        }
                        Err(e) => {
                            Poll::Ready(Some(Err(e)))
                        },
                    }
                } else {
                    // Need more data to complete the event
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => {
                self.finished = true;
                Poll::Ready(Some(Err(e)))
            }
            Poll::Ready(None) => {
                // Process any remaining data in buffer
                if !self.buffer.trim().is_empty() {
                    if let Some(double_newline_pos) = self.buffer.find("\n\n") {
                        let event_data = self.buffer[..double_newline_pos].to_string();
                        self.buffer.drain(..double_newline_pos + 2);
                        
                        match Self::parse_sse_chunk(&event_data) {
                            Ok(Some(chunk)) => {
                                // To collect usage so checking finish_reason is not enough.
                                // if chunk.finish_reason.is_some() {
                                if chunk.usage.is_some() {
                                    self.finished = true;
                                }
                                Poll::Ready(Some(Ok(chunk)))
                            }
                            Ok(None) => {
                                // Continue polling for next chunk
                                cx.waker().wake_by_ref();
                                Poll::Pending
                            }
                            Err(e) => {
                                Poll::Ready(Some(Err(e)))
                            },
                        }
                    } else {
                        Poll::Ready(None)
                    }
                } else {
                    self.finished = true;
                
                    Poll::Ready(None)
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream adapter for different LLM providers
pub struct StreamAdapter;

impl StreamAdapter {
    /// Create a stream from raw HTTP response bytes
    pub fn from_bytes_stream(
        stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, LLMError>> + Send>>,
    ) -> Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin> {
        let string_stream = stream.map(|result| {
            result.and_then(|bytes| {
                String::from_utf8(bytes.to_vec())
                    .map_err(|e| LLMError::SerializationError(format!("Invalid UTF-8: {}", e)))
            })
        });

        Box::new(LLMStream::new(Box::pin(string_stream)))
    }

    /// Create a stream from Server-Sent Events
    pub fn from_sse_stream(
        stream: Pin<Box<dyn Stream<Item = Result<String, LLMError>> + Send>>,
    ) -> Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin> {
        Box::new(LLMStream::new(stream))
    }

    /// Buffer stream chunks to reduce the number of small updates
    pub fn buffer_stream(
        stream: Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>,
        buffer_size: usize,
    ) -> Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin> {
        Box::new(BufferedStream::new(stream, buffer_size))
    }
}

/// Buffered stream to accumulate small chunks
struct BufferedStream {
    inner: Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>,
    buffer: String,
    reasoning_buffer: String,
    buffer_size: usize,
    finished: bool,
}

// Implement Unpin for BufferedStream
impl Unpin for BufferedStream {}

impl BufferedStream {
    fn new(
        stream: Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>,
        buffer_size: usize,
    ) -> Self {
        Self {
            inner: stream,
            buffer: String::new(),
            reasoning_buffer: String::new(),
            buffer_size,
            finished: false,
        }
    }
}

impl Stream for BufferedStream {
    type Item = Result<ChatStreamChunk, LLMError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.finished && self.buffer.is_empty() {
            return Poll::Ready(None);
        }

        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    if let Some(content) = chunk.content {
                        self.buffer.push_str(&content);
                    }

                    if let Some(reasoning_content) = chunk.reasoning_content {
                        self.reasoning_buffer.push_str(&reasoning_content);
                    }

                    // If we have a finish reason or buffer is full, emit the chunk
                    if chunk.finish_reason.is_some() || self.buffer.len() >= self.buffer_size {
                        let buffered_content = if self.buffer.is_empty() {
                            None
                        } else {
                            Some(std::mem::take(&mut self.buffer))
                        };

                        let buffered_reasoning_content = if self.reasoning_buffer.is_empty() {
                            None
                        } else {
                            Some(std::mem::take(&mut self.reasoning_buffer))
                        };

                        if chunk.finish_reason.is_some() {
                            self.finished = true;
                        }

                        return Poll::Ready(Some(Ok(ChatStreamChunk {
                            content: buffered_content,
                            reasoning_content: buffered_reasoning_content,
                            finish_reason: chunk.finish_reason,
                            usage: chunk.usage,
                        })));
                    }
                    // Continue accumulating
                }
                Poll::Ready(Some(Err(e))) => {
                    self.finished = true;
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Ready(None) => {
                    self.finished = true;

                    let content = if !self.buffer.is_empty() {
                        std::mem::take(&mut self.buffer)
                    } else {
                        String::new()
                    };

                    let reasoning_content = if !self.reasoning_buffer.is_empty() {
                        std::mem::take(&mut self.reasoning_buffer)
                    } else {
                        String::new()
                    };
                    
                    // Emit any remaining buffered content
                    if !content.is_empty() {
                        if !reasoning_content.is_empty() {
                            return Poll::Ready(Some(Ok(ChatStreamChunk {
                                content: Some(content),
                                reasoning_content: Some(reasoning_content),
                                finish_reason: Some(FinishReason::Stop),
                                usage: None,
                            })));
                        } else {
                            return Poll::Ready(Some(Ok(ChatStreamChunk {
                                content: Some(content),
                                reasoning_content: None,
                                finish_reason: Some(FinishReason::Stop),
                                usage: None,
                            })));
                        }
                    } else {
                        if !reasoning_content.is_empty() {
                            return Poll::Ready(Some(Ok(ChatStreamChunk {
                                content: None,
                                reasoning_content: Some(reasoning_content),
                                finish_reason: Some(FinishReason::Stop),
                                usage: None,
                            })));
                        } else {
                            return Poll::Ready(None);
                        }
                    }
                }
                Poll::Pending => {
                    // If we have buffered content and no more data is immediately available,
                    // emit what we have
                    let content = if !self.buffer.is_empty() {
                        std::mem::take(&mut self.buffer)
                    } else {
                        String::new()
                    };

                    let reasoning_content = if !self.reasoning_buffer.is_empty() {
                        std::mem::take(&mut self.reasoning_buffer)
                    } else {
                        String::new()
                    };
                    
                    // Emit any remaining buffered content
                    if !content.is_empty() {
                        if !reasoning_content.is_empty() {
                            return Poll::Ready(Some(Ok(ChatStreamChunk {
                                content: Some(content),
                                reasoning_content: Some(reasoning_content),
                                finish_reason: None,
                                usage: None,
                            })));
                        } else {
                            return Poll::Ready(Some(Ok(ChatStreamChunk {
                                content: Some(content),
                                reasoning_content: None,
                                finish_reason: None,
                                usage: None,
                            })));
                        }
                    } else {
                        if !reasoning_content.is_empty() {
                            return Poll::Ready(Some(Ok(ChatStreamChunk {
                                content: None,
                                reasoning_content: Some(reasoning_content),
                                finish_reason: None,
                                usage: None,
                            })));
                        } else {
                            return Poll::Pending;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    #[test]
    fn test_parse_openai_sse_chunk() {
        let sse_data = r#"data: {"choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        
        let result = LLMStream::parse_sse_chunk(sse_data).unwrap();
        assert!(result.is_some());
        
        let chunk = result.unwrap();
        assert_eq!(chunk.content, Some("Hello".to_string()));
        assert!(chunk.finish_reason.is_none());
    }

    #[test]
    fn test_parse_claude_sse_chunk() {
        let sse_data = r#"data: {"type":"content_block_delta","delta":{"text":"Hello"}}"#;
        
        let result = LLMStream::parse_sse_chunk(sse_data).unwrap();
        assert!(result.is_some());
        
        let chunk = result.unwrap();
        assert_eq!(chunk.content, Some("Hello".to_string()));
        assert!(chunk.finish_reason.is_none());
    }

    #[test]
    fn test_parse_done_chunk() {
        let sse_data = "data: [DONE]";
        
        let result = LLMStream::parse_sse_chunk(sse_data).unwrap();
        assert!(result.is_some());
        
        let chunk = result.unwrap();
        assert!(chunk.content.is_none());
        assert_eq!(chunk.finish_reason, Some(FinishReason::Stop));
    }

    #[tokio::test]
    async fn test_buffered_stream() {
        let chunks = vec![
            Ok(ChatStreamChunk {
                content: Some("Hello".to_string()),
                finish_reason: None,
                usage: None,
            }),
            Ok(ChatStreamChunk {
                content: Some(" world".to_string()),
                finish_reason: None,
                usage: None,
            }),
            Ok(ChatStreamChunk {
                content: Some("!".to_string()),
                finish_reason: Some(FinishReason::Stop),
                usage: None,
            }),
        ];

        let stream = stream::iter(chunks);
        let buffered = StreamAdapter::buffer_stream(Box::new(stream), 20);
        
        let collected: Vec<_> = buffered.collect().await;
        assert_eq!(collected.len(), 1);
        
        let chunk = collected[0].as_ref().unwrap();
        assert_eq!(chunk.content, Some("Hello world!".to_string()));
        assert_eq!(chunk.finish_reason, Some(FinishReason::Stop));
    }
}