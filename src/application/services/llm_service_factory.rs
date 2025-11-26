use crate::application::services::integrated_llm_service::{IntegratedLLMService, IntegratedLLMConfig, LoadBalancingStrategy};
use crate::infrastructure::llm::{LLMProviderRegistry, LLMProviderFactory};
use crate::error::{PlatformError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use log::{info, warn, error};

/// Configuration for LLM service factory
#[derive(Debug, Clone)]
pub struct LLMServiceFactoryConfig {
    pub openai_api_key: Option<String>,
    pub openai_base_url: Option<String>,
    pub claude_api_key: Option<String>,
    pub local_llm_url: Option<String>,
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub enable_health_monitoring: bool,
    pub health_check_interval_secs: u64,
    pub max_retries: u32,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout_secs: u64,
    pub enable_fallback: bool,
    pub request_timeout_secs: u64,
}

impl Default for LLMServiceFactoryConfig {
    fn default() -> Self {
        Self {
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            openai_base_url: std::env::var("OPENAI_BASE_URL").ok(),
            claude_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            local_llm_url: std::env::var("LOCAL_LLM_URL").ok(),
            load_balancing_strategy: LoadBalancingStrategy::HealthBased,
            enable_health_monitoring: true,
            health_check_interval_secs: 60,
            max_retries: 3,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_secs: 30,
            enable_fallback: true,
            request_timeout_secs: 30,
        }
    }
}

/// Factory for creating and configuring LLM services
pub struct LLMServiceFactory;

impl LLMServiceFactory {
    /// Create a new integrated LLM service with the given configuration
    pub async fn create_integrated_service(
        config: LLMServiceFactoryConfig,
    ) -> Result<Arc<IntegratedLLMService>> {
        let mut registry = LLMProviderRegistry::new();
        let mut fallback_providers = Vec::new();

        // Register OpenAI provider if configured
        if let Some(api_key) = &config.openai_api_key {
            match LLMProviderFactory::create_openai_provider(
                api_key.clone(),
                config.openai_base_url.clone(),
            ) {
                Ok(provider) => {
                    registry.register_provider("openai".to_string(), provider);
                    fallback_providers.push("openai".to_string());
                    info!("Registered OpenAI provider");
                }
                Err(e) => {
                    warn!("Failed to register OpenAI provider: {}", e);
                }
            }
        }

        // Register Claude provider if configured
        if let Some(api_key) = &config.claude_api_key {
            match LLMProviderFactory::create_claude_provider(api_key.clone()) {
                Ok(provider) => {
                    registry.register_provider("claude".to_string(), provider);
                    fallback_providers.push("claude".to_string());
                    info!("Registered Claude provider");
                }
                Err(e) => {
                    warn!("Failed to register Claude provider: {}", e);
                }
            }
        }

        // Register Local LLM provider if configured
        if let Some(base_url) = &config.local_llm_url {
            match LLMProviderFactory::create_local_llm_provider(base_url.clone()) {
                Ok(provider) => {
                    registry.register_provider("local_llm".to_string(), provider);
                    fallback_providers.push("local_llm".to_string());
                    info!("Registered Local LLM provider");
                }
                Err(e) => {
                    warn!("Failed to register Local LLM provider: {}", e);
                }
            }
        }

        // Check if we have at least one provider
        if fallback_providers.is_empty() {
            return Err(PlatformError::ConfigurationError(
                "No LLM providers configured. Please set at least one of: OPENAI_API_KEY, ANTHROPIC_API_KEY, or LOCAL_LLM_URL".to_string()
            ));
        }

        // Create integrated service configuration
        let integrated_config = IntegratedLLMConfig {
            load_balancing_strategy: config.load_balancing_strategy,
            health_check_interval: Duration::from_secs(config.health_check_interval_secs),
            max_retries: config.max_retries,
            circuit_breaker_threshold: config.circuit_breaker_threshold,
            circuit_breaker_timeout: Duration::from_secs(config.circuit_breaker_timeout_secs),
            enable_fallback: config.enable_fallback,
            fallback_providers,
            request_timeout: Duration::from_secs(config.request_timeout_secs),
        };

        // Create the integrated service
        let service = Arc::new(IntegratedLLMService::new(
            Arc::new(registry),
            integrated_config,
        ));

        // Initialize health monitoring
        service.initialize_health_monitoring().await;

        // Start health monitoring if enabled
        if config.enable_health_monitoring {
            Self::start_health_monitoring(service.clone(), Duration::from_secs(config.health_check_interval_secs));
        }

        info!("Created integrated LLM service with {} providers", service.get_provider_health_status().await.len());

        Ok(service)
    }

    /// Create a service with custom provider weights for weighted random load balancing
    pub async fn create_service_with_weights(
        mut config: LLMServiceFactoryConfig,
        weights: HashMap<String, f32>,
    ) -> Result<Arc<IntegratedLLMService>> {
        config.load_balancing_strategy = LoadBalancingStrategy::WeightedRandom(weights);
        Self::create_integrated_service(config).await
    }

    /// Create a service optimized for high availability (multiple fallbacks, aggressive health checking)
    pub async fn create_high_availability_service(
        mut config: LLMServiceFactoryConfig,
    ) -> Result<Arc<IntegratedLLMService>> {
        config.enable_fallback = true;
        config.enable_health_monitoring = true;
        config.health_check_interval_secs = 30; // More frequent health checks
        config.circuit_breaker_threshold = 3; // Lower threshold
        config.circuit_breaker_timeout_secs = 15; // Faster recovery
        config.max_retries = 5; // More retries
        config.load_balancing_strategy = LoadBalancingStrategy::HealthBased;

        Self::create_integrated_service(config).await
    }

    /// Create a service optimized for performance (response time based load balancing)
    pub async fn create_performance_optimized_service(
        mut config: LLMServiceFactoryConfig,
    ) -> Result<Arc<IntegratedLLMService>> {
        config.load_balancing_strategy = LoadBalancingStrategy::ResponseTimeBased;
        config.health_check_interval_secs = 45; // Regular health checks to update response times
        config.request_timeout_secs = 15; // Shorter timeout for faster failover
        config.max_retries = 2; // Fewer retries for faster response

        Self::create_integrated_service(config).await
    }

    /// Start background health monitoring task
    fn start_health_monitoring(service: Arc<IntegratedLLMService>, interval: Duration) {
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                match tokio::time::timeout(Duration::from_secs(30), service.run_health_checks()).await {
                    Ok(()) => {
                        // Health checks completed successfully
                    }
                    Err(_) => {
                        error!("Health check timeout - some providers may be unresponsive");
                    }
                }
            }
        });
        
        info!("Started health monitoring with interval: {:?}", interval);
    }

    /// Create a minimal service for testing (no external dependencies)
    #[cfg(test)]
    pub async fn create_test_service() -> Result<Arc<IntegratedLLMService>> {
        use crate::application::services::integrated_llm_service_test::MockLLMProvider;
        
        let mut registry = LLMProviderRegistry::new();
        
        // Add mock providers for testing
        registry.register_provider("test_openai".to_string(), Arc::new(MockLLMProvider::new("test_openai")));
        registry.register_provider("test_claude".to_string(), Arc::new(MockLLMProvider::new("test_claude")));
        
        let config = IntegratedLLMConfig {
            fallback_providers: vec!["test_openai".to_string(), "test_claude".to_string()],
            ..Default::default()
        };
        
        let service = Arc::new(IntegratedLLMService::new(Arc::new(registry), config));
        service.initialize_health_monitoring().await;
        
        Ok(service)
    }

    /// Validate the factory configuration
    pub fn validate_config(config: &LLMServiceFactoryConfig) -> Result<()> {
        // Check if at least one provider is configured
        if config.openai_api_key.is_none() 
            && config.claude_api_key.is_none() 
            && config.local_llm_url.is_none() {
            return Err(PlatformError::ConfigurationError(
                "At least one LLM provider must be configured".to_string()
            ));
        }

        // Validate timeout values
        if config.request_timeout_secs == 0 {
            return Err(PlatformError::ConfigurationError(
                "Request timeout must be greater than 0".to_string()
            ));
        }

        if config.health_check_interval_secs == 0 {
            return Err(PlatformError::ConfigurationError(
                "Health check interval must be greater than 0".to_string()
            ));
        }

        // Validate circuit breaker settings
        if config.circuit_breaker_threshold == 0 {
            return Err(PlatformError::ConfigurationError(
                "Circuit breaker threshold must be greater than 0".to_string()
            ));
        }

        // Validate retry settings
        if config.max_retries == 0 {
            return Err(PlatformError::ConfigurationError(
                "Max retries must be greater than 0".to_string()
            ));
        }

        // Validate weighted random strategy if applicable
        if let LoadBalancingStrategy::WeightedRandom(weights) = &config.load_balancing_strategy {
            if weights.is_empty() {
                return Err(PlatformError::ConfigurationError(
                    "Weighted random strategy requires at least one weight".to_string()
                ));
            }
            
            for (provider, weight) in weights {
                if *weight <= 0.0 {
                    return Err(PlatformError::ConfigurationError(
                        format!("Weight for provider '{}' must be greater than 0", provider)
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get configuration from environment variables with validation
    pub fn config_from_env() -> Result<LLMServiceFactoryConfig> {
        let config = LLMServiceFactoryConfig::default();
        Self::validate_config(&config)?;
        Ok(config)
    }

    /// Create configuration with custom settings
    pub fn create_config() -> LLMServiceFactoryConfigBuilder {
        LLMServiceFactoryConfigBuilder::new()
    }
}

/// Builder for LLM service factory configuration
pub struct LLMServiceFactoryConfigBuilder {
    config: LLMServiceFactoryConfig,
}

impl LLMServiceFactoryConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: LLMServiceFactoryConfig::default(),
        }
    }

    pub fn with_openai(mut self, api_key: String, base_url: Option<String>) -> Self {
        self.config.openai_api_key = Some(api_key);
        self.config.openai_base_url = base_url;
        self
    }

    pub fn with_claude(mut self, api_key: String) -> Self {
        self.config.claude_api_key = Some(api_key);
        self
    }

    pub fn with_local_llm(mut self, base_url: String) -> Self {
        self.config.local_llm_url = Some(base_url);
        self
    }

    pub fn with_load_balancing_strategy(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.config.load_balancing_strategy = strategy;
        self
    }

    pub fn with_health_monitoring(mut self, enabled: bool, interval_secs: u64) -> Self {
        self.config.enable_health_monitoring = enabled;
        self.config.health_check_interval_secs = interval_secs;
        self
    }

    pub fn with_circuit_breaker(mut self, threshold: u32, timeout_secs: u64) -> Self {
        self.config.circuit_breaker_threshold = threshold;
        self.config.circuit_breaker_timeout_secs = timeout_secs;
        self
    }

    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    pub fn with_fallback(mut self, enabled: bool) -> Self {
        self.config.enable_fallback = enabled;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.config.request_timeout_secs = timeout_secs;
        self
    }

    pub fn build(self) -> Result<LLMServiceFactoryConfig> {
        LLMServiceFactory::validate_config(&self.config)?;
        Ok(self.config)
    }
}

impl Default for LLMServiceFactoryConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = LLMServiceFactory::create_config()
            .with_openai("sk-test123".to_string(), None)
            .with_claude("sk-ant-test123".to_string())
            .with_load_balancing_strategy(LoadBalancingStrategy::RoundRobin)
            .with_health_monitoring(true, 30)
            .with_circuit_breaker(3, 15)
            .with_retries(5)
            .with_fallback(true)
            .with_timeout(20)
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.circuit_breaker_threshold, 3);
        assert_eq!(config.request_timeout_secs, 20);
        assert!(config.enable_fallback);
    }

    #[test]
    fn test_config_validation_no_providers() {
        let config = LLMServiceFactoryConfig {
            openai_api_key: None,
            claude_api_key: None,
            local_llm_url: None,
            ..Default::default()
        };

        let result = LLMServiceFactory::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("At least one LLM provider"));
    }

    #[test]
    fn test_config_validation_invalid_timeout() {
        let config = LLMServiceFactoryConfig {
            openai_api_key: Some("sk-test".to_string()),
            request_timeout_secs: 0,
            ..Default::default()
        };

        let result = LLMServiceFactory::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Request timeout"));
    }

    #[test]
    fn test_config_validation_weighted_random() {
        let mut weights = HashMap::new();
        weights.insert("openai".to_string(), -1.0); // Invalid negative weight

        let config = LLMServiceFactoryConfig {
            openai_api_key: Some("sk-test".to_string()),
            load_balancing_strategy: LoadBalancingStrategy::WeightedRandom(weights),
            ..Default::default()
        };

        let result = LLMServiceFactory::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be greater than 0"));
    }

    #[tokio::test]
    async fn test_create_test_service() {
        let result = LLMServiceFactory::create_test_service().await;
        assert!(result.is_ok());
        
        let service = result.unwrap();
        let health_status = service.get_provider_health_status().await;
        assert_eq!(health_status.len(), 2);
    }
}