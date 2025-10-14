use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::services::{SessionApplicationService, AuditApplicationService, ExecutionHistoryApplicationService},
    domain::{
        entities::{AuditAction, ResourceType},
        value_objects::{SessionId, ChatMessage, MessageRole},
    },
    error::Result,
    presentation::extractors::AuthenticatedUser,
};

// Session DTOs
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMessageRequest {
    pub role: String,
    pub content: String,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct SetContextRequest {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub metadata: Option<Value>,
    pub created_at: String,
}

// Audit DTOs
#[derive(Debug, Deserialize)]
pub struct QueryAuditLogsRequest {
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(default)]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

fn default_page_size() -> u64 {
    50
}

#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub id: String,
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: Option<Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct AuditLogsListResponse {
    pub logs: Vec<AuditLogResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

// Execution History DTOs
#[derive(Debug, Deserialize)]
pub struct QueryExecutionsRequest {
    pub flow_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(default)]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

#[derive(Debug, Serialize)]
pub struct ExecutionHistoryResponse {
    pub id: String,
    pub flow_id: String,
    pub flow_version: i32,
    pub tenant_id: String,
    pub user_id: String,
    pub session_id: Option<String>,
    pub status: String,
    pub input_data: Option<Value>,
    pub output_data: Option<Value>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub execution_time_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionStepResponse {
    pub id: String,
    pub execution_id: String,
    pub step_name: String,
    pub step_type: String,
    pub status: String,
    pub input_data: Option<Value>,
    pub output_data: Option<Value>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub execution_time_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionDetailsResponse {
    pub execution: ExecutionHistoryResponse,
    pub steps: Vec<ExecutionStepResponse>,
    pub metrics: ExecutionMetricsResponse,
}

#[derive(Debug, Serialize)]
pub struct ExecutionMetricsResponse {
    pub total_steps: u32,
    pub successful_steps: u32,
    pub failed_steps: u32,
    pub total_execution_time_ms: i64,
    pub average_step_time_ms: i64,
}

#[derive(Debug, Serialize)]
pub struct ExecutionsListResponse {
    pub executions: Vec<ExecutionHistoryResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

// Session Handlers
pub async fn create_session(
    State(service): State<Arc<SessionApplicationService>>,
    user: AuthenticatedUser,
    Json(req): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse> {
    let session = service.create_session(user.tenant_id, user.user_id, req.title).await?;
    Ok((StatusCode::CREATED, Json(session_to_response(&session))))
}

pub async fn get_session(
    State(service): State<Arc<SessionApplicationService>>,
    user: AuthenticatedUser,
    Path(session_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let session = service.get_session(&SessionId(session_id), &user.tenant_id, &user.user_id).await?;
    Ok(Json(session_to_response(&session)))
}

pub async fn list_sessions(
    State(service): State<Arc<SessionApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<QueryAuditLogsRequest>,
) -> Result<impl IntoResponse> {
    let offset = query.page * query.page_size;
    let sessions = service.list_user_sessions(&user.user_id, offset, query.page_size).await?;
    let total = service.count_user_sessions(&user.user_id).await?;
    
    let response = serde_json::json!({
        "sessions": sessions.iter().map(session_to_response).collect::<Vec<_>>(),
        "total": total,
        "page": query.page,
        "page_size": query.page_size,
    });

    Ok(Json(response))
}

pub async fn update_session(
    State(service): State<Arc<SessionApplicationService>>,
    user: AuthenticatedUser,
    Path(session_id): Path<Uuid>,
    Json(req): Json<UpdateSessionRequest>,
) -> Result<impl IntoResponse> {
    let session = service.update_session_title(&SessionId(session_id), &user.tenant_id, &user.user_id, req.title).await?;
    Ok(Json(session_to_response(&session)))
}

pub async fn delete_session(
    State(service): State<Arc<SessionApplicationService>>,
    user: AuthenticatedUser,
    Path(session_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service.delete_session(&SessionId(session_id), &user.tenant_id, &user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_message(
    State(service): State<Arc<SessionApplicationService>>,
    user: AuthenticatedUser,
    Path(session_id): Path<Uuid>,
    Json(req): Json<AddMessageRequest>,
) -> Result<impl IntoResponse> {
    let role = parse_message_role(&req.role)?;
    
    // Parse metadata if provided
    let metadata = req.metadata.map(|m| {
        serde_json::from_value(m).unwrap_or_default()
    });
    
    let chat_message = ChatMessage {
        role,
        content: req.content,
        metadata,
        timestamp: chrono::Utc::now(),
    };

    let message = service.add_message(&SessionId(session_id), &user.tenant_id, &user.user_id, chat_message).await?;
    Ok((StatusCode::CREATED, Json(message_to_response(&message))))
}

pub async fn set_context(
    State(service): State<Arc<SessionApplicationService>>,
    user: AuthenticatedUser,
    Path(session_id): Path<Uuid>,
    Json(req): Json<SetContextRequest>,
) -> Result<impl IntoResponse> {
    service.set_context_variable(&SessionId(session_id), &user.tenant_id, &user.user_id, req.key, req.value).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_context(
    State(service): State<Arc<SessionApplicationService>>,
    user: AuthenticatedUser,
    Path((session_id, key)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse> {
    let value = service.get_context_variable(&SessionId(session_id), &user.tenant_id, &user.user_id, &key).await?;
    Ok(Json(serde_json::json!({ "value": value })))
}

// Audit Handlers
pub async fn query_audit_logs(
    State(service): State<Arc<AuditApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<QueryAuditLogsRequest>,
) -> Result<impl IntoResponse> {
    let action = query.action.as_ref().and_then(|a| parse_audit_action(a).ok());
    let resource_type = query.resource_type.as_ref().and_then(|rt| parse_resource_type(rt).ok());
    let start_date = query.start_date.as_ref().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));
    let end_date = query.end_date.as_ref().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));

    let (logs, total) = service.query_logs_paginated(
        user.tenant_id.0,
        query.page + 1, // Service expects 1-based page
        query.page_size,
        query.user_id,
        action,
        resource_type,
        start_date,
        end_date,
    ).await?;

    let response = AuditLogsListResponse {
        logs: logs.iter().map(audit_log_to_response).collect(),
        total,
        page: query.page,
        page_size: query.page_size,
    };

    Ok(Json(response))
}

pub async fn get_audit_statistics(
    State(service): State<Arc<AuditApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<QueryAuditLogsRequest>,
) -> Result<impl IntoResponse> {
    let start_date = query.start_date.as_ref().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));
    let end_date = query.end_date.as_ref().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));

    let stats = service.get_statistics(user.tenant_id.0, start_date, end_date).await?;
    
    // Convert stats to JSON manually since it doesn't implement Serialize
    let response = serde_json::json!({
        "total_count": stats.total_count,
        "action_counts": stats.action_counts,
        "resource_type_counts": stats.resource_type_counts,
        "user_activity": stats.user_activity,
    });
    
    Ok(Json(response))
}

// Execution History Handlers
pub async fn query_executions(
    State(service): State<Arc<ExecutionHistoryApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<QueryExecutionsRequest>,
) -> Result<impl IntoResponse> {
    let start_date = query.start_date.as_ref().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));
    let end_date = query.end_date.as_ref().and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));

    let (executions, total) = service.query_executions_paginated(
        user.tenant_id.0,
        query.page + 1, // Service expects 1-based page
        query.page_size,
        query.flow_id,
        query.user_id,
        query.status,
        start_date,
        end_date,
    ).await?;

    let response = ExecutionsListResponse {
        executions: executions.iter().map(execution_to_response).collect(),
        total,
        page: query.page,
        page_size: query.page_size,
    };

    Ok(Json(response))
}

pub async fn get_execution_details(
    State(service): State<Arc<ExecutionHistoryApplicationService>>,
    _user: AuthenticatedUser,
    Path(execution_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let details = service.get_execution_details(execution_id).await?
        .ok_or_else(|| crate::error::PlatformError::NotFound("Execution not found".to_string()))?;

    let response = ExecutionDetailsResponse {
        execution: execution_to_response(&details.0),
        steps: details.1.iter().map(step_to_response).collect(),
        metrics: metrics_to_response(&details.2),
    };

    Ok(Json(response))
}

// Helper functions
fn session_to_response(session: &crate::domain::entities::ChatSession) -> SessionResponse {
    SessionResponse {
        id: session.id.0.to_string(),
        tenant_id: session.tenant_id.0.to_string(),
        user_id: session.user_id.0.to_string(),
        title: session.title.clone(),
        created_at: session.created_at.to_rfc3339(),
        updated_at: session.updated_at.to_rfc3339(),
    }
}

fn message_to_response(message: &crate::domain::entities::Message) -> MessageResponse {
    MessageResponse {
        id: message.id.0.to_string(),
        session_id: message.session_id.0.to_string(),
        role: format!("{:?}", message.message.role),
        content: message.message.content.clone(),
        metadata: message.message.metadata.as_ref().map(|m| serde_json::to_value(m).unwrap_or(Value::Null)),
        created_at: message.message.timestamp.to_rfc3339(),
    }
}

fn audit_log_to_response(log: &crate::domain::entities::AuditLog) -> AuditLogResponse {
    AuditLogResponse {
        id: log.id.to_string(),
        tenant_id: log.tenant_id.to_string(),
        user_id: log.user_id.map(|id| id.to_string()),
        action: format!("{:?}", log.action),
        resource_type: format!("{:?}", log.resource_type),
        resource_id: log.resource_id.map(|id| id.to_string()),
        details: log.details.clone(),
        ip_address: log.ip_address.clone(),
        user_agent: log.user_agent.clone(),
        created_at: log.created_at.to_rfc3339(),
    }
}

fn execution_to_response(exec: &crate::domain::entities::FlowExecutionHistory) -> ExecutionHistoryResponse {
    ExecutionHistoryResponse {
        id: exec.id.to_string(),
        flow_id: exec.flow_id.to_string(),
        flow_version: exec.flow_version,
        tenant_id: exec.tenant_id.to_string(),
        user_id: exec.user_id.to_string(),
        session_id: exec.session_id.map(|id| id.to_string()),
        status: format!("{:?}", exec.status),
        input_data: exec.input_data.clone(),
        output_data: exec.output_data.clone(),
        error_message: exec.error_message.clone(),
        started_at: exec.started_at.to_rfc3339(),
        completed_at: exec.completed_at.map(|dt| dt.to_rfc3339()),
        execution_time_ms: exec.execution_time_ms.map(|t| t as i64),
    }
}

fn step_to_response(step: &crate::domain::entities::ExecutionStep) -> ExecutionStepResponse {
    ExecutionStepResponse {
        id: step.id.to_string(),
        execution_id: step.execution_id.to_string(),
        step_name: step.step_name.clone(),
        step_type: step.step_type.clone(),
        status: format!("{:?}", step.status),
        input_data: step.input_data.clone(),
        output_data: step.output_data.clone(),
        error_message: step.error_message.clone(),
        started_at: step.started_at.to_rfc3339(),
        completed_at: step.completed_at.map(|dt| dt.to_rfc3339()),
        execution_time_ms: step.execution_time_ms.map(|t| t as i64),
    }
}

fn metrics_to_response(metrics: &crate::domain::entities::ExecutionMetrics) -> ExecutionMetricsResponse {
    ExecutionMetricsResponse {
        total_steps: metrics.total_steps,
        successful_steps: metrics.completed_steps,
        failed_steps: metrics.failed_steps,
        total_execution_time_ms: metrics.total_execution_time_ms as i64,
        average_step_time_ms: metrics.average_step_time_ms as i64,
    }
}

fn parse_message_role(role: &str) -> Result<MessageRole> {
    match role.to_lowercase().as_str() {
        "user" => Ok(MessageRole::User),
        "assistant" => Ok(MessageRole::Assistant),
        "system" => Ok(MessageRole::System),
        _ => Err(crate::error::PlatformError::ValidationError(
            format!("Invalid message role: {}", role)
        )),
    }
}

fn parse_audit_action(action: &str) -> Result<AuditAction> {
    match action.to_uppercase().as_str() {
        "CREATE" => Ok(AuditAction::Create),
        "UPDATE" => Ok(AuditAction::Update),
        "DELETE" => Ok(AuditAction::Delete),
        "EXECUTE" => Ok(AuditAction::Execute),
        _ => Err(crate::error::PlatformError::ValidationError(
            format!("Invalid audit action: {}", action)
        )),
    }
}

fn parse_resource_type(resource_type: &str) -> Result<ResourceType> {
    match resource_type.to_lowercase().as_str() {
        "flow" => Ok(ResourceType::Flow),
        "session" => Ok(ResourceType::Session),
        "user" => Ok(ResourceType::User),
        _ => Err(crate::error::PlatformError::ValidationError(
            format!("Invalid resource type: {}", resource_type)
        )),
    }
}
