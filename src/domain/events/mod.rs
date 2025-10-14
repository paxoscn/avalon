pub mod auth_events;
pub mod audit_events;

pub use auth_events::*;
pub use audit_events::*;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync {
    fn event_id(&self) -> Uuid;
    fn event_type(&self) -> &'static str;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn aggregate_id(&self) -> Uuid;
    fn version(&self) -> i64;
}

/// Event metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
}

impl EventMetadata {
    pub fn new(version: i64) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            version,
            correlation_id: None,
            causation_id: None,
        }
    }

    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_causation(mut self, causation_id: Uuid) -> Self {
        self.causation_id = Some(causation_id);
        self
    }
}