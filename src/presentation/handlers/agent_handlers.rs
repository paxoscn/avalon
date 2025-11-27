use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::{
        dto::agent_dto::*,
        services::AgentApplicationService,
    },
    domain::value_objects::{AgentId, ConfigId, MCPToolId, FlowId},
    error::Result,
    presentation::extractors::AuthenticatedUser,
};

use crate::application::dto::agent_dto::{AgentChatRequest, CompleteInterviewRequest};

// ============================================================================
// CRUD Handlers
// ============================================================================

/// Create a new agent
pub async fn create_agent(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Json(dto): Json<CreateAgentDto>,
) -> Result<impl IntoResponse> {
    let agent = service.create_agent(dto, user.tenant_id, user.user_id).await?;
    Ok((StatusCode::CREATED, Json(agent)))
}

/// Get agent by ID
pub async fn get_agent(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let agent = service.get_agent(AgentId::from_uuid(agent_id), user.user_id).await?;
    Ok(Json(agent))
}

/// Update agent
pub async fn update_agent(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
    Json(dto): Json<UpdateAgentDto>,
) -> Result<impl IntoResponse> {
    let agent = service.update_agent(AgentId::from_uuid(agent_id), dto, user.user_id).await?;
    Ok(Json(agent))
}

/// Delete agent
pub async fn delete_agent(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service.delete_agent(AgentId::from_uuid(agent_id), user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// List agents with pagination
pub async fn list_agents(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<AgentListQuery>,
) -> Result<impl IntoResponse> {
    let params = PaginationParams {
        page: query.page,
        limit: query.limit,
    };

    let include_fired = query.include_fired.unwrap_or(false);
    let response = service.list_agents(user.tenant_id, user.user_id, params, include_fired).await?;
    Ok(Json(response))
}

/// List agents created by the user
pub async fn list_created_agents(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<AgentListQuery>,
) -> Result<impl IntoResponse> {
    let params = PaginationParams {
        page: query.page,
        limit: query.limit,
    };

    let response = service.list_created_agents(user.user_id, params).await?;
    Ok(Json(response))
}

// ============================================================================
// Copy Handler
// ============================================================================

/// Copy an existing agent
pub async fn copy_agent(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let agent = service.copy_agent(
        AgentId::from_uuid(agent_id),
        user.user_id,
        user.tenant_id,
    ).await?;
    Ok((StatusCode::CREATED, Json(agent)))
}

// ============================================================================
// Employment Management Handlers
// ============================================================================

/// Employ an agent
pub async fn employ_agent(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let agent = service.employ_agent(AgentId::from_uuid(agent_id), user.user_id).await?;
    Ok((StatusCode::CREATED, Json(agent)))
}

/// Fire an agent
pub async fn fire_agent(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service.fire_agent(AgentId::from_uuid(agent_id), user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// List employed agents
pub async fn list_employed_agents(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<AgentListQuery>,
) -> Result<impl IntoResponse> {
    let params = PaginationParams {
        page: query.page,
        limit: query.limit,
    };

    let include_fired = query.include_fired.unwrap_or(false);
    let response = service.list_employed_agents(user.user_id, params, include_fired).await?;
    Ok(Json(response))
}

// ============================================================================
// Allocation Management Handlers
// ============================================================================

/// Allocate an agent
pub async fn allocate_agent(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service.allocate_agent(AgentId::from_uuid(agent_id), user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Terminate allocation with an agent
pub async fn terminate_allocation(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service.terminate_allocation(AgentId::from_uuid(agent_id), user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// List allocated agents
pub async fn list_allocated_agents(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Query(query): Query<AgentListQuery>,
) -> Result<impl IntoResponse> {
    let params = PaginationParams {
        page: query.page,
        limit: query.limit,
    };

    let response = service.list_allocated_agents(user.user_id, params).await?;
    Ok(Json(response))
}

// ============================================================================
// Resource Management Handlers - Knowledge Base
// ============================================================================

/// Add knowledge base to agent
pub async fn add_knowledge_base(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path((agent_id, config_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    service.add_knowledge_base(
        AgentId::from_uuid(agent_id),
        ConfigId::from_uuid(config_id),
        user.user_id,
    ).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Remove knowledge base from agent
pub async fn remove_knowledge_base(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path((agent_id, config_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    service.remove_knowledge_base(
        AgentId::from_uuid(agent_id),
        ConfigId::from_uuid(config_id),
        user.user_id,
    ).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Resource Management Handlers - MCP Tool
// ============================================================================

/// Add MCP tool to agent
pub async fn add_mcp_tool(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path((agent_id, tool_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    service.add_mcp_tool(
        AgentId::from_uuid(agent_id),
        MCPToolId::from_uuid(tool_id),
        user.user_id,
    ).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Remove MCP tool from agent
pub async fn remove_mcp_tool(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path((agent_id, tool_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    service.remove_mcp_tool(
        AgentId::from_uuid(agent_id),
        MCPToolId::from_uuid(tool_id),
        user.user_id,
    ).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Resource Management Handlers - Flow
// ============================================================================

/// Add flow to agent
pub async fn add_flow(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path((agent_id, flow_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    service.add_flow(
        AgentId::from_uuid(agent_id),
        FlowId::from_uuid(flow_id),
        user.user_id,
    ).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Remove flow from agent
pub async fn remove_flow(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path((agent_id, flow_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    service.remove_flow(
        AgentId::from_uuid(agent_id),
        FlowId::from_uuid(flow_id),
        user.user_id,
    ).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Chat Handler
// ============================================================================

/// Chat with an agent
pub async fn chat_with_agent(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<AgentChatRequest>,
) -> Result<impl IntoResponse> {
    let session_id = req.session_id.map(crate::domain::value_objects::SessionId);
    
    let response = service.chat(
        AgentId::from_uuid(agent_id),
        req.message,
        session_id,
        user.user_id,
        user.tenant_id,
    ).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// Chat with an agent (SSE streaming)
pub async fn chat_with_agent_stream(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<AgentChatRequest>,
) -> Result<impl IntoResponse> {
    use axum::response::sse::{Event, Sse};
    use futures::stream::StreamExt;

    let session_id = req.session_id.map(crate::domain::value_objects::SessionId);
    
    let stream = service.chat_stream(
        AgentId::from_uuid(agent_id),
        req.message,
        session_id,
        user.user_id,
        user.tenant_id,
    ).await?;

    // Transform the stream into SSE events
    let sse_stream = stream.map(|chunk_result| {
        let event = match chunk_result {
            Ok(chunk) => {
                let json = serde_json::to_string(&chunk).unwrap_or_else(|_| "{}".to_string());
                Event::default().data(json)
            }
            Err(e) => {
                let error_chunk = crate::application::dto::agent_dto::AgentChatStreamChunk {
                    chunk_type: "error".to_string(),
                    content: None,
                    reasoning_content: None,
                    session_id: None,
                    message_id: None,
                    reply_id: None,
                    metadata: None,
                    finish_reason: None,
                    error: Some(format!("{}", e)),
                };
                let json = serde_json::to_string(&error_chunk).unwrap_or_else(|_| "{}".to_string());
                Event::default().data(json)
            }
        };
        Ok::<_, std::convert::Infallible>(event)
    });

    Ok(Sse::new(sse_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive")
    ))
}

// ============================================================================
// Statistics Handler
// ============================================================================

/// Get agent usage statistics
pub async fn get_agent_usage_stats(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
    Query(query): Query<AgentUsageStatsQuery>,
) -> Result<impl IntoResponse> {
    let response = service.get_agent_usage_stats(
        AgentId::from_uuid(agent_id),
        query,
        user.user_id,
    ).await?;

    Ok(Json(response))
}

// ============================================================================
// Interview Handlers
// ============================================================================

/// Start an interview with an agent
pub async fn start_interview(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    service.start_interview(
        AgentId::from_uuid(agent_id),
        user.user_id,
        user.tenant_id,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Complete an interview (pass or fail)
pub async fn complete_interview(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<CompleteInterviewRequest>,
) -> Result<impl IntoResponse> {
    service.complete_interview(
        AgentId::from_uuid(agent_id),
        user.user_id,
        user.tenant_id,
        req.passed,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}
