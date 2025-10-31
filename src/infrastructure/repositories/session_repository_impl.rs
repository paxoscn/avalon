use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QuerySelect, PaginatorTrait, QueryOrder};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use crate::domain::entities::{ChatSession, Message};
use crate::domain::repositories::{ChatSessionRepository, MessageRepository};
use crate::domain::value_objects::{SessionId, TenantId, UserId, MessageId, SessionContext, ChatMessage, MessageRole};
use crate::infrastructure::database::entities;
use crate::error::{Result, PlatformError};

pub struct ChatSessionRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl ChatSessionRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_domain(entity: entities::chat_session::Model) -> Result<ChatSession> {
        let context: SessionContext = match entity.context {
            Some(json) => serde_json::from_value(json)
                .map_err(|e| PlatformError::ValidationError(format!("Invalid session context: {}", e)))?,
            None => SessionContext::new(),
        };

        Ok(ChatSession {
            id: SessionId::from_uuid(entity.id),
            tenant_id: TenantId::from_uuid(entity.tenant_id),
            user_id: UserId::from_uuid(entity.user_id),
            title: entity.title,
            context,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        })
    }

    fn domain_to_active_model(session: &ChatSession) -> Result<entities::chat_session::ActiveModel> {
        use sea_orm::ActiveValue::Set;
        
        let context_json = serde_json::to_value(&session.context)
            .map_err(|e| PlatformError::ValidationError(format!("Failed to serialize session context: {}", e)))?;

        Ok(entities::chat_session::ActiveModel {
            id: Set(session.id.0),
            tenant_id: Set(session.tenant_id.0),
            user_id: Set(session.user_id.0),
            title: Set(session.title.clone()),
            context: Set(Some(context_json)),
            created_at: Set(session.created_at),
            updated_at: Set(session.updated_at),
        })
    }
}

