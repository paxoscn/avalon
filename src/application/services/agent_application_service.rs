use async_trait::async_trait;
use std::sync::Arc;
use sea_orm::PaginatorTrait;

use crate::{
    application::dto::agent_dto::*,
    domain::{
        entities::Agent,
        repositories::{
            AgentAllocationRepository, AgentRepository, FlowRepository, MCPToolRepository,
            UserRepository, VectorConfigRepository,
        },
        value_objects::{AgentId, ConfigId, FlowId, MCPToolId, TenantId, UserId},
    },
    error::{PlatformError, Result},
};

/// Agent application service trait
#[async_trait]
pub trait AgentApplicationService: Send + Sync {
    /// Create a new agent
    async fn create_agent(
        &self,
        dto: CreateAgentDto,
        tenant_id: TenantId,
        creator_id: UserId,
    ) -> Result<AgentDto>;

    /// Get agent by ID
    async fn get_agent(&self, id: AgentId, user_id: UserId) -> Result<AgentDetailDto>;

    /// Update agent
    async fn update_agent(
        &self,
        id: AgentId,
        dto: UpdateAgentDto,
        user_id: UserId,
    ) -> Result<AgentDto>;

    /// Delete agent
    async fn delete_agent(&self, id: AgentId, user_id: UserId) -> Result<()>;

    /// List agents with pagination
    async fn list_agents(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        params: PaginationParams,
        include_fired: bool,
    ) -> Result<PaginatedResponse<AgentCardDto>>;

