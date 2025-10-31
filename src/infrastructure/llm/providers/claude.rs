use crate::domain::services::llm_service::{
    LLMProvider, LLMError, ChatRequest, ChatResponse, ChatStreamChunk, ModelInfo, 
    ConnectionTestResult, TokenUsage, FinishReason
};
use crate::infrastructure::llm::providers::{
    HttpClient, HttpClientConfig, ProviderConfig, ProviderUtils
};
use crate::infrastructure::llm::streaming::StreamAdapter;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


/// Claude (Anthropic) API provider implementation
pub struct ClaudeProvider {
    config: ProviderConfig,
    http_client: HttpClient,
}

#[derive(Debug, Serialize)]
struct ClaudeChatRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ClaudeChatResponse {
    id: String,
    r#type: String,
    role: String,
    content: Vec<ClaudeContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    r#type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ClaudeErrorResponse {
    r#type: String,
    error: ClaudeError,
}

#[derive(Debug, Deserialize)]
struct ClaudeError {
    r#type: String,
    message: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String) -> Result<Self, LLMError> {
        ProviderUtils::validate_api_key(&api_key, "claude")?;
        
        let base_url = "https://api.anthropic.com/v1".to_string();

        let config = ProviderConfig {
            api_key,
            base_url,
            default_model: "claude-3-haiku-20240307".to_string(),
            http_config: HttpClientConfig::default(),
            custom_headers: HashMap::new(),
        };

        let http_client = HttpClient::new(config.http_config.clone())?;

        Ok(Self {
            config,
            http_client,
        })
    }

    pub fn with_custom_config(mut self, http_config: HttpClientConfig) -> Result<Self, LLMError> {
        self.http_client = HttpClient::new(http_config.clone())?;
        self.config.http_config = http_config;
        Ok(self)
    }

    pub fn add_custom_header(&mut self, key: String, value: String) {
        self.config.custom_headers.insert(key, value);
    }

    fn build_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), self.config.api_key.clone());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        
        for (key, value) in &self.config.custom_headers {
            headers.insert(key.clone(), value.clone());
        }
        
        headers
    }

    fn convert_request(&self, request: ChatRequest) -> Result<ClaudeChatRequest, LLMError> {
        use crate::domain::value_objects::chat_message::{MessageContent, ContentPart};
        
        let mut messages = Vec::new();
        let mut system_message = None;

        for msg in request.messages {
            let content = match &msg.content {
                MessageContent::Text(text) => serde_json::json!(text),
                MessageContent::Multimodal(parts) => {
                    let content_parts: Vec<serde_json::Value> = parts
                        .iter()
                        .map(|part| match part {
                            ContentPart::Text { text } => serde_json::json!({
                                "type": "text",
                                "text": text
                            }),
                            ContentPart::ImageUrl { image_url } => {
                                serde_json::json!({
                                    "type": "image",
                                    "source": {
                                        "type": "url",
                                        "url": image_url.url
                                    }
                                })
                            }
                        })
                        .collect();
                    serde_json::json!(content_parts)
                }
            };
            
            match msg.role {
                crate::domain::value_objects::MessageRole::System => {
                    // Claude expects system message as a string
                    system_message = Some(msg.get_text_content());
                }
                crate::domain::value_objects::MessageRole::User => {
                    messages.push(ClaudeMessage {
                        role: "user".to_string(),
                        content,
                    });
                }
                crate::domain::value_objects::MessageRole::Assistant => {
                    messages.push(ClaudeMessage {
                        role: "assistant".to_string(),
                        content,
                    });
                }
                crate::domain::value_objects::MessageRole::Tool => {
                    // Claude doesn't have a separate tool role, treat as user message
                    messages.push(ClaudeMessage {
                        role: "user".to_string(),
                        content,
                    });
                }
            }
        }

        // Claude requires at least max_tokens to be set
        let max_tokens = request.max_tokens.unwrap_or(1000);

        Ok(ClaudeChatRequest {
            model: request.model,
            messages,
            system: system_message,
            max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: request.stop_sequences,
            stream: request.stream,
        })
    }

    fn convert_response(&self, response: ClaudeChatResponse) -> Result<ChatResponse, LLMError> {
        let content = response.content
            .first()
            .ok_or_else(|| LLMError::ProviderError("No content in response".to_string()))?;

        let finish_reason = match response.stop_reason.as_deref() {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("stop_sequence") => FinishReason::Stop,
            _ => FinishReason::Stop,
        };

        let usage = TokenUsage {
            prompt_tokens: response.usage.input_tokens,
            completion_tokens: response.usage.output_tokens,
            total_tokens: response.usage.input_tokens + response.usage.output_tokens,
        };

        Ok(ChatResponse {
            content: content.text.clone(),
            model_used: response.model,
            usage,
            finish_reason,
            metadata: None,
        })
    }

    async fn make_test_request(&self) -> Result<(), LLMError> {
        let url = format!("{}/messages", self.config.base_url);
        let headers = self.build_headers();
        
        let test_request = ClaudeChatRequest {
            model: self.config.default_model.clone(),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: serde_json::json!("Hello"),
            }],
            system: None,
            max_tokens: 10,
            temperature: Some(0.1),
            top_p: None,
            stop_sequences: None,
            stream: false,
        };

        let _response: ClaudeChatResponse = self.http_client
            .post_json(&url, &headers, &test_request)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl LLMProvider for ClaudeProvider {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, LLMError> {
        let url = format!("{}/messages", self.config.base_url);
        let headers = self.build_headers();
        let claude_request = self.convert_request(request)?;

        let response: ClaudeChatResponse = self.http_client
            .post_json(&url, &headers, &claude_request)
            .await?;

        self.convert_response(response)
    }

    async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>, LLMError> {
        // Claude doesn't provide embedding endpoints
        Err(LLMError::ProviderError(
            "Claude does not support embeddings. Use OpenAI or another provider for embeddings.".to_string()
        ))
    }

    async fn stream_chat_completion(
        &self,
        request: ChatRequest,
    ) -> Result<Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>, LLMError> {
        let url = format!("{}/messages", self.config.base_url);
        let headers = self.build_headers();
        let mut claude_request = self.convert_request(request)?;
        claude_request.stream = true;

        let response = self.http_client
            .post_stream(&url, &headers, &claude_request)
            .await?;

        let byte_stream = response.bytes_stream().map(|result| {
            result.map_err(|e| LLMError::NetworkError(format!("Stream error: {}", e)))
        });

        Ok(StreamAdapter::from_bytes_stream(Box::pin(byte_stream)))
    }

    fn get_model_info(&self) -> Vec<ModelInfo> {
        ProviderUtils::create_default_models("claude")
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    async fn test_connection(&self) -> Result<ConnectionTestResult, LLMError> {
        let start_time = std::time::Instant::now();
        
        match self.make_test_request().await {
            Ok(()) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                Ok(ConnectionTestResult {
                    success: true,
                    response_time_ms: response_time,
                    error_message: None,
                    model_info: Some(ModelInfo {
                        id: self.config.default_model.clone(),
                        name: "Claude 3 Haiku".to_string(),
                        description: Some("Fast and cost-effective Claude model".to_string()),
                        context_length: Some(200000),
                        supports_streaming: true,
                        supports_tools: true,
                        supports_vision: true,
                    }),
                })
            }
            Err(e) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                Ok(ConnectionTestResult {
                    success: false,
                    response_time_ms: response_time,
                    error_message: Some(e.to_string()),
                    model_info: None,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{ChatMessage, MessageRole};
    use chrono::Utc;

    fn create_test_provider() -> ClaudeProvider {
        ClaudeProvider::new("sk-ant-test1234567890".to_string()).unwrap()
    }

    #[test]
    fn test_provider_creation() {
        let provider = create_test_provider();
        assert_eq!(provider.config.base_url, "https://api.anthropic.com/v1");
        assert!(provider.config.api_key.starts_with("sk-ant-"));
    }

    #[test]
    fn test_invalid_api_key() {
        let result = ClaudeProvider::new("invalid-key".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_request() {
        use crate::domain::value_objects::chat_message::MessageContent;
        
        let provider = create_test_provider();
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: MessageContent::Text("You are a helpful assistant".to_string()),
                metadata: None,
                timestamp: Utc::now(),
            },
            ChatMessage {
                role: MessageRole::User,
                content: MessageContent::Text("Hello".to_string()),
                metadata: None,
                timestamp: Utc::now(),
            },
        ];

        let request = ChatRequest {
            messages,
            model: "claude-3-haiku-20240307".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: Some(1.0),
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: None,
            stream: false,
            tenant_id: uuid::Uuid::new_v4(),
        };

        let claude_request = provider.convert_request(request).unwrap();
        assert_eq!(claude_request.model, "claude-3-haiku-20240307");
        assert_eq!(claude_request.messages.len(), 1); // System message is separate
        assert_eq!(claude_request.messages[0].role, "user");
        assert_eq!(claude_request.messages[0].content, serde_json::json!("Hello"));
        assert_eq!(claude_request.system, Some("You are a helpful assistant".to_string()));
        assert_eq!(claude_request.max_tokens, 1000);
        assert!(!claude_request.stream);
    }

    #[test]
    fn test_build_headers() {
        let provider = create_test_provider();
        let headers = provider.build_headers();
        
        assert!(headers.contains_key("x-api-key"));
        assert!(headers.get("x-api-key").unwrap().starts_with("sk-ant-"));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(headers.get("anthropic-version"), Some(&"2023-06-01".to_string()));
    }

    #[test]
    fn test_custom_headers() {
        let mut provider = create_test_provider();
        provider.add_custom_header("X-Custom-Header".to_string(), "custom-value".to_string());
        
        let headers = provider.build_headers();
        assert_eq!(headers.get("X-Custom-Header"), Some(&"custom-value".to_string()));
    }

    #[test]
    fn test_supports_streaming() {
        let provider = create_test_provider();
        assert!(provider.supports_streaming());
    }

    #[test]
    fn test_get_model_info() {
        let provider = create_test_provider();
        let models = provider.get_model_info();
        
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id.contains("claude-3")));
        assert!(models.iter().any(|m| m.supports_streaming));
    }

    #[tokio::test]
    async fn test_embedding_not_supported() {
        let provider = create_test_provider();
        let result = provider.generate_embedding("test").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not support embeddings"));
    }
}