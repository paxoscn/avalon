use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QuerySelect, PaginatorTrait, QueryOrder, Set};
use std::sync::Arc;
use chrono::Utc;
use crate::domain::entities::Agent;
use crate::domain::repositories::{AgentRepository, AgentAllocationRepository};
use crate::domain::value_objects::{AgentId, TenantId, UserId, ConfigId, MCPToolId, FlowId};
use crate::infrastructure::database::entities;
use crate::error::{Result, PlatformError};

pub struct AgentRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl AgentRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_domain(entity: entities::agent::Model) -> Result<Agent> {
        // Parse JSON arrays for IDs
        let knowledge_base_ids: Vec<uuid::Uuid> = serde_json::from_value(entity.knowledge_base_ids.clone())
            .map_err(|e| PlatformError::ValidationError(format!("Invalid knowledge_base_ids: {}", e)))?;
        
        let mcp_tool_ids: Vec<uuid::Uuid> = serde_json::from_value(entity.mcp_tool_ids.clone())
            .map_err(|e| PlatformError::ValidationError(format!("Invalid mcp_tool_ids: {}", e)))?;
        
        let flow_ids: Vec<uuid::Uuid> = serde_json::from_value(entity.flow_ids.clone())
            .map_err(|e| PlatformError::ValidationError(format!("Invalid flow_ids: {}", e)))?;
        
        let preset_questions: Vec<String> = serde_json::from_value(entity.preset_questions.clone())
            .map_err(|e| PlatformError::ValidationError(format!("Invalid preset_questions: {}", e)))?;

        Ok(Agent {
            id: AgentId::from_uuid(entity.id),
            tenant_id: TenantId::from_uuid(entity.tenant_id),
            name: entity.name,
            avatar: entity.avatar,
            greeting: entity.greeting,
            knowledge_base_ids: knowledge_base_ids.into_iter().map(ConfigId::from_uuid).collect(),
            mcp_tool_ids: mcp_tool_ids.into_iter().map(MCPToolId::from_uuid).collect(),
            flow_ids: flow_ids.into_iter().map(FlowId::from_uuid).collect(),
            system_prompt: entity.system_prompt,
            additional_settings: entity.additional_settings,
            preset_questions,
            source_agent_id: entity.source_agent_id.map(AgentId::from_uuid),
            creator_id: UserId::from_uuid(entity.creator_id),
            employer_id: entity.employer_id.map(UserId::from_uuid),
            fired_at: entity.fired_at,
            is_published: entity.is_published,
            published_at: entity.published_at,
            price: entity.price,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        })
    }

    fn domain_to_active_model(agent: &Agent) -> Result<entities::agent::ActiveModel> {
        // Convert ID vectors to JSON
        let knowledge_base_ids: Vec<uuid::Uuid> = agent.knowledge_base_ids.iter().map(|id| id.0).collect();
        let mcp_tool_ids: Vec<uuid::Uuid> = agent.mcp_tool_ids.iter().map(|id| id.0).collect();
        let flow_ids: Vec<uuid::Uuid> = agent.flow_ids.iter().map(|id| id.0).collect();
        
        let knowledge_base_ids_json = serde_json::to_value(knowledge_base_ids)
            .map_err(|e| PlatformError::ValidationError(format!("Failed to serialize knowledge_base_ids: {}", e)))?;
        
        let mcp_tool_ids_json = serde_json::to_value(mcp_tool_ids)
            .map_err(|e| PlatformError::ValidationError(format!("Failed to serialize mcp_tool_ids: {}", e)))?;
        
        let flow_ids_json = serde_json::to_value(flow_ids)
            .map_err(|e| PlatformError::ValidationError(format!("Failed to serialize flow_ids: {}", e)))?;
        
        let preset_questions_json = serde_json::to_value(&agent.preset_questions)
            .map_err(|e| PlatformError::ValidationError(format!("Failed to serialize preset_questions: {}", e)))?;

        Ok(entities::agent::ActiveModel {
            id: Set(agent.id.0),
            tenant_id: Set(agent.tenant_id.0),
            name: Set(agent.name.clone()),
            avatar: Set(agent.avatar.clone()),
            greeting: Set(agent.greeting.clone()),
            knowledge_base_ids: Set(knowledge_base_ids_json),
            mcp_tool_ids: Set(mcp_tool_ids_json),
            flow_ids: Set(flow_ids_json),
            system_prompt: Set(agent.system_prompt.clone()),
            additional_settings: Set(agent.additional_settings.clone()),
            preset_questions: Set(preset_questions_json),
            source_agent_id: Set(agent.source_agent_id.map(|id| id.0)),
            creator_id: Set(agent.creator_id.0),
            employer_id: Set(agent.employer_id.map(|id| id.0)),
            fired_at: Set(agent.fired_at),
            is_published: Set(agent.is_published),
            published_at: Set(agent.published_at),
            price: Set(agent.price),
            created_at: Set(agent.created_at),
            updated_at: Set(agent.updated_at),
        })
    }
}

