use async_trait::async_trait;
use crate::domain::entities::VectorConfigEntity;
use crate::domain::value_objects::{TenantId, ConfigId};
use crate::error::PlatformError;

/// Repository interface for vector configuration management
#[async_trait]
pub trait VectorConfigRepository: Send + Sync {
    /// Find vector configuration by ID
    async fn find_by_id(&self, id: ConfigId) -> Result<Option<VectorConfigEntity>, PlatformError>;
    
    /// Find vector configuration by tenant and name
    async fn find_by_tenant_and_name(
        &self, 
        tenant_id: TenantId, 
        name: &str
    ) -> Result<Option<VectorConfigEntity>, PlatformError>;
    
    /// Find all vector configurations for a tenant
    async fn find_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<VectorConfigEntity>, PlatformError>;
    
    /// Find the default vector configuration for a tenant
    async fn find_default_by_tenant(&self, tenant_id: TenantId) -> Result<Option<VectorConfigEntity>, PlatformError>;
    
    /// Save a vector configuration (create or update)
    async fn save(&self, config: &VectorConfigEntity) -> Result<(), PlatformError>;
    
    /// Delete a vector configuration by ID
    async fn delete(&self, id: ConfigId) -> Result<(), PlatformError>;
    
    /// Set a configuration as default for a tenant (unsets others)
    async fn set_as_default(&self, id: ConfigId, tenant_id: TenantId) -> Result<(), PlatformError>;
    
    /// Check if a configuration name exists for a tenant
    async fn exists_by_tenant_and_name(
        &self, 
        tenant_id: TenantId, 
        name: &str
    ) -> Result<bool, PlatformError>;
    
    /// Count configurations for a tenant
    async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64, PlatformError>;
    
    /// Find configurations by provider for a tenant
    async fn find_by_tenant_and_provider(
        &self, 
        tenant_id: TenantId, 
        provider: &str
    ) -> Result<Vec<VectorConfigEntity>, PlatformError>;
    
    /// Find configurations for a tenant with pagination
    async fn find_by_tenant_paginated(
        &self,
        tenant_id: TenantId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<VectorConfigEntity>, PlatformError>;
}