use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::{
        dto::{
            CreateMCPToolRequest, UpdateMCPToolRequest, MCPToolResponse,
            MCPToolListResponse, CallMCPToolRequest, CallMCPToolResponse,
            TestMCPToolRequest, TestMCPToolResponse, MCPToolVersionResponse,
            MCPToolStatsResponse, MCPToolListQuery, RollbackVersionRequest,
            ValidateToolConfigRequest, ValidateToolConfigResponse,
        },
        services::MCPApplicationService,
    },
    domain::{
        services::mcp_tool_service::ToolCallContext,
        value_objects::ids::MCPToolId,
    },
    error::PlatformError,
    presentation::extractors::AuthenticatedUser,
};

/// 创建MCP工具
pub async fn create_mcp_tool(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Json(request): Json<CreateMCPToolRequest>,
) -> Result<Json<MCPToolResponse>, PlatformError> {
    // 验证请求中的租户ID与用户的租户ID匹配
    if request.tenant_id.0 != user.tenant_id.0 {
        return Err(PlatformError::AuthorizationFailed(
            "Cannot create tool for different tenant".to_string()
        ));
    }

    let response = service
        .create_tool(request, user.user_id)
        .await?;

    Ok(Json(response))
}

/// 更新MCP工具
pub async fn update_mcp_tool(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Path(tool_id): Path<Uuid>,
    Json(request): Json<UpdateMCPToolRequest>,
) -> Result<Json<MCPToolResponse>, PlatformError> {
    let response = service
        .update_tool(MCPToolId(tool_id), request, user.user_id)
        .await?;

    Ok(Json(response))
}

