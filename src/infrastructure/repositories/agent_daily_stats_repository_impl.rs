use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
use sea_orm::prelude::Decimal;
use std::sync::Arc;
use chrono::NaiveDate;
use crate::domain::entities::AgentDailyStats;
use crate::domain::repositories::AgentDailyStatsRepository;
use crate::domain::value_objects::{AgentId, TenantId};
use crate::error::{Result, PlatformError};
use crate::infrastructure::database::entities;

pub struct AgentDailyStatsRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl AgentDailyStatsRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_domain(entity: entities::agent_daily_stats::Model) -> AgentDailyStats {
        AgentDailyStats {
            id: entity.id,
            agent_id: AgentId::from_uuid(entity.agent_id),
            tenant_id: TenantId::from_uuid(entity.tenant_id),
            stat_date: entity.stat_date,
            interview_count: entity.interview_count,
            interview_passed_count: entity.interview_passed_count,
            employment_count: entity.employment_count,
            session_count: entity.session_count,
            message_count: entity.message_count,
            token_count: entity.token_count,
            revenue: entity.revenue.to_string().parse::<f64>().unwrap_or(0.0),
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }

    fn domain_to_active_model(stats: &AgentDailyStats) -> entities::agent_daily_stats::ActiveModel {
        entities::agent_daily_stats::ActiveModel {
            id: Set(stats.id),
            agent_id: Set(stats.agent_id.0),
            tenant_id: Set(stats.tenant_id.0),
            stat_date: Set(stats.stat_date),
            interview_count: Set(stats.interview_count),
            interview_passed_count: Set(stats.interview_passed_count),
            employment_count: Set(stats.employment_count),
            session_count: Set(stats.session_count),
            message_count: Set(stats.message_count),
            token_count: Set(stats.token_count),
            revenue: Set(Decimal::try_from(stats.revenue).unwrap_or(Decimal::ZERO)),
            created_at: Set(stats.created_at),
            updated_at: Set(stats.updated_at),
        }
    }
}

#[async_trait]
impl AgentDailyStatsRepository for AgentDailyStatsRepositoryImpl {
    async fn create(&self, stats: &AgentDailyStats) -> Result<AgentDailyStats> {
        let active_model = Self::domain_to_active_model(stats);
        
        let result = entities::agent_daily_stats::Entity::insert(active_model)
            .exec(self.db.as_ref())
            .await?;

        let created = entities::agent_daily_stats::Entity::find_by_id(result.last_insert_id)
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| PlatformError::NotFound("Created stats not found".to_string()))?;

        Ok(Self::entity_to_domain(created))
    }

    async fn update(&self, stats: &AgentDailyStats) -> Result<AgentDailyStats> {
        let active_model = Self::domain_to_active_model(stats);
        
        let updated = active_model
            .update(self.db.as_ref())
            .await?;

        Ok(Self::entity_to_domain(updated))
    }

    async fn find_by_agent_and_date(
        &self,
        agent_id: &AgentId,
        stat_date: NaiveDate,
    ) -> Result<Option<AgentDailyStats>> {
        let stats = entities::agent_daily_stats::Entity::find()
            .filter(entities::agent_daily_stats::Column::AgentId.eq(agent_id.0))
            .filter(entities::agent_daily_stats::Column::StatDate.eq(stat_date))
            .one(self.db.as_ref())
            .await?;

        Ok(stats.map(Self::entity_to_domain))
    }

    async fn find_by_agent_and_date_range(
        &self,
        agent_id: &AgentId,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AgentDailyStats>> {
        let stats = entities::agent_daily_stats::Entity::find()
            .filter(entities::agent_daily_stats::Column::AgentId.eq(agent_id.0))
            .filter(entities::agent_daily_stats::Column::StatDate.gte(start_date))
            .filter(entities::agent_daily_stats::Column::StatDate.lte(end_date))
            .all(self.db.as_ref())
            .await?;

        Ok(stats.into_iter().map(Self::entity_to_domain).collect())
    }

    async fn find_by_tenant_and_date(
        &self,
        tenant_id: &TenantId,
        stat_date: NaiveDate,
    ) -> Result<Vec<AgentDailyStats>> {
        let stats = entities::agent_daily_stats::Entity::find()
            .filter(entities::agent_daily_stats::Column::TenantId.eq(tenant_id.0))
            .filter(entities::agent_daily_stats::Column::StatDate.eq(stat_date))
            .all(self.db.as_ref())
            .await?;

        Ok(stats.into_iter().map(Self::entity_to_domain).collect())
    }

    async fn find_by_tenant_and_date_range(
        &self,
        tenant_id: &TenantId,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AgentDailyStats>> {
        let stats = entities::agent_daily_stats::Entity::find()
            .filter(entities::agent_daily_stats::Column::TenantId.eq(tenant_id.0))
            .filter(entities::agent_daily_stats::Column::StatDate.gte(start_date))
            .filter(entities::agent_daily_stats::Column::StatDate.lte(end_date))
            .all(self.db.as_ref())
            .await?;

        Ok(stats.into_iter().map(Self::entity_to_domain).collect())
    }

    async fn get_or_create(
        &self,
        agent_id: &AgentId,
        tenant_id: &TenantId,
        stat_date: NaiveDate,
    ) -> Result<AgentDailyStats> {
        // Try to find existing stats
        if let Some(stats) = self.find_by_agent_and_date(agent_id, stat_date).await? {
            return Ok(stats);
        }

        // Create new stats if not found
        let new_stats = AgentDailyStats::new(*agent_id, *tenant_id, stat_date);
        self.create(&new_stats).await
    }
}
