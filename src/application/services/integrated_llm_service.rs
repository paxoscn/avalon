use crate::domain::services::llm_service::{
    LLMDomainService, LLMError, ChatRequest, ChatResponse, ChatStreamChunk, ModelInfo, 
    ConnectionTestResult
};
use crate::domain::value_objects::{ModelConfig, ChatMessage};
use crate::infrastructure::llm::{LLMProviderRegistry};
use crate::infrastructure::llm::error_handling::{RetryWrapper, RetryConfig, CircuitBreaker};
use async_trait::async_trait;
use futures::Stream;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Load balancing strategy for multiple providers
#[derive(Debug, Clone, PartialEq)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    Random,
    WeightedRandom(HashMap<String, f32>),
    HealthBased,
    ResponseTimeBased,
}

/// Provider health status
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub provider_name: String,
    pub is_healthy: bool,
    pub last_check: std::time::Instant,
    pub response_time_ms: u64,
    pub error_count: u32,
    pub success_count: u32,
}

/// Configuration for the integrated LLM service
#[derive(Debug, Clone)]
pub struct IntegratedLLMConfig {
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub health_check_interval: Duration,
    pub max_retries: u32,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout: Duration,
    pub enable_fallback: bool,
    pub fallback_providers: Vec<String>,
    pub request_timeout: Duration,
}

impl Default for IntegratedLLMConfig {
    fn default() -> Self {
        Self {
            load_balancing_strategy: LoadBalancingStrategy::HealthBased,
            health_check_interval: Duration::from_secs(60),
            max_retries: 3,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: Duration::from_secs(30),
            enable_fallback: true,
            fallback_providers: vec!["openai".to_string(), "claude".to_string()],
            request_timeout: Duration::from_secs(30),
        }
    }
}

/// Integrated LLM service that coordinates multiple providers
pub struct IntegratedLLMService {
    provider_registry: Arc<LLMProviderRegistry>,
    config: IntegratedLLMConfig,
    provider_health: Arc<RwLock<HashMap<String, ProviderHealth>>>,
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    retry_wrapper: Arc<RetryWrapper>,
    round_robin_counter: Arc<std::sync::atomic::AtomicUsize>,
}

impl IntegratedLLMService {
    pub fn new(
        provider_registry: Arc<LLMProviderRegistry>,
        config: IntegratedLLMConfig,
    ) -> Self {
        let retry_config = RetryConfig {
            max_attempts: config.max_retries,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            retryable_errors: vec![
                crate::infrastructure::llm::RetryableErrorType::RateLimit,
                crate::infrastructure::llm::RetryableErrorType::NetworkError,
                crate::infrastructure::llm::RetryableErrorType::InternalServerError,
            ],
        };

        let retry_wrapper = Arc::new(RetryWrapper::new(retry_config));
        
        Self {
            provider_registry,
            config,
            provider_health: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            retry_wrapper,
            round_robin_counter: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Initialize health monitoring for all providers
    pub async fn initialize_health_monitoring(&self) {
        let providers = self.provider_registry.list_providers();
        let mut health_map = self.provider_health.write().await;
        let mut circuit_breaker_map = self.circuit_breakers.write().await;

        for provider_name in providers {
            health_map.insert(provider_name.clone(), ProviderHealth {
                provider_name: provider_name.clone(),
                is_healthy: true,
                last_check: std::time::Instant::now(),
                response_time_ms: 0,
                error_count: 0,
                success_count: 0,
            });

            circuit_breaker_map.insert(
                provider_name.clone(),
                Arc::new(CircuitBreaker::new(
                    self.config.circuit_breaker_threshold,
                    self.config.circuit_breaker_timeout,
                )),
            );
        }

        info!("Initialized health monitoring for {} providers", health_map.len());
    }

    /// Select the best provider based on the load balancing strategy
    async fn select_provider(&self, model_config: &ModelConfig) -> Result<String, LLMError> {
        let provider_name = format!("{:?}", model_config.provider).to_lowercase();
        let available_providers = self.get_available_providers().await;

        if available_providers.is_empty() {
            return Err(LLMError::ProviderError("No healthy providers available".to_string()));
        }

        // If the requested provider is available and healthy, prefer it
        if available_providers.contains(&provider_name) {
            return Ok(provider_name);
        }

        // Otherwise, select based on strategy
        match &self.config.load_balancing_strategy {
            LoadBalancingStrategy::RoundRobin => {
                let index = self.round_robin_counter
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed) % available_providers.len();
                Ok(available_providers[index].clone())
            }
            LoadBalancingStrategy::Random => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                Ok(available_providers.choose(&mut rng).unwrap().clone())
            }
            LoadBalancingStrategy::WeightedRandom(weights) => {
                self.select_weighted_random_provider(&available_providers, weights).await
            }
            LoadBalancingStrategy::HealthBased => {
                self.select_healthiest_provider(&available_providers).await
            }
            LoadBalancingStrategy::ResponseTimeBased => {
                self.select_fastest_provider(&available_providers).await
            }
        }
    }

