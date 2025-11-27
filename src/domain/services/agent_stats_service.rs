use std::sync::Arc;
use chrono::Utc;
use rust_decimal::Decimal;
use crate::domain::repositories::AgentDailyStatsRepository;
use crate::domain::value_objects::{AgentId, TenantId};
use crate::error::Result;

/// Domain service for managing agent statistics
pub struct AgentStatsService {
    stats_repo: Arc<dyn AgentDailyStatsRepository>,
}

impl AgentStatsService {
    pub fn new(stats_repo: Arc<dyn AgentDailyStatsRepository>) -> Self {
        Self { stats_repo }
    }

    /// Record an interview attempt
    pub async fn record_interview(&self, agent_id: AgentId, tenant_id: TenantId) -> Result<()> {
        let today = Utc::now().date_naive();
        let mut stats = self.stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
        
        stats.increment_interview();
        self.stats_repo.update(&stats).await?;
        
        Ok(())
    }

    /// Record a passed interview
    pub async fn record_interview_passed(&self, agent_id: AgentId, tenant_id: TenantId) -> Result<()> {
        let today = Utc::now().date_naive();
        let mut stats = self.stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
        
        stats.increment_interview_passed();
        self.stats_repo.update(&stats).await?;
        
        Ok(())
    }

    /// Record an employment
    pub async fn record_employment(&self, agent_id: AgentId, tenant_id: TenantId) -> Result<()> {
        let today = Utc::now().date_naive();
        let mut stats = self.stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
        
        stats.increment_employment();
        self.stats_repo.update(&stats).await?;
        
        Ok(())
    }

    /// Record a new session
    pub async fn record_session(&self, agent_id: AgentId, tenant_id: TenantId) -> Result<()> {
        let today = Utc::now().date_naive();
        let mut stats = self.stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
        
        stats.increment_session();
        self.stats_repo.update(&stats).await?;
        
        Ok(())
    }

    /// Record messages in a session
    pub async fn record_messages(&self, agent_id: AgentId, tenant_id: TenantId, count: i64) -> Result<()> {
        let today = Utc::now().date_naive();
        let mut stats = self.stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
        
        stats.add_messages(count);
        self.stats_repo.update(&stats).await?;
        
        Ok(())
    }

    /// Record tokens used
    pub async fn record_tokens(&self, agent_id: AgentId, tenant_id: TenantId, count: i64) -> Result<()> {
        let today = Utc::now().date_naive();
        let mut stats = self.stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
        
        stats.add_tokens(count);
        self.stats_repo.update(&stats).await?;
        
        Ok(())
    }

    /// Record revenue
    pub async fn record_revenue(&self, agent_id: AgentId, tenant_id: TenantId, amount: Decimal) -> Result<()> {
        let today = Utc::now().date_naive();
        let mut stats = self.stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
        
        stats.add_revenue(amount);
        self.stats_repo.update(&stats).await?;
        
        Ok(())
    }
}
