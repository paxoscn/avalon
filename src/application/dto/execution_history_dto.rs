use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Request to query execution history
#[derive(Debug, Clone, Deserialize)]
pub struct QueryExecutionsRequest {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub flow_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub status: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Response for execution history query
#[derive(Debug, Clone, Serialize)]
pub struct QueryExecutionsResponse {
    pub executions: Vec<ExecutionDto>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

/// Execution DTO
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionDto {
    pub id: Uuid,
    pub flow_id: Uuid,
    pub flow_version: i32,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
    pub status: String,
    pub input_data: Option<Value>,
    pub output_data: Option<Value>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
}

/// Execution step DTO
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionStepDto {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub step_name: String,
    pub step_type: String,
    pub status: String,
    pub input_data: Option<Value>,
    pub output_data: Option<Value>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
    pub metadata: Option<Value>,
}

/// Execution metrics DTO
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionMetricsDto {
    pub execution_id: Uuid,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub failed_steps: u32,
    pub skipped_steps: u32,
    pub total_execution_time_ms: i32,
    pub average_step_time_ms: i32,
    pub slowest_step: Option<String>,
    pub slowest_step_time_ms: Option<i32>,
}

/// Response for execution details
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionDetailsResponse {
    pub execution: ExecutionDto,
    pub steps: Vec<ExecutionStepDto>,
    pub metrics: ExecutionMetricsDto,
}
