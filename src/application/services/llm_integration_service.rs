use crate::application::services::{
    llm_application_service::{LLMApplicationService, LLMApplicationServiceImpl},
    integrated_llm_service::{IntegratedLLMService, LoadBalancingStrategy},
    llm_service_factory::{LLMServiceFactory, LLMServiceFactoryConfig},
};
use crate::domain::repositories::LLMConfigRepository;
use crate::domain::services::llm_service::{LLMDomainService, LLMError, ChatResponse, ModelInfo, ConnectionTestResult};
use crate::domain::value_objects::{ConfigId, TenantId, ModelConfig, ChatMessage};
use crate::error::{PlatformError, Result};
use crate::infrastructure::llm::LLMProviderRegistry;
use futures::Stream;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error};

/// Comprehensive LLM integration service that coordinates all LLM operations
/// This service acts as the main entry point for all LLM-related functionality
pub struct LLMIntegrationService {
    /// Configuration management service
    config_service: Arc<dyn LLMApplicationService>,
    /// Multi-provider LLM service with load balancing and fallback
    integrated_service: Arc<IntegratedLLMService>,
    /// Provider registry for managing available providers
    provider_registry: Arc<LLMProviderRegistry>,
    /// Service configuration
    service_config: LLMIntegrationConfig,
}

/// Configuration for the LLM integration service
#[derive(Debug, Clone)]
pub struct LLMIntegrationConfig {
    pub enable_auto_failover: bool,
    pub health_check_interval: Duration,
    pub max_concurrent_requests: usize,
    pub request_timeout: Duration,
    pub enable_metrics: bool,
    pub enable_caching: bool,
    pub cache_ttl: Duration,
}

impl Default for LLMIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_auto_failover: true,
            health_check_interval: Duration::from_secs(60),
            max_concurrent_requests: 100,
            request_timeout: Duration::from_secs(30),
            enable_metrics: true,
            enable_caching: false,
            cache_ttl: Duration::from_secs(300),
        }
    }
}

/// Service metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct LLMServiceMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub provider_usage: HashMap<String, u64>,
    pub error_counts: HashMap<String, u64>,
}

impl LLMIntegrationService {
    /// Create a new LLM integration service
    pub async fn new(
        config_repository: Arc<dyn LLMConfigRepository>,
        service_config: LLMIntegrationConfig,
        factory_config: LLMServiceFactoryConfig,
    ) -> Result<Self> {
        // Create the integrated LLM service with load balancing and fallback
        let integrated_service = LLMServiceFactory::create_integrated_service(factory_config).await?;
        
        // Create a new provider registry (we'll need to expose this from IntegratedLLMService or create it separately)
        let provider_registry = Arc::new(LLMProviderRegistry::new());
        
        // Create the configuration management service
        let config_service = Arc::new(LLMApplicationServiceImpl::new(
            config_repository,
            integrated_service.clone(),
            provider_registry.clone(),
        ));

        Ok(Self {
            config_service,
            integrated_service,
            provider_registry,
            service_config,
        })
    }

    /// Create a service with high availability configuration
    pub async fn new_high_availability(
        config_repository: Arc<dyn LLMConfigRepository>,
    ) -> Result<Self> {
        let factory_config = LLMServiceFactory::config_from_env()?;
        let integrated_service = LLMServiceFactory::create_high_availability_service(factory_config).await?;
        
        let provider_registry = Arc::new(LLMProviderRegistry::new());
        let config_service = Arc::new(LLMApplicationServiceImpl::new(
            config_repository,
            integrated_service.clone(),
            provider_registry.clone(),
        ));

        let service_config = LLMIntegrationConfig {
            enable_auto_failover: true,
            health_check_interval: Duration::from_secs(30),
            max_concurrent_requests: 200,
            request_timeout: Duration::from_secs(15),
            enable_metrics: true,
            enable_caching: true,
            cache_ttl: Duration::from_secs(600),
        };

        Ok(Self {
            config_service,
            integrated_service,
            provider_registry,
            service_config,
        })
    }

