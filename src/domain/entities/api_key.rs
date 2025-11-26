use crate::domain::value_objects::{APIKeyId, PermissionScope, ResourceType, TenantId, UserId};
use crate::error::PlatformError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// API Key entity for managing programmatic access to resources
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APIKey {
    pub id: APIKeyId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub name: String,
    pub key_hash: String,
    pub permission_scope: PermissionScope,
    pub enabled: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl APIKey {
    /// Create a new API key
    pub fn new(
        tenant_id: TenantId,
        user_id: UserId,
        name: String,
        key_hash: String,
        permission_scope: PermissionScope,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Self, PlatformError> {
        // Validate name
        if name.trim().is_empty() {
            return Err(PlatformError::ValidationError(
                "API key name cannot be empty".to_string()
            ));
        }
        if name.len() > 255 {
            return Err(PlatformError::ValidationError(
                "API key name cannot exceed 255 characters".to_string()
            ));
        }

        // Validate key hash
        if key_hash.is_empty() {
            return Err(PlatformError::ValidationError(
                "API key hash cannot be empty".to_string()
            ));
        }

        // Validate expiration date
        if let Some(expires) = expires_at {
            if expires <= Utc::now() {
                return Err(PlatformError::ValidationError(
                    "API key expiration date must be in the future".to_string()
                ));
            }
        }

        // Validate permission scope
        permission_scope.validate()?;

        let now = Utc::now();

        Ok(APIKey {
            id: APIKeyId::new(),
            tenant_id,
            user_id,
            name: name.trim().to_string(),
            key_hash,
            permission_scope,
            enabled: true,
            expires_at,
            last_used_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Check if the API key is valid (enabled and not expired)
    pub fn is_valid(&self) -> bool {
        self.is_enabled() && !self.is_expired()
    }

    /// Check if the API key is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if the API key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if the API key can access a specific resource
    pub fn can_access_resource(&self, resource_type: ResourceType, resource_id: Uuid) -> bool {
        if !self.is_valid() {
            return false;
        }
        self.permission_scope.can_access_resource(resource_type, resource_id)
    }

    /// Enable the API key
    pub fn enable(&mut self) {
        self.enabled = true;
        self.updated_at = Utc::now();
    }

    /// Disable the API key
    pub fn disable(&mut self) {
        self.enabled = false;
        self.updated_at = Utc::now();
    }

    /// Update the last used timestamp
    pub fn update_last_used(&mut self) {
        self.last_used_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Update the API key name
    pub fn update_name(&mut self, name: String) -> Result<(), PlatformError> {
        if name.trim().is_empty() {
            return Err(PlatformError::ValidationError(
                "API key name cannot be empty".to_string()
            ));
        }
        if name.len() > 255 {
            return Err(PlatformError::ValidationError(
                "API key name cannot exceed 255 characters".to_string()
            ));
        }

        self.name = name.trim().to_string();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update the expiration date
    pub fn update_expiration(&mut self, expires_at: Option<DateTime<Utc>>) -> Result<(), PlatformError> {
        if let Some(expires) = expires_at {
            if expires <= Utc::now() {
                return Err(PlatformError::ValidationError(
                    "API key expiration date must be in the future".to_string()
                ));
            }
        }

        self.expires_at = expires_at;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if the API key belongs to a specific tenant
    pub fn belongs_to_tenant(&self, tenant_id: &TenantId) -> bool {
        &self.tenant_id == tenant_id
    }

    /// Check if the API key belongs to a specific user
    pub fn belongs_to_user(&self, user_id: &UserId) -> bool {
        &self.user_id == user_id
    }

    /// Validate the API key entity
    pub fn validate(&self) -> Result<(), PlatformError> {
        // Validate name
        if self.name.trim().is_empty() {
            return Err(PlatformError::ValidationError(
                "API key name cannot be empty".to_string()
            ));
        }
        if self.name.len() > 255 {
            return Err(PlatformError::ValidationError(
                "API key name cannot exceed 255 characters".to_string()
            ));
        }

        // Validate key hash
        if self.key_hash.is_empty() {
            return Err(PlatformError::ValidationError(
                "API key hash cannot be empty".to_string()
            ));
        }

        // Validate permission scope
        self.permission_scope.validate()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{AgentId, APIKeyToken};

    #[test]
    fn test_api_key_creation() {
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
        );

        assert!(api_key.is_ok());
        let api_key = api_key.unwrap();
        assert_eq!(api_key.name, "Test API Key");
        assert!(api_key.is_enabled());
        assert!(!api_key.is_expired());
        assert!(api_key.is_valid());
    }

    #[test]
    fn test_api_key_validation() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        let permission_scope = PermissionScope::empty();

        // Empty name should fail
        let result = APIKey::new(
            tenant_id,
            user_id,
            "".to_string(),
            key_hash.clone(),
            permission_scope.clone(),
            None,
        );
        assert!(result.is_err());

        // Name too long should fail
        let long_name = "a".repeat(256);
        let result = APIKey::new(
            tenant_id,
            user_id,
            long_name,
            key_hash.clone(),
            permission_scope.clone(),
            None,
        );
        assert!(result.is_err());

        // Past expiration date should fail
        let past_date = Utc::now() - chrono::Duration::days(1);
        let result = APIKey::new(
            tenant_id,
            user_id,
            "Test".to_string(),
            key_hash,
            permission_scope,
            Some(past_date),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_api_key_enable_disable() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        let permission_scope = PermissionScope::empty();

        let mut api_key = APIKey::new(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            key_hash,
            permission_scope,
            None,
        )
        .unwrap();

        assert!(api_key.is_enabled());
        assert!(api_key.is_valid());

        api_key.disable();
        assert!(!api_key.is_enabled());
        assert!(!api_key.is_valid());

        api_key.enable();
        assert!(api_key.is_enabled());
        assert!(api_key.is_valid());
    }

    #[test]
    fn test_api_key_expiration() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        let permission_scope = PermissionScope::empty();

        // Create key with future expiration
        let future_date = Utc::now() + chrono::Duration::days(30);
        let api_key = APIKey::new(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            key_hash,
            permission_scope,
            Some(future_date),
        )
        .unwrap();

        assert!(!api_key.is_expired());
        assert!(api_key.is_valid());
    }

    #[test]
    fn test_api_key_resource_access() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        
        let agent_id = AgentId::new();
        let permission_scope = PermissionScope::new(vec![agent_id.0], vec![], vec![], vec![]);

        let api_key = APIKey::new(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            key_hash,
            permission_scope,
            None,
        )
        .unwrap();

        // Should have access to the specified agent
        assert!(api_key.can_access_resource(ResourceType::Agent, agent_id.0));

        // Should not have access to other agents
        let other_agent_id = AgentId::new();
        assert!(!api_key.can_access_resource(ResourceType::Agent, other_agent_id.0));

        // Should not have access to flows
        let flow_id = Uuid::new_v4();
        assert!(!api_key.can_access_resource(ResourceType::Flow, flow_id));
    }

    #[test]
    fn test_api_key_update_last_used() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        let permission_scope = PermissionScope::empty();

        let mut api_key = APIKey::new(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            key_hash,
            permission_scope,
            None,
        )
        .unwrap();

        assert!(api_key.last_used_at.is_none());

        api_key.update_last_used();
        assert!(api_key.last_used_at.is_some());
    }

    #[test]
    fn test_api_key_belongs_to() {
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

        assert!(api_key.belongs_to_tenant(&tenant_id));
        assert!(api_key.belongs_to_user(&user_id));

        let other_tenant_id = TenantId::new();
        let other_user_id = UserId::new();
        assert!(!api_key.belongs_to_tenant(&other_tenant_id));
        assert!(!api_key.belongs_to_user(&other_user_id));
    }
}
