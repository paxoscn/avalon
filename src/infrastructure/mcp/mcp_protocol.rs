use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::domain::entities::mcp_tool::MCPTool;
use crate::domain::value_objects::tool_config::{ParameterSchema, ParameterType, ParameterPosition, ToolConfig};

/// MCP工具描述符 - 符合MCP协议的工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolDescriptor {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP工具列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolListResponse {
    pub tools: Vec<MCPToolDescriptor>,
}

/// MCP工具调用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolCallResponse {
    pub content: Vec<MCPContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isError")]
    pub is_error: Option<bool>,
}

/// MCP内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl MCPContent {
    /// 创建文本类型的内容
    pub fn text(text: String) -> Self {
        Self {
            content_type: "text".to_string(),
            text: Some(text),
        }
    }

    /// 创建错误类型的内容
    pub fn error(error_message: String) -> Self {
        Self {
            content_type: "text".to_string(),
            text: Some(error_message),
        }
    }
}

impl MCPToolCallResponse {
    /// 创建成功响应
    pub fn success(content: String) -> Self {
        Self {
            content: vec![MCPContent::text(content)],
            is_error: None,
        }
    }

    /// 创建错误响应
    pub fn error(error_message: String) -> Self {
        Self {
            content: vec![MCPContent::error(error_message)],
            is_error: Some(true),
        }
    }
}

/// 将MCPTool转换为MCP协议格式的工具描述符
pub fn tool_to_mcp_format(tool: &MCPTool) -> MCPToolDescriptor {
    let input_schema = match &tool.config {
        ToolConfig::HTTP(http_config) => parameters_to_json_schema(&http_config.parameters),
    };

    MCPToolDescriptor {
        name: tool.name.clone(),
        description: tool.description.clone(),
        input_schema,
    }
}

