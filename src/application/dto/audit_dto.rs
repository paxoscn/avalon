use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Request to query audit logs
#[derive(Debug, Clone, Deserialize)]
pub struct QueryAuditLogsRequest {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Response for audit log query
#[derive(Debug, Clone, Serialize)]
pub struct QueryAuditLogsResponse {
    pub logs: Vec<AuditLogDto>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

/// Audit log DTO
#[derive(Debug, Clone, Serialize)]
pub struct AuditLogDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub details: Option<Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request to get audit statistics
#[derive(Debug, Clone, Deserialize)]
pub struct GetAuditStatisticsRequest {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Response for audit statistics
#[derive(Debug, Clone, Serialize)]
pub struct AuditStatisticsDto {
    pub total_count: u64,
    pub action_counts: Vec<ActionCount>,
    pub resource_type_counts: Vec<ResourceTypeCount>,
    pub user_activity: Vec<UserActivity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionCount {
    pub action: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceTypeCount {
    pub resource_type: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserActivity {
    pub user_id: Uuid,
    pub count: u64,
}

/// Request to export audit logs
#[derive(Debug, Clone, Deserialize)]
pub struct ExportAuditLogsRequest {
    pub format: ExportFormat,
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
}
