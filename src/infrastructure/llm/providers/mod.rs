pub mod openai;
pub mod claude;
pub mod local_llm;

pub use openai::OpenAIProvider;
pub use claude::ClaudeProvider;
pub use local_llm::LocalLLMProvider;

use crate::domain::services::llm_service::{LLMError, ModelInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: std::time::Duration,
    pub max_retries: u32,
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            user_agent: "agent-platform/0.1.0".to_string(),
        }
    }
}

/// Common provider configuration
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: String,
    pub default_model: String,
    pub http_config: HttpClientConfig,
    pub custom_headers: HashMap<String, String>,
}

/// Standard API response format for chat completions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<StandardUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: Option<StandardMessage>,
    pub delta: Option<StandardMessage>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<StandardToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardToolCall {
    pub id: String,
    pub r#type: String,
    pub function: StandardFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// HTTP client wrapper with common functionality
pub struct HttpClient {
    client: reqwest::Client,
    config: HttpClientConfig,
}

impl HttpClient {
    pub fn new(config: HttpClientConfig) -> Result<Self, LLMError> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| LLMError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    pub async fn post_json<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        body: &T,
    ) -> Result<R, LLMError> {
        let mut request = self.client.post(url);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request
            .json(body)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(format!("Request failed: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| LLMError::NetworkError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(crate::infrastructure::llm::ErrorMapper::map_http_error(
                status.as_u16(),
                &response_text,
            ));
        }

        serde_json::from_str(&response_text)
            .map_err(|e| LLMError::SerializationError(format!("Failed to parse response: {}", e)))
    }

    pub async fn post_stream<T: Serialize>(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        body: &T,
    ) -> Result<reqwest::Response, LLMError> {
        let mut request = self.client.post(url);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request
            .json(body)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(format!("Stream request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::infrastructure::llm::ErrorMapper::map_http_error(
                status.as_u16(),
                &error_text,
            ));
        }

        Ok(response)
    }

    pub async fn get<R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<R, LLMError> {
        let mut request = self.client.get(url);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(format!("GET request failed: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| LLMError::NetworkError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(crate::infrastructure::llm::ErrorMapper::map_http_error(
                status.as_u16(),
                &response_text,
            ));
        }

        serde_json::from_str(&response_text)
            .map_err(|e| LLMError::SerializationError(format!("Failed to parse response: {}", e)))
    }
}

/// Utility functions for provider implementations
pub struct ProviderUtils;

impl ProviderUtils {
    /// Convert domain ChatMessage to provider-specific format
    pub fn convert_messages_to_standard(
        messages: &[crate::domain::value_objects::ChatMessage],
    ) -> Vec<StandardMessage> {
        messages
            .iter()
            .map(|msg| StandardMessage {
                role: format!("{:?}", msg.role).to_lowercase(),
                content: Some(msg.content.clone()),
                tool_calls: None, // TODO: Implement tool calls conversion
            })
            .collect()
    }

    /// Create default model info for a provider
    pub fn create_default_models(provider_name: &str) -> Vec<ModelInfo> {
        match provider_name {
            "openai" => vec![
                ModelInfo {
                    id: "gpt-3.5-turbo".to_string(),
                    name: "GPT-3.5 Turbo".to_string(),
                    description: Some("Fast and efficient model for most tasks".to_string()),
                    context_length: Some(4096),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_vision: false,
                },
                ModelInfo {
                    id: "gpt-4".to_string(),
                    name: "GPT-4".to_string(),
                    description: Some("Most capable model for complex tasks".to_string()),
                    context_length: Some(8192),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_vision: false,
                },
                ModelInfo {
                    id: "gpt-4-turbo".to_string(),
                    name: "GPT-4 Turbo".to_string(),
                    description: Some("Latest GPT-4 model with improved performance".to_string()),
                    context_length: Some(128000),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_vision: true,
                },
            ],
            "claude" => vec![
                ModelInfo {
                    id: "claude-3-haiku-20240307".to_string(),
                    name: "Claude 3 Haiku".to_string(),
                    description: Some("Fast and cost-effective model".to_string()),
                    context_length: Some(200000),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_vision: true,
                },
                ModelInfo {
                    id: "claude-3-sonnet-20240229".to_string(),
                    name: "Claude 3 Sonnet".to_string(),
                    description: Some("Balanced performance and speed".to_string()),
                    context_length: Some(200000),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_vision: true,
                },
                ModelInfo {
                    id: "claude-3-opus-20240229".to_string(),
                    name: "Claude 3 Opus".to_string(),
                    description: Some("Most capable model for complex tasks".to_string()),
                    context_length: Some(200000),
                    supports_streaming: true,
                    supports_tools: true,
                    supports_vision: true,
                },
            ],
            _ => vec![
                ModelInfo {
                    id: "default".to_string(),
                    name: "Default Model".to_string(),
                    description: Some("Default model for local LLM".to_string()),
                    context_length: Some(4096),
                    supports_streaming: true,
                    supports_tools: false,
                    supports_vision: false,
                },
            ],
        }
    }

