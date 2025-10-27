use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{AgentId, UserId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentEmployment {
    pub agent_id: AgentId,
    pub user_id: UserId,
    pub employed_at: DateTime<Utc>,
}

impl AgentEmployment {
    pub fn new(agent_id: AgentId, user_id: UserId) -> Self {
        AgentEmployment {
            agent_id,
            user_id,
            employed_at: Utc::now(),
        }
    }
}
