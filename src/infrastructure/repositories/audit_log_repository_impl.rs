use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::*;
use uuid::Uuid;

use crate::domain::entities::{AuditAction, AuditLog, ResourceType};
use crate::domain::repositories::{AuditLogFilter, AuditLogRepository, AuditStatistics};
use crate::error::{PlatformError, Result};
use crate::infrastructure::database::entities::audit_log;

pub struct AuditLogRepositoryImpl {
    db: DatabaseConnection,
}

impl AuditLogRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn to_domain(&self, model: audit_log::Model) -> AuditLog {
        AuditLog {
            id: model.id,
            tenant_id: model.tenant_id,
            user_id: model.user_id,
            action: AuditAction::from(model.action),
            resource_type: ResourceType::from(model.resource_type),
            resource_id: model.resource_id,
            details: model.details,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            created_at: model.created_at,
        }
    }

    fn to_active_model(&self, audit_log: &AuditLog) -> audit_log::ActiveModel {
        audit_log::ActiveModel {
            id: Set(audit_log.id),
            tenant_id: Set(audit_log.tenant_id),
            user_id: Set(audit_log.user_id),
            action: Set(audit_log.action.as_str().to_string()),
            resource_type: Set(audit_log.resource_type.as_str().to_string()),
            resource_id: Set(audit_log.resource_id),
            details: Set(audit_log.details.clone()),
            ip_address: Set(audit_log.ip_address.clone()),
            user_agent: Set(audit_log.user_agent.clone()),
            created_at: Set(audit_log.created_at),
        }
    }
}

