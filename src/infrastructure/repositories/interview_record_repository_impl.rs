use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
use sea_orm::PaginatorTrait;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::{InterviewRecord, InterviewStatus};
use crate::domain::repositories::InterviewRecordRepository;
use crate::domain::value_objects::{AgentId, TenantId, UserId};
use crate::error::{Result, PlatformError};
use crate::infrastructure::database::entities;

pub struct InterviewRecordRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl InterviewRecordRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_domain(entity: entities::interview_record::Model) -> InterviewRecord {
        let status = match entity.status {
            entities::interview_record::InterviewStatus::Pending => InterviewStatus::Pending,
            entities::interview_record::InterviewStatus::InProgress => InterviewStatus::InProgress,
            entities::interview_record::InterviewStatus::Passed => InterviewStatus::Passed,
            entities::interview_record::InterviewStatus::Failed => InterviewStatus::Failed,
            entities::interview_record::InterviewStatus::Cancelled => InterviewStatus::Cancelled,
        };

        InterviewRecord {
            id: entity.id,
            agent_id: AgentId::from_uuid(entity.agent_id),
            tenant_id: TenantId::from_uuid(entity.tenant_id),
            user_id: entity.user_id.map(UserId::from_uuid),
            session_id: entity.session_id,
            status,
            score: entity.score,
            feedback: entity.feedback,
            questions: entity.questions,
            answers: entity.answers,
            started_at: entity.started_at,
            completed_at: entity.completed_at,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }

    fn domain_to_active_model(record: &InterviewRecord) -> entities::interview_record::ActiveModel {
        let status = match record.status {
            InterviewStatus::Pending => entities::interview_record::InterviewStatus::Pending,
            InterviewStatus::InProgress => entities::interview_record::InterviewStatus::InProgress,
            InterviewStatus::Passed => entities::interview_record::InterviewStatus::Passed,
            InterviewStatus::Failed => entities::interview_record::InterviewStatus::Failed,
            InterviewStatus::Cancelled => entities::interview_record::InterviewStatus::Cancelled,
        };

        entities::interview_record::ActiveModel {
            id: Set(record.id),
            agent_id: Set(record.agent_id.0),
            tenant_id: Set(record.tenant_id.0),
            user_id: Set(record.user_id.map(|id| id.0)),
            session_id: Set(record.session_id),
            status: Set(status),
            score: Set(record.score),
            feedback: Set(record.feedback.clone()),
            questions: Set(record.questions.clone()),
            answers: Set(record.answers.clone()),
            started_at: Set(record.started_at),
            completed_at: Set(record.completed_at),
            created_at: Set(record.created_at),
            updated_at: Set(record.updated_at),
        }
    }
}

#[async_trait]
impl InterviewRecordRepository for InterviewRecordRepositoryImpl {
    async fn create(&self, record: &InterviewRecord) -> Result<InterviewRecord> {
        let active_model = Self::domain_to_active_model(record);
        
        let result = entities::interview_record::Entity::insert(active_model)
            .exec(self.db.as_ref())
            .await?;

        let created = entities::interview_record::Entity::find_by_id(result.last_insert_id)
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| PlatformError::NotFound("Created interview record not found".to_string()))?;

        Ok(Self::entity_to_domain(created))
    }

    async fn update(&self, record: &InterviewRecord) -> Result<InterviewRecord> {
        let active_model = Self::domain_to_active_model(record);
        
        let updated = active_model
            .update(self.db.as_ref())
            .await?;

        Ok(Self::entity_to_domain(updated))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<InterviewRecord>> {
        let record = entities::interview_record::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await?;

        Ok(record.map(Self::entity_to_domain))
    }

    async fn find_by_agent(&self, agent_id: &AgentId) -> Result<Vec<InterviewRecord>> {
        let records = entities::interview_record::Entity::find()
            .filter(entities::interview_record::Column::AgentId.eq(agent_id.0))
            .all(self.db.as_ref())
            .await?;

        Ok(records.into_iter().map(Self::entity_to_domain).collect())
    }

    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<InterviewRecord>> {
        let records = entities::interview_record::Entity::find()
            .filter(entities::interview_record::Column::TenantId.eq(tenant_id.0))
            .all(self.db.as_ref())
            .await?;

        Ok(records.into_iter().map(Self::entity_to_domain).collect())
    }

    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<InterviewRecord>> {
        let records = entities::interview_record::Entity::find()
            .filter(entities::interview_record::Column::UserId.eq(user_id.0))
            .all(self.db.as_ref())
            .await?;

        Ok(records.into_iter().map(Self::entity_to_domain).collect())
    }

    async fn find_by_session(&self, session_id: Uuid) -> Result<Option<InterviewRecord>> {
        let record = entities::interview_record::Entity::find()
            .filter(entities::interview_record::Column::SessionId.eq(session_id))
            .one(self.db.as_ref())
            .await?;

        Ok(record.map(Self::entity_to_domain))
    }

    async fn find_by_agent_and_date_range(
        &self,
        agent_id: &AgentId,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<InterviewRecord>> {
        let records = entities::interview_record::Entity::find()
            .filter(entities::interview_record::Column::AgentId.eq(agent_id.0))
            .filter(entities::interview_record::Column::CreatedAt.gte(start_date))
            .filter(entities::interview_record::Column::CreatedAt.lte(end_date))
            .all(self.db.as_ref())
            .await?;

        Ok(records.into_iter().map(Self::entity_to_domain).collect())
    }

    async fn count_by_agent(&self, agent_id: &AgentId) -> Result<i64> {
        let count = entities::interview_record::Entity::find()
            .filter(entities::interview_record::Column::AgentId.eq(agent_id.0))
            .count(self.db.as_ref())
            .await?;

        Ok(count as i64)
    }

    async fn count_passed_by_agent(&self, agent_id: &AgentId) -> Result<i64> {
        let count = entities::interview_record::Entity::find()
            .filter(entities::interview_record::Column::AgentId.eq(agent_id.0))
            .filter(entities::interview_record::Column::Status.eq(entities::interview_record::InterviewStatus::Passed))
            .count(self.db.as_ref())
            .await?;

        Ok(count as i64)
    }
}
