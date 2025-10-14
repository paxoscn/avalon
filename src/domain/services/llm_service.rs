use crate::domain::value_objects::{ModelConfig, ChatMessage, MessageMetadata};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// LLM domain service interface defining business rules and operations
#[async_trait]
pub trait LLMDomainService: Send + Sync {
    /// Generate chat completion using the specified model configuration
    async fn chat_completion(
        &self,
        config: &ModelConfig,
        messages: Vec<ChatMessage>,
        tenant_id: Uuid,
    ) -> Result<ChatResponse, LLMError>;

    /// Generate embeddings for the given text
    async fn generate_embedding(
        &self,
        config: &ModelConfig,
        text: &str,
        tenant_id: Uuid,
    ) -> Result<Vec<f32>, LLMError>;

    /// Stream chat completion responses
    async fn stream_chat_completion(
        &self,
        config: &ModelConfig,
        messages: Vec<ChatMessage>,
        tenant_id: Uuid,
    ) -> Result<Box<dyn futures::Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>, LLMError>;

    /// Validate model configuration
    fn validate_config(&self, config: &ModelConfig) -> Result<ValidationResult, LLMError>;

    /// Check if a model configuration supports streaming
    fn supports_streaming(&self, config: &ModelConfig) -> bool;

    /// Get available models for a provider
    async fn get_available_models(&self, provider: &str) -> Result<Vec<ModelInfo>, LLMError>;

    /// Test connection to a model provider
    async fn test_connection(&self, config: &ModelConfig) -> Result<ConnectionTestResult, LLMError>;

    /// Calculate token count for messages (estimation)
    fn estimate_token_count(&self, messages: &[ChatMessage], model: &str) -> Result<u32, LLMError>;
}

/// Response from chat completion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model_used: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
    pub metadata: Option<MessageMetadata>,
}

/// Streaming chunk for chat completion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatStreamChunk {
    pub content: Option<String>,
    pub finish_reason: Option<FinishReason>,
    pub usage: Option<TokenUsage>,
}

/// Token usage information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Reason why the completion finished
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    Error,
}

/// Model information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub context_length: Option<u32>,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
}

/// Configuration validation result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Connection test result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
    pub model_info: Option<ModelInfo>,
}

/// LLM domain errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum LLMError {
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Content filtered: {0}")]
    ContentFiltered(String),

    #[error("Token limit exceeded: {0}")]
    TokenLimitExceeded(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Streaming not supported for model: {0}")]
    StreamingNotSupported(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// LLM domain service implementation
pub struct LLMDomainServiceImpl {
    // This will be injected with the infrastructure layer providers
    providers: HashMap<String, Box<dyn LLMProvider>>,
}

/// Provider trait that will be implemented in infrastructure layer
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, LLMError>;
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, LLMError>;
    async fn stream_chat_completion(&self, request: ChatRequest) -> Result<Box<dyn futures::Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>, LLMError>;
    fn get_model_info(&self) -> Vec<ModelInfo>;
    fn supports_streaming(&self) -> bool;
    async fn test_connection(&self) -> Result<ConnectionTestResult, LLMError>;
}

/// Request structure for chat completion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub stream: bool,
    pub tenant_id: Uuid,
}

impl LLMDomainServiceImpl {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register_provider(&mut self, provider_name: String, provider: Box<dyn LLMProvider>) {
        self.providers.insert(provider_name, provider);
    }

    fn get_provider(&self, provider_name: &str) -> Result<&dyn LLMProvider, LLMError> {
        self.providers
            .get(provider_name)
            .map(|p| p.as_ref())
            .ok_or_else(|| LLMError::ProviderError(format!("Provider '{}' not found", provider_name)))
    }

    fn build_chat_request(&self, config: &ModelConfig, messages: Vec<ChatMessage>, tenant_id: Uuid, stream: bool) -> ChatRequest {
        ChatRequest {
            messages,
            model: config.model_name.clone(),
            temperature: config.parameters.temperature,
            max_tokens: config.parameters.max_tokens,
            top_p: config.parameters.top_p,
            frequency_penalty: config.parameters.frequency_penalty,
            presence_penalty: config.parameters.presence_penalty,
            stop_sequences: config.parameters.stop_sequences.clone(),
            stream,
            tenant_id,
        }
    }
}

#[async_trait]
impl LLMDomainService for LLMDomainServiceImpl {
    async fn chat_completion(
        &self,
        config: &ModelConfig,
        messages: Vec<ChatMessage>,
        tenant_id: Uuid,
    ) -> Result<ChatResponse, LLMError> {
        // Validate configuration first
        let validation = self.validate_config(config)?;
        if !validation.is_valid {
            return Err(LLMError::InvalidConfiguration(validation.errors.join(", ")));
        }

        // Validate messages
        for message in &messages {
            message.validate().map_err(|e| LLMError::InvalidConfiguration(e))?;
        }

        let provider_name = format!("{:?}", config.provider).to_lowercase();
        let provider = self.get_provider(&provider_name)?;
        
        let request = self.build_chat_request(config, messages, tenant_id, false);
        provider.chat_completion(request).await
    }

