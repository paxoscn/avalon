use crate::domain::value_objects::{AgentId, ConfigId, FlowId, MCPToolId, TenantId, UserId};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub tenant_id: TenantId,
    pub name: String,
    pub avatar: Option<String>,
    pub greeting: Option<String>,
    pub knowledge_base_ids: Vec<ConfigId>,
    pub mcp_tool_ids: Vec<MCPToolId>,
    pub flow_ids: Vec<FlowId>,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Vec<String>,
    pub source_agent_id: Option<AgentId>,
    pub creator_id: UserId,
    pub employer_id: Option<UserId>,
    pub fired_at: Option<DateTime<Utc>>,
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub price: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Agent {
    pub fn new(
        tenant_id: TenantId,
        name: String,
        system_prompt: String,
        creator_id: UserId,
    ) -> Result<Self, String> {
        // Validate name
        if name.trim().is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }
        if name.len() > 255 {
            return Err("Agent name cannot exceed 255 characters".to_string());
        }

        // Validate system prompt
        if system_prompt.trim().is_empty() {
            return Err("System prompt cannot be empty".to_string());
        }

        let now = Utc::now();

        Ok(Agent {
            id: AgentId::new(),
            tenant_id,
            name,
            avatar: None,
            greeting: None,
            knowledge_base_ids: Vec::new(),
            mcp_tool_ids: Vec::new(),
            flow_ids: Vec::new(),
            system_prompt,
            additional_settings: None,
            preset_questions: Vec::new(),
            source_agent_id: None,
            creator_id,
            employer_id: None,
            fired_at: None,
            is_published: false,
            published_at: None,
            price: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_name(&mut self, name: String) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }
        if name.len() > 255 {
            return Err("Agent name cannot exceed 255 characters".to_string());
        }

        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_avatar(&mut self, avatar: Option<String>) {
        self.avatar = avatar;
        self.updated_at = Utc::now();
    }

    pub fn update_greeting(&mut self, greeting: Option<String>) {
        self.greeting = greeting;
        self.updated_at = Utc::now();
    }

    pub fn update_system_prompt(&mut self, prompt: String) -> Result<(), String> {
        if prompt.trim().is_empty() {
            return Err("System prompt cannot be empty".to_string());
        }

        self.system_prompt = prompt;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_additional_settings(&mut self, settings: Option<String>) {
        self.additional_settings = settings;
        self.updated_at = Utc::now();
    }

    pub fn update_price(&mut self, price: Option<Decimal>) -> Result<(), String> {
        if let Some(p) = price {
            if p < Decimal::ZERO {
                return Err("Price cannot be negative".to_string());
            }
            // Validate precision (4 decimal places)
            if p.scale() > 4 {
                return Err("Price precision cannot exceed 4 decimal places".to_string());
            }
        }

        self.price = price;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_preset_questions(&mut self, questions: Vec<String>) -> Result<(), String> {
        if questions.len() > 3 {
            return Err("Preset questions cannot exceed 3 items".to_string());
        }

        self.preset_questions = questions;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_knowledge_base(&mut self, config_id: ConfigId) {
        if !self.knowledge_base_ids.contains(&config_id) {
            self.knowledge_base_ids.push(config_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_knowledge_base(&mut self, config_id: &ConfigId) {
        if let Some(pos) = self
            .knowledge_base_ids
            .iter()
            .position(|id| id == config_id)
        {
            self.knowledge_base_ids.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    pub fn add_mcp_tool(&mut self, tool_id: MCPToolId) {
        if !self.mcp_tool_ids.contains(&tool_id) {
            self.mcp_tool_ids.push(tool_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_mcp_tool(&mut self, tool_id: &MCPToolId) {
        if let Some(pos) = self.mcp_tool_ids.iter().position(|id| id == tool_id) {
            self.mcp_tool_ids.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    pub fn add_flow(&mut self, flow_id: FlowId) {
        if !self.flow_ids.contains(&flow_id) {
            self.flow_ids.push(flow_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_flow(&mut self, flow_id: &FlowId) {
        if let Some(pos) = self.flow_ids.iter().position(|id| id == flow_id) {
            self.flow_ids.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    pub fn is_creator(&self, user_id: &UserId) -> bool {
        &self.creator_id == user_id
    }

    pub fn can_modify(&self, user_id: &UserId) -> bool {
        self.is_creator(user_id)
    }

    pub fn copy_from(&self, new_creator_id: UserId) -> Self {
        let now = Utc::now();

        Agent {
            id: AgentId::new(),
            tenant_id: self.tenant_id,
            name: self.name.clone(),
            avatar: self.avatar.clone(),
            greeting: self.greeting.clone(),
            knowledge_base_ids: self.knowledge_base_ids.clone(),
            mcp_tool_ids: self.mcp_tool_ids.clone(),
            flow_ids: self.flow_ids.clone(),
            system_prompt: self.system_prompt.clone(),
            additional_settings: self.additional_settings.clone(),
            preset_questions: self.preset_questions.clone(),
            source_agent_id: Some(self.id),
            creator_id: new_creator_id,
            employer_id: None,
            fired_at: None,
            is_published: false,
            published_at: None,
            price: self.price,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn copy_for_employment(&self, employer_id: UserId) -> Self {
        let now = Utc::now();

        Agent {
            id: AgentId::new(),
            tenant_id: self.tenant_id,
            name: self.name.clone(),
            avatar: self.avatar.clone(),
            greeting: self.greeting.clone(),
            knowledge_base_ids: self.knowledge_base_ids.clone(),
            mcp_tool_ids: self.mcp_tool_ids.clone(),
            flow_ids: self.flow_ids.clone(),
            system_prompt: self.system_prompt.clone(),
            additional_settings: self.additional_settings.clone(),
            preset_questions: self.preset_questions.clone(),
            source_agent_id: Some(self.id),
            creator_id: self.creator_id,
            employer_id: Some(employer_id),
            fired_at: None,
            is_published: false,
            published_at: None,
            price: self.price,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn employ(&mut self, employer_id: UserId) -> Result<(), String> {
        if self.is_employed() {
            return Err("Agent is already employed".to_string());
        }
        if self.is_fired() {
            return Err("Agent has been fired and cannot be employed again".to_string());
        }

        self.employer_id = Some(employer_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn fire(&mut self) -> Result<(), String> {
        if !self.is_employed() {
            return Err("Agent is not employed".to_string());
        }
        if self.is_fired() {
            return Err("Agent has already been fired".to_string());
        }

        self.fired_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn is_employed(&self) -> bool {
        self.employer_id.is_some()
    }

    pub fn is_fired(&self) -> bool {
        self.fired_at.is_some()
    }

    pub fn is_employer(&self, user_id: &UserId) -> bool {
        match &self.employer_id {
            Some(employer) => employer == user_id,
            None => false,
        }
    }

    pub fn publish(&mut self) -> Result<(), String> {
        if self.is_published {
            return Err("Agent is already published".to_string());
        }

        self.is_published = true;
        self.published_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn unpublish(&mut self) -> Result<(), String> {
        if !self.is_published {
            return Err("Agent is not published".to_string());
        }

        self.is_published = false;
        self.published_at = None;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if self.name.trim().is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }
        if self.name.len() > 255 {
            return Err("Agent name cannot exceed 255 characters".to_string());
        }

        // Validate system prompt
        if self.system_prompt.trim().is_empty() {
            return Err("System prompt cannot be empty".to_string());
        }

        // Validate preset questions count
        if self.preset_questions.len() > 3 {
            return Err("Preset questions cannot exceed 3 items".to_string());
        }

        Ok(())
    }
}