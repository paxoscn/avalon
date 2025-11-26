use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use crate::{
    application::services::APIKeyApplicationService,
    presentation::handlers::api_key_handlers,
};

/// Create API key management routes
pub fn api_key_routes(service: Arc<APIKeyApplicationService>) -> Router {
    Router::new()
        // CRUD operations
        .route("/api-keys", post(api_key_handlers::create_api_key))
        .route("/api-keys", get(api_key_handlers::list_api_keys))
        .route("/api-keys/{id}", get(api_key_handlers::get_api_key))
        .route("/api-keys/{id}", patch(api_key_handlers::update_api_key))
        .route("/api-keys/{id}", delete(api_key_handlers::delete_api_key))
        .with_state(service)
}
