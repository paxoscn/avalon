use async_trait::async_trait;
use crate::domain::entities::Agent;
use crate::domain::value_objects::{AgentId, TenantId, UserId};
use crate::error::Result;

/// Agent repository interface for managing Agent entities
#[async_trait]
pub trait AgentRepository: Send + Sync {
    /// Find an agent by ID
    async fn find_by_id(&self, id: &AgentId) -> Result<Option<Agent>>;
    
    /// Find all agents by tenant (including fired agents)
    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<Agent>>;
    
    /// Find all active (non-fired) agents by tenant
    async fn find_by_tenant_active(&self, tenant_id: &TenantId) -> Result<Vec<Agent>>;
    
    /// Find agents created by a specific user
    async fn find_by_creator(&self, creator_id: &UserId) -> Result<Vec<Agent>>;
    
    /// Find agents employed by a specific user (employer_id matches user_id)
    async fn find_by_employer(&self, employer_id: &UserId) -> Result<Vec<Agent>>;
    
    /// Find agents allocated to a specific user
    async fn find_allocated_to_user(&self, user_id: &UserId) -> Result<Vec<Agent>>;
    
    /// Save an agent (create or update)
    async fn save(&self, agent: &Agent) -> Result<()>;
    
    /// Delete an agent by ID
    async fn delete(&self, id: &AgentId) -> Result<()>;
    
    /// Count all agents by tenant (including fired agents)
    async fn count_by_tenant(&self, tenant_id: &TenantId) -> Result<u64>;
    
    /// Count active (non-fired) agents by tenant
    async fn count_by_tenant_active(&self, tenant_id: &TenantId) -> Result<u64>;
    
    /// Find agents with pagination (including fired agents)
    async fn find_by_tenant_paginated(
        &self,
        tenant_id: &TenantId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Agent>>;
    
    /// Find active (non-fired) agents with pagination
    async fn find_by_tenant_active_paginated(
        &self,
        tenant_id: &TenantId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Agent>>;
    
    /// Find published agents by tenant
    async fn find_by_tenant_published(&self, tenant_id: &TenantId) -> Result<Vec<Agent>>;
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

/// Agent allocation repository interface for managing allocation relationships
#[async_trait]
pub trait AgentAllocationRepository: Send + Sync {
    /// Create an allocation relationship between a user and an agent
    async fn allocate(&self, agent_id: &AgentId, user_id: &UserId) -> Result<()>;
    
    /// Terminate an allocation relationship
    async fn terminate(&self, agent_id: &AgentId, user_id: &UserId) -> Result<()>;
    
    /// Check if a user has been allocated an agent
    async fn is_allocated(&self, agent_id: &AgentId, user_id: &UserId) -> Result<bool>;
    
    /// Find all users who have been allocated a specific agent
    async fn find_by_agent(&self, agent_id: &AgentId) -> Result<Vec<UserId>>;
    
    /// Find all agents allocated to a specific user
    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<AgentId>>;
}
