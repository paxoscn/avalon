use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::application::dto::{
    ActionCount, AuditLogDto, AuditStatisticsDto, ExportAuditLogsRequest, 
    GetAuditStatisticsRequest, QueryAuditLogsRequest, QueryAuditLogsResponse, 
    ResourceTypeCount, UserActivity,
};
use crate::application::dto::audit_dto::ExportFormat;
use crate::application::services::AuditApplicationService;
use crate::domain::entities::{AuditAction, ResourceType};
use crate::domain::repositories::AuditLogFilter;
use crate::error::Result;
use crate::presentation::extractors::AuthenticatedUser;

pub struct AuditHandlers {
    audit_service: Arc<AuditApplicationService>,
}

impl AuditHandlers {
    pub fn new(audit_service: Arc<AuditApplicationService>) -> Self {
        Self { audit_service }
    }
}

/// Query audit logs
pub async fn query_audit_logs(
    State(service): State<Arc<AuditApplicationService>>,
    user: AuthenticatedUser,
    Query(request): Query<QueryAuditLogsRequest>,
) -> Result<impl IntoResponse> {
    let page = request.page.unwrap_or(1);
    let page_size = request.page_size.unwrap_or(20);

    let action = request.action.map(|a| AuditAction::from(a));
    let resource_type = request.resource_type.map(|rt| ResourceType::from(rt));

    let (logs, total) = service
        .query_logs_paginated(
            user.tenant_id.0,
            page,
            page_size,
            request.user_id,
            action,
            resource_type,
            request.start_date,
            request.end_date,
        )
        .await?;

    let log_dtos: Vec<AuditLogDto> = logs
        .into_iter()
        .map(|log| AuditLogDto {
            id: log.id,
            tenant_id: log.tenant_id,
            user_id: log.user_id,
            action: log.action.as_str().to_string(),
            resource_type: log.resource_type.as_str().to_string(),
            resource_id: log.resource_id,
            details: log.details,
            ip_address: log.ip_address,
            user_agent: log.user_agent,
            created_at: log.created_at,
        })
        .collect();

    let response = QueryAuditLogsResponse {
        logs: log_dtos,
        total,
        page,
        page_size,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get audit statistics
pub async fn get_audit_statistics(
    State(service): State<Arc<AuditApplicationService>>,
    user: AuthenticatedUser,
    Query(request): Query<GetAuditStatisticsRequest>,
) -> Result<impl IntoResponse> {
    let statistics = service
        .get_statistics(user.tenant_id.0, request.start_date, request.end_date)
        .await?;

    let response = AuditStatisticsDto {
        total_count: statistics.total_count,
        action_counts: statistics
            .action_counts
            .into_iter()
            .map(|(action, count)| ActionCount { action, count })
            .collect(),
        resource_type_counts: statistics
            .resource_type_counts
            .into_iter()
            .map(|(resource_type, count)| ResourceTypeCount {
                resource_type,
                count,
            })
            .collect(),
        user_activity: statistics
            .user_activity
            .into_iter()
            .map(|(user_id, count)| UserActivity { user_id, count })
            .collect(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Export audit logs
pub async fn export_audit_logs(
    State(service): State<Arc<AuditApplicationService>>,
    user: AuthenticatedUser,
    Json(request): Json<ExportAuditLogsRequest>,
) -> Result<impl IntoResponse> {
    let action = request.action.map(|a| AuditAction::from(a));
    let resource_type = request.resource_type.map(|rt| ResourceType::from(rt));

    let mut filter = AuditLogFilter::new(user.tenant_id.0);

    if let Some(uid) = request.user_id {
        filter = filter.with_user_id(uid);
    }

    if let Some(act) = action {
        filter = filter.with_action(act);
    }

    if let Some(rt) = resource_type {
        filter = filter.with_resource_type(rt);
    }

    if let Some(start) = request.start_date {
        if let Some(end) = request.end_date {
            filter = filter.with_date_range(start, end);
        }
    }

    let logs = service.query_logs(&filter).await?;

    let log_dtos: Vec<AuditLogDto> = logs
        .into_iter()
        .map(|log| AuditLogDto {
            id: log.id,
            tenant_id: log.tenant_id,
            user_id: log.user_id,
            action: log.action.as_str().to_string(),
            resource_type: log.resource_type.as_str().to_string(),
            resource_id: log.resource_id,
            details: log.details,
            ip_address: log.ip_address,
            user_agent: log.user_agent,
            created_at: log.created_at,
        })
        .collect();

    match request.format {
        ExportFormat::Json => {
            Ok((StatusCode::OK, Json(log_dtos)).into_response())
        }
        ExportFormat::Csv => {
            // For CSV, we'll return a simple text representation
            // In production, you'd use a proper CSV library
            let mut csv = String::from("id,tenant_id,user_id,action,resource_type,resource_id,created_at\n");
            for log in log_dtos {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    log.id,
                    log.tenant_id,
                    log.user_id.map(|u| u.to_string()).unwrap_or_default(),
                    log.action,
                    log.resource_type,
                    log.resource_id.map(|r| r.to_string()).unwrap_or_default(),
                    log.created_at.to_rfc3339()
                ));
            }
            Ok((
                StatusCode::OK,
                [(
                    axum::http::header::CONTENT_TYPE,
                    "text/csv; charset=utf-8",
                )],
                csv,
            )
                .into_response())
        }
    }
}
