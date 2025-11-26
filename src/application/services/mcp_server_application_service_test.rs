#[cfg(test)]
mod integration_tests {
    use super::super::mcp_server_application_service::*;
    use crate::{
        application::dto::{APIKeyAuthContext, PermissionScopeDTO},
        domain::{
            entities::MCPTool,
            repositories::mcp_tool_repository::{MCPToolQueryOptions, MCPToolQueryResult, MCPToolRepository},
            services::mcp_tool_service::{ToolCallContext, ToolCallResult},
            value_objects::{
                ids::{MCPToolId, TenantId, UserId},
                tool_config::{HTTPToolConfig, HttpMethod, ToolConfig},
            },
        },
        error::{PlatformError, Result},
        infrastructure::mcp::{
            proxy_service::{MCPProxyService, MCPToolStats},
            protocol_handler::{MCPRequest, MCPResponse},
        },
    };
    use async_trait::async_trait;
    use serde_json::Value;
    use std::{collections::HashMap, sync::Arc};
    use uuid::Uuid;

    // Mock repository for testing
    struct MockMCPToolRepository {
        tools: Vec<MCPTool>,
    }

    impl MockMCPToolRepository {
        fn new(tools: Vec<MCPTool>) -> Self {
            Self { tools }
        }
    }

    #[async_trait]
    impl MCPToolRepository for MockMCPToolRepository {
        async fn find_by_id(&self, id: MCPToolId) -> Result<Option<MCPTool>> {
            Ok(self.tools.iter().find(|t| t.id == id).cloned())
        }

        async fn find_by_tenant_and_name(
            &self,
            tenant_id: TenantId,
            name: &str,
        ) -> Result<Option<MCPTool>> {
            Ok(self
                .tools
                .iter()
                .find(|t| t.tenant_id == tenant_id && t.name == name)
                .cloned())
        }

        async fn find_by_options(&self, options: MCPToolQueryOptions) -> Result<MCPToolQueryResult> {
            let mut filtered: Vec<MCPTool> = self.tools.clone();

            if let Some(tenant_id) = options.tenant_id {
                filtered.retain(|t| t.tenant_id == tenant_id);
            }

            if let Some(status) = options.status {
                // Filter by status - "active" means can_execute() returns true
                if status.to_lowercase() == "active" {
                    filtered.retain(|t| t.can_execute());
                } else {
                    filtered.retain(|t| !t.can_execute());
                }
            }

            let total_count = filtered.len() as u64;

            Ok(MCPToolQueryResult {
                tools: filtered,
                total_count,
            })
        }

        async fn find_by_tenant_id(&self, tenant_id: TenantId) -> Result<Vec<MCPTool>> {
            Ok(self
                .tools
                .iter()
                .filter(|t| t.tenant_id == tenant_id)
                .cloned()
                .collect())
        }

        async fn find_by_created_by(&self, _created_by: UserId) -> Result<Vec<MCPTool>> {
            Ok(vec![])
        }

        async fn save(&self, _tool: &MCPTool) -> Result<()> {
            Ok(())
        }

        async fn update(&self, _tool: &MCPTool) -> Result<()> {
            Ok(())
        }

        async fn update_without_new_version(&self, _tool: &MCPTool) -> Result<()> {
            Ok(())
        }

        async fn delete(&self, _id: MCPToolId) -> Result<()> {
            Ok(())
        }

        async fn exists_by_tenant_and_name(
            &self,
            _tenant_id: TenantId,
            _name: &str,
            _exclude_id: Option<MCPToolId>,
        ) -> Result<bool> {
            Ok(false)
        }

        async fn count_by_tenant(&self, _tenant_id: TenantId) -> Result<u64> {
            Ok(0)
        }

        async fn find_active_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<MCPTool>> {
            Ok(self
                .tools
                .iter()
                .filter(|t| t.tenant_id == tenant_id && t.can_execute())
                .cloned()
                .collect())
        }

        async fn get_version_history(
            &self,
            _tool_id: MCPToolId,
        ) -> Result<Vec<crate::domain::entities::MCPToolVersion>> {
            Ok(vec![])
        }

