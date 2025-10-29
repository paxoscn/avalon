use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::domain::{
    entities::MCPTool,
    repositories::mcp_tool_repository::{MCPToolRepository, MCPToolQueryOptions},
    value_objects::ids::{TenantId, UserId},
};
use crate::error::PlatformError;
use crate::infrastructure::mcp::mcp_protocol::{
    tool_to_mcp_format, MCPContent, MCPToolCallResponse, MCPToolDescriptor, MCPToolListResponse,
};
use crate::infrastructure::mcp::proxy_service::MCPProxyService;

/// MCP Server Handler - 提供标准MCP协议接口
pub struct MCPServerHandler {
    tool_repository: Arc<dyn MCPToolRepository>,
    proxy_service: Arc<dyn MCPProxyService>,
}

impl MCPServerHandler {
    /// 创建新的MCP Server Handler
    pub fn new(
        tool_repository: Arc<dyn MCPToolRepository>,
        proxy_service: Arc<dyn MCPProxyService>,
    ) -> Self {
        Self {
            tool_repository,
            proxy_service,
        }
    }

    /// 处理tools/list请求 - 返回租户的工具列表
    pub async fn handle_list_tools(
        &self,
        tenant_id: TenantId,
        page: Option<u64>,
        limit: Option<u64>,
    ) -> Result<MCPToolListResponse, PlatformError> {
        // 设置分页参数
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(50).min(100); // 最大100条
        let offset = (page - 1) * limit;

        // 构建查询选项
        let options = MCPToolQueryOptions::new()
            .with_tenant_id(tenant_id)
            .with_pagination(limit, offset);

        // 查询工具列表
        let query_result = self.tool_repository.find_by_options(options).await?;

        // 转换为MCP格式
        let tools: Vec<MCPToolDescriptor> = query_result
            .tools
            .iter()
            .map(|tool| tool_to_mcp_format(tool))
            .collect();

        Ok(MCPToolListResponse { tools })
    }

