use crate::domain::entities::LLMConfig;
use crate::domain::value_objects::{ConfigId, TenantId};
use crate::error::Result;
use async_trait::async_trait;

/// Repository interface for LLM configurations
#[async_trait]
pub trait LLMConfigRepository: Send + Sync {
    /// Find LLM configuration by ID
    async fn find_by_id(&self, id: ConfigId) -> Result<Option<LLMConfig>>;

    /// Find all LLM configurations for a tenant
    async fn find_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<LLMConfig>>;

    /// Find active LLM configurations for a tenant
    async fn find_active_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<LLMConfig>>;

    /// Find the default LLM configuration for a tenant
    async fn find_default_by_tenant(&self, tenant_id: TenantId) -> Result<Option<LLMConfig>>;

    /// Find LLM configuration by name within a tenant
    async fn find_by_tenant_and_name(&self, tenant_id: TenantId, name: &str) -> Result<Option<LLMConfig>>;

    /// Save LLM configuration (create or update)
    async fn save(&self, config: &LLMConfig) -> Result<()>;

    /// Delete LLM configuration
    async fn delete(&self, id: ConfigId) -> Result<()>;

    /// Check if a configuration name exists for a tenant
    async fn name_exists(&self, tenant_id: TenantId, name: &str) -> Result<bool>;

    /// Count configurations for a tenant
    async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64>;

    /// Find configurations by provider for a tenant
    async fn find_by_tenant_and_provider(&self, tenant_id: TenantId, provider: &str) -> Result<Vec<LLMConfig>>;

    /// Set a configuration as default (and unset others)
    async fn set_as_default(&self, tenant_id: TenantId, config_id: ConfigId) -> Result<()>;

    /// Find configurations with pagination
    async fn find_by_tenant_paginated(
        &self,
        tenant_id: TenantId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<LLMConfig>>;
}