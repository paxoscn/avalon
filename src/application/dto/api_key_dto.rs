use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::APIKey;
use crate::domain::value_objects::{APIKeyToken, PermissionScope};

/// DTO for permission scope
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionScopeDTO {
    #[serde(default)]
    pub agent_ids: Vec<Uuid>,
    #[serde(default)]
    pub flow_ids: Vec<Uuid>,
    #[serde(default)]
    pub mcp_tool_ids: Vec<Uuid>,
    #[serde(default)]
    pub vector_store_ids: Vec<Uuid>,
}

impl From<PermissionScope> for PermissionScopeDTO {
    fn from(scope: PermissionScope) -> Self {
        Self {
            agent_ids: scope.agent_ids,
            flow_ids: scope.flow_ids,
            mcp_tool_ids: scope.mcp_tool_ids,
            vector_store_ids: scope.vector_store_ids,
        }
    }
}

impl From<PermissionScopeDTO> for PermissionScope {
    fn from(dto: PermissionScopeDTO) -> Self {
        Self::new(
            dto.agent_ids,
            dto.flow_ids,
            dto.mcp_tool_ids,
            dto.vector_store_ids,
        )
    }
}

impl From<&PermissionScope> for PermissionScopeDTO {
    fn from(scope: &PermissionScope) -> Self {
        Self {
            agent_ids: scope.agent_ids.clone(),
            flow_ids: scope.flow_ids.clone(),
            mcp_tool_ids: scope.mcp_tool_ids.clone(),
            vector_store_ids: scope.vector_store_ids.clone(),
        }
    }
}

impl From<&PermissionScopeDTO> for PermissionScope {
    fn from(dto: &PermissionScopeDTO) -> Self {
        Self::new(
            dto.agent_ids.clone(),
            dto.flow_ids.clone(),
            dto.mcp_tool_ids.clone(),
            dto.vector_store_ids.clone(),
        )
    }
}

/// Request DTO for creating an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAPIKeyRequest {
    pub name: String,
    pub permission_scope: PermissionScopeDTO,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response DTO for creating an API key (includes token)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAPIKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub token: String,
    pub permission_scope: PermissionScopeDTO,
    pub enabled: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl CreateAPIKeyResponse {
    /// Create a response from an API key entity and token
    pub fn from_api_key_and_token(api_key: &APIKey, token: &APIKeyToken) -> Self {
        Self {
            id: api_key.id.0,
            name: api_key.name.clone(),
            token: token.as_str().to_string(),
            permission_scope: (&api_key.permission_scope).into(),
            enabled: api_key.enabled,
            expires_at: api_key.expires_at,
            created_at: api_key.created_at,
        }
    }
}

/// DTO for API key details (without token)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIKeyDTO {
    pub id: Uuid,
    pub name: String,
    pub permission_scope: PermissionScopeDTO,
    pub enabled: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<APIKey> for APIKeyDTO {
    fn from(api_key: APIKey) -> Self {
        Self {
            id: api_key.id.0,
            name: api_key.name,
            permission_scope: api_key.permission_scope.into(),
            enabled: api_key.enabled,
            expires_at: api_key.expires_at,
            last_used_at: api_key.last_used_at,
            created_at: api_key.created_at,
            updated_at: api_key.updated_at,
        }
    }
}

impl From<&APIKey> for APIKeyDTO {
    fn from(api_key: &APIKey) -> Self {
        Self {
            id: api_key.id.0,
            name: api_key.name.clone(),
            permission_scope: (&api_key.permission_scope).into(),
            enabled: api_key.enabled,
            expires_at: api_key.expires_at,
            last_used_at: api_key.last_used_at,
            created_at: api_key.created_at,
            updated_at: api_key.updated_at,
        }
    }
}

/// Request DTO for updating an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAPIKeyRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Auth context for API key authentication
#[derive(Debug, Clone)]
pub struct APIKeyAuthContext {
    pub api_key_id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub permission_scope: PermissionScopeDTO,
}

impl From<APIKey> for APIKeyAuthContext {
    fn from(api_key: APIKey) -> Self {
        Self {
            api_key_id: api_key.id.0,
            tenant_id: api_key.tenant_id.0,
            user_id: api_key.user_id.0,
            permission_scope: api_key.permission_scope.into(),
        }
    }
}

impl From<&APIKey> for APIKeyAuthContext {
    fn from(api_key: &APIKey) -> Self {
        Self {
            api_key_id: api_key.id.0,
            tenant_id: api_key.tenant_id.0,
            user_id: api_key.user_id.0,
            permission_scope: (&api_key.permission_scope).into(),
        }
    }
}

