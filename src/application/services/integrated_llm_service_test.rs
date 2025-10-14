use super::integrated_llm_service::*;
use crate::domain::services::llm_service::{
    LLMProvider, LLMError, ChatRequest, ChatResponse, ChatStreamChunk, ModelInfo, 
    ConnectionTestResult, TokenUsage, FinishReason, LLMDomainService
};
use crate::domain::value_objects::{
    ModelConfig, ModelProvider, ModelParameters, ModelCredentials, ChatMessage
};
use crate::infrastructure::llm::LLMProviderRegistry;
use async_trait::async_trait;
use futures::{Stream, stream};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Mock LLM Provider for testing
#[derive(Clone)]
pub struct MockLLMProvider {
    pub name: String,
    pub should_fail: Arc<Mutex<bool>>,
    pub response_delay: Duration,
    pub call_count: Arc<Mutex<u32>>,
}

impl MockLLMProvider {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_fail: Arc::new(Mutex::new(false)),
            response_delay: Duration::from_millis(100),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.response_delay = delay;
        self
    }

    pub fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }

    pub fn get_call_count(&self) -> u32 {
        *self.call_count.lock().unwrap()
    }

    pub fn reset_call_count(&self) {
        *self.call_count.lock().unwrap() = 0;
    }
}

#[async_trait]
impl LLMProvider for MockLLMProvider {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, LLMError> {
        // Increment call count
        *self.call_count.lock().unwrap() += 1;

        // Simulate response delay
        if self.response_delay > Duration::from_millis(0) {
            sleep(self.response_delay).await;
        }

        // Check if should fail
        if *self.should_fail.lock().unwrap() {
            return Err(LLMError::ProviderError(format!("Mock provider {} failed", self.name)));
        }

        Ok(ChatResponse {
            content: format!("Response from {} for: {}", self.name, 
                request.messages.first().map(|m| &m.content).unwrap_or(&"".to_string())),
            model_used: request.model,
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
            finish_reason: FinishReason::Stop,
            metadata: None,
        })
    }

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, LLMError> {
        *self.call_count.lock().unwrap() += 1;

        if *self.should_fail.lock().unwrap() {
            return Err(LLMError::ProviderError(format!("Mock provider {} failed", self.name)));
        }

        // Return a simple mock embedding
        Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5])
    }

    async fn stream_chat_completion(
        &self,
        request: ChatRequest,
    ) -> Result<Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>, LLMError> {
        *self.call_count.lock().unwrap() += 1;

        if *self.should_fail.lock().unwrap() {
            return Err(LLMError::ProviderError(format!("Mock provider {} failed", self.name)));
        }

        let chunks = vec![
            Ok(ChatStreamChunk {
                content: Some("Hello".to_string()),
                finish_reason: None,
                usage: None,
            }),
            Ok(ChatStreamChunk {
                content: Some(" from ".to_string()),
                finish_reason: None,
                usage: None,
            }),
            Ok(ChatStreamChunk {
                content: Some(self.name.clone()),
                finish_reason: Some(FinishReason::Stop),
                usage: Some(TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 15,
                    total_tokens: 25,
                }),
            }),
        ];

        Ok(Box::new(stream::iter(chunks)))
    }

    fn get_model_info(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: format!("{}-model-1", self.name),
                name: format!("{} Model 1", self.name),
                description: Some(format!("Test model from {}", self.name)),
                context_length: Some(4096),
                supports_streaming: true,
                supports_tools: true,
                supports_vision: false,
            }
        ]
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    async fn test_connection(&self) -> Result<ConnectionTestResult, LLMError> {
        *self.call_count.lock().unwrap() += 1;

        if *self.should_fail.lock().unwrap() {
            return Ok(ConnectionTestResult {
                success: false,
                response_time_ms: 100,
                error_message: Some(format!("Mock provider {} connection failed", self.name)),
                model_info: None,
            });
        }

        Ok(ConnectionTestResult {
            success: true,
            response_time_ms: 50,
            error_message: None,
            model_info: Some(ModelInfo {
                id: format!("{}-model-1", self.name),
                name: format!("{} Model 1", self.name),
                description: Some(format!("Test model from {}", self.name)),
                context_length: Some(4096),
                supports_streaming: true,
                supports_tools: true,
                supports_vision: false,
            }),
        })
    }
}

