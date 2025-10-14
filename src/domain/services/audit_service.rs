use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;
use std::sync::Arc;

use crate::domain::entities::{AuditAction, AuditContext, AuditLog, ResourceType};
use crate::domain::repositories::{AuditLogFilter, AuditLogRepository, AuditStatistics};
use crate::error::Result;
use chrono::{DateTime, Utc};

/// Domain service for audit logging
#[async_trait]
pub trait AuditService: Send + Sync {
    /// Log an audit event
    async fn log_event(
        &self,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        action: AuditAction,
        resource_type: ResourceType,
        resource_id: Option<Uuid>,
        details: Option<Value>,
        context: Option<AuditContext>,
    ) -> Result<Uuid>;

    /// Query audit logs
    async fn query_logs(&self, filter: &AuditLogFilter) -> Result<Vec<AuditLog>>;

    /// Count audit logs
    async fn count_logs(&self, filter: &AuditLogFilter) -> Result<u64>;

    /// Get audit statistics
    async fn get_statistics(
        &self,
        tenant_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<AuditStatistics>;
}

/// Implementation of audit service
pub struct AuditServiceImpl {
    audit_log_repository: Arc<dyn AuditLogRepository>,
}

impl AuditServiceImpl {
    pub fn new(audit_log_repository: Arc<dyn AuditLogRepository>) -> Self {
        Self {
            audit_log_repository,
        }
    }
}

#[async_trait]
impl AuditService for AuditServiceImpl {
    async fn log_event(
        &self,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        action: AuditAction,
        resource_type: ResourceType,
        resource_id: Option<Uuid>,
        details: Option<Value>,
        context: Option<AuditContext>,
    ) -> Result<Uuid> {
        let mut audit_log = AuditLog::new(tenant_id, user_id, action, resource_type, resource_id);

        if let Some(details) = details {
            audit_log = audit_log.with_details(details);
        }

        if let Some(ctx) = context {
            if let Some(ip) = ctx.ip_address {
                audit_log = audit_log.with_ip_address(ip);
            }
            if let Some(ua) = ctx.user_agent {
                audit_log = audit_log.with_user_agent(ua);
            }
        }

        let audit_id = audit_log.id;
        self.audit_log_repository.create(&audit_log).await?;

        Ok(audit_id)
    }

    async fn query_logs(&self, filter: &AuditLogFilter) -> Result<Vec<AuditLog>> {
        self.audit_log_repository.find_with_filter(filter).await
    }

    async fn count_logs(&self, filter: &AuditLogFilter) -> Result<u64> {
        self.audit_log_repository.count_with_filter(filter).await
    }

    async fn get_statistics(
        &self,
        tenant_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<AuditStatistics> {
        self.audit_log_repository
            .get_statistics(tenant_id, start_date, end_date)
            .await
    }
}
