use crate::domain::services::llm_service::{
    LLMProvider, LLMError, ChatRequest, ChatResponse, ChatStreamChunk, ModelInfo, 
    ConnectionTestResult, TokenUsage, FinishReason
};
use crate::infrastructure::llm::providers::{
    HttpClient, HttpClientConfig, ProviderConfig, ProviderUtils, StandardChatResponse
};
use crate::infrastructure::llm::streaming::StreamAdapter;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tracing::{debug};

/// OpenAI API provider implementation
pub struct OpenAIProvider {
    config: ProviderConfig,
    http_client: HttpClient,
}

#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelInfo {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

#[derive(Debug, Serialize)]
struct OpenAIEmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
    usage: OpenAIEmbeddingUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
    index: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

impl OpenAIProvider {
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self, LLMError> {
        ProviderUtils::validate_api_key(&api_key, "openai")?;
        
        let base_url = base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string());
        ProviderUtils::validate_base_url(&base_url)?;

        let config = ProviderConfig {
            api_key,
            base_url,
            default_model: "gpt-3.5-turbo".to_string(),
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
        headers.insert("Authorization".to_string(), format!("Bearer {}", self.config.api_key));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        for (key, value) in &self.config.custom_headers {
            headers.insert(key.clone(), value.clone());
        }
        
        headers
    }

