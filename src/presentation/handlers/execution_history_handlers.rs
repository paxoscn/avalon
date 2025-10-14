use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{
    ExecutionDetailsResponse, ExecutionDto, ExecutionMetricsDto, ExecutionStepDto,
    QueryExecutionsRequest, QueryExecutionsResponse,
};
use crate::application::services::ExecutionHistoryApplicationService;
use crate::error::{PlatformError, Result};
use crate::presentation::extractors::AuthenticatedUser;

/// Query execution history
pub async fn query_executions(
    State(service): State<Arc<ExecutionHistoryApplicationService>>,
    user: AuthenticatedUser,
    Query(request): Query<QueryExecutionsRequest>,
) -> Result<impl IntoResponse> {
    let page = request.page.unwrap_or(1);
    let page_size = request.page_size.unwrap_or(20);

    let (executions, total) = service
        .query_executions_paginated(
            user.tenant_id.0,
            page,
            page_size,
            request.flow_id,
            request.user_id,
            request.status,
            request.start_date,
            request.end_date,
        )
        .await?;

    let execution_dtos: Vec<ExecutionDto> = executions
        .into_iter()
        .map(|exec| ExecutionDto {
            id: exec.id,
            flow_id: exec.flow_id,
            flow_version: exec.flow_version,
            tenant_id: exec.tenant_id,
            user_id: exec.user_id,
            session_id: exec.session_id,
            status: exec.status.as_str().to_string(),
            input_data: exec.input_data,
            output_data: exec.output_data,
            error_message: exec.error_message,
            started_at: exec.started_at,
            completed_at: exec.completed_at,
            execution_time_ms: exec.execution_time_ms,
        })
        .collect();

    let response = QueryExecutionsResponse {
        executions: execution_dtos,
        total,
        page,
        page_size,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get execution details with steps and metrics
pub async fn get_execution_details(
    State(service): State<Arc<ExecutionHistoryApplicationService>>,
    user: AuthenticatedUser,
    Path(execution_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let details = service
        .get_execution_details(execution_id)
        .await?
        .ok_or_else(|| PlatformError::NotFound("Execution not found".to_string()))?;

    let (execution, steps, metrics) = details;

    // Verify tenant access
    if execution.tenant_id != user.tenant_id.0 {
        return Err(PlatformError::AuthorizationFailed(
            "Access denied".to_string(),
        ));
    }

    let execution_dto = ExecutionDto {
        id: execution.id,
        flow_id: execution.flow_id,
        flow_version: execution.flow_version,
        tenant_id: execution.tenant_id,
        user_id: execution.user_id,
        session_id: execution.session_id,
        status: execution.status.as_str().to_string(),
        input_data: execution.input_data,
        output_data: execution.output_data,
        error_message: execution.error_message,
        started_at: execution.started_at,
        completed_at: execution.completed_at,
        execution_time_ms: execution.execution_time_ms,
    };

    let step_dtos: Vec<ExecutionStepDto> = steps
        .into_iter()
        .map(|step| ExecutionStepDto {
            id: step.id,
            execution_id: step.execution_id,
            step_name: step.step_name,
            step_type: step.step_type,
            status: step.status.as_str().to_string(),
            input_data: step.input_data,
            output_data: step.output_data,
            error_message: step.error_message,
            started_at: step.started_at,
            completed_at: step.completed_at,
            execution_time_ms: step.execution_time_ms,
            metadata: step.metadata,
        })
        .collect();

    let metrics_dto = ExecutionMetricsDto {
        execution_id: metrics.execution_id,
        total_steps: metrics.total_steps,
        completed_steps: metrics.completed_steps,
        failed_steps: metrics.failed_steps,
        skipped_steps: metrics.skipped_steps,
        total_execution_time_ms: metrics.total_execution_time_ms,
        average_step_time_ms: metrics.average_step_time_ms,
        slowest_step: metrics.slowest_step,
        slowest_step_time_ms: metrics.slowest_step_time_ms,
    };

    let response = ExecutionDetailsResponse {
        execution: execution_dto,
        steps: step_dtos,
        metrics: metrics_dto,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get execution steps
pub async fn get_execution_steps(
    State(service): State<Arc<ExecutionHistoryApplicationService>>,
    user: AuthenticatedUser,
    Path(execution_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    // First verify the execution exists and belongs to the tenant
    let execution = service
        .get_execution(execution_id)
        .await?
        .ok_or_else(|| PlatformError::NotFound("Execution not found".to_string()))?;

    if execution.tenant_id != user.tenant_id.0 {
        return Err(PlatformError::AuthorizationFailed(
            "Access denied".to_string(),
        ));
    }

    let steps = service.get_execution_steps(execution_id).await?;

    let step_dtos: Vec<ExecutionStepDto> = steps
        .into_iter()
        .map(|step| ExecutionStepDto {
            id: step.id,
            execution_id: step.execution_id,
            step_name: step.step_name,
            step_type: step.step_type,
            status: step.status.as_str().to_string(),
            input_data: step.input_data,
            output_data: step.output_data,
            error_message: step.error_message,
            started_at: step.started_at,
            completed_at: step.completed_at,
            execution_time_ms: step.execution_time_ms,
            metadata: step.metadata,
        })
        .collect();

    Ok((StatusCode::OK, Json(step_dtos)))
}

/// Get execution metrics
pub async fn get_execution_metrics(
    State(service): State<Arc<ExecutionHistoryApplicationService>>,
    user: AuthenticatedUser,
    Path(execution_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    // First verify the execution exists and belongs to the tenant
    let execution = service
        .get_execution(execution_id)
        .await?
        .ok_or_else(|| PlatformError::NotFound("Execution not found".to_string()))?;

    if execution.tenant_id != user.tenant_id.0 {
        return Err(PlatformError::AuthorizationFailed(
            "Access denied".to_string(),
        ));
    }

    let metrics = service.get_execution_metrics(execution_id).await?;

    let metrics_dto = ExecutionMetricsDto {
        execution_id: metrics.execution_id,
        total_steps: metrics.total_steps,
        completed_steps: metrics.completed_steps,
        failed_steps: metrics.failed_steps,
        skipped_steps: metrics.skipped_steps,
        total_execution_time_ms: metrics.total_execution_time_ms,
        average_step_time_ms: metrics.average_step_time_ms,
        slowest_step: metrics.slowest_step,
        slowest_step_time_ms: metrics.slowest_step_time_ms,
    };

    Ok((StatusCode::OK, Json(metrics_dto)))
}
