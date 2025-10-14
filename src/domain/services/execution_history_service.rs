use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::entities::{ExecutionMetrics, ExecutionStep, FlowExecutionHistory};
use crate::domain::repositories::{ExecutionFilter, ExecutionHistoryRepository};
use crate::error::Result;

/// Domain service for execution history tracking
#[async_trait]
pub trait ExecutionHistoryService: Send + Sync {
    /// Start tracking a new execution
    async fn start_execution(
        &self,
        flow_id: Uuid,
        flow_version: i32,
        tenant_id: Uuid,
        user_id: Uuid,
        session_id: Option<Uuid>,
        input_data: Option<Value>,
    ) -> Result<Uuid>;

    /// Complete an execution successfully
    async fn complete_execution(&self, execution_id: Uuid, output_data: Option<Value>) -> Result<()>;

    /// Mark an execution as failed
    async fn fail_execution(&self, execution_id: Uuid, error_message: String) -> Result<()>;

    /// Start tracking an execution step
    async fn start_step(
        &self,
        execution_id: Uuid,
        step_name: String,
        step_type: String,
        input_data: Option<Value>,
    ) -> Result<Uuid>;

    /// Complete an execution step
    async fn complete_step(&self, step_id: Uuid, output_data: Option<Value>) -> Result<()>;

    /// Mark an execution step as failed
    async fn fail_step(&self, step_id: Uuid, error_message: String) -> Result<()>;

    /// Get execution by ID
    async fn get_execution(&self, execution_id: Uuid) -> Result<Option<FlowExecutionHistory>>;

    /// Get execution steps
    async fn get_execution_steps(&self, execution_id: Uuid) -> Result<Vec<ExecutionStep>>;

    /// Get execution metrics
    async fn get_execution_metrics(&self, execution_id: Uuid) -> Result<ExecutionMetrics>;

    /// Query executions with filters
    async fn query_executions(&self, filter: &ExecutionFilter) -> Result<Vec<FlowExecutionHistory>>;

    /// Count executions with filters
    async fn count_executions(&self, filter: &ExecutionFilter) -> Result<u64>;
}

/// Implementation of execution history service
pub struct ExecutionHistoryServiceImpl {
    execution_history_repository: Box<dyn ExecutionHistoryRepository>,
}

impl ExecutionHistoryServiceImpl {
    pub fn new(execution_history_repository: Box<dyn ExecutionHistoryRepository>) -> Self {
        Self {
            execution_history_repository,
        }
    }
}

#[async_trait]
impl ExecutionHistoryService for ExecutionHistoryServiceImpl {
    async fn start_execution(
        &self,
        flow_id: Uuid,
        flow_version: i32,
        tenant_id: Uuid,
        user_id: Uuid,
        session_id: Option<Uuid>,
        input_data: Option<Value>,
    ) -> Result<Uuid> {
        let mut execution = FlowExecutionHistory::new(
            flow_id,
            flow_version,
            tenant_id,
            user_id,
            session_id,
            input_data,
        );
        execution.start();

        let execution_id = execution.id;
        self.execution_history_repository
            .create_execution(&execution)
            .await?;

        Ok(execution_id)
    }

    async fn complete_execution(&self, execution_id: Uuid, output_data: Option<Value>) -> Result<()> {
        let mut execution = self
            .execution_history_repository
            .find_execution_by_id(execution_id)
            .await?
            .ok_or_else(|| crate::error::PlatformError::NotFound("Execution not found".to_string()))?;

        execution.complete(output_data);
        self.execution_history_repository
            .update_execution(&execution)
            .await
    }

    async fn fail_execution(&self, execution_id: Uuid, error_message: String) -> Result<()> {
        let mut execution = self
            .execution_history_repository
            .find_execution_by_id(execution_id)
            .await?
            .ok_or_else(|| crate::error::PlatformError::NotFound("Execution not found".to_string()))?;

        execution.fail(error_message);
        self.execution_history_repository
            .update_execution(&execution)
            .await
    }

    async fn start_step(
        &self,
        execution_id: Uuid,
        step_name: String,
        step_type: String,
        input_data: Option<Value>,
    ) -> Result<Uuid> {
        let mut step = ExecutionStep::new(execution_id, step_name, step_type);
        if let Some(input) = input_data {
            step = step.with_input(input);
        }
        step.start();

        let step_id = step.id;
        self.execution_history_repository.create_step(&step).await?;

        Ok(step_id)
    }

    async fn complete_step(&self, step_id: Uuid, output_data: Option<Value>) -> Result<()> {
        // Note: We need to find the step first, but we don't have a find_step_by_id method
        // For now, we'll need to get all steps and find the one we need
        // In a production system, you'd want to add a find_step_by_id method
        let steps = self
            .execution_history_repository
            .find_steps_by_execution_id(step_id)
            .await?;

        let mut step = steps
            .into_iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| crate::error::PlatformError::NotFound("Step not found".to_string()))?;

        step.complete(output_data);
        self.execution_history_repository.update_step(&step).await
    }

    async fn fail_step(&self, step_id: Uuid, error_message: String) -> Result<()> {
        let steps = self
            .execution_history_repository
            .find_steps_by_execution_id(step_id)
            .await?;

        let mut step = steps
            .into_iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| crate::error::PlatformError::NotFound("Step not found".to_string()))?;

        step.fail(error_message);
        self.execution_history_repository.update_step(&step).await
    }

    async fn get_execution(&self, execution_id: Uuid) -> Result<Option<FlowExecutionHistory>> {
        self.execution_history_repository
            .find_execution_by_id(execution_id)
            .await
    }

    async fn get_execution_steps(&self, execution_id: Uuid) -> Result<Vec<ExecutionStep>> {
        self.execution_history_repository
            .find_steps_by_execution_id(execution_id)
            .await
    }

    async fn get_execution_metrics(&self, execution_id: Uuid) -> Result<ExecutionMetrics> {
        self.execution_history_repository
            .get_execution_metrics(execution_id)
            .await
    }

    async fn query_executions(&self, filter: &ExecutionFilter) -> Result<Vec<FlowExecutionHistory>> {
        self.execution_history_repository
            .find_executions_with_filter(filter)
            .await
    }

    async fn count_executions(&self, filter: &ExecutionFilter) -> Result<u64> {
        self.execution_history_repository
            .count_executions_with_filter(filter)
            .await
    }
}
