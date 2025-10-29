use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// HTTP方法枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::PATCH => write!(f, "PATCH"),
        }
    }
}

/// 参数类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

/// 参数位置枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterPosition {
    Body,    // 请求体参数
    Header,  // HTTP头参数
    Path,    // 路径参数
}

impl Default for ParameterPosition {
    fn default() -> Self {
        ParameterPosition::Body
    }
}

/// 参数模式定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub name: String,
    pub parameter_type: ParameterType,
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub enum_values: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub position: ParameterPosition,
}

impl ParameterSchema {
    pub fn new(name: String, parameter_type: ParameterType, required: bool) -> Self {
        Self {
            name,
            parameter_type,
            description: None,
            required,
            default_value: None,
            enum_values: None,
            position: ParameterPosition::default(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_default(mut self, default_value: serde_json::Value) -> Self {
        self.default_value = Some(default_value);
        self
    }

    pub fn with_enum_values(mut self, enum_values: Vec<serde_json::Value>) -> Self {
        self.enum_values = Some(enum_values);
        self
    }

    pub fn with_position(mut self, position: ParameterPosition) -> Self {
        self.position = position;
        self
    }

    /// 验证参数值
    pub fn validate_value(&self, value: &serde_json::Value) -> Result<(), String> {
        // 检查必需参数
        if self.required && value.is_null() {
            return Err(format!("Parameter '{}' is required", self.name));
        }

        // 如果值为null且不是必需的，则跳过类型检查
        if value.is_null() {
            return Ok(());
        }

        // 类型检查
        match (&self.parameter_type, value) {
            (ParameterType::String, serde_json::Value::String(_)) => {},
            (ParameterType::Number, serde_json::Value::Number(_)) => {},
            (ParameterType::Boolean, serde_json::Value::Bool(_)) => {},
            (ParameterType::Object, serde_json::Value::Object(_)) => {},
            (ParameterType::Array, serde_json::Value::Array(_)) => {},
            _ => {
                return Err(format!(
                    "Parameter '{}' expected type {:?} but got {:?}",
                    self.name, self.parameter_type, value
                ));
            }
        }

        // 枚举值检查
        if let Some(enum_values) = &self.enum_values {
            if !enum_values.contains(value) {
                return Err(format!(
                    "Parameter '{}' value must be one of: {:?}",
                    self.name, enum_values
                ));
            }
        }

        Ok(())
    }
}

/// HTTP工具配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HTTPToolConfig {
    pub endpoint: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub parameters: Vec<ParameterSchema>,
    pub timeout_seconds: Option<u64>,
    pub retry_count: Option<u32>,
    pub response_template: Option<String>,
}

impl HTTPToolConfig {
    pub fn new(endpoint: String, method: HttpMethod) -> Self {
        Self {
            endpoint,
            method,
            headers: HashMap::new(),
            parameters: Vec::new(),
            timeout_seconds: Some(30),
            retry_count: Some(3),
            response_template: None,
        }
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn with_parameter(mut self, parameter: ParameterSchema) -> Self {
        self.parameters.push(parameter);
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }

    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.retry_count = Some(retry_count);
        self
    }

    pub fn with_response_template(mut self, response_template: String) -> Self {
        self.response_template = Some(response_template);
        self
    }

    /// 验证HTTP工具配置
    pub fn validate(&self) -> Result<(), String> {
        // 验证URL格式
        Url::parse(&self.endpoint)
            .map_err(|e| format!("Invalid endpoint URL: {}", e))?;

        // 验证参数名称唯一性
        let mut param_names = std::collections::HashSet::new();
        for param in &self.parameters {
            if !param_names.insert(&param.name) {
                return Err(format!("Duplicate parameter name: {}", param.name));
            }
        }

        // 验证超时时间
        if let Some(timeout) = self.timeout_seconds {
            if timeout == 0 || timeout > 300 {
                return Err("Timeout must be between 1 and 300 seconds".to_string());
            }
        }

        // 验证重试次数
        if let Some(retry_count) = self.retry_count {
            if retry_count > 10 {
                return Err("Retry count cannot exceed 10".to_string());
            }
        }

        // 验证路径参数一致性
        self.validate_path_parameters()?;

        // 验证header参数命名规范
        self.validate_header_parameters()?;

        Ok(())
    }

    /// 验证路径参数与endpoint的一致性
    fn validate_path_parameters(&self) -> Result<(), String> {
        // 提取endpoint中的所有路径参数占位符 {paramName}
        let placeholder_regex = regex::Regex::new(r"\{([^}]+)\}")
            .map_err(|e| format!("Failed to compile regex: {}", e))?;
        
        let mut placeholders = std::collections::HashSet::new();
        for cap in placeholder_regex.captures_iter(&self.endpoint) {
            if let Some(param_name) = cap.get(1) {
                placeholders.insert(param_name.as_str().to_string());
            }
        }

        // 收集所有position为path的参数
        let mut path_params = std::collections::HashSet::new();
        for param in &self.parameters {
            if param.position == ParameterPosition::Path {
                path_params.insert(param.name.clone());
            }
        }

        // 验证每个占位符都有对应的path参数
        for placeholder in &placeholders {
            if !path_params.contains(placeholder) {
                return Err(format!(
                    "Path parameter '{}' in endpoint has no corresponding parameter definition with position=path",
                    placeholder
                ));
            }
        }

        // 验证每个path参数都在endpoint中有对应的占位符
        for path_param in &path_params {
            if !placeholders.contains(path_param) {
                return Err(format!(
                    "Parameter '{}' has position=path but is not used in endpoint URL",
                    path_param
                ));
            }
        }

        Ok(())
    }

    /// 验证header参数命名规范
    fn validate_header_parameters(&self) -> Result<(), String> {
        // HTTP header名称规范：字母、数字、连字符
        let header_name_regex = regex::Regex::new(r"^[a-zA-Z0-9\-]+$")
            .map_err(|e| format!("Failed to compile regex: {}", e))?;

        for param in &self.parameters {
            if param.position == ParameterPosition::Header {
                if !header_name_regex.is_match(&param.name) {
                    return Err(format!(
                        "Header parameter '{}' has invalid name. Header names must contain only letters, numbers, and hyphens",
                        param.name
                    ));
                }
            }
        }

        Ok(())
    }

    /// 验证调用参数
    pub fn validate_call_parameters(&self, params: &serde_json::Value) -> Result<(), String> {
        let params_obj = params.as_object()
            .ok_or("Parameters must be a JSON object")?;

        // 验证每个定义的参数
        for param_schema in &self.parameters {
            let param_value = params_obj.get(&param_schema.name)
                .unwrap_or(&serde_json::Value::Null);
            param_schema.validate_value(param_value)?;
        }

        // 检查是否有未定义的参数
        for (param_name, _) in params_obj {
            if !self.parameters.iter().any(|p| &p.name == param_name) {
                return Err(format!("Unknown parameter: {}", param_name));
            }
        }

        Ok(())
    }
}

/// 工具配置枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolConfig {
    HTTP(HTTPToolConfig),
    // 未来可以扩展其他类型的工具配置
    // WebSocket(WebSocketToolConfig),
    // GraphQL(GraphQLToolConfig),
}

impl Default for ToolConfig {
    fn default() -> Self {
        ToolConfig::HTTP(HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::GET,
        ))
    }
}

