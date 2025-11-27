use crate::{
    application::services::*,
    config::AppConfig,
    domain::{repositories::FileRepository, services::*},
    error::Result,
    infrastructure::{
        llm::LLMProviderRegistry, mcp::MCPProxyServiceImpl, repositories::*,
        vector::VectorStoreRegistry, Database, RedisCache,
    },
    presentation::{
        middleware::auth_middleware,
        routes::{
            agent_routes, api_key_routes, audit_routes, create_app_router, create_mcp_api_routes,
            create_mcp_server_api_routes, dashboard_routes,
            execution_history_routes, file_routes, flow_routes, llm_config_routes, session_routes,
            vector_config_routes,
        },
        handlers::Counter,
    },
};
use axum::{middleware, Router};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::fs::ServeDir,
};
use rmcp::transport::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;

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
        log::info!("Starting server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
            crate::error::PlatformError::InternalError(format!("Failed to bind to {}: {}", addr, e))
        })?;

        axum::serve(listener, app).await.map_err(|e| {
            crate::error::PlatformError::InternalError(format!("Server error: {}", e))
        })?;

        Ok(())
    }

    pub fn create_app(&self) -> Router {
        // Create repositories
        let user_repository = Arc::new(UserRepositoryImpl::new(self.database.connection()));
        let tenant_repository = Arc::new(TenantRepositoryImpl::new(self.database.connection()));
        let flow_repository = Arc::new(FlowRepositoryImpl::new(self.database.connection()));
        let flow_version_repository =
            Arc::new(FlowVersionRepositoryImpl::new(self.database.connection()));
        let flow_execution_repository =
            Arc::new(FlowExecutionRepositoryImpl::new(self.database.connection()));
        let llm_config_repository =
            Arc::new(LLMConfigRepositoryImpl::new(self.database.connection()));
        let vector_config_repository =
            Arc::new(VectorConfigRepositoryImpl::new(self.database.connection()));
        let mcp_tool_repository = Arc::new(MCPToolRepositoryImpl::new(self.database.connection()));
        let mcp_version_repository = Arc::new(MCPToolVersionRepositoryImpl::new(
            self.database.connection(),
        ));
        let session_repository =
            Arc::new(ChatSessionRepositoryImpl::new(self.database.connection()));
        let message_repository = Arc::new(MessageRepositoryImpl::new(self.database.connection()));
        let audit_repository = Arc::new(AuditLogRepositoryImpl::new(self.database.connection()));
        let execution_history_repository = Arc::new(ExecutionHistoryRepositoryImpl::new(
            self.database.connection(),
        ));
        let agent_repository = Arc::new(AgentRepositoryImpl::new(self.database.connection()));
        let agent_allocation_repository = Arc::new(AgentAllocationRepositoryImpl::new(
            self.database.connection(),
        ));
        let api_key_repository = Arc::new(APIKeyRepositoryImpl::new(self.database.connection()));

        let vector_store_registry = Arc::new(VectorStoreRegistry::new());
        let llm_provider_registry = Arc::new(LLMProviderRegistry::new());
        let mcp_proxy_service = Arc::new(MCPProxyServiceImpl::new());

        // Create domain services
        let auth_domain_service = Arc::new(AuthenticationDomainServiceImpl::new(
            self.config.jwt_secret.clone(),
            Some(self.config.bcrypt_cost),
        ));

        let flow_domain_service: Arc<dyn FlowDomainService> =
            Arc::new(FlowDomainServiceImpl::new());
        let llm_domain_service: Arc<dyn LLMDomainService> =
            Arc::new(LLMDomainServiceImpl::new(llm_provider_registry.clone()));
        let vector_store_domain_service: Arc<dyn VectorStoreDomainService> =
            Arc::new(VectorStoreDomainServiceImpl::new());
        let mcp_domain_service: Arc<dyn MCPToolDomainService> =
            Arc::new(MCPToolDomainServiceImpl::new());
        let session_domain_service: Arc<SessionDomainService> =
            Arc::new(SessionDomainService::new(30));
        let audit_domain_service: Arc<dyn AuditService> =
            Arc::new(AuditServiceImpl::new(audit_repository));
        let api_key_domain_service: Arc<dyn APIKeyService> =
            Arc::new(APIKeyDomainService::new(api_key_repository.clone()));

        let execution_engine = ExecutionEngineFactory::create_with_services(
            llm_domain_service.clone(),
            llm_config_repository.clone(),
            vector_store_domain_service.clone(),
            mcp_domain_service.clone(),
            mcp_tool_repository.clone(),
        );

        // Create application services
        let auth_service: Arc<dyn AuthApplicationService> =
            Arc::new(AuthApplicationServiceImpl::new(
                user_repository.clone(),
                tenant_repository,
                auth_domain_service,
                None, // Use default token expiry
            ));

        let flow_service: Arc<dyn FlowApplicationService> =
            Arc::new(FlowApplicationServiceImpl::new(
                flow_repository.clone(),
                flow_version_repository,
                flow_execution_repository,
                flow_domain_service,
                Some(execution_engine),
            ));

        let llm_service: Arc<dyn LLMApplicationService> = Arc::new(LLMApplicationServiceImpl::new(
            llm_config_repository.clone(),
            llm_domain_service.clone(),
            llm_provider_registry,
        ));

        let vector_service = Arc::new(VectorApplicationService::new(
            vector_config_repository.clone(),
        ));

        // let vector_storage_service = Arc::new(VectorStorageApplicationService::new(
        //     vector_service,
        //     vector_store_registry,
        // ));

        let mcp_service: Arc<dyn MCPApplicationService> = Arc::new(MCPApplicationServiceImpl::new(
            mcp_tool_repository.clone(),
            mcp_version_repository,
            mcp_domain_service,
            mcp_proxy_service,
        ));

        let mcp_proxy_service = Arc::new(MCPProxyServiceImpl::new());

        let mcp_server_service: Arc<dyn MCPServerApplicationService> = Arc::new(MCPServerApplicationServiceImpl::new(
            mcp_tool_repository.clone(),
            mcp_proxy_service,
        ));

        let streamable_http_service = StreamableHttpService::new(
            || Ok(Counter::new()),
            LocalSessionManager::default().into(),
            Default::default(),
        );

        let session_service = Arc::new(SessionApplicationService::new(
            session_repository.clone(),
            message_repository.clone(),
            session_domain_service.clone(),
        ));

        let audit_service = Arc::new(AuditApplicationService::new(audit_domain_service));

        let api_key_service = Arc::new(APIKeyApplicationService::new(
            api_key_domain_service,
            api_key_repository,
            audit_service.clone(),
        ));

        let execution_history_service = Arc::new(ExecutionHistoryServiceImpl::new(
            execution_history_repository,
        ));

        let execution_history_application_service = Arc::new(
            ExecutionHistoryApplicationService::new(execution_history_service),
        );

        // Create agent daily stats repository and service
        let agent_daily_stats_repository: Arc<dyn crate::domain::repositories::AgentDailyStatsRepository> = 
            Arc::new(AgentDailyStatsRepositoryImpl::new(self.database.connection()));
        let agent_stats_service = Arc::new(AgentStatsService::new(agent_daily_stats_repository));

        // Create interview record repository
        let interview_record_repository: Arc<dyn crate::domain::repositories::InterviewRecordRepository> = 
            Arc::new(crate::infrastructure::repositories::InterviewRecordRepositoryImpl::new(self.database.connection()));

        let agent_service: Arc<dyn AgentApplicationService> =
            Arc::new(AgentApplicationServiceImpl::new(
                agent_repository.clone(),
                agent_allocation_repository.clone(),
                vector_config_repository.clone(),
                mcp_tool_repository.clone(),
                flow_repository.clone(),
                user_repository.clone(),
                interview_record_repository.clone(),
            )
            .with_session_service(session_service.clone())
            .with_llm_service(llm_domain_service.clone())
            .with_llm_config_repo(llm_config_repository.clone())
            .with_db(self.database.connection())
            .with_stats_service(agent_stats_service));

        // Create file repository and service (using OSS)
        let file_repository: Arc<dyn FileRepository> = Arc::new(
            OssFileRepositoryImpl::new(self.config.oss.clone())
                .expect("Failed to initialize OSS client")
        );
        let file_service: Arc<dyn FileApplicationService> =
            Arc::new(FileApplicationServiceImpl::new(file_repository));

        // Create dashboard service
        let dashboard_service: Arc<dyn DashboardApplicationService> =
            Arc::new(DashboardApplicationServiceImpl::new(
                agent_repository.clone(),
                flow_repository.clone(),
                mcp_tool_repository.clone(),
                vector_config_repository.clone(),
                session_repository.clone(),
            ));

        // Configure CORS
        let cors = self.create_cors_layer();

        // Create application router with all routes
        let app = Router::new()
            // Auth routes (includes /api/health and /api/auth/*)
            .merge(create_app_router(auth_service.clone()))
            // API routes
            .nest(
                "/api",
                Router::new()
                    // Agent management routes
                    .merge(agent_routes(agent_service))
                    // Flow management routes
                    .merge(flow_routes(flow_service))
                    // Configuration routes
                    .merge(llm_config_routes(llm_service))
                    .merge(vector_config_routes(vector_service))
                    // Session and audit routes
                    .merge(session_routes(session_service))
                    .merge(audit_routes(audit_service))
                    .merge(execution_history_routes(
                        execution_history_application_service,
                    ))
                    // MCP tool routes
                    .merge(create_mcp_api_routes(mcp_service))
                    // File upload routes
                    .merge(file_routes(file_service))
                    // API key management routes
                    .merge(api_key_routes(api_key_service))
                    // Dashboard statistics routes
                    .merge(dashboard_routes(dashboard_service))
                    .route_layer(middleware::from_fn_with_state(
                        auth_service.clone(),
                        auth_middleware,
                    ))
                    // MCP server routes
                    .merge(create_mcp_server_api_routes(streamable_http_service)),
            )
            // Serve uploaded files (no auth required for downloads)
            .nest_service("/files", ServeDir::new("/tmp/uploads"));

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
