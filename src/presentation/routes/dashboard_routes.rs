use axum::{routing::get, Router};
use std::sync::Arc;

use crate::{
    application::services::DashboardApplicationService,
    presentation::handlers::dashboard_handlers,
};

/// Create dashboard routes
pub fn dashboard_routes(service: Arc<dyn DashboardApplicationService>) -> Router {
    Router::new()
        .route("/dashboard/stats", get(dashboard_handlers::get_dashboard_stats))
        .with_state(service)
}
