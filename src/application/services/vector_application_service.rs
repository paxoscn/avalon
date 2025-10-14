use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entities::VectorConfigEntity;
use crate::domain::repositories::VectorConfigRepository;
use crate::domain::value_objects::{TenantId, ConfigId};
use crate::error::PlatformError;
use crate::infrastructure::vector::{VectorProvider, VectorStoreFactory, VectorStore};

/// Application service for vector configuration management
pub struct VectorApplicationService {
    vector_config_repository: Arc<dyn VectorConfigRepository>,
}

impl VectorApplicationService {
    pub fn new(vector_config_repository: Arc<dyn VectorConfigRepository>) -> Self {
        Self {
            vector_config_repository,
        }
    }
    
    /// Create a new vector configuration
    pub async fn create_config(
        &self,
        tenant_id: TenantId,
        name: String,
        provider: VectorProvider,
        connection_params: HashMap<String, String>,
    ) -> Result<VectorConfigEntity, PlatformError> {
        // Check if name already exists for this tenant
        if self.vector_config_repository
            .exists_by_tenant_and_name(tenant_id, &name)
            .await? 
        {
            return Err(PlatformError::Conflict(
                format!("Vector configuration with name '{}' already exists", name)
            ));
        }
        
        // Create and validate the configuration
        let config = VectorConfigEntity::new(tenant_id, name, provider, connection_params);
        config.validate()?;
        
        // Test the connection before saving
        self.test_connection(&config).await?;
        
        // Save the configuration
        self.vector_config_repository.save(&config).await?;
        
        Ok(config)
    }
    
    /// Update an existing vector configuration
    pub async fn update_config(
        &self,
        id: ConfigId,
        name: Option<String>,
        connection_params: Option<HashMap<String, String>>,
    ) -> Result<VectorConfigEntity, PlatformError> {
        // Find existing configuration
        let mut config = self.vector_config_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Vector configuration not found".to_string()))?;
        
        // Update fields if provided
        if let Some(new_name) = name {
            // Check if new name conflicts with existing configs
            if new_name != config.name {
                if self.vector_config_repository
                    .exists_by_tenant_and_name(config.tenant_id, &new_name)
                    .await? 
                {
                    return Err(PlatformError::Conflict(
                        format!("Vector configuration with name '{}' already exists", new_name)
                    ));
                }
            }
            config = config.update_name(new_name);
        }
        
        if let Some(new_params) = connection_params {
            config = config.update_connection_params(new_params);
        }
        
        // Validate the updated configuration
        config.validate()?;
        
        // Test the connection with new parameters
        self.test_connection(&config).await?;
        
        // Save the updated configuration
        self.vector_config_repository.save(&config).await?;
        
        Ok(config)
    }
    
    /// Delete a vector configuration
    pub async fn delete_config(&self, id: ConfigId) -> Result<(), PlatformError> {
        // Check if configuration exists
        let config = self.vector_config_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Vector configuration not found".to_string()))?;
        
        // Don't allow deletion of default configuration if it's the only one
        if config.is_default {
            let count = self.vector_config_repository
                .count_by_tenant(config.tenant_id)
                .await?;
            
            if count <= 1 {
                return Err(PlatformError::ValidationError(
                    "Cannot delete the only vector configuration".to_string()
                ));
            }
        }
        
        // Delete the configuration
        self.vector_config_repository.delete(id).await?;
        
        Ok(())
    }
    
    /// Get vector configuration by ID
    pub async fn get_config(&self, id: ConfigId) -> Result<VectorConfigEntity, PlatformError> {
        self.vector_config_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Vector configuration not found".to_string()))
    }
    
    /// Get all vector configurations for a tenant
    pub async fn get_configs_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<VectorConfigEntity>, PlatformError> {
        self.vector_config_repository.find_by_tenant(tenant_id).await
    }
    
    /// Get default vector configuration for a tenant
    pub async fn get_default_config(&self, tenant_id: TenantId) -> Result<VectorConfigEntity, PlatformError> {
        self.vector_config_repository
            .find_default_by_tenant(tenant_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("No default vector configuration found".to_string()))
    }
    
