use std::sync::Arc;
use crate::domain::entities::{ChatSession, Message};
use crate::domain::repositories::{ChatSessionRepository, MessageRepository};
use crate::domain::services::SessionDomainService;
use crate::domain::value_objects::{SessionId, TenantId, UserId, ChatMessage};
use crate::error::{Result, PlatformError};
use chrono::Utc;
use tokio::time::{interval, Duration};

/// Application service for session lifecycle management
pub struct SessionApplicationService {
    session_repo: Arc<dyn ChatSessionRepository>,
    message_repo: Arc<dyn MessageRepository>,
    domain_service: Arc<SessionDomainService>,
}

impl SessionApplicationService {
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

    /// Create a new chat session
    pub async fn create_session(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        title: Option<String>,
    ) -> Result<ChatSession> {
        // Create session using domain service
        let session = self.domain_service.create_session(tenant_id, user_id, title)?;

        // Persist session
        self.session_repo.save(&session).await?;

        Ok(session)
    }

    /// Get session by ID with access validation
    pub async fn get_session(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<ChatSession> {
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        // Validate access
        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        Ok(session)
    }

    /// List sessions for a user with pagination
    pub async fn list_user_sessions(
        &self,
        _tenant_id: TenantId,
        user_id: &UserId,
        page: u64,
        limit: u64,
    ) -> Result<(Vec<ChatSession>, u64)> {
        let offset = page * limit;
        let sessions = self.session_repo
            .find_by_user_paginated(user_id, offset, limit)
            .await?;
        let total = self.session_repo.count_by_user(user_id).await?;
        Ok((sessions, total))
    }

    /// List active sessions for a user
    pub async fn list_active_sessions(&self, user_id: &UserId) -> Result<Vec<ChatSession>> {
        let timeout = self.domain_service.default_timeout();
        self.session_repo
            .find_active_by_user(user_id, timeout)
            .await
    }

    /// Update session title
    pub async fn update_session_title(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        title: Option<String>,
    ) -> Result<ChatSession> {
        let mut session = self.get_session(session_id, tenant_id, user_id).await?;

        // Update title using domain service
        self.domain_service.update_session_title(&mut session, title)?;

        // Persist changes
        self.session_repo.save(&session).await?;

        Ok(session)
    }

    /// Add message to session
    pub async fn add_message(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        message: ChatMessage,
    ) -> Result<Message> {
        let mut session = self.get_session(session_id, tenant_id, user_id).await?;

        // Add message using domain service
        let msg = self
            .domain_service
            .add_message_to_session(&mut session, message)?;

        // Persist message
        self.message_repo.save(&msg).await?;

        // Update session
        self.session_repo.save(&session).await?;

        Ok(msg)
    }

    /// Set context variable in session
    pub async fn set_context_variable(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        key: String,
        value: serde_json::Value,
    ) -> Result<()> {
        let mut session = self.get_session(session_id, tenant_id, user_id).await?;

        // Set context using domain service
        self.domain_service
            .set_session_context(&mut session, key, value)?;

        // Persist changes
        self.session_repo.save(&session).await?;

        Ok(())
    }

    /// Get context variable from session
    pub async fn get_context_variable(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        key: &str,
    ) -> Result<Option<serde_json::Value>> {
        let session = self.get_session(session_id, tenant_id, user_id).await?;

        Ok(self
            .domain_service
            .get_session_context(&session, key))
    }

    /// Get all messages from a session
    pub async fn get_session_messages(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<Vec<Message>> {
        // Validate access first
        let _session = self.get_session(session_id, tenant_id, user_id).await?;

        // Get all messages for the session
        self.message_repo.find_by_session(session_id).await
    }

    /// Update session summary for context compression
    pub async fn update_session_summary(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        summary: String,
    ) -> Result<()> {
        let mut session = self.get_session(session_id, tenant_id, user_id).await?;

        // Update summary using domain service
        self.domain_service
            .update_session_summary(&mut session, summary)?;

        // Persist changes
        self.session_repo.save(&session).await?;

        Ok(())
    }

    /// Delete session
    pub async fn delete_session(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<()> {
        // Validate access first
        let session = self.get_session(session_id, tenant_id, user_id).await?;

        // Delete all messages in session
        self.message_repo.delete_by_session(&session.id).await?;

        // Delete session
        self.session_repo.delete(&session.id).await?;

        Ok(())
    }

    /// Count sessions for a user
    pub async fn count_user_sessions(&self, user_id: &UserId) -> Result<u64> {
        self.session_repo.count_by_user(user_id).await
    }

    /// Clean up expired sessions (background task)
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let cutoff_time = self.domain_service.calculate_expiration_time(Utc::now());
        self.session_repo.delete_expired(cutoff_time).await
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>, interval_minutes: u64) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_minutes * 60));

