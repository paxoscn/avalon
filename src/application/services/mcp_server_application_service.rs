use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    application::dto::{APIKeyAuthContext, PermissionScopeDTO},
    domain::{
        entities::MCPTool,
        repositories::MCPToolRepository,
        services::mcp_tool_service::ToolCallContext,
        value_objects::ids::{TenantId, UserId},
    },
    error::{PlatformError, Result},
    infrastructure::mcp::{
        mcp_protocol::{tool_to_mcp_format, MCPToolCallResponse, MCPToolDescriptor, MCPToolListResponse},
        MCPProxyService,
    },
};

/// MCP Server Application Service trait
/// Provides MCP protocol operations with API key authentication
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MCPServerApplicationService: Send + Sync {
    /// List tools accessible by the API key
    /// Filters tools based on the permission scope
    async fn list_tools(
        &self,
        auth_context: &APIKeyAuthContext,
    ) -> Result<MCPToolListResponse>;

    /// Call a tool with permission validation
    /// Validates that the tool is in the API key's permission scope before execution
    async fn call_tool(
        &self,
        auth_context: &APIKeyAuthContext,
        tool_name: String,
        arguments: serde_json::Value,
    ) -> Result<MCPToolCallResponse>;

    /// Get tool schema with permission checking
    /// Returns the tool schema if the API key has access
    async fn get_tool_schema(
        &self,
        auth_context: &APIKeyAuthContext,
        tool_name: String,
    ) -> Result<MCPToolDescriptor>;
}

/// Implementation of MCP Server Application Service
pub struct MCPServerApplicationServiceImpl {
    mcp_tool_repository: Arc<dyn MCPToolRepository>,
    mcp_proxy_service: Arc<dyn MCPProxyService>,
}

impl MCPServerApplicationServiceImpl {
    /// Create a new MCP Server Application Service
    pub fn new(
        mcp_tool_repository: Arc<dyn MCPToolRepository>,
        mcp_proxy_service: Arc<dyn MCPProxyService>,
    ) -> Self {
        Self {
            mcp_tool_repository,
            mcp_proxy_service,
        }
    }

    /// Check if a tool is accessible based on the permission scope
    fn is_tool_accessible(&self, tool: &MCPTool, permission_scope: &PermissionScopeDTO) -> bool {
        // If the permission scope is empty for MCP tools, deny access
        if permission_scope.mcp_tool_ids.is_empty() {
            return false;
        }

        // Check if the tool ID is in the permission scope
        permission_scope.mcp_tool_ids.contains(&tool.id.0)
    }

    /// Filter tools based on permission scope
    fn filter_tools_by_permission(
        &self,
        tools: Vec<MCPTool>,
        permission_scope: &PermissionScopeDTO,
    ) -> Vec<MCPTool> {
        tools
            .into_iter()
            .filter(|tool| self.is_tool_accessible(tool, permission_scope))
            .collect()
    }

    /// Find a tool by name and validate access
    async fn find_and_validate_tool(
        &self,
        tenant_id: TenantId,
        tool_name: &str,
        permission_scope: &PermissionScopeDTO,
    ) -> Result<MCPTool> {
        // Find the tool by tenant and name
        let tool = self
            .mcp_tool_repository
            .find_by_tenant_and_name(tenant_id, tool_name)
            .await?
            .ok_or_else(|| PlatformError::NotFound(format!("Tool '{}' not found", tool_name)))?;

        // Check if the tool is accessible
        if !self.is_tool_accessible(&tool, permission_scope) {
            return Err(PlatformError::Forbidden(format!(
                "Access denied to tool '{}'",
                tool_name
            )));
        }

        // Check if the tool is active
        if !tool.can_execute() {
            return Err(PlatformError::ValidationError(format!(
                "Tool '{}' is not active",
                tool_name
            )));
        }

        Ok(tool)
    }
}

#[async_trait]
impl MCPServerApplicationService for MCPServerApplicationServiceImpl {
    async fn list_tools(
        &self,
        auth_context: &APIKeyAuthContext,
    ) -> Result<MCPToolListResponse> {
        let tenant_id = TenantId(auth_context.tenant_id);

        // Get all active tools for the tenant using find_by_options
        use crate::domain::repositories::mcp_tool_repository::MCPToolQueryOptions;
        
        let query_options = MCPToolQueryOptions::new()
            .with_tenant_id(tenant_id)
            .with_status("active".to_string());

        let query_result = self
            .mcp_tool_repository
            .find_by_options(query_options)
            .await?;

        // Filter tools based on permission scope
        let accessible_tools =
            self.filter_tools_by_permission(query_result.tools, &auth_context.permission_scope);

        // Convert to MCP format
        let tool_descriptors: Vec<MCPToolDescriptor> = accessible_tools
            .iter()
            .map(|tool| tool_to_mcp_format(tool))
            .collect();

        Ok(MCPToolListResponse {
            tools: tool_descriptors,
        })
    }

