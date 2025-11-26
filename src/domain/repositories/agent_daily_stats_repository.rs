use async_trait::async_trait;
use chrono::NaiveDate;
use crate::domain::entities::AgentDailyStats;
use crate::domain::value_objects::{AgentId, TenantId};
use crate::error::Result;

#[async_trait]
pub trait AgentDailyStatsRepository: Send + Sync {
    async fn create(&self, stats: &AgentDailyStats) -> Result<AgentDailyStats>;
    
    async fn update(&self, stats: &AgentDailyStats) -> Result<AgentDailyStats>;
    
    async fn find_by_agent_and_date(
        &self,
        agent_id: &AgentId,
        stat_date: NaiveDate,
    ) -> Result<Option<AgentDailyStats>>;
    
    async fn find_by_agent_and_date_range(
        &self,
        agent_id: &AgentId,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AgentDailyStats>>;
    
    async fn find_by_tenant_and_date(
        &self,
        tenant_id: &TenantId,
        stat_date: NaiveDate,
    ) -> Result<Vec<AgentDailyStats>>;
    
    async fn find_by_tenant_and_date_range(
        &self,
        tenant_id: &TenantId,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AgentDailyStats>>;
    
    async fn get_or_create(
        &self,
        agent_id: &AgentId,
        tenant_id: &TenantId,
        stat_date: NaiveDate,
    ) -> Result<AgentDailyStats>;
}