    /// Create a service optimized for performance
    pub async fn new_performance_optimized(
        config_repository: Arc<dyn LLMConfigRepository>,
    ) -> Result<Self> {
        let factory_config = LLMServiceFactory::config_from_env()?;
        let integrated_service = LLMServiceFactory::create_performance_optimized_service(factory_config).await?;
        
        let provider_registry = Arc::new(LLMProviderRegistry::new());
        let config_service = Arc::new(LLMApplicationServiceImpl::new(
            config_repository,
            integrated_service.clone(),
            provider_registry.clone(),
        ));

        let service_config = LLMIntegrationConfig {
            enable_auto_failover: true,
            health_check_interval: Duration::from_secs(45),
            max_concurrent_requests: 150,
            request_timeout: Duration::from_secs(10),
            enable_metrics: true,
            enable_caching: true,
            cache_ttl: Duration::from_secs(180),
        };

        Ok(Self {
            config_service,
            integrated_service,
            provider_registry,
            service_config,
        })
    }

    /// Get the configuration management service
    pub fn config_service(&self) -> Arc<dyn LLMApplicationService> {
        self.config_service.clone()
    }

    /// Get the integrated LLM service
    pub fn integrated_service(&self) -> Arc<IntegratedLLMService> {
        self.integrated_service.clone()
    }

    /// Chat completion using tenant's default configuration
    pub async fn chat_completion_with_default_config(
        &self,
        tenant_id: TenantId,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResponse> {
        let config = self.config_service
            .get_default_config(tenant_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("No default LLM configuration found".to_string()))?;

        self.integrated_service
            .chat_completion(&config.model_config, messages, tenant_id.0, None)
            .await
            .map_err(PlatformError::from)
    }

    /// Chat completion using specific configuration
    pub async fn chat_completion_with_config(
        &self,
        config_id: ConfigId,
        tenant_id: TenantId,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResponse> {
        let config = self.config_service
            .get_config(config_id, tenant_id)
            .await?;

        self.integrated_service
            .chat_completion(&config.model_config, messages, tenant_id.0, None)
            .await
            .map_err(PlatformError::from)
    }

    /// Stream chat completion using tenant's default configuration
    pub async fn stream_chat_completion_with_default_config(
        &self,
        tenant_id: TenantId,
        messages: Vec<ChatMessage>,
    ) -> Result<Box<dyn Stream<Item = std::result::Result<crate::domain::services::llm_service::ChatStreamChunk, LLMError>> + Send + Unpin>> {
        let config = self.config_service
            .get_default_config(tenant_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("No default LLM configuration found".to_string()))?;

        self.integrated_service
            .stream_chat_completion(&config.model_config, messages, tenant_id.0)
            .await
            .map_err(PlatformError::from)
    }

    /// Generate embeddings using tenant's default configuration
    pub async fn generate_embedding_with_default_config(
        &self,
        tenant_id: TenantId,
        text: &str,
    ) -> Result<Vec<f32>> {
        let config = self.config_service
            .get_default_config(tenant_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("No default LLM configuration found".to_string()))?;

        self.integrated_service
            .generate_embedding(&config.model_config, text, tenant_id.0)
            .await
            .map_err(PlatformError::from)
    }

    /// Test all provider connections and update health status
    pub async fn test_all_provider_connections(&self) -> Result<HashMap<String, ConnectionTestResult>> {
        let results = self.provider_registry.test_all_connections().await;
        
        // Update health status based on test results
        for (provider_name, result) in &results {
            self.integrated_service.set_provider_health(provider_name, result.success).await;
        }

        Ok(results)
    }

    /// Get comprehensive health status of all providers
    pub async fn get_health_status(&self) -> Result<HashMap<String, crate::application::services::integrated_llm_service::ProviderHealth>> {
        Ok(self.integrated_service.get_provider_health_status().await)
    }

    /// Run health checks for all providers
    pub async fn run_health_checks(&self) -> Result<()> {
        self.integrated_service.run_health_checks().await;
        Ok(())
    }

    /// Get service metrics
    pub async fn get_metrics(&self) -> Result<LLMServiceMetrics> {
        // In a real implementation, this would collect metrics from various sources
        // For now, return default metrics
        Ok(LLMServiceMetrics::default())
    }

    /// Validate a model configuration without saving it
    pub async fn validate_model_config(&self, config: &ModelConfig) -> Result<()> {
        self.integrated_service
            .validate_config(config)
            .map_err(PlatformError::from)?;
        Ok(())
    }

    /// Get available models for all providers
    pub async fn get_all_available_models(&self) -> Result<HashMap<String, Vec<ModelInfo>>> {
        let mut all_models = HashMap::new();
        let providers = self.provider_registry.list_providers();

        for provider_name in providers {
            match self.integrated_service.get_available_models(&provider_name).await {
                Ok(models) => {
                    all_models.insert(provider_name, models);
                }
                Err(e) => {
                    warn!("Failed to get models for provider '{}': {}", provider_name, e);
                }
            }
        }

        Ok(all_models)
    }

    /// Estimate token count for messages
    pub fn estimate_token_count(&self, messages: &[ChatMessage], model: &str) -> Result<u32> {
        self.integrated_service
            .estimate_token_count(messages, model)
            .map_err(PlatformError::from)
    }

    /// Start background health monitoring
    pub fn start_health_monitoring(&self) {
        let service = self.integrated_service.clone();
        let interval = self.service_config.health_check_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                match tokio::time::timeout(Duration::from_secs(30), service.run_health_checks()).await {
                    Ok(()) => {
                        info!("Health checks completed successfully");
                    }
                    Err(_) => {
                        error!("Health check timeout - some providers may be unresponsive");
                    }
                }
            }
        });
        
