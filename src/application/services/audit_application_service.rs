use chrono::{DateTime, Utc};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::{AuditAction, AuditContext, AuditLog, ResourceType};
use crate::domain::repositories::{AuditLogFilter, AuditStatistics};
use crate::domain::services::AuditService;
use crate::error::Result;

/// Application service for audit logging
pub struct AuditApplicationService {
    audit_service: Arc<dyn AuditService>,
}

impl AuditApplicationService {
    pub fn new(audit_service: Arc<dyn AuditService>) -> Self {
        Self { audit_service }
    }

    /// Log an audit event
    pub async fn log_event(
        &self,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        action: AuditAction,
        resource_type: ResourceType,
        resource_id: Option<Uuid>,
        details: Option<Value>,
        context: Option<AuditContext>,
    ) -> Result<Uuid> {
        self.audit_service
            .log_event(
                tenant_id,
                user_id,
                action,
                resource_type,
                resource_id,
                details,
                context,
            )
            .await
    }

    /// Query audit logs with filters
    pub async fn query_logs(&self, filter: &AuditLogFilter) -> Result<Vec<AuditLog>> {
        self.audit_service.query_logs(filter).await
    }

    /// Count audit logs with filters
    pub async fn count_logs(&self, filter: &AuditLogFilter) -> Result<u64> {
        self.audit_service.count_logs(filter).await
    }

    /// Get audit statistics for a tenant
    pub async fn get_statistics(
        &self,
        tenant_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<AuditStatistics> {
        self.audit_service
            .get_statistics(tenant_id, start_date, end_date)
            .await
    }

    /// Query logs with pagination
    pub async fn query_logs_paginated(
        &self,
        tenant_id: Uuid,
        page: u64,
        page_size: u64,
        user_id: Option<Uuid>,
        action: Option<AuditAction>,
        resource_type: Option<ResourceType>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(Vec<AuditLog>, u64)> {
        let offset = (page - 1) * page_size;
        
        let mut filter = AuditLogFilter::new(tenant_id)
            .with_pagination(page_size, offset);

        if let Some(uid) = user_id {
            filter = filter.with_user_id(uid);
        }

        if let Some(act) = action {
            filter = filter.with_action(act);
        }

        if let Some(rt) = resource_type {
            filter = filter.with_resource_type(rt);
        }

        if let Some(start) = start_date {
            if let Some(end) = end_date {
                filter = filter.with_date_range(start, end);
            }
        }

        let logs = self.query_logs(&filter).await?;
        let total = self.count_logs(&filter).await?;

        Ok((logs, total))
    }
}