#[async_trait]
impl ChatSessionRepository for ChatSessionRepositoryImpl {
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<ChatSession>> {
        let session = entities::ChatSession::find_by_id(id.0)
            .one(self.db.as_ref())
            .await?;

        match session {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<ChatSession>> {
        let sessions = entities::ChatSession::find()
            .filter(entities::chat_session::Column::UserId.eq(user_id.0))
            .order_by_desc(entities::chat_session::Column::UpdatedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in sessions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<ChatSession>> {
        let sessions = entities::ChatSession::find()
            .filter(entities::chat_session::Column::TenantId.eq(tenant_id.0))
            .order_by_desc(entities::chat_session::Column::UpdatedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in sessions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_tenant_and_user(&self, tenant_id: &TenantId, user_id: &UserId) -> Result<Vec<ChatSession>> {
        let sessions = entities::ChatSession::find()
            .filter(entities::chat_session::Column::TenantId.eq(tenant_id.0))
            .filter(entities::chat_session::Column::UserId.eq(user_id.0))
            .order_by_desc(entities::chat_session::Column::UpdatedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in sessions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_active_by_user(&self, user_id: &UserId, timeout_minutes: u64) -> Result<Vec<ChatSession>> {
        let cutoff_time = Utc::now() - chrono::Duration::minutes(timeout_minutes as i64);
        
        let sessions = entities::ChatSession::find()
            .filter(entities::chat_session::Column::UserId.eq(user_id.0))
            .filter(entities::chat_session::Column::UpdatedAt.gt(cutoff_time))
            .order_by_desc(entities::chat_session::Column::UpdatedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in sessions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn save(&self, session: &ChatSession) -> Result<()> {
        let active_model = Self::domain_to_active_model(session)?;
        
        // Check if session exists
        let existing = entities::ChatSession::find_by_id(session.id.0)
            .one(self.db.as_ref())
            .await?;

        if existing.is_some() {
            // Update existing session
            entities::ChatSession::update(active_model)
                .exec(self.db.as_ref())
                .await?;
        } else {
            // Insert new session
            entities::ChatSession::insert(active_model)
                .exec(self.db.as_ref())
                .await?;
        }

        Ok(())
    }

    async fn delete(&self, id: &SessionId) -> Result<()> {
        entities::ChatSession::delete_by_id(id.0)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn delete_expired(&self, before: DateTime<Utc>) -> Result<u64> {
        let result = entities::ChatSession::delete_many()
            .filter(entities::chat_session::Column::UpdatedAt.lt(before))
            .exec(self.db.as_ref())
            .await?;

        Ok(result.rows_affected)
    }

    async fn count_by_user(&self, user_id: &UserId) -> Result<u64> {
        let count = entities::ChatSession::find()
            .filter(entities::chat_session::Column::UserId.eq(user_id.0))
            .count(self.db.as_ref())
            .await?;

        Ok(count)
    }

    async fn find_by_user_paginated(
        &self, 
        user_id: &UserId, 
        offset: u64, 
        limit: u64
    ) -> Result<Vec<ChatSession>> {
        let sessions = entities::ChatSession::find()
            .filter(entities::chat_session::Column::UserId.eq(user_id.0))
            .order_by_desc(entities::chat_session::Column::UpdatedAt)
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in sessions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }
}

pub struct MessageRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl MessageRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_domain(entity: entities::chat_message::Model) -> Result<Message> {
        let role = match entity.role {
            entities::chat_message::MessageRole::User => MessageRole::User,
            entities::chat_message::MessageRole::Assistant => MessageRole::Assistant,
            entities::chat_message::MessageRole::System => MessageRole::System,
        };

        let metadata = match entity.metadata {
            Some(json) => Some(serde_json::from_value(json)
                .map_err(|e| PlatformError::ValidationError(format!("Invalid message metadata: {}", e)))?),
            None => None,
        };

        let chat_message = ChatMessage {
            role,
            content: crate::domain::value_objects::chat_message::MessageContent::Text(entity.content),
            metadata,
            timestamp: entity.created_at,
        };

        Ok(Message {
            id: MessageId::from_uuid(entity.id),
            session_id: SessionId::from_uuid(entity.session_id),
            message: chat_message,
        })
    }

    fn domain_to_active_model(message: &Message) -> Result<entities::chat_message::ActiveModel> {
        use sea_orm::ActiveValue::Set;
        
        let role = match message.message.role {
            MessageRole::User => entities::chat_message::MessageRole::User,
            MessageRole::Assistant => entities::chat_message::MessageRole::Assistant,
            MessageRole::System => entities::chat_message::MessageRole::System,
            MessageRole::Tool => entities::chat_message::MessageRole::System, // Map Tool to System for now
        };

        let metadata_json = match &message.message.metadata {
            Some(metadata) => Some(serde_json::to_value(metadata)
                .map_err(|e| PlatformError::ValidationError(format!("Failed to serialize message metadata: {}", e)))?),
            None => None,
        };

        Ok(entities::chat_message::ActiveModel {
            id: Set(message.id.0),
            session_id: Set(message.session_id.0),
            role: Set(role),
            content: Set(message.message.get_text_content()),
            metadata: Set(metadata_json),
            created_at: Set(message.message.timestamp),
        })
    }
}

#[async_trait]
impl MessageRepository for MessageRepositoryImpl {
    async fn find_by_id(&self, id: &MessageId) -> Result<Option<Message>> {
        let message = entities::ChatMessage::find_by_id(id.0)
            .one(self.db.as_ref())
            .await?;

        match message {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_session(&self, session_id: &SessionId) -> Result<Vec<Message>> {
        let messages = entities::ChatMessage::find()
            .filter(entities::chat_message::Column::SessionId.eq(session_id.0))
            .order_by_asc(entities::chat_message::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in messages {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_recent_by_session(&self, session_id: &SessionId, limit: u64) -> Result<Vec<Message>> {
        let messages = entities::ChatMessage::find()
            .filter(entities::chat_message::Column::SessionId.eq(session_id.0))
            .order_by_desc(entities::chat_message::Column::CreatedAt)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in messages {
            result.push(Self::entity_to_domain(entity)?);
        }
        // Reverse to get chronological order
        result.reverse();
        Ok(result)
    }

    async fn find_by_session_paginated(
        &self, 
        session_id: &SessionId, 
        offset: u64, 
        limit: u64
    ) -> Result<Vec<Message>> {
        let messages = entities::ChatMessage::find()
            .filter(entities::chat_message::Column::SessionId.eq(session_id.0))
            .order_by_asc(entities::chat_message::Column::CreatedAt)
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in messages {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn save(&self, message: &Message) -> Result<()> {
        let active_model = Self::domain_to_active_model(message)?;
        
        // Check if message exists
        let existing = entities::ChatMessage::find_by_id(message.id.0)
            .one(self.db.as_ref())
            .await?;

        if existing.is_some() {
            // Update existing message
            entities::ChatMessage::update(active_model)
                .exec(self.db.as_ref())
                .await?;
        } else {
            // Insert new message
            entities::ChatMessage::insert(active_model)
                .exec(self.db.as_ref())
                .await?;
        }

        Ok(())
    }

    async fn delete(&self, id: &MessageId) -> Result<()> {
        entities::ChatMessage::delete_by_id(id.0)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn delete_by_session(&self, session_id: &SessionId) -> Result<()> {
        entities::ChatMessage::delete_many()
            .filter(entities::chat_message::Column::SessionId.eq(session_id.0))
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn count_by_session(&self, session_id: &SessionId) -> Result<u64> {
        let count = entities::ChatMessage::find()
            .filter(entities::chat_message::Column::SessionId.eq(session_id.0))
            .count(self.db.as_ref())
            .await?;

        Ok(count)
    }

    async fn search_by_content(
        &self, 
        session_id: &SessionId, 
        query: &str, 
        limit: u64
    ) -> Result<Vec<Message>> {
        let messages = entities::ChatMessage::find()
            .filter(entities::chat_message::Column::SessionId.eq(session_id.0))
            .filter(entities::chat_message::Column::Content.contains(query))
            .order_by_desc(entities::chat_message::Column::CreatedAt)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in messages {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }
}