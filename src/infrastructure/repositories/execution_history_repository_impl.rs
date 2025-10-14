use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::{
    ExecutionMetrics, ExecutionStatus, ExecutionStep, FlowExecutionHistory, StepStatus,
};
use crate::domain::repositories::{ExecutionFilter, ExecutionHistoryRepository};
use crate::error::{PlatformError, Result};
use crate::infrastructure::database::entities::{execution_step, flow_execution};

pub struct ExecutionHistoryRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl ExecutionHistoryRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn execution_to_domain(&self, model: flow_execution::Model) -> FlowExecutionHistory {
        FlowExecutionHistory {
            id: model.id,
            flow_id: model.flow_id,
            flow_version: model.flow_version,
            tenant_id: model.tenant_id,
            user_id: model.user_id,
            session_id: model.session_id,
            status: self.db_status_to_domain(&model.status),
            input_data: model.input_data,
            output_data: model.output_data,
            error_message: model.error_message,
            started_at: model.started_at,
            completed_at: model.completed_at,
            execution_time_ms: model.execution_time_ms,
        }
    }

    fn execution_to_active_model(&self, execution: &FlowExecutionHistory) -> flow_execution::ActiveModel {
        flow_execution::ActiveModel {
            id: Set(execution.id),
            flow_id: Set(execution.flow_id),
            flow_version: Set(execution.flow_version),
            tenant_id: Set(execution.tenant_id),
            user_id: Set(execution.user_id),
            session_id: Set(execution.session_id),
            status: Set(self.domain_status_to_db(&execution.status)),
            input_data: Set(execution.input_data.clone()),
            output_data: Set(execution.output_data.clone()),
            error_message: Set(execution.error_message.clone()),
            started_at: Set(execution.started_at),
            completed_at: Set(execution.completed_at),
            execution_time_ms: Set(execution.execution_time_ms),
        }
    }

    fn step_to_domain(&self, model: execution_step::Model) -> ExecutionStep {
        ExecutionStep {
            id: model.id,
            execution_id: model.execution_id,
            step_name: model.step_name,
            step_type: model.step_type,
            status: self.db_step_status_to_domain(&model.status),
            input_data: model.input_data,
            output_data: model.output_data,
            error_message: model.error_message,
            started_at: model.started_at,
            completed_at: model.completed_at,
            execution_time_ms: model.execution_time_ms,
            metadata: model.metadata,
        }
    }

    fn step_to_active_model(&self, step: &ExecutionStep) -> execution_step::ActiveModel {
        execution_step::ActiveModel {
            id: Set(step.id),
            execution_id: Set(step.execution_id),
            step_name: Set(step.step_name.clone()),
            step_type: Set(step.step_type.clone()),
            status: Set(self.domain_step_status_to_db(&step.status)),
            input_data: Set(step.input_data.clone()),
            output_data: Set(step.output_data.clone()),
            error_message: Set(step.error_message.clone()),
            started_at: Set(step.started_at),
            completed_at: Set(step.completed_at),
            execution_time_ms: Set(step.execution_time_ms),
            metadata: Set(step.metadata.clone()),
        }
    }

    fn db_status_to_domain(&self, status: &flow_execution::ExecutionStatus) -> ExecutionStatus {
        match status {
            flow_execution::ExecutionStatus::Pending => ExecutionStatus::Pending,
            flow_execution::ExecutionStatus::Running => ExecutionStatus::Running,
            flow_execution::ExecutionStatus::Completed => ExecutionStatus::Completed,
            flow_execution::ExecutionStatus::Failed => ExecutionStatus::Failed,
            flow_execution::ExecutionStatus::Cancelled => ExecutionStatus::Cancelled,
        }
    }

    fn domain_status_to_db(&self, status: &ExecutionStatus) -> flow_execution::ExecutionStatus {
        match status {
            ExecutionStatus::Pending => flow_execution::ExecutionStatus::Pending,
            ExecutionStatus::Running => flow_execution::ExecutionStatus::Running,
            ExecutionStatus::Completed => flow_execution::ExecutionStatus::Completed,
            ExecutionStatus::Failed => flow_execution::ExecutionStatus::Failed,
            ExecutionStatus::Cancelled => flow_execution::ExecutionStatus::Cancelled,
        }
    }

    fn db_step_status_to_domain(&self, status: &execution_step::StepStatus) -> StepStatus {
        match status {
            execution_step::StepStatus::Pending => StepStatus::Pending,
            execution_step::StepStatus::Running => StepStatus::Running,
            execution_step::StepStatus::Completed => StepStatus::Completed,
            execution_step::StepStatus::Failed => StepStatus::Failed,
            execution_step::StepStatus::Skipped => StepStatus::Skipped,
        }
    }

    fn domain_step_status_to_db(&self, status: &StepStatus) -> execution_step::StepStatus {
        match status {
            StepStatus::Pending => execution_step::StepStatus::Pending,
            StepStatus::Running => execution_step::StepStatus::Running,
            StepStatus::Completed => execution_step::StepStatus::Completed,
            StepStatus::Failed => execution_step::StepStatus::Failed,
            StepStatus::Skipped => execution_step::StepStatus::Skipped,
        }
    }
}