    /// Validate API key format
    pub fn validate_api_key(api_key: &str, provider: &str) -> Result<(), LLMError> {
        if api_key.trim().is_empty() {
            return Err(LLMError::InvalidConfiguration(
                "API key cannot be empty".to_string(),
            ));
        }

        match provider {
            "openai" => {
                if !api_key.starts_with("sk-") {
                    return Err(LLMError::InvalidConfiguration(
                        "OpenAI API key must start with 'sk-'".to_string(),
                    ));
                }
            }
            "claude" => {
                if !api_key.starts_with("sk-ant-") {
                    return Err(LLMError::InvalidConfiguration(
                        "Claude API key must start with 'sk-ant-'".to_string(),
                    ));
                }
            }
            _ => {
                // No specific validation for other providers
            }
        }

        Ok(())
    }

    /// Validate base URL format
    pub fn validate_base_url(url: &str) -> Result<(), LLMError> {
        if url.trim().is_empty() {
            return Err(LLMError::InvalidConfiguration(
                "Base URL cannot be empty".to_string(),
            ));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(LLMError::InvalidConfiguration(
                "Base URL must start with http:// or https://".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{ChatMessage, MessageRole};
    use chrono::Utc;

    #[test]
    fn test_convert_messages_to_standard() {
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                metadata: None,
                timestamp: Utc::now(),
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: "Hi there!".to_string(),
                metadata: None,
                timestamp: Utc::now(),
            },
        ];

        let standard_messages = ProviderUtils::convert_messages_to_standard(&messages);
        assert_eq!(standard_messages.len(), 2);
        assert_eq!(standard_messages[0].role, "user");
        assert_eq!(standard_messages[0].content, Some("Hello".to_string()));
        assert_eq!(standard_messages[1].role, "assistant");
        assert_eq!(standard_messages[1].content, Some("Hi there!".to_string()));
    }

    #[test]
    fn test_validate_openai_api_key() {
        assert!(ProviderUtils::validate_api_key("sk-1234567890", "openai").is_ok());
        assert!(ProviderUtils::validate_api_key("invalid-key", "openai").is_err());
        assert!(ProviderUtils::validate_api_key("", "openai").is_err());
    }

    #[test]
    fn test_validate_claude_api_key() {
        assert!(ProviderUtils::validate_api_key("sk-ant-1234567890", "claude").is_ok());
        assert!(ProviderUtils::validate_api_key("sk-1234567890", "claude").is_err());
        assert!(ProviderUtils::validate_api_key("", "claude").is_err());
    }

    #[test]
    fn test_validate_base_url() {
        assert!(ProviderUtils::validate_base_url("https://api.openai.com").is_ok());
        assert!(ProviderUtils::validate_base_url("http://localhost:8080").is_ok());
        assert!(ProviderUtils::validate_base_url("invalid-url").is_err());
        assert!(ProviderUtils::validate_base_url("").is_err());
    }

    #[test]
    fn test_create_default_models() {
        let openai_models = ProviderUtils::create_default_models("openai");
        assert!(!openai_models.is_empty());
        assert!(openai_models.iter().any(|m| m.id == "gpt-3.5-turbo"));

        let claude_models = ProviderUtils::create_default_models("claude");
        assert!(!claude_models.is_empty());
        assert!(claude_models.iter().any(|m| m.id.contains("claude-3")));
    }
}