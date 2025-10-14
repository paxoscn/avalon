use async_trait::async_trait;
use crate::domain::entities::Tenant;
use crate::domain::value_objects::{TenantId, TenantName};
use crate::error::Result;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Find a tenant by their ID
    async fn find_by_id(&self, id: TenantId) -> Result<Option<Tenant>>;
    
    /// Find a tenant by name
    async fn find_by_name(&self, name: &str) -> Result<Option<Tenant>>;
    
    /// Find all tenants
    async fn find_all(&self) -> Result<Vec<Tenant>>;
    
    /// Save a tenant (create or update)
    async fn save(&self, tenant: &Tenant) -> Result<()>;
    
    /// Delete a tenant by ID
    async fn delete(&self, id: TenantId) -> Result<()>;
    
    /// Check if a tenant name exists
    async fn name_exists(&self, name: &str) -> Result<bool>;
    
    /// Count all tenants
    async fn count(&self) -> Result<u64>;
    
    /// Find tenants with pagination
    async fn find_paginated(&self, offset: u64, limit: u64) -> Result<Vec<Tenant>>;
}