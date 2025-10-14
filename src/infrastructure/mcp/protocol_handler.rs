use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{
    entities::MCPTool,
    value_objects::ids::{MCPToolId, TenantId},
};
use crate::infrastructure::mcp::{
    error_handling::{MCPError, MCPErrorResponse},
    http_converter::{HTTPToMCPConverter, MCPToolResult},
};

/// MCP协议版本
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// MCP请求消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

impl MCPRequest {
    pub fn new(method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::String(Uuid::new_v4().to_string())),
            method,
            params,
        }
    }

    pub fn with_id(mut self, id: Value) -> Self {
        self.id = Some(id);
        self
    }

    /// 验证请求格式
    pub fn validate(&self) -> Result<(), MCPError> {
        if self.jsonrpc != "2.0" {
            return Err(MCPError::InvalidToolConfig(
                "Invalid JSON-RPC version".to_string()
            ));
        }

        if self.method.is_empty() {
            return Err(MCPError::InvalidToolConfig(
                "Method cannot be empty".to_string()
            ));
        }

        Ok(())
    }
}

/// MCP响应消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<MCPErrorResponse>,
}

impl MCPResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, error: MCPErrorResponse) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }

    /// 检查响应是否成功
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
}

/// MCP工具信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolInfo {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

impl MCPToolInfo {
    pub fn from_mcp_tool(tool: &MCPTool) -> Self {
        let input_schema = Self::generate_input_schema(tool);
        
        Self {
            name: tool.name.clone(),
            description: tool.description.clone(),
            input_schema,
        }
    }

    /// 根据工具配置生成输入模式
    fn generate_input_schema(tool: &MCPTool) -> Value {
        match &tool.config {
            crate::domain::value_objects::tool_config::ToolConfig::HTTP(http_config) => {
                let mut properties = serde_json::Map::new();
                let mut required = Vec::new();

                for param in &http_config.parameters {
                    let param_schema = json!({
                        "type": Self::parameter_type_to_json_schema(&param.parameter_type),
                        "description": param.description.as_ref().unwrap_or(&"".to_string())
                    });

                    properties.insert(param.name.clone(), param_schema);

                    if param.required {
                        required.push(param.name.clone());
                    }
                }

                json!({
                    "type": "object",
                    "properties": properties,
                    "required": required
                })
            }
        }
    }

    /// 将参数类型转换为JSON Schema类型
    fn parameter_type_to_json_schema(param_type: &crate::domain::value_objects::tool_config::ParameterType) -> &'static str {
        match param_type {
            crate::domain::value_objects::tool_config::ParameterType::String => "string",
            crate::domain::value_objects::tool_config::ParameterType::Number => "number",
            crate::domain::value_objects::tool_config::ParameterType::Boolean => "boolean",
            crate::domain::value_objects::tool_config::ParameterType::Object => "object",
            crate::domain::value_objects::tool_config::ParameterType::Array => "array",
        }
    }
}

/// MCP协议处理器
#[derive(Clone)]
pub struct MCPProtocolHandler {
    converter: HTTPToMCPConverter,
    tools: HashMap<String, MCPTool>,
}

impl MCPProtocolHandler {
    pub fn new() -> Self {
        Self {
            converter: HTTPToMCPConverter::new(),
            tools: HashMap::new(),
        }
    }