    /// Set a configuration as default for a tenant
    pub async fn set_as_default(&self, id: ConfigId, tenant_id: TenantId) -> Result<(), PlatformError> {
        // Verify the configuration exists and belongs to the tenant
        let config = self.vector_config_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Vector configuration not found".to_string()))?;
        
        if config.tenant_id != tenant_id {
            return Err(PlatformError::AuthorizationFailed(
                "Configuration does not belong to the specified tenant".to_string()
            ));
        }
        
        // Set as default
        self.vector_config_repository.set_as_default(id, tenant_id).await?;
        
        Ok(())
    }
    
    /// Test connection to a vector store
    pub async fn test_connection(&self, config: &VectorConfigEntity) -> Result<(), PlatformError> {
        let store_config = config.to_store_config();
        let store = VectorStoreFactory::create_store(store_config).await?;
        store.test_connection().await
    }
    
    /// Test connection by configuration ID
    pub async fn test_connection_by_id(&self, id: ConfigId) -> Result<(), PlatformError> {
        let config = self.get_config(id).await?;
        self.test_connection(&config).await
    }
    
    /// Get vector store instance from configuration
    pub async fn get_vector_store(&self, id: ConfigId) -> Result<Box<dyn VectorStore>, PlatformError> {
        let config = self.get_config(id).await?;
        let store_config = config.to_store_config();
        VectorStoreFactory::create_store(store_config).await
    }
    
    /// Get default vector store for a tenant
    pub async fn get_default_vector_store(&self, tenant_id: TenantId) -> Result<Box<dyn VectorStore>, PlatformError> {
        let config = self.get_default_config(tenant_id).await?;
        let store_config = config.to_store_config();
        VectorStoreFactory::create_store(store_config).await
    }
    
    /// Get configurations by provider
    pub async fn get_configs_by_provider(
        &self, 
        tenant_id: TenantId, 
        provider: VectorProvider
    ) -> Result<Vec<VectorConfigEntity>, PlatformError> {
        self.vector_config_repository
            .find_by_tenant_and_provider(tenant_id, provider.as_str())
            .await
    }
    
    /// Get health status of all configurations for a tenant
    pub async fn get_health_status(&self, tenant_id: TenantId) -> Result<HashMap<String, bool>, PlatformError> {
        let configs = self.get_configs_by_tenant(tenant_id).await?;
        let mut health_status = HashMap::new();
        
        for config in configs {
            let is_healthy = match self.test_connection(&config).await {
                Ok(_) => true,
                Err(_) => false,
            };
            health_status.insert(config.name.clone(), is_healthy);
        }
        
        Ok(health_status)
    }
    
    /// Validate configuration parameters for a provider
    pub fn validate_provider_params(
        provider: VectorProvider,
        params: &HashMap<String, String>,
    ) -> Result<(), PlatformError> {
        let temp_config = VectorConfigEntity::new(
            TenantId::new(),
            "temp".to_string(),
            provider,
            params.clone(),
        );
        
        temp_config.validate().map_err(|e| PlatformError::ValidationError(e))
    }
    
    /// Get required parameters for a provider
    pub fn get_required_params(provider: VectorProvider) -> Vec<String> {
        match provider {
            VectorProvider::Pinecone => vec![
                "api_key".to_string(),
                "environment".to_string(),
                "index_name".to_string(),
            ],
            VectorProvider::ChromaDB => vec![
                "base_url".to_string(),
                "collection_name".to_string(),
            ],
            VectorProvider::Weaviate => vec![
                "base_url".to_string(),
                "class_name".to_string(),
            ],
            VectorProvider::Qdrant => vec![
                "base_url".to_string(),
                "collection_name".to_string(),
            ],
            VectorProvider::Milvus => vec![
                "base_url".to_string(),
                "collection_name".to_string(),
            ],
        }
    }
    
