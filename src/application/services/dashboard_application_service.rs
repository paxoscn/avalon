use std::sync::Arc;
use async_trait::async_trait;

use crate::{
    domain::{
        repositories::{
            AgentRepository, FlowRepository, MCPToolRepository,
            ChatSessionRepository, VectorConfigRepository,
        },
        value_objects::{TenantId, UserId},
    },
    error::Result,
};

#[derive(Debug, Clone)]
pub struct DashboardStats {
    pub agents_count: u64,
    pub flows_count: u64,
    pub mcp_tools_count: u64,
    pub knowledge_bases_count: u64,
    pub sessions_count: u64,
}

#[async_trait]
pub trait DashboardApplicationService: Send + Sync {
    /// Get dashboard statistics for a user
    async fn get_dashboard_stats(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<DashboardStats>;
}

pub struct DashboardApplicationServiceImpl {
    agent_repo: Arc<dyn AgentRepository>,
    flow_repo: Arc<dyn FlowRepository>,
    mcp_repo: Arc<dyn MCPToolRepository>,
    vector_repo: Arc<dyn VectorConfigRepository>,
    session_repo: Arc<dyn ChatSessionRepository>,
}

impl DashboardApplicationServiceImpl {
    pub fn new(
        agent_repo: Arc<dyn AgentRepository>,
        flow_repo: Arc<dyn FlowRepository>,
        mcp_repo: Arc<dyn MCPToolRepository>,
        vector_repo: Arc<dyn VectorConfigRepository>,
        session_repo: Arc<dyn ChatSessionRepository>,
    ) -> Self {
        Self {
            agent_repo,
            flow_repo,
            mcp_repo,
            vector_repo,
            session_repo,
        }
    }
}

#[async_trait]
impl DashboardApplicationService for DashboardApplicationServiceImpl {
    async fn get_dashboard_stats(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<DashboardStats> {
        // Fetch all counts in parallel for better performance
        let (agents_count, flows_count, mcp_tools_count, knowledge_bases_count, sessions_count) = tokio::join!(
            self.agent_repo.count_by_tenant_active(&tenant_id),
            self.flow_repo.count_by_tenant(&tenant_id),
            self.mcp_repo.count_by_tenant(tenant_id),
            self.vector_repo.count_by_tenant(tenant_id),
            self.session_repo.count_by_user(&user_id),
        );

        Ok(DashboardStats {
            agents_count: agents_count?,
            flows_count: flows_count?,
            mcp_tools_count: mcp_tools_count?,
            knowledge_bases_count: knowledge_bases_count?,
            sessions_count: sessions_count?,
        })
    }
}
