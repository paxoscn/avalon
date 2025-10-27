use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Create Agent request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentDto {
    pub name: String,
    pub avatar: Option<String>,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Vec<String>,
    pub knowledge_base_ids: Vec<Uuid>,
    pub mcp_tool_ids: Vec<Uuid>,
    pub flow_ids: Vec<Uuid>,
}

/// Update Agent request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgentDto {
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub system_prompt: Option<String>,
    pub additional_settings: Option<String>,
    pub preset_questions: Option<Vec<String>>,
}

/// Agent response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Vec<String>,
    pub source_agent_id: Option<Uuid>,
    pub creator_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Agent card DTO for list view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCardDto {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub system_prompt_preview: String,
    pub creator_name: String,
    pub is_employed: bool,
    pub is_creator: bool,
    pub created_at: DateTime<Utc>,
}

/// Agent detail DTO with full information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDetailDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub knowledge_bases: Vec<VectorConfigSummaryDto>,
    pub mcp_tools: Vec<MCPToolSummaryDto>,
    pub flows: Vec<FlowSummaryDto>,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Vec<String>,
    pub source_agent: Option<AgentSourceDto>,
    pub creator: UserSummaryDto,
    pub is_employed: bool,
    pub is_creator: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Agent source reference DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSourceDto {
    pub id: Uuid,
    pub name: String,
}

/// User summary DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummaryDto {
    pub id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
}

/// Vector config summary DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfigSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
}

/// MCP tool summary DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

/// Flow summary DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u64, limit: u64) -> Self {
        let total_pages = if limit > 0 {
            (total + limit - 1) / limit
        } else {
            0
        };

        Self {
            items,
            total,
            page,
            limit,
            total_pages,
        }
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
        }
    }
}

impl PaginationParams {
    pub fn get_page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_limit(&self) -> u64 {
        self.limit.unwrap_or(20).clamp(1, 100)
    }

    pub fn get_offset(&self) -> u64 {
        (self.get_page() - 1) * self.get_limit()
    }
}

/// Add resource request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddResourceDto {
    pub resource_id: Uuid,
}

/// Remove resource request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveResourceDto {
    pub resource_id: Uuid,
}

/// Agent list query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentListQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub employed_only: Option<bool>,
    pub search: Option<String>,
}

impl Default for AgentListQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
            employed_only: None,
            search: None,
        }
    }
}
