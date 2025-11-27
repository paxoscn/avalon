use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Create Agent request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentDto {
    pub name: String,
    pub avatar: Option<String>,
    pub greeting: Option<String>,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Vec<String>,
    pub knowledge_base_ids: Vec<Uuid>,
    pub mcp_tool_ids: Vec<Uuid>,
    pub flow_ids: Vec<Uuid>,
    pub price: Option<rust_decimal::Decimal>,
}

/// Update Agent request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgentDto {
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub greeting: Option<String>,
    pub system_prompt: Option<String>,
    pub additional_settings: Option<String>,
    pub preset_questions: Option<Vec<String>>,
    pub price: Option<rust_decimal::Decimal>,
}

/// Agent response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub greeting: Option<String>,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Vec<String>,
    pub source_agent_id: Option<Uuid>,
    pub creator_id: Uuid,
    pub employer_id: Option<Uuid>,
    pub fired_at: Option<DateTime<Utc>>,
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub price: Option<rust_decimal::Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Agent card DTO for list view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCardDto {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub greeting: Option<String>,
    pub system_prompt_preview: String,
    pub creator_name: String,
    pub is_employer: bool,
    pub is_allocated: bool,
    pub is_creator: bool,
    pub is_fired: bool,
    pub fired_at: Option<DateTime<Utc>>,
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub price: Option<rust_decimal::Decimal>,
    pub created_at: DateTime<Utc>,
}

/// Agent detail DTO with full information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDetailDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub greeting: Option<String>,
    pub knowledge_bases: Vec<VectorConfigSummaryDto>,
    pub mcp_tools: Vec<MCPToolSummaryDto>,
    pub flows: Vec<FlowSummaryDto>,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Vec<String>,
    pub source_agent: Option<AgentSourceDto>,
    pub creator: UserSummaryDto,
    pub employer: Option<UserSummaryDto>,
    pub is_employer: bool,
    pub is_allocated: bool,
    pub is_creator: bool,
    pub is_fired: bool,
    pub fired_at: Option<DateTime<Utc>>,
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub price: Option<rust_decimal::Decimal>,
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
    pub allocated_only: Option<bool>,
    pub include_fired: Option<bool>,
    pub search: Option<String>,
}

impl Default for AgentListQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
            employed_only: None,
            allocated_only: None,
            include_fired: None,
            search: None,
        }
    }
}

/// Agent chat request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatRequest {
    pub message: String,
    pub session_id: Option<Uuid>,
    pub stream: Option<bool>,
}

/// Agent chat stream chunk DTO (for SSE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatStreamChunk {
    #[serde(rename = "type")]
    pub chunk_type: String,
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
    pub session_id: Option<Uuid>,
    pub message_id: Option<Uuid>,
    pub reply_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub finish_reason: Option<String>,
    pub error: Option<String>,
}

/// Agent chat response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatResponse {
    pub session_id: Uuid,
    pub message_id: Uuid,
    pub reply_id: Uuid,
    pub reply: String,
    pub metadata: Option<serde_json::Value>,
}

/// Agent usage stats query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsageStatsQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// Agent usage stats DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsageStatsDto {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub date: String,
    pub interview_count: i64,
    pub interview_passed_count: i64,
    pub employment_count: i64,
    pub total_sessions: i64,
    pub total_messages: i64,
    pub total_tokens: i64,
    pub unique_users: i64,
    pub revenue: f64,
    pub avg_session_duration_seconds: Option<f64>,
}

/// Agent usage stats summary DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsageStatsSummaryDto {
    pub total_interviews: i64,
    pub total_interviews_passed: i64,
    pub total_employments: i64,
    pub total_sessions: i64,
    pub total_messages: i64,
    pub total_tokens: i64,
    pub unique_users: i64,
    pub total_revenue: f64,
}

/// Agent usage stats response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsageStatsResponse {
    pub items: Vec<AgentUsageStatsDto>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64,
    pub summary: Option<AgentUsageStatsSummaryDto>,
}

/// Complete interview request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteInterviewRequest {
    pub passed: bool,
    pub score: Option<i32>,
    pub feedback: Option<String>,
}

/// Interview record DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewRecordDto {
    pub id: String,
    pub agent_id: String,
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub status: String,
    pub score: Option<i32>,
    pub feedback: Option<String>,
    pub questions: Option<serde_json::Value>,
    pub answers: Option<serde_json::Value>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
