use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::value_objects::{AgentId, UserId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentAllocation {
    pub agent_id: AgentId,
    pub user_id: UserId,
    pub allocated_at: DateTime<Utc>,
}

impl AgentAllocation {
    pub fn new(agent_id: AgentId, user_id: UserId) -> Self {
        AgentAllocation {
            agent_id,
            user_id,
            allocated_at: Utc::now(),
        }
    }
}
