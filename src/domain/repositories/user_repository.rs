use async_trait::async_trait;
use crate::domain::entities::User;
use crate::domain::value_objects::{UserId, TenantId, Username};
use crate::error::Result;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their ID
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>>;
    
    /// Find a user by tenant ID and username
    async fn find_by_tenant_and_username(&self, tenant_id: TenantId, username: &str) -> Result<Option<User>>;
    
    /// Find all users belonging to a tenant
    async fn find_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<User>>;
    
    /// Save a user (create or update)
    async fn save(&self, user: &User) -> Result<()>;
    
    /// Delete a user by ID
    async fn delete(&self, id: UserId) -> Result<()>;
    
    /// Check if a username exists within a tenant
    async fn username_exists(&self, tenant_id: TenantId, username: &str) -> Result<bool>;
    
    /// Count users in a tenant
    async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64>;
    
    /// Find users with pagination
    async fn find_by_tenant_paginated(
        &self, 
        tenant_id: TenantId, 
        offset: u64, 
        limit: u64
    ) -> Result<Vec<User>>;
}