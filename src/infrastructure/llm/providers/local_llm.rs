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


/// Local LLM provider implementation (compatible with OpenAI-like APIs)
/// This can work with Ollama, LocalAI, vLLM, or any OpenAI-compatible local server
pub struct LocalLLMProvider {
    config: ProviderConfig,
    http_client: HttpClient,
}

#[derive(Debug, Serialize)]
struct LocalLLMChatRequest {
    model: String,
    messages: Vec<LocalLLMMessage>,
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
}

#[derive(Debug, Serialize, Deserialize)]
struct LocalLLMMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct LocalLLMModelsResponse {
    #[serde(default)]
    data: Vec<LocalLLMModelInfo>,
    #[serde(default)]
    models: Vec<LocalLLMModelInfo>, // Some local APIs use "models" instead of "data"
}

#[derive(Debug, Deserialize)]
struct LocalLLMModelInfo {
    id: String,
    #[serde(default)]
    object: String,
    #[serde(default)]
    created: u64,
    #[serde(default)]
    owned_by: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct LocalLLMEmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Debug, Deserialize)]
struct LocalLLMEmbeddingResponse {
    data: Vec<LocalLLMEmbeddingData>,
    #[serde(default)]
    usage: Option<LocalLLMEmbeddingUsage>,
}

#[derive(Debug, Deserialize)]
struct LocalLLMEmbeddingData {
    embedding: Vec<f32>,
    #[serde(default)]
    index: u32,
}

#[derive(Debug, Deserialize)]
struct LocalLLMEmbeddingUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct HealthResponse {
    #[serde(default)]
    status: String,
    #[serde(default)]
    message: String,
}

impl LocalLLMProvider {
    pub fn new(base_url: String) -> Result<Self, LLMError> {
        ProviderUtils::validate_base_url(&base_url)?;

        let config = ProviderConfig {
            api_key: "local".to_string(), // Local LLMs often don't require API keys
            base_url,
            default_model: "default".to_string(),
            http_config: HttpClientConfig {
                timeout: std::time::Duration::from_secs(60), // Local LLMs might be slower
                ..Default::default()
            },
            custom_headers: HashMap::new(),
        };

        let http_client = HttpClient::new(config.http_config.clone())?;

        Ok(Self {
            config,
            http_client,
        })
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.config.api_key = api_key;
        self
    }

    pub fn with_default_model(mut self, model: String) -> Self {
        self.config.default_model = model;
        self
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
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        // Only add authorization header if API key is not "local"
        if self.config.api_key != "local" && !self.config.api_key.is_empty() {
            headers.insert("Authorization".to_string(), format!("Bearer {}", self.config.api_key));
        }
        
        for (key, value) in &self.config.custom_headers {
            headers.insert(key.clone(), value.clone());
        }
        
        headers
    }

    fn convert_request(&self, request: ChatRequest) -> LocalLLMChatRequest {
        let messages = request.messages
            .iter()
            .map(|msg| LocalLLMMessage {
                role: format!("{:?}", msg.role).to_lowercase(),
                content: msg.content.clone(),
            })
            .collect();

        LocalLLMChatRequest {
            model: request.model,
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            frequency_penalty: request.frequency_penalty,
            presence_penalty: request.presence_penalty,
            stop: request.stop_sequences,
            stream: request.stream,
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
        // Try different common endpoints for model listing
        let endpoints = vec![
            format!("{}/v1/models", self.config.base_url),
            format!("{}/models", self.config.base_url),
            format!("{}/api/tags", self.config.base_url), // Ollama endpoint
        ];

        let headers = self.build_headers();

        for endpoint in endpoints {
            match self.try_fetch_models(&endpoint, &headers).await {
                Ok(models) if !models.is_empty() => return Ok(models),
                Ok(_) => continue, // Empty response, try next endpoint
                Err(_) => continue, // Error, try next endpoint
            }
        }

        // If all endpoints fail, return default models
        Ok(ProviderUtils::create_default_models("local_llm"))
    }

    async fn try_fetch_models(&self, url: &str, headers: &HashMap<String, String>) -> Result<Vec<ModelInfo>, LLMError> {
        let response: LocalLLMModelsResponse = self.http_client
            .get(url, headers)
            .await?;

        let mut models = Vec::new();
        let model_list = if !response.data.is_empty() {
            response.data
        } else {
            response.models
        };

        for model in model_list {
            models.push(ModelInfo {
                id: model.id.clone(),
                name: model.name.unwrap_or_else(|| model.id.clone()),
                description: Some(format!("Local LLM model owned by {}", model.owned_by)),
                context_length: Some(4096), // Default context length
                supports_streaming: true,
                supports_tools: false, // Most local LLMs don't support tools yet
                supports_vision: false, // Most local LLMs don't support vision yet
            });
        }

        Ok(models)
    }

    async fn check_health(&self) -> Result<(), LLMError> {
        let health_endpoints = vec![
            format!("{}/health", self.config.base_url),
            format!("{}/v1/health", self.config.base_url),
            format!("{}/api/version", self.config.base_url), // Ollama endpoint
        ];

        let headers = self.build_headers();

        for endpoint in health_endpoints {
            if let Ok(_) = self.http_client.get::<HealthResponse>(&endpoint, &headers).await {
                return Ok(());
            }
        }

        // If health endpoints fail, try a simple model list request
        self.fetch_available_models().await.map(|_| ())
    }
}

#[async_trait]
impl LLMProvider for LocalLLMProvider {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, LLMError> {
        let url = format!("{}/v1/chat/completions", self.config.base_url);
        let headers = self.build_headers();
        let local_request = self.convert_request(request);

