pub mod ids;
pub mod flow_definition;
pub mod model_config;
pub mod chat_message;
pub mod tool_config;
pub mod auth;
pub mod vector_storage;

pub use ids::*;
pub use flow_definition::*;
pub use model_config::*;
pub use chat_message::*;
pub use tool_config::*;
pub use auth::*;
pub use vector_storage::*;

use serde::{Deserialize, Serialize};

// Name value objects
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Username(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantName(pub String);

// Version value object
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version(pub i32);

impl Version {
    pub fn new() -> Self {
        Version(1)
    }
    
    pub fn initial() -> Self {
        Version(1)
    }
    
    pub fn next(&self) -> Self {
        Version(self.0 + 1)
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new()
    }
}

// Validation for name value objects
impl FlowName {
    pub fn new(name: String) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Flow name cannot be empty".to_string());
        }
        if name.len() > 255 {
            return Err("Flow name cannot exceed 255 characters".to_string());
        }
        Ok(FlowName(name.trim().to_string()))
    }
}

impl Username {
    pub fn new(username: String) -> Result<Self, String> {
        if username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if username.len() > 255 {
            return Err("Username cannot exceed 255 characters".to_string());
        }
        Ok(Username(username.trim().to_string()))
    }
}

impl TenantName {
    pub fn new(name: String) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Tenant name cannot be empty".to_string());
        }
        if name.len() > 255 {
            return Err("Tenant name cannot exceed 255 characters".to_string());
        }
        Ok(TenantName(name.trim().to_string()))
    }
}