        async fn rollback_to_version(
            &self,
            _tool_id: MCPToolId,
            _target_version: i32,
            _created_by: UserId,
            _change_log: Option<String>,
        ) -> Result<MCPTool> {
            Err(PlatformError::NotFound("Not implemented".to_string()))
        }

        async fn compare_versions(
            &self,
            _tool_id: MCPToolId,
            _from_version: i32,
            _to_version: i32,
        ) -> Result<crate::domain::entities::VersionDiff> {
            Err(PlatformError::NotFound("Not implemented".to_string()))
        }

        async fn create_version(
            &self,
            _tool: &MCPTool,
            _change_log: Option<String>,
        ) -> Result<crate::domain::entities::MCPToolVersion> {
            Err(PlatformError::NotFound("Not implemented".to_string()))
        }
    }

    // Mock proxy service for testing
    struct MockMCPProxyService;

    #[async_trait]
    impl MCPProxyService for MockMCPProxyService {
        async fn register_tool(&self, _tool: MCPTool) -> Result<()> {
            Ok(())
        }

        async fn unregister_tool(&self, _tool_id: MCPToolId) -> Result<()> {
            Ok(())
        }

        async fn get_tenant_tools(&self, _tenant_id: TenantId) -> Result<Vec<MCPTool>> {
            Ok(vec![])
        }

        async fn call_tool(
            &self,
            _tool_id: MCPToolId,
            _parameters: Value,
            _context: ToolCallContext,
        ) -> Result<ToolCallResult> {
            Ok(ToolCallResult::success(
                serde_json::json!({"result": "success"}),
                100,
            ))
        }

        async fn handle_mcp_request(
            &self,
            _request: MCPRequest,
            _tenant_id: TenantId,
        ) -> Result<MCPResponse> {
            Err(PlatformError::NotFound("Not implemented".to_string()))
        }

        async fn test_tool_connection(&self, _tool_id: MCPToolId) -> Result<ToolCallResult> {
            Ok(ToolCallResult::success(
                serde_json::json!({"status": "ok"}),
                50,
            ))
        }

        async fn get_tool_stats(&self, _tenant_id: TenantId) -> Result<MCPToolStats> {
            Ok(MCPToolStats {
                total_tools: 0,
                active_tools: 0,
                inactive_tools: 0,
                tools_by_type: HashMap::new(),
            })
        }
    }

    fn create_test_tool(tenant_id: TenantId, tool_id: Uuid, name: &str, active: bool) -> MCPTool {
        let http_config = HTTPToolConfig {
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
            name.to_string(),
            Some("Test tool".to_string()),
            ToolConfig::HTTP(http_config),
            UserId::new(),
        );
        tool.id = MCPToolId(tool_id);
        if active {
            tool.activate();
        }
        tool
    }

    #[tokio::test]
    async fn test_list_tools_filters_by_permission() {
        let tenant_id = TenantId::new();
        let tool_id_1 = Uuid::new_v4();
        let tool_id_2 = Uuid::new_v4();
        let tool_id_3 = Uuid::new_v4();

        let tools = vec![
            create_test_tool(tenant_id, tool_id_1, "tool-1", true),
            create_test_tool(tenant_id, tool_id_2, "tool-2", true),
            create_test_tool(tenant_id, tool_id_3, "tool-3", true),
        ];

        let repository = Arc::new(MockMCPToolRepository::new(tools));
        let proxy_service = Arc::new(MockMCPProxyService);
        let service = MCPServerApplicationServiceImpl::new(repository, proxy_service);

        let auth_context = APIKeyAuthContext {
            api_key_id: Uuid::new_v4(),
            tenant_id: tenant_id.0,
            user_id: Uuid::new_v4(),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![],
                flow_ids: vec![],
                mcp_tool_ids: vec![tool_id_1, tool_id_3], // Only tool-1 and tool-3
                vector_store_ids: vec![],
            },
        };

        let result = service.list_tools(&auth_context).await.unwrap();

