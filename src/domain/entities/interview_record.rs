use crate::domain::value_objects::{AgentId, TenantId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InterviewStatus {
    Pending,
    InProgress,
    Passed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterviewRecord {
    pub id: Uuid,
    pub agent_id: AgentId,
    pub tenant_id: TenantId,
    pub user_id: Option<UserId>,
    pub session_id: Option<Uuid>,
    pub status: InterviewStatus,
    pub score: Option<i32>,
    pub feedback: Option<String>,
    pub questions: Option<serde_json::Value>,
    pub answers: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InterviewRecord {
    pub fn new(agent_id: AgentId, tenant_id: TenantId, user_id: Option<UserId>) -> Self {
        let now = Utc::now();
        
        InterviewRecord {
            id: Uuid::new_v4(),
            agent_id,
            tenant_id,
            user_id,
            session_id: None,
            status: InterviewStatus::Pending,
            score: None,
            feedback: None,
            questions: None,
            answers: None,
            started_at: None,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn start(&mut self, session_id: Uuid) {
        self.status = InterviewStatus::InProgress;
        self.session_id = Some(session_id);
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn complete(&mut self, status: InterviewStatus, score: Option<i32>, feedback: Option<String>) {
        self.status = status;
        self.score = score;
        self.feedback = feedback;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn cancel(&mut self) {
        self.status = InterviewStatus::Cancelled;
        self.updated_at = Utc::now();
    }

    pub fn set_questions(&mut self, questions: serde_json::Value) {
        self.questions = Some(questions);
        self.updated_at = Utc::now();
    }

    pub fn set_answers(&mut self, answers: serde_json::Value) {
        self.answers = Some(answers);
        self.updated_at = Utc::now();
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.status, InterviewStatus::Passed | InterviewStatus::Failed)
    }

    pub fn is_passed(&self) -> bool {
        matches!(self.status, InterviewStatus::Passed)
    }

    pub fn duration_seconds(&self) -> Option<i64> {
        if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            Some((completed - started).num_seconds())
        } else {
            None
        }
    }
}
