use std::sync::Arc;
use crate::domain::entities::Message;
use crate::domain::repositories::{ChatSessionRepository, MessageRepository};
use crate::domain::services::SessionDomainService;
use crate::domain::value_objects::{SessionId, TenantId, UserId, MessageId};
use crate::error::{Result, PlatformError};

/// Application service for message storage and retrieval
pub struct MessageApplicationService {
    session_repo: Arc<dyn ChatSessionRepository>,
    message_repo: Arc<dyn MessageRepository>,
    domain_service: Arc<SessionDomainService>,
}

impl MessageApplicationService {
    pub fn new(
        session_repo: Arc<dyn ChatSessionRepository>,
        message_repo: Arc<dyn MessageRepository>,
        domain_service: Arc<SessionDomainService>,
    ) -> Self {
        Self {
            session_repo,
            message_repo,
            domain_service,
        }
    }

    /// Get message by ID with access validation
    pub async fn get_message(
        &self,
        message_id: &MessageId,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<Message> {
        let message = self
            .message_repo
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Message not found".to_string()))?;

        // Validate session access
        let session = self
            .session_repo
            .find_by_id(&message.session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        Ok(message)
    }

    /// Get all messages in a session
    pub async fn get_session_messages(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<Vec<Message>> {
        // Validate session access
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        // Get all messages
        self.message_repo.find_by_session(session_id).await
    }

    /// Get recent messages in a session
    pub async fn get_recent_messages(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        limit: u64,
    ) -> Result<Vec<Message>> {
        // Validate session access
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        // Get recent messages
        self.message_repo
            .find_recent_by_session(session_id, limit)
            .await
    }

    /// Get messages with pagination
    pub async fn get_messages_paginated(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Message>> {
        // Validate session access
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        // Get paginated messages
        self.message_repo
            .find_by_session_paginated(session_id, offset, limit)
            .await
    }

    /// Count messages in a session
    pub async fn count_session_messages(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<u64> {
        // Validate session access
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        // Count messages
        self.message_repo.count_by_session(session_id).await
    }

    /// Search messages by content
    pub async fn search_messages(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        query: &str,
        limit: u64,
    ) -> Result<Vec<Message>> {
        // Validate session access
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        // Validate query
        if query.trim().is_empty() {
            return Err(PlatformError::ValidationError(
                "Search query cannot be empty".to_string(),
            ));
        }

        if query.len() > 500 {
            return Err(PlatformError::ValidationError(
                "Search query cannot exceed 500 characters".to_string(),
            ));
        }

        // Search messages
        self.message_repo
            .search_by_content(session_id, query, limit)
            .await
    }

    /// Delete a message
    pub async fn delete_message(
        &self,
        message_id: &MessageId,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<()> {
        // Get message and validate access
        let message = self.get_message(message_id, tenant_id, user_id).await?;

        // Delete message
        self.message_repo.delete(&message.id).await?;

        Ok(())
    }

    /// Delete all messages in a session
    pub async fn delete_session_messages(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<()> {
        // Validate session access
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        // Delete all messages
        self.message_repo.delete_by_session(session_id).await?;

        Ok(())
    }

    /// Get conversation history for LLM context
    /// Returns messages formatted for LLM consumption
    pub async fn get_conversation_history(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        max_messages: Option<u64>,
    ) -> Result<Vec<Message>> {
        let limit = max_messages.unwrap_or(50);

        // Get recent messages
        self.get_recent_messages(session_id, tenant_id, user_id, limit)
            .await
    }

    /// Filter messages by role
    pub async fn get_messages_by_role(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        role: &str,
    ) -> Result<Vec<Message>> {
        let messages = self
            .get_session_messages(session_id, tenant_id, user_id)
            .await?;

        let filtered: Vec<Message> = messages
            .into_iter()
            .filter(|msg| {
                let msg_role = match &msg.message.role {
                    crate::domain::value_objects::MessageRole::User => "user",
                    crate::domain::value_objects::MessageRole::Assistant => "assistant",
                    crate::domain::value_objects::MessageRole::System => "system",
                    crate::domain::value_objects::MessageRole::Tool => "tool",
                };
                msg_role == role
            })
            .collect();

        Ok(filtered)
    }
}
