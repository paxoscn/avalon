use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::domain::value_objects::{SessionId, TenantId, UserId, ChatMessage, SessionContext, MessageId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: SessionId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub title: Option<String>,
    pub context: SessionContext,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub session_id: SessionId,
    pub message: ChatMessage,
}

impl ChatSession {
    pub fn new(tenant_id: TenantId, user_id: UserId, title: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            tenant_id,
            user_id,
            title,
            context: SessionContext::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_title(&mut self, title: Option<String>) -> Result<(), String> {
        if let Some(ref t) = title {
            if t.len() > 255 {
                return Err("Session title cannot exceed 255 characters".to_string());
            }
        }
        
        self.title = title;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_message(&mut self, message: ChatMessage) -> Result<(), String> {
        message.validate()?;
        
        self.context.increment_message_count();
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_context_variable(&mut self, key: String, value: serde_json::Value) {
        self.context.set_variable(key, value);
        self.updated_at = Utc::now();
    }

    pub fn get_context_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.context.get_variable(key)
    }

    pub fn update_summary(&mut self, summary: String) {
        self.context.conversation_summary = Some(summary);
        self.context.update_activity();
        self.updated_at = Utc::now();
    }

    pub fn is_expired(&self, timeout_minutes: u64) -> bool {
        self.context.is_expired(timeout_minutes)
    }

    pub fn belongs_to_tenant(&self, tenant_id: &TenantId) -> bool {
        &self.tenant_id == tenant_id
    }

    pub fn belongs_to_user(&self, user_id: &UserId) -> bool {
        &self.user_id == user_id
    }

    pub fn get_message_count(&self) -> u32 {
        self.context.message_count
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref title) = self.title {
            if title.len() > 255 {
                return Err("Session title cannot exceed 255 characters".to_string());
            }
        }
        Ok(())
    }
}

impl Message {
    pub fn new(session_id: SessionId, message: ChatMessage) -> Result<Self, String> {
        message.validate()?;
        
        Ok(Self {
            id: MessageId::new(),
            session_id,
            message,
        })
    }

    pub fn belongs_to_session(&self, session_id: &SessionId) -> bool {
        &self.session_id == session_id
    }

    pub fn validate(&self) -> Result<(), String> {
        self.message.validate()
    }
}