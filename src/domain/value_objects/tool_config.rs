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

/// 参数模式定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub name: String,
    pub parameter_type: ParameterType,
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub enum_values: Option<Vec<serde_json::Value>>,
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
}