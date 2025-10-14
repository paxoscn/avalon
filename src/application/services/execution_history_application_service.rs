use chrono::{DateTime, Utc};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::{ExecutionMetrics, ExecutionStep, FlowExecutionHistory};
use crate::domain::repositories::ExecutionFilter;
use crate::domain::services::ExecutionHistoryService;
use crate::error::Result;

/// Application service for execution history
pub struct ExecutionHistoryApplicationService {
    execution_history_service: Arc<dyn ExecutionHistoryService>,
}

impl ExecutionHistoryApplicationService {
    pub fn new(execution_history_service: Arc<dyn ExecutionHistoryService>) -> Self {
        Self {
            execution_history_service,
        }
    }

    /// Start tracking a new execution
    pub async fn start_execution(
        &self,
        flow_id: Uuid,
        flow_version: i32,
        tenant_id: Uuid,
        user_id: Uuid,
        session_id: Option<Uuid>,
        input_data: Option<Value>,
    ) -> Result<Uuid> {
        self.execution_history_service
            .start_execution(flow_id, flow_version, tenant_id, user_id, session_id, input_data)
            .await
    }

    /// Complete an execution
    pub async fn complete_execution(&self, execution_id: Uuid, output_data: Option<Value>) -> Result<()> {
        self.execution_history_service
            .complete_execution(execution_id, output_data)
            .await
    }

    /// Fail an execution
    pub async fn fail_execution(&self, execution_id: Uuid, error_message: String) -> Result<()> {
        self.execution_history_service
            .fail_execution(execution_id, error_message)
            .await
    }

    /// Start tracking an execution step
    pub async fn start_step(
        &self,
        execution_id: Uuid,
        step_name: String,
        step_type: String,
        input_data: Option<Value>,
    ) -> Result<Uuid> {
        self.execution_history_service
            .start_step(execution_id, step_name, step_type, input_data)
            .await
    }

    /// Complete an execution step
    pub async fn complete_step(&self, step_id: Uuid, output_data: Option<Value>) -> Result<()> {
        self.execution_history_service
            .complete_step(step_id, output_data)
            .await
    }

    /// Fail an execution step
    pub async fn fail_step(&self, step_id: Uuid, error_message: String) -> Result<()> {
        self.execution_history_service
            .fail_step(step_id, error_message)
            .await
    }

    /// Get execution by ID
    pub async fn get_execution(&self, execution_id: Uuid) -> Result<Option<FlowExecutionHistory>> {
        self.execution_history_service
            .get_execution(execution_id)
            .await
    }

    /// Get execution steps
    pub async fn get_execution_steps(&self, execution_id: Uuid) -> Result<Vec<ExecutionStep>> {
        self.execution_history_service
            .get_execution_steps(execution_id)
            .await
    }

    /// Get execution metrics
    pub async fn get_execution_metrics(&self, execution_id: Uuid) -> Result<ExecutionMetrics> {
        self.execution_history_service
            .get_execution_metrics(execution_id)
            .await
    }

