use async_trait::async_trait;
use crate::domain::entities::{Flow, FlowVersion, FlowExecution, FlowStatus, FlowExecutionStatus};
use crate::domain::value_objects::{FlowId, TenantId, UserId, SessionId, FlowExecutionId, Version};
use crate::error::Result;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait FlowRepository: Send + Sync {
    /// Find a flow by ID
    async fn find_by_id(&self, id: &FlowId) -> Result<Option<Flow>>;
    
    /// Find flows by tenant
    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<Flow>>;
    
    /// Find flows by tenant and status
    async fn find_by_tenant_and_status(&self, tenant_id: &TenantId, status: &FlowStatus) -> Result<Vec<Flow>>;
    
    /// Find flows created by a specific user
    async fn find_by_creator(&self, created_by: &UserId) -> Result<Vec<Flow>>;
    
    /// Save a flow (create or update)
    async fn save(&self, flow: &Flow) -> Result<()>;
    
    /// Delete a flow by ID
    async fn delete(&self, id: &FlowId) -> Result<()>;
    
    /// Count flows by tenant
    async fn count_by_tenant(&self, tenant_id: &TenantId) -> Result<u64>;
    
    /// Find flows with pagination
    async fn find_by_tenant_paginated(
        &self, 
        tenant_id: &TenantId, 
        offset: u64, 
        limit: u64
    ) -> Result<Vec<Flow>>;
    
    /// Check if flow name exists within tenant
    async fn name_exists_in_tenant(&self, tenant_id: &TenantId, name: &str) -> Result<bool>;
}

#[async_trait]
pub trait FlowVersionRepository: Send + Sync {
    /// Find a flow version by ID
    async fn find_by_id(&self, id: &FlowId) -> Result<Option<FlowVersion>>;
    
    /// Find a specific version of a flow
    async fn find_by_flow_and_version(&self, flow_id: &FlowId, version: &Version) -> Result<Option<FlowVersion>>;
    
    /// Find all versions of a flow
    async fn find_by_flow(&self, flow_id: &FlowId) -> Result<Vec<FlowVersion>>;
    
    /// Find the latest version of a flow
    async fn find_latest_by_flow(&self, flow_id: &FlowId) -> Result<Option<FlowVersion>>;
    
    /// Save a flow version
    async fn save(&self, version: &FlowVersion, tenant_id: &TenantId) -> Result<()>;
    
    /// Delete a flow version
    async fn delete(&self, id: &FlowId) -> Result<()>;
    
    /// Delete all versions of a flow
    async fn delete_by_flow(&self, flow_id: &FlowId) -> Result<()>;
    
    /// Count versions of a flow
    async fn count_by_flow(&self, flow_id: &FlowId) -> Result<u64>;
    
    /// Find versions with pagination
    async fn find_by_flow_paginated(
        &self, 
        flow_id: &FlowId, 
        offset: u64, 
        limit: u64
    ) -> Result<Vec<FlowVersion>>;
}

#[async_trait]
pub trait FlowExecutionRepository: Send + Sync {
    /// Find a flow execution by ID
    async fn find_by_id(&self, id: &FlowExecutionId) -> Result<Option<FlowExecution>>;
    
    /// Find executions by flow ID
    async fn find_by_flow(&self, flow_id: &FlowId) -> Result<Vec<FlowExecution>>;
    
    /// Find executions by tenant
    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<FlowExecution>>;
    
    /// Find executions by user
    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<FlowExecution>>;
    
    /// Find executions by session
    async fn find_by_session(&self, session_id: &SessionId) -> Result<Vec<FlowExecution>>;
    
    /// Find executions by status
    async fn find_by_status(&self, tenant_id: &TenantId, status: &FlowExecutionStatus) -> Result<Vec<FlowExecution>>;
    
    /// Find executions within a time range
    async fn find_by_time_range(
        &self,
        tenant_id: &TenantId,
        start: DateTime<Utc>,
        end: DateTime<Utc>
    ) -> Result<Vec<FlowExecution>>;
    
    /// Save a flow execution (create or update)
    async fn save(&self, execution: &FlowExecution) -> Result<()>;
    
    /// Delete a flow execution
    async fn delete(&self, id: &FlowExecutionId) -> Result<()>;
    
    /// Count executions by flow
    async fn count_by_flow(&self, flow_id: &FlowId) -> Result<u64>;
    
    /// Count executions by tenant
    async fn count_by_tenant(&self, tenant_id: &TenantId) -> Result<u64>;
    
    /// Find executions with pagination
    async fn find_by_tenant_paginated(
        &self,
        tenant_id: &TenantId,
        offset: u64,
        limit: u64
    ) -> Result<Vec<FlowExecution>>;
    
    /// Find recent executions for a flow
    async fn find_recent_by_flow(
        &self,
        flow_id: &FlowId,
        limit: u64
    ) -> Result<Vec<FlowExecution>>;
    
    /// Find failed executions for analysis
    async fn find_failed_executions(
        &self,
        tenant_id: &TenantId,
        limit: u64
    ) -> Result<Vec<FlowExecution>>;
}