/// 删除MCP工具
pub async fn delete_mcp_tool(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Path(tool_id): Path<Uuid>,
) -> Result<StatusCode, PlatformError> {
    service
        .delete_tool(MCPToolId(tool_id), user.user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// 获取MCP工具详情
pub async fn get_mcp_tool(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Path(tool_id): Path<Uuid>,
) -> Result<Json<MCPToolResponse>, PlatformError> {
    let response = service
        .get_tool(MCPToolId(tool_id), user.user_id)
        .await?;

    Ok(Json(response))
}

/// 获取MCP工具列表
pub async fn list_mcp_tools(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<MCPToolListQuery>,
) -> Result<Json<MCPToolListResponse>, PlatformError> {
    let response = service
        .list_tools(
            user.tenant_id,
            query.page,
            query.limit,
        )
        .await?;

    Ok(Json(response))
}

/// 调用MCP工具
pub async fn call_mcp_tool(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Path(tool_id): Path<Uuid>,
    Json(request): Json<CallMCPToolRequest>,
) -> Result<Json<CallMCPToolResponse>, PlatformError> {
    // 创建调用上下文
    let mut context = ToolCallContext::new(
        user.tenant_id,
        user.user_id,
        Uuid::new_v4().to_string(),
    );

    if let Some(session_id) = request.session_id.clone() {
        context = context.with_session_id(session_id);
    }

    if let Some(metadata) = request.metadata.clone() {
        for (key, value) in metadata {
            context = context.with_metadata(key, value);
        }
    }

    let response = service
        .call_tool(MCPToolId(tool_id), request, context)
        .await?;

    Ok(Json(response))
}

/// 测试MCP工具连接
pub async fn test_mcp_tool(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Path(tool_id): Path<Uuid>,
    Json(request): Json<TestMCPToolRequest>,
) -> Result<Json<TestMCPToolResponse>, PlatformError> {
    let response = service
        .test_tool(MCPToolId(tool_id), request, user.user_id)
        .await?;

    Ok(Json(response))
}

/// 激活MCP工具
pub async fn activate_mcp_tool(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Path(tool_id): Path<Uuid>,
) -> Result<Json<MCPToolResponse>, PlatformError> {
    let response = service
        .activate_tool(MCPToolId(tool_id), user.user_id)
        .await?;

    Ok(Json(response))
}

/// 停用MCP工具
pub async fn deactivate_mcp_tool(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Path(tool_id): Path<Uuid>,
) -> Result<Json<MCPToolResponse>, PlatformError> {
    let response = service
        .deactivate_tool(MCPToolId(tool_id), user.user_id)
        .await?;

    Ok(Json(response))
}

/// 获取工具版本列表
pub async fn list_tool_versions(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Path(tool_id): Path<Uuid>,
) -> Result<Json<Vec<MCPToolVersionResponse>>, PlatformError> {
    let response = service
        .list_tool_versions(MCPToolId(tool_id), user.user_id)
        .await?;

    Ok(Json(response))
}

/// 回退工具版本
pub async fn rollback_tool_version(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
    Path(tool_id): Path<Uuid>,
    Json(request): Json<RollbackVersionRequest>,
) -> Result<Json<MCPToolResponse>, PlatformError> {
    let response = service
        .rollback_tool_version(
            MCPToolId(tool_id),
            request.target_version,
            user.user_id,
        )
        .await?;

    Ok(Json(response))
}

/// 获取工具统计信息
pub async fn get_tool_stats(
    State(service): State<Arc<dyn MCPApplicationService>>,
    user: AuthenticatedUser,
) -> Result<Json<MCPToolStatsResponse>, PlatformError> {
    let response = service
        .get_tool_stats(user.tenant_id)
        .await?;

    Ok(Json(response))
}

/// 验证工具配置
pub async fn validate_tool_config(
    State(service): State<Arc<dyn MCPApplicationService>>,
    _user: AuthenticatedUser,
    Json(request): Json<ValidateToolConfigRequest>,
) -> Result<Json<ValidateToolConfigResponse>, PlatformError> {
    let validation_result = service
        .validate_tool_config(&request.config)
        .await?;

    let response = ValidateToolConfigResponse {
        valid: validation_result.valid,
        errors: validation_result.errors,
        warnings: validation_result.warnings,
    };

    Ok(Json(response))
}

/// 获取工具配置模板
pub async fn get_tool_config_template(
    _user: AuthenticatedUser,
    Path(tool_type): Path<String>,
) -> Result<Json<Value>, PlatformError> {
    let template = match tool_type.as_str() {
        "http" => serde_json::json!({
            "type": "http",
            "endpoint": "https://api.example.com/endpoint",
            "method": "GET",
            "headers": {},
            "parameters": [],
            "timeout_seconds": 30,
            "retry_count": 3
        }),
        _ => {
            return Err(PlatformError::ValidationError(
                format!("Unknown tool type: {}", tool_type)
            ));
        }
    };

    Ok(Json(template))
}

/// 获取支持的工具类型列表
pub async fn get_supported_tool_types(
    _user: AuthenticatedUser,
) -> Result<Json<Vec<String>>, PlatformError> {
    let tool_types = vec!["http".to_string()];
    Ok(Json(tool_types))
}

/// 健康检查端点
pub async fn health_check() -> Result<Json<Value>, PlatformError> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "mcp-tools",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use mockall::predicate::*;
    use serde_json::json;
    use tower::ServiceExt;

    use crate::{
        application::services::MockMCPApplicationService,
        domain::{
            entities::MCPToolStatus,
            value_objects::{
                ids::{TenantId, UserId, MCPToolId},
                tool_config::{HTTPToolConfig, HttpMethod, ToolConfig},
            },
        },
        presentation::extractors::AuthenticatedUser,
    };

    fn create_test_user() -> AuthenticatedUser {
        AuthenticatedUser {
            user_id: UserId(Uuid::new_v4()),
            tenant_id: TenantId(Uuid::new_v4()),
            username: "testuser".to_string(),
            nickname: Some("Test User".to_string()),
        }
    }

    fn create_test_tool_response() -> MCPToolResponse {
        MCPToolResponse {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            name: "test-tool".to_string(),
            description: Some("Test tool".to_string()),
            current_version: 1,
            status: crate::domain::entities::MCPToolStatus::Active,
            config: ToolConfig::HTTP(HTTPToolConfig::new(
                "https://api.example.com/test".to_string(),
                HttpMethod::GET,
            )),
            created_by: Uuid::new_v4(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_mcp_tool_success() {
        let mut service = MockMCPApplicationService::new();
        let user = create_test_user();
        let tool_response = create_test_tool_response();

        service
            .expect_create_tool()
            .times(1)
            .returning(move |_, _| Ok(tool_response.clone()));

        let request = CreateMCPToolRequest {
            tenant_id: user.tenant_id,
            name: "test-tool".to_string(),
            description: Some("Test tool".to_string()),
            config: ToolConfig::HTTP(HTTPToolConfig::new(
                "https://api.example.com/test".to_string(),
                HttpMethod::GET,
            )),
        };

        let result = create_mcp_tool(
            State(Arc::new(service)),
            user,
            Json(request),
        ).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.name, "test-tool");
    }

    #[tokio::test]
    async fn test_create_mcp_tool_wrong_tenant() {
        let service = MockMCPApplicationService::new();
        let user = create_test_user();

        let request = CreateMCPToolRequest {
            tenant_id: TenantId(Uuid::new_v4()), // Different tenant ID
            name: "test-tool".to_string(),
            description: Some("Test tool".to_string()),
            config: ToolConfig::HTTP(HTTPToolConfig::new(
                "https://api.example.com/test".to_string(),
                HttpMethod::GET,
            )),
        };

        let result = create_mcp_tool(
            State(Arc::new(service)),
            user,
            Json(request),
        ).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::AuthorizationFailed(msg) => {
                assert!(msg.contains("different tenant"));
            }
            _ => panic!("Expected AuthorizationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_get_mcp_tool_success() {
        let mut service = MockMCPApplicationService::new();
        let user = create_test_user();
        let tool_response = create_test_tool_response();
        let tool_id = tool_response.id;

        service
            .expect_get_tool()
            .with(eq(MCPToolId(tool_id)), eq(user.user_id))
            .times(1)
            .returning(move |_, _| Ok(tool_response.clone()));

        let result = get_mcp_tool(
            State(Arc::new(service)),
            user,
            Path(tool_id),
        ).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.id, tool_id);
    }

    #[tokio::test]
    async fn test_list_mcp_tools_success() {
        let mut service = MockMCPApplicationService::new();
        let user = create_test_user();
        let tool_response = create_test_tool_response();

        let list_response = MCPToolListResponse {
            tools: vec![tool_response],
            total: 1,
            page: 1,
            limit: 20,
            total_pages: 1,
        };

        service
            .expect_list_tools()
            .times(1)
            .returning(move |_, _, _| Ok(list_response.clone()));

        let query = MCPToolListQuery::default();

        let result = list_mcp_tools(
            State(Arc::new(service)),
            user,
            Query(query),
        ).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.tools.len(), 1);
        assert_eq!(response.0.total, 1);
    }

    #[tokio::test]
    async fn test_call_mcp_tool_success() {
        let mut service = MockMCPApplicationService::new();
        let user = create_test_user();
        let tool_id = Uuid::new_v4();

        let call_response = CallMCPToolResponse {
            success: true,
            result: Some(json!({"result": "success"})),
            error: None,
            execution_time_ms: 100,
            metadata: std::collections::HashMap::new(),
        };

        service
            .expect_call_tool()
            .times(1)
            .returning(move |_, _, _| Ok(call_response.clone()));

        let request = CallMCPToolRequest {
            parameters: json!({"param1": "value1"}),
            session_id: Some("test-session".to_string()),
            metadata: None,
        };

        let result = call_mcp_tool(
            State(Arc::new(service)),
            user,
            Path(tool_id),
            Json(request),
        ).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.0.success);
        assert_eq!(response.0.execution_time_ms, 100);
    }

    #[tokio::test]
    async fn test_validate_tool_config_success() {
        let mut service = MockMCPApplicationService::new();
        let user = create_test_user();

        let validation_result = crate::domain::services::mcp_tool_service::ConfigValidationResult::valid();

        service
            .expect_validate_tool_config()
            .times(1)
            .returning(move |_| Ok(validation_result.clone()));

        let request = ValidateToolConfigRequest {
            config: ToolConfig::HTTP(HTTPToolConfig::new(
                "https://api.example.com/test".to_string(),
                HttpMethod::GET,
            )),
        };

        let result = validate_tool_config(
            State(Arc::new(service)),
            user,
            Json(request),
        ).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.0.valid);
        assert!(response.0.errors.is_empty());
    }

    #[tokio::test]
    async fn test_get_supported_tool_types() {
        let user = create_test_user();

        let result = get_supported_tool_types(user).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.0.contains(&"http".to_string()));
    }

    #[tokio::test]
    async fn test_health_check() {
        let result = health_check().await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0["status"], "healthy");
        assert_eq!(response.0["service"], "mcp-tools");
    }
}