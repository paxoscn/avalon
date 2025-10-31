use axum::{routing::post, Router};
use std::sync::Arc;

use crate::{
    application::services::FileApplicationService,
    presentation::handlers::file_handlers,
};

pub fn file_routes(service: Arc<dyn FileApplicationService>) -> Router {
    Router::new()
        .route("/files/upload", post(file_handlers::upload_file))
        .with_state(service)
}
