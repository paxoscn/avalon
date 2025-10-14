use crate::{
    application::services::{AuthApplicationService, AuthApplicationServiceImpl},
    config::AppConfig,
    domain::services::AuthenticationDomainServiceImpl,
    error::Result,
    infrastructure::repositories::{TenantRepositoryImpl, UserRepositoryImpl},
    infrastructure::{Database, RedisCache},
    presentation::routes::create_app_router,
};
use axum::Router;
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

        // Create domain services
        let auth_domain_service = Arc::new(AuthenticationDomainServiceImpl::new(
            self.config.jwt_secret.clone(),
            Some(self.config.bcrypt_cost),
        ));

        // Create application services
        let auth_service: Arc<dyn AuthApplicationService> =
            Arc::new(AuthApplicationServiceImpl::new(
                user_repository,
                tenant_repository,
                auth_domain_service,
                None, // Use default token expiry
            ));

        // Configure CORS
        let cors = self.create_cors_layer();

        // Create application router with authentication and CORS
        create_app_router(auth_service).layer(cors)
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
