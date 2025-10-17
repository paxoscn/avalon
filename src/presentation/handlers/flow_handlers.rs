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
    application::services::FlowApplicationService,
    domain::value_objects::{FlowId, SessionId, FlowExecutionId, FlowDefinition},
    error::{PlatformError, Result},
    presentation::extractors::AuthenticatedUser,
};

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct CreateFlowRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImportDslRequest {
    pub name: String,
    pub dsl: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteFlowRequest {
    pub session_id: Option<Uuid>,
    pub input_data: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVersionRequest {
    pub definition: Value,
    pub change_log: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RollbackRequest {
    pub target_version: i32,
}

#[derive(Debug, Deserialize)]
pub struct ListFlowsQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_page() -> u64 {
    1
}

fn default_limit() -> u64 {
    20
}

#[derive(Debug, Deserialize)]
pub struct ListExecutionsQuery {
    pub flow_id: Option<Uuid>,
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

#[derive(Debug, Serialize)]
pub struct FlowResponse {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub description: Option<String>,
    pub current_version: i32,
    pub status: String,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct FlowListResponse {
    pub flows: Vec<FlowResponse>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct FlowVersionResponse {
    pub id: String,
    pub flow_id: String,
    pub version: i32,
    pub definition: Value,
    pub change_log: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct FlowExecutionResponse {
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
    pub execution_time_ms: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionListResponse {
    pub executions: Vec<FlowExecutionResponse>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct ImportDslResponse {
    pub flow: FlowResponse,
    pub validation: ValidationResultResponse,
}

#[derive(Debug, Serialize)]
pub struct ValidationResultResponse {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// Handlers
pub async fn create_flow(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Json(req): Json<CreateFlowRequest>,
) -> Result<impl IntoResponse> {
    let flow = service.create_flow(
        user.tenant_id,
        req.name,
        req.description,
        user.user_id,
    ).await?;

    Ok((StatusCode::CREATED, Json(flow_to_response(&flow))))
}

pub async fn get_flow(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Path(flow_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let flow = service.get_flow(FlowId(flow_id), user.tenant_id).await?;
    Ok(Json(flow_to_response(&flow)))
}

pub async fn list_flows(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<ListFlowsQuery>,
) -> Result<impl IntoResponse> {
    // Convert from 1-based (API) to 0-based (internal)
    let page = query.page.saturating_sub(1);
    let limit = query.limit;
    
    let (flows, total) = service.list_flows(user.tenant_id, page, limit).await?;
    
    // Calculate total_pages
    let total_pages = if limit > 0 {
        (total + limit - 1) / limit
    } else {
        0
    };
    
    let response = FlowListResponse {
        flows: flows.iter().map(flow_to_response).collect(),
        total,
        page: page + 1,  // Convert back to 1-based for API response
        limit,
        total_pages,
    };

    Ok(Json(response))
}

pub async fn update_flow(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Path(flow_id): Path<Uuid>,
    Json(req): Json<UpdateFlowRequest>,
) -> Result<impl IntoResponse> {
    let flow = service.update_flow(
        FlowId(flow_id),
        user.tenant_id,
        req.name,
        req.description,
    ).await?;

    Ok(Json(flow_to_response(&flow)))
}

pub async fn delete_flow(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Path(flow_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service.delete_flow(FlowId(flow_id), user.tenant_id, user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn activate_flow(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Path(flow_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let flow = service.activate_flow(FlowId(flow_id), user.tenant_id).await?;
    Ok(Json(flow_to_response(&flow)))
}

pub async fn archive_flow(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Path(flow_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let flow = service.archive_flow(FlowId(flow_id), user.tenant_id).await?;
    Ok(Json(flow_to_response(&flow)))
}

pub async fn import_from_dsl(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Json(req): Json<ImportDslRequest>,
) -> Result<impl IntoResponse> {
    let (flow, validation) = service.import_from_dsl(
        user.tenant_id,
        req.name,
        req.dsl,
        user.user_id,
    ).await?;

    let response = ImportDslResponse {
        flow: flow_to_response(&flow),
        validation: ValidationResultResponse {
            is_valid: validation.is_valid,
            errors: validation.errors,
            warnings: validation.warnings,
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn validate_definition(
    State(service): State<Arc<dyn FlowApplicationService>>,
    _user: AuthenticatedUser,
    Json(definition_json): Json<Value>,
) -> Result<impl IntoResponse> {
    let definition = FlowDefinition::from_json(&definition_json)
        .map_err(|e| PlatformError::ValidationError(e))?;
    
    let validation = service.validate_flow_definition(definition).await?;
    
    let response = ValidationResultResponse {
        is_valid: validation.is_valid,
        errors: validation.errors,
        warnings: validation.warnings,
    };

    Ok(Json(response))
}

pub async fn execute_flow(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Path(flow_id): Path<Uuid>,
    Json(req): Json<ExecuteFlowRequest>,
) -> Result<impl IntoResponse> {
    let session_id = req.session_id.map(SessionId);
    
    let execution = service.execute_flow(
        FlowId(flow_id),
        user.tenant_id,
        user.user_id,
        session_id,
        req.input_data,
    ).await?;

    Ok((StatusCode::CREATED, Json(execution_to_response(&execution))))
}

pub async fn get_execution_status(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Path(execution_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let execution = service.get_execution_status(FlowExecutionId(execution_id), user.tenant_id).await?;
    Ok(Json(execution_to_response(&execution)))
}

pub async fn list_executions(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Path(flow_id): Path<Uuid>,
    Query(query): Query<ListExecutionsQuery>,
) -> Result<impl IntoResponse> {
    // Convert from 1-based (API) to 0-based (internal)
    let page = query.page.saturating_sub(1);
    let limit = query.limit;
    
    let flow_id = Some(flow_id).map(FlowId);
    let (executions, total) = service.list_executions(
        user.tenant_id,
        flow_id,
        page,
        limit,
    ).await?;

    // Calculate total_pages
    let total_pages = if limit > 0 {
        (total + limit - 1) / limit
    } else {
        0
    };

    let response = ExecutionListResponse {
        executions: executions.iter().map(execution_to_response).collect(),
        total,
        page: page + 1,  // Convert back to 1-based for API response
        limit,
        total_pages,
    };

    Ok(Json(response))
}

pub async fn create_version(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Path(flow_id): Path<Uuid>,
    Json(req): Json<CreateVersionRequest>,
) -> Result<impl IntoResponse> {
    let definition = FlowDefinition::from_json(&req.definition)
        .map_err(|e| PlatformError::ValidationError(e))?;

    let version = service.create_version(
        FlowId(flow_id),
        user.tenant_id,
        definition,
        req.change_log,
        user.user_id,
    ).await?;

    Ok((StatusCode::CREATED, Json(version_to_response(&version))))
}

pub async fn get_versions(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Path(flow_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let versions = service.get_versions(FlowId(flow_id), user.tenant_id).await?;
    let response: Vec<FlowVersionResponse> = versions.iter().map(version_to_response).collect();
    Ok(Json(response))
}

pub async fn rollback_to_version(
    State(service): State<Arc<dyn FlowApplicationService>>,
    user: AuthenticatedUser,
    Path(flow_id): Path<Uuid>,
    Json(req): Json<RollbackRequest>,
) -> Result<impl IntoResponse> {
    let flow = service.rollback_to_version(
        FlowId(flow_id),
        user.tenant_id,
        req.target_version,
        user.user_id,
    ).await?;

    Ok(Json(flow_to_response(&flow)))
}

// Helper functions
fn flow_to_response(flow: &crate::domain::entities::Flow) -> FlowResponse {
    FlowResponse {
        id: flow.id.0.to_string(),
        tenant_id: flow.tenant_id.0.to_string(),
        name: flow.name.0.clone(),
        description: flow.description.clone(),
        current_version: flow.current_version.0,
        status: format!("{:?}", flow.status),
        created_by: flow.created_by.0.to_string(),
        created_at: flow.created_at.to_rfc3339(),
        updated_at: flow.updated_at.to_rfc3339(),
    }
}

fn version_to_response(version: &crate::domain::entities::FlowVersion) -> FlowVersionResponse {
    FlowVersionResponse {
        id: version.id.0.to_string(),
        flow_id: version.flow_id.0.to_string(),
        version: version.version.0,
        definition: version.definition.to_json(),
        change_log: version.change_log.clone(),
        created_by: version.created_by.0.to_string(),
        created_at: version.created_at.to_rfc3339(),
    }
}

fn execution_to_response(execution: &crate::domain::entities::FlowExecution) -> FlowExecutionResponse {
    FlowExecutionResponse {
        id: execution.id.0.to_string(),
        flow_id: execution.flow_id.0.to_string(),
        flow_version: execution.flow_version.0,
        tenant_id: execution.tenant_id.0.to_string(),
        user_id: execution.user_id.0.to_string(),
        session_id: execution.session_id.as_ref().map(|s| s.0.to_string()),
        status: format!("{:?}", execution.status),
        input_data: execution.input_data.clone(),
        output_data: execution.output_data.clone(),
        error_message: execution.error_message.clone(),
        started_at: execution.started_at.to_rfc3339(),
        completed_at: execution.completed_at.map(|dt| dt.to_rfc3339()),
        execution_time_ms: execution.execution_time_ms,
    }
}
