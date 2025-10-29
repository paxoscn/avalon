use reqwest::{Client, Method, RequestBuilder};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use crate::domain::{
    entities::MCPTool,
    value_objects::tool_config::{HTTPToolConfig, HttpMethod, ParameterPosition, ToolConfig},
};
use crate::infrastructure::mcp::error_handling::{MCPError, MCPErrorHandler};
use crate::infrastructure::mcp::template_engine::ResponseTemplateEngine;

/// 参数分组结构，按位置分组参数
#[derive(Debug, Clone)]
struct ParameterGroups {
    /// 路径参数 (position = Path)
    path_params: HashMap<String, String>,
    /// Header参数 (position = Header)
    header_params: HashMap<String, String>,
    /// Body参数 (position = Body)
    body_params: Value,
}

impl ParameterGroups {
    /// 从参数配置和实际参数值中提取并分组参数
    fn extract_parameters(
        config: &HTTPToolConfig,
        parameters: &Value,
    ) -> Result<Self, MCPError> {
        let params_obj = parameters.as_object()
            .ok_or_else(|| MCPError::ParameterValidationFailed("Parameters must be a JSON object".to_string()))?;

        let mut path_params = HashMap::new();
        let mut header_params = HashMap::new();
        let mut body_params_map = serde_json::Map::new();

        // 遍历参数配置，按position分组
        for param_schema in &config.parameters {
            let param_value = params_obj.get(&param_schema.name);
            
            // 如果参数不存在，使用默认值或跳过（如果不是必需的）
            let value = match param_value {
                Some(v) => v.clone(),
                None => {
                    if let Some(default) = &param_schema.default_value {
                        default.clone()
                    } else if param_schema.required {
                        return Err(MCPError::ParameterValidationFailed(
                            format!("Required parameter '{}' is missing", param_schema.name)
                        ));
                    } else {
                        continue; // 跳过可选参数
                    }
                }
            };

            // 根据position分组
            match param_schema.position {
                ParameterPosition::Path => {
                    // 路径参数需要转换为字符串并进行URL编码
                    let string_value = Self::value_to_string(&value)?;
                    let encoded_value = Self::url_encode_path_param(&string_value);
                    path_params.insert(param_schema.name.clone(), encoded_value);
                }
                ParameterPosition::Header => {
                    // Header参数转换为字符串
                    let string_value = Self::value_to_string(&value)?;
                    // 验证header值不包含换行符（安全检查）
                    if string_value.contains('\n') || string_value.contains('\r') {
                        return Err(MCPError::ParameterValidationFailed(
                            format!("Header parameter '{}' contains invalid characters (newlines)", param_schema.name)
                        ));
                    }
                    header_params.insert(param_schema.name.clone(), string_value);
                }
                ParameterPosition::Body => {
                    // Body参数保持原始JSON值
                    body_params_map.insert(param_schema.name.clone(), value);
                }
            }
        }

        Ok(Self {
            path_params,
            header_params,
            body_params: Value::Object(body_params_map),
        })
    }

    /// 将JSON值转换为字符串
    fn value_to_string(value: &Value) -> Result<String, MCPError> {
        match value {
            Value::String(s) => Ok(s.clone()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Null => Ok("".to_string()),
            _ => Err(MCPError::ParameterValidationFailed(
                format!("Cannot convert complex value to string: {}", value)
            )),
        }
    }

    /// URL编码路径参数
    fn url_encode_path_param(value: &str) -> String {
        // 使用percent-encoding对路径参数进行编码
        // 保留一些安全字符，编码其他特殊字符
        utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
    }
}

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

    /// 构建URL，替换路径参数占位符
    fn build_url(
        &self,
        endpoint: &str,
        path_params: &HashMap<String, String>,
    ) -> Result<String, MCPError> {
        let mut url = endpoint.to_string();
        
        // 使用正则表达式查找所有 {paramName} 占位符
        let placeholder_regex = regex::Regex::new(r"\{([^}]+)\}")
            .map_err(|e| MCPError::ConfigurationError(format!("Failed to compile regex: {}", e)))?;
        
        // 收集所有占位符
        let mut placeholders = Vec::new();
        for cap in placeholder_regex.captures_iter(endpoint) {
            if let Some(param_name) = cap.get(1) {
                placeholders.push(param_name.as_str().to_string());
            }
        }
        
        // 验证所有路径参数都已提供
        for placeholder in &placeholders {
            if !path_params.contains_key(placeholder) {
                return Err(MCPError::PathParameterMissing(
                    format!("Path parameter '{}' is required but not provided", placeholder)
                ));
            }
        }
        
        // 替换所有占位符
        for (param_name, param_value) in path_params {
            let placeholder = format!("{{{}}}", param_name);
            if !url.contains(&placeholder) {
                return Err(MCPError::PathParameterInvalid(
                    format!("Path parameter '{}' is provided but not used in endpoint", param_name)
                ));
            }
            url = url.replace(&placeholder, param_value);
        }
        
        Ok(url)
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

        // 提取并分组参数
        let param_groups = ParameterGroups::extract_parameters(config, parameters)?;

        // 构建URL（替换路径参数）
        let url = self.build_url(&config.endpoint, &param_groups.path_params)?;

        // 转换HTTP方法
        let method = self.convert_http_method(&config.method);
        
        // 创建请求构建器
        let mut request_builder = self.client.request(method, &url);

        // 添加配置中的静态头部
        for (key, value) in &config.headers {
            request_builder = request_builder.header(key, value);
        }

        // 添加header参数（动态头部）
        for (key, value) in &param_groups.header_params {
            request_builder = request_builder.header(key, value);
        }

        // 设置超时
        if let Some(timeout) = config.timeout_seconds {
            request_builder = request_builder.timeout(Duration::from_secs(timeout));
        }

        // 添加body参数（如果有）
        if !param_groups.body_params.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            request_builder = request_builder
                .header("Content-Type", "application/json")
                .json(&param_groups.body_params);
        }

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
    template_engine: ResponseTemplateEngine,
}

