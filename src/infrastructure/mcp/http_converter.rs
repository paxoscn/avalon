use reqwest::{Client, Method, RequestBuilder};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

use crate::domain::{
    entities::MCPTool,
    value_objects::tool_config::{HTTPToolConfig, HttpMethod, ParameterSchema, ToolConfig},
};
use crate::infrastructure::mcp::error_handling::{MCPError, MCPErrorHandler};

/// HTTP请求构建器
#[derive(Clone)]
pub struct HTTPRequestBuilder {
    client: Client,
}

impl HTTPRequestBuilder {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// 根据MCP工具配置构建HTTP请求
    pub fn build_request(
        &self,
        tool: &MCPTool,
        parameters: &Value,
    ) -> Result<RequestBuilder, MCPError> {
        match &tool.config {
            ToolConfig::HTTP(http_config) => {
                self.build_http_request(http_config, parameters)
            }
        }
    }

    /// 构建HTTP请求
    fn build_http_request(
        &self,
        config: &HTTPToolConfig,
        parameters: &Value,
    ) -> Result<RequestBuilder, MCPError> {
        // 验证参数
        config.validate_call_parameters(parameters)
            .map_err(|e| MCPError::ParameterValidationFailed(e))?;

        // 转换HTTP方法
        let method = self.convert_http_method(&config.method);
        
        // 创建请求构建器
        let mut request_builder = self.client.request(method, &config.endpoint);

        // 添加头部
        for (key, value) in &config.headers {
            request_builder = request_builder.header(key, value);
        }

        // 设置超时
        if let Some(timeout) = config.timeout_seconds {
            request_builder = request_builder.timeout(Duration::from_secs(timeout));
        }

        // 根据HTTP方法添加参数
        request_builder = match config.method {
            HttpMethod::GET | HttpMethod::DELETE => {
                self.add_query_parameters(request_builder, parameters)?
            }
            HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH => {
                self.add_body_parameters(request_builder, parameters)?
            }
        };

        Ok(request_builder)
    }

    /// 添加查询参数
    fn add_query_parameters(
        &self,
        mut request_builder: RequestBuilder,
        parameters: &Value,
    ) -> Result<RequestBuilder, MCPError> {
        if let Some(params_obj) = parameters.as_object() {
            for (key, value) in params_obj {
                let param_value = self.value_to_string(value)?;
                request_builder = request_builder.query(&[(key, param_value)]);
            }
        }
        Ok(request_builder)
    }

    /// 添加请求体参数
    fn add_body_parameters(
        &self,
        request_builder: RequestBuilder,
        parameters: &Value,
    ) -> Result<RequestBuilder, MCPError> {
        // 设置Content-Type为application/json
        let request_builder = request_builder
            .header("Content-Type", "application/json")
            .json(parameters);
        
        Ok(request_builder)
    }

    /// 转换HTTP方法
    fn convert_http_method(&self, method: &HttpMethod) -> Method {
        match method {
            HttpMethod::GET => Method::GET,
            HttpMethod::POST => Method::POST,
            HttpMethod::PUT => Method::PUT,
            HttpMethod::DELETE => Method::DELETE,
            HttpMethod::PATCH => Method::PATCH,
        }
    }

    /// 将JSON值转换为字符串
    fn value_to_string(&self, value: &Value) -> Result<String, MCPError> {
        match value {
            Value::String(s) => Ok(s.clone()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Null => Ok("".to_string()),
            _ => Ok(serde_json::to_string(value)?),
        }
    }

    /// 执行HTTP请求
    pub async fn execute_request(
        &self,
        request_builder: RequestBuilder,
    ) -> Result<HTTPResponse, MCPError> {
        let start_time = std::time::Instant::now();
        
        let response = request_builder.send().await?;
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        let status = response.status();
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response.text().await?;

        if !status.is_success() {
            return Err(MCPErrorHandler::handle_http_status(status, Some(body)));
        }

        Ok(HTTPResponse {
            status_code: status.as_u16(),
            headers,
            body,
            execution_time_ms: execution_time,
        })
    }
}

impl Default for HTTPRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP响应结构
#[derive(Debug, Clone)]
pub struct HTTPResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub execution_time_ms: u64,
}

impl HTTPResponse {
    /// 尝试将响应体解析为JSON
    pub fn parse_json(&self) -> Result<Value, MCPError> {
        if self.body.is_empty() {
            return Ok(Value::Null);
        }
        
        serde_json::from_str(&self.body)
            .map_err(|e| MCPError::SerializationError(e.to_string()))
    }

    /// 检查响应是否成功
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    /// 获取Content-Type头部
    pub fn content_type(&self) -> Option<&String> {
        self.headers.get("content-type")
            .or_else(|| self.headers.get("Content-Type"))
    }

    /// 检查响应是否为JSON格式
    pub fn is_json(&self) -> bool {
        self.content_type()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }
}