impl ToolConfig {
    /// 验证工具配置
    pub fn validate(&self) -> Result<(), String> {
        match self {
            ToolConfig::HTTP(config) => config.validate(),
        }
    }

    /// 验证调用参数
    pub fn validate_call_parameters(&self, params: &serde_json::Value) -> Result<(), String> {
        match self {
            ToolConfig::HTTP(config) => config.validate_call_parameters(params),
        }
    }

    /// 获取工具类型字符串
    pub fn tool_type(&self) -> &'static str {
        match self {
            ToolConfig::HTTP(_) => "http",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parameter_schema_validation() {
        let param = ParameterSchema::new("test".to_string(), ParameterType::String, true);
        
        // Valid string value
        assert!(param.validate_value(&json!("hello")).is_ok());
        
        // Invalid type
        assert!(param.validate_value(&json!(123)).is_err());
        
        // Required parameter missing
        assert!(param.validate_value(&json!(null)).is_err());
    }

    #[test]
    fn test_parameter_schema_with_enum() {
        let param = ParameterSchema::new("status".to_string(), ParameterType::String, true)
            .with_enum_values(vec![json!("active"), json!("inactive")]);
        
        // Valid enum value
        assert!(param.validate_value(&json!("active")).is_ok());
        
        // Invalid enum value
        assert!(param.validate_value(&json!("unknown")).is_err());
    }

    #[test]
    fn test_parameter_position_default() {
        let param = ParameterSchema::new("test".to_string(), ParameterType::String, true);
        assert_eq!(param.position, ParameterPosition::Body);
    }

    #[test]
    fn test_parameter_position_serialization() {
        let param = ParameterSchema::new("test".to_string(), ParameterType::String, true)
            .with_position(ParameterPosition::Path);
        
        let json = serde_json::to_string(&param).unwrap();
        assert!(json.contains("\"position\":\"path\""));
        
        let deserialized: ParameterSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.position, ParameterPosition::Path);
    }

