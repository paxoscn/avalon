use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// MCP错误类型
#[derive(Debug, Error)]
pub enum MCPError {
    #[error("HTTP request failed: {0}")]
    HttpRequestFailed(String),
    
    #[error("Invalid tool configuration: {0}")]
    InvalidToolConfig(String),
    
    #[error("Parameter validation failed: {0}")]
    ParameterValidationFailed(String),
    
    #[error("Tool execution timeout")]
    ExecutionTimeout,
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<reqwest::Error> for MCPError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            MCPError::ExecutionTimeout
        } else if error.is_connect() {
            MCPError::NetworkError(error.to_string())
        } else {
            MCPError::HttpRequestFailed(error.to_string())
        }
    }
}

impl From<serde_json::Error> for MCPError {
    fn from(error: serde_json::Error) -> Self {
        MCPError::SerializationError(error.to_string())
    }
}

/// MCP协议错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPErrorResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl MCPErrorResponse {
    pub fn new(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// 创建参数错误响应
    pub fn invalid_params(message: String) -> Self {
        Self::new(-32602, message)
    }

    /// 创建方法未找到错误响应
    pub fn method_not_found(method: String) -> Self {
        Self::new(-32601, format!("Method not found: {}", method))
    }

    /// 创建内部错误响应
    pub fn internal_error(message: String) -> Self {
        Self::new(-32603, message)
    }

    /// 创建解析错误响应
    pub fn parse_error(message: String) -> Self {
        Self::new(-32700, message)
    }

    /// 创建无效请求错误响应
    pub fn invalid_request(message: String) -> Self {
        Self::new(-32600, message)
    }
}

impl fmt::Display for MCPErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MCP Error {}: {}", self.code, self.message)
    }
}

/// MCP错误处理工具
pub struct MCPErrorHandler;

impl MCPErrorHandler {
    /// 将MCPError转换为MCPErrorResponse
    pub fn to_mcp_error(error: MCPError) -> MCPErrorResponse {
        match error {
            MCPError::ParameterValidationFailed(msg) => MCPErrorResponse::invalid_params(msg),
            MCPError::ToolNotFound(tool) => MCPErrorResponse::method_not_found(tool),
            MCPError::SerializationError(msg) => MCPErrorResponse::parse_error(msg),
            MCPError::InvalidToolConfig(msg) => MCPErrorResponse::invalid_request(msg),
            _ => MCPErrorResponse::internal_error(error.to_string()),
        }
    }

    /// 处理HTTP错误状态码
    pub fn handle_http_status(status: reqwest::StatusCode, body: Option<String>) -> MCPError {
        match status {
            reqwest::StatusCode::BAD_REQUEST => {
                MCPError::ParameterValidationFailed(
                    body.unwrap_or_else(|| "Bad request".to_string())
                )
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                MCPError::AuthenticationFailed(
                    body.unwrap_or_else(|| "Unauthorized".to_string())
                )
            }
            reqwest::StatusCode::NOT_FOUND => {
                MCPError::ToolNotFound(
                    body.unwrap_or_else(|| "Tool not found".to_string())
                )
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                MCPError::RateLimitExceeded
            }
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                MCPError::InternalError(
                    body.unwrap_or_else(|| "Internal server error".to_string())
                )
            }
            _ => {
                MCPError::HttpRequestFailed(
                    format!("HTTP {}: {}", status, body.unwrap_or_default())
                )
            }
        }
    }

    /// 创建超时错误
    pub fn timeout_error(timeout_seconds: u64) -> MCPError {
        MCPError::ExecutionTimeout
    }

    /// 创建网络错误
    pub fn network_error(message: String) -> MCPError {
        MCPError::NetworkError(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_error_response_creation() {
        let error = MCPErrorResponse::invalid_params("Missing required parameter".to_string());
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Missing required parameter");
    }

    #[test]
    fn test_mcp_error_conversion() {
        let mcp_error = MCPError::ParameterValidationFailed("Invalid param".to_string());
        let error_response = MCPErrorHandler::to_mcp_error(mcp_error);
        assert_eq!(error_response.code, -32602);
    }

    #[test]
    fn test_http_status_handling() {
        let error = MCPErrorHandler::handle_http_status(
            reqwest::StatusCode::BAD_REQUEST,
            Some("Invalid input".to_string())
        );
        
        match error {
            MCPError::ParameterValidationFailed(msg) => {
                assert_eq!(msg, "Invalid input");
            }
            _ => panic!("Expected ParameterValidationFailed error"),
        }
    }
}