#[async_trait]
impl ExecutionHistoryRepository for ExecutionHistoryRepositoryImpl {
    async fn create_execution(&self, execution: &FlowExecutionHistory) -> Result<()> {
        let active_model = self.execution_to_active_model(execution);
        flow_execution::Entity::insert(active_model)
            .exec(self.db.as_ref())
            .await
            .map_err(PlatformError::from)?;
        Ok(())
    }

    async fn update_execution(&self, execution: &FlowExecutionHistory) -> Result<()> {
        let active_model = self.execution_to_active_model(execution);
        flow_execution::Entity::update(active_model)
            .exec(self.db.as_ref())
            .await
            .map_err(PlatformError::from)?;
        Ok(())
    }

    async fn find_execution_by_id(&self, id: Uuid) -> Result<Option<FlowExecutionHistory>> {
        let model = flow_execution::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(PlatformError::from)?;

        Ok(model.map(|m| self.execution_to_domain(m)))
    }

    async fn find_executions_with_filter(&self, filter: &ExecutionFilter) -> Result<Vec<FlowExecutionHistory>> {
        let mut query = flow_execution::Entity::find()
            .filter(flow_execution::Column::TenantId.eq(filter.tenant_id));

        if let Some(flow_id) = filter.flow_id {
            query = query.filter(flow_execution::Column::FlowId.eq(flow_id));
        }

        if let Some(user_id) = filter.user_id {
            query = query.filter(flow_execution::Column::UserId.eq(user_id));
        }

        if let Some(session_id) = filter.session_id {
            query = query.filter(flow_execution::Column::SessionId.eq(session_id));
        }

        if let Some(ref status) = filter.status {
            query = query.filter(flow_execution::Column::Status.eq(status.as_str()));
        }

        if let Some(start_date) = filter.start_date {
            query = query.filter(flow_execution::Column::StartedAt.gte(start_date));
        }

        if let Some(end_date) = filter.end_date {
            query = query.filter(flow_execution::Column::StartedAt.lte(end_date));
        }

        query = query.order_by_desc(flow_execution::Column::StartedAt);

        if let Some(limit) = filter.limit {
            query = query.limit(limit);
        }

        if let Some(offset) = filter.offset {
            query = query.offset(offset);
        }

        let models = query
            .all(self.db.as_ref())
            .await
            .map_err(PlatformError::from)?;

        Ok(models.into_iter().map(|m| self.execution_to_domain(m)).collect())
    }

    async fn count_executions_with_filter(&self, filter: &ExecutionFilter) -> Result<u64> {
        let mut query = flow_execution::Entity::find()
            .filter(flow_execution::Column::TenantId.eq(filter.tenant_id));

        if let Some(flow_id) = filter.flow_id {
            query = query.filter(flow_execution::Column::FlowId.eq(flow_id));
        }

        if let Some(user_id) = filter.user_id {
            query = query.filter(flow_execution::Column::UserId.eq(user_id));
        }

        if let Some(session_id) = filter.session_id {
            query = query.filter(flow_execution::Column::SessionId.eq(session_id));
        }

        if let Some(ref status) = filter.status {
            query = query.filter(flow_execution::Column::Status.eq(status.as_str()));
        }

        if let Some(start_date) = filter.start_date {
            query = query.filter(flow_execution::Column::StartedAt.gte(start_date));
        }

        if let Some(end_date) = filter.end_date {
            query = query.filter(flow_execution::Column::StartedAt.lte(end_date));
        }

        query
            .count(self.db.as_ref())
            .await
            .map_err(PlatformError::from)
    }

    async fn create_step(&self, step: &ExecutionStep) -> Result<()> {
        let active_model = self.step_to_active_model(step);
        execution_step::Entity::insert(active_model)
            .exec(self.db.as_ref())
            .await
            .map_err(PlatformError::from)?;
        Ok(())
    }

    async fn update_step(&self, step: &ExecutionStep) -> Result<()> {
        let active_model = self.step_to_active_model(step);
        execution_step::Entity::update(active_model)
            .exec(self.db.as_ref())
            .await
            .map_err(PlatformError::from)?;
        Ok(())
    }

    async fn find_steps_by_execution_id(&self, execution_id: Uuid) -> Result<Vec<ExecutionStep>> {
        let models = execution_step::Entity::find()
            .filter(execution_step::Column::ExecutionId.eq(execution_id))
            .order_by_asc(execution_step::Column::StartedAt)
            .all(self.db.as_ref())
            .await
            .map_err(PlatformError::from)?;

        Ok(models.into_iter().map(|m| self.step_to_domain(m)).collect())
    }

    async fn get_execution_metrics(&self, execution_id: Uuid) -> Result<ExecutionMetrics> {
        let steps = self.find_steps_by_execution_id(execution_id).await?;
        Ok(ExecutionMetrics::from_steps(execution_id, &steps))
    }

    async fn delete_executions_older_than(&self, date: DateTime<Utc>) -> Result<u64> {
        let result = flow_execution::Entity::delete_many()
            .filter(flow_execution::Column::StartedAt.lt(date))
            .exec(self.db.as_ref())
            .await
            .map_err(PlatformError::from)?;

        Ok(result.rows_affected)
    }
}