#[async_trait]
impl AuditLogRepository for AuditLogRepositoryImpl {
    async fn create(&self, audit_log: &AuditLog) -> Result<()> {
        let active_model = self.to_active_model(audit_log);
        audit_log::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(PlatformError::from)?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLog>> {
        let model = audit_log::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(PlatformError::from)?;

        Ok(model.map(|m| self.to_domain(m)))
    }

    async fn find_with_filter(&self, filter: &AuditLogFilter) -> Result<Vec<AuditLog>> {
        let mut query = audit_log::Entity::find()
            .filter(audit_log::Column::TenantId.eq(filter.tenant_id));

        if let Some(user_id) = filter.user_id {
            query = query.filter(audit_log::Column::UserId.eq(user_id));
        }

        if let Some(ref action) = filter.action {
            query = query.filter(audit_log::Column::Action.eq(action.as_str()));
        }

        if let Some(ref resource_type) = filter.resource_type {
            query = query.filter(audit_log::Column::ResourceType.eq(resource_type.as_str()));
        }

        if let Some(resource_id) = filter.resource_id {
            query = query.filter(audit_log::Column::ResourceId.eq(resource_id));
        }

        if let Some(start_date) = filter.start_date {
            query = query.filter(audit_log::Column::CreatedAt.gte(start_date));
        }

        if let Some(end_date) = filter.end_date {
            query = query.filter(audit_log::Column::CreatedAt.lte(end_date));
        }

        query = query.order_by_desc(audit_log::Column::CreatedAt);

        if let Some(limit) = filter.limit {
            query = query.limit(limit);
        }

        if let Some(offset) = filter.offset {
            query = query.offset(offset);
        }

        let models = query
            .all(&self.db)
            .await
            .map_err(PlatformError::from)?;

        Ok(models.into_iter().map(|m| self.to_domain(m)).collect())
    }

    async fn count_with_filter(&self, filter: &AuditLogFilter) -> Result<u64> {
        let mut query = audit_log::Entity::find()
            .filter(audit_log::Column::TenantId.eq(filter.tenant_id));

        if let Some(user_id) = filter.user_id {
            query = query.filter(audit_log::Column::UserId.eq(user_id));
        }

        if let Some(ref action) = filter.action {
            query = query.filter(audit_log::Column::Action.eq(action.as_str()));
        }

        if let Some(ref resource_type) = filter.resource_type {
            query = query.filter(audit_log::Column::ResourceType.eq(resource_type.as_str()));
        }

        if let Some(resource_id) = filter.resource_id {
            query = query.filter(audit_log::Column::ResourceId.eq(resource_id));
        }

        if let Some(start_date) = filter.start_date {
            query = query.filter(audit_log::Column::CreatedAt.gte(start_date));
        }

        if let Some(end_date) = filter.end_date {
            query = query.filter(audit_log::Column::CreatedAt.lte(end_date));
        }

        query
            .count(&self.db)
            .await
            .map_err(PlatformError::from)
    }

    async fn get_statistics(
        &self,
        tenant_id: Uuid,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<AuditStatistics> {
        let mut base_query = audit_log::Entity::find()
            .filter(audit_log::Column::TenantId.eq(tenant_id));

        if let Some(start) = start_date {
            base_query = base_query.filter(audit_log::Column::CreatedAt.gte(start));
        }

        if let Some(end) = end_date {
            base_query = base_query.filter(audit_log::Column::CreatedAt.lte(end));
        }

        // Get total count
        let total_count = base_query
            .clone()
            .count(&self.db)
            .await
            .map_err(PlatformError::from)?;

        // Get action counts
        let action_counts = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"
                SELECT action, COUNT(*) as count
                FROM audit_logs
                WHERE tenant_id = ?
                AND (? IS NULL OR created_at >= ?)
                AND (? IS NULL OR created_at <= ?)
                GROUP BY action
                ORDER BY count DESC
                "#,
                vec![
                    tenant_id.into(),
                    start_date.into(),
                    start_date.into(),
                    end_date.into(),
                    end_date.into(),
                ],
            ))
            .await
            .map_err(PlatformError::from)?
            .into_iter()
            .map(|row| {
                let action: String = row.try_get("", "action").unwrap_or_default();
                let count: i64 = row.try_get("", "count").unwrap_or(0);
                (action, count as u64)
            })
            .collect();

        // Get resource type counts
        let resource_type_counts = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"
                SELECT resource_type, COUNT(*) as count
                FROM audit_logs
                WHERE tenant_id = ?
                AND (? IS NULL OR created_at >= ?)
                AND (? IS NULL OR created_at <= ?)
                GROUP BY resource_type
                ORDER BY count DESC
                "#,
                vec![
                    tenant_id.into(),
                    start_date.into(),
                    start_date.into(),
                    end_date.into(),
                    end_date.into(),
                ],
            ))
            .await
            .map_err(PlatformError::from)?
            .into_iter()
            .map(|row| {
                let resource_type: String = row.try_get("", "resource_type").unwrap_or_default();
                let count: i64 = row.try_get("", "count").unwrap_or(0);
                (resource_type, count as u64)
            })
            .collect();

        // Get user activity
        let user_activity = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::MySql,
                r#"
                SELECT user_id, COUNT(*) as count
                FROM audit_logs
                WHERE tenant_id = ?
                AND user_id IS NOT NULL
                AND (? IS NULL OR created_at >= ?)
                AND (? IS NULL OR created_at <= ?)
                GROUP BY user_id
                ORDER BY count DESC
                LIMIT 10
                "#,
                vec![
                    tenant_id.into(),
                    start_date.into(),
                    start_date.into(),
                    end_date.into(),
                    end_date.into(),
                ],
            ))
            .await
            .map_err(PlatformError::from)?
            .into_iter()
            .filter_map(|row| {
                let user_id: Option<Uuid> = row.try_get("", "user_id").ok()?;
                let count: i64 = row.try_get("", "count").unwrap_or(0);
                user_id.map(|uid| (uid, count as u64))
            })
            .collect();

        Ok(AuditStatistics {
            total_count,
            action_counts,
            resource_type_counts,
            user_activity,
        })
    }

    async fn delete_older_than(&self, date: DateTime<Utc>) -> Result<u64> {
        let result = audit_log::Entity::delete_many()
            .filter(audit_log::Column::CreatedAt.lt(date))
            .exec(&self.db)
            .await
            .map_err(PlatformError::from)?;

        Ok(result.rows_affected)
    }
}