/// 将参数列表转换为JSON Schema格式
pub fn parameters_to_json_schema(parameters: &[ParameterSchema]) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for param in parameters {
        // 只包含body和header参数到JSON Schema中
        // path参数在URL中处理，不需要在inputSchema中定义
        if param.position == ParameterPosition::Path {
            continue;
        }

        let mut param_schema = serde_json::Map::new();

        // 设置类型
        let type_str = match param.parameter_type {
            ParameterType::String => "string",
            ParameterType::Number => "number",
            ParameterType::Boolean => "boolean",
            ParameterType::Object => "object",
            ParameterType::Array => "array",
        };
        param_schema.insert("type".to_string(), json!(type_str));

        // 设置描述
        if let Some(ref description) = param.description {
            param_schema.insert("description".to_string(), json!(description));
        }

        // 设置枚举值
        if let Some(ref enum_values) = param.enum_values {
            param_schema.insert("enum".to_string(), json!(enum_values));
        }

        // 设置默认值
        if let Some(ref default_value) = param.default_value {
            param_schema.insert("default".to_string(), default_value.clone());
        }

        properties.insert(param.name.clone(), Value::Object(param_schema));

        // 添加到required列表
        if param.required {
            required.push(param.name.clone());
        }
    }

    json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::ids::{TenantId, UserId};
    use crate::domain::value_objects::tool_config::{HTTPToolConfig, HttpMethod};

    #[test]
    fn test_mcp_content_text() {
        let content = MCPContent::text("Hello, world!".to_string());
        assert_eq!(content.content_type, "text");
        assert_eq!(content.text, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_mcp_content_error() {
        let content = MCPContent::error("Error occurred".to_string());
        assert_eq!(content.content_type, "text");
        assert_eq!(content.text, Some("Error occurred".to_string()));
    }

    #[test]
    fn test_mcp_tool_call_response_success() {
        let response = MCPToolCallResponse::success("Operation completed".to_string());
        assert_eq!(response.content.len(), 1);
        assert_eq!(response.content[0].text, Some("Operation completed".to_string()));
        assert_eq!(response.is_error, None);
    }

    #[test]
    fn test_mcp_tool_call_response_error() {
        let response = MCPToolCallResponse::error("Something went wrong".to_string());
        assert_eq!(response.content.len(), 1);
        assert_eq!(response.content[0].text, Some("Something went wrong".to_string()));
        assert_eq!(response.is_error, Some(true));
    }

    #[test]
    fn test_parameters_to_json_schema_basic() {
        let parameters = vec![
            ParameterSchema {
                name: "username".to_string(),
                parameter_type: ParameterType::String,
                description: Some("User name".to_string()),
                required: true,
                default_value: None,
                enum_values: None,
                position: ParameterPosition::Body,
            },
            ParameterSchema {
                name: "age".to_string(),
                parameter_type: ParameterType::Number,
                description: Some("User age".to_string()),
                required: false,
                default_value: Some(json!(18)),
                enum_values: None,
                position: ParameterPosition::Body,
            },
        ];

        let schema = parameters_to_json_schema(&parameters);
        
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["username"].is_object());
        assert_eq!(schema["properties"]["username"]["type"], "string");
        assert_eq!(schema["properties"]["username"]["description"], "User name");
        assert_eq!(schema["properties"]["age"]["type"], "number");
        assert_eq!(schema["properties"]["age"]["default"], 18);
        
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], "username");
    }

    #[test]
    fn test_parameters_to_json_schema_with_enum() {
        let parameters = vec![
            ParameterSchema {
                name: "status".to_string(),
                parameter_type: ParameterType::String,
                description: Some("Status".to_string()),
                required: true,
                default_value: None,
                enum_values: Some(vec![json!("active"), json!("inactive")]),
                position: ParameterPosition::Body,
            },
        ];

        let schema = parameters_to_json_schema(&parameters);
        
        let enum_values = schema["properties"]["status"]["enum"].as_array().unwrap();
        assert_eq!(enum_values.len(), 2);
        assert_eq!(enum_values[0], "active");
        assert_eq!(enum_values[1], "inactive");
    }

    #[test]
    fn test_parameters_to_json_schema_excludes_path_params() {
        let parameters = vec![
            ParameterSchema {
                name: "userId".to_string(),
                parameter_type: ParameterType::String,
                description: Some("User ID".to_string()),
                required: true,
                default_value: None,
                enum_values: None,
                position: ParameterPosition::Path,
            },
            ParameterSchema {
                name: "name".to_string(),
                parameter_type: ParameterType::String,
                description: Some("User name".to_string()),
                required: true,
                default_value: None,
                enum_values: None,
                position: ParameterPosition::Body,
            },
        ];

        let schema = parameters_to_json_schema(&parameters);
        
        // Path parameter should not be in the schema
        assert!(schema["properties"]["userId"].is_null());
        // Body parameter should be in the schema
        assert!(schema["properties"]["name"].is_object());
    }

    #[test]
    fn test_tool_to_mcp_format() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        
        let http_config = HTTPToolConfig {
            endpoint: "https://api.example.com/users".to_string(),
            method: HttpMethod::GET,
            headers: std::collections::HashMap::new(),
            parameters: vec![
                ParameterSchema {
                    name: "query".to_string(),
                    parameter_type: ParameterType::String,
                    description: Some("Search query".to_string()),
                    required: false,
                    default_value: None,
                    enum_values: None,
                    position: ParameterPosition::Body,
                },
            ],
            timeout_seconds: Some(30),
            retry_count: Some(3),
            response_template: None,
        };

        let tool = MCPTool::new(
            tenant_id,
            "search-users".to_string(),
            Some("Search for users".to_string()),
            ToolConfig::HTTP(http_config),
            user_id,
        );

        let descriptor = tool_to_mcp_format(&tool);
        
        assert_eq!(descriptor.name, "search-users");
        assert_eq!(descriptor.description, Some("Search for users".to_string()));
        assert_eq!(descriptor.input_schema["type"], "object");
        assert!(descriptor.input_schema["properties"]["query"].is_object());
    }

    #[test]
    fn test_mcp_tool_descriptor_serialization() {
        let descriptor = MCPToolDescriptor {
            name: "test-tool".to_string(),
            description: Some("Test tool".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "param1": {
                        "type": "string"
                    }
                },
                "required": ["param1"]
            }),
        };

        let json_str = serde_json::to_string(&descriptor).unwrap();
        assert!(json_str.contains("\"name\":\"test-tool\""));
        assert!(json_str.contains("\"inputSchema\""));
        
        let deserialized: MCPToolDescriptor = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.name, "test-tool");
    }
}