fn create_test_model_config() -> ModelConfig {
    ModelConfig {
        provider: ModelProvider::OpenAI,
        model_name: "gpt-3.5-turbo".to_string(),
        parameters: ModelParameters::default(),
        credentials: ModelCredentials::default(),
    }
}

fn create_test_messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage::new_user_message("Hello, how are you?".to_string()),
    ]
}

async fn create_test_service_with_providers() -> IntegratedLLMService {
    let mut registry = LLMProviderRegistry::new();
    
    // Add mock providers
    let openai_provider = Arc::new(MockLLMProvider::new("openai"));
    let claude_provider = Arc::new(MockLLMProvider::new("claude"));
    let local_provider = Arc::new(MockLLMProvider::new("local"));
    
    registry.register_provider("openai".to_string(), openai_provider);
    registry.register_provider("claude".to_string(), claude_provider);
    registry.register_provider("local".to_string(), local_provider);
    
    let config = IntegratedLLMConfig::default();
    let service = IntegratedLLMService::new(Arc::new(registry), config);
    
    // Initialize health monitoring
    service.initialize_health_monitoring().await;
    
    service
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_integrated_service_creation() {
        let service = create_test_service_with_providers().await;
        let health_status = service.get_provider_health_status().await;
        
        assert_eq!(health_status.len(), 3);
        assert!(health_status.contains_key("openai"));
        assert!(health_status.contains_key("claude"));
        assert!(health_status.contains_key("local"));
    }

    #[tokio::test]
    async fn test_chat_completion_success() {
        let service = create_test_service_with_providers().await;
        let config = create_test_model_config();
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        let result = service.chat_completion(&config, messages, tenant_id).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.content.contains("openai"));
        assert_eq!(response.usage.total_tokens, 30);
    }

    #[tokio::test]
    async fn test_chat_completion_with_fallback() {
        let service = create_test_service_with_providers().await;
        
        // Make the primary provider (openai) fail
        service.set_provider_health("openai", false).await;
        
        let config = create_test_model_config();
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        let result = service.chat_completion(&config, messages, tenant_id).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        // Should use fallback provider (claude)
        assert!(response.content.contains("claude"));
    }

    #[tokio::test]
    async fn test_load_balancing_round_robin() {
        let mut registry = LLMProviderRegistry::new();
        
        let provider1 = Arc::new(MockLLMProvider::new("provider1"));
        let provider2 = Arc::new(MockLLMProvider::new("provider2"));
        
        registry.register_provider("provider1".to_string(), provider1.clone());
        registry.register_provider("provider2".to_string(), provider2.clone());
        
        let mut config = IntegratedLLMConfig::default();
        config.load_balancing_strategy = LoadBalancingStrategy::RoundRobin;
        
        let service = IntegratedLLMService::new(Arc::new(registry), config);
        service.initialize_health_monitoring().await;
        
        let model_config = ModelConfig {
            provider: ModelProvider::OpenAI, // This provider doesn't exist, so it should round-robin
            model_name: "test-model".to_string(),
            parameters: ModelParameters::default(),
            credentials: ModelCredentials::default(),
        };
        
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        // Make multiple requests to test round-robin
        for _ in 0..4 {
            let _ = service.chat_completion(&model_config, messages.clone(), tenant_id).await;
        }

        // Both providers should have been called
        let call_count1 = provider1.get_call_count();
        let call_count2 = provider2.get_call_count();
        
        assert!(call_count1 > 0);
        assert!(call_count2 > 0);
        assert_eq!(call_count1 + call_count2, 4);
    }

    #[tokio::test]
    async fn test_health_based_load_balancing() {
        let mut registry = LLMProviderRegistry::new();
        
        let fast_provider = Arc::new(MockLLMProvider::new("fast").with_delay(Duration::from_millis(10)));
        let slow_provider = Arc::new(MockLLMProvider::new("slow").with_delay(Duration::from_millis(100)));
        
        registry.register_provider("fast".to_string(), fast_provider.clone());
        registry.register_provider("slow".to_string(), slow_provider.clone());
        
        let mut config = IntegratedLLMConfig::default();
        config.load_balancing_strategy = LoadBalancingStrategy::HealthBased;
        
        let service = IntegratedLLMService::new(Arc::new(registry), config);
        service.initialize_health_monitoring().await;
        
        // Run health checks to establish baseline
        service.run_health_checks().await;
        
        let model_config = ModelConfig {
            provider: ModelProvider::OpenAI, // Non-existent provider to trigger load balancing
            model_name: "test-model".to_string(),
            parameters: ModelParameters::default(),
            credentials: ModelCredentials::default(),
        };
        
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        // Make multiple requests
        for _ in 0..5 {
            let _ = service.chat_completion(&model_config, messages.clone(), tenant_id).await;
        }

        // The fast provider should be preferred
        let fast_calls = fast_provider.get_call_count();
        let slow_calls = slow_provider.get_call_count();
        
        assert!(fast_calls >= slow_calls);
    }

    #[tokio::test]
    async fn test_circuit_breaker_functionality() {
        let mut registry = LLMProviderRegistry::new();
        
        let failing_provider = Arc::new(MockLLMProvider::new("failing"));
        failing_provider.set_should_fail(true);
        
        registry.register_provider("failing".to_string(), failing_provider.clone());
        
        let mut config = IntegratedLLMConfig::default();
        config.circuit_breaker_threshold = 2; // Low threshold for testing
        config.enable_fallback = false; // Disable fallback to test circuit breaker
        
        let service = IntegratedLLMService::new(Arc::new(registry), config);
        service.initialize_health_monitoring().await;
        
        let model_config = ModelConfig {
            provider: ModelProvider::OpenAI,
            model_name: "test-model".to_string(),
            parameters: ModelParameters::default(),
            credentials: ModelCredentials::default(),
        };
        
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        // Make requests that should trigger circuit breaker
        for _ in 0..5 {
            let result = service.chat_completion(&model_config, messages.clone(), tenant_id).await;
            assert!(result.is_err());
        }

        // Provider should have been called initially but then circuit breaker should kick in
        let call_count = failing_provider.get_call_count();
        assert!(call_count < 5); // Circuit breaker should have prevented some calls
    }

    #[tokio::test]
    async fn test_generate_embedding() {
        let service = create_test_service_with_providers().await;
        let config = create_test_model_config();
        let tenant_id = Uuid::new_v4();

        let result = service.generate_embedding(&config, "test text", tenant_id).await;
        
        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 5);
        assert_eq!(embedding[0], 0.1);
    }

    #[tokio::test]
    async fn test_stream_chat_completion() {
        let service = create_test_service_with_providers().await;
        let config = create_test_model_config();
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        let result = service.stream_chat_completion(&config, messages, tenant_id).await;
        
        assert!(result.is_ok());
        let mut stream = result.unwrap();
        
        let mut chunks = Vec::new();
        while let Some(chunk_result) = stream.next().await {
            assert!(chunk_result.is_ok());
            chunks.push(chunk_result.unwrap());
        }
        
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].content, Some("Hello".to_string()));
        assert_eq!(chunks[1].content, Some(" from ".to_string()));
        assert!(chunks[2].content.as_ref().unwrap().contains("openai"));
        assert_eq!(chunks[2].finish_reason, Some(FinishReason::Stop));
    }

    #[tokio::test]
    async fn test_validate_config() {
        let service = create_test_service_with_providers().await;
        let config = create_test_model_config();
        
        let result = service.validate_config(&config);
        assert!(result.is_ok());
        
        let validation = result.unwrap();
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());
    }

    #[tokio::test]
    async fn test_supports_streaming() {
        let service = create_test_service_with_providers().await;
        let config = create_test_model_config();
        
        assert!(service.supports_streaming(&config));
    }

    #[tokio::test]
    async fn test_get_available_models() {
        let service = create_test_service_with_providers().await;
        
        let result = service.get_available_models("openai").await;
        assert!(result.is_ok());
        
        let models = result.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "openai-model-1");
        assert!(models[0].supports_streaming);
    }

    #[tokio::test]
    async fn test_test_connection() {
        let service = create_test_service_with_providers().await;
        let config = create_test_model_config();
        
        let result = service.test_connection(&config).await;
        assert!(result.is_ok());
        
        let connection_result = result.unwrap();
        assert!(connection_result.success);
        assert_eq!(connection_result.response_time_ms, 50);
        assert!(connection_result.model_info.is_some());
    }

    #[tokio::test]
    async fn test_estimate_token_count() {
        let service = create_test_service_with_providers().await;
        let messages = vec![
            ChatMessage::new_user_message("Hello, how are you today?".to_string()),
            ChatMessage::new_assistant_message("I'm doing well, thank you for asking!".to_string()),
        ];
        
        let result = service.estimate_token_count(&messages, "gpt-3.5-turbo");
        assert!(result.is_ok());
        
        let token_count = result.unwrap();
        assert!(token_count > 0);
        assert!(token_count < 100); // Should be reasonable for short messages
    }

    #[tokio::test]
    async fn test_health_check_updates() {
        let service = create_test_service_with_providers().await;
        
        // Run health checks
        service.run_health_checks().await;
        
        let health_status = service.get_provider_health_status().await;
        
        // All providers should be healthy initially
        for (_, health) in health_status {
            assert!(health.is_healthy);
            assert!(health.success_count > 0);
        }
    }

    #[tokio::test]
    async fn test_weighted_random_load_balancing() {
        let mut registry = LLMProviderRegistry::new();
        
        let provider1 = Arc::new(MockLLMProvider::new("provider1"));
        let provider2 = Arc::new(MockLLMProvider::new("provider2"));
        
        registry.register_provider("provider1".to_string(), provider1.clone());
        registry.register_provider("provider2".to_string(), provider2.clone());
        
        let mut weights = HashMap::new();
        weights.insert("provider1".to_string(), 3.0); // Higher weight
        weights.insert("provider2".to_string(), 1.0);
        
        let mut config = IntegratedLLMConfig::default();
        config.load_balancing_strategy = LoadBalancingStrategy::WeightedRandom(weights);
        
        let service = IntegratedLLMService::new(Arc::new(registry), config);
        service.initialize_health_monitoring().await;
        
        let model_config = ModelConfig {
            provider: ModelProvider::OpenAI, // Non-existent to trigger load balancing
            model_name: "test-model".to_string(),
            parameters: ModelParameters::default(),
            credentials: ModelCredentials::default(),
        };
        
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        // Make many requests to test weighted distribution
        for _ in 0..20 {
            let _ = service.chat_completion(&model_config, messages.clone(), tenant_id).await;
        }

        let calls1 = provider1.get_call_count();
        let calls2 = provider2.get_call_count();
        
        // Provider1 should have more calls due to higher weight
        assert!(calls1 > calls2);
        assert_eq!(calls1 + calls2, 20);
    }

    #[tokio::test]
    async fn test_response_time_based_load_balancing() {
        let mut registry = LLMProviderRegistry::new();
        
        let fast_provider = Arc::new(MockLLMProvider::new("fast").with_delay(Duration::from_millis(10)));
        let slow_provider = Arc::new(MockLLMProvider::new("slow").with_delay(Duration::from_millis(200)));
        
        registry.register_provider("fast".to_string(), fast_provider.clone());
        registry.register_provider("slow".to_string(), slow_provider.clone());
        
        let mut config = IntegratedLLMConfig::default();
        config.load_balancing_strategy = LoadBalancingStrategy::ResponseTimeBased;
        
        let service = IntegratedLLMService::new(Arc::new(registry), config);
        service.initialize_health_monitoring().await;
        
        // Run health checks to establish response times
        service.run_health_checks().await;
        
        let model_config = ModelConfig {
            provider: ModelProvider::OpenAI, // Non-existent to trigger load balancing
            model_name: "test-model".to_string(),
            parameters: ModelParameters::default(),
            credentials: ModelCredentials::default(),
        };
        
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        // Make multiple requests
        for _ in 0..10 {
            let _ = service.chat_completion(&model_config, messages.clone(), tenant_id).await;
        }

        let fast_calls = fast_provider.get_call_count();
        let slow_calls = slow_provider.get_call_count();
        
        // Fast provider should be heavily preferred
        assert!(fast_calls >= slow_calls);
    }

    #[tokio::test]
    async fn test_all_providers_unhealthy() {
        let service = create_test_service_with_providers().await;
        
        // Mark all providers as unhealthy
        service.set_provider_health("openai", false).await;
        service.set_provider_health("claude", false).await;
        service.set_provider_health("local", false).await;
        
        let config = create_test_model_config();
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        let result = service.chat_completion(&config, messages, tenant_id).await;
        
        // Should fail when no providers are healthy
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No healthy providers"));
    }
}

