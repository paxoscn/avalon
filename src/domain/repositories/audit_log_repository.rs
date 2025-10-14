use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::{AuditAction, AuditLog, ResourceType};
use crate::error::Result;

/// Query filters for audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditLogFilter {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: Option<AuditAction>,
    pub resource_type: Option<ResourceType>,
    pub resource_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl AuditLogFilter {
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            tenant_id,
            ..Default::default()
        }
    }

    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_action(mut self, action: AuditAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_resource_type(mut self, resource_type: ResourceType) -> Self {
        self.resource_type = Some(resource_type);
        self
    }

    pub fn with_resource_id(mut self, resource_id: Uuid) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    pub fn with_date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }

    pub fn with_pagination(mut self, limit: u64, offset: u64) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

/// Statistics for audit logs
#[derive(Debug, Clone)]
pub struct AuditStatistics {
    pub total_count: u64,
    pub action_counts: Vec<(String, u64)>,
    pub resource_type_counts: Vec<(String, u64)>,
    pub user_activity: Vec<(Uuid, u64)>,
}

/// Repository interface for audit logs
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// Create a new audit log entry
    async fn create(&self, audit_log: &AuditLog) -> Result<()>;

    /// Find audit log by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLog>>;

    /// Find audit logs with filters
    async fn find_with_filter(&self, filter: &AuditLogFilter) -> Result<Vec<AuditLog>>;

    /// Count audit logs with filters
    async fn count_with_filter(&self, filter: &AuditLogFilter) -> Result<u64>;

    /// Get audit statistics for a tenant
    async fn get_statistics(
        &self,
        tenant_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<AuditStatistics>;

    /// Delete old audit logs (for cleanup)
    async fn delete_older_than(&self, date: DateTime<Utc>) -> Result<u64>;
}
