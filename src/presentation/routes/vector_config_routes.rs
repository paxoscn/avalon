use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::application::services::VectorApplicationService;
use crate::presentation::handlers::vector_config_handlers::*;

/// Create vector configuration routes
pub fn create_vector_config_routes() -> Router<Arc<VectorApplicationService>> {
    Router::new()
        // Configuration CRUD operations
        .route("/configs", post(create_vector_config))
        .route("/configs", get(list_vector_configs))
        .route("/configs/{id}", get(get_vector_config))
        .route("/configs/{id}", put(update_vector_config))
        .route("/configs/{id}", delete(delete_vector_config))
        
        // Default configuration management
        .route("/configs/default", get(get_default_vector_config))
        .route("/configs/default", post(set_default_vector_config))
        
        // Connection testing and health
        .route("/configs/{id}/test", post(test_vector_config_connection))
        .route("/configs/health", get(get_vector_configs_health))
        
        // Provider information and validation
        .route("/providers/{provider}/params", get(get_provider_params))
        .route("/providers/{provider}/validate", post(validate_provider_params))
}