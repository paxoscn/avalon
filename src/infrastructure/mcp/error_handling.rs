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
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Parameter validation failed: {0}")]
    ParameterValidationFailed(String),
    
    #[error("Path parameter missing: {0}")]
    PathParameterMissing(String),
    
    #[error("Path parameter invalid: {0}")]
    PathParameterInvalid(String),
    
    #[error("Template render error: {0}")]
    TemplateRenderError(String),
    
    #[error("Template syntax error: {0}")]
    TemplateSyntaxError(String),
    
    #[error("Parameter position mismatch: {0}")]
    ParameterPositionMismatch(String),
    
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

impl From<crate::infrastructure::mcp::template_engine::TemplateError> for MCPError {
    fn from(error: crate::infrastructure::mcp::template_engine::TemplateError) -> Self {
        use crate::infrastructure::mcp::template_engine::TemplateError;
        match error {
            TemplateError::SyntaxError(msg) => MCPError::TemplateSyntaxError(msg),
            TemplateError::RenderError(msg) => MCPError::TemplateRenderError(msg),
            TemplateError::CompilationError(msg) => MCPError::TemplateSyntaxError(msg),
        }
    }
}

impl From<url::ParseError> for MCPError {
    fn from(error: url::ParseError) -> Self {
        MCPError::ConfigurationError(format!("Invalid URL: {}", error))
    }
}

impl From<regex::Error> for MCPError {
    fn from(error: regex::Error) -> Self {
        MCPError::InternalError(format!("Regex error: {}", error))
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
            MCPError::PathParameterMissing(msg) => MCPErrorResponse::invalid_params(msg),
            MCPError::PathParameterInvalid(msg) => MCPErrorResponse::invalid_params(msg),
            MCPError::ParameterPositionMismatch(msg) => MCPErrorResponse::invalid_params(msg),
            MCPError::TemplateRenderError(msg) => MCPErrorResponse::internal_error(msg),
            MCPError::TemplateSyntaxError(msg) => MCPErrorResponse::invalid_request(msg),
            MCPError::ToolNotFound(tool) => MCPErrorResponse::method_not_found(tool),
            MCPError::SerializationError(msg) => MCPErrorResponse::parse_error(msg),
            MCPError::InvalidToolConfig(msg) => MCPErrorResponse::invalid_request(msg),
            MCPError::ConfigurationError(msg) => MCPErrorResponse::invalid_request(msg),
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
    pub fn timeout_error(_timeout_seconds: u64) -> MCPError {
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

    #[test]
    fn test_template_error_conversion() {
        use crate::infrastructure::mcp::template_engine::TemplateError;
        
        // Test SyntaxError conversion
        let syntax_error = TemplateError::SyntaxError("Invalid syntax".to_string());
        let mcp_error: MCPError = syntax_error.into();
        match mcp_error {
            MCPError::TemplateSyntaxError(msg) => {
                assert_eq!(msg, "Invalid syntax");
            }
            _ => panic!("Expected TemplateSyntaxError"),
        }
        
        // Test RenderError conversion
        let render_error = TemplateError::RenderError("Render failed".to_string());
        let mcp_error: MCPError = render_error.into();
        match mcp_error {
            MCPError::TemplateRenderError(msg) => {
                assert_eq!(msg, "Render failed");
            }
            _ => panic!("Expected TemplateRenderError"),
        }
        
        // Test CompilationError conversion
        let compilation_error = TemplateError::CompilationError("Compilation failed".to_string());
        let mcp_error: MCPError = compilation_error.into();
        match mcp_error {
            MCPError::TemplateSyntaxError(msg) => {
                assert_eq!(msg, "Compilation failed");
            }
            _ => panic!("Expected TemplateSyntaxError"),
        }
    }

    #[test]
    fn test_url_parse_error_conversion() {
        // Create an invalid URL to trigger ParseError
        let parse_result = url::Url::parse("not a valid url");
        assert!(parse_result.is_err());
        
        let parse_error = parse_result.unwrap_err();
        let mcp_error: MCPError = parse_error.into();
        
        match mcp_error {
            MCPError::ConfigurationError(msg) => {
                assert!(msg.contains("Invalid URL"));
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_regex_error_conversion() {
        // Create an invalid regex to trigger regex::Error
        let regex_result = regex::Regex::new("[invalid");
        assert!(regex_result.is_err());
        
        let regex_error = regex_result.unwrap_err();
        let mcp_error: MCPError = regex_error.into();
        
        match mcp_error {
            MCPError::InternalError(msg) => {
                assert!(msg.contains("Regex error"));
            }
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_path_parameter_errors() {
        // Test PathParameterMissing error
        let missing_error = MCPError::PathParameterMissing("userId".to_string());
        assert_eq!(missing_error.to_string(), "Path parameter missing: userId");
        
        // Test PathParameterInvalid error
        let invalid_error = MCPError::PathParameterInvalid("Invalid format".to_string());
        assert_eq!(invalid_error.to_string(), "Path parameter invalid: Invalid format");
    }

    #[test]
    fn test_template_errors() {
        // Test TemplateRenderError
        let render_error = MCPError::TemplateRenderError("Failed to render".to_string());
        assert_eq!(render_error.to_string(), "Template render error: Failed to render");
        
        // Test TemplateSyntaxError
        let syntax_error = MCPError::TemplateSyntaxError("Invalid syntax".to_string());
        assert_eq!(syntax_error.to_string(), "Template syntax error: Invalid syntax");
    }

    #[test]
    fn test_parameter_position_mismatch_error() {
        let mismatch_error = MCPError::ParameterPositionMismatch(
            "Parameter 'userId' is marked as path but not found in endpoint".to_string()
        );
        assert_eq!(
            mismatch_error.to_string(),
            "Parameter position mismatch: Parameter 'userId' is marked as path but not found in endpoint"
        );
    }

    #[test]
    fn test_mcp_error_handler_converts_new_errors() {
        // Test that new error types are properly converted to MCP error responses
        
        // PathParameterMissing -> invalid_params
        let missing_error = MCPError::PathParameterMissing("userId".to_string());
        let response = MCPErrorHandler::to_mcp_error(missing_error);
        assert_eq!(response.code, -32602);
        
        // TemplateSyntaxError -> invalid_request
        let syntax_error = MCPError::TemplateSyntaxError("Bad syntax".to_string());
        let response = MCPErrorHandler::to_mcp_error(syntax_error);
        assert_eq!(response.code, -32600);
        
        // TemplateRenderError -> internal_error
        let render_error = MCPError::TemplateRenderError("Render failed".to_string());
        let response = MCPErrorHandler::to_mcp_error(render_error);
        assert_eq!(response.code, -32603);
    }
}