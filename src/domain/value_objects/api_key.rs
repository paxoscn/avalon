use crate::domain::value_objects::{AgentId, ConfigId, FlowId, MCPToolId};
use crate::error::PlatformError;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// API Key Token value object
/// Format: pk_<base64url-encoded-32-bytes>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct APIKeyToken(String);

impl APIKeyToken {
    const PREFIX: &'static str = "pk_";
    const TOKEN_BYTES: usize = 32;

    /// Generate a new cryptographically secure API key token
    pub fn generate() -> Result<Self, PlatformError> {
        let rng = SystemRandom::new();
        let mut token_bytes = [0u8; Self::TOKEN_BYTES];
        
        rng.fill(&mut token_bytes)
            .map_err(|e| PlatformError::InternalError(format!("Failed to generate random token: {:?}", e)))?;
        
        let encoded = URL_SAFE_NO_PAD.encode(token_bytes);
        let token = format!("{}{}", Self::PREFIX, encoded);
        
        Ok(APIKeyToken(token))
    }

    /// Create from an existing token string (for validation)
    pub fn from_string(token: String) -> Result<Self, PlatformError> {
        Self::validate_format(&token)?;
        Ok(APIKeyToken(token))
    }

    /// Validate token format
    pub fn validate_format(token: &str) -> Result<(), PlatformError> {
        if !token.starts_with(Self::PREFIX) {
            return Err(PlatformError::ValidationError(
                "Invalid API key format: must start with 'pk_'".to_string()
            ));
        }

        let encoded_part = &token[Self::PREFIX.len()..];
        
        // Validate base64url encoding
        match URL_SAFE_NO_PAD.decode(encoded_part) {
            Ok(decoded) => {
                if decoded.len() != Self::TOKEN_BYTES {
                    return Err(PlatformError::ValidationError(
                        format!("Invalid API key format: expected {} bytes, got {}", Self::TOKEN_BYTES, decoded.len())
                    ));
                }
            }
            Err(_) => {
                return Err(PlatformError::ValidationError(
                    "Invalid API key format: invalid base64url encoding".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Generate SHA-256 hash of the token for storage
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.0.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Get the token as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

/// Resource types that can be accessed via API keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Agent,
    Flow,
    McpTool,
    VectorStore,
}

impl ResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Agent => "agent",
            ResourceType::Flow => "flow",
            ResourceType::McpTool => "mcp_tool",
            ResourceType::VectorStore => "vector_store",
        }
    }
}

/// Permission scope defining which resources an API key can access
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionScope {
    #[serde(default)]
    pub agent_ids: Vec<Uuid>,
    #[serde(default)]
    pub flow_ids: Vec<Uuid>,
    #[serde(default)]
    pub mcp_tool_ids: Vec<Uuid>,
    #[serde(default)]
    pub vector_store_ids: Vec<Uuid>,
}

impl PermissionScope {
    /// Create a new permission scope
    pub fn new(
        agent_ids: Vec<Uuid>,
        flow_ids: Vec<Uuid>,
        mcp_tool_ids: Vec<Uuid>,
        vector_store_ids: Vec<Uuid>,
    ) -> Self {
        Self {
            agent_ids,
            flow_ids,
            mcp_tool_ids,
            vector_store_ids,
        }
    }

    /// Create an empty permission scope (no access to any resources)
    pub fn empty() -> Self {
        Self {
            agent_ids: Vec::new(),
            flow_ids: Vec::new(),
            mcp_tool_ids: Vec::new(),
            vector_store_ids: Vec::new(),
        }
    }

    /// Check if the scope grants access to a specific agent
    pub fn can_access_agent(&self, agent_id: &AgentId) -> bool {
        self.agent_ids.contains(&agent_id.0)
    }

    /// Check if the scope grants access to a specific flow
    pub fn can_access_flow(&self, flow_id: &FlowId) -> bool {
        self.flow_ids.contains(&flow_id.0)
    }

    /// Check if the scope grants access to a specific MCP tool
    pub fn can_access_mcp_tool(&self, mcp_tool_id: &MCPToolId) -> bool {
        self.mcp_tool_ids.contains(&mcp_tool_id.0)
    }

    /// Check if the scope grants access to a specific vector store
    pub fn can_access_vector_store(&self, vector_store_id: &ConfigId) -> bool {
        self.vector_store_ids.contains(&vector_store_id.0)
    }

    /// Check if the scope grants access to a resource by type and ID
    pub fn can_access_resource(&self, resource_type: ResourceType, resource_id: Uuid) -> bool {
        match resource_type {
            ResourceType::Agent => self.agent_ids.contains(&resource_id),
            ResourceType::Flow => self.flow_ids.contains(&resource_id),
            ResourceType::McpTool => self.mcp_tool_ids.contains(&resource_id),
            ResourceType::VectorStore => self.vector_store_ids.contains(&resource_id),
        }
    }

    /// Check if the scope is empty (grants no permissions)
    pub fn is_empty(&self) -> bool {
        self.agent_ids.is_empty()
            && self.flow_ids.is_empty()
            && self.mcp_tool_ids.is_empty()
            && self.vector_store_ids.is_empty()
    }

    /// Merge another permission scope into this one
    pub fn merge(&mut self, other: &PermissionScope) {
        for id in &other.agent_ids {
            if !self.agent_ids.contains(id) {
                self.agent_ids.push(*id);
            }
        }
        for id in &other.flow_ids {
            if !self.flow_ids.contains(id) {
                self.flow_ids.push(*id);
            }
        }
        for id in &other.mcp_tool_ids {
            if !self.mcp_tool_ids.contains(id) {
                self.mcp_tool_ids.push(*id);
            }
        }
        for id in &other.vector_store_ids {
            if !self.vector_store_ids.contains(id) {
                self.vector_store_ids.push(*id);
            }
        }
    }

    /// Validate the permission scope
    pub fn validate(&self) -> Result<(), PlatformError> {
        // For now, just ensure it's not completely empty
        // Future: could add limits on number of resources per type
        Ok(())
    }
}

impl Default for PermissionScope {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_token_generation() {
        let token1 = APIKeyToken::generate().unwrap();
        let token2 = APIKeyToken::generate().unwrap();
        
        // Tokens should be different
        assert_ne!(token1.as_str(), token2.as_str());
        
        // Tokens should have correct prefix
        assert!(token1.as_str().starts_with("pk_"));
        assert!(token2.as_str().starts_with("pk_"));
    }

    #[test]
    fn test_api_key_token_validation() {
        let token = APIKeyToken::generate().unwrap();
        
        // Valid token should pass validation
        assert!(APIKeyToken::validate_format(token.as_str()).is_ok());
        
        // Invalid tokens should fail
        assert!(APIKeyToken::validate_format("invalid").is_err());
        assert!(APIKeyToken::validate_format("sk_invalid").is_err());
        assert!(APIKeyToken::validate_format("pk_").is_err());
    }

    #[test]
    fn test_api_key_token_hash() {
        let token = APIKeyToken::generate().unwrap();
        let hash1 = token.hash();
        let hash2 = token.hash();
        
        // Same token should produce same hash
        assert_eq!(hash1, hash2);
        
        // Hash should be 64 characters (SHA-256 hex)
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_permission_scope_empty() {
        let scope = PermissionScope::empty();
        assert!(scope.is_empty());
        
        let agent_id = AgentId::new();
        assert!(!scope.can_access_agent(&agent_id));
    }

    #[test]
    fn test_permission_scope_access() {
        let agent_id = AgentId::new();
        let flow_id = FlowId::new();
        
        let scope = PermissionScope::new(
            vec![agent_id.0],
            vec![flow_id.0],
            vec![],
            vec![],
        );
        
        assert!(!scope.is_empty());
        assert!(scope.can_access_agent(&agent_id));
        assert!(scope.can_access_flow(&flow_id));
        
        let other_agent_id = AgentId::new();
        assert!(!scope.can_access_agent(&other_agent_id));
    }

    #[test]
    fn test_permission_scope_merge() {
        let agent_id1 = AgentId::new();
        let agent_id2 = AgentId::new();
        
        let mut scope1 = PermissionScope::new(vec![agent_id1.0], vec![], vec![], vec![]);
        let scope2 = PermissionScope::new(vec![agent_id2.0], vec![], vec![], vec![]);
        
        scope1.merge(&scope2);
        
        assert!(scope1.can_access_agent(&agent_id1));
        assert!(scope1.can_access_agent(&agent_id2));
    }

    #[test]
    fn test_resource_type_as_str() {
        assert_eq!(ResourceType::Agent.as_str(), "agent");
        assert_eq!(ResourceType::Flow.as_str(), "flow");
        assert_eq!(ResourceType::McpTool.as_str(), "mcp_tool");
        assert_eq!(ResourceType::VectorStore.as_str(), "vector_store");
    }
}
