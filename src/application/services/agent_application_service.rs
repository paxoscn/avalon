use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    application::dto::agent_dto::*,
    domain::{
        entities::Agent,
        repositories::{
            AgentEmploymentRepository, AgentRepository, FlowRepository, MCPToolRepository,
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

    /// Employ an agent
    async fn employ_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<()>;

    /// Terminate employment
    async fn terminate_employment(&self, agent_id: AgentId, user_id: UserId) -> Result<()>;

    /// List employed agents
    async fn list_employed_agents(
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
}

/// Agent application service implementation
pub struct AgentApplicationServiceImpl {
    agent_repo: Arc<dyn AgentRepository>,
    employment_repo: Arc<dyn AgentEmploymentRepository>,
    vector_config_repo: Arc<dyn VectorConfigRepository>,
    mcp_tool_repo: Arc<dyn MCPToolRepository>,
    flow_repo: Arc<dyn FlowRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl AgentApplicationServiceImpl {
    pub fn new(
        agent_repo: Arc<dyn AgentRepository>,
        employment_repo: Arc<dyn AgentEmploymentRepository>,
        vector_config_repo: Arc<dyn VectorConfigRepository>,
        mcp_tool_repo: Arc<dyn MCPToolRepository>,
        flow_repo: Arc<dyn FlowRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            agent_repo,
            employment_repo,
            vector_config_repo,
            mcp_tool_repo,
            flow_repo,
            user_repo,
        }
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

        // Check if user has employed this agent
        let is_employed = self.employment_repo.is_employed(&agent.id, user_id).await?;

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
            is_employed,
            is_creator: agent.is_creator(user_id),
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

        // Check if user has employed this agent
        let is_employed = self.employment_repo.is_employed(&agent.id, user_id).await?;

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
            is_employed,
            is_creator: agent.is_creator(user_id),
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

        // Delete agent (employment relationships will be cascade deleted by database)
        self.agent_repo.delete(&id).await?;

        Ok(())
    }

    async fn list_agents(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<AgentCardDto>> {
        let page = params.get_page();
        let limit = params.get_limit();
        let offset = params.get_offset();

        // Get agents with pagination
        let agents = self
            .agent_repo
            .find_by_tenant_paginated(&tenant_id, offset, limit)
            .await?;
        let total = self.agent_repo.count_by_tenant(&tenant_id).await?;

        // Convert to card DTOs
        let mut cards = Vec::new();
        for agent in agents {
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

    async fn employ_agent(&self, agent_id: AgentId, user_id: UserId) -> Result<()> {
        // Verify agent exists
        let _agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Create employment relationship
        self.employment_repo.employ(&agent_id, &user_id).await?;

        Ok(())
    }

    async fn terminate_employment(&self, agent_id: AgentId, user_id: UserId) -> Result<()> {
        // Verify agent exists
        let _agent = self
            .agent_repo
            .find_by_id(&agent_id)
            .await?
            .ok_or_else(|| {
                PlatformError::AgentNotFound(format!("Agent {} not found", agent_id.0))
            })?;

        // Terminate employment relationship
        self.employment_repo.terminate(&agent_id, &user_id).await?;

        Ok(())
    }

    async fn list_employed_agents(
        &self,
        user_id: UserId,
        params: PaginationParams,
    ) -> Result<PaginatedResponse<AgentCardDto>> {
        // Get all employed agents (no pagination at repository level for now)
        let agents = self.agent_repo.find_employed_by_user(&user_id).await?;

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
}