/// Query result for listing API keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIKeyListResponse {
    pub items: Vec<APIKeyDTO>,
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{TenantId, UserId};

    #[test]
    fn test_permission_scope_dto_conversion() {
        let agent_id = uuid::Uuid::new_v4();
        let flow_id = uuid::Uuid::new_v4();
        let mcp_tool_id = uuid::Uuid::new_v4();
        let vector_store_id = uuid::Uuid::new_v4();

        // Test PermissionScope -> PermissionScopeDTO
        let scope = PermissionScope::new(
            vec![agent_id],
            vec![flow_id],
            vec![mcp_tool_id],
            vec![vector_store_id],
        );

        let dto: PermissionScopeDTO = scope.clone().into();
        assert_eq!(dto.agent_ids, vec![agent_id]);
        assert_eq!(dto.flow_ids, vec![flow_id]);
        assert_eq!(dto.mcp_tool_ids, vec![mcp_tool_id]);
        assert_eq!(dto.vector_store_ids, vec![vector_store_id]);

        // Test PermissionScopeDTO -> PermissionScope
        let scope_back: PermissionScope = dto.into();
        assert_eq!(scope_back, scope);
    }

    #[test]
    fn test_permission_scope_dto_reference_conversion() {
        let agent_id = uuid::Uuid::new_v4();
        let scope = PermissionScope::new(vec![agent_id], vec![], vec![], vec![]);

        // Test &PermissionScope -> PermissionScopeDTO
        let dto: PermissionScopeDTO = (&scope).into();
        assert_eq!(dto.agent_ids, vec![agent_id]);

        // Test &PermissionScopeDTO -> PermissionScope
        let scope_back: PermissionScope = (&dto).into();
        assert_eq!(scope_back, scope);
    }

    #[test]
    fn test_api_key_dto_conversion() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        let permission_scope = PermissionScope::empty();

        let api_key = APIKey::new(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            key_hash,
            permission_scope,
            None,
        )
        .unwrap();

        // Test APIKey -> APIKeyDTO
        let dto: APIKeyDTO = api_key.clone().into();
        assert_eq!(dto.id, api_key.id.0);
        assert_eq!(dto.name, api_key.name);
        assert_eq!(dto.enabled, api_key.enabled);
        assert_eq!(dto.expires_at, api_key.expires_at);
        assert_eq!(dto.last_used_at, api_key.last_used_at);
        assert_eq!(dto.created_at, api_key.created_at);
        assert_eq!(dto.updated_at, api_key.updated_at);

        // Test &APIKey -> APIKeyDTO
        let dto_ref: APIKeyDTO = (&api_key).into();
        assert_eq!(dto_ref.id, api_key.id.0);
        assert_eq!(dto_ref.name, api_key.name);
    }

    #[test]
    fn test_create_api_key_response_from_api_key_and_token() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        let permission_scope = PermissionScope::empty();

        let api_key = APIKey::new(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            key_hash,
            permission_scope,
            None,
        )
        .unwrap();

        let response = CreateAPIKeyResponse::from_api_key_and_token(&api_key, &token);

        assert_eq!(response.id, api_key.id.0);
        assert_eq!(response.name, api_key.name);
        assert_eq!(response.token, token.as_str());
        assert_eq!(response.enabled, api_key.enabled);
        assert_eq!(response.expires_at, api_key.expires_at);
        assert_eq!(response.created_at, api_key.created_at);
    }

    #[test]
    fn test_api_key_auth_context_conversion() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        let agent_id = uuid::Uuid::new_v4();
        let permission_scope = PermissionScope::new(vec![agent_id], vec![], vec![], vec![]);

        let api_key = APIKey::new(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            key_hash,
            permission_scope,
            None,
        )
        .unwrap();

        // Test APIKey -> APIKeyAuthContext
        let context: APIKeyAuthContext = api_key.clone().into();
        assert_eq!(context.api_key_id, api_key.id.0);
        assert_eq!(context.tenant_id, api_key.tenant_id.0);
        assert_eq!(context.user_id, api_key.user_id.0);
        assert_eq!(context.permission_scope.agent_ids, vec![agent_id]);

        // Test &APIKey -> APIKeyAuthContext
        let context_ref: APIKeyAuthContext = (&api_key).into();
        assert_eq!(context_ref.api_key_id, api_key.id.0);
        assert_eq!(context_ref.tenant_id, api_key.tenant_id.0);
        assert_eq!(context_ref.user_id, api_key.user_id.0);
    }

    #[test]
    fn test_permission_scope_dto_empty() {
        let scope = PermissionScope::empty();
        let dto: PermissionScopeDTO = scope.into();

        assert!(dto.agent_ids.is_empty());
        assert!(dto.flow_ids.is_empty());
        assert!(dto.mcp_tool_ids.is_empty());
        assert!(dto.vector_store_ids.is_empty());
    }

    #[test]
    fn test_permission_scope_dto_serialization() {
        let agent_id = uuid::Uuid::new_v4();
        let dto = PermissionScopeDTO {
            agent_ids: vec![agent_id],
            flow_ids: vec![],
            mcp_tool_ids: vec![],
            vector_store_ids: vec![],
        };

        // Test serialization
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains(&agent_id.to_string()));

        // Test deserialization
        let deserialized: PermissionScopeDTO = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.agent_ids, vec![agent_id]);
    }
}