        assert_eq!(result.tools.len(), 2);
        assert!(result.tools.iter().any(|t| t.name == "tool-1"));
        assert!(result.tools.iter().any(|t| t.name == "tool-3"));
        assert!(!result.tools.iter().any(|t| t.name == "tool-2"));
    }

    #[tokio::test]
    async fn test_list_tools_returns_empty_for_no_permissions() {
        let tenant_id = TenantId::new();
        let tool_id_1 = Uuid::new_v4();

        let tools = vec![create_test_tool(tenant_id, tool_id_1, "tool-1", true)];

        let repository = Arc::new(MockMCPToolRepository::new(tools));
        let proxy_service = Arc::new(MockMCPProxyService);
        let service = MCPServerApplicationServiceImpl::new(repository, proxy_service);

        let auth_context = APIKeyAuthContext {
            api_key_id: Uuid::new_v4(),
            tenant_id: tenant_id.0,
            user_id: Uuid::new_v4(),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![],
                flow_ids: vec![],
                mcp_tool_ids: vec![], // Empty permissions
                vector_store_ids: vec![],
            },
        };

        let result = service.list_tools(&auth_context).await.unwrap();

        assert_eq!(result.tools.len(), 0);
    }

    #[tokio::test]
    async fn test_call_tool_with_valid_permission() {
        let tenant_id = TenantId::new();
        let tool_id = Uuid::new_v4();

        let tools = vec![create_test_tool(tenant_id, tool_id, "test-tool", true)];

        let repository = Arc::new(MockMCPToolRepository::new(tools));
        let proxy_service = Arc::new(MockMCPProxyService);
        let service = MCPServerApplicationServiceImpl::new(repository, proxy_service);

        let auth_context = APIKeyAuthContext {
            api_key_id: Uuid::new_v4(),
            tenant_id: tenant_id.0,
            user_id: Uuid::new_v4(),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![],
                flow_ids: vec![],
                mcp_tool_ids: vec![tool_id],
                vector_store_ids: vec![],
            },
        };

        let result = service
            .call_tool(&auth_context, "test-tool".to_string(), serde_json::json!({}))
            .await
            .unwrap();

        assert_eq!(result.is_error, None);
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_call_tool_without_permission() {
        let tenant_id = TenantId::new();
        let tool_id = Uuid::new_v4();

        let tools = vec![create_test_tool(tenant_id, tool_id, "test-tool", true)];

        let repository = Arc::new(MockMCPToolRepository::new(tools));
        let proxy_service = Arc::new(MockMCPProxyService);
        let service = MCPServerApplicationServiceImpl::new(repository, proxy_service);

        let auth_context = APIKeyAuthContext {
            api_key_id: Uuid::new_v4(),
            tenant_id: tenant_id.0,
            user_id: Uuid::new_v4(),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![],
                flow_ids: vec![],
                mcp_tool_ids: vec![Uuid::new_v4()], // Different tool ID
                vector_store_ids: vec![],
            },
        };

        let result = service
            .call_tool(&auth_context, "test-tool".to_string(), serde_json::json!({}))
            .await;

        assert!(result.is_err());
        match result {
            Err(PlatformError::Forbidden(_)) => (),
            _ => panic!("Expected Forbidden error"),
        }
    }

    #[tokio::test]
    async fn test_get_tool_schema_with_permission() {
        let tenant_id = TenantId::new();
        let tool_id = Uuid::new_v4();

        let tools = vec![create_test_tool(tenant_id, tool_id, "test-tool", true)];

        let repository = Arc::new(MockMCPToolRepository::new(tools));
        let proxy_service = Arc::new(MockMCPProxyService);
        let service = MCPServerApplicationServiceImpl::new(repository, proxy_service);

        let auth_context = APIKeyAuthContext {
            api_key_id: Uuid::new_v4(),
            tenant_id: tenant_id.0,
            user_id: Uuid::new_v4(),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![],
                flow_ids: vec![],
                mcp_tool_ids: vec![tool_id],
                vector_store_ids: vec![],
            },
        };

        let result = service
            .get_tool_schema(&auth_context, "test-tool".to_string())
            .await
            .unwrap();

        assert_eq!(result.name, "test-tool");
        assert_eq!(result.description, Some("Test tool".to_string()));
    }
}
