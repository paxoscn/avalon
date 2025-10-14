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
        page_size: u64,
        flow_id: Option<Uuid>,
        user_id: Option<Uuid>,
        status: Option<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(Vec<FlowExecutionHistory>, u64)> {
        let offset = (page - 1) * page_size;

        let mut filter = ExecutionFilter::new(tenant_id).with_pagination(page_size, offset);

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
