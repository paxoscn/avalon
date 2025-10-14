use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{
    entities::MCPTool,
    services::mcp_tool_service::{ToolCallContext, ToolCallResult},
    value_objects::ids::{MCPToolId, TenantId, UserId},
};
use crate::error::PlatformError;
use crate::infrastructure::mcp::{
    error_handling::MCPError,
    http_converter::HTTPToMCPConverter,
    protocol_handler::{MCPProtocolHandler, MCPRequest, MCPResponse},
};

/// MCP代理服务接口
#[async_trait]
pub trait MCPProxyService: Send + Sync {
    /// 注册工具
    async fn register_tool(&self, tool: MCPTool) -> Result<(), PlatformError>;

    /// 注销工具
    async fn unregister_tool(&self, tool_id: MCPToolId) -> Result<(), PlatformError>;

    /// 获取租户的所有工具
    async fn get_tenant_tools(&self, tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError>;

    /// 调用工具
    async fn call_tool(
        &self,
        tool_id: MCPToolId,
        parameters: Value,
        context: ToolCallContext,
    ) -> Result<ToolCallResult, PlatformError>;

    /// 处理MCP协议请求
    async fn handle_mcp_request(
        &self,
        request: MCPRequest,
        tenant_id: TenantId,
    ) -> Result<MCPResponse, PlatformError>;

    /// 测试工具连接
    async fn test_tool_connection(&self, tool_id: MCPToolId) -> Result<ToolCallResult, PlatformError>;

    /// 获取工具统计信息
    async fn get_tool_stats(&self, tenant_id: TenantId) -> Result<MCPToolStats, PlatformError>;
}

/// MCP工具统计信息
#[derive(Debug, Clone)]
pub struct MCPToolStats {
    pub total_tools: usize,
    pub active_tools: usize,
    pub inactive_tools: usize,
    pub tools_by_type: HashMap<String, usize>,
}

/// MCP代理服务实现
pub struct MCPProxyServiceImpl {
    /// 按租户组织的协议处理器
    handlers: Arc<RwLock<HashMap<TenantId, MCPProtocolHandler>>>,
    /// HTTP转换器
    converter: HTTPToMCPConverter,
    /// 工具存储（按租户和工具ID组织）
    tools: Arc<RwLock<HashMap<TenantId, HashMap<MCPToolId, MCPTool>>>>,
}

impl MCPProxyServiceImpl {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            converter: HTTPToMCPConverter::new(),
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取或创建租户的协议处理器
    async fn get_or_create_handler(&self, tenant_id: TenantId) -> MCPProtocolHandler {
        let mut handlers = self.handlers.write().await;
        
        if let Some(handler) = handlers.get(&tenant_id) {
            handler.clone()
        } else {
            let handler = MCPProtocolHandler::new();
            handlers.insert(tenant_id, handler.clone());
            handler
        }
    }

    /// 更新租户的协议处理器
    async fn update_handler(&self, tenant_id: TenantId, handler: MCPProtocolHandler) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(tenant_id, handler);
    }

    /// 根据工具ID查找工具和租户
    async fn find_tool_by_id(&self, tool_id: MCPToolId) -> Option<(TenantId, MCPTool)> {
        let tools = self.tools.read().await;
        
        for (tenant_id, tenant_tools) in tools.iter() {
            if let Some(tool) = tenant_tools.get(&tool_id) {
                return Some((*tenant_id, tool.clone()));
            }
        }
        
        None
    }