        info!("Started LLM health monitoring with interval: {:?}", interval);
    }

    /// Shutdown the service gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down LLM integration service");
        // In a real implementation, this would clean up resources, stop background tasks, etc.
        Ok(())
    }
}

/// Builder for creating LLM integration service with custom configuration
pub struct LLMIntegrationServiceBuilder {
    config_repository: Option<Arc<dyn LLMConfigRepository>>,
    service_config: LLMIntegrationConfig,
    factory_config: Option<LLMServiceFactoryConfig>,
}

impl LLMIntegrationServiceBuilder {
    pub fn new() -> Self {
        Self {
            config_repository: None,
            service_config: LLMIntegrationConfig::default(),
            factory_config: None,
        }
    }

    pub fn with_config_repository(mut self, repository: Arc<dyn LLMConfigRepository>) -> Self {
        self.config_repository = Some(repository);
        self
    }

    pub fn with_service_config(mut self, config: LLMIntegrationConfig) -> Self {
        self.service_config = config;
        self
    }

    pub fn with_factory_config(mut self, config: LLMServiceFactoryConfig) -> Self {
        self.factory_config = Some(config);
        self
    }

    pub fn with_load_balancing_strategy(mut self, strategy: LoadBalancingStrategy) -> Self {
        if let Some(ref mut factory_config) = self.factory_config {
            factory_config.load_balancing_strategy = strategy;
        } else {
            let mut factory_config = LLMServiceFactoryConfig::default();
            factory_config.load_balancing_strategy = strategy;
            self.factory_config = Some(factory_config);
        }
        self
    }

    pub fn with_health_monitoring(mut self, enabled: bool, interval: Duration) -> Self {
        self.service_config.health_check_interval = interval;
        
        if let Some(ref mut factory_config) = self.factory_config {
            factory_config.enable_health_monitoring = enabled;
            factory_config.health_check_interval_secs = interval.as_secs();
        } else {
            let mut factory_config = LLMServiceFactoryConfig::default();
            factory_config.enable_health_monitoring = enabled;
            factory_config.health_check_interval_secs = interval.as_secs();
            self.factory_config = Some(factory_config);
        }
        self
    }

    pub fn with_auto_failover(mut self, enabled: bool) -> Self {
        self.service_config.enable_auto_failover = enabled;
        
        if let Some(ref mut factory_config) = self.factory_config {
            factory_config.enable_fallback = enabled;
        } else {
            let mut factory_config = LLMServiceFactoryConfig::default();
            factory_config.enable_fallback = enabled;
            self.factory_config = Some(factory_config);
        }
        self
    }

    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.service_config.request_timeout = timeout;
        
        if let Some(ref mut factory_config) = self.factory_config {
            factory_config.request_timeout_secs = timeout.as_secs();
        } else {
            let mut factory_config = LLMServiceFactoryConfig::default();
            factory_config.request_timeout_secs = timeout.as_secs();
            self.factory_config = Some(factory_config);
        }
        self
    }

    pub fn with_caching(mut self, enabled: bool, ttl: Duration) -> Self {
        self.service_config.enable_caching = enabled;
        self.service_config.cache_ttl = ttl;
        self
    }

    pub async fn build(self) -> Result<LLMIntegrationService> {
        let config_repository = self.config_repository
            .ok_or_else(|| PlatformError::ConfigurationError("Config repository is required".to_string()))?;

        let factory_config = self.factory_config
            .unwrap_or_else(|| LLMServiceFactoryConfig::default());

        LLMIntegrationService::new(config_repository, self.service_config, factory_config).await
    }
}

impl Default for LLMIntegrationServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

