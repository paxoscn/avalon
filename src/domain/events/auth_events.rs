use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use super::{DomainEvent, EventMetadata};

/// User authentication successful event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserAuthenticatedEvent {
    pub metadata: EventMetadata,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub username: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl UserAuthenticatedEvent {
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        username: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        version: i64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(version),
            user_id,
            tenant_id,
            username,
            ip_address,
            user_agent,
        }
    }
}

impl DomainEvent for UserAuthenticatedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "UserAuthenticated"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn version(&self) -> i64 {
        self.metadata.version
    }
}

/// User authentication failed event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserAuthenticationFailedEvent {
    pub metadata: EventMetadata,
    pub tenant_id: Uuid,
    pub username: String,
    pub reason: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl UserAuthenticationFailedEvent {
    pub fn new(
        tenant_id: Uuid,
        username: String,
        reason: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        version: i64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(version),
            tenant_id,
            username,
            reason,
            ip_address,
            user_agent,
        }
    }
}

impl DomainEvent for UserAuthenticationFailedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "UserAuthenticationFailed"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_id(&self) -> Uuid {
        // Use tenant_id as aggregate since user might not exist
        self.tenant_id
    }

    fn version(&self) -> i64 {
        self.metadata.version
    }
}

/// User logged out event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLoggedOutEvent {
    pub metadata: EventMetadata,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub username: String,
    pub token_id: Uuid,
    pub ip_address: Option<String>,
}

impl UserLoggedOutEvent {
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        username: String,
        token_id: Uuid,
        ip_address: Option<String>,
        version: i64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(version),
            user_id,
            tenant_id,
            username,
            token_id,
            ip_address,
        }
    }
}

impl DomainEvent for UserLoggedOutEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "UserLoggedOut"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn version(&self) -> i64 {
        self.metadata.version
    }
}

/// Token refreshed event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenRefreshedEvent {
    pub metadata: EventMetadata,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub old_token_id: Uuid,
    pub new_token_id: Uuid,
    pub ip_address: Option<String>,
}

impl TokenRefreshedEvent {
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        old_token_id: Uuid,
        new_token_id: Uuid,
        ip_address: Option<String>,
        version: i64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(version),
            user_id,
            tenant_id,
            old_token_id,
            new_token_id,
            ip_address,
        }
    }
}

impl DomainEvent for TokenRefreshedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "TokenRefreshed"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn version(&self) -> i64 {
        self.metadata.version
    }
}

/// Password changed event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PasswordChangedEvent {
    pub metadata: EventMetadata,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub username: String,
    pub changed_by: Uuid,
}

impl PasswordChangedEvent {
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        username: String,
        changed_by: Uuid,
        version: i64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(version),
            user_id,
            tenant_id,
            username,
            changed_by,
        }
    }
}

impl DomainEvent for PasswordChangedEvent {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "PasswordChanged"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn version(&self) -> i64 {
        self.metadata.version
    }
}