    /// Query executions with pagination
    pub async fn query_executions_paginated(
        &self,
        tenant_id: Uuid,
        page: u64,
        limit: u64,
        flow_id: Option<Uuid>,
        user_id: Option<Uuid>,
        status: Option<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(Vec<FlowExecutionHistory>, u64)> {
        let offset = page * limit;

        let mut filter = ExecutionFilter::new(tenant_id).with_pagination(limit, offset);

        if let Some(fid) = flow_id {
            filter = filter.with_flow_id(fid);
        }

        if let Some(uid) = user_id {
            filter = filter.with_user_id(uid);
        }

        if let Some(st) = status {
            filter = filter.with_status(st);
        }

        if let Some(start) = start_date {
            if let Some(end) = end_date {
                filter = filter.with_date_range(start, end);
            }
        }

        let executions = self.execution_history_service.query_executions(&filter).await?;
        let total = self.execution_history_service.count_executions(&filter).await?;

        Ok((executions, total))
    }

    /// Get execution with steps and metrics
    pub async fn get_execution_details(
        &self,
        execution_id: Uuid,
    ) -> Result<Option<(FlowExecutionHistory, Vec<ExecutionStep>, ExecutionMetrics)>> {
        let execution = self.get_execution(execution_id).await?;

        if let Some(exec) = execution {
            let steps = self.get_execution_steps(execution_id).await?;
            let metrics = self.get_execution_metrics(execution_id).await?;
            Ok(Some((exec, steps, metrics)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::services::ExecutionHistoryService;
    use crate::domain::repositories::ExecutionFilter;
    use async_trait::async_trait;
    use mockall::mock;
    use std::sync::Arc;

    mock! {
        ExecutionHistoryServiceImpl {}

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
            ) -> Result<Uuid>;

            async fn complete_execution(&self, execution_id: Uuid, output_data: Option<Value>) -> Result<()>;
            async fn fail_execution(&self, execution_id: Uuid, error_message: String) -> Result<()>;
            async fn start_step(
                &self,
                execution_id: Uuid,
                step_name: String,
                step_type: String,
                input_data: Option<Value>,
            ) -> Result<Uuid>;
            async fn complete_step(&self, step_id: Uuid, output_data: Option<Value>) -> Result<()>;
            async fn fail_step(&self, step_id: Uuid, error_message: String) -> Result<()>;
            async fn get_execution(&self, execution_id: Uuid) -> Result<Option<FlowExecutionHistory>>;
            async fn get_execution_steps(&self, execution_id: Uuid) -> Result<Vec<ExecutionStep>>;
            async fn get_execution_metrics(&self, execution_id: Uuid) -> Result<ExecutionMetrics>;
            async fn query_executions(&self, filter: &ExecutionFilter) -> Result<Vec<FlowExecutionHistory>>;
            async fn count_executions(&self, filter: &ExecutionFilter) -> Result<u64>;
        }
    }

    #[tokio::test]
    async fn test_query_executions_paginated_zero_based() {
        let mut mock_service = MockExecutionHistoryServiceImpl::new();
        let tenant_id = Uuid::new_v4();

        // Mock query_executions to return empty vec
        mock_service
            .expect_query_executions()
            .times(1)
            .returning(|_| Ok(vec![]));

        // Mock count_executions to return total count
        mock_service
            .expect_count_executions()
            .times(1)
            .returning(|_| Ok(30));

        let service = ExecutionHistoryApplicationService::new(Arc::new(mock_service));

        // Test page 0 (first page) with limit 10
        let result = service
            .query_executions_paginated(tenant_id, 0, 10, None, None, None, None, None)
            .await;

        assert!(result.is_ok());
        let (executions, total) = result.unwrap();
        assert_eq!(executions.len(), 0);
        assert_eq!(total, 30);
    }

    #[tokio::test]
    async fn test_query_executions_paginated_offset_calculation() {
        let mut mock_service = MockExecutionHistoryServiceImpl::new();
        let tenant_id = Uuid::new_v4();

        // Verify that offset is calculated as page * limit
        mock_service
            .expect_query_executions()
            .times(1)
            .withf(|filter: &ExecutionFilter| {
                // For page=3, limit=10, offset should be 30
                filter.limit == Some(10) && filter.offset == Some(30)
            })
            .returning(|_| Ok(vec![]));

        mock_service
            .expect_count_executions()
            .times(1)
            .returning(|_| Ok(100));

        let service = ExecutionHistoryApplicationService::new(Arc::new(mock_service));

        // Test page 3 with limit 10 (offset should be 3 * 10 = 30)
        let result = service
            .query_executions_paginated(tenant_id, 3, 10, None, None, None, None, None)
            .await;

        assert!(result.is_ok());
        let (_, total) = result.unwrap();
        assert_eq!(total, 100);
    }

    #[tokio::test]
    async fn test_query_executions_paginated_total_count_accuracy() {
        let mut mock_service = MockExecutionHistoryServiceImpl::new();
        let tenant_id = Uuid::new_v4();

        mock_service
            .expect_query_executions()
            .times(1)
            .returning(|_| Ok(vec![]));

        // Verify total count is returned accurately
        mock_service
            .expect_count_executions()
            .times(1)
            .returning(|_| Ok(42));

        let service = ExecutionHistoryApplicationService::new(Arc::new(mock_service));

        let result = service
            .query_executions_paginated(tenant_id, 0, 20, None, None, None, None, None)
            .await;

        assert!(result.is_ok());
        let (_, total) = result.unwrap();
        assert_eq!(total, 42);
    }
}
