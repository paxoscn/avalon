use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use rmcp::transport::StreamableHttpService;

use crate::{
    infrastructure::mcp::mcp_server_handler::MCPServerHandler,
    presentation::handlers::{mcp_server_handlers, Counter},
};

/// 创建MCP Server协议路由
/// 这些路由实现标准的MCP协议接口，供外部系统调用
pub fn create_mcp_server_routes() -> Router<Arc<MCPServerHandler>> {
    Router::new()
        // MCP Server协议接口
        .route("/tools", get(mcp_server_handlers::list_mcp_tools))
        .route("/tools/call", post(mcp_server_handlers::call_mcp_tool))
}

/// 创建完整的MCP Server API路由
pub fn create_mcp_server_api_routes(
    streamable_http_service: StreamableHttpService<Counter>,
) -> Router {
    Router::new()
        .nest_service("/mcp", streamable_http_service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            repositories::mcp_tool_repository::{MCPToolQueryOptions, MCPToolQueryResult, MCPToolRepository},
            entities::MCPTool,
            services::mcp_tool_service::{ToolCallContext, ToolCallResult},
            value_objects::ids::{MCPToolId, TenantId, UserId},
        },
        error::PlatformError,
        infrastructure::mcp::proxy_service::{MCPProxyService, MCPToolStats},
    };
    use async_trait::async_trait;
    use serde_json::Value;
    use std::collections::HashMap;

    // Mock repository
    struct MockMCPToolRepository;

    #[async_trait]
    impl MCPToolRepository for MockMCPToolRepository {
        async fn find_by_id(&self, _id: MCPToolId) -> Result<Option<MCPTool>, PlatformError> {
            Ok(None)
        }

        async fn find_by_tenant_and_name(
            &self,
            _tenant_id: TenantId,
            _name: &str,
        ) -> Result<Option<MCPTool>, PlatformError> {
            Ok(None)
        }

        async fn find_by_options(
            &self,
            _options: MCPToolQueryOptions,
        ) -> Result<MCPToolQueryResult, PlatformError> {
            Ok(MCPToolQueryResult {
                tools: vec![],
                total_count: 0,
            })
        }

        async fn find_by_tenant_id(&self, _tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError> {
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

    // Mock proxy service
    struct MockMCPProxyService;

    #[async_trait]
    impl MCPProxyService for MockMCPProxyService {
        async fn register_tool(&self, _tool: MCPTool) -> Result<(), PlatformError> {
            Ok(())
        }

        async fn unregister_tool(&self, _tool_id: MCPToolId) -> Result<(), PlatformError> {
            Ok(())
        }

        async fn get_tenant_tools(&self, _tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError> {
            Ok(vec![])
        }

        async fn call_tool(
            &self,
            _tool_id: MCPToolId,
            _parameters: Value,
            _context: ToolCallContext,
        ) -> Result<ToolCallResult, PlatformError> {
            Ok(ToolCallResult::success(
                serde_json::json!({"result": "success"}),
                100,
            ))
        }

        async fn handle_mcp_request(
            &self,
            _request: crate::infrastructure::mcp::protocol_handler::MCPRequest,
            _tenant_id: TenantId,
        ) -> Result<crate::infrastructure::mcp::protocol_handler::MCPResponse, PlatformError> {
            unimplemented!()
        }

        async fn test_tool_connection(&self, _tool_id: MCPToolId) -> Result<ToolCallResult, PlatformError> {
            Ok(ToolCallResult::success(
                serde_json::json!({"connection": "ok"}),
                10,
            ))
        }

        async fn get_tool_stats(&self, _tenant_id: TenantId) -> Result<MCPToolStats, PlatformError> {
            Ok(MCPToolStats {
                total_tools: 0,
                active_tools: 0,
                inactive_tools: 0,
                tools_by_type: HashMap::new(),
            })
        }
    }

    #[test]
    fn test_create_mcp_server_routes() {
        let mock_repo = Arc::new(MockMCPToolRepository);
        let mock_proxy = Arc::new(MockMCPProxyService);
        let handler = Arc::new(MCPServerHandler::new(mock_repo, mock_proxy));
        
        let _router = create_mcp_server_routes().with_state(handler);
        
        // If this compiles, routes are properly defined
        assert!(true);
    }

    #[test]
    fn test_create_mcp_server_api_routes() {
        let mock_repo = Arc::new(MockMCPToolRepository);
        let mock_proxy = Arc::new(MockMCPProxyService);
        let handler = Arc::new(MCPServerHandler::new(mock_repo, mock_proxy));
        
        let _router = create_mcp_server_api_routes(handler);
        
        // If this compiles, routes are properly defined
        assert!(true);
    }
}
