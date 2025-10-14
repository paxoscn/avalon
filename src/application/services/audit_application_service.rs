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
        limit: u64,
        user_id: Option<Uuid>,
        action: Option<AuditAction>,
        resource_type: Option<ResourceType>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(Vec<AuditLog>, u64)> {
        let offset = page * limit;
        
        let mut filter = AuditLogFilter::new(tenant_id)
            .with_pagination(limit, offset);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{AuditAction, ResourceType};
    use crate::domain::repositories::{AuditLogFilter, AuditLogRepository, AuditStatistics};
    use crate::domain::services::AuditService;
    use async_trait::async_trait;
    use mockall::mock;
    use std::sync::Arc;

    mock! {
        AuditServiceImpl {}

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
            ) -> Result<Uuid>;

            async fn query_logs(&self, filter: &AuditLogFilter) -> Result<Vec<AuditLog>>;
            async fn count_logs(&self, filter: &AuditLogFilter) -> Result<u64>;
            async fn get_statistics(
                &self,
                tenant_id: Uuid,
                start_date: Option<DateTime<Utc>>,
                end_date: Option<DateTime<Utc>>,
            ) -> Result<AuditStatistics>;
        }
    }

    #[tokio::test]
    async fn test_query_logs_paginated_zero_based() {
        let mut mock_service = MockAuditServiceImpl::new();
        let tenant_id = Uuid::new_v4();

        // Mock query_logs to return empty vec
        mock_service
            .expect_query_logs()
            .times(1)
            .returning(|_| Ok(vec![]));

        // Mock count_logs to return total count
        mock_service
            .expect_count_logs()
            .times(1)
            .returning(|_| Ok(50));

        let service = AuditApplicationService::new(Arc::new(mock_service));

        // Test page 0 (first page) with limit 20
        let result = service
            .query_logs_paginated(tenant_id, 0, 20, None, None, None, None, None)
            .await;

        assert!(result.is_ok());
        let (logs, total) = result.unwrap();
        assert_eq!(logs.len(), 0);
        assert_eq!(total, 50);
    }

    #[tokio::test]
    async fn test_query_logs_paginated_offset_calculation() {
        let mut mock_service = MockAuditServiceImpl::new();
        let tenant_id = Uuid::new_v4();

        // Verify that offset is calculated as page * limit
        mock_service
            .expect_query_logs()
            .times(1)
            .withf(|filter: &AuditLogFilter| {
                // For page=2, limit=20, offset should be 40
                filter.limit == Some(20) && filter.offset == Some(40)
            })
            .returning(|_| Ok(vec![]));

        mock_service
            .expect_count_logs()
            .times(1)
            .returning(|_| Ok(100));

        let service = AuditApplicationService::new(Arc::new(mock_service));

        // Test page 2 with limit 20 (offset should be 2 * 20 = 40)
        let result = service
            .query_logs_paginated(tenant_id, 2, 20, None, None, None, None, None)
            .await;

        assert!(result.is_ok());
        let (_, total) = result.unwrap();
        assert_eq!(total, 100);
    }

    #[tokio::test]
    async fn test_query_logs_paginated_total_count_accuracy() {
        let mut mock_service = MockAuditServiceImpl::new();
        let tenant_id = Uuid::new_v4();

        mock_service
            .expect_query_logs()
            .times(1)
            .returning(|_| Ok(vec![]));

        // Verify total count is returned accurately
        mock_service
            .expect_count_logs()
            .times(1)
            .returning(|_| Ok(75));

        let service = AuditApplicationService::new(Arc::new(mock_service));

        let result = service
            .query_logs_paginated(tenant_id, 0, 20, None, None, None, None, None)
            .await;

        assert!(result.is_ok());
        let (_, total) = result.unwrap();
        assert_eq!(total, 75);
    }
}
