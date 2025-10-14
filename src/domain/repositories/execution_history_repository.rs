use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::{ExecutionMetrics, ExecutionStep, FlowExecutionHistory};
use crate::error::Result;

/// Query filters for flow executions
#[derive(Debug, Clone, Default)]
pub struct ExecutionFilter {
    pub tenant_id: Uuid,
    pub flow_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub status: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl ExecutionFilter {
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            tenant_id,
            ..Default::default()
        }
    }

    pub fn with_flow_id(mut self, flow_id: Uuid) -> Self {
        self.flow_id = Some(flow_id);
        self
    }

    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_session_id(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }

    pub fn with_pagination(mut self, limit: u64, offset: u64) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

/// Repository interface for execution history
#[async_trait]
pub trait ExecutionHistoryRepository: Send + Sync {
    /// Create a new flow execution record
    async fn create_execution(&self, execution: &FlowExecutionHistory) -> Result<()>;

    /// Update an existing flow execution
    async fn update_execution(&self, execution: &FlowExecutionHistory) -> Result<()>;

    /// Find execution by ID
    async fn find_execution_by_id(&self, id: Uuid) -> Result<Option<FlowExecutionHistory>>;

    /// Find executions with filters
    async fn find_executions_with_filter(&self, filter: &ExecutionFilter) -> Result<Vec<FlowExecutionHistory>>;

    /// Count executions with filters
    async fn count_executions_with_filter(&self, filter: &ExecutionFilter) -> Result<u64>;

    /// Create an execution step
    async fn create_step(&self, step: &ExecutionStep) -> Result<()>;

    /// Update an execution step
    async fn update_step(&self, step: &ExecutionStep) -> Result<()>;

    /// Find steps for an execution
    async fn find_steps_by_execution_id(&self, execution_id: Uuid) -> Result<Vec<ExecutionStep>>;

    /// Get execution metrics
    async fn get_execution_metrics(&self, execution_id: Uuid) -> Result<ExecutionMetrics>;

    /// Delete old executions (for cleanup)
    async fn delete_executions_older_than(&self, date: DateTime<Utc>) -> Result<u64>;
}
