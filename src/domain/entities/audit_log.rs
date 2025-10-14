use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Audit action types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    Create,
    Update,
    Delete,
    Execute,
    Login,
    Logout,
    View,
    Export,
    Import,
    Custom(String),
}

impl AuditAction {
    pub fn as_str(&self) -> &str {
        match self {
            AuditAction::Create => "create",
            AuditAction::Update => "update",
            AuditAction::Delete => "delete",
            AuditAction::Execute => "execute",
            AuditAction::Login => "login",
            AuditAction::Logout => "logout",
            AuditAction::View => "view",
            AuditAction::Export => "export",
            AuditAction::Import => "import",
            AuditAction::Custom(s) => s,
        }
    }
}

impl From<String> for AuditAction {
    fn from(s: String) -> Self {
        match s.as_str() {
            "create" => AuditAction::Create,
            "update" => AuditAction::Update,
            "delete" => AuditAction::Delete,
            "execute" => AuditAction::Execute,
            "login" => AuditAction::Login,
            "logout" => AuditAction::Logout,
            "view" => AuditAction::View,
            "export" => AuditAction::Export,
            "import" => AuditAction::Import,
            _ => AuditAction::Custom(s),
        }
    }
}

/// Resource types that can be audited
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Flow,
    FlowVersion,
    FlowExecution,
    MCPTool,
    MCPToolVersion,
    LLMConfig,
    VectorConfig,
    User,
    Tenant,
    Session,
    Custom(String),
}

impl ResourceType {
    pub fn as_str(&self) -> &str {
        match self {
            ResourceType::Flow => "flow",
            ResourceType::FlowVersion => "flow_version",
            ResourceType::FlowExecution => "flow_execution",
            ResourceType::MCPTool => "mcp_tool",
            ResourceType::MCPToolVersion => "mcp_tool_version",
            ResourceType::LLMConfig => "llm_config",
            ResourceType::VectorConfig => "vector_config",
            ResourceType::User => "user",
            ResourceType::Tenant => "tenant",
            ResourceType::Session => "session",
            ResourceType::Custom(s) => s,
        }
    }
}

impl From<String> for ResourceType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "flow" => ResourceType::Flow,
            "flow_version" => ResourceType::FlowVersion,
            "flow_execution" => ResourceType::FlowExecution,
            "mcp_tool" => ResourceType::MCPTool,
            "mcp_tool_version" => ResourceType::MCPToolVersion,
            "llm_config" => ResourceType::LLMConfig,
            "vector_config" => ResourceType::VectorConfig,
            "user" => ResourceType::User,
            "tenant" => ResourceType::Tenant,
            "session" => ResourceType::Session,
            _ => ResourceType::Custom(s),
        }
    }
}

/// Audit log domain entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub resource_type: ResourceType,
    pub resource_id: Option<Uuid>,
    pub details: Option<Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    pub fn new(
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        action: AuditAction,
        resource_type: ResourceType,
        resource_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            action,
            resource_type,
            resource_id,
            details: None,
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
}

/// Audit context for capturing request metadata
#[derive(Debug, Clone, Default)]
pub struct AuditContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuditContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
}