    /// 注册工具
    pub fn register_tool(&mut self, tool: MCPTool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// 注销工具
    pub fn unregister_tool(&mut self, tool_name: &str) {
        self.tools.remove(tool_name);
    }

    /// 获取所有工具
    pub fn get_tools(&self) -> Vec<&MCPTool> {
        self.tools.values().collect()
    }

    /// 根据名称获取工具
    pub fn get_tool(&self, name: &str) -> Option<&MCPTool> {
        self.tools.get(name)
    }

    /// 处理MCP请求
    pub async fn handle_request(&self, request: MCPRequest) -> MCPResponse {
        // 验证请求
        if let Err(error) = request.validate() {
            return MCPResponse::error(
                request.id,
                crate::infrastructure::mcp::error_handling::MCPErrorHandler::to_mcp_error(error),
            );
        }

        // 根据方法分发请求
        match request.method.as_str() {
            "tools/list" => self.handle_list_tools(request).await,
            "tools/call" => self.handle_call_tool(request).await,
            "initialize" => self.handle_initialize(request).await,
            _ => MCPResponse::error(
                request.id,
                MCPErrorResponse::method_not_found(request.method),
            ),
        }
    }

    /// 处理初始化请求
    async fn handle_initialize(&self, request: MCPRequest) -> MCPResponse {
        let result = json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "agent-platform-mcp-server",
                "version": "1.0.0"
            }
        });

        MCPResponse::success(request.id, result)
    }

    /// 处理工具列表请求
    async fn handle_list_tools(&self, request: MCPRequest) -> MCPResponse {
        let tools: Vec<MCPToolInfo> = self.tools
            .values()
            .filter(|tool| tool.can_execute())
            .map(|tool| MCPToolInfo::from_mcp_tool(tool))
            .collect();

        let result = json!({
            "tools": tools
        });

        MCPResponse::success(request.id, result)
    }

    /// 处理工具调用请求
    async fn handle_call_tool(&self, request: MCPRequest) -> MCPResponse {
        let params = match request.params {
            Some(params) => params,
            None => {
                return MCPResponse::error(
                    request.id,
                    MCPErrorResponse::invalid_params("Missing parameters".to_string()),
                );
            }
        };

        // 解析工具调用参数
        let tool_call = match self.parse_tool_call_params(&params) {
            Ok(call) => call,
            Err(error) => {
                return MCPResponse::error(
                    request.id,
                    crate::infrastructure::mcp::error_handling::MCPErrorHandler::to_mcp_error(error),
                );
            }
        };

        // 获取工具
        let tool = match self.get_tool(&tool_call.name) {
            Some(tool) => tool,
            None => {
                return MCPResponse::error(
                    request.id,
                    MCPErrorResponse::method_not_found(tool_call.name),
                );
            }
        };

        // 检查工具是否可执行
        if !tool.can_execute() {
            return MCPResponse::error(
                request.id,
                MCPErrorResponse::internal_error("Tool is not active".to_string()),
            );
        }

        // 执行工具
        match self.converter.execute_tool(tool, &tool_call.arguments).await {
            Ok(result) => {
                let mcp_result = json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result.result).unwrap_or_default()
                    }],
                    "isError": false
                });
                MCPResponse::success(request.id, mcp_result)
            }
            Err(error) => {
                let error_result = json!({
                    "content": [{
                        "type": "text",
                        "text": error.to_string()
                    }],
                    "isError": true
                });
                MCPResponse::success(request.id, error_result)
            }
        }
    }

    /// 解析工具调用参数
    fn parse_tool_call_params(&self, params: &Value) -> Result<ToolCallParams, MCPError> {
        let name = params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::ParameterValidationFailed("Missing tool name".to_string()))?
            .to_string();

        let arguments = params.get("arguments")
            .cloned()
            .unwrap_or(json!({}));

        Ok(ToolCallParams { name, arguments })
    }

    /// 批量注册工具
    pub fn register_tools(&mut self, tools: Vec<MCPTool>) {
        for tool in tools {
            self.register_tool(tool);
        }
    }

    /// 清空所有工具
    pub fn clear_tools(&mut self) {
        self.tools.clear();
    }

    /// 获取工具数量
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// 获取活跃工具数量
    pub fn active_tool_count(&self) -> usize {
        self.tools.values().filter(|tool| tool.can_execute()).count()
    }
}

impl Default for MCPProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具调用参数
#[derive(Debug, Clone)]
struct ToolCallParams {
    name: String,
    arguments: Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{
        ids::{TenantId, UserId},
        tool_config::{HTTPToolConfig, HttpMethod, ToolConfig},
    };

    fn create_test_tool() -> MCPTool {
        let config = HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::GET,
        );

        let mut tool = MCPTool::new(
            TenantId::new(),
            "test-tool".to_string(),
            Some("Test tool".to_string()),
            ToolConfig::HTTP(config),
            UserId::new(),
        );
        tool.activate();
        tool
    }

    #[test]
    fn test_mcp_request_validation() {
        let valid_request = MCPRequest::new("tools/list".to_string(), None);
        assert!(valid_request.validate().is_ok());

        let invalid_request = MCPRequest {
            jsonrpc: "1.0".to_string(),
            id: None,
            method: "test".to_string(),
            params: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_mcp_response_creation() {
        let success_response = MCPResponse::success(
            Some(json!("test-id")),
            json!({"result": "success"})
        );
        assert!(success_response.is_success());

        let error_response = MCPResponse::error(
            Some(json!("test-id")),
            MCPErrorResponse::internal_error("Test error".to_string())
        );
        assert!(!error_response.is_success());
    }

    #[test]
    fn test_mcp_tool_info_generation() {
        let tool = create_test_tool();
        let tool_info = MCPToolInfo::from_mcp_tool(&tool);
        
        assert_eq!(tool_info.name, "test-tool");
        assert_eq!(tool_info.description, Some("Test tool".to_string()));
    }

    #[test]
    fn test_protocol_handler_tool_registration() {
        let mut handler = MCPProtocolHandler::new();
        let tool = create_test_tool();
        
        assert_eq!(handler.tool_count(), 0);
        
        handler.register_tool(tool);
        assert_eq!(handler.tool_count(), 1);
        assert_eq!(handler.active_tool_count(), 1);
        
        handler.unregister_tool("test-tool");
        assert_eq!(handler.tool_count(), 0);
    }

    #[tokio::test]
    async fn test_handle_initialize_request() {
        let handler = MCPProtocolHandler::new();
        let request = MCPRequest::new("initialize".to_string(), None);
        
        let response = handler.handle_request(request).await;
        assert!(response.is_success());
        
        if let Some(result) = response.result {
            assert_eq!(result["protocolVersion"], MCP_PROTOCOL_VERSION);
        }
    }

    #[tokio::test]
    async fn test_handle_list_tools_request() {
        let mut handler = MCPProtocolHandler::new();
        handler.register_tool(create_test_tool());
        
        let request = MCPRequest::new("tools/list".to_string(), None);
        let response = handler.handle_request(request).await;
        
        assert!(response.is_success());
        
        if let Some(result) = response.result {
            let tools = result["tools"].as_array().unwrap();
            assert_eq!(tools.len(), 1);
            assert_eq!(tools[0]["name"], "test-tool");
        }
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let handler = MCPProtocolHandler::new();
        let request = MCPRequest::new("unknown/method".to_string(), None);
        
        let response = handler.handle_request(request).await;
        assert!(!response.is_success());
        
        if let Some(error) = response.error {
            assert_eq!(error.code, -32601); // Method not found
        }
    }
}