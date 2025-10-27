use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use chrono::Utc;

pub type Result<T> = std::result::Result<T, PlatformError>;

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("Flow execution failed: {0}")]
    FlowExecutionFailed(String),
    
    #[error("DSL parsing failed: {0}")]
    DSLParsingFailed(String),
    
    #[error("LLM provider error: {0}")]
    LLMProviderError(String),
    
    #[error("Vector store error: {0}")]
    VectorStoreError(String),
    
    #[error("MCP tool error: {0}")]
    MCPToolError(String),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Agent unauthorized: {0}")]
    AgentUnauthorized(String),
    
    #[error("Agent validation error: {0}")]
    AgentValidationError(String),
    
    #[error("Agent already employed: {0}")]
    AgentAlreadyEmployed(String),
    
    #[error("Agent not employed: {0}")]
    AgentNotEmployed(String),
    
    #[error("Preset questions limit exceeded")]
    PresetQuestionsLimitExceeded,
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    

    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl From<String> for PlatformError {
    fn from(msg: String) -> Self {
        PlatformError::InternalError(msg)
    }
}

impl From<crate::domain::services::llm_service::LLMError> for PlatformError {
    fn from(err: crate::domain::services::llm_service::LLMError) -> Self {
        match err {
            crate::domain::services::llm_service::LLMError::InvalidConfiguration(msg) => PlatformError::ValidationError(msg),
            crate::domain::services::llm_service::LLMError::AuthenticationFailed(msg) => PlatformError::AuthenticationFailed(msg),
            crate::domain::services::llm_service::LLMError::NetworkError(msg) => PlatformError::InternalError(format!("Network error: {}", msg)),
            _ => PlatformError::InternalError(err.to_string()),
        }
    }
}

impl IntoResponse for PlatformError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            PlatformError::AuthenticationFailed(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            PlatformError::AuthorizationFailed(_) => (StatusCode::FORBIDDEN, self.to_string()),
            PlatformError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            PlatformError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            PlatformError::ConfigurationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            PlatformError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
            PlatformError::AgentNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            PlatformError::AgentUnauthorized(_) => (StatusCode::FORBIDDEN, self.to_string()),
            PlatformError::AgentValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            PlatformError::AgentAlreadyEmployed(_) => (StatusCode::CONFLICT, self.to_string()),
            PlatformError::AgentNotEmployed(_) => (StatusCode::NOT_FOUND, self.to_string()),
            PlatformError::PresetQuestionsLimitExceeded => (StatusCode::BAD_REQUEST, self.to_string()),
            // _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            // FIXME For debugging only.
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", self.to_string())),
        };

        let body = Json(json!({
            "error": error_message,
            "timestamp": Utc::now().to_rfc3339()
        }));

        (status, body).into_response()
    }
}

// Convenience macros for error creation
#[macro_export]
macro_rules! auth_error {
    ($msg:expr) => {
        $crate::error::PlatformError::AuthenticationFailed($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::PlatformError::AuthenticationFailed(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        $crate::error::PlatformError::ValidationError($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::PlatformError::ValidationError(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! not_found_error {
    ($msg:expr) => {
        $crate::error::PlatformError::NotFound($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::PlatformError::NotFound(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! internal_error {
    ($msg:expr) => {
        $crate::error::PlatformError::InternalError($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::PlatformError::InternalError(format!($fmt, $($arg)*))
    };
}

// Agent-specific error macros
#[macro_export]
macro_rules! agent_not_found {
    ($msg:expr) => {
        $crate::error::PlatformError::AgentNotFound($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::PlatformError::AgentNotFound(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! agent_unauthorized {
    ($msg:expr) => {
        $crate::error::PlatformError::AgentUnauthorized($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::PlatformError::AgentUnauthorized(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! agent_validation_error {
    ($msg:expr) => {
        $crate::error::PlatformError::AgentValidationError($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::PlatformError::AgentValidationError(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! agent_already_employed {
    ($msg:expr) => {
        $crate::error::PlatformError::AgentAlreadyEmployed($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::PlatformError::AgentAlreadyEmployed(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! agent_not_employed {
    ($msg:expr) => {
        $crate::error::PlatformError::AgentNotEmployed($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::PlatformError::AgentNotEmployed(format!($fmt, $($arg)*))
    };
}