use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::{
    application::services::{SessionApplicationService, AuditApplicationService, ExecutionHistoryApplicationService},
    presentation::handlers::session_audit_handlers,
};

pub fn session_routes(service: Arc<SessionApplicationService>) -> Router {
    Router::new()
        .route("/sessions", post(session_audit_handlers::create_session))
        .route("/sessions", get(session_audit_handlers::list_sessions))
        .route("/sessions/{session_id}", get(session_audit_handlers::get_session))
        .route("/sessions/{session_id}", put(session_audit_handlers::update_session))
        .route("/sessions/{session_id}", delete(session_audit_handlers::delete_session))
        .route("/sessions/{session_id}/messages", get(session_audit_handlers::get_session_messages))
        .route("/sessions/{session_id}/messages", post(session_audit_handlers::add_message))
        .route("/sessions/{session_id}/context", post(session_audit_handlers::set_context))
        .route("/sessions/{session_id}/context/{key}", get(session_audit_handlers::get_context))
        .with_state(service)
}

pub fn audit_routes(service: Arc<AuditApplicationService>) -> Router {
    Router::new()
        .route("/audit/logs", get(session_audit_handlers::query_audit_logs))
        .route("/audit/statistics", get(session_audit_handlers::get_audit_statistics))
        .with_state(service)
}

pub fn execution_history_routes(service: Arc<ExecutionHistoryApplicationService>) -> Router {
    Router::new()
        .route("/execution-history", get(session_audit_handlers::query_executions))
        .route("/execution-history/{execution_id}", get(session_audit_handlers::get_execution_details))
        .with_state(service)
}
