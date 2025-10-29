use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::{
    error::PlatformError,
    infrastructure::mcp::{
        mcp_protocol::{MCPToolCallResponse, MCPToolListResponse},
        mcp_server_handler::MCPServerHandler,
    },
    presentation::extractors::AuthenticatedUser,
};

/// MCP Server工具列表查询参数
#[derive(Debug, Deserialize)]
pub struct MCPServerListQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

/// MCP Server工具调用请求
#[derive(Debug, Deserialize, Serialize)]
pub struct MCPServerCallRequest {
    pub name: String,
    pub arguments: Value,
}

/// 获取MCP工具列表 (MCP Server接口)
/// GET /api/v1/mcp/tools
pub async fn list_mcp_tools(
    State(handler): State<Arc<MCPServerHandler>>,
    user: AuthenticatedUser,
    Query(query): Query<MCPServerListQuery>,
) -> Result<Json<MCPToolListResponse>, PlatformError> {
    let response = handler
        .handle_list_tools(user.tenant_id, query.page, query.limit)
        .await?;

    Ok(Json(response))
}

/// 调用MCP工具 (MCP Server接口)
/// POST /api/v1/mcp/tools/call
pub async fn call_mcp_tool(
    State(handler): State<Arc<MCPServerHandler>>,
    user: AuthenticatedUser,
    Json(request): Json<MCPServerCallRequest>,
) -> Result<Json<MCPToolCallResponse>, PlatformError> {
    let response = handler
        .handle_call_tool(user.tenant_id, user.user_id, request.name, request.arguments)
        .await?;

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::MCPTool,
        repositories::mcp_tool_repository::{MCPToolQueryOptions, MCPToolQueryResult, MCPToolRepository},
        services::mcp_tool_service::{ToolCallContext, ToolCallResult},
        value_objects::{
            ids::{MCPToolId, TenantId, UserId},
            tool_config::{HTTPToolConfig, HttpMethod, ParameterSchema, ParameterType, ToolConfig, ParameterPosition},
        },
    };
    use crate::infrastructure::mcp::proxy_service::{MCPProxyService, MCPToolStats};
    use async_trait::async_trait;
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
            tenant_id: TenantId,
            name: &str,
        ) -> Result<Option<MCPTool>, PlatformError> {
            if name == "test-tool" {
                let config = HTTPToolConfig {
                    endpoint: "https://api.example.com/test".to_string(),
                    method: HttpMethod::GET,
                    headers: HashMap::new(),
                    parameters: vec![],
                    timeout_seconds: Some(30),
                    retry_count: Some(3),
                    response_template: None,
                };

                let mut tool = MCPTool::new(
                    tenant_id,
                    "test-tool".to_string(),
                    Some("Test tool".to_string()),
                    ToolConfig::HTTP(config),
                    UserId::new(),
                );
                tool.activate();

                Ok(Some(tool))
            } else {
                Ok(None)
            }
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

    #[tokio::test]
    async fn test_list_mcp_tools_handler() {
        let mock_repo = Arc::new(MockMCPToolRepository);
        let mock_proxy = Arc::new(MockMCPProxyService);
        let handler = Arc::new(MCPServerHandler::new(mock_repo, mock_proxy));

        let user = AuthenticatedUser {
            user_id: UserId::new(),
            tenant_id: TenantId::new(),
        };

        let query = MCPServerListQuery {
            page: Some(1),
            limit: Some(10),
        };

        let result = list_mcp_tools(State(handler), user, Query(query)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_call_mcp_tool_handler() {
        let mock_repo = Arc::new(MockMCPToolRepository);
        let mock_proxy = Arc::new(MockMCPProxyService);
        let handler = Arc::new(MCPServerHandler::new(mock_repo, mock_proxy));

        let user = AuthenticatedUser {
            user_id: UserId::new(),
            tenant_id: TenantId::new(),
        };

        let request = MCPServerCallRequest {
            name: "test-tool".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = call_mcp_tool(State(handler), user, Json(request)).await;
        assert!(result.is_ok());
    }
}