    /// 处理tools/call请求 - 执行工具调用
    pub async fn handle_call_tool(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        tool_name: String,
        arguments: Value,
    ) -> Result<MCPToolCallResponse, PlatformError> {
        // 根据租户ID和工具名称查找工具
        let tool = self
            .tool_repository
            .find_by_tenant_and_name(tenant_id, &tool_name)
            .await?
            .ok_or_else(|| {
                PlatformError::NotFound(format!("Tool '{}' not found", tool_name))
            })?;

        // 验证工具是否可以执行
        if !tool.can_execute() {
            return Ok(MCPToolCallResponse::error(format!(
                "Tool '{}' is not in active state",
                tool_name
            )));
        }

        // 验证参数
        if let Err(validation_error) = tool.config.validate_call_parameters(&arguments) {
            return Ok(MCPToolCallResponse::error(format!(
                "Parameter validation failed: {}",
                validation_error
            )));
        }

        // 创建工具调用上下文
        let context = crate::domain::services::mcp_tool_service::ToolCallContext::new(
            tenant_id,
            user_id,
            format!("mcp-server-call-{}", tool_name),
        );

        // 执行工具调用
        match self
            .proxy_service
            .call_tool(tool.id, arguments, context)
            .await
        {
            Ok(result) => {
                if result.success {
                    // 成功响应
                    let content_text = if let Some(ref res) = result.result {
                        serde_json::to_string_pretty(res)
                            .unwrap_or_else(|_| res.to_string())
                    } else {
                        "null".to_string()
                    };
                    Ok(MCPToolCallResponse::success(content_text))
                } else {
                    // 工具执行失败
                    let error_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
                    Ok(MCPToolCallResponse::error(error_msg))
                }
            }
            Err(e) => {
                // 系统错误
                Ok(MCPToolCallResponse::error(format!(
                    "Tool execution failed: {}",
                    e
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::mcp_tool::MCPToolStatus,
        repositories::mcp_tool_repository::MCPToolQueryResult,
        value_objects::{
            ids::MCPToolId,
            tool_config::{HTTPToolConfig, HttpMethod, ParameterSchema, ParameterType, ToolConfig},
        },
    };
    use crate::domain::services::mcp_tool_service::ToolCallResult;
    use std::collections::HashMap;

    // Mock repository for testing
    struct MockMCPToolRepository {
        tools: HashMap<String, MCPTool>,
    }

    impl MockMCPToolRepository {
        fn new() -> Self {
            Self {
                tools: HashMap::new(),
            }
        }

        fn add_tool(&mut self, tool: MCPTool) {
            let key = format!("{}:{}", tool.tenant_id, tool.name);
            self.tools.insert(key, tool);
        }
    }

    #[async_trait]
    impl MCPToolRepository for MockMCPToolRepository {
        async fn find_by_id(&self, _id: MCPToolId) -> Result<Option<MCPTool>, PlatformError> {
            Ok(None)
        }

        async fn find_by_tenant_and_name(
            &self,
            tenant_id: TenantId,
            name: &str,
        ) -> Result<Option<MCPTool>, PlatformError> {
            let key = format!("{}:{}", tenant_id, name);
            Ok(self.tools.get(&key).cloned())
        }

        async fn find_by_options(
            &self,
            options: MCPToolQueryOptions,
        ) -> Result<MCPToolQueryResult, PlatformError> {
            let tenant_id = options.tenant_id.unwrap();
            let tools: Vec<MCPTool> = self
                .tools
                .values()
                .filter(|t| t.tenant_id == tenant_id)
                .cloned()
                .collect();

            Ok(MCPToolQueryResult {
                total_count: tools.len() as u64,
                tools,
            })
        }

        async fn find_by_tenant_id(
            &self,
            _tenant_id: TenantId,
        ) -> Result<Vec<MCPTool>, PlatformError> {
            Ok(vec![])
        }

        async fn find_by_created_by(&self, _created_by: UserId) -> Result<Vec<MCPTool>, PlatformError> {
            Ok(vec![])
        }

        async fn save(&self, _tool: &MCPTool) -> Result<(), PlatformError> {
            Ok(())
        }

        async fn update(&self, _tool: &MCPTool) -> Result<(), PlatformError> {
            Ok(())
        }

        async fn delete(&self, _id: MCPToolId) -> Result<(), PlatformError> {
            Ok(())
        }

        async fn exists_by_tenant_and_name(
            &self,
            _tenant_id: TenantId,
            _name: &str,
            _exclude_id: Option<MCPToolId>,
        ) -> Result<bool, PlatformError> {
            Ok(false)
        }

        async fn count_by_tenant(&self, _tenant_id: TenantId) -> Result<u64, PlatformError> {
            Ok(0)
        }

        async fn find_active_by_tenant(&self, _tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError> {
            Ok(vec![])
        }

        async fn get_version_history(&self, _tool_id: MCPToolId) -> Result<Vec<crate::domain::entities::MCPToolVersion>, PlatformError> {
            Ok(vec![])
        }

        async fn rollback_to_version(
            &self,
            _tool_id: MCPToolId,
            _target_version: i32,
            _created_by: UserId,
            _change_log: Option<String>,
        ) -> Result<MCPTool, PlatformError> {
            Err(PlatformError::NotFound("Not implemented".to_string()))
        }

        async fn compare_versions(
            &self,
            _tool_id: MCPToolId,
            _from_version: i32,
            _to_version: i32,
        ) -> Result<crate::domain::entities::VersionDiff, PlatformError> {
            Err(PlatformError::NotFound("Not implemented".to_string()))
        }

        async fn create_version(
            &self,
            _tool: &MCPTool,
            _change_log: Option<String>,
        ) -> Result<crate::domain::entities::MCPToolVersion, PlatformError> {
            Err(PlatformError::NotFound("Not implemented".to_string()))
        }
    }

    // Mock proxy service for testing
    struct MockMCPProxyService {
        should_succeed: bool,
    }

    impl MockMCPProxyService {
        fn new(should_succeed: bool) -> Self {
            Self { should_succeed }
        }
    }

    #[async_trait]
    impl MCPProxyService for MockMCPProxyService {
        async fn register_tool(&self, _tool: MCPTool) -> Result<(), PlatformError> {
            Ok(())
        }

        async fn unregister_tool(&self, _tool_id: MCPToolId) -> Result<(), PlatformError> {
            Ok(())
        }

        async fn get_tenant_tools(
            &self,
            _tenant_id: TenantId,
        ) -> Result<Vec<MCPTool>, PlatformError> {
            Ok(vec![])
        }

        async fn call_tool(
            &self,
            _tool_id: MCPToolId,
            _parameters: Value,
            _context: crate::domain::services::mcp_tool_service::ToolCallContext,
        ) -> Result<ToolCallResult, PlatformError> {
            if self.should_succeed {
                Ok(ToolCallResult::success(
                    serde_json::json!({"result": "success"}),
                    100,
                ))
            } else {
                Ok(ToolCallResult::error("Tool execution failed".to_string(), 50))
            }
        }

        async fn handle_mcp_request(
            &self,
            _request: crate::infrastructure::mcp::protocol_handler::MCPRequest,
            _tenant_id: TenantId,
        ) -> Result<crate::infrastructure::mcp::protocol_handler::MCPResponse, PlatformError> {
            unimplemented!()
        }

        async fn test_tool_connection(
            &self,
            _tool_id: MCPToolId,
        ) -> Result<ToolCallResult, PlatformError> {
            Ok(ToolCallResult::success(
                serde_json::json!({"connection": "ok"}),
                10,
            ))
        }

        async fn get_tool_stats(
            &self,
            _tenant_id: TenantId,
        ) -> Result<crate::infrastructure::mcp::proxy_service::MCPToolStats, PlatformError> {
            Ok(crate::infrastructure::mcp::proxy_service::MCPToolStats {
                total_tools: 0,
                active_tools: 0,
                inactive_tools: 0,
                tools_by_type: HashMap::new(),
            })
        }
    }

    fn create_test_tool(tenant_id: TenantId, name: &str, active: bool) -> MCPTool {
        let config = HTTPToolConfig {
            endpoint: "https://api.example.com/test".to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            parameters: vec![ParameterSchema {
                name: "query".to_string(),
                parameter_type: ParameterType::String,
                description: Some("Search query".to_string()),
                required: false,
                default_value: None,
                enum_values: None,
                position: crate::domain::value_objects::tool_config::ParameterPosition::Body,
            }],
            timeout_seconds: Some(30),
            retry_count: Some(3),
            response_template: None,
        };

        let mut tool = MCPTool::new(
            tenant_id,
            name.to_string(),
            Some("Test tool".to_string()),
            ToolConfig::HTTP(config),
            UserId::new(),
        );

        if active {
            tool.activate();
        }

        tool
    }

    #[tokio::test]
    async fn test_handle_list_tools() {
        let tenant_id = TenantId::new();
        let mut mock_repo = MockMCPToolRepository::new();

        // Add test tools
        mock_repo.add_tool(create_test_tool(tenant_id, "tool1", true));
        mock_repo.add_tool(create_test_tool(tenant_id, "tool2", true));

        let mock_proxy = MockMCPProxyService::new(true);

        let handler = MCPServerHandler::new(Arc::new(mock_repo), Arc::new(mock_proxy));

        let response = handler.handle_list_tools(tenant_id, None, None).await.unwrap();

        assert_eq!(response.tools.len(), 2);
        assert!(response.tools.iter().any(|t| t.name == "tool1"));
        assert!(response.tools.iter().any(|t| t.name == "tool2"));
    }

    #[tokio::test]
    async fn test_handle_list_tools_with_pagination() {
        let tenant_id = TenantId::new();
        let mut mock_repo = MockMCPToolRepository::new();

        // Add test tools
        for i in 1..=5 {
            mock_repo.add_tool(create_test_tool(tenant_id, &format!("tool{}", i), true));
        }

        let mock_proxy = MockMCPProxyService::new(true);

        let handler = MCPServerHandler::new(Arc::new(mock_repo), Arc::new(mock_proxy));

        // Request page 1 with limit 2
        let response = handler
            .handle_list_tools(tenant_id, Some(1), Some(2))
            .await
            .unwrap();

        // Should return all tools (mock doesn't implement pagination)
        assert!(response.tools.len() > 0);
    }

    #[tokio::test]
    async fn test_handle_call_tool_success() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let mut mock_repo = MockMCPToolRepository::new();

        let tool = create_test_tool(tenant_id, "test-tool", true);
        mock_repo.add_tool(tool);

        let mock_proxy = MockMCPProxyService::new(true);

        let handler = MCPServerHandler::new(Arc::new(mock_repo), Arc::new(mock_proxy));

        let arguments = serde_json::json!({
            "query": "test"
        });

        let response = handler
            .handle_call_tool(tenant_id, user_id, "test-tool".to_string(), arguments)
            .await
            .unwrap();

        assert_eq!(response.is_error, None);
        assert_eq!(response.content.len(), 1);
    }

    #[tokio::test]
    async fn test_handle_call_tool_not_found() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let mock_repo = MockMCPToolRepository::new();
        let mock_proxy = MockMCPProxyService::new(true);

        let handler = MCPServerHandler::new(Arc::new(mock_repo), Arc::new(mock_proxy));

        let arguments = serde_json::json!({});

        let result = handler
            .handle_call_tool(tenant_id, user_id, "nonexistent-tool".to_string(), arguments)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_call_tool_inactive() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let mut mock_repo = MockMCPToolRepository::new();

        let tool = create_test_tool(tenant_id, "inactive-tool", false);
        mock_repo.add_tool(tool);

        let mock_proxy = MockMCPProxyService::new(true);

        let handler = MCPServerHandler::new(Arc::new(mock_repo), Arc::new(mock_proxy));

        let arguments = serde_json::json!({});

        let response = handler
            .handle_call_tool(tenant_id, user_id, "inactive-tool".to_string(), arguments)
            .await
            .unwrap();

        assert_eq!(response.is_error, Some(true));
        assert!(response.content[0].text.as_ref().unwrap().contains("not in active state"));
    }

    #[tokio::test]
    async fn test_handle_call_tool_execution_failure() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let mut mock_repo = MockMCPToolRepository::new();

        let tool = create_test_tool(tenant_id, "failing-tool", true);
        mock_repo.add_tool(tool);

        let mock_proxy = MockMCPProxyService::new(false);

        let handler = MCPServerHandler::new(Arc::new(mock_repo), Arc::new(mock_proxy));

        let arguments = serde_json::json!({});

        let response = handler
            .handle_call_tool(tenant_id, user_id, "failing-tool".to_string(), arguments)
            .await
            .unwrap();

        assert_eq!(response.is_error, Some(true));
    }
}