    async fn get_available_providers(&self) -> Vec<String> {
        let health_map = self.provider_health.read().await;
        health_map
            .values()
            .filter(|health| health.is_healthy)
            .map(|health| health.provider_name.clone())
            .collect()
    }

    async fn select_weighted_random_provider(
        &self,
        providers: &[String],
        weights: &HashMap<String, f32>,
    ) -> Result<String, LLMError> {
        use rand::Rng;
        
        let total_weight: f32 = providers
            .iter()
            .map(|p| weights.get(p).unwrap_or(&1.0))
            .sum();

        if total_weight <= 0.0 {
            return Err(LLMError::ProviderError("Invalid weights configuration".to_string()));
        }

        let mut rng = rand::thread_rng();
        let mut random_value = rng.gen::<f32>() * total_weight;

        for provider in providers {
            let weight = weights.get(provider).unwrap_or(&1.0);
            random_value -= weight;
            if random_value <= 0.0 {
                return Ok(provider.clone());
            }
        }

        // Fallback to first provider
        Ok(providers[0].clone())
    }

    async fn select_healthiest_provider(&self, providers: &[String]) -> Result<String, LLMError> {
        let health_map = self.provider_health.read().await;
        
        let best_provider = providers
            .iter()
            .filter_map(|name| health_map.get(name))
            .max_by_key(|health| {
                let success_rate = if health.success_count + health.error_count > 0 {
                    health.success_count as f32 / (health.success_count + health.error_count) as f32
                } else {
                    1.0
                };
                (success_rate * 1000.0) as u32
            })
            .map(|health| health.provider_name.clone());

        best_provider.ok_or_else(|| LLMError::ProviderError("No healthy providers found".to_string()))
    }

    async fn select_fastest_provider(&self, providers: &[String]) -> Result<String, LLMError> {
        let health_map = self.provider_health.read().await;
        
        let fastest_provider = providers
            .iter()
            .filter_map(|name| health_map.get(name))
            .min_by_key(|health| health.response_time_ms)
            .map(|health| health.provider_name.clone());

        fastest_provider.ok_or_else(|| LLMError::ProviderError("No providers found".to_string()))
    }

    async fn execute_with_provider<F, T>(&self, provider_name: &str, operation: F) -> Result<T, LLMError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, LLMError>> + Send>> + Send + Sync,
    {
        let circuit_breaker = {
            let circuit_breakers = self.circuit_breakers.read().await;
            circuit_breakers.get(provider_name).cloned()
        };

        if let Some(cb) = circuit_breaker {
            let start_time = std::time::Instant::now();
            
            let result = cb.execute(|| operation()).await;
            
            let elapsed = start_time.elapsed();
            self.update_provider_health(provider_name, result.is_ok(), elapsed.as_millis() as u64).await;
            
            result
        } else {
            operation().await
        }
    }

