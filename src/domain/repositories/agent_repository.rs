use async_trait::async_trait;
use crate::domain::entities::Agent;
use crate::domain::value_objects::{AgentId, TenantId, UserId};
use crate::error::Result;

/// Agent repository interface for managing Agent entities
#[async_trait]
pub trait AgentRepository: Send + Sync {
    /// Find an agent by ID
    async fn find_by_id(&self, id: &AgentId) -> Result<Option<Agent>>;
    
    /// Find all agents by tenant
    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<Agent>>;
    
    /// Find agents created by a specific user
    async fn find_by_creator(&self, creator_id: &UserId) -> Result<Vec<Agent>>;
    
    /// Find agents employed by a specific user
    async fn find_employed_by_user(&self, user_id: &UserId) -> Result<Vec<Agent>>;
    
    /// Save an agent (create or update)
    async fn save(&self, agent: &Agent) -> Result<()>;
    
    /// Delete an agent by ID
    async fn delete(&self, id: &AgentId) -> Result<()>;
    
    /// Count agents by tenant
    async fn count_by_tenant(&self, tenant_id: &TenantId) -> Result<u64>;
    
    /// Find agents with pagination
    async fn find_by_tenant_paginated(
        &self,
        tenant_id: &TenantId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Agent>>;
}

/// Agent employment repository interface for managing employment relationships
#[async_trait]
pub trait AgentEmploymentRepository: Send + Sync {
    /// Create an employment relationship between a user and an agent
    async fn employ(&self, agent_id: &AgentId, user_id: &UserId) -> Result<()>;
    
    /// Terminate an employment relationship
    async fn terminate(&self, agent_id: &AgentId, user_id: &UserId) -> Result<()>;
    
    /// Check if a user has employed an agent
    async fn is_employed(&self, agent_id: &AgentId, user_id: &UserId) -> Result<bool>;
    
    /// Find all users who have employed a specific agent
    async fn find_by_agent(&self, agent_id: &AgentId) -> Result<Vec<UserId>>;
    
    /// Find all agents employed by a specific user
    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<AgentId>>;
}