    async fn generate_embedding(
        &self,
        config: &ModelConfig,
        text: &str,
        tenant_id: Uuid,
    ) -> Result<Vec<f32>, LLMError> {
        if text.trim().is_empty() {
            return Err(LLMError::InvalidConfiguration("Text cannot be empty".to_string()));
        }

        let provider_name = format!("{:?}", config.provider).to_lowercase();
        let provider = self.get_provider(&provider_name)?;
        
        provider.generate_embedding(text).await
    }

    async fn stream_chat_completion(
        &self,
        config: &ModelConfig,
        messages: Vec<ChatMessage>,
        tenant_id: Uuid,
    ) -> Result<Box<dyn futures::Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>, LLMError> {
        if !self.supports_streaming(config) {
            return Err(LLMError::StreamingNotSupported(config.model_name.clone()));
        }

        // Validate configuration and messages
        let validation = self.validate_config(config)?;
        if !validation.is_valid {
            return Err(LLMError::InvalidConfiguration(validation.errors.join(", ")));
        }

        for message in &messages {
            message.validate().map_err(|e| LLMError::InvalidConfiguration(e))?;
        }

        let provider_name = format!("{:?}", config.provider).to_lowercase();
        let provider = self.get_provider(&provider_name)?;
        
        let request = self.build_chat_request(config, messages, tenant_id, true);
        provider.stream_chat_completion(request).await
    }

    fn validate_config(&self, config: &ModelConfig) -> Result<ValidationResult, LLMError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate using the existing method
        if let Err(e) = config.validate() {
            errors.push(e);
        }

        // Additional domain-specific validations
        if config.model_name.trim().is_empty() {
            errors.push("Model name is required".to_string());
        }

        // Check for potential issues
        if let Some(temp) = config.parameters.temperature {
            if temp > 1.5 {
                warnings.push("High temperature may produce unpredictable results".to_string());
            }
        }

        if let Some(max_tokens) = config.parameters.max_tokens {
            if max_tokens > 4000 {
                warnings.push("High token count may be expensive".to_string());
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    fn supports_streaming(&self, config: &ModelConfig) -> bool {
        config.supports_streaming()
    }

    async fn get_available_models(&self, provider: &str) -> Result<Vec<ModelInfo>, LLMError> {
        let provider = self.get_provider(provider)?;
        Ok(provider.get_model_info())
    }

    async fn test_connection(&self, config: &ModelConfig) -> Result<ConnectionTestResult, LLMError> {
        let provider_name = format!("{:?}", config.provider).to_lowercase();
        let provider = self.get_provider(&provider_name)?;
        provider.test_connection().await
    }

    fn estimate_token_count(&self, messages: &[ChatMessage], model: &str) -> Result<u32, LLMError> {
        // Simple estimation based on character count
        // This is a rough approximation - real implementations would use proper tokenizers
        let total_chars: usize = messages.iter()
            .map(|m| m.content.len())
            .sum();
        
        // Rough estimation: 1 token ≈ 4 characters for English text
        let estimated_tokens = (total_chars as f32 / 4.0).ceil() as u32;
        
        // Add some overhead for message formatting and system prompts
        let overhead = messages.len() as u32 * 10;
        
        Ok(estimated_tokens + overhead)
    }
}

impl Default for LLMDomainServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_errors(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn with_warnings(warnings: Vec<String>) -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings,
        }
    }
}

impl TokenUsage {
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{ModelProvider, ModelParameters, ModelCredentials};

    fn create_test_config() -> ModelConfig {
        ModelConfig {
            provider: ModelProvider::OpenAI,
            model_name: "gpt-3.5-turbo".to_string(),
            parameters: ModelParameters::default(),
            credentials: ModelCredentials::default(),
        }
    }

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_with_errors() {
        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let result = ValidationResult::with_errors(errors.clone());
        assert!(!result.is_valid);
        assert_eq!(result.errors, errors);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_token_usage_calculation() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[tokio::test]
    async fn test_llm_service_validate_config() {
        let service = LLMDomainServiceImpl::new();
        let config = create_test_config();
        
        let result = service.validate_config(&config).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_supports_streaming() {
        let service = LLMDomainServiceImpl::new();
        let config = create_test_config();
        
        assert!(service.supports_streaming(&config));
    }

    #[test]
    fn test_estimate_token_count() {
        let service = LLMDomainServiceImpl::new();
        let messages = vec![
            ChatMessage::new_user_message("Hello, how are you?".to_string()),
            ChatMessage::new_assistant_message("I'm doing well, thank you!".to_string()),
        ];
        
        let count = service.estimate_token_count(&messages, "gpt-3.5-turbo").unwrap();
        assert!(count > 0);
    }
}