    #[test]
    fn test_http_tool_config_validation() {
        let config = HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::POST,
        );
        
        assert!(config.validate().is_ok());
        
        // Invalid URL
        let invalid_config = HTTPToolConfig::new(
            "not-a-url".to_string(),
            HttpMethod::GET,
        );
        
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_http_tool_config_parameter_validation() {
        let config = HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::POST,
        )
        .with_parameter(ParameterSchema::new("name".to_string(), ParameterType::String, true))
        .with_parameter(ParameterSchema::new("age".to_string(), ParameterType::Number, false));
        
        // Valid parameters
        let valid_params = json!({
            "name": "John",
            "age": 30
        });
        assert!(config.validate_call_parameters(&valid_params).is_ok());
        
        // Missing required parameter
        let invalid_params = json!({
            "age": 30
        });
        assert!(config.validate_call_parameters(&invalid_params).is_err());
        
        // Unknown parameter
        let unknown_params = json!({
            "name": "John",
            "unknown": "value"
        });
        assert!(config.validate_call_parameters(&unknown_params).is_err());
    }

    #[test]
    fn test_path_parameter_validation_success() {
        let config = HTTPToolConfig::new(
            "https://api.example.com/users/{userId}/orders/{orderId}".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("userId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path)
        )
        .with_parameter(
            ParameterSchema::new("orderId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path)
        );
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_path_parameter_validation_missing_placeholder() {
        let config = HTTPToolConfig::new(
            "https://api.example.com/users/{userId}".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("userId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path)
        )
        .with_parameter(
            ParameterSchema::new("orderId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path)
        );
        
        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("orderId"));
        assert!(error_msg.contains("not used in endpoint"));
    }

    #[test]
    fn test_path_parameter_validation_missing_definition() {
        let config = HTTPToolConfig::new(
            "https://api.example.com/users/{userId}/orders/{orderId}".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("userId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path)
        );
        
        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("orderId"));
        assert!(error_msg.contains("no corresponding parameter definition"));
    }

    #[test]
    fn test_header_parameter_validation_success() {
        let config = HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("Authorization".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Header)
        )
        .with_parameter(
            ParameterSchema::new("X-Custom-Header".to_string(), ParameterType::String, false)
                .with_position(ParameterPosition::Header)
        );
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_header_parameter_validation_invalid_name() {
        let config = HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("Invalid Header!".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Header)
        );
        
        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Invalid Header!"));
        assert!(error_msg.contains("invalid name"));
    }

    #[test]
    fn test_response_template_field() {
        let config = HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::GET,
        )
        .with_response_template("Result: {{ .data }}".to_string());
        
        assert_eq!(config.response_template, Some("Result: {{ .data }}".to_string()));
    }
    
    #[test]
    fn test_template_syntax_validation() {
        use crate::infrastructure::mcp::template_engine::ResponseTemplateEngine;
        
        let engine = ResponseTemplateEngine::new();
        
        // Valid template
        let valid_template = "Hello {{ name }}!";
        assert!(engine.validate_template(valid_template).is_ok());
        
        // Invalid template - missing closing brace
        let invalid_template = "Hello {{ name }";
        assert!(engine.validate_template(invalid_template).is_err());
        
        // Invalid template - unclosed block
        let invalid_block = "{{#if condition}}text";
        assert!(engine.validate_template(invalid_block).is_err());
    }
}