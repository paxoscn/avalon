use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    application::services::DashboardApplicationService,
    error::Result,
    presentation::extractors::AuthenticatedUser,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStatsResponse {
    pub agents_count: u64,
    pub flows_count: u64,
    pub mcp_tools_count: u64,
    pub knowledge_bases_count: u64,
    pub sessions_count: u64,
}

/// Get dashboard statistics
pub async fn get_dashboard_stats(
    State(service): State<Arc<dyn DashboardApplicationService>>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    let stats = service
        .get_dashboard_stats(user.tenant_id, user.user_id)
        .await?;

    let response = DashboardStatsResponse {
        agents_count: stats.agents_count,
        flows_count: stats.flows_count,
        mcp_tools_count: stats.mcp_tools_count,
        knowledge_bases_count: stats.knowledge_bases_count,
        sessions_count: stats.sessions_count,
    };

    Ok(Json(response))
}
