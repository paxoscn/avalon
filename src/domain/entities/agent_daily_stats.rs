use crate::domain::value_objects::{AgentId, TenantId};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDailyStats {
    pub id: Uuid,
    pub agent_id: AgentId,
    pub tenant_id: TenantId,
    pub stat_date: NaiveDate,
    pub interview_count: i64,
    pub interview_passed_count: i64,
    pub employment_count: i64,
    pub session_count: i64,
    pub message_count: i64,
    pub token_count: i64,
    pub revenue: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AgentDailyStats {
    pub fn new(agent_id: AgentId, tenant_id: TenantId, stat_date: NaiveDate) -> Self {
        let now = Utc::now();
        
        AgentDailyStats {
            id: Uuid::new_v4(),
            agent_id,
            tenant_id,
            stat_date,
            interview_count: 0,
            interview_passed_count: 0,
            employment_count: 0,
            session_count: 0,
            message_count: 0,
            token_count: 0,
            revenue: 0.0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn increment_interview(&mut self) {
        self.interview_count += 1;
        self.updated_at = Utc::now();
    }

    pub fn increment_interview_passed(&mut self) {
        self.interview_passed_count += 1;
        self.updated_at = Utc::now();
    }

    pub fn increment_employment(&mut self) {
        self.employment_count += 1;
        self.updated_at = Utc::now();
    }

    pub fn increment_session(&mut self) {
        self.session_count += 1;
        self.updated_at = Utc::now();
    }

    pub fn add_messages(&mut self, count: i64) {
        self.message_count += count;
        self.updated_at = Utc::now();
    }

    pub fn add_tokens(&mut self, count: i64) {
        self.token_count += count;
        self.updated_at = Utc::now();
    }

    pub fn add_revenue(&mut self, amount: f64) {
        self.revenue += amount;
        self.updated_at = Utc::now();
    }

    pub fn get_interview_pass_rate(&self) -> f64 {
        if self.interview_count == 0 {
            return 0.0;
        }
        (self.interview_passed_count as f64 / self.interview_count as f64) * 100.0
    }

    pub fn get_employment_rate(&self) -> f64 {
        if self.interview_passed_count == 0 {
            return 0.0;
        }
        (self.employment_count as f64 / self.interview_passed_count as f64) * 100.0
    }

    pub fn get_average_messages_per_session(&self) -> f64 {
        if self.session_count == 0 {
            return 0.0;
        }
        self.message_count as f64 / self.session_count as f64
    }

    pub fn get_average_tokens_per_message(&self) -> f64 {
        if self.message_count == 0 {
            return 0.0;
        }
        self.token_count as f64 / self.message_count as f64
    }

    pub fn get_average_revenue_per_session(&self) -> f64 {
        if self.session_count == 0 {
            return 0.0;
        }
        self.revenue / self.session_count as f64
    }
}