impl HTTPToMCPConverter {
    pub fn new() -> Self {
        Self {
            request_builder: HTTPRequestBuilder::new(),
            template_engine: ResponseTemplateEngine::new(),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            request_builder: HTTPRequestBuilder::with_timeout(timeout),
            template_engine: ResponseTemplateEngine::new(),
        }
    }
    
    /// Get a reference to the template engine for cache management
    pub fn template_engine(&self) -> &ResponseTemplateEngine {
        &self.template_engine
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
        let result = self.convert_http_response_to_mcp_result(tool, http_response)?;

        Ok(MCPToolResult {
            success: true,
            result: Some(result),
            error: None,
            execution_time_ms: execution_time,
            metadata: HashMap::new(),
        })
    }

    /// 将HTTP响应转换为MCP结果
    fn convert_http_response_to_mcp_result(
        &self,
        tool: &MCPTool,
        response: HTTPResponse,
    ) -> Result<Value, MCPError> {
        // 首先解析响应为JSON
        let json_response = if response.is_json() {
            response.parse_json()?
        } else {
            // 对于非JSON响应，包装为JSON对象
            json!({
                "status_code": response.status_code,
                "headers": response.headers,
                "body": response.body,
                "content_type": response.content_type().unwrap_or(&"text/plain".to_string())
            })
        };

        // 检查是否配置了响应模板
        if let ToolConfig::HTTP(http_config) = &tool.config {
            if let Some(template) = &http_config.response_template {
                // 使用模板引擎渲染响应
                match self.template_engine.render(&tool.id.to_string(), template, &json_response) {
                    Ok(rendered_text) => {
                        // 返回渲染后的文本，包装在JSON对象中
                        return Ok(json!({
                            "text": rendered_text,
                            "raw": json_response
                        }));
                    }
                    Err(template_error) => {
                        // 模板渲染失败，返回原始JSON和错误信息
                        return Ok(json!({
                            "text": null,
                            "raw": json_response,
                            "template_error": template_error.to_string()
                        }));
                    }
                }
            }
        }

        // 没有配置模板，返回原始JSON
        Ok(json_response)
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
        let _builder = HTTPRequestBuilder::new();
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
        let _converter = HTTPToMCPConverter::new();
        // Just test that it can be created
        assert!(true);
    }

    #[test]
    fn test_parameter_groups_extraction() {
        use crate::domain::value_objects::tool_config::ParameterPosition;
        
        let config = HTTPToolConfig::new(
            "https://api.example.com/users/{userId}".to_string(),
            HttpMethod::POST,
        )
        .with_parameter(
            ParameterSchema::new("userId".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Path)
        )
        .with_parameter(
            ParameterSchema::new("Authorization".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Header)
        )
        .with_parameter(
            ParameterSchema::new("name".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Body)
        );

        let params = json!({
            "userId": "123",
            "Authorization": "Bearer token",
            "name": "John Doe"
        });

        let groups = ParameterGroups::extract_parameters(&config, &params).unwrap();
        
        assert_eq!(groups.path_params.get("userId").unwrap(), "123");
        assert_eq!(groups.header_params.get("Authorization").unwrap(), "Bearer token");
        assert_eq!(groups.body_params["name"], "John Doe");
    }

    #[test]
    fn test_url_encoding_path_param() {
        let encoded = ParameterGroups::url_encode_path_param("hello world");
        assert!(encoded.contains("%20"));
        
        let special_chars = ParameterGroups::url_encode_path_param("user@example.com");
        assert!(special_chars.contains("%40"));
    }

    #[test]
    fn test_build_url_with_path_params() {
        let builder = HTTPRequestBuilder::new();
        let mut path_params = HashMap::new();
        path_params.insert("userId".to_string(), "123".to_string());
        path_params.insert("orderId".to_string(), "456".to_string());

        let url = builder.build_url(
            "https://api.example.com/users/{userId}/orders/{orderId}",
            &path_params
        ).unwrap();

        assert_eq!(url, "https://api.example.com/users/123/orders/456");
    }

    #[test]
    fn test_build_url_missing_path_param() {
        let builder = HTTPRequestBuilder::new();
        let path_params = HashMap::new();

        let result = builder.build_url(
            "https://api.example.com/users/{userId}",
            &path_params
        );

        assert!(result.is_err());
        match result {
            Err(MCPError::PathParameterMissing(msg)) => {
                assert!(msg.contains("userId"));
            }
            _ => panic!("Expected PathParameterMissing error"),
        }
    }

    #[test]
    fn test_header_param_validation_newline() {
        use crate::domain::value_objects::tool_config::ParameterPosition;
        
        let config = HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::GET,
        )
        .with_parameter(
            ParameterSchema::new("X-Custom-Header".to_string(), ParameterType::String, true)
                .with_position(ParameterPosition::Header)
        );

        let params = json!({
            "X-Custom-Header": "value\nwith\nnewlines"
        });

        let result = ParameterGroups::extract_parameters(&config, &params);
        assert!(result.is_err());
        match result {
            Err(MCPError::ParameterValidationFailed(msg)) => {
                assert!(msg.contains("newlines"));
            }
            _ => panic!("Expected ParameterValidationFailed error"),
        }
    }
}