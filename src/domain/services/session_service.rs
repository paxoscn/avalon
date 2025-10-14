use crate::domain::entities::{ChatSession, Message};
use crate::domain::value_objects::{TenantId, UserId, ChatMessage};
use crate::error::{Result, PlatformError};
use chrono::{DateTime, Utc};

/// Domain service for session lifecycle management
pub struct SessionDomainService {
    default_timeout_minutes: u64,
}

impl SessionDomainService {
    pub fn new(default_timeout_minutes: u64) -> Self {
        Self {
            default_timeout_minutes,
        }
    }

    /// Create a new chat session
    pub fn create_session(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        title: Option<String>,
    ) -> Result<ChatSession> {
        let session = ChatSession::new(tenant_id, user_id, title);
        session.validate()?;
        Ok(session)
    }

    /// Validate session ownership
    pub fn validate_session_access(
        &self,
        session: &ChatSession,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<()> {
        if !session.belongs_to_tenant(tenant_id) {
            return Err(PlatformError::AuthorizationFailed(
                "Session does not belong to the specified tenant".to_string(),
            ));
        }

        if !session.belongs_to_user(user_id) {
            return Err(PlatformError::AuthorizationFailed(
                "Session does not belong to the specified user".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if session is expired
    pub fn is_session_expired(&self, session: &ChatSession) -> bool {
        session.is_expired(self.default_timeout_minutes)
    }

    /// Check if session is expired with custom timeout
    pub fn is_session_expired_with_timeout(
        &self,
        session: &ChatSession,
        timeout_minutes: u64,
    ) -> bool {
        session.is_expired(timeout_minutes)
    }

    /// Add a message to session and update context
    pub fn add_message_to_session(
        &self,
        session: &mut ChatSession,
        message: ChatMessage,
    ) -> Result<Message> {
        // Validate message
        message.validate()?;

        // Add message to session (updates context)
        session.add_message(message.clone())?;

        // Create message entity
        let msg = Message::new(session.id.clone(), message)?;
        Ok(msg)
    }

    /// Update session title
    pub fn update_session_title(
        &self,
        session: &mut ChatSession,
        title: Option<String>,
    ) -> Result<()> {
        session.update_title(title)?;
        Ok(())
    }

    /// Update session summary for context compression
    pub fn update_session_summary(
        &self,
        session: &mut ChatSession,
        summary: String,
    ) -> Result<()> {
        if summary.len() > 5000 {
            return Err(PlatformError::ValidationError(
                "Session summary cannot exceed 5000 characters".to_string(),
            ));
        }

        session.update_summary(summary);
        Ok(())
    }

    /// Set context variable in session
    pub fn set_session_context(
        &self,
        session: &mut ChatSession,
        key: String,
        value: serde_json::Value,
    ) -> Result<()> {
        if key.is_empty() {
            return Err(PlatformError::ValidationError(
                "Context key cannot be empty".to_string(),
            ));
        }

        session.set_context_variable(key, value);
        Ok(())
    }

    /// Get context variable from session
    pub fn get_session_context(
        &self,
        session: &ChatSession,
        key: &str,
    ) -> Option<serde_json::Value> {
        session.get_context_variable(key).cloned()
    }

    /// Calculate session expiration time
    pub fn calculate_expiration_time(&self, from: DateTime<Utc>) -> DateTime<Utc> {
        from - chrono::Duration::minutes(self.default_timeout_minutes as i64)
    }

    /// Validate message for session
    pub fn validate_message(&self, message: &ChatMessage) -> Result<()> {
        message.validate()
            .map_err(|e| PlatformError::ValidationError(e))
    }

    /// Get default timeout
    pub fn default_timeout(&self) -> u64 {
        self.default_timeout_minutes
    }
}

impl Default for SessionDomainService {
    fn default() -> Self {
        Self::new(60) // Default 60 minutes timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let service = SessionDomainService::default();
        let tenant_id = TenantId::new();
        let user_id = UserId::new();

        let session = service
            .create_session(tenant_id.clone(), user_id.clone(), Some("Test Session".to_string()))
            .unwrap();

        assert_eq!(session.tenant_id, tenant_id);
        assert_eq!(session.user_id, user_id);
        assert_eq!(session.title, Some("Test Session".to_string()));
    }

    #[test]
    fn test_validate_session_access() {
        let service = SessionDomainService::default();
        let tenant_id = TenantId::new();
        let user_id = UserId::new();

        let session = service
            .create_session(tenant_id.clone(), user_id.clone(), None)
            .unwrap();

        // Valid access
        assert!(service
            .validate_session_access(&session, &tenant_id, &user_id)
            .is_ok());

        // Invalid tenant
        let other_tenant = TenantId::new();
        assert!(service
            .validate_session_access(&session, &other_tenant, &user_id)
            .is_err());

        // Invalid user
        let other_user = UserId::new();
        assert!(service
            .validate_session_access(&session, &tenant_id, &other_user)
            .is_err());
    }

    #[test]
    fn test_add_message_to_session() {
        let service = SessionDomainService::default();
        let tenant_id = TenantId::new();
        let user_id = UserId::new();

        let mut session = service
            .create_session(tenant_id, user_id, None)
            .unwrap();

        let message = ChatMessage::new_user_message("Hello".to_string());

        let result = service.add_message_to_session(&mut session, message);
        assert!(result.is_ok());
        assert_eq!(session.get_message_count(), 1);
    }

    #[test]
    fn test_session_expiration() {
        let service = SessionDomainService::new(30); // 30 minutes timeout
        let tenant_id = TenantId::new();
        let user_id = UserId::new();

        let session = service
            .create_session(tenant_id, user_id, None)
            .unwrap();

        // New session should not be expired
        assert!(!service.is_session_expired(&session));

        // Test with custom timeout
        assert!(!service.is_session_expired_with_timeout(&session, 60));
    }
}