/// Integration tests that test the service with real-like scenarios
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_concurrent_requests() {
        let service = Arc::new(create_test_service_with_providers().await);
        let config = create_test_model_config();
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        let mut handles = Vec::new();
        
        // Spawn multiple concurrent requests
        for i in 0..10 {
            let service_clone = service.clone();
            let config_clone = config.clone();
            let messages_clone = messages.clone();
            
            let handle = tokio::spawn(async move {
                service_clone.chat_completion(&config_clone, messages_clone, tenant_id).await
            });
            
            handles.push(handle);
        }

        // Wait for all requests to complete
        let mut success_count = 0;
        for handle in handles {
            if let Ok(result) = handle.await {
                if result.is_ok() {
                    success_count += 1;
                }
            }
        }

        // All requests should succeed
        assert_eq!(success_count, 10);
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let mut registry = LLMProviderRegistry::new();
        
        // Create a provider with very long delay
        let slow_provider = Arc::new(MockLLMProvider::new("slow").with_delay(Duration::from_secs(5)));
        registry.register_provider("slow".to_string(), slow_provider);
        
        let mut config = IntegratedLLMConfig::default();
        config.request_timeout = Duration::from_millis(100); // Very short timeout
        
        let service = IntegratedLLMService::new(Arc::new(registry), config);
        service.initialize_health_monitoring().await;
        
        let model_config = create_test_model_config();
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        // Request should timeout quickly
        let result = timeout(
            Duration::from_millis(500),
            service.chat_completion(&model_config, messages, tenant_id)
        ).await;

        assert!(result.is_ok()); // Timeout should complete
        // The actual request might succeed or fail depending on timing
    }

    #[tokio::test]
    async fn test_provider_recovery() {
        let mut registry = LLMProviderRegistry::new();
        
        let provider = Arc::new(MockLLMProvider::new("recovery_test"));
        registry.register_provider("recovery_test".to_string(), provider.clone());
        
        let service = IntegratedLLMService::new(Arc::new(registry), IntegratedLLMConfig::default());
        service.initialize_health_monitoring().await;
        
        let model_config = ModelConfig {
            provider: ModelProvider::OpenAI,
            model_name: "recovery_test".to_string(),
            parameters: ModelParameters::default(),
            credentials: ModelCredentials::default(),
        };
        
        let messages = create_test_messages();
        let tenant_id = Uuid::new_v4();

        // First, make provider fail
        provider.set_should_fail(true);
        let result1 = service.chat_completion(&model_config, messages.clone(), tenant_id).await;
        assert!(result1.is_err());

        // Then make it recover
        provider.set_should_fail(false);
        let result2 = service.chat_completion(&model_config, messages, tenant_id).await;
        assert!(result2.is_ok());
    }
}