        let response: StandardChatResponse = self.http_client
            .post_json(&url, &headers, &local_request)
            .await?;

        self.convert_response(response)
    }

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, LLMError> {
        let url = format!("{}/v1/embeddings", self.config.base_url);
        let headers = self.build_headers();
        
        let request = LocalLLMEmbeddingRequest {
            model: self.config.default_model.clone(),
            input: text.to_string(),
        };

        let response: LocalLLMEmbeddingResponse = self.http_client
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
        let url = format!("{}/v1/chat/completions", self.config.base_url);
        let headers = self.build_headers();
        let mut local_request = self.convert_request(request);
        local_request.stream = true;

        let response = self.http_client
            .post_stream(&url, &headers, &local_request)
            .await?;

        let byte_stream = response.bytes_stream().map(|result| {
            result.map_err(|e| LLMError::NetworkError(format!("Stream error: {}", e)))
        });

        Ok(StreamAdapter::from_bytes_stream(Box::pin(byte_stream)))
    }

    fn get_model_info(&self) -> Vec<ModelInfo> {
        ProviderUtils::create_default_models("local_llm")
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    async fn test_connection(&self) -> Result<ConnectionTestResult, LLMError> {
        let start_time = std::time::Instant::now();
        
        match self.check_health().await {
            Ok(()) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                Ok(ConnectionTestResult {
                    success: true,
                    response_time_ms: response_time,
                    error_message: None,
                    model_info: Some(ModelInfo {
                        id: self.config.default_model.clone(),
                        name: "Local LLM".to_string(),
                        description: Some("Local LLM server".to_string()),
                        context_length: Some(4096),
                        supports_streaming: true,
                        supports_tools: false,
                        supports_vision: false,
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

    fn create_test_provider() -> LocalLLMProvider {
        LocalLLMProvider::new("http://localhost:11434".to_string()).unwrap()
    }

    #[test]
    fn test_provider_creation() {
        let provider = create_test_provider();
        assert_eq!(provider.config.base_url, "http://localhost:11434");
        assert_eq!(provider.config.api_key, "local");
    }

    #[test]
    fn test_with_api_key() {
        let provider = create_test_provider()
            .with_api_key("test-key".to_string());
        assert_eq!(provider.config.api_key, "test-key");
    }

    #[test]
    fn test_with_default_model() {
        let provider = create_test_provider()
            .with_default_model("llama2".to_string());
        assert_eq!(provider.config.default_model, "llama2");
    }

    #[test]
    fn test_invalid_base_url() {
        let result = LocalLLMProvider::new("invalid-url".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_request() {
        let provider = create_test_provider();
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                metadata: None,
                timestamp: Utc::now(),
            },
        ];

        let request = ChatRequest {
            messages,
            model: "llama2".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: Some(1.0),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            stop_sequences: None,
            stream: false,
            tenant_id: uuid::Uuid::new_v4(),
        };

        let local_request = provider.convert_request(request);
        assert_eq!(local_request.model, "llama2");
        assert_eq!(local_request.messages.len(), 1);
        assert_eq!(local_request.messages[0].role, "user");
        assert_eq!(local_request.messages[0].content, "Hello");
        assert_eq!(local_request.temperature, Some(0.7));
        assert!(!local_request.stream);
    }

    #[test]
    fn test_build_headers_without_api_key() {
        let provider = create_test_provider();
        let headers = provider.build_headers();
        
        assert!(!headers.contains_key("Authorization"));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_build_headers_with_api_key() {
        let provider = create_test_provider()
            .with_api_key("test-key".to_string());
        let headers = provider.build_headers();
        
        assert!(headers.contains_key("Authorization"));
        assert_eq!(headers.get("Authorization"), Some(&"Bearer test-key".to_string()));
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
        assert!(models.iter().any(|m| m.id == "default"));
        assert!(models.iter().any(|m| m.supports_streaming));
    }
}