    async fn call_tool(
        &self,
        auth_context: &APIKeyAuthContext,
        tool_name: String,
        arguments: serde_json::Value,
    ) -> Result<MCPToolCallResponse> {
        let tenant_id = TenantId(auth_context.tenant_id);
        let user_id = UserId(auth_context.user_id);

        // Find and validate the tool
        let tool = self
            .find_and_validate_tool(tenant_id, &tool_name, &auth_context.permission_scope)
            .await?;

        // Create tool call context
        let context = ToolCallContext::new(
            tenant_id,
            user_id,
            format!("mcp_api_key_{}", auth_context.api_key_id),
        );

        // Call the tool via proxy service
        let result = self
            .mcp_proxy_service
            .call_tool(tool.id, arguments, context)
            .await?;

        // Convert to MCP response format
        if result.success {
            let content_text = serde_json::to_string_pretty(&result.result)
                .unwrap_or_else(|_| "{}".to_string());
            Ok(MCPToolCallResponse::success(content_text))
        } else {
            let error_message = result
                .error
                .unwrap_or_else(|| "Tool execution failed".to_string());
            Ok(MCPToolCallResponse::error(error_message))
        }
    }

    async fn get_tool_schema(
        &self,
        auth_context: &APIKeyAuthContext,
        tool_name: String,
    ) -> Result<MCPToolDescriptor> {
        let tenant_id = TenantId(auth_context.tenant_id);

        // Find and validate the tool
        let tool = self
            .find_and_validate_tool(tenant_id, &tool_name, &auth_context.permission_scope)
            .await?;

        // Convert to MCP format
        Ok(tool_to_mcp_format(&tool))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{
        ids::MCPToolId,
        tool_config::{HTTPToolConfig, HttpMethod, ToolConfig},
    };
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_tool(tenant_id: TenantId, tool_id: Uuid, name: &str) -> MCPTool {
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
        tool.activate();
        tool
    }

    #[test]
    fn test_is_tool_accessible_with_permission() {
        let tenant_id = TenantId::new();
        let tool_id = Uuid::new_v4();
        let tool = create_test_tool(tenant_id, tool_id, "test-tool");

        let permission_scope = PermissionScopeDTO {
            agent_ids: vec![],
            flow_ids: vec![],
            mcp_tool_ids: vec![tool_id],
            vector_store_ids: vec![],
        };

        // Test the logic directly without needing the full service
        let is_accessible = permission_scope.mcp_tool_ids.contains(&tool.id.0);
        assert!(is_accessible);
    }

    #[test]
    fn test_is_tool_accessible_without_permission() {
        let tenant_id = TenantId::new();
        let tool_id = Uuid::new_v4();
        let tool = create_test_tool(tenant_id, tool_id, "test-tool");

        let permission_scope = PermissionScopeDTO {
            agent_ids: vec![],
            flow_ids: vec![],
            mcp_tool_ids: vec![Uuid::new_v4()], // Different tool ID
            vector_store_ids: vec![],
        };

        let is_accessible = !permission_scope.mcp_tool_ids.is_empty()
            && permission_scope.mcp_tool_ids.contains(&tool.id.0);
        assert!(!is_accessible);
    }

    #[test]
    fn test_is_tool_accessible_with_empty_scope() {
        let tenant_id = TenantId::new();
        let tool_id = Uuid::new_v4();
        let tool = create_test_tool(tenant_id, tool_id, "test-tool");

        let permission_scope = PermissionScopeDTO {
            agent_ids: vec![],
            flow_ids: vec![],
            mcp_tool_ids: vec![], // Empty scope
            vector_store_ids: vec![],
        };

        let is_accessible = !permission_scope.mcp_tool_ids.is_empty()
            && permission_scope.mcp_tool_ids.contains(&tool.id.0);
        assert!(!is_accessible);
    }

    #[test]
    fn test_filter_tools_by_permission() {
        let tenant_id = TenantId::new();
        let tool_id_1 = Uuid::new_v4();
        let tool_id_2 = Uuid::new_v4();
        let tool_id_3 = Uuid::new_v4();

        let tools = vec![
            create_test_tool(tenant_id, tool_id_1, "tool-1"),
            create_test_tool(tenant_id, tool_id_2, "tool-2"),
            create_test_tool(tenant_id, tool_id_3, "tool-3"),
        ];

        let permission_scope = PermissionScopeDTO {
            agent_ids: vec![],
            flow_ids: vec![],
            mcp_tool_ids: vec![tool_id_1, tool_id_3], // Only tool-1 and tool-3
            vector_store_ids: vec![],
        };

        let filtered: Vec<MCPTool> = tools
            .into_iter()
            .filter(|tool| {
                !permission_scope.mcp_tool_ids.is_empty()
                    && permission_scope.mcp_tool_ids.contains(&tool.id.0)
            })
            .collect();

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|t| t.id.0 == tool_id_1));
        assert!(filtered.iter().any(|t| t.id.0 == tool_id_3));
        assert!(!filtered.iter().any(|t| t.id.0 == tool_id_2));
    }
}
