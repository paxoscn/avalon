use std::sync::Arc;
use crate::domain::entities::{ChatSession, Message};
use crate::domain::repositories::{ChatSessionRepository, MessageRepository};
use crate::domain::services::SessionDomainService;
use crate::domain::value_objects::{SessionId, TenantId, UserId, ChatMessage, MessageRole};
use crate::error::{Result, PlatformError};
use serde_json::{json, Value};

/// Service for managing conversation context
pub struct ContextManagementService {
    session_repo: Arc<dyn ChatSessionRepository>,
    message_repo: Arc<dyn MessageRepository>,
    domain_service: Arc<SessionDomainService>,
    max_context_messages: usize,
    max_context_tokens: usize,
}

impl ContextManagementService {
    pub fn new(
        session_repo: Arc<dyn ChatSessionRepository>,
        message_repo: Arc<dyn MessageRepository>,
        domain_service: Arc<SessionDomainService>,
    ) -> Self {
        Self {
            session_repo,
            message_repo,
            domain_service,
            max_context_messages: 50,
            max_context_tokens: 4000,
        }
    }

    pub fn with_limits(mut self, max_messages: usize, max_tokens: usize) -> Self {
        self.max_context_messages = max_messages;
        self.max_context_tokens = max_tokens;
        self
    }

    /// Extract conversation context for flow execution
    pub async fn extract_context_for_flow(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<Value> {
        // Validate session access
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        // Get recent messages
        let messages = self
            .message_repo
            .find_recent_by_session(session_id, self.max_context_messages as u64)
            .await?;

        // Build context object
        let context = self.build_context_object(&session, &messages)?;

        Ok(context)
    }

    /// Build context object from session and messages
    fn build_context_object(&self, session: &ChatSession, messages: &[Message]) -> Result<Value> {
        let mut message_history = Vec::new();

        for msg in messages {
            let role_str = match msg.message.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
                MessageRole::Tool => "tool",
            };

            let mut msg_obj = json!({
                "role": role_str,
                "content": msg.message.content,
                "timestamp": msg.message.timestamp.to_rfc3339(),
            });

            // Add metadata if present
            if let Some(ref metadata) = msg.message.metadata {
                msg_obj["metadata"] = serde_json::to_value(metadata)
                    .map_err(|e| PlatformError::InternalError(format!("Failed to serialize metadata: {}", e)))?;
            }

            message_history.push(msg_obj);
        }

        let context = json!({
            "session_id": session.id.0.to_string(),
            "tenant_id": session.tenant_id.0.to_string(),
            "user_id": session.user_id.0.to_string(),
            "message_count": session.get_message_count(),
            "message_history": message_history,
            "session_variables": session.context.variables,
            "conversation_summary": session.context.conversation_summary,
            "last_activity": session.context.last_activity.to_rfc3339(),
        });

        Ok(context)
    }

    /// Compress context by summarizing old messages
    pub async fn compress_context(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        summary: String,
    ) -> Result<()> {
        // Validate session access
        let mut session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        // Update session summary
        self.domain_service
            .update_session_summary(&mut session, summary)?;

        // Persist changes
        self.session_repo.save(&session).await?;

        Ok(())
    }

    /// Get context window for LLM
    /// Returns messages formatted for LLM with token estimation
    pub async fn get_context_window(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<Vec<ChatMessage>> {
        // Validate session access
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        // Get recent messages
        let messages = self
            .message_repo
            .find_recent_by_session(session_id, self.max_context_messages as u64)
            .await?;

        // Apply token-based truncation
        let context_messages = self.apply_token_limit(messages)?;

        Ok(context_messages)
    }

    /// Apply token limit to messages
    fn apply_token_limit(&self, messages: Vec<Message>) -> Result<Vec<ChatMessage>> {
        let mut result = Vec::new();
        let mut estimated_tokens = 0;

        // Process messages in reverse order (most recent first)
        for msg in messages.iter().rev() {
            // Rough token estimation: ~4 characters per token
            let msg_tokens = msg.message.get_text_content().len() / 4;

            if estimated_tokens + msg_tokens > self.max_context_tokens {
                break;
            }

            result.push(msg.message.clone());
            estimated_tokens += msg_tokens;
        }

        // Reverse to get chronological order
        result.reverse();

        Ok(result)
    }

    /// Merge context from multiple sessions
    pub async fn merge_session_contexts(
        &self,
        session_ids: &[SessionId],
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<Value> {
        let mut merged_messages = Vec::new();
        let mut merged_variables = serde_json::Map::new();

        for session_id in session_ids {
            // Validate session access
            let session = self
                .session_repo
                .find_by_id(session_id)
                .await?
                .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

            self.domain_service
                .validate_session_access(&session, tenant_id, user_id)?;

            // Get messages
            let messages = self
                .message_repo
                .find_recent_by_session(session_id, 10)
                .await?;

            for msg in messages {
                merged_messages.push(json!({
                    "session_id": session_id.0.to_string(),
                    "role": format!("{:?}", msg.message.role),
                    "content": msg.message.content,
                    "timestamp": msg.message.timestamp.to_rfc3339(),
                }));
            }

            // Merge variables
            for (key, value) in &session.context.variables {
                merged_variables.insert(key.clone(), value.clone());
            }
        }

        Ok(json!({
            "merged_messages": merged_messages,
            "merged_variables": merged_variables,
        }))
    }

    /// Extract key information from context for flow execution
    pub async fn extract_flow_parameters(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        parameter_keys: &[String],
    ) -> Result<Value> {
        // Validate session access
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        let mut parameters = serde_json::Map::new();

        for key in parameter_keys {
            if let Some(value) = session.context.variables.get(key) {
                parameters.insert(key.clone(), value.clone());
            }
        }

        Ok(Value::Object(parameters))
    }

    /// Store flow execution results back to session context
    pub async fn store_flow_results(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
        results: Value,
    ) -> Result<()> {
        // Validate session access
        let mut session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        // Store results in session context
        self.domain_service
            .set_session_context(&mut session, "last_flow_results".to_string(), results)?;

        // Persist changes
        self.session_repo.save(&session).await?;

        Ok(())
    }

    /// Get context statistics
    pub async fn get_context_stats(
        &self,
        session_id: &SessionId,
        tenant_id: &TenantId,
        user_id: &UserId,
    ) -> Result<Value> {
        // Validate session access
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Session not found".to_string()))?;

        self.domain_service
            .validate_session_access(&session, tenant_id, user_id)?;

        // Get message count
        let message_count = self.message_repo.count_by_session(session_id).await?;

        // Get recent messages for token estimation
        let messages = self
            .message_repo
            .find_recent_by_session(session_id, 50)
            .await?;

        let total_chars: usize = messages.iter().map(|m| m.message.get_text_content().len()).sum();
        let estimated_tokens = total_chars / 4;

        Ok(json!({
            "session_id": session_id.0.to_string(),
            "total_messages": message_count,
            "recent_messages": messages.len(),
            "estimated_tokens": estimated_tokens,
            "variable_count": session.context.variables.len(),
            "has_summary": session.context.conversation_summary.is_some(),
            "last_activity": session.context.last_activity.to_rfc3339(),
        }))
    }
}