    async fn update_provider_health(&self, provider_name: &str, success: bool, response_time_ms: u64) {
        let mut health_map = self.provider_health.write().await;
        
        if let Some(health) = health_map.get_mut(provider_name) {
            health.last_check = std::time::Instant::now();
            health.response_time_ms = response_time_ms;
            
            if success {
                health.success_count += 1;
                health.is_healthy = true;
            } else {
                health.error_count += 1;
                // Mark as unhealthy if error rate is too high
                let total_requests = health.success_count + health.error_count;
                if total_requests >= 10 {
                    let error_rate = health.error_count as f32 / total_requests as f32;
                    health.is_healthy = error_rate < 0.5; // 50% error threshold
                }
            }
        }
    }

    async fn try_fallback_providers<F>(
        &self,
        original_error: LLMError,
        _model_config: &ModelConfig,
        operation: F,
    ) -> Result<ChatResponse, LLMError> 
    where
        F: Fn(&str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ChatResponse, LLMError>> + Send>> + Send + Sync,
    {
        if !self.config.enable_fallback {
            return Err(original_error);
        }

        warn!("Primary provider failed, trying fallback providers");

        for fallback_provider in &self.config.fallback_providers {
            if let Some(_provider) = self.provider_registry.get_provider(fallback_provider) {
                match self.execute_with_provider(fallback_provider, || operation(fallback_provider)).await {
                    Ok(response) => {
                        info!("Fallback provider '{}' succeeded", fallback_provider);
                        return Ok(response);
                    }
                    Err(e) => {
                        warn!("Fallback provider '{}' failed: {}", fallback_provider, e);
                        continue;
                    }
                }
            }
        }

        error!("All fallback providers failed");
        Err(original_error)
    }

    /// Get health status of all providers
    pub async fn get_provider_health_status(&self) -> HashMap<String, ProviderHealth> {
        self.provider_health.read().await.clone()
    }

    /// Manually update provider health (for testing or external health checks)
    pub async fn set_provider_health(&self, provider_name: &str, is_healthy: bool) {
        let mut health_map = self.provider_health.write().await;
        if let Some(health) = health_map.get_mut(provider_name) {
            health.is_healthy = is_healthy;
            health.last_check = std::time::Instant::now();
        }
    }

    /// Run periodic health checks
    pub async fn run_health_checks(&self) {
        let providers = self.provider_registry.list_providers();
        
        for provider_name in providers {
            if let Some(provider) = self.provider_registry.get_provider(&provider_name) {
                let start_time = std::time::Instant::now();
                
                match provider.test_connection().await {
                    Ok(result) => {
                        let response_time = start_time.elapsed().as_millis() as u64;
                        self.update_provider_health(&provider_name, result.success, response_time).await;
                        
                        if result.success {
                            info!("Health check passed for provider '{}' ({}ms)", provider_name, response_time);
                        } else {
                            warn!("Health check failed for provider '{}': {:?}", provider_name, result.error_message);
                        }
                    }
                    Err(e) => {
                        let response_time = start_time.elapsed().as_millis() as u64;
                        self.update_provider_health(&provider_name, false, response_time).await;
                        warn!("Health check error for provider '{}': {}", provider_name, e);
                    }
                }
            }
        }
    }
}

#[async_trait]
impl LLMDomainService for IntegratedLLMService {
    async fn chat_completion(
        &self,
        config: &ModelConfig,
        messages: Vec<ChatMessage>,
        tenant_id: Uuid,
    ) -> Result<ChatResponse, LLMError> {
        let provider_name = self.select_provider(config).await?;
        
        let operation = {
            let provider_registry = self.provider_registry.clone();
            let config = config.clone();
            let messages = messages.clone();
            
            move |selected_provider: &str| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ChatResponse, LLMError>> + Send>> {
                let provider_registry = provider_registry.clone();
                let config = config.clone();
                let messages = messages.clone();
                let selected_provider = selected_provider.to_string();
                
                Box::pin(async move {
                    if let Some(provider) = provider_registry.create_provider(&config) {
                        let request = ChatRequest {
                            messages,
                            model: config.model_name,
                            temperature: config.parameters.temperature,
                            max_tokens: config.parameters.max_tokens,
                            top_p: config.parameters.top_p,
                            frequency_penalty: config.parameters.frequency_penalty,
                            presence_penalty: config.parameters.presence_penalty,
                            stop_sequences: config.parameters.stop_sequences,
                            stream: false,
                            tenant_id,
                        };
                        
                        provider.chat_completion(request).await
                    } else {
                        Err(LLMError::ProviderError(format!("Provider '{}' not found", selected_provider)))
                    }
                })
            }
        };

        match self.execute_with_provider(&provider_name, || operation(&provider_name)).await {
            Ok(response) => Ok(response),
            Err(e) => self.try_fallback_providers(e, config, operation).await,
        }
    }

