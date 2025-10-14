use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::value_objects::{TenantId, ConfigId};
use crate::infrastructure::vector::{VectorProvider, VectorStoreConfig};

/// Domain entity for vector database configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorConfigEntity {
    pub id: ConfigId,
    pub tenant_id: TenantId,
    pub name: String,
    pub provider: VectorProvider,
    pub connection_params: HashMap<String, String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VectorConfigEntity {
    pub fn new(
        tenant_id: TenantId,
        name: String,
        provider: VectorProvider,
        connection_params: HashMap<String, String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ConfigId::new(),
            tenant_id,
            name,
            provider,
            connection_params,
            is_default: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn with_id(mut self, id: ConfigId) -> Self {
        self.id = id;
        self
    }
    
    pub fn set_as_default(mut self) -> Self {
        self.is_default = true;
        self.updated_at = Utc::now();
        self
    }
    
    pub fn unset_as_default(mut self) -> Self {
        self.is_default = false;
        self.updated_at = Utc::now();
        self
    }
    
    pub fn update_connection_params(mut self, params: HashMap<String, String>) -> Self {
        self.connection_params = params;
        self.updated_at = Utc::now();
        self
    }
    
    pub fn update_name(mut self, name: String) -> Self {
        self.name = name;
        self.updated_at = Utc::now();
        self
    }
    
    /// Convert to infrastructure VectorStoreConfig
    pub fn to_store_config(&self) -> VectorStoreConfig {
        VectorStoreConfig {
            provider: self.provider.clone(),
            connection_params: self.connection_params.clone(),
            default_namespace: None,
            timeout_seconds: 30, // Default timeout
            max_retries: 3,      // Default retries
        }
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Configuration name cannot be empty".to_string());
        }
        
        if self.name.len() > 255 {
            return Err("Configuration name cannot exceed 255 characters".to_string());
        }
        
        // Validate provider-specific required parameters
        match self.provider {
            VectorProvider::Pinecone => {
                self.validate_required_params(&["api_key", "environment", "index_name"])?;
            },
            VectorProvider::ChromaDB => {
                self.validate_required_params(&["base_url", "collection_name"])?;
            },
            VectorProvider::Weaviate => {
                self.validate_required_params(&["base_url", "class_name"])?;
            },
            VectorProvider::Qdrant => {
                self.validate_required_params(&["base_url", "collection_name"])?;
            },
            VectorProvider::Milvus => {
                self.validate_required_params(&["base_url", "collection_name"])?;
            },
        }
        
        Ok(())
    }
    
    fn validate_required_params(&self, required_params: &[&str]) -> Result<(), String> {
        for param in required_params {
            if !self.connection_params.contains_key(*param) {
                return Err(format!("Missing required parameter: {}", param));
            }
            
            if self.connection_params.get(*param).unwrap().trim().is_empty() {
                return Err(format!("Parameter '{}' cannot be empty", param));
            }
        }
        Ok(())
    }
    
    /// Check if this configuration has sensitive data
    pub fn has_sensitive_data(&self) -> bool {
        self.connection_params.keys().any(|key| {
            key.to_lowercase().contains("key") ||
            key.to_lowercase().contains("token") ||
            key.to_lowercase().contains("password") ||
            key.to_lowercase().contains("secret")
        })
    }
    
    /// Get a sanitized version for logging (removes sensitive data)
    pub fn sanitized(&self) -> Self {
        let mut sanitized = self.clone();
        for (key, value) in &mut sanitized.connection_params {
            if key.to_lowercase().contains("key") ||
               key.to_lowercase().contains("token") ||
               key.to_lowercase().contains("password") ||
               key.to_lowercase().contains("secret") {
                *value = "***".to_string();
            }
        }
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    fn create_test_config() -> VectorConfigEntity {
        let mut params = HashMap::new();
        params.insert("api_key".to_string(), "test-key".to_string());
        params.insert("environment".to_string(), "test-env".to_string());
        params.insert("index_name".to_string(), "test-index".to_string());
        
        VectorConfigEntity::new(
            TenantId::new(),
            "Test Config".to_string(),
            VectorProvider::Pinecone,
            params,
        )
    }

    #[test]
    fn test_new_vector_config() {
        let config = create_test_config();
        
        assert_eq!(config.name, "Test Config");
        assert_eq!(config.provider, VectorProvider::Pinecone);
        assert!(!config.is_default);
        assert_eq!(config.connection_params.len(), 3);
    }

    #[test]
    fn test_set_as_default() {
        let config = create_test_config().set_as_default();
        assert!(config.is_default);
    }

    #[test]
    fn test_validate_success() {
        let config = create_test_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let mut params = HashMap::new();
        params.insert("api_key".to_string(), "test-key".to_string());
        
        let config = VectorConfigEntity::new(
            TenantId::new(),
            "".to_string(),
            VectorProvider::Pinecone,
            params,
        );
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_missing_required_params() {
        let params = HashMap::new(); // Empty params
        
        let config = VectorConfigEntity::new(
            TenantId::new(),
            "Test Config".to_string(),
            VectorProvider::Pinecone,
            params,
        );
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_has_sensitive_data() {
        let config = create_test_config();
        assert!(config.has_sensitive_data()); // Contains "api_key"
    }

    #[test]
    fn test_sanitized() {
        let config = create_test_config();
        let sanitized = config.sanitized();
        
        assert_eq!(sanitized.connection_params.get("api_key").unwrap(), "***");
        assert_eq!(sanitized.connection_params.get("environment").unwrap(), "test-env");
    }

    #[test]
    fn test_to_store_config() {
        let config = create_test_config();
        let store_config = config.to_store_config();
        
        assert_eq!(store_config.provider, VectorProvider::Pinecone);
        assert_eq!(store_config.connection_params, config.connection_params);
        assert_eq!(store_config.timeout_seconds, 30);
        assert_eq!(store_config.max_retries, 3);
    }
}