    fn convert_request(&self, request: ChatRequest) -> OpenAIChatRequest {
        use crate::domain::value_objects::chat_message::{MessageContent, ContentPart};
        
        let messages = request.messages
            .iter()
            .map(|msg| {
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
                                    let mut img_obj = serde_json::json!({
                                        "type": "image_url",
                                        "image_url": {
                                            "url": image_url.url
                                        }
                                    });
                                    if let Some(detail) = &image_url.detail {
                                        img_obj["image_url"]["detail"] = serde_json::json!(detail);
                                    }
                                    img_obj
                                }
                            })
                            .collect();
                        serde_json::json!(content_parts)
                    }
                };
                
                OpenAIMessage {
                    role: format!("{:?}", msg.role).to_lowercase(),
                    content,
                }
            })
            .collect();

        // Convert response_format to OpenAI format
        let response_format = request.response_format.map(|rf| {
            serde_json::json!({
                "type": rf.format_type,
                "json_schema": rf.json_schema.map(|js| serde_json::json!({
                    "name": js.name,
                    "strict": js.strict,
                    "schema": js.schema
                }))
            })
        });

        OpenAIChatRequest {
            model: request.model,
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stop: request.stop_sequences,
            stream: request.stream,
            response_format,
        }
    }

    fn convert_response(&self, response: StandardChatResponse) -> Result<ChatResponse, LLMError> {
        let choice = response.choices
            .first()
            .ok_or_else(|| LLMError::ProviderError("No choices in response".to_string()))?;

        let message = choice.message
            .as_ref()
            .ok_or_else(|| LLMError::ProviderError("No message in choice".to_string()))?;

        let content = message.content
            .as_ref()
            .ok_or_else(|| LLMError::ProviderError("No content in message".to_string()))?;

        let finish_reason = match choice.finish_reason.as_deref() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("content_filter") => FinishReason::ContentFilter,
            Some("tool_calls") => FinishReason::ToolCalls,
            _ => FinishReason::Stop,
        };

        let usage = response.usage
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            })
            .unwrap_or_else(|| TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            });

        Ok(ChatResponse {
            content: content.clone(),
            model_used: response.model,
            usage,
            finish_reason,
            metadata: None,
        })
    }

    async fn fetch_available_models(&self) -> Result<Vec<ModelInfo>, LLMError> {
        let url = format!("{}/models", self.config.base_url);
        let headers = self.build_headers();

        let response: OpenAIModelsResponse = self.http_client
            .get(&url, &headers)
            .await?;

        let mut models = Vec::new();
        for model in response.data {
            if model.id.starts_with("gpt-") {
                let supports_streaming = true;
                let supports_tools = model.id.contains("gpt-4") || model.id.contains("gpt-3.5-turbo");
                let supports_vision = model.id.contains("vision") || model.id.contains("gpt-4");
                
                let context_length = if model.id.contains("gpt-4-turbo") {
                    Some(128000)
                } else if model.id.contains("gpt-4") {
                    Some(8192)
                } else if model.id.contains("gpt-3.5-turbo") {
                    Some(4096)
                } else {
                    Some(4096)
                };

                models.push(ModelInfo {
                    id: model.id.clone(),
                    name: model.id,
                    description: Some(format!("OpenAI model owned by {}", model.owned_by)),
                    context_length,
                    supports_streaming,
                    supports_tools,
                    supports_vision,
                });
            }
        }

        if models.is_empty() {
            // Fallback to default models if API doesn't return any
            models = ProviderUtils::create_default_models("openai");
        }

        Ok(models)
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, LLMError> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let headers = self.build_headers();
        let openai_request = self.convert_request(request);
        debug!("openai_request = {:?}", openai_request);

        let response: StandardChatResponse = self.http_client
            .post_json(&url, &headers, &openai_request)
            .await?;
        debug!("response = {:?}", response);

        self.convert_response(response)
    }

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, LLMError> {
        let url = format!("{}/embeddings", self.config.base_url);
        let headers = self.build_headers();
        
        let request = OpenAIEmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: text.to_string(),
        };

        let response: OpenAIEmbeddingResponse = self.http_client
            .post_json(&url, &headers, &request)
            .await?;

        response.data
            .first()
            .map(|data| data.embedding.clone())
            .ok_or_else(|| LLMError::ProviderError("No embedding data in response".to_string()))
    }

    async fn stream_chat_completion(
        &self,
        request: ChatRequest,
    ) -> Result<Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>, LLMError> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let headers = self.build_headers();
        let mut openai_request = self.convert_request(request);
        openai_request.stream = true;

        let response = self.http_client
            .post_stream(&url, &headers, &openai_request)
            .await?;

        let byte_stream = response.bytes_stream().map(|result| {
            result.map_err(|e| LLMError::NetworkError(format!("Stream error: {}", e)))
        });

        Ok(StreamAdapter::from_bytes_stream(Box::pin(byte_stream)))
    }

    fn get_model_info(&self) -> Vec<ModelInfo> {
        ProviderUtils::create_default_models("openai")
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    async fn test_connection(&self) -> Result<ConnectionTestResult, LLMError> {
        let start_time = std::time::Instant::now();
        
        match self.fetch_available_models().await {
            Ok(models) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                Ok(ConnectionTestResult {
                    success: true,
                    response_time_ms: response_time,
                    error_message: None,
                    model_info: models.first().cloned(),
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

    fn create_test_provider() -> OpenAIProvider {
        OpenAIProvider::new(
            "sk-test1234567890".to_string(),
            Some("https://api.openai.com/v1".to_string()),
        ).unwrap()
    }

    #[test]
    fn test_provider_creation() {
        let provider = create_test_provider();
        assert_eq!(provider.config.base_url, "https://api.openai.com/v1");
        assert!(provider.config.api_key.starts_with("sk-"));
    }

    #[test]
    fn test_invalid_api_key() {
        let result = OpenAIProvider::new(
            "invalid-key".to_string(),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_request() {
        use crate::domain::value_objects::chat_message::MessageContent;
        
        let provider = create_test_provider();
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: MessageContent::Text("Hello".to_string()),
                metadata: None,
                timestamp: Utc::now(),
            },
        ];

        let request = ChatRequest {
            messages,
            model: "gpt-3.5-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: Some(1.0),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            stop_sequences: None,
            stream: false,
            tenant_id: uuid::Uuid::new_v4(),
            response_format: None,
        };

        let openai_request = provider.convert_request(request);
        assert_eq!(openai_request.model, "gpt-3.5-turbo");
        assert_eq!(openai_request.messages.len(), 1);
        assert_eq!(openai_request.messages[0].role, "user");
        assert_eq!(openai_request.messages[0].content, serde_json::json!("Hello"));
        assert_eq!(openai_request.temperature, Some(0.7));
        assert!(!openai_request.stream);
    }

    #[test]
    fn test_build_headers() {
        let provider = create_test_provider();
        let headers = provider.build_headers();
        
        assert!(headers.contains_key("Authorization"));
        assert!(headers.get("Authorization").unwrap().starts_with("Bearer sk-"));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
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
        assert!(models.iter().any(|m| m.id == "gpt-3.5-turbo"));
        assert!(models.iter().any(|m| m.supports_streaming));
    }
}