use async_trait::async_trait;
use crate::domain::entities::{ChatSession, Message};
use crate::domain::value_objects::{SessionId, TenantId, UserId, MessageId};
use crate::error::Result;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait ChatSessionRepository: Send + Sync {
    /// Find a session by ID
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<ChatSession>>;
    
    /// Find sessions by user
    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<ChatSession>>;
    
    /// Find sessions by tenant
    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<ChatSession>>;
    
    /// Find sessions by tenant and user
    async fn find_by_tenant_and_user(&self, tenant_id: &TenantId, user_id: &UserId) -> Result<Vec<ChatSession>>;
    
    /// Find active sessions (not expired)
    async fn find_active_by_user(&self, user_id: &UserId, timeout_minutes: u64) -> Result<Vec<ChatSession>>;
    
    /// Save a session (create or update)
    async fn save(&self, session: &ChatSession) -> Result<()>;
    
    /// Delete a session by ID
    async fn delete(&self, id: &SessionId) -> Result<()>;
    
    /// Delete expired sessions
    async fn delete_expired(&self, before: DateTime<Utc>) -> Result<u64>;
    
    /// Count sessions by user
    async fn count_by_user(&self, user_id: &UserId) -> Result<u64>;
    
    /// Find sessions with pagination
    async fn find_by_user_paginated(
        &self, 
        user_id: &UserId, 
        offset: u64, 
        limit: u64
    ) -> Result<Vec<ChatSession>>;
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    /// Find a message by ID
    async fn find_by_id(&self, id: &MessageId) -> Result<Option<Message>>;
    
    /// Find messages by session
    async fn find_by_session(&self, session_id: &SessionId) -> Result<Vec<Message>>;
    
    /// Find recent messages by session
    async fn find_recent_by_session(&self, session_id: &SessionId, limit: u64) -> Result<Vec<Message>>;
    
    /// Find messages by session with pagination
    async fn find_by_session_paginated(
        &self, 
        session_id: &SessionId, 
        offset: u64, 
        limit: u64
    ) -> Result<Vec<Message>>;
    
    /// Save a message
    async fn save(&self, message: &Message) -> Result<()>;
    
    /// Delete a message by ID
    async fn delete(&self, id: &MessageId) -> Result<()>;
    
    /// Delete all messages in a session
    async fn delete_by_session(&self, session_id: &SessionId) -> Result<()>;
    
    /// Count messages in a session
    async fn count_by_session(&self, session_id: &SessionId) -> Result<u64>;
    
    /// Find messages by content search
    async fn search_by_content(
        &self, 
        session_id: &SessionId, 
        query: &str, 
        limit: u64
    ) -> Result<Vec<Message>>;
}