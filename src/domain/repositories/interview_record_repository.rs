use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::InterviewRecord;
use crate::domain::value_objects::{AgentId, TenantId, UserId};
use crate::error::Result;

#[async_trait]
pub trait InterviewRecordRepository: Send + Sync {
    async fn create(&self, record: &InterviewRecord) -> Result<InterviewRecord>;
    async fn update(&self, record: &InterviewRecord) -> Result<InterviewRecord>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<InterviewRecord>>;
    async fn find_by_agent(&self, agent_id: &AgentId) -> Result<Vec<InterviewRecord>>;
    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<InterviewRecord>>;
    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<InterviewRecord>>;
    async fn find_by_session(&self, session_id: Uuid) -> Result<Option<InterviewRecord>>;
    async fn find_by_agent_and_date_range(
        &self,
        agent_id: &AgentId,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<InterviewRecord>>;
    async fn count_by_agent(&self, agent_id: &AgentId) -> Result<i64>;
    async fn count_passed_by_agent(&self, agent_id: &AgentId) -> Result<i64>;
}
