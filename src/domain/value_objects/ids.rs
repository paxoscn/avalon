use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ID value objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlowId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MCPToolId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MCPToolVersionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlowExecutionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

// Implementations for ID value objects
impl UserId {
    pub fn new() -> Self {
        UserId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        UserId(uuid)
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        UserId(uuid)
    }
}

impl TenantId {
    pub fn new() -> Self {
        TenantId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        TenantId(uuid)
    }
}

impl From<Uuid> for TenantId {
    fn from(uuid: Uuid) -> Self {
        TenantId(uuid)
    }
}

impl FlowId {
    pub fn new() -> Self {
        FlowId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        FlowId(uuid)
    }
}

impl SessionId {
    pub fn new() -> Self {
        SessionId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        SessionId(uuid)
    }
}

impl MCPToolId {
    pub fn new() -> Self {
        MCPToolId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        MCPToolId(uuid)
    }
}

impl MCPToolVersionId {
    pub fn new() -> Self {
        MCPToolVersionId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        MCPToolVersionId(uuid)
    }
}

impl FlowExecutionId {
    pub fn new() -> Self {
        FlowExecutionId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        FlowExecutionId(uuid)
    }
}

impl MessageId {
    pub fn new() -> Self {
        MessageId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        MessageId(uuid)
    }
}

impl ConfigId {
    pub fn new() -> Self {
        ConfigId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        ConfigId(uuid)
    }
    
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        let uuid = Uuid::parse_str(s)?;
        Ok(ConfigId(uuid))
    }
}

impl AgentId {
    pub fn new() -> Self {
        AgentId(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        AgentId(uuid)
    }
}

impl From<Uuid> for AgentId {
    fn from(uuid: Uuid) -> Self {
        AgentId(uuid)
    }
}

// Display implementations for all ID types
impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for FlowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for MCPToolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for MCPToolVersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for FlowExecutionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for ConfigId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}