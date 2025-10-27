use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

use crate::application::services::VectorStorageApplicationService;
use crate::presentation::handlers::vector_storage_handlers::*;

/// Create vector storage routes
pub fn create_vector_storage_routes() -> Router<Arc<VectorStorageApplicationService>> {
    Router::new()
        // Vector operations
        .route("/vectors", post(upsert_vector))
        .route("/vectors/batch", post(upsert_vectors_batch))
        .route("/vectors/search", post(search_vectors))
        .route("/vectors", delete(delete_vectors))
        .route("/vectors/batch-operation", post(execute_batch_operation))
        
        // Statistics and information
        .route("/stats", get(get_storage_stats))
        .route("/stats/{namespace}", get(get_namespace_stats))
        
        // Namespace management
        .route("/namespaces", get(list_namespaces))
        
        // Multi-store operations
        .route("/search/multi-store", post(multi_store_search))
        .route("/stores", get(get_available_stores))
        .route("/stores/test", get(test_all_connections))
}