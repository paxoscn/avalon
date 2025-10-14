use std::collections::HashMap;
use std::sync::Arc;

use crate::application::services::VectorApplicationService;
use crate::domain::entities::VectorConfigEntity;
use crate::domain::repositories::VectorConfigRepository;
use crate::domain::value_objects::{TenantId, ConfigId};
use crate::error::PlatformError;
use crate::infrastructure::vector::VectorProvider;

// Mock repository for testing
pub struct MockVectorConfigRepository {
    configs: std::sync::Mutex<HashMap<ConfigId, VectorConfigEntity>>,
}

impl MockVectorConfigRepository {
    pub fn new() -> Self {
        Self {
            configs: std::sync::Mutex::new(HashMap::new()),
        }
    }
    
    pub fn with_config(self, config: VectorConfigEntity) -> Self {
        let mut configs = self.configs.lock().unwrap();
        configs.insert(config.id, config);
        drop(configs);
        self
    }
}

#[async_trait::async_trait]
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

fn create_test_config(tenant_id: TenantId, name: &str) -> VectorConfigEntity {
    let mut params = HashMap::new();
    params.insert("api_key".to_string(), "test-key".to_string());
    params.insert("environment".to_string(), "test-env".to_string());
    params.insert("index_name".to_string(), "test-index".to_string());
    
    VectorConfigEntity::new(
        tenant_id,
        name.to_string(),
        VectorProvider::Pinecone,
        params,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_required_params() {
        let params = VectorApplicationService::get_required_params(VectorProvider::Pinecone);
        assert_eq!(params.len(), 3);
        assert!(params.contains(&"api_key".to_string()));
        assert!(params.contains(&"environment".to_string()));
        assert!(params.contains(&"index_name".to_string()));
    }

    #[tokio::test]
    async fn test_get_optional_params() {
        let params = VectorApplicationService::get_optional_params(VectorProvider::ChromaDB);
        assert!(params.contains(&"api_key".to_string()));
    }

    #[tokio::test]
    async fn test_validate_provider_params_success() {
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

    #[tokio::test]
    async fn test_get_configs_by_tenant() {
        let tenant_id = TenantId::new();
        let config = create_test_config(tenant_id, "Test Config");
        
        let repo = Arc::new(MockVectorConfigRepository::new().with_config(config.clone()));
        let service = VectorApplicationService::new(repo);
        
        let configs = service.get_configs_by_tenant(tenant_id).await.unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].name, "Test Config");
    }

    #[tokio::test]
    async fn test_get_config_by_id() {
        let tenant_id = TenantId::new();
        let config = create_test_config(tenant_id, "Test Config");
        let config_id = config.id;
        
        let repo = Arc::new(MockVectorConfigRepository::new().with_config(config));
        let service = VectorApplicationService::new(repo);
        
        let retrieved_config = service.get_config(config_id).await.unwrap();
        assert_eq!(retrieved_config.name, "Test Config");
        assert_eq!(retrieved_config.id, config_id);
    }

    #[tokio::test]
    async fn test_get_config_not_found() {
        let service = create_test_service();
        let config_id = ConfigId::new();
        
        let result = service.get_config(config_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PlatformError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_get_configs_by_provider() {
        let tenant_id = TenantId::new();
        let config = create_test_config(tenant_id, "Pinecone Config");
        
        let repo = Arc::new(MockVectorConfigRepository::new().with_config(config));
        let service = VectorApplicationService::new(repo);
        
        let configs = service.get_configs_by_provider(tenant_id, VectorProvider::Pinecone).await.unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].provider, VectorProvider::Pinecone);
    }

    #[tokio::test]
    async fn test_get_default_config_not_found() {
        let service = create_test_service();
        let tenant_id = TenantId::new();
        
        let result = service.get_default_config(tenant_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PlatformError::NotFound(_)));
    }
}