    /// List agents created by the user
    async fn list_created_agents(
        &self,
        user_id: UserId,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<AgentCardDto>>;

    /// Copy an agent
    async fn copy_agent(
        &self,
        source_id: AgentId,
        user_id: UserId,
        tenant_id: TenantId,
    ) -> Result<AgentDto>;

    /// Employ an agent (creates a copy with employer_id set)
    async fn employ_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<AgentDto>;

    /// Fire an agent (sets fired_at timestamp)
    async fn fire_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<()>;

    /// List employed agents
    async fn list_employed_agents(
        &self,
        user_id: UserId,
        params: PaginationParams,
        include_fired: bool,
    ) -> Result<PaginatedResponse<AgentCardDto>>;

    /// Allocate an agent
    async fn allocate_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<()>;

    /// Terminate allocation
    async fn terminate_allocation(&self, agent_id: AgentId, user_id: UserId) -> Result<()>;

    /// List allocated agents
    async fn list_allocated_agents(
        &self,
        user_id: UserId,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<AgentCardDto>>;

    /// Add knowledge base to agent
    async fn add_knowledge_base(
        &self,
        agent_id: AgentId,
        config_id: ConfigId,
        user_id: UserId,
    ) -> Result<()>;

    /// Remove knowledge base from agent
    async fn remove_knowledge_base(
        &self,
        agent_id: AgentId,
        config_id: ConfigId,
        user_id: UserId,
    ) -> Result<()>;

    /// Add MCP tool to agent
    async fn add_mcp_tool(
        &self,
        agent_id: AgentId,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<()>;

    /// Remove MCP tool from agent
    async fn remove_mcp_tool(
        &self,
        agent_id: AgentId,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<()>;

    /// Add flow to agent
    async fn add_flow(&self, agent_id: AgentId, flow_id: FlowId, user_id: UserId) -> Result<()>;

    /// Remove flow from agent
    async fn remove_flow(&self, agent_id: AgentId, flow_id: FlowId, user_id: UserId) -> Result<()>;

    /// Chat with an agent
    async fn chat(
        &self,
        agent_id: AgentId,
        message: String,
        session_id: Option<crate::domain::value_objects::SessionId>,
        user_id: UserId,
        tenant_id: TenantId,
    ) -> Result<crate::application::dto::agent_dto::AgentChatResponse>;

    /// Chat with an agent (streaming)
    async fn chat_stream(
        &self,
        agent_id: AgentId,
        message: String,
        session_id: Option<crate::domain::value_objects::SessionId>,
        user_id: UserId,
        tenant_id: TenantId,
    ) -> Result<Box<dyn futures::Stream<Item = Result<crate::application::dto::agent_dto::AgentChatStreamChunk>> + Send + Unpin>>;

    /// Get agent usage statistics
    async fn get_agent_usage_stats(
        &self,
        agent_id: AgentId,
        query: AgentUsageStatsQuery,
        user_id: UserId,
    ) -> Result<AgentUsageStatsResponse>;

    /// Start an interview with an agent
    async fn start_interview(&self, agent_id: AgentId, user_id: UserId, tenant_id: TenantId) -> Result<()>;

    /// Complete an interview (pass or fail)
    async fn complete_interview(&self, agent_id: AgentId, user_id: UserId, tenant_id: TenantId, passed: bool) -> Result<()>;
}

/// Agent application service implementation
pub struct AgentApplicationServiceImpl {
    agent_repo: Arc<dyn AgentRepository>,
    allocation_repo: Arc<dyn AgentAllocationRepository>,
    vector_config_repo: Arc<dyn VectorConfigRepository>,
    mcp_tool_repo: Arc<dyn MCPToolRepository>,
    flow_repo: Arc<dyn FlowRepository>,
    user_repo: Arc<dyn UserRepository>,
    session_service: Option<Arc<crate::application::services::SessionApplicationService>>,
    llm_service: Option<Arc<dyn crate::domain::services::llm_service::LLMDomainService>>,
    llm_config_repo: Option<Arc<dyn crate::domain::repositories::LLMConfigRepository>>,
    db: Option<Arc<sea_orm::DatabaseConnection>>,
    stats_service: Option<Arc<crate::domain::services::AgentStatsService>>,
}

impl AgentApplicationServiceImpl {
    pub fn new(
        agent_repo: Arc<dyn AgentRepository>,
        allocation_repo: Arc<dyn AgentAllocationRepository>,
        vector_config_repo: Arc<dyn VectorConfigRepository>,
        mcp_tool_repo: Arc<dyn MCPToolRepository>,
        flow_repo: Arc<dyn FlowRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            agent_repo,
            allocation_repo,
            vector_config_repo,
            mcp_tool_repo,
            flow_repo,
            user_repo,
            session_service: None,
            llm_service: None,
            llm_config_repo: None,
            db: None,
            stats_service: None,
        }
    }

    /// Set session service for chat functionality
    pub fn with_session_service(mut self, session_service: Arc<crate::application::services::SessionApplicationService>) -> Self {
        self.session_service = Some(session_service);
        self
    }

    /// Set LLM service for chat functionality
    pub fn with_llm_service(mut self, llm_service: Arc<dyn crate::domain::services::llm_service::LLMDomainService>) -> Self {
        self.llm_service = Some(llm_service);
        self
    }

    /// Set LLM config repository for chat functionality
    pub fn with_llm_config_repo(mut self, llm_config_repo: Arc<dyn crate::domain::repositories::LLMConfigRepository>) -> Self {
        self.llm_config_repo = Some(llm_config_repo);
        self
    }

    /// Set database connection for statistics queries
    pub fn with_db(mut self, db: Arc<sea_orm::DatabaseConnection>) -> Self {
        self.db = Some(db);
        self
    }

    /// Set stats service for usage tracking
    pub fn with_stats_service(mut self, stats_service: Arc<crate::domain::services::AgentStatsService>) -> Self {
        self.stats_service = Some(stats_service);
        self
    }

    /// Verify that the user can modify the agent (is the creator)
    async fn verify_can_modify(&self, agent: &Agent, user_id: &UserId) -> Result<()> {
        if !agent.can_modify(user_id) {
            return Err(PlatformError::AgentUnauthorized(
                "Only the creator can modify this agent".to_string(),
            ));
        }
        Ok(())
    }

    /// Convert domain Agent to AgentDto
    fn agent_to_dto(&self, agent: &Agent) -> AgentDto {
        AgentDto {
            id: agent.id.0,
            tenant_id: agent.tenant_id.0,
            name: agent.name.clone(),
            avatar: agent.avatar.clone(),
            greeting: agent.greeting.clone(),
            system_prompt: agent.system_prompt.clone(),
            additional_settings: agent.additional_settings.clone(),
            preset_questions: agent.preset_questions.clone(),
            source_agent_id: agent.source_agent_id.map(|id| id.0),
            creator_id: agent.creator_id.0,
            employer_id: agent.employer_id.map(|id| id.0),
            fired_at: agent.fired_at,
            created_at: agent.created_at,
            updated_at: agent.updated_at,
        }
    }

    /// Convert domain Agent to AgentCardDto
    async fn agent_to_card_dto(&self, agent: &Agent, user_id: &UserId) -> Result<AgentCardDto> {
        // Get creator information
        let creator = self
            .user_repo
            .find_by_id(agent.creator_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Creator not found".to_string()))?;

        // Check if user is the employer
        let is_employer = agent.is_employer(user_id);

        // Check if user has been allocated this agent
        let is_allocated = self
            .allocation_repo
            .is_allocated(&agent.id, user_id)
            .await?;

        // Create preview of system prompt (first 200 characters)
        let system_prompt_preview = if agent.system_prompt.len() > 200 {
            format!("{}...", &agent.system_prompt[..200])
        } else {
            agent.system_prompt.clone()
        };

        Ok(AgentCardDto {
            id: agent.id.0,
            name: agent.name.clone(),
            avatar: agent.avatar.clone(),
            greeting: agent.greeting.clone(),
            system_prompt_preview,
            creator_name: creator
                .nickname
                .clone()
                .unwrap_or(creator.username.0.clone()),
            is_employer,
            is_allocated,
            is_creator: agent.is_creator(user_id),
            is_fired: agent.is_fired(),
            fired_at: agent.fired_at,
            created_at: agent.created_at,
        })
    }

    /// Convert domain Agent to AgentDetailDto
    async fn agent_to_detail_dto(&self, agent: &Agent, user_id: &UserId) -> Result<AgentDetailDto> {
        // Get creator information
        let creator = self
            .user_repo
            .find_by_id(agent.creator_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Creator not found".to_string()))?;

        // Get employer information if exists
        let employer = if let Some(employer_id) = agent.employer_id {
            let employer_user = self
                .user_repo
                .find_by_id(employer_id)
                .await?
                .ok_or_else(|| PlatformError::NotFound("Employer not found".to_string()))?;
            Some(UserSummaryDto {
                id: employer_user.id.0,
                username: employer_user.username.0,
                nickname: employer_user.nickname,
            })
        } else {
            None
        };

        // Get knowledge bases
        let mut knowledge_bases = Vec::new();
        for config_id in &agent.knowledge_base_ids {
            if let Some(config) = self.vector_config_repo.find_by_id(*config_id).await? {
                knowledge_bases.push(VectorConfigSummaryDto {
                    id: config.id.0,
                    name: config.name,
                    provider: config.provider.as_str().to_string(),
                });
            }
        }

        // Get MCP tools
        let mut mcp_tools = Vec::new();
        for tool_id in &agent.mcp_tool_ids {
            if let Some(tool) = self.mcp_tool_repo.find_by_id(*tool_id).await? {
                mcp_tools.push(MCPToolSummaryDto {
                    id: tool.id.0,
                    name: tool.name,
                    description: tool.description,
                });
            }
        }

        // Get flows
        let mut flows = Vec::new();
        for flow_id in &agent.flow_ids {
            if let Some(flow) = self.flow_repo.find_by_id(flow_id).await? {
                flows.push(FlowSummaryDto {
                    id: flow.id.0,
                    name: flow.name.0,
                    description: flow.description,
                });
            }
        }

        // Get source agent if exists
        let source_agent = if let Some(source_id) = agent.source_agent_id {
            if let Some(source) = self.agent_repo.find_by_id(&source_id).await? {
                Some(AgentSourceDto {
                    id: source.id.0,
                    name: source.name,
                })
            } else {
                None
            }
        } else {
            None
        };

        // Check if user is the employer
        let is_employer = agent.is_employer(user_id);

        // Check if user has been allocated this agent
        let is_allocated = self
            .allocation_repo
            .is_allocated(&agent.id, user_id)
            .await?;

        Ok(AgentDetailDto {
            id: agent.id.0,
            tenant_id: agent.tenant_id.0,
            name: agent.name.clone(),
            avatar: agent.avatar.clone(),
            greeting: agent.greeting.clone(),
            knowledge_bases,
            mcp_tools,
            flows,
            system_prompt: agent.system_prompt.clone(),
            additional_settings: agent.additional_settings.clone(),
            preset_questions: agent.preset_questions.clone(),
            source_agent,
            creator: UserSummaryDto {
                id: creator.id.0,
                username: creator.username.0,
                nickname: creator.nickname,
            },
            employer,
            is_employer,
            is_allocated,
            is_creator: agent.is_creator(user_id),
            is_fired: agent.is_fired(),
            fired_at: agent.fired_at,
            created_at: agent.created_at,
            updated_at: agent.updated_at,
        })
    }
}

#[async_trait]
impl AgentApplicationService for AgentApplicationServiceImpl {
    async fn create_agent(
        &self,
        dto: CreateAgentDto,
        tenant_id: TenantId,
        creator_id: UserId,
    ) -> Result<AgentDto> {
        // Create agent entity
        let mut agent = Agent::new(tenant_id, dto.name, dto.system_prompt, creator_id)
            .map_err(|e| PlatformError::AgentValidationError(e))?;

        // Set optional fields
        agent.update_avatar(dto.avatar);
        agent.update_greeting(dto.greeting);
        agent.update_additional_settings(dto.additional_settings);

        if !dto.preset_questions.is_empty() {
            agent
                .set_preset_questions(dto.preset_questions)
                .map_err(|e| PlatformError::AgentValidationError(e))?;
        }

        // Add resources
        for kb_id in dto.knowledge_base_ids {
            agent.add_knowledge_base(ConfigId::from_uuid(kb_id));
        }
        for tool_id in dto.mcp_tool_ids {
            agent.add_mcp_tool(MCPToolId::from_uuid(tool_id));
        }
        for flow_id in dto.flow_ids {
            agent.add_flow(FlowId::from_uuid(flow_id));
        }

        // Validate agent
        agent
            .validate()
            .map_err(|e| PlatformError::AgentValidationError(e))?;

        // Save agent
        self.agent_repo.save(&agent).await?;

        Ok(self.agent_to_dto(&agent))
    }

    async fn get_agent(&self, id: AgentId, user_id: UserId) -> Result<AgentDetailDto> {
        let agent = self
            .agent_repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| PlatformError::AgentNotFound(format!("Agent {} not found", id.0)))?;

        self.agent_to_detail_dto(&agent, &user_id).await
    }

    async fn update_agent(
        &self,
        id: AgentId,
        dto: UpdateAgentDto,
        user_id: UserId,
    ) -> Result<AgentDto> {
        let mut agent = self
            .agent_repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| PlatformError::AgentNotFound(format!("Agent {} not found", id.0)))?;

        // Verify permission
        self.verify_can_modify(&agent, &user_id).await?;

        // Update fields
        if let Some(name) = dto.name {
            agent
                .update_name(name)
                .map_err(|e| PlatformError::AgentValidationError(e))?;
        }

        if let Some(avatar) = dto.avatar {
            agent.update_avatar(Some(avatar));
        }

        if let Some(greeting) = dto.greeting {
            agent.update_greeting(Some(greeting));
        }

        if let Some(system_prompt) = dto.system_prompt {
            agent
                .update_system_prompt(system_prompt)
                .map_err(|e| PlatformError::AgentValidationError(e))?;
        }

        if let Some(additional_settings) = dto.additional_settings {
            agent.update_additional_settings(Some(additional_settings));
        }

        if let Some(preset_questions) = dto.preset_questions {
            agent
                .set_preset_questions(preset_questions)
                .map_err(|e| PlatformError::AgentValidationError(e))?;
        }

        // Validate agent
        agent
            .validate()
            .map_err(|e| PlatformError::AgentValidationError(e))?;

        // Save agent
        self.agent_repo.save(&agent).await?;

        Ok(self.agent_to_dto(&agent))
    }

    async fn delete_agent(&self, id: AgentId, user_id: UserId) -> Result<()> {
        let agent = self
            .agent_repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| PlatformError::AgentNotFound(format!("Agent {} not found", id.0)))?;

        // Verify permission
        self.verify_can_modify(&agent, &user_id).await?;

        // Delete agent (employment and allocation relationships will be cascade deleted by database)
        self.agent_repo.delete(&id).await?;

        Ok(())
    }

    async fn list_agents(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        params: PaginationParams,
        include_fired: bool,
    ) -> Result<PaginatedResponse<AgentCardDto>> {
        let page = params.get_page();
        let limit = params.get_limit();
        let offset = params.get_offset() as usize;

        // Get all agents by tenant (filter by fired status)
        let agents = if include_fired {
            self.agent_repo.find_by_tenant(&tenant_id).await?
        } else {
            self.agent_repo.find_by_tenant_active(&tenant_id).await?
        };

        // Filter out employed agents (those with employer_id set)
        let visible_agents: Vec<_> = agents
            .into_iter()
            .filter(|agent| agent.employer_id.is_none())
            .collect();

        let total = visible_agents.len() as u64;

        // Apply pagination manually
        let paginated_agents: Vec<_> = visible_agents
            .into_iter()
            .skip(offset)
            .take(limit as usize)
            .collect();

        // Convert to card DTOs
        let mut cards = Vec::new();
        for agent in paginated_agents {
            cards.push(self.agent_to_card_dto(&agent, &user_id).await?);
        }

        Ok(PaginatedResponse::new(cards, total, page, limit))
    }

    async fn list_created_agents(
        &self,
        user_id: UserId,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<AgentCardDto>> {
        // Get all agents created by the user
        let agents = self.agent_repo.find_by_creator(&user_id).await?;

        // Filter out agents that have a source_agent_id (copied or employed agents)
        let original_agents: Vec<_> = agents
            .into_iter()
            .filter(|agent| agent.source_agent_id.is_none())
            .collect();

        let total = original_agents.len() as u64;
        let page = params.get_page();
        let limit = params.get_limit();
        let offset = params.get_offset() as usize;

        // Apply pagination manually
        let paginated_agents: Vec<_> = original_agents
            .into_iter()
            .skip(offset)
            .take(limit as usize)
            .collect();

        // Convert to card DTOs
        let mut cards = Vec::new();
        for agent in paginated_agents {
            cards.push(self.agent_to_card_dto(&agent, &user_id).await?);
        }

        Ok(PaginatedResponse::new(cards, total, page, limit))
    }

    async fn copy_agent(
        &self,
        source_id: AgentId,
        user_id: UserId,
        tenant_id: TenantId,
    ) -> Result<AgentDto> {
        let source_agent = self
            .agent_repo
            .find_by_id(&source_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", source_id.0))
            })?;

        // Verify the source agent belongs to the same tenant
        if source_agent.tenant_id != tenant_id {
            return Err(PlatformError::AgentUnauthorized(
                "Cannot copy agent from different tenant".to_string(),
            ));
        }

        // Create a copy
        let copied_agent = source_agent.copy_from(user_id);

        // Validate the copied agent
        copied_agent
            .validate()
            .map_err(|e| PlatformError::AgentValidationError(e))?;

        // Save the copied agent
        self.agent_repo.save(&copied_agent).await?;

        Ok(self.agent_to_dto(&copied_agent))
    }

    async fn employ_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<AgentDto> {
        // Verify agent exists
        let source_agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Create a copy of the agent with employer_id set
        let employed_agent = source_agent.copy_for_employment(user_id);

        // Validate the employed agent
        employed_agent
            .validate()
            .map_err(|e| PlatformError::AgentValidationError(e))?;

        // Save the employed agent
        self.agent_repo.save(&employed_agent).await?;

        // Record employment statistics
        if let Some(stats_service) = &self.stats_service {
            let _ = stats_service.record_employment(agent_id, source_agent.tenant_id).await;
        }

        Ok(self.agent_to_dto(&employed_agent))
    }

    async fn fire_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<()> {
        // Get the agent
        let mut agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify the user is the employer
        if !agent.is_employer(&user_id) {
            return Err(PlatformError::AgentNotEmployer(
                "Only the employer can fire this agent".to_string(),
            ));
        }

        // Fire the agent (sets fired_at timestamp)
        agent
            .fire()
            .map_err(|e| PlatformError::AgentValidationError(e))?;

        // Save the updated agent
        self.agent_repo.save(&agent).await?;

        Ok(())
    }

    async fn list_employed_agents(
        &self,
        user_id: UserId,
        params: PaginationParams,
        include_fired: bool,
    ) -> Result<PaginatedResponse<AgentCardDto>> {
        // Get all agents employed by the user
        let mut agents = self.agent_repo.find_by_employer(&user_id).await?;

        // Filter out fired agents if not including them
        if !include_fired {
            agents.retain(|agent| !agent.is_fired());
        }

        let total = agents.len() as u64;
        let page = params.get_page();
        let limit = params.get_limit();
        let offset = params.get_offset() as usize;

        // Apply pagination manually
        let paginated_agents: Vec<_> = agents
            .into_iter()
            .skip(offset)
            .take(limit as usize)
            .collect();

        // Convert to card DTOs
        let mut cards = Vec::new();
        for agent in paginated_agents {
            cards.push(self.agent_to_card_dto(&agent, &user_id).await?);
        }

        Ok(PaginatedResponse::new(cards, total, page, limit))
    }

    async fn allocate_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<()> {
        // Verify agent exists
        let _agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Create allocation relationship
        self.allocation_repo.allocate(&agent_id, &user_id).await?;

        Ok(())
    }

    async fn terminate_allocation(&self, agent_id: AgentId, user_id: UserId) -> Result<()> {
        // Verify agent exists
        let _agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Terminate allocation relationship
        self.allocation_repo.terminate(&agent_id, &user_id).await?;

        Ok(())
    }

    async fn list_allocated_agents(
        &self,
        user_id: UserId,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<AgentCardDto>> {
        // Get all allocated agents (no pagination at repository level for now)
        let agents = self.agent_repo.find_allocated_to_user(&user_id).await?;

        let total = agents.len() as u64;
        let page = params.get_page();
        let limit = params.get_limit();
        let offset = params.get_offset() as usize;

        // Apply pagination manually
        let paginated_agents: Vec<_> = agents
            .into_iter()
            .skip(offset)
            .take(limit as usize)
            .collect();

        // Convert to card DTOs
        let mut cards = Vec::new();
        for agent in paginated_agents {
            cards.push(self.agent_to_card_dto(&agent, &user_id).await?);
        }

        Ok(PaginatedResponse::new(cards, total, page, limit))
    }

    async fn add_knowledge_base(
        &self,
        agent_id: AgentId,
        config_id: ConfigId,
        user_id: UserId,
    ) -> Result<()> {
        let mut agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify permission
        self.verify_can_modify(&agent, &user_id).await?;

        // Verify knowledge base exists
        let _config = self
            .vector_config_repo
            .find_by_id(config_id)
            .await?
            .ok_or_else(|| {
                PlatformError::NotFound(format!("Knowledge base {} not found", config_id.0))
            })?;

        // Add knowledge base
        agent.add_knowledge_base(config_id);

        // Save agent
        self.agent_repo.save(&agent).await?;

        Ok(())
    }

    async fn remove_knowledge_base(
        &self,
        agent_id: AgentId,
        config_id: ConfigId,
        user_id: UserId,
    ) -> Result<()> {
        let mut agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify permission
        self.verify_can_modify(&agent, &user_id).await?;

        // Remove knowledge base
        agent.remove_knowledge_base(&config_id);

        // Save agent
        self.agent_repo.save(&agent).await?;

        Ok(())
    }

    async fn add_mcp_tool(
        &self,
        agent_id: AgentId,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<()> {
        let mut agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify permission
        self.verify_can_modify(&agent, &user_id).await?;

        // Verify MCP tool exists
        let _tool = self
            .mcp_tool_repo
            .find_by_id(tool_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound(format!("MCP tool {} not found", tool_id.0)))?;

        // Add MCP tool
        agent.add_mcp_tool(tool_id);

        // Save agent
        self.agent_repo.save(&agent).await?;

        Ok(())
    }

    async fn remove_mcp_tool(
        &self,
        agent_id: AgentId,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<()> {
        let mut agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify permission
        self.verify_can_modify(&agent, &user_id).await?;

        // Remove MCP tool
        agent.remove_mcp_tool(&tool_id);

        // Save agent
        self.agent_repo.save(&agent).await?;

        Ok(())
    }

    async fn add_flow(&self, agent_id: AgentId, flow_id: FlowId, user_id: UserId) -> Result<()> {
        let mut agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify permission
        self.verify_can_modify(&agent, &user_id).await?;

        // Verify flow exists
        let _flow = self
            .flow_repo
            .find_by_id(&flow_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound(format!("Flow {} not found", flow_id.0)))?;

        // Add flow
        agent.add_flow(flow_id);

        // Save agent
        self.agent_repo.save(&agent).await?;

        Ok(())
    }

    async fn remove_flow(&self, agent_id: AgentId, flow_id: FlowId, user_id: UserId) -> Result<()> {
        let mut agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify permission
        self.verify_can_modify(&agent, &user_id).await?;

        // Remove flow
        agent.remove_flow(&flow_id);

        // Save agent
        self.agent_repo.save(&agent).await?;

        Ok(())
    }

    async fn chat(
        &self,
        agent_id: AgentId,
        message: String,
        session_id: Option<crate::domain::value_objects::SessionId>,
        user_id: UserId,
        tenant_id: TenantId,
    ) -> Result<crate::application::dto::agent_dto::AgentChatResponse> {
        use crate::domain::value_objects::{ChatMessage, MessageRole};
        use crate::domain::value_objects::chat_message::MessageMetadata;

        // Get the agent
        let agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify agent belongs to the same tenant
        if agent.tenant_id != tenant_id {
            return Err(PlatformError::AgentUnauthorized(
                "Agent does not belong to your tenant".to_string(),
            ));
        }

        // Get or create session
        let session_service = self.session_service.as_ref()
            .ok_or_else(|| PlatformError::InternalError("Session service not configured".to_string()))?;

        let is_new_session = session_id.is_none();
        let session_id = match session_id {
            Some(sid) => sid,
            None => {
                // Create a new session
                let session = session_service
                    .create_session(tenant_id, user_id, Some(format!("Chat with {}", agent.name)))
                    .await?;
                
                // Record new session statistics
                if let Some(stats_service) = &self.stats_service {
                    let _ = stats_service.record_session(agent_id, tenant_id).await;
                }
                
                session.id
            }
        };

        // Add user message to session
        let user_metadata = MessageMetadata {
            model_used: None,
            tokens_used: None,
            response_time_ms: None,
            tool_calls: None,
            custom_data: std::collections::HashMap::from([
                ("agent_id".to_string(), serde_json::json!(agent_id.0.to_string())),
                ("agent_name".to_string(), serde_json::json!(agent.name.clone())),
            ]),
        };

        let user_chat_message = ChatMessage {
            role: MessageRole::User,
            content: crate::domain::value_objects::chat_message::MessageContent::Text(message.clone()),
            metadata: Some(user_metadata),
            timestamp: chrono::Utc::now(),
        };

        let user_message = session_service
            .add_message(&session_id, &tenant_id, &user_id, user_chat_message)
            .await?;

        // Get LLM service and config
        let llm_service = self.llm_service.as_ref()
            .ok_or_else(|| PlatformError::InternalError("LLM service not configured".to_string()))?;

        let llm_config_repo = self.llm_config_repo.as_ref()
            .ok_or_else(|| PlatformError::InternalError("LLM config repository not configured".to_string()))?;

        // Get the first available LLM config for the tenant (TODO: allow agent to specify preferred config)
        let llm_configs = llm_config_repo.find_by_tenant(tenant_id).await?;
        let llm_config = llm_configs.first()
            .ok_or_else(|| PlatformError::NotFound("No LLM configuration found for tenant".to_string()))?;

        // Build conversation history
        let mut messages = vec![
            ChatMessage::new_system_message(agent.system_prompt.clone()),
        ];

        // Add greeting if this is the first message
        if let Some(greeting) = &agent.greeting {
            messages.push(ChatMessage::new_assistant_message(greeting.clone()));
        }

        // Add user message
        messages.push(ChatMessage::new_user_message(message));

        // Call LLM
        let response = llm_service
            .chat_completion(
                &llm_config.model_config,
                messages,
                tenant_id.0,
                None,
            )
            .await
            .map_err(|e| PlatformError::InternalError(format!("LLM error: {}", e)))?;

        // Add assistant response to session
        let assistant_metadata = MessageMetadata {
            model_used: Some(response.model_used.clone()),
            tokens_used: Some(response.usage.total_tokens),
            response_time_ms: None,
            tool_calls: None,
            custom_data: std::collections::HashMap::from([
                ("agent_id".to_string(), serde_json::json!(agent_id.0.to_string())),
                ("agent_name".to_string(), serde_json::json!(agent.name.clone())),
            ]),
        };

        let assistant_chat_message = ChatMessage {
            role: MessageRole::Assistant,
            content: crate::domain::value_objects::chat_message::MessageContent::Text(response.content.clone()),
            metadata: Some(assistant_metadata),
            timestamp: chrono::Utc::now(),
        };

        let assistant_message = session_service
            .add_message(&session_id, &tenant_id, &user_id, assistant_chat_message)
            .await?;

        // Record message and token statistics
        if let Some(stats_service) = &self.stats_service {
            // Record 2 messages (user + assistant)
            let _ = stats_service.record_messages(agent_id, tenant_id, 2).await;
            // Record tokens used
            let _ = stats_service.record_tokens(agent_id, tenant_id, response.usage.total_tokens as i64).await;
        }

        Ok(crate::application::dto::agent_dto::AgentChatResponse {
            session_id: session_id.0,
            message_id: user_message.id.0,
            reply_id: assistant_message.id.0,
            reply: response.content,
            metadata: Some(serde_json::json!({
                "model": response.model_used,
                "tokens_used": response.usage.total_tokens,
                "finish_reason": format!("{:?}", response.finish_reason),
            })),
        })
    }

    async fn get_agent_usage_stats(
        &self,
        agent_id: AgentId,
        query: AgentUsageStatsQuery,
        user_id: UserId,
    ) -> Result<AgentUsageStatsResponse> {
        use sea_orm::{EntityTrait, QueryFilter, QuerySelect, ColumnTrait, QueryOrder};
        use chrono::NaiveDate;

        // Verify agent exists and user has access
        let agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify user is the creator
        if !agent.is_creator(&user_id) {
            return Err(PlatformError::AgentUnauthorized(
                "Only the creator can view agent statistics".to_string(),
            ));
        }

        // Get database connection
        let db = self.db.as_ref()
            .ok_or_else(|| PlatformError::InternalError("Database connection not configured".to_string()))?;

        // Parse date range
        let start_date = query.start_date
            .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
            .unwrap_or_else(|| {
                chrono::Utc::now().date_naive() - chrono::Duration::days(30)
            });

        let end_date = query.end_date
            .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
            .unwrap_or_else(|| chrono::Utc::now().date_naive());

        // Get pagination parameters
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;

        // Query agent_daily_stats table
        use crate::infrastructure::database::entities::agent_daily_stats;

        // Get total count
        let total = agent_daily_stats::Entity::find()
            .filter(agent_daily_stats::Column::AgentId.eq(agent_id.0))
            .filter(agent_daily_stats::Column::StatDate.gte(start_date))
            .filter(agent_daily_stats::Column::StatDate.lte(end_date))
            .count(db.as_ref())
            .await?;

        // Get paginated results
        let stats = agent_daily_stats::Entity::find()
            .filter(agent_daily_stats::Column::AgentId.eq(agent_id.0))
            .filter(agent_daily_stats::Column::StatDate.gte(start_date))
            .filter(agent_daily_stats::Column::StatDate.lte(end_date))
            .order_by_desc(agent_daily_stats::Column::StatDate)
            .offset(offset)
            .limit(page_size)
            .all(db.as_ref())
            .await?;

        // Calculate summary
        let summary_query = agent_daily_stats::Entity::find()
            .filter(agent_daily_stats::Column::AgentId.eq(agent_id.0))
            .filter(agent_daily_stats::Column::StatDate.gte(start_date))
            .filter(agent_daily_stats::Column::StatDate.lte(end_date))
            .all(db.as_ref())
            .await?;

        let summary = if !summary_query.is_empty() {
            let total_interviews: i64 = summary_query.iter().map(|s| s.interview_count).sum();
            let total_interviews_passed: i64 = summary_query.iter().map(|s| s.interview_passed_count).sum();
            let total_employments: i64 = summary_query.iter().map(|s| s.employment_count).sum();
            let total_sessions: i64 = summary_query.iter().map(|s| s.session_count).sum();
            let total_messages: i64 = summary_query.iter().map(|s| s.message_count).sum();
            let total_tokens: i64 = summary_query.iter().map(|s| s.token_count).sum();
            let total_revenue: f64 = summary_query.iter().map(|s| s.revenue.to_string().parse::<f64>().unwrap_or(0.0)).sum();
            
            // For unique_users, we'll use a simple count for now
            // TODO: Implement proper unique user counting across date range
            let unique_users = total_sessions; // Placeholder

            Some(AgentUsageStatsSummaryDto {
                total_interviews,
                total_interviews_passed,
                total_employments,
                total_sessions,
                total_messages,
                total_tokens,
                unique_users,
                total_revenue,
            })
        } else {
            None
        };

        // Convert to DTOs
        let items: Vec<AgentUsageStatsDto> = stats
            .into_iter()
            .map(|stat| AgentUsageStatsDto {
                agent_id: stat.agent_id,
                agent_name: agent.name.clone(),
                date: stat.stat_date.format("%Y-%m-%d").to_string(),
                interview_count: stat.interview_count,
                interview_passed_count: stat.interview_passed_count,
                employment_count: stat.employment_count,
                total_sessions: stat.session_count,
                total_messages: stat.message_count,
                total_tokens: stat.token_count,
                unique_users: 0, // TODO: Calculate unique users per day if needed
                revenue: stat.revenue.to_string().parse::<f64>().unwrap_or(0.0),
                avg_session_duration_seconds: None, // TODO: Calculate if needed
            })
            .collect();

        let total_pages = if page_size > 0 {
            (total + page_size - 1) / page_size
        } else {
            0
        };

        Ok(AgentUsageStatsResponse {
            items,
            page,
            page_size,
            total,
            total_pages,
            summary,
        })
    }

    async fn start_interview(&self, agent_id: AgentId, user_id: UserId, tenant_id: TenantId) -> Result<()> {
        // Verify agent exists
        let agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify agent belongs to the same tenant
        if agent.tenant_id != tenant_id {
            return Err(PlatformError::AgentUnauthorized(
                "Agent does not belong to your tenant".to_string(),
            ));
        }

        // Record interview statistics
        if let Some(stats_service) = &self.stats_service {
            stats_service.record_interview(agent_id, tenant_id).await?;
        }

        Ok(())
    }

    async fn complete_interview(&self, agent_id: AgentId, user_id: UserId, tenant_id: TenantId, passed: bool) -> Result<()> {
        // Verify agent exists
        let agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify agent belongs to the same tenant
        if agent.tenant_id != tenant_id {
            return Err(PlatformError::AgentUnauthorized(
                "Agent does not belong to your tenant".to_string(),
            ));
        }

        // Record interview passed statistics if passed
        if passed {
            if let Some(stats_service) = &self.stats_service {
                stats_service.record_interview_passed(agent_id, tenant_id).await?;
            }
        }

        Ok(())
    }

    async fn chat_stream(
        &self,
        agent_id: AgentId,
        message: String,
        session_id: Option<crate::domain::value_objects::SessionId>,
        user_id: UserId,
        tenant_id: TenantId,
    ) -> Result<Box<dyn futures::Stream<Item = Result<crate::application::dto::agent_dto::AgentChatStreamChunk>> + Send + Unpin>> {
        use crate::domain::value_objects::{ChatMessage, MessageRole};
        use crate::domain::value_objects::chat_message::MessageMetadata;
        use futures::StreamExt;

        // Get the agent
        let agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Verify agent belongs to the same tenant
        if agent.tenant_id != tenant_id {
            return Err(PlatformError::AgentUnauthorized(
                "Agent does not belong to your tenant".to_string(),
            ));
        }

        // Get or create session
        let session_service = self.session_service.as_ref()
            .ok_or_else(|| PlatformError::InternalError("Session service not configured".to_string()))?
            .clone();

        let is_new_session = session_id.is_none();
        let session_id = match session_id {
            Some(sid) => sid,
            None => {
                // Create a new session
                let session = session_service
                    .create_session(tenant_id, user_id, Some(format!("Chat with {}", agent.name)))
                    .await?;
                
                // Record new session statistics
                if let Some(stats_service) = &self.stats_service {
                    let _ = stats_service.record_session(agent_id, tenant_id).await;
                }
                
                session.id
            }
        };

        // Add user message to session
        let user_metadata = MessageMetadata {
            model_used: None,
            tokens_used: None,
            response_time_ms: None,
            tool_calls: None,
            custom_data: std::collections::HashMap::from([
                ("agent_id".to_string(), serde_json::json!(agent_id.0.to_string())),
                ("agent_name".to_string(), serde_json::json!(agent.name.clone())),
            ]),
        };

        let user_chat_message = ChatMessage {
            role: MessageRole::User,
            content: crate::domain::value_objects::chat_message::MessageContent::Text(message.clone()),
            metadata: Some(user_metadata),
            timestamp: chrono::Utc::now(),
        };

        let user_message = session_service
            .add_message(&session_id, &tenant_id, &user_id, user_chat_message)
            .await?;

        // Get LLM service and config
        let llm_service = self.llm_service.as_ref()
            .ok_or_else(|| PlatformError::InternalError("LLM service not configured".to_string()))?
            .clone();

        let llm_config_repo = self.llm_config_repo.as_ref()
            .ok_or_else(|| PlatformError::InternalError("LLM config repository not configured".to_string()))?
            .clone();

        // Get the first available LLM config for the tenant
        let llm_configs = llm_config_repo.find_by_tenant(tenant_id).await?;
        let llm_config = llm_configs.first()
            .ok_or_else(|| PlatformError::NotFound("No LLM configuration found for tenant".to_string()))?
            .clone();

        // Build conversation history
        let mut messages = vec![
            ChatMessage::new_system_message(agent.system_prompt.clone()),
        ];

        // Add greeting if this is the first message
        if let Some(greeting) = &agent.greeting {
            messages.push(ChatMessage::new_assistant_message(greeting.clone()));
        }

        // Add user message
        messages.push(ChatMessage::new_user_message(message));

        // Call LLM streaming
        let stream = llm_service
            .stream_chat_completion(
                &llm_config.model_config,
                messages,
                tenant_id.0,
            )
            .await
            .map_err(|e| PlatformError::InternalError(format!("LLM error: {}", e)))?;

        // Transform the stream
        let agent_id_clone = agent_id;
        let agent_name = agent.name.clone();
        let session_id_clone = session_id;
        let user_message_id = user_message.id;
        let session_service_clone = session_service.clone();
        let tenant_id_clone = tenant_id;
        let user_id_clone = user_id;
        let stats_service = self.stats_service.clone();

        let mut accumulated_content = String::new();
        let mut total_tokens = 0u32;
        let mut model_used = String::new();
        let mut reply_message_id: Option<crate::domain::value_objects::MessageId> = None;

        let transformed_stream = stream.then(move |chunk_result| {
            let session_service = session_service_clone.clone();
            let agent_name = agent_name.clone();
            let stats_service = stats_service.clone();
            let mut accumulated_content = accumulated_content.clone();
            let mut model_used = model_used.clone();
            
            async move {
                match chunk_result {
                    Ok(chunk) => {
                        // Accumulate content
                        if let Some(content) = &chunk.content {
                            accumulated_content.push_str(content);
                        }

                        // Update usage info
                        if let Some(usage) = &chunk.usage {
                            total_tokens = usage.total_tokens;
                        }

                        // Check if this is the final chunk
                        if let Some(finish_reason) = &chunk.finish_reason {
                            // Save the complete assistant message
                            let assistant_metadata = MessageMetadata {
                                model_used: Some(model_used.clone()),
                                tokens_used: Some(total_tokens),
                                response_time_ms: None,
                                tool_calls: None,
                                custom_data: std::collections::HashMap::from([
                                    ("agent_id".to_string(), serde_json::json!(agent_id_clone.0.to_string())),
                                    ("agent_name".to_string(), serde_json::json!(agent_name.clone())),
                                ]),
                            };

                            let assistant_chat_message = ChatMessage {
                                role: MessageRole::Assistant,
                                content: crate::domain::value_objects::chat_message::MessageContent::Text(accumulated_content.clone()),
                                metadata: Some(assistant_metadata),
                                timestamp: chrono::Utc::now(),
                            };

                            if let Ok(assistant_message) = session_service
                                .add_message(&session_id_clone, &tenant_id_clone, &user_id_clone, assistant_chat_message)
                                .await
                            {
                                reply_message_id = Some(assistant_message.id);

                                // Record statistics
                                if let Some(stats_svc) = &stats_service {
                                    let _ = stats_svc.record_messages(agent_id_clone, tenant_id_clone, 2).await;
                                    let _ = stats_svc.record_tokens(agent_id_clone, tenant_id_clone, total_tokens as i64).await;
                                }

                                // Return final chunk with all IDs
                                return Ok(crate::application::dto::agent_dto::AgentChatStreamChunk {
                                    chunk_type: "done".to_string(),
                                    content: None,
                                    reasoning_content: None,
                                    session_id: Some(session_id_clone.0),
                                    message_id: Some(user_message_id.0),
                                    reply_id: Some(assistant_message.id.0),
                                    metadata: Some(serde_json::json!({
                                        "model": model_used,
                                        "tokens_used": total_tokens,
                                        "finish_reason": format!("{:?}", finish_reason),
                                    })),
                                    finish_reason: Some(format!("{:?}", finish_reason)),
                                    error: None,
                                });
                            }
                        }

                        // Return content chunk
                        Ok(crate::application::dto::agent_dto::AgentChatStreamChunk {
                            chunk_type: "content".to_string(),
                            content: chunk.content,
                            reasoning_content: chunk.reasoning_content,
                            session_id: Some(session_id_clone.0),
                            message_id: Some(user_message_id.0),
                            reply_id: reply_message_id.map(|id| id.0),
                            metadata: None,
                            finish_reason: None,
                            error: None,
                        })
                    }
                    Err(e) => {
                        // Return error chunk
                        Ok(crate::application::dto::agent_dto::AgentChatStreamChunk {
                            chunk_type: "error".to_string(),
                            content: None,
                            reasoning_content: None,
                            session_id: Some(session_id_clone.0),
                            message_id: Some(user_message_id.0),
                            reply_id: None,
                            metadata: None,
                            finish_reason: None,
                            error: Some(format!("{}", e)),
                        })
                    }
                }
            }
        });

        Ok(Box::new(Box::pin(transformed_stream)))
    }
}
