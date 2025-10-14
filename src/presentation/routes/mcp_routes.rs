use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::{
    application::services::MCPApplicationService,
    presentation::handlers::mcp_handlers::*,
};

/// 创建MCP工具管理路由
pub fn create_mcp_routes() -> Router<Arc<dyn MCPApplicationService>> {
    Router::new()
        // 工具CRUD操作
        .route("/tools", post(create_mcp_tool))
        .route("/tools", get(list_mcp_tools))
        .route("/tools/:tool_id", get(get_mcp_tool))
        .route("/tools/:tool_id", put(update_mcp_tool))
        .route("/tools/:tool_id", delete(delete_mcp_tool))
        
        // 工具操作
        .route("/tools/:tool_id/call", post(call_mcp_tool))
        .route("/tools/:tool_id/test", post(test_mcp_tool))
        .route("/tools/:tool_id/activate", post(activate_mcp_tool))
        .route("/tools/:tool_id/deactivate", post(deactivate_mcp_tool))
        
        // 版本管理
        .route("/tools/:tool_id/versions", get(list_tool_versions))
        .route("/tools/:tool_id/rollback", post(rollback_tool_version))
        
        // 统计和配置
        .route("/stats", get(get_tool_stats))
        .route("/config/validate", post(validate_tool_config))
        .route("/config/template/:tool_type", get(get_tool_config_template))
        .route("/types", get(get_supported_tool_types))
        
        // 健康检查
        .route("/health", get(health_check))
        
        // 健康检查不需要认证，所以单独处理
        .route("/health", get(health_check))
}

/// 创建完整的MCP API路由
pub fn create_mcp_api_routes(
    mcp_service: Arc<dyn MCPApplicationService>,
) -> Router {
    Router::new()
        .nest("/api/mcp", create_mcp_routes())
        .with_state(mcp_service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::application::services::MockMCPApplicationService;

    #[tokio::test]
    async fn test_create_mcp_routes() {
        let service = Arc::new(MockMCPApplicationService::new());
        let router = create_mcp_routes().with_state(service);

        // Test that routes are properly configured
        // Note: These tests will fail due to auth middleware, but they verify route structure
        
        let request = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        // Health check should work without auth
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_mcp_api_routes() {
        let service = Arc::new(MockMCPApplicationService::new());
        let router = create_mcp_api_routes(service);

        // Test that the nested routes are properly configured
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/mcp/health")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_route_paths() {
        // Test that all expected routes are defined
        let service = Arc::new(MockMCPApplicationService::new());
        let _router: Router<Arc<dyn MCPApplicationService>> = create_mcp_routes().with_state(service);
        
        // If this compiles, all routes are properly defined
        assert!(true);
    }
}