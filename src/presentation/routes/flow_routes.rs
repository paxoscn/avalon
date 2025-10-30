use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::{
    application::services::FlowApplicationService,
    presentation::handlers::flow_handlers,
};

pub fn flow_routes(service: Arc<dyn FlowApplicationService>) -> Router {
    Router::new()
        // Flow CRUD
        .route("/flows", post(flow_handlers::create_flow))
        .route("/flows", get(flow_handlers::list_flows))
        .route("/flows/{flow_id}", get(flow_handlers::get_flow))
        .route("/flows/{flow_id}", put(flow_handlers::update_flow))
        .route("/flows/{flow_id}", delete(flow_handlers::delete_flow))
        
        // Flow status management
        .route("/flows/{flow_id}/activate", post(flow_handlers::activate_flow))
        .route("/flows/{flow_id}/archive", post(flow_handlers::archive_flow))
        
        // DSL import and validation
        .route("/flows/import-dsl", post(flow_handlers::import_from_dsl))
        .route("/flows/validate-definition", post(flow_handlers::validate_definition))
        
        // Flow execution
        .route("/flows/{flow_id}/execute", post(flow_handlers::execute_flow))
        .route("/flows/{flow_id}/executions/{execution_id}", get(flow_handlers::get_execution_status))
        .route("/flows/{flow_id}/executions", get(flow_handlers::list_executions))
        
        // Version management
        .route("/flows/{flow_id}/versions", post(flow_handlers::create_version))
        .route("/flows/{flow_id}/versions", get(flow_handlers::get_versions))
        .route("/flows/{flow_id}/rollback", post(flow_handlers::rollback_to_version))

        .route("/flow-executions/{execution_id}", get(flow_handlers::get_execution_status))
        
        .with_state(service)
}
