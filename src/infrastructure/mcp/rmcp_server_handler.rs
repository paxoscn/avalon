use rmcp::{
    handler::server::ServerHandler,
    model::*,
    service::{serve_server, RequestContext, RoleServer},
    ErrorData,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{error, info, warn};

use crate::{
    application::{
        dto::APIKeyAuthContext,
        services::mcp_server_application_service::MCPServerApplicationService,
    },
    error::PlatformError,
};

/// Configuration for the RMCP server
#[derive(Debug, Clone)]
pub struct RMCPServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout_seconds: u64,
}

impl Default for RMCPServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3001,
            max_connections: 100,
            request_timeout_seconds: 30,
        }
    }
}

/// RMCP Server Handler - Implements MCP protocol using rmcp library
pub struct RMCPServerHandler {
    mcp_service: Arc<dyn MCPServerApplicationService>,
    config: RMCPServerConfig,
    auth_context: Arc<RwLock<Option<APIKeyAuthContext>>>,
}

impl RMCPServerHandler {
    /// Create a new RMCP server handler
    pub fn new(
        mcp_service: Arc<dyn MCPServerApplicationService>,
        config: RMCPServerConfig,
    ) -> Self {
        Self {
            mcp_service,
            config,
            auth_context: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the authentication context for the current connection
    pub async fn set_auth_context(&self, context: APIKeyAuthContext) {
        let mut auth = self.auth_context.write().await;
        *auth = Some(context);
    }

    /// Get the authentication context
    async fn get_auth_context(&self) -> Result<APIKeyAuthContext, ErrorData> {
        let auth = self.auth_context.read().await;
        auth.clone().ok_or_else(|| {
            ErrorData::invalid_request("Authentication required", None)
        })
    }

    /// Build and start the RMCP server using stdio transport
    pub async fn start(self) -> Result<(), PlatformError> {
        info!("Starting RMCP server with stdio transport");

        // Create stdio transport using tokio's stdin/stdout
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        // Serve the server
        serve_server(self, (stdin, stdout))
            .await
            .map_err(|e| PlatformError::InternalError(format!("RMCP server error: {}", e)))?;

        Ok(())
    }
}

/// Implement the ServerHandler trait for handling MCP protocol operations
impl ServerHandler for RMCPServerHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: None,
                }),
                ..Default::default()
            },
            server_info: Implementation {
                name: "Agent Platform MCP Server".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some("Use API key authentication to access tools".into()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        // Get authentication context
        let auth_context = self.get_auth_context().await?;

        // Call the MCP service to list tools
        match self.mcp_service.list_tools(&auth_context).await {
            Ok(response) => {
                // Convert MCPToolDescriptor to rmcp Tool
                let tools: Vec<Tool> = response
                    .tools
                    .into_iter()
                    .map(|descriptor| {
                        // Convert Value to Arc<JsonObject>
                        let input_schema = if let Value::Object(obj) = descriptor.input_schema {
                            Arc::new(obj)
                        } else {
                            Arc::new(serde_json::Map::new())
                        };

                        Tool {
                            name: descriptor.name.into(),
                            title: None,
                            description: descriptor.description.map(|d| d.into()),
                            input_schema,
                            output_schema: None,
                            annotations: None,
                            icons: None,
                        }
                    })
                    .collect();

                info!("Listed {} tools for API key", tools.len());
                Ok(ListToolsResult {
                    tools,
                    next_cursor: None,
                })
            }
            Err(e) => {
                error!("Failed to list tools: {}", e);
                Err(ErrorData::internal_error(
                    format!("Failed to list tools: {}", e),
                    None,
                ))
            }
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        // Get authentication context
        let auth_context = self.get_auth_context().await?;

        info!("Calling tool: {}", request.name);

        // Convert arguments to Value
        let arguments = request
            .arguments
            .map(|args| serde_json::to_value(args).unwrap_or(Value::Object(Default::default())))
            .unwrap_or(Value::Object(Default::default()));

        // Call the MCP service to execute the tool
        match self
            .mcp_service
            .call_tool(&auth_context, request.name.to_string(), arguments)
            .await
        {
            Ok(response) => {
                // Check if the response indicates an error
                if response.is_error.unwrap_or(false) {
                    let error_message = response
                        .content
                        .first()
                        .and_then(|c| c.text.clone())
                        .unwrap_or_else(|| "Unknown error".to_string());

                    warn!("Tool execution failed: {}", error_message);
                    return Err(ErrorData::internal_error(error_message, None));
                }

                // Extract the result content
                let content_text = response
                    .content
                    .first()
                    .and_then(|c| c.text.clone())
                    .unwrap_or_else(|| "{}".to_string());

                info!("Tool {} executed successfully", request.name);
                Ok(CallToolResult {
                    content: vec![Content::text(content_text)],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            }
            Err(e) => {
                error!("Failed to call tool {}: {}", request.name, e);
                Err(ErrorData::internal_error(
                    format!("Failed to call tool: {}", e),
                    None,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::dto::PermissionScopeDTO,
        infrastructure::mcp::mcp_protocol::{MCPContent, MCPToolCallResponse, MCPToolDescriptor, MCPToolListResponse},
    };
    use async_trait::async_trait;
    use uuid::Uuid;

    // Mock MCP Server Application Service for testing
    struct MockMCPServerApplicationService {
        tools: Vec<MCPToolDescriptor>,
        should_succeed: bool,
    }

    #[async_trait]
    impl MCPServerApplicationService for MockMCPServerApplicationService {
        async fn list_tools(
            &self,
            _auth_context: &APIKeyAuthContext,
        ) -> Result<MCPToolListResponse, PlatformError> {
            Ok(MCPToolListResponse {
                tools: self.tools.clone(),
            })
        }

        async fn call_tool(
            &self,
            _auth_context: &APIKeyAuthContext,
            tool_name: String,
            _arguments: Value,
        ) -> Result<MCPToolCallResponse, PlatformError> {
            if self.should_succeed {
                Ok(MCPToolCallResponse {
                    content: vec![MCPContent::text(format!(
                        "{{\"result\": \"Tool {} executed successfully\"}}",
                        tool_name
                    ))],
                    is_error: None,
                })
            } else {
                Ok(MCPToolCallResponse {
                    content: vec![MCPContent::error("Tool execution failed".to_string())],
                    is_error: Some(true),
                })
            }
        }

        async fn get_tool_schema(
            &self,
            _auth_context: &APIKeyAuthContext,
            _tool_name: String,
        ) -> Result<MCPToolDescriptor, PlatformError> {
            Ok(MCPToolDescriptor {
                name: "test-tool".to_string(),
                description: Some("Test tool".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            })
        }
    }

    fn create_test_auth_context() -> APIKeyAuthContext {
        APIKeyAuthContext {
            api_key_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            permission_scope: PermissionScopeDTO {
                agent_ids: vec![],
                flow_ids: vec![],
                mcp_tool_ids: vec![Uuid::new_v4()],
                vector_store_ids: vec![],
            },
        }
    }

    fn create_test_tool_descriptor(name: &str) -> MCPToolDescriptor {
        MCPToolDescriptor {
            name: name.to_string(),
            description: Some(format!("{} description", name)),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "param1": {
                        "type": "string"
                    }
                }
            }),
        }
    }

    #[tokio::test]
    async fn test_rmcp_server_handler_creation() {
        let mock_service = Arc::new(MockMCPServerApplicationService {
            tools: vec![],
            should_succeed: true,
        });

        let config = RMCPServerConfig::default();
        let handler = RMCPServerHandler::new(mock_service, config);

        assert!(handler.auth_context.read().await.is_none());
    }

    #[tokio::test]
    async fn test_set_and_get_auth_context() {
        let mock_service = Arc::new(MockMCPServerApplicationService {
            tools: vec![],
            should_succeed: true,
        });

        let config = RMCPServerConfig::default();
        let handler = RMCPServerHandler::new(mock_service, config);

        let auth_context = create_test_auth_context();
        handler.set_auth_context(auth_context.clone()).await;

        let retrieved_context = handler.get_auth_context().await.unwrap();
        assert_eq!(retrieved_context.api_key_id, auth_context.api_key_id);
        assert_eq!(retrieved_context.tenant_id, auth_context.tenant_id);
        assert_eq!(retrieved_context.user_id, auth_context.user_id);
    }

    #[test]
    fn test_rmcp_server_config_default() {
        let config = RMCPServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3001);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.request_timeout_seconds, 30);
    }
}