#[async_trait]
impl AgentRepository for AgentRepositoryImpl {
    async fn find_by_id(&self, id: &AgentId) -> Result<Option<Agent>> {
        let agent = entities::agent::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await?;

        match agent {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<Agent>> {
        let agents = entities::agent::Entity::find()
            .filter(entities::agent::Column::TenantId.eq(tenant_id.0))
            .order_by_desc(entities::agent::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in agents {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_tenant_active(&self, tenant_id: &TenantId) -> Result<Vec<Agent>> {
        let agents = entities::agent::Entity::find()
            .filter(entities::agent::Column::TenantId.eq(tenant_id.0))
            .filter(entities::agent::Column::FiredAt.is_null())
            .order_by_desc(entities::agent::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in agents {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_creator(&self, creator_id: &UserId) -> Result<Vec<Agent>> {
        let agents = entities::agent::Entity::find()
            .filter(entities::agent::Column::CreatorId.eq(creator_id.0))
            .order_by_desc(entities::agent::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in agents {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_employer(&self, employer_id: &UserId) -> Result<Vec<Agent>> {
        let agents = entities::agent::Entity::find()
            .filter(entities::agent::Column::EmployerId.eq(employer_id.0))
            .order_by_desc(entities::agent::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in agents {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_allocated_to_user(&self, user_id: &UserId) -> Result<Vec<Agent>> {
        // Join with allocations table to find allocated agents
        let agent_ids: Vec<uuid::Uuid> = entities::agent_allocation::Entity::find()
            .filter(entities::agent_allocation::Column::UserId.eq(user_id.0))
            .all(self.db.as_ref())
            .await?
            .into_iter()
            .map(|e| e.agent_id)
            .collect();

        if agent_ids.is_empty() {
            return Ok(Vec::new());
        }

        let agents = entities::agent::Entity::find()
            .filter(entities::agent::Column::Id.is_in(agent_ids))
            .order_by_desc(entities::agent::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in agents {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn save(&self, agent: &Agent) -> Result<()> {
        let active_model = Self::domain_to_active_model(agent)?;
        
        // Check if agent exists
        let existing = entities::agent::Entity::find_by_id(agent.id.0)
            .one(self.db.as_ref())
            .await?;

        if existing.is_some() {
            // Update existing agent
            entities::agent::Entity::update(active_model)
                .exec(self.db.as_ref())
                .await?;
        } else {
            // Insert new agent
            entities::agent::Entity::insert(active_model)
                .exec(self.db.as_ref())
                .await?;
        }

        Ok(())
    }

    async fn delete(&self, id: &AgentId) -> Result<()> {
        entities::agent::Entity::delete_by_id(id.0)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn count_by_tenant(&self, tenant_id: &TenantId) -> Result<u64> {
        let count = entities::agent::Entity::find()
            .filter(entities::agent::Column::TenantId.eq(tenant_id.0))
            .count(self.db.as_ref())
            .await?;

        Ok(count)
    }

    async fn count_by_tenant_active(&self, tenant_id: &TenantId) -> Result<u64> {
        let count = entities::agent::Entity::find()
            .filter(entities::agent::Column::TenantId.eq(tenant_id.0))
            .filter(entities::agent::Column::FiredAt.is_null())
            .count(self.db.as_ref())
            .await?;

        Ok(count)
    }

    async fn find_by_tenant_paginated(
        &self,
        tenant_id: &TenantId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Agent>> {
        let agents = entities::agent::Entity::find()
            .filter(entities::agent::Column::TenantId.eq(tenant_id.0))
            .order_by_desc(entities::agent::Column::CreatedAt)
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in agents {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_tenant_active_paginated(
        &self,
        tenant_id: &TenantId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Agent>> {
        let agents = entities::agent::Entity::find()
            .filter(entities::agent::Column::TenantId.eq(tenant_id.0))
            .filter(entities::agent::Column::FiredAt.is_null())
            .order_by_desc(entities::agent::Column::CreatedAt)
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in agents {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_tenant_published(&self, tenant_id: &TenantId) -> Result<Vec<Agent>> {
        let agents = entities::agent::Entity::find()
            .filter(entities::agent::Column::TenantId.eq(tenant_id.0))
            .filter(entities::agent::Column::IsPublished.eq(true))
            .filter(entities::agent::Column::EmployerId.is_null())
            .order_by_desc(entities::agent::Column::CreatedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in agents {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }
}

pub struct AgentAllocationRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl AgentAllocationRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AgentAllocationRepository for AgentAllocationRepositoryImpl {
    async fn allocate(&self, agent_id: &AgentId, user_id: &UserId) -> Result<()> {
        // Check if allocation already exists
        let existing = entities::agent_allocation::Entity::find()
            .filter(entities::agent_allocation::Column::AgentId.eq(agent_id.0))
            .filter(entities::agent_allocation::Column::UserId.eq(user_id.0))
            .one(self.db.as_ref())
            .await?;

        if existing.is_some() {
            return Err(PlatformError::AgentAlreadyAllocated(
                format!("User {} has already been allocated agent {}", user_id.0, agent_id.0)
            ));
        }

        let allocation = entities::agent_allocation::ActiveModel {
            agent_id: Set(agent_id.0),
            user_id: Set(user_id.0),
            allocated_at: Set(Utc::now()),
        };

        entities::agent_allocation::Entity::insert(allocation)
            .exec(self.db.as_ref())
            .await?;

        Ok(())
    }

    async fn terminate(&self, agent_id: &AgentId, user_id: &UserId) -> Result<()> {
        let result = entities::agent_allocation::Entity::delete_many()
            .filter(entities::agent_allocation::Column::AgentId.eq(agent_id.0))
            .filter(entities::agent_allocation::Column::UserId.eq(user_id.0))
            .exec(self.db.as_ref())
            .await?;

        if result.rows_affected == 0 {
            return Err(PlatformError::AgentNotAllocated(
                format!("User {} has not allocated agent {}", user_id.0, agent_id.0)
            ));
        }

        Ok(())
    }

    async fn is_allocated(&self, agent_id: &AgentId, user_id: &UserId) -> Result<bool> {
        let count = entities::agent_allocation::Entity::find()
            .filter(entities::agent_allocation::Column::AgentId.eq(agent_id.0))
            .filter(entities::agent_allocation::Column::UserId.eq(user_id.0))
            .count(self.db.as_ref())
            .await?;

        Ok(count > 0)
    }

    async fn find_by_agent(&self, agent_id: &AgentId) -> Result<Vec<UserId>> {
        let allocations = entities::agent_allocation::Entity::find()
            .filter(entities::agent_allocation::Column::AgentId.eq(agent_id.0))
            .all(self.db.as_ref())
            .await?;

        Ok(allocations.into_iter().map(|e| UserId::from_uuid(e.user_id)).collect())
    }

    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<AgentId>> {
        let allocations = entities::agent_allocation::Entity::find()
            .filter(entities::agent_allocation::Column::UserId.eq(user_id.0))
            .all(self.db.as_ref())
            .await?;

        Ok(allocations.into_iter().map(|e| AgentId::from_uuid(e.agent_id)).collect())
    }
}