    /// 验证工具访问权限
    fn validate_tool_access(&self, tool: &MCPTool, context: &ToolCallContext) -> Result<(), PlatformError> {
        if !tool.can_access(&context.tenant_id) {
            return Err(PlatformError::AuthorizationFailed(
                "Tool does not belong to user's tenant".to_string()
            ));
        }

        if !tool.can_execute() {
            return Err(PlatformError::ValidationError(
                "Tool is not in active state".to_string()
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl MCPProxyService for MCPProxyServiceImpl {
    async fn register_tool(&self, tool: MCPTool) -> Result<(), PlatformError> {
        let tenant_id = tool.tenant_id;
        let tool_id = tool.id;

        // 存储工具
        {
            let mut tools = self.tools.write().await;
            let tenant_tools = tools.entry(tenant_id).or_insert_with(HashMap::new);
            tenant_tools.insert(tool_id, tool.clone());
        }

        // 更新协议处理器
        let mut handler = self.get_or_create_handler(tenant_id).await;
        handler.register_tool(tool);
        self.update_handler(tenant_id, handler).await;

        Ok(())
    }

    async fn unregister_tool(&self, tool_id: MCPToolId) -> Result<(), PlatformError> {
        // 查找工具
        let (tenant_id, tool) = self.find_tool_by_id(tool_id).await
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 从存储中移除
        {
            let mut tools = self.tools.write().await;
            if let Some(tenant_tools) = tools.get_mut(&tenant_id) {
                tenant_tools.remove(&tool_id);
                
                // 如果租户没有工具了，移除整个租户条目
                if tenant_tools.is_empty() {
                    tools.remove(&tenant_id);
                }
            }
        }

        // 更新协议处理器
        let mut handler = self.get_or_create_handler(tenant_id).await;
        handler.unregister_tool(&tool.name);
        self.update_handler(tenant_id, handler).await;

        Ok(())
    }

    async fn get_tenant_tools(&self, tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError> {
        let tools = self.tools.read().await;
        
        let tenant_tools = tools.get(&tenant_id)
            .map(|tools| tools.values().cloned().collect())
            .unwrap_or_default();

        Ok(tenant_tools)
    }

    async fn call_tool(
        &self,
        tool_id: MCPToolId,
        parameters: Value,
        context: ToolCallContext,
    ) -> Result<ToolCallResult, PlatformError> {
        // 查找工具
        let (tenant_id, tool) = self.find_tool_by_id(tool_id).await
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 验证访问权限
        self.validate_tool_access(&tool, &context)?;

        // 执行工具调用
        let start_time = std::time::Instant::now();
        
        match self.converter.execute_tool(&tool, &parameters).await {
            Ok(mcp_result) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                Ok(ToolCallResult::success(
                    mcp_result.result.unwrap_or(serde_json::json!(null)),
                    execution_time,
                ))
            }
            Err(mcp_error) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                Ok(ToolCallResult::error(
                    mcp_error.to_string(),
                    execution_time,
                ))
            }
        }
    }

    async fn handle_mcp_request(
        &self,
        request: MCPRequest,
        tenant_id: TenantId,
    ) -> Result<MCPResponse, PlatformError> {
        let handler = self.get_or_create_handler(tenant_id).await;
        let response = handler.handle_request(request).await;
        Ok(response)
    }

    async fn test_tool_connection(&self, tool_id: MCPToolId) -> Result<ToolCallResult, PlatformError> {
        // 查找工具
        let (_tenant_id, tool) = self.find_tool_by_id(tool_id).await
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 测试连接
        let start_time = std::time::Instant::now();
        
        match self.converter.test_tool_connection(&tool).await {
            Ok(mcp_result) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                Ok(ToolCallResult::success(
                    mcp_result.result.unwrap_or(serde_json::json!({"connection_test": "passed"})),
                    execution_time,
                ))
            }
            Err(mcp_error) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                Ok(ToolCallResult::error(
                    format!("Connection test failed: {}", mcp_error),
                    execution_time,
                ))
            }
        }
    }

    async fn get_tool_stats(&self, tenant_id: TenantId) -> Result<MCPToolStats, PlatformError> {
        let tools = self.tools.read().await;
        
        let empty_map = HashMap::new();
        let tenant_tools = tools.get(&tenant_id).unwrap_or(&empty_map);
        
        let total_tools = tenant_tools.len();
        let active_tools = tenant_tools.values().filter(|tool| tool.can_execute()).count();
        let inactive_tools = total_tools - active_tools;
        
        let mut tools_by_type = HashMap::new();
        for tool in tenant_tools.values() {
            let tool_type = tool.config.tool_type();
            *tools_by_type.entry(tool_type.to_string()).or_insert(0) += 1;
        }

        Ok(MCPToolStats {
            total_tools,
            active_tools,
            inactive_tools,
            tools_by_type,
        })
    }
}

impl Default for MCPProxyServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP代理服务构建器
pub struct MCPProxyServiceBuilder {
    converter_timeout: Option<std::time::Duration>,
}

impl MCPProxyServiceBuilder {
    pub fn new() -> Self {
        Self {
            converter_timeout: None,
        }
    }

    pub fn with_converter_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.converter_timeout = Some(timeout);
        self
    }

    pub fn build(self) -> MCPProxyServiceImpl {
        let converter = if let Some(timeout) = self.converter_timeout {
            HTTPToMCPConverter::with_timeout(timeout)
        } else {
            HTTPToMCPConverter::new()
        };

        MCPProxyServiceImpl {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            converter,
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MCPProxyServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{
        ids::{TenantId, UserId},
        tool_config::{HTTPToolConfig, HttpMethod, ToolConfig},
    };

    fn create_test_tool(tenant_id: TenantId) -> MCPTool {
        let config = HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::GET,
        );

        let mut tool = MCPTool::new(
            tenant_id,
            "test-tool".to_string(),
            Some("Test tool".to_string()),
            ToolConfig::HTTP(config),
            UserId::new(),
        );
        tool.activate();
        tool
    }

    fn create_test_context(tenant_id: TenantId, user_id: UserId) -> ToolCallContext {
        ToolCallContext::new(tenant_id, user_id, "test-request".to_string())
    }

    #[tokio::test]
    async fn test_proxy_service_creation() {
        let service = MCPProxyServiceImpl::new();
        // Just test that it can be created
        assert!(true);
    }

    #[tokio::test]
    async fn test_tool_registration() {
        let service = MCPProxyServiceImpl::new();
        let tenant_id = TenantId::new();
        let tool = create_test_tool(tenant_id);
        
        let result = service.register_tool(tool.clone()).await;
        assert!(result.is_ok());
        
        let tenant_tools = service.get_tenant_tools(tenant_id).await.unwrap();
        assert_eq!(tenant_tools.len(), 1);
        assert_eq!(tenant_tools[0].name, "test-tool");
    }

    #[tokio::test]
    async fn test_tool_unregistration() {
        let service = MCPProxyServiceImpl::new();
        let tenant_id = TenantId::new();
        let tool = create_test_tool(tenant_id);
        let tool_id = tool.id;
        
        service.register_tool(tool).await.unwrap();
        
        let result = service.unregister_tool(tool_id).await;
        assert!(result.is_ok());
        
        let tenant_tools = service.get_tenant_tools(tenant_id).await.unwrap();
        assert_eq!(tenant_tools.len(), 0);
    }

    #[tokio::test]
    async fn test_get_tool_stats() {
        let service = MCPProxyServiceImpl::new();
        let tenant_id = TenantId::new();
        let tool = create_test_tool(tenant_id);
        
        service.register_tool(tool).await.unwrap();
        
        let stats = service.get_tool_stats(tenant_id).await.unwrap();
        assert_eq!(stats.total_tools, 1);
        assert_eq!(stats.active_tools, 1);
        assert_eq!(stats.inactive_tools, 0);
        assert_eq!(stats.tools_by_type.get("http"), Some(&1));
    }

    #[tokio::test]
    async fn test_proxy_service_builder() {
        let service = MCPProxyServiceBuilder::new()
            .with_converter_timeout(std::time::Duration::from_secs(60))
            .build();
        
        // Just test that it can be built
        assert!(true);
    }
}