    async fn generate_embedding(
        &self,
        config: &ModelConfig,
        text: &str,
        _tenant_id: Uuid,
    ) -> Result<Vec<f32>, LLMError> {
        let provider_name = self.select_provider(config).await?;
        
        let provider_name_clone = provider_name.clone();
        let config_clone = config.clone();
        self.execute_with_provider(&provider_name, move || {
            let provider_registry = self.provider_registry.clone();
            let text = text.to_string();
            let provider_name = provider_name_clone.clone();
            let config = config_clone.clone();
            
            Box::pin(async move {
                if let Some(provider) = provider_registry.create_provider(&config) {
                    provider.generate_embedding(&text).await
                } else {
                    Err(LLMError::ProviderError(format!("Provider '{}' not found", provider_name)))
                }
            })
        }).await
    }

    async fn stream_chat_completion(
        &self,
        config: &ModelConfig,
        messages: Vec<ChatMessage>,
        tenant_id: Uuid,
    ) -> Result<Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>, LLMError> {
        let provider_name = self.select_provider(config).await?;
        
        if let Some(provider) = self.provider_registry.create_provider(&config) {
            let request = ChatRequest {
                messages,
                model: config.model_name.clone(),
                temperature: config.parameters.temperature,
                max_tokens: config.parameters.max_tokens,
                top_p: config.parameters.top_p,
                frequency_penalty: config.parameters.frequency_penalty,
                presence_penalty: config.parameters.presence_penalty,
                stop_sequences: config.parameters.stop_sequences.clone(),
                stream: true,
                tenant_id,
            };
            
            provider.stream_chat_completion(request).await
        } else {
            Err(LLMError::ProviderError(format!("Provider '{}' not found", provider_name)))
        }
    }

    fn validate_config(&self, config: &ModelConfig) -> Result<crate::domain::services::llm_service::ValidationResult, LLMError> {
        // Use the existing validation from ModelConfig
        match config.validate() {
            Ok(()) => Ok(crate::domain::services::llm_service::ValidationResult::success()),
            Err(e) => Ok(crate::domain::services::llm_service::ValidationResult::with_errors(vec![e])),
        }
    }

    fn supports_streaming(&self, config: &ModelConfig) -> bool {
        config.supports_streaming()
    }

    async fn get_available_models(&self, provider: &str) -> Result<Vec<ModelInfo>, LLMError> {
        if let Some(provider_impl) = self.provider_registry.get_provider(provider) {
            Ok(provider_impl.get_model_info())
        } else {
            Err(LLMError::ProviderError(format!("Provider '{}' not found", provider)))
        }
    }

    async fn test_connection(&self, config: &ModelConfig) -> Result<ConnectionTestResult, LLMError> {
        let provider_name = format!("{:?}", config.provider).to_lowercase();
        
        if let Some(provider) = self.provider_registry.get_provider(&provider_name) {
            provider.test_connection().await
        } else {
            Ok(ConnectionTestResult {
                success: false,
                response_time_ms: 0,
                error_message: Some(format!("Provider '{}' not available", provider_name)),
                model_info: None,
            })
        }
    }

    fn estimate_token_count(&self, messages: &[ChatMessage], _model: &str) -> Result<u32, LLMError> {
        // Simple estimation based on character count
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
