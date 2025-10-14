use crate::{
    application::services::*,
    config::AppConfig,
    domain::services::*,
    error::Result,
    infrastructure::{llm::LLMProviderRegistry, mcp::{proxy_service, MCPProxyServiceImpl}, repositories::*, vector::VectorStoreRegistry, Database, RedisCache},
    presentation::{
        routes::{
            audit_routes, create_app_router, create_mcp_api_routes, execution_history_routes, flow_routes, llm_config_routes, session_routes, vector_config_routes
        },
        handlers::{
            login_handler, refresh_token_handler, logout_handler,
            change_password_handler, me_handler, health_handler,
        },
        middleware::auth_middleware,
    },
};
use axum::{
    middleware,
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub struct Server {
    config: AppConfig,
    database: Arc<Database>,
    #[allow(dead_code)]
    cache: Arc<RedisCache>,
}

impl Server {
    pub fn new(config: AppConfig, database: Arc<Database>, cache: Arc<RedisCache>) -> Self {
        Self {
            config,
            database,
            cache,
        }
    }

    pub async fn start(self) -> Result<()> {
        let app = self.create_app();

        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        tracing::info!("Starting server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
            crate::error::PlatformError::InternalError(format!("Failed to bind to {}: {}", addr, e))
        })?;

        axum::serve(listener, app).await.map_err(|e| {
            crate::error::PlatformError::InternalError(format!("Server error: {}", e))
        })?;

        Ok(())
    }

    fn create_app(&self) -> Router {
        // Create repositories
        let user_repository = Arc::new(UserRepositoryImpl::new(self.database.connection()));
        let tenant_repository = Arc::new(TenantRepositoryImpl::new(self.database.connection()));
        let flow_repository = Arc::new(FlowRepositoryImpl::new(self.database.connection()));
        let flow_version_repository = Arc::new(FlowVersionRepositoryImpl::new(self.database.connection()));
        let flow_execution_repository = Arc::new(FlowExecutionRepositoryImpl::new(self.database.connection()));
        let llm_config_repository = Arc::new(LLMConfigRepositoryImpl::new(self.database.connection()));
        let vector_config_repository = Arc::new(VectorConfigRepositoryImpl::new(self.database.connection()));
        let mcp_tool_repository = Arc::new(MCPToolRepositoryImpl::new(self.database.connection()));
        let mcp_version_repository = Arc::new(MCPToolVersionRepositoryImpl::new(self.database.connection()));
        let session_repository = Arc::new(ChatSessionRepositoryImpl::new(self.database.connection()));
        let message_repository = Arc::new(MessageRepositoryImpl::new(self.database.connection()));
        let audit_repository = Arc::new(AuditLogRepositoryImpl::new(self.database.connection()));
        let execution_history_repository = Arc::new(ExecutionHistoryRepositoryImpl::new(self.database.connection()));

        let vector_store_registry = Arc::new(VectorStoreRegistry::new());
        let llm_provider_registry = Arc::new(LLMProviderRegistry::new());
        let mcp_proxy_service = Arc::new(MCPProxyServiceImpl::new());

        // Create domain services
        let auth_domain_service = Arc::new(AuthenticationDomainServiceImpl::new(
            self.config.jwt_secret.clone(),
            Some(self.config.bcrypt_cost),
        ));
        
        let flow_domain_service: Arc<dyn FlowDomainService> = Arc::new(FlowDomainServiceImpl::new());
        let llm_domain_service: Arc<dyn LLMDomainService> = Arc::new(LLMDomainServiceImpl::new());
        let vector_store_domain_service: Arc<dyn VectorStoreDomainService> = Arc::new(VectorStoreDomainServiceImpl::new());
        let mcp_domain_service: Arc<dyn MCPToolDomainService> = Arc::new(MCPToolDomainServiceImpl::new());
        let session_domain_service: Arc<SessionDomainService> = Arc::new(SessionDomainService::new(30));
        let audit_domain_service: Arc<dyn AuditService> = Arc::new(AuditServiceImpl::new(audit_repository));

        let execution_engine = Arc::new(ExecutionEngineImpl::new(vec![]));

        // Create application services
        let auth_service: Arc<dyn AuthApplicationService> =
            Arc::new(AuthApplicationServiceImpl::new(
                user_repository,
                tenant_repository,
                auth_domain_service,
                None, // Use default token expiry
            ));

        let flow_service: Arc<dyn FlowApplicationService> =
            Arc::new(FlowApplicationServiceImpl::new(
                flow_repository,
                flow_version_repository,
                flow_execution_repository,
                flow_domain_service,
                Some(execution_engine),
            ));

        let llm_service: Arc<dyn LLMApplicationService> =
            Arc::new(LLMApplicationServiceImpl::new(
                llm_config_repository,
                llm_domain_service,
                llm_provider_registry,
            ));

        let vector_service = Arc::new(VectorApplicationService::new(
            vector_config_repository.clone(),
        ));

        // let vector_storage_service = Arc::new(VectorStorageApplicationService::new(
        //     vector_service,
        //     vector_store_registry,
        // ));

        let mcp_service: Arc<dyn MCPApplicationService> =
            Arc::new(MCPApplicationServiceImpl::new(
                mcp_tool_repository,
                mcp_version_repository,
                mcp_domain_service,
                mcp_proxy_service,
            ));

        let session_service = Arc::new(SessionApplicationService::new(
            session_repository,
            message_repository,
            session_domain_service,
        ));

        let audit_service = Arc::new(AuditApplicationService::new(
            audit_domain_service,
        ));

        let execution_history_service = Arc::new(ExecutionHistoryServiceImpl::new(
            execution_history_repository,
        ));

        let execution_history_application_service = Arc::new(ExecutionHistoryApplicationService::new(
            execution_history_service,
        ));

        // Configure CORS
        let cors = self.create_cors_layer();

        // Create application router with all routes
        let app = Router::new()
            // Auth routes (includes /api/health and /api/auth/*)
            .merge(create_app_router(auth_service.clone()))
            // API routes
            .nest("/api", Router::new()
                // Flow management routes
                .merge(flow_routes(flow_service))
                // Configuration routes
                .merge(llm_config_routes(llm_service))
                .merge(vector_config_routes(vector_service))
                // Session and audit routes
                .merge(session_routes(session_service))
                .merge(audit_routes(audit_service))
                .merge(execution_history_routes(execution_history_application_service))
                // MCP tool routes
                .merge(create_mcp_api_routes(mcp_service))
                .route_layer(middleware::from_fn_with_state(
                    auth_service.clone(),
                    auth_middleware,
                ))
            );

        // Apply CORS
        app.layer(cors)
    }

    fn create_cors_layer(&self) -> CorsLayer {
        // Build list of allowed origins
        let mut origins = Vec::new();

        // Add localhost origins if enabled
        if self.config.cors.allow_all_localhost {
            let localhost_ports = vec![3000, 3001, 5173, 8080, 8081];
            for port in localhost_ports {
                origins.push(format!("http://localhost:{}", port));
                origins.push(format!("http://127.0.0.1:{}", port));
            }
        }

        // Add custom allowed origins from config
        for origin in &self.config.cors.allowed_origins {
            origins.push(origin.clone());
        }

        // If no origins configured, allow localhost:3000 as default
        if origins.is_empty() {
            origins.push("http://localhost:3000".to_string());
        }

        // Convert strings to HeaderValues for tower-http
        let origin_headers: Vec<_> = origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(origin_headers)
            .allow_methods(Any)
            .allow_headers(Any)
        // Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` with `Access-Control-Allow-Headers: *`
        // .allow_credentials(true)
    }
}
