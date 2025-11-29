use crate::domain::entities::LLMConfig;
use crate::domain::repositories::LLMConfigRepository;
use crate::domain::services::llm_service::{LLMDomainService, ConnectionTestResult, ModelInfo};
use crate::domain::value_objects::{ConfigId, TenantId, ModelConfig};
use crate::error::{PlatformError, Result};
use crate::infrastructure::llm::LLMProviderRegistry;
use async_trait::async_trait;
use std::sync::Arc;

/// Application service for LLM configuration management
#[async_trait]
pub trait LLMApplicationService: Send + Sync {
    /// Create a new LLM configuration
    async fn create_config(
        &self,
        tenant_id: TenantId,
        name: String,
        model_config: ModelConfig,
        description: Option<String>,
    ) -> Result<LLMConfig>;

    /// Update an existing LLM configuration
    async fn update_config(
        &self,
        config_id: ConfigId,
        tenant_id: TenantId,
        name: Option<String>,
        model_config: Option<ModelConfig>,
        description: Option<String>,
    ) -> Result<LLMConfig>;

    /// Delete an LLM configuration
    async fn delete_config(&self, config_id: ConfigId, tenant_id: TenantId) -> Result<()>;

    /// Get LLM configuration by ID
    async fn get_config(&self, config_id: ConfigId, tenant_id: TenantId) -> Result<LLMConfig>;

    /// List all LLM configurations for a tenant
    async fn list_configs(&self, tenant_id: TenantId) -> Result<Vec<LLMConfig>>;

    /// List active LLM configurations for a tenant
    async fn list_active_configs(&self, tenant_id: TenantId) -> Result<Vec<LLMConfig>>;

    /// Get the default LLM configuration for a tenant
    async fn get_default_config(&self, tenant_id: TenantId) -> Result<Option<LLMConfig>>;

    /// Set a configuration as default
    async fn set_default_config(&self, config_id: ConfigId, tenant_id: TenantId) -> Result<()>;

    /// Test connection to an LLM provider
    async fn test_connection(&self, config_id: ConfigId, tenant_id: TenantId) -> Result<ConnectionTestResult>;

    /// Test connection with a model configuration (without saving)
    async fn test_model_config(&self, model_config: ModelConfig) -> Result<ConnectionTestResult>;

    /// Test connection with actual prompts and get LLM response
    async fn test_connection_with_prompts(
        &self,
        config_id: ConfigId,
        tenant_id: TenantId,
        messages: Vec<crate::domain::value_objects::ChatMessage>,
    ) -> Result<crate::domain::services::llm_service::ChatResponse>;

    /// Get available models for a provider
    async fn get_available_models(&self, provider: &str) -> Result<Vec<ModelInfo>>;

    /// Validate a model configuration
    async fn validate_config(&self, model_config: ModelConfig) -> Result<()>;

    /// List configurations with pagination
    async fn list_configs_paginated(
        &self,
        tenant_id: TenantId,
        page: u64,
        limit: u64,
    ) -> Result<(Vec<LLMConfig>, u64)>;

    /// Get configurations by provider
    async fn get_configs_by_provider(&self, tenant_id: TenantId, provider: &str) -> Result<Vec<LLMConfig>>;
}

pub struct LLMApplicationServiceImpl {
    config_repository: Arc<dyn LLMConfigRepository>,
    llm_domain_service: Arc<dyn LLMDomainService>,
    provider_registry: Arc<LLMProviderRegistry>,
}

impl LLMApplicationServiceImpl {
    pub fn new(
        config_repository: Arc<dyn LLMConfigRepository>,
        llm_domain_service: Arc<dyn LLMDomainService>,
        provider_registry: Arc<LLMProviderRegistry>,
    ) -> Self {
        Self {
            config_repository,
            llm_domain_service,
            provider_registry,
        }
    }

    async fn ensure_config_belongs_to_tenant(&self, config_id: ConfigId, tenant_id: TenantId) -> Result<LLMConfig> {
        let config = self.config_repository
            .find_by_id(config_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("LLM configuration not found".to_string()))?;

        if config.tenant_id != tenant_id {
            return Err(PlatformError::Forbidden("Access denied to this configuration".to_string()));
        }

        Ok(config)
    }

