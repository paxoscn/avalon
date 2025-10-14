#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::{
        application::{
            dto::{CallMCPToolRequest, CreateMCPToolRequest},
            services::mcp_application_service::{MCPApplicationService, MCPApplicationServiceImpl},
        },
        domain::{
            entities::{MCPTool, MCPToolStatus},
            repositories::{
                mcp_tool_repository::{MCPToolQueryOptions, MCPToolQueryResult, MCPToolRepository},
                mcp_tool_version_repository::MCPToolVersionRepository,
            },
            services::mcp_tool_service::{
                ConfigValidationResult, MCPToolDomainService, PermissionCheckResult,
                ToolCallContext, ToolCallResult,
            },
            value_objects::{
                ids::{MCPToolId, TenantId, UserId},
                tool_config::{HTTPToolConfig, HttpMethod, ToolConfig},
            },
        },
        error::PlatformError,
        infrastructure::mcp::{MCPProxyService, MCPToolStats},
    };

    // Create mock implementations using mockall
    mockall::mock! {
        pub MCPToolRepositoryImpl {}

        #[async_trait::async_trait]
        impl MCPToolRepository for MCPToolRepositoryImpl {
            async fn find_by_id(&self, id: MCPToolId) -> Result<Option<MCPTool>, PlatformError>;
            async fn find_by_tenant_and_name(&self, tenant_id: TenantId, name: &str) -> Result<Option<MCPTool>, PlatformError>;
            async fn find_by_options(&self, options: MCPToolQueryOptions) -> Result<MCPToolQueryResult, PlatformError>;
            async fn find_by_tenant_id(&self, tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError>;
            async fn find_by_created_by(&self, created_by: UserId) -> Result<Vec<MCPTool>, PlatformError>;
            async fn save(&self, tool: &MCPTool) -> Result<(), PlatformError>;
            async fn update(&self, tool: &MCPTool) -> Result<(), PlatformError>;
            async fn delete(&self, id: MCPToolId) -> Result<(), PlatformError>;
            async fn exists_by_tenant_and_name(&self, tenant_id: TenantId, name: &str, exclude_id: Option<MCPToolId>) -> Result<bool, PlatformError>;
            async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64, PlatformError>;
            async fn find_active_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError>;
            async fn get_version_history(&self, tool_id: MCPToolId) -> Result<Vec<crate::domain::entities::MCPToolVersion>, PlatformError>;
            async fn rollback_to_version(&self, tool_id: MCPToolId, target_version: i32, created_by: UserId, change_log: Option<String>) -> Result<MCPTool, PlatformError>;
            async fn compare_versions(&self, tool_id: MCPToolId, from_version: i32, to_version: i32) -> Result<crate::domain::entities::VersionDiff, PlatformError>;
            async fn create_version(&self, tool: &MCPTool, change_log: Option<String>) -> Result<crate::domain::entities::MCPToolVersion, PlatformError>;
        }
    }

    mockall::mock! {
        pub MCPToolVersionRepositoryImpl {}

        #[async_trait::async_trait]
        impl MCPToolVersionRepository for MCPToolVersionRepositoryImpl {
            async fn find_by_id(&self, id: crate::domain::value_objects::ids::MCPToolVersionId) -> Result<Option<crate::domain::entities::MCPToolVersion>, PlatformError>;
            async fn find_by_tool_and_version(&self, tool_id: MCPToolId, version: i32) -> Result<Option<crate::domain::entities::MCPToolVersion>, PlatformError>;
            async fn find_by_options(&self, options: crate::domain::repositories::mcp_tool_version_repository::MCPToolVersionQueryOptions) -> Result<crate::domain::repositories::mcp_tool_version_repository::MCPToolVersionQueryResult, PlatformError>;
            async fn find_by_tool_id(&self, tool_id: MCPToolId) -> Result<Vec<crate::domain::entities::MCPToolVersion>, PlatformError>;
            async fn find_latest_by_tool_id(&self, tool_id: MCPToolId) -> Result<Option<crate::domain::entities::MCPToolVersion>, PlatformError>;
            async fn find_recent_by_tool_id(&self, tool_id: MCPToolId, limit: u64) -> Result<Vec<crate::domain::entities::MCPToolVersion>, PlatformError>;
            async fn save(&self, version: &crate::domain::entities::MCPToolVersion) -> Result<(), PlatformError>;
            async fn update(&self, version: &crate::domain::entities::MCPToolVersion) -> Result<(), PlatformError>;
            async fn delete(&self, id: crate::domain::value_objects::ids::MCPToolVersionId) -> Result<(), PlatformError>;
            async fn delete_by_tool_id(&self, tool_id: MCPToolId) -> Result<(), PlatformError>;
            async fn exists_by_tool_and_version(&self, tool_id: MCPToolId, version: i32) -> Result<bool, PlatformError>;
            async fn get_next_version_number(&self, tool_id: MCPToolId) -> Result<i32, PlatformError>;
            async fn count_by_tool_id(&self, tool_id: MCPToolId) -> Result<u64, PlatformError>;
            async fn compare_versions(&self, tool_id: MCPToolId, from_version: i32, to_version: i32) -> Result<crate::domain::entities::VersionDiff, PlatformError>;
            async fn get_version_history(&self, tool_id: MCPToolId) -> Result<Vec<crate::domain::entities::MCPToolVersion>, PlatformError>;
            async fn rollback_to_version(&self, tool_id: MCPToolId, target_version: i32, created_by: UserId, change_log: Option<String>) -> Result<crate::domain::entities::MCPToolVersion, PlatformError>;
        }
    }

    mockall::mock! {
        pub MCPToolDomainServiceImpl {}

        #[async_trait::async_trait]
        impl MCPToolDomainService for MCPToolDomainServiceImpl {
            async fn validate_tool_config(&self, config: &ToolConfig) -> Result<ConfigValidationResult, PlatformError>;
            async fn check_tool_permission(&self, tool: &MCPTool, context: &ToolCallContext) -> Result<PermissionCheckResult, PlatformError>;
            async fn check_call_permission(&self, tool: &MCPTool, context: &ToolCallContext, parameters: &serde_json::Value) -> Result<PermissionCheckResult, PlatformError>;
            async fn validate_call_parameters(&self, tool: &MCPTool, parameters: &serde_json::Value) -> Result<(), PlatformError>;
            async fn test_tool_connection(&self, config: &ToolConfig) -> Result<ToolCallResult, PlatformError>;
            fn create_call_context(&self, tenant_id: TenantId, user_id: UserId, request_id: String) -> ToolCallContext;
            fn can_execute_tool(&self, tool: &MCPTool) -> bool;
            async fn validate_tool_name_uniqueness(&self, tenant_id: TenantId, name: &str, exclude_tool_id: Option<MCPToolId>) -> Result<bool, PlatformError>;
        }
    }

    mockall::mock! {
        pub MCPProxyServiceImpl {}

        #[async_trait::async_trait]
        impl MCPProxyService for MCPProxyServiceImpl {
            async fn register_tool(&self, tool: MCPTool) -> Result<(), PlatformError>;
            async fn unregister_tool(&self, tool_id: MCPToolId) -> Result<(), PlatformError>;
            async fn get_tenant_tools(&self, tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError>;
            async fn call_tool(&self, tool_id: MCPToolId, parameters: serde_json::Value, context: ToolCallContext) -> Result<ToolCallResult, PlatformError>;
            async fn handle_mcp_request(&self, request: crate::infrastructure::mcp::protocol_handler::MCPRequest, tenant_id: TenantId) -> Result<crate::infrastructure::mcp::protocol_handler::MCPResponse, PlatformError>;
            async fn test_tool_connection(&self, tool_id: MCPToolId) -> Result<ToolCallResult, PlatformError>;
            async fn get_tool_stats(&self, tenant_id: TenantId) -> Result<MCPToolStats, PlatformError>;
        }
    }

    fn create_test_tool(tenant_id: TenantId, user_id: UserId) -> MCPTool {
        let config =
            HTTPToolConfig::new("https://api.example.com/test".to_string(), HttpMethod::GET);

        MCPTool::new(
            tenant_id,
            "test-tool".to_string(),
            Some("Test tool".to_string()),
            ToolConfig::HTTP(config),
            user_id,
        )
    }

    #[tokio::test]
    async fn test_create_tool_success() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();

        let mut tool_repo = MockMCPToolRepositoryImpl::new();
        let mut version_repo = MockMCPToolVersionRepositoryImpl::new();
        let mut domain_service = MockMCPToolDomainServiceImpl::new();
        let mut proxy_service = MockMCPProxyServiceImpl::new();

        // Setup expectations
        domain_service
            .expect_validate_tool_config()
            .times(1)
            .returning(|_| Ok(ConfigValidationResult::valid()));

        domain_service
            .expect_validate_tool_name_uniqueness()
            .with(eq(tenant_id), eq("test-tool"), eq(None))
            .times(1)
            .returning(|_, _, _| Ok(true));

        tool_repo.expect_save().times(1).returning(|_| Ok(()));

        version_repo.expect_save().times(1).returning(|_| Ok(()));

        proxy_service
            .expect_register_tool()
            .times(1)
            .returning(|_| Ok(()));

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        let request = CreateMCPToolRequest {
            tenant_id,
            name: "test-tool".to_string(),
            description: Some("Test tool".to_string()),
            config: ToolConfig::HTTP(HTTPToolConfig::new(
                "https://api.example.com/test".to_string(),
                HttpMethod::GET,
            )),
        };

        let result = service.create_tool(request, user_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.name, "test-tool");
        assert_eq!(response.tenant_id, tenant_id.0);
        assert_eq!(response.created_by, user_id.0);
    }

    #[tokio::test]
    async fn test_create_tool_invalid_config() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();

        let tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let mut domain_service = MockMCPToolDomainServiceImpl::new();
        let proxy_service = MockMCPProxyServiceImpl::new();

        // Setup expectations - invalid config
        domain_service
            .expect_validate_tool_config()
            .times(1)
            .returning(|_| {
                Ok(ConfigValidationResult::invalid(vec![
                    "Invalid endpoint".to_string()
                ]))
            });

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        let request = CreateMCPToolRequest {
            tenant_id,
            name: "test-tool".to_string(),
            description: Some("Test tool".to_string()),
            config: ToolConfig::HTTP(HTTPToolConfig::new(
                "".to_string(), // Invalid empty endpoint
                HttpMethod::GET,
            )),
        };

        let result = service.create_tool(request, user_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            PlatformError::ValidationError(msg) => {
                assert!(msg.contains("Invalid endpoint"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_create_tool_duplicate_name() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();

        let tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let mut domain_service = MockMCPToolDomainServiceImpl::new();
        let proxy_service = MockMCPProxyServiceImpl::new();

        // Setup expectations
        domain_service
            .expect_validate_tool_config()
            .times(1)
            .returning(|_| Ok(ConfigValidationResult::valid()));

        domain_service
            .expect_validate_tool_name_uniqueness()
            .with(eq(tenant_id), eq("test-tool"), eq(None))
            .times(1)
            .returning(|_, _, _| Ok(false)); // Name not unique

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        let request = CreateMCPToolRequest {
            tenant_id,
            name: "test-tool".to_string(),
            description: Some("Test tool".to_string()),
            config: ToolConfig::HTTP(HTTPToolConfig::new(
                "https://api.example.com/test".to_string(),
                HttpMethod::GET,
            )),
        };

        let result = service.create_tool(request, user_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            PlatformError::ValidationError(msg) => {
                assert!(msg.contains("already exists"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_get_tool_success() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let tool = create_test_tool(tenant_id, user_id);
        let tool_id = tool.id;

        let mut tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let mut domain_service = MockMCPToolDomainServiceImpl::new();
        let proxy_service = MockMCPProxyServiceImpl::new();

        // Setup expectations
        tool_repo
            .expect_find_by_id()
            .with(eq(tool_id))
            .times(1)
            .returning(move |_| Ok(Some(tool.clone())));

        domain_service
            .expect_create_call_context()
            .times(1)
            .returning(move |tenant_id, user_id, request_id| {
                ToolCallContext::new(tenant_id, user_id, request_id)
            });

        domain_service
            .expect_check_tool_permission()
            .times(1)
            .returning(|_, _| Ok(PermissionCheckResult::allowed()));

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        let result = service.get_tool(tool_id, user_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.id, tool_id.0);
        assert_eq!(response.name, "test-tool");
    }

    #[tokio::test]
    async fn test_get_tool_not_found() {
        let user_id = UserId::new();
        let tool_id = MCPToolId::new();

        let mut tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let domain_service = MockMCPToolDomainServiceImpl::new();
        let proxy_service = MockMCPProxyServiceImpl::new();

        // Setup expectations - tool not found
        tool_repo
            .expect_find_by_id()
            .with(eq(tool_id))
            .times(1)
            .returning(|_| Ok(None));

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        let result = service.get_tool(tool_id, user_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            PlatformError::NotFound(msg) => {
                assert_eq!(msg, "Tool not found");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_tool_access_denied() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let tool = create_test_tool(tenant_id, user_id);
        let tool_id = tool.id;

        let mut tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let mut domain_service = MockMCPToolDomainServiceImpl::new();
        let proxy_service = MockMCPProxyServiceImpl::new();

        // Setup expectations
        tool_repo
            .expect_find_by_id()
            .with(eq(tool_id))
            .times(1)
            .returning(move |_| Ok(Some(tool.clone())));

        domain_service
            .expect_create_call_context()
            .times(1)
            .returning(move |tenant_id, user_id, request_id| {
                ToolCallContext::new(tenant_id, user_id, request_id)
            });

        domain_service
            .expect_check_tool_permission()
            .times(1)
            .returning(|_, _| Ok(PermissionCheckResult::denied("Access denied".to_string())));

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        let result = service.get_tool(tool_id, user_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            PlatformError::AuthorizationFailed(msg) => {
                assert_eq!(msg, "Access denied");
            }
            _ => panic!("Expected AuthorizationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_call_tool_success() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let tool = create_test_tool(tenant_id, user_id);
        let tool_id = tool.id;

        let mut tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let mut domain_service = MockMCPToolDomainServiceImpl::new();
        let mut proxy_service = MockMCPProxyServiceImpl::new();

        // Setup expectations
        tool_repo
            .expect_find_by_id()
            .with(eq(tool_id))
            .times(1)
            .returning(move |_| Ok(Some(tool.clone())));

        domain_service
            .expect_check_call_permission()
            .times(1)
            .returning(|_, _, _| Ok(PermissionCheckResult::allowed()));

        domain_service
            .expect_validate_call_parameters()
            .times(1)
            .returning(|_, _| Ok(()));

        let call_result = ToolCallResult::success(serde_json::json!({"result": "success"}), 100);

        proxy_service
            .expect_call_tool()
            .times(1)
            .returning(move |_, _, _| Ok(call_result.clone()));

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        let request = CallMCPToolRequest {
            parameters: serde_json::json!({"param1": "value1"}),
            session_id: Some("test-session".to_string()),
            metadata: None,
        };

        let context = ToolCallContext::new(tenant_id, user_id, "test-request".to_string());

        let result = service.call_tool(tool_id, request, context).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.execution_time_ms, 100);
    }

    #[tokio::test]
    async fn test_list_tools_success() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let tool = create_test_tool(tenant_id, user_id);

        let mut tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let domain_service = MockMCPToolDomainServiceImpl::new();
        let proxy_service = MockMCPProxyServiceImpl::new();

        // Setup expectations
        tool_repo
            .expect_find_by_options()
            .times(1)
            .returning(move |_| {
                Ok(MCPToolQueryResult {
                    tools: vec![tool.clone()],
                    total_count: 1,
                })
            });

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        // Use 0-based pagination: page=0 for first page
        let result = service.list_tools(tenant_id, 0, 20).await;
        assert!(result.is_ok());

        let (tools, total) = result.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn test_list_tools_offset_calculation() {
        let tenant_id = TenantId::new();

        let mut tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let domain_service = MockMCPToolDomainServiceImpl::new();
        let proxy_service = MockMCPProxyServiceImpl::new();

        // Verify that offset is calculated as page * limit
        tool_repo
            .expect_find_by_options()
            .times(1)
            .withf(|options: &MCPToolQueryOptions| {
                // For page=3, limit=10, offset should be 30
                options.limit == Some(10) && options.offset == Some(30)
            })
            .returning(|_| {
                Ok(MCPToolQueryResult {
                    tools: vec![],
                    total_count: 100,
                })
            });

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        // Test page 3 with limit 10 (offset should be 3 * 10 = 30)
        let result = service.list_tools(tenant_id, 3, 10).await;
        assert!(result.is_ok());

        let (_, total) = result.unwrap();
        assert_eq!(total, 100);
    }

    #[tokio::test]
    async fn test_list_tools_total_count_accuracy() {
        let tenant_id = TenantId::new();

        let mut tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let domain_service = MockMCPToolDomainServiceImpl::new();
        let proxy_service = MockMCPProxyServiceImpl::new();

        // Verify total count is returned accurately
        tool_repo
            .expect_find_by_options()
            .times(1)
            .returning(|_| {
                Ok(MCPToolQueryResult {
                    tools: vec![],
                    total_count: 67,
                })
            });

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        let result = service.list_tools(tenant_id, 0, 20).await;
        assert!(result.is_ok());

        let (_, total) = result.unwrap();
        assert_eq!(total, 67);
    }

    #[tokio::test]
    async fn test_get_tool_stats_success() {
        let tenant_id = TenantId::new();

        let tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let domain_service = MockMCPToolDomainServiceImpl::new();
        let mut proxy_service = MockMCPProxyServiceImpl::new();

        let mut tools_by_type = HashMap::new();
        tools_by_type.insert("http".to_string(), 2);

        let stats = MCPToolStats {
            total_tools: 2,
            active_tools: 1,
            inactive_tools: 1,
            tools_by_type,
        };

        proxy_service
            .expect_get_tool_stats()
            .with(eq(tenant_id))
            .times(1)
            .returning(move |_| Ok(stats.clone()));

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        let result = service.get_tool_stats(tenant_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.total_tools, 2);
        assert_eq!(response.active_tools, 1);
        assert_eq!(response.inactive_tools, 1);
        assert_eq!(response.tools_by_type.get("http"), Some(&2));
    }

    #[tokio::test]
    async fn test_activate_tool_success() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let mut tool = create_test_tool(tenant_id, user_id);
        tool.deactivate(); // Start with inactive tool
        let tool_id = tool.id;

        let mut tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let mut domain_service = MockMCPToolDomainServiceImpl::new();
        let mut proxy_service = MockMCPProxyServiceImpl::new();

        // Setup expectations
        tool_repo
            .expect_find_by_id()
            .with(eq(tool_id))
            .times(1)
            .returning(move |_| Ok(Some(tool.clone())));

        domain_service
            .expect_create_call_context()
            .times(1)
            .returning(move |tenant_id, user_id, request_id| {
                ToolCallContext::new(tenant_id, user_id, request_id)
            });

        domain_service
            .expect_check_tool_permission()
            .times(1)
            .returning(|_, _| Ok(PermissionCheckResult::allowed()));

        tool_repo.expect_save().times(1).returning(|_| Ok(()));

        proxy_service
            .expect_register_tool()
            .times(1)
            .returning(|_| Ok(()));

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        let result = service.activate_tool(tool_id, user_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, MCPToolStatus::Active);
    }

    #[tokio::test]
    async fn test_delete_tool_success() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let tool = create_test_tool(tenant_id, user_id);
        let tool_id = tool.id;

        let mut tool_repo = MockMCPToolRepositoryImpl::new();
        let version_repo = MockMCPToolVersionRepositoryImpl::new();
        let mut domain_service = MockMCPToolDomainServiceImpl::new();
        let mut proxy_service = MockMCPProxyServiceImpl::new();

        // Setup expectations
        tool_repo
            .expect_find_by_id()
            .with(eq(tool_id))
            .times(1)
            .returning(move |_| Ok(Some(tool.clone())));

        domain_service
            .expect_create_call_context()
            .times(1)
            .returning(move |tenant_id, user_id, request_id| {
                ToolCallContext::new(tenant_id, user_id, request_id)
            });

        domain_service
            .expect_check_tool_permission()
            .times(1)
            .returning(|_, _| Ok(PermissionCheckResult::allowed()));

        proxy_service
            .expect_unregister_tool()
            .with(eq(tool_id))
            .times(1)
            .returning(|_| Ok(()));

        tool_repo
            .expect_delete()
            .with(eq(tool_id))
            .times(1)
            .returning(|_| Ok(()));

        let service = MCPApplicationServiceImpl::new(
            Arc::new(tool_repo),
            Arc::new(version_repo),
            Arc::new(domain_service),
            Arc::new(proxy_service),
        );

        let result = service.delete_tool(tool_id, user_id).await;
        assert!(result.is_ok());
    }
}