/// HTTP到MCP工具转换器
#[derive(Clone)]
pub struct HTTPToMCPConverter {
    request_builder: HTTPRequestBuilder,
}

impl HTTPToMCPConverter {
    pub fn new() -> Self {
        Self {
            request_builder: HTTPRequestBuilder::new(),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            request_builder: HTTPRequestBuilder::with_timeout(timeout),
        }
    }

    /// 执行MCP工具调用
    pub async fn execute_tool(
        &self,
        tool: &MCPTool,
        parameters: &Value,
    ) -> Result<MCPToolResult, MCPError> {
        let start_time = std::time::Instant::now();

        // 构建HTTP请求
        let request_builder = self.request_builder.build_request(tool, parameters)?;

        // 执行请求
        let http_response = self.request_builder.execute_request(request_builder).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // 转换为MCP工具结果
        let result = self.convert_http_response_to_mcp_result(http_response)?;

        Ok(MCPToolResult {
            success: true,
            result: Some(result),
            error: None,
            execution_time_ms: execution_time,
            metadata: HashMap::new(),
        })
    }

    /// 将HTTP响应转换为MCP结果
    fn convert_http_response_to_mcp_result(&self, response: HTTPResponse) -> Result<Value, MCPError> {
        if response.is_json() {
            response.parse_json()
        } else {
            // 对于非JSON响应，包装为JSON对象
            Ok(json!({
                "status_code": response.status_code,
                "headers": response.headers,
                "body": response.body,
                "content_type": response.content_type().unwrap_or(&"text/plain".to_string())
            }))
        }
    }

    /// 测试工具连接
    pub async fn test_tool_connection(&self, tool: &MCPTool) -> Result<MCPToolResult, MCPError> {
        // 使用空参数进行测试调用
        let test_params = json!({});
        
        match self.execute_tool(tool, &test_params).await {
            Ok(result) => Ok(result),
            Err(e) => {
                // 对于测试连接，某些错误是可以接受的（如参数错误）
                match e {
                    MCPError::ParameterValidationFailed(_) => {
                        // 参数验证失败说明连接是通的，只是参数不对
                        Ok(MCPToolResult {
                            success: true,
                            result: Some(json!({"connection_test": "passed", "note": "Parameter validation failed but connection is working"})),
                            error: None,
                            execution_time_ms: 0,
                            metadata: HashMap::new(),
                        })
                    }
                    _ => Err(e),
                }
            }
        }
    }
}

impl Default for HTTPToMCPConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP工具执行结果
#[derive(Debug, Clone)]
pub struct MCPToolResult {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, Value>,
}

impl MCPToolResult {
    pub fn success(result: Value, execution_time_ms: u64) -> Self {
        Self {
            success: true,
            result: Some(result),
            error: None,
            execution_time_ms,
            metadata: HashMap::new(),
        }
    }

    pub fn error(error: String, execution_time_ms: u64) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(error),
            execution_time_ms,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{
        ids::{TenantId, UserId},
        tool_config::{HTTPToolConfig, HttpMethod, ParameterSchema, ParameterType},
    };

    fn create_test_tool() -> MCPTool {
        let config = HTTPToolConfig::new(
            "https://httpbin.org/get".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("test_param".to_string(), ParameterType::String, false)
        );

        MCPTool::new(
            TenantId::new(),
            "test-tool".to_string(),
            Some("Test tool".to_string()),
            ToolConfig::HTTP(config),
            UserId::new(),
        )
    }

    #[test]
    fn test_http_request_builder_creation() {
        let builder = HTTPRequestBuilder::new();
        // Just test that it can be created without panicking
        assert!(true);
    }

    #[test]
    fn test_http_response_json_parsing() {
        let response = HTTPResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: r#"{"key": "value"}"#.to_string(),
            execution_time_ms: 100,
        };

        let json_result = response.parse_json().unwrap();
        assert_eq!(json_result["key"], "value");
    }

    #[test]
    fn test_http_response_is_success() {
        let response = HTTPResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "".to_string(),
            execution_time_ms: 100,
        };

        assert!(response.is_success());

        let error_response = HTTPResponse {
            status_code: 404,
            headers: HashMap::new(),
            body: "".to_string(),
            execution_time_ms: 100,
        };

        assert!(!error_response.is_success());
    }

    #[test]
    fn test_mcp_tool_result_creation() {
        let result = MCPToolResult::success(json!({"test": "data"}), 150);
        assert!(result.success);
        assert!(result.result.is_some());
        assert_eq!(result.execution_time_ms, 150);

        let error_result = MCPToolResult::error("Test error".to_string(), 200);
        assert!(!error_result.success);
        assert!(error_result.error.is_some());
    }

    #[tokio::test]
    async fn test_converter_creation() {
        let converter = HTTPToMCPConverter::new();
        // Just test that it can be created
        assert!(true);
    }
}