    /// Get optional parameters for a provider
    pub fn get_optional_params(provider: VectorProvider) -> Vec<String> {
        match provider {
            VectorProvider::Pinecone => vec![],
            VectorProvider::ChromaDB => vec!["api_key".to_string()],
            VectorProvider::Weaviate => vec!["api_key".to_string()],
            VectorProvider::Qdrant => vec!["api_key".to_string()],
            VectorProvider::Milvus => vec!["api_key".to_string(), "username".to_string(), "password".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::VectorConfigRepository;
    use async_trait::async_trait;
    use std::sync::Mutex;
    

    // Mock repository for testing
    struct MockVectorConfigRepository {
        configs: Mutex<HashMap<ConfigId, VectorConfigEntity>>,
    }

    impl MockVectorConfigRepository {
        fn new() -> Self {
            Self {
                configs: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl VectorConfigRepository for MockVectorConfigRepository {
        async fn find_by_id(&self, id: ConfigId) -> Result<Option<VectorConfigEntity>, PlatformError> {
            let configs = self.configs.lock().unwrap();
            Ok(configs.get(&id).cloned())
        }
        
        async fn find_by_tenant_and_name(&self, tenant_id: TenantId, name: &str) -> Result<Option<VectorConfigEntity>, PlatformError> {
            let configs = self.configs.lock().unwrap();
            Ok(configs.values().find(|c| c.tenant_id == tenant_id && c.name == name).cloned())
        }
        
        async fn find_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<VectorConfigEntity>, PlatformError> {
            let configs = self.configs.lock().unwrap();
            Ok(configs.values().filter(|c| c.tenant_id == tenant_id).cloned().collect())
        }
        
        async fn find_default_by_tenant(&self, tenant_id: TenantId) -> Result<Option<VectorConfigEntity>, PlatformError> {
            let configs = self.configs.lock().unwrap();
            Ok(configs.values().find(|c| c.tenant_id == tenant_id && c.is_default).cloned())
        }
        
        async fn save(&self, config: &VectorConfigEntity) -> Result<(), PlatformError> {
            let mut configs = self.configs.lock().unwrap();
            configs.insert(config.id, config.clone());
            Ok(())
        }
        
        async fn delete(&self, id: ConfigId) -> Result<(), PlatformError> {
            let mut configs = self.configs.lock().unwrap();
            configs.remove(&id);
            Ok(())
        }
        
        async fn set_as_default(&self, id: ConfigId, tenant_id: TenantId) -> Result<(), PlatformError> {
            let mut configs = self.configs.lock().unwrap();
            
            // Unset all defaults for tenant
            for config in configs.values_mut() {
                if config.tenant_id == tenant_id {
                    config.is_default = false;
                }
            }
            
            // Set the specified config as default
            if let Some(config) = configs.get_mut(&id) {
                if config.tenant_id == tenant_id {
                    config.is_default = true;
                }
            }
            
            Ok(())
        }
        
        async fn exists_by_tenant_and_name(&self, tenant_id: TenantId, name: &str) -> Result<bool, PlatformError> {
            let configs = self.configs.lock().unwrap();
            Ok(configs.values().any(|c| c.tenant_id == tenant_id && c.name == name))
        }
        
        async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64, PlatformError> {
            let configs = self.configs.lock().unwrap();
            Ok(configs.values().filter(|c| c.tenant_id == tenant_id).count() as u64)
        }
        
        async fn find_by_tenant_and_provider(&self, tenant_id: TenantId, provider: &str) -> Result<Vec<VectorConfigEntity>, PlatformError> {
            let configs = self.configs.lock().unwrap();
            let provider_enum = VectorProvider::from_str(provider)?;
            Ok(configs.values()
                .filter(|c| c.tenant_id == tenant_id && c.provider == provider_enum)
                .cloned()
                .collect())
        }
    }

    fn create_test_service() -> VectorApplicationService {
        let repo = Arc::new(MockVectorConfigRepository::new());
        VectorApplicationService::new(repo)
    }

    #[tokio::test]
    async fn test_get_required_params() {
        let params = VectorApplicationService::get_required_params(VectorProvider::Pinecone);
        assert_eq!(params.len(), 3);
        assert!(params.contains(&"api_key".to_string()));
        assert!(params.contains(&"environment".to_string()));
        assert!(params.contains(&"index_name".to_string()));
    }

    #[tokio::test]
    async fn test_validate_provider_params() {
        let mut params = HashMap::new();
        params.insert("api_key".to_string(), "test-key".to_string());
        params.insert("environment".to_string(), "test-env".to_string());
        params.insert("index_name".to_string(), "test-index".to_string());
        
        let result = VectorApplicationService::validate_provider_params(VectorProvider::Pinecone, &params);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_provider_params_missing() {
        let params = HashMap::new(); // Empty params
        
        let result = VectorApplicationService::validate_provider_params(VectorProvider::Pinecone, &params);
        assert!(result.is_err());
    }
}