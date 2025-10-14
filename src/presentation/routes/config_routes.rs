use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::{
    application::services::{LLMApplicationService, VectorApplicationService},
    presentation::handlers::config_handlers,
};

pub fn llm_config_routes(service: Arc<dyn LLMApplicationService>) -> Router {
    Router::new()
        .route("/config/llm", post(config_handlers::create_llm_config))
        .route("/config/llm", get(config_handlers::list_llm_configs))
        .route("/config/llm/{config_id}", get(config_handlers::get_llm_config))
        .route("/config/llm/{config_id}", put(config_handlers::update_llm_config))
        .route("/config/llm/{config_id}", delete(config_handlers::delete_llm_config))
        .route("/config/llm/{config_id}/set-default", post(config_handlers::set_default_llm_config))
        .route("/config/llm/{config_id}/test", post(config_handlers::test_llm_connection))
        .route("/llm-providers/{provider}/models", get(config_handlers::get_available_models))
        .with_state(service)
}

pub fn vector_config_routes(service: Arc<VectorApplicationService>) -> Router {
    Router::new()
        .route("/config/vector", post(config_handlers::create_vector_config))
        .route("/config/vector", get(config_handlers::list_vector_configs))
        .route("/config/vector/{config_id}", get(config_handlers::get_vector_config))
        .route("/config/vector/{config_id}", put(config_handlers::update_vector_config))
        .route("/config/vector/{config_id}", delete(config_handlers::delete_vector_config))
        .route("/config/vector/{config_id}/set-default", post(config_handlers::set_default_vector_config))
        .route("/config/vector/{config_id}/test", post(config_handlers::test_vector_connection))
        .route("/vector-providers/{provider}/params", get(config_handlers::get_vector_provider_params))
        .route("/config/vector/health", get(config_handlers::get_vector_health_status))
        .with_state(service)
}