            loop {
                ticker.tick().await;

                match self.cleanup_expired_sessions().await {
                    Ok(count) => {
                        if count > 0 {
                            log::info!("Cleaned up {} expired sessions", count);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to cleanup expired sessions: {}", e);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::{ChatSessionRepository, MessageRepository};
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use mockall::mock;
    use std::sync::Arc;

    mock! {
        ChatSessionRepositoryImpl {}

        #[async_trait]
        impl ChatSessionRepository for ChatSessionRepositoryImpl {
            async fn find_by_id(&self, id: &SessionId) -> Result<Option<ChatSession>>;
            async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<ChatSession>>;
            async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<ChatSession>>;
            async fn find_by_tenant_and_user(&self, tenant_id: &TenantId, user_id: &UserId) -> Result<Vec<ChatSession>>;
            async fn find_active_by_user(&self, user_id: &UserId, timeout_minutes: u64) -> Result<Vec<ChatSession>>;
            async fn save(&self, session: &ChatSession) -> Result<()>;
            async fn delete(&self, id: &SessionId) -> Result<()>;
            async fn delete_expired(&self, before: DateTime<Utc>) -> Result<u64>;
            async fn count_by_user(&self, user_id: &UserId) -> Result<u64>;
            async fn find_by_user_paginated(&self, user_id: &UserId, offset: u64, limit: u64) -> Result<Vec<ChatSession>>;
        }
    }

    mock! {
        MessageRepositoryImpl {}

        #[async_trait]
        impl MessageRepository for MessageRepositoryImpl {
            async fn find_by_id(&self, id: &crate::domain::value_objects::ids::MessageId) -> Result<Option<Message>>;
            async fn find_by_session(&self, session_id: &SessionId) -> Result<Vec<Message>>;
            async fn find_recent_by_session(&self, session_id: &SessionId, limit: u64) -> Result<Vec<Message>>;
            async fn find_by_session_paginated(&self, session_id: &SessionId, offset: u64, limit: u64) -> Result<Vec<Message>>;
            async fn save(&self, message: &Message) -> Result<()>;
            async fn delete(&self, id: &crate::domain::value_objects::ids::MessageId) -> Result<()>;
            async fn delete_by_session(&self, session_id: &SessionId) -> Result<()>;
            async fn count_by_session(&self, session_id: &SessionId) -> Result<u64>;
            async fn search_by_content(&self, session_id: &SessionId, query: &str, limit: u64) -> Result<Vec<Message>>;
        }
    }



    #[tokio::test]
    async fn test_list_user_sessions_zero_based() {
        let mut session_repo = MockChatSessionRepositoryImpl::new();
        let message_repo = MockMessageRepositoryImpl::new();
        let domain_service = Arc::new(SessionDomainService::new(30));

        let user_id = UserId::new();
        let tenant_id = TenantId::new();

        // Mock find_by_user_paginated to return empty vec
        session_repo
            .expect_find_by_user_paginated()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        // Mock count_by_user to return total count
        session_repo
            .expect_count_by_user()
            .times(1)
            .returning(|_| Ok(25));

        let service = SessionApplicationService::new(
            Arc::new(session_repo),
            Arc::new(message_repo),
            domain_service,
        );

        // Test page 0 (first page) with limit 10
        let result = service.list_user_sessions(tenant_id, &user_id, 0, 10).await;

        assert!(result.is_ok());
        let (sessions, total) = result.unwrap();
        assert_eq!(sessions.len(), 0);
        assert_eq!(total, 25);
    }

    #[tokio::test]
    async fn test_list_user_sessions_offset_calculation() {
        let mut session_repo = MockChatSessionRepositoryImpl::new();
        let message_repo = MockMessageRepositoryImpl::new();
        let domain_service = Arc::new(SessionDomainService::new(30));

        let user_id = UserId::new();
        let tenant_id = TenantId::new();

        // Verify that offset is calculated as page * limit
        session_repo
            .expect_find_by_user_paginated()
            .times(1)
            .withf(|_, offset, limit| {
                // For page=2, limit=15, offset should be 30
                *offset == 30 && *limit == 15
            })
            .returning(|_, _, _| Ok(vec![]));

        session_repo
            .expect_count_by_user()
            .times(1)
            .returning(|_| Ok(50));

        let service = SessionApplicationService::new(
            Arc::new(session_repo),
            Arc::new(message_repo),
            domain_service,
        );

        // Test page 2 with limit 15 (offset should be 2 * 15 = 30)
        let result = service.list_user_sessions(tenant_id, &user_id, 2, 15).await;

        assert!(result.is_ok());
        let (_, total) = result.unwrap();
        assert_eq!(total, 50);
    }

    #[tokio::test]
    async fn test_list_user_sessions_total_count_accuracy() {
        let mut session_repo = MockChatSessionRepositoryImpl::new();
        let message_repo = MockMessageRepositoryImpl::new();
        let domain_service = Arc::new(SessionDomainService::new(30));

        let user_id = UserId::new();
        let tenant_id = TenantId::new();

        session_repo
            .expect_find_by_user_paginated()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        // Verify total count is returned accurately
        session_repo
            .expect_count_by_user()
            .times(1)
            .returning(|_| Ok(17));

        let service = SessionApplicationService::new(
            Arc::new(session_repo),
            Arc::new(message_repo),
            domain_service,
        );

        let result = service.list_user_sessions(tenant_id, &user_id, 0, 20).await;

        assert!(result.is_ok());
        let (_, total) = result.unwrap();
        assert_eq!(total, 17);
    }
}
