use async_trait::async_trait;
use crate::domain::entities::APIKey;
use crate::domain::value_objects::{APIKeyId, TenantId, UserId};
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Query options for pagination and filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptions {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub enabled_filter: Option<bool>,
    pub include_expired: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            offset: None,
            limit: Some(50),
            enabled_filter: None,
            include_expired: false,
        }
    }
}

/// Query result with pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIKeyQueryResult {
    pub items: Vec<APIKey>,
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}

/// Repository interface for API key persistence
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait APIKeyRepository: Send + Sync {
    /// Save an API key (create or update)
    async fn save(&self, api_key: &APIKey) -> Result<()>;
    
    /// Find an API key by its ID
    async fn find_by_id(&self, id: APIKeyId) -> Result<Option<APIKey>>;
    
    /// Find an API key by its hash
    async fn find_by_key_hash(&self, key_hash: &str) -> Result<Option<APIKey>>;
    
    /// Find all API keys belonging to a tenant with pagination and filtering
    async fn find_by_tenant(&self, tenant_id: TenantId, options: QueryOptions) -> Result<APIKeyQueryResult>;
    
    /// Find all API keys belonging to a user with pagination and filtering
    async fn find_by_user(&self, user_id: UserId, options: QueryOptions) -> Result<APIKeyQueryResult>;
    
    /// Update an existing API key
    async fn update(&self, api_key: &APIKey) -> Result<()>;
    
    /// Delete an API key by ID
    async fn delete(&self, id: APIKeyId) -> Result<()>;
    
    /// Count API keys belonging to a tenant
    async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64>;
}
