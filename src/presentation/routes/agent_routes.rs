use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::{
    application::services::AgentApplicationService,
    presentation::handlers::agent_handlers,
};

/// Create agent management routes
pub fn agent_routes(service: Arc<dyn AgentApplicationService>) -> Router {
    Router::new()
        // Basic CRUD operations
        .route("/agents", post(agent_handlers::create_agent))
        .route("/agents", get(agent_handlers::list_agents))
        .route("/agents/{agent_id}", get(agent_handlers::get_agent))
        .route("/agents/{agent_id}", put(agent_handlers::update_agent))
        .route("/agents/{agent_id}", delete(agent_handlers::delete_agent))
        
        // Copy operation
        .route("/agents/{agent_id}/copy", post(agent_handlers::copy_agent))
        
        // Employment management
        .route("/agents/{agent_id}/employ", post(agent_handlers::employ_agent))
        .route("/agents/{agent_id}/employ", delete(agent_handlers::terminate_employment))
        .route("/agents/employed", get(agent_handlers::list_employed_agents))
        
        // Allocation management
        .route("/agents/{agent_id}/allocate", post(agent_handlers::allocate_agent))
        .route("/agents/{agent_id}/allocate", delete(agent_handlers::terminate_allocation))
        .route("/agents/allocated", get(agent_handlers::list_allocated_agents))

        // Created
        .route("/agents/created", get(agent_handlers::list_created_agents))
        
        // Resource management - Knowledge Base
        .route(
            "/agents/{agent_id}/knowledge-bases/{config_id}",
            post(agent_handlers::add_knowledge_base),
        )
        .route(
            "/agents/{agent_id}/knowledge-bases/{config_id}",
            delete(agent_handlers::remove_knowledge_base),
        )
        
        // Resource management - MCP Tool
        .route(
            "/agents/{agent_id}/mcp-tools/{tool_id}",
            post(agent_handlers::add_mcp_tool),
        )
        .route(
            "/agents/{agent_id}/mcp-tools/{tool_id}",
            delete(agent_handlers::remove_mcp_tool),
        )
        
        // Resource management - Flow
        .route(
            "/agents/{agent_id}/flows/{flow_id}",
            post(agent_handlers::add_flow),
        )
        .route(
            "/agents/{agent_id}/flows/{flow_id}",
            delete(agent_handlers::remove_flow),
        )
        
        .with_state(service)
}
