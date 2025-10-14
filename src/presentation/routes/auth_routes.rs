use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::{
    application::services::AuthApplicationService,
    presentation::{
        handlers::{
            login_handler, refresh_token_handler, logout_handler,
            change_password_handler, me_handler, health_handler,
        },
        middleware::auth_middleware,
        routes::*
    },
};

/// Create authentication routes
pub fn create_auth_routes(auth_service: Arc<dyn AuthApplicationService>) -> Router {
    // Create protected routes that require authentication
    let protected_routes = Router::new()
        .route("/auth/me", get(me_handler))
        .route("/auth/change-password", post(change_password_handler))
        .route_layer(middleware::from_fn_with_state(
            auth_service.clone(),
            auth_middleware,
        ));

    // Combine public and protected routes
    Router::new()
        // Public routes (no authentication required)
        .route("/health", get(health_handler))
        .route("/auth/login", post(login_handler))
        .route("/auth/refresh", post(refresh_token_handler))
        .route("/auth/logout", post(logout_handler))
        // Merge protected routes
        .merge(protected_routes)
        // Add auth service to state for all routes
        .with_state(auth_service)
}

/// Create a router with all application routes
pub fn create_app_router(auth_service: Arc<dyn AuthApplicationService>) -> Router {
    Router::new()
        .nest("/api", create_auth_routes(auth_service))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::services::MockAuthApplicationService;

    #[test]
    fn test_create_auth_routes() {
        let auth_service = Arc::new(MockAuthApplicationService::new());
        let _router = create_auth_routes(auth_service);
        // Just test that the router can be created without panicking
        assert!(true);
    }

    #[test]
    fn test_create_app_router() {
        let auth_service = Arc::new(MockAuthApplicationService::new());
        let _router = create_app_router(auth_service);
        // Just test that the router can be created without panicking
        assert!(true);
    }
}