    async fn validate_unique_name(&self, tenant_id: TenantId, name: &str, exclude_id: Option<ConfigId>) -> Result<()> {
        if let Some(existing) = self.config_repository.find_by_tenant_and_name(tenant_id, name).await? {
            if exclude_id.is_none() || exclude_id.unwrap() != existing.id {
                return Err(PlatformError::ValidationError(
                    "Configuration name already exists for this tenant".to_string()
                ));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl LLMApplicationService for LLMApplicationServiceImpl {
    async fn create_config(
        &self,
        tenant_id: TenantId,
        name: String,
        model_config: ModelConfig,
        description: Option<String>,
    ) -> Result<LLMConfig> {
        // Validate the name is unique
        self.validate_unique_name(tenant_id, &name, None).await?;

        // Validate the model configuration
        let validation_result = self.llm_domain_service.validate_config(&model_config)?;
        if !validation_result.is_valid {
            return Err(PlatformError::ValidationError(
                format!("Invalid model configuration: {}", validation_result.errors.join(", "))
            ));
        }

        // Create the configuration
        let mut config = LLMConfig::new(tenant_id, name, model_config);
        if let Some(desc) = description {
            config = config.with_description(desc);
        }

        // If this is the first configuration for the tenant, make it default
        let existing_count = self.config_repository.count_by_tenant(tenant_id).await?;
        if existing_count == 0 {
            config = config.set_as_default();
        }

        // Save the configuration
        self.config_repository.save(&config).await?;

        Ok(config)
    }

    async fn update_config(
        &self,
        config_id: ConfigId,
        tenant_id: TenantId,
        name: Option<String>,
        model_config: Option<ModelConfig>,
        description: Option<String>,
    ) -> Result<LLMConfig> {
        let mut config = self.ensure_config_belongs_to_tenant(config_id, tenant_id).await?;

        // Update name if provided
        if let Some(new_name) = name {
            self.validate_unique_name(tenant_id, &new_name, Some(config_id)).await?;
            config = config.update_name(new_name)
                .map_err(|e| PlatformError::ValidationError(e))?;
        }

        // Update model configuration if provided
        if let Some(new_model_config) = model_config {
            let validation_result = self.llm_domain_service.validate_config(&new_model_config)?;
            if !validation_result.is_valid {
                return Err(PlatformError::ValidationError(
                    format!("Invalid model configuration: {}", validation_result.errors.join(", "))
                ));
            }
            config = config.update_config(new_model_config)
                .map_err(|e| PlatformError::ValidationError(e))?;
        }

        // Update description if provided
        if let Some(desc) = description {
            config = config.with_description(desc);
        }

        // Save the updated configuration
        self.config_repository.save(&config).await?;

        Ok(config)
    }

    async fn delete_config(&self, config_id: ConfigId, tenant_id: TenantId) -> Result<()> {
        let config = self.ensure_config_belongs_to_tenant(config_id, tenant_id).await?;

        // If this is the default configuration, we need to handle it carefully
        if config.is_default {
            let other_configs = self.config_repository.find_by_tenant(tenant_id).await?;
            let remaining_configs: Vec<_> = other_configs.into_iter()
                .filter(|c| c.id != config_id)
                .collect();

            // If there are other configurations, make the first one default
            if let Some(new_default) = remaining_configs.first() {
                self.config_repository.set_as_default(tenant_id, new_default.id).await?;
            }
        }

        self.config_repository.delete(config_id).await?;
        Ok(())
    }

    async fn get_config(&self, config_id: ConfigId, tenant_id: TenantId) -> Result<LLMConfig> {
        self.ensure_config_belongs_to_tenant(config_id, tenant_id).await
    }

    async fn list_configs(&self, tenant_id: TenantId) -> Result<Vec<LLMConfig>> {
        self.config_repository.find_by_tenant(tenant_id).await
    }

    async fn list_active_configs(&self, tenant_id: TenantId) -> Result<Vec<LLMConfig>> {
        self.config_repository.find_active_by_tenant(tenant_id).await
    }

    async fn get_default_config(&self, tenant_id: TenantId) -> Result<Option<LLMConfig>> {
        self.config_repository.find_default_by_tenant(tenant_id).await
    }

    async fn set_default_config(&self, config_id: ConfigId, tenant_id: TenantId) -> Result<()> {
        // Ensure the configuration exists and belongs to the tenant
        self.ensure_config_belongs_to_tenant(config_id, tenant_id).await?;

        // Set as default
        self.config_repository.set_as_default(tenant_id, config_id).await?;

        Ok(())
    }

    async fn test_connection(&self, config_id: ConfigId, tenant_id: TenantId) -> Result<ConnectionTestResult> {
        let config = self.ensure_config_belongs_to_tenant(config_id, tenant_id).await?;
        self.test_model_config(config.model_config).await
    }

    async fn test_model_config(&self, model_config: ModelConfig) -> Result<ConnectionTestResult> {
        // Validate the configuration first
        let validation_result = self.llm_domain_service.validate_config(&model_config)?;
        if !validation_result.is_valid {
            return Ok(ConnectionTestResult {
                success: false,
                response_time_ms: 0,
                error_message: Some(format!("Invalid configuration: {}", validation_result.errors.join(", "))),
                model_info: None,
            });
        }

        // Get the provider and test connection
        let provider_name = format!("{:?}", model_config.provider).to_lowercase();
        // if let Some(provider) = self.provider_registry.get_provider(&provider_name) {
        if let Some(provider) = self.provider_registry.create_provider(&model_config) {
            provider.test_connection().await.map_err(PlatformError::from)
        } else {
            Ok(ConnectionTestResult {
                success: false,
                response_time_ms: 0,
                error_message: Some(format!("Provider '{}' not available", provider_name)),
                model_info: None,
            })
        }
    }

    async fn get_available_models(&self, provider: &str) -> Result<Vec<ModelInfo>> {
        self.llm_domain_service.get_available_models(provider).await.map_err(PlatformError::from)
    }

    async fn validate_config(&self, model_config: ModelConfig) -> Result<()> {
        let validation_result = self.llm_domain_service.validate_config(&model_config)?;
        if !validation_result.is_valid {
            return Err(PlatformError::ValidationError(
                format!("Invalid configuration: {}", validation_result.errors.join(", "))
            ));
        }
        Ok(())
    }

    async fn list_configs_paginated(
        &self,
        tenant_id: TenantId,
        page: u64,
        limit: u64,
    ) -> Result<(Vec<LLMConfig>, u64)> {
        let offset = page * limit;
        let configs = self.config_repository.find_by_tenant_paginated(tenant_id, offset, limit).await?;
        let total = self.config_repository.count_by_tenant(tenant_id).await?;
        Ok((configs, total))
    }

    async fn get_configs_by_provider(&self, tenant_id: TenantId, provider: &str) -> Result<Vec<LLMConfig>> {
        self.config_repository.find_by_tenant_and_provider(tenant_id, provider).await
    }

    async fn test_connection_with_prompts(
        &self,
        config_id: ConfigId,
        tenant_id: TenantId,
        messages: Vec<crate::domain::value_objects::ChatMessage>,
    ) -> Result<crate::domain::services::llm_service::ChatResponse> {
        use crate::domain::services::llm_service::ChatRequest;
        
        // Get the config and ensure it belongs to the tenant
        let config = self.ensure_config_belongs_to_tenant(config_id, tenant_id).await?;
        
        // Validate the configuration
        let validation_result = self.llm_domain_service.validate_config(&config.model_config)?;
        if !validation_result.is_valid {
            return Err(PlatformError::ValidationError(
                format!("Invalid configuration: {}", validation_result.errors.join(", "))
            ));
        }
        
        // Create provider and make the actual LLM call
        if let Some(provider) = self.provider_registry.create_provider(&config.model_config) {
            let request = ChatRequest {
                messages,
                model: config.model_config.model_name.clone(),
                temperature: config.model_config.parameters.temperature,
                max_tokens: config.model_config.parameters.max_tokens,
                top_p: config.model_config.parameters.top_p,
                frequency_penalty: config.model_config.parameters.frequency_penalty,
                presence_penalty: config.model_config.parameters.presence_penalty,
                stop_sequences: config.model_config.parameters.stop_sequences.clone(),
                stream: false,
                stream_options: Some(
                    crate::domain::services::llm_service::StreamOptions {
                        include_obfuscation: true,
                        include_usage: true,
                    }
                ),
                tenant_id: tenant_id.0,
                response_format: None,
            };
            
            provider.chat_completion(request).await.map_err(PlatformError::from)
        } else {
            let provider_name = format!("{:?}", config.model_config.provider).to_lowercase();
            Err(PlatformError::InternalError(
                format!("Provider '{}' not available", provider_name)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{ModelProvider, ModelParameters, ModelCredentials};
    use std::collections::HashMap;
    use mockall::mock;

    mock! {
        ConfigRepo {}

        #[async_trait]
        impl LLMConfigRepository for ConfigRepo {
            async fn find_by_id(&self, id: ConfigId) -> Result<Option<LLMConfig>>;
            async fn find_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<LLMConfig>>;
            async fn find_active_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<LLMConfig>>;
            async fn find_default_by_tenant(&self, tenant_id: TenantId) -> Result<Option<LLMConfig>>;
            async fn find_by_tenant_and_name(&self, tenant_id: TenantId, name: &str) -> Result<Option<LLMConfig>>;
            async fn save(&self, config: &LLMConfig) -> Result<()>;
            async fn delete(&self, id: ConfigId) -> Result<()>;
            async fn name_exists(&self, tenant_id: TenantId, name: &str) -> Result<bool>;
            async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64>;
            async fn find_by_tenant_and_provider(&self, tenant_id: TenantId, provider: &str) -> Result<Vec<LLMConfig>>;
            async fn set_as_default(&self, tenant_id: TenantId, config_id: ConfigId) -> Result<()>;
            async fn find_by_tenant_paginated(&self, tenant_id: TenantId, offset: u64, limit: u64) -> Result<Vec<LLMConfig>>;
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

    #[tokio::test]
    async fn test_create_config_success() {
        let mut mock_repo = MockConfigRepo::new();
        let tenant_id = TenantId::new();
        let model_config = create_test_model_config();

        mock_repo
            .expect_find_by_tenant_and_name()
            .returning(|_, _| Ok(None));

        mock_repo
            .expect_count_by_tenant()
            .returning(|_| Ok(0));

        mock_repo
            .expect_save()
            .returning(|_| Ok(()));

        // Note: In a real test, we would need to mock the LLM domain service and provider registry
        // For now, this test structure shows how the service would be tested
    }

    #[tokio::test]
    async fn test_list_configs_paginated_zero_based() {
        let mut mock_repo = MockConfigRepo::new();
        let tenant_id = TenantId::new();

        // Mock find_by_tenant_paginated to return empty vec
        mock_repo
            .expect_find_by_tenant_paginated()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        // Mock count_by_tenant to return total count
        mock_repo
            .expect_count_by_tenant()
            .times(1)
            .returning(|_| Ok(15));

        let llm_domain_service = Arc::new(crate::domain::services::llm_service::LLMDomainServiceImpl::new(Arc::new(LLMProviderRegistry::new())));
        let provider_registry = Arc::new(crate::infrastructure::llm::LLMProviderRegistry::new());

        let service = LLMApplicationServiceImpl::new(
            Arc::new(mock_repo),
            llm_domain_service,
            provider_registry,
        );

        // Test page 0 (first page) with limit 10
        let result = service.list_configs_paginated(tenant_id, 0, 10).await;

        assert!(result.is_ok());
        let (configs, total) = result.unwrap();
        assert_eq!(configs.len(), 0);
        assert_eq!(total, 15);
    }

    #[tokio::test]
    async fn test_list_configs_paginated_offset_calculation() {
        let mut mock_repo = MockConfigRepo::new();
        let tenant_id = TenantId::new();

        // Verify that offset is calculated as page * limit
        mock_repo
            .expect_find_by_tenant_paginated()
            .times(1)
            .withf(|_, offset, limit| {
                // For page=4, limit=5, offset should be 20
                *offset == 20 && *limit == 5
            })
            .returning(|_, _, _| Ok(vec![]));

        mock_repo
            .expect_count_by_tenant()
            .times(1)
            .returning(|_| Ok(50));

        let llm_domain_service = Arc::new(crate::domain::services::llm_service::LLMDomainServiceImpl::new(Arc::new(LLMProviderRegistry::new())));
        let provider_registry = Arc::new(crate::infrastructure::llm::LLMProviderRegistry::new());

        let service = LLMApplicationServiceImpl::new(
            Arc::new(mock_repo),
            llm_domain_service,
            provider_registry,
        );

        // Test page 4 with limit 5 (offset should be 4 * 5 = 20)
        let result = service.list_configs_paginated(tenant_id, 4, 5).await;

        assert!(result.is_ok());
        let (_, total) = result.unwrap();
        assert_eq!(total, 50);
    }

    #[tokio::test]
    async fn test_list_configs_paginated_total_count_accuracy() {
        let mut mock_repo = MockConfigRepo::new();
        let tenant_id = TenantId::new();

        mock_repo
            .expect_find_by_tenant_paginated()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        // Verify total count is returned accurately
        mock_repo
            .expect_count_by_tenant()
            .times(1)
            .returning(|_| Ok(33));

        let llm_domain_service = Arc::new(crate::domain::services::llm_service::LLMDomainServiceImpl::new(Arc::new(LLMProviderRegistry::new())));
        let provider_registry = Arc::new(crate::infrastructure::llm::LLMProviderRegistry::new());

        let service = LLMApplicationServiceImpl::new(
            Arc::new(mock_repo),
            llm_domain_service,
            provider_registry,
        );

        let result = service.list_configs_paginated(tenant_id, 0, 20).await;

        assert!(result.is_ok());
        let (_, total) = result.unwrap();
        assert_eq!(total, 33);
    }
}