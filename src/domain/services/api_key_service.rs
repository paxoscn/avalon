use async_trait::async_trait;
use crate::domain::entities::APIKey;
use crate::domain::repositories::APIKeyRepository;
use crate::domain::value_objects::{APIKeyToken, PermissionScope, ResourceType, TenantId, UserId};
use crate::error::{PlatformError, Result};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Domain service trait for API key operations
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait APIKeyService: Send + Sync {
    /// Create a new API key with token generation
    async fn create_api_key(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        name: String,
        permission_scope: PermissionScope,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(APIKey, APIKeyToken)>;
    
    /// Validate an API key token and return the API key entity
    async fn validate_and_get_key(&self, token: &str) -> Result<APIKey>;
    
    /// Check if an API key has permission to access a specific resource
    async fn check_resource_permission(
        &self,
        api_key: &APIKey,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Result<bool>;
}

/// Implementation of the API key domain service
pub struct APIKeyDomainService {
    repository: Arc<dyn APIKeyRepository>,
}

impl APIKeyDomainService {
    /// Create a new API key domain service
    pub fn new(repository: Arc<dyn APIKeyRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl APIKeyService for APIKeyDomainService {
    async fn create_api_key(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        name: String,
        permission_scope: PermissionScope,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(APIKey, APIKeyToken)> {
        // Generate a cryptographically secure token
        let token = APIKeyToken::generate()?;
        
        // Hash the token for storage
        let key_hash = token.hash();
        
        // Create the API key entity
        let api_key = APIKey::new(
            tenant_id,
            user_id,
            name,
            key_hash,
            permission_scope,
            expires_at,
        )?;
        
        // Save to repository
        self.repository.save(&api_key).await?;
        
        // Return both the entity and the plain token (only time it's returned)
        Ok((api_key, token))
    }
    
    async fn validate_and_get_key(&self, token: &str) -> Result<APIKey> {
        // Validate token format
        APIKeyToken::validate_format(token)?;
        
        // Create token object and hash it
        let api_key_token = APIKeyToken::from_string(token.to_string())?;
        let key_hash = api_key_token.hash();
        
        // Look up the API key by hash
        let api_key = self.repository.find_by_key_hash(&key_hash).await?
            .ok_or_else(|| PlatformError::AuthenticationFailed("Invalid API key".to_string()))?;
        
        // Check if the key is enabled
        if !api_key.is_enabled() {
            return Err(PlatformError::AuthenticationFailed("API key is disabled".to_string()));
        }
        
        // Check if the key is expired
        if api_key.is_expired() {
            return Err(PlatformError::AuthenticationFailed("API key has expired".to_string()));
        }
        
        Ok(api_key)
    }
    
    async fn check_resource_permission(
        &self,
        api_key: &APIKey,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Result<bool> {
        // First verify the API key is valid
        if !api_key.is_valid() {
            return Ok(false);
        }
        
        // Check if the resource is in the permission scope
        let has_permission = api_key.can_access_resource(resource_type, resource_id);
        
        Ok(has_permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::MockAPIKeyRepository;
    use crate::domain::value_objects::{AgentId, FlowId};
    
    #[tokio::test]
    async fn test_create_api_key() {
        let mut mock_repo = MockAPIKeyRepository::new();
        
        // Expect save to be called once
        mock_repo
            .expect_save()
            .times(1)
            .returning(|_| Ok(()));
        
        let service = APIKeyDomainService::new(Arc::new(mock_repo));
        
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let permission_scope = PermissionScope::empty();
        
        let result = service.create_api_key(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            permission_scope,
            None,
        ).await;
        
        assert!(result.is_ok());
        let (api_key, token) = result.unwrap();
        
        // Verify API key properties
        assert_eq!(api_key.name, "Test API Key");
        assert_eq!(api_key.tenant_id, tenant_id);
        assert_eq!(api_key.user_id, user_id);
        assert!(api_key.is_enabled());
        
        // Verify token format
        assert!(token.as_str().starts_with("pk_"));
        
        // Verify hash matches
        assert_eq!(api_key.key_hash, token.hash());
    }
    
    #[tokio::test]
    async fn test_validate_and_get_key_success() {
        let mut mock_repo = MockAPIKeyRepository::new();
        
        // Create a test API key
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        let permission_scope = PermissionScope::empty();
        
        let api_key = APIKey::new(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            key_hash.clone(),
            permission_scope,
            None,
        ).unwrap();
        
        let api_key_clone = api_key.clone();
        
        // Expect find_by_key_hash to be called
        mock_repo
            .expect_find_by_key_hash()
            .times(1)
            .withf(move |hash| hash == &key_hash)
            .returning(move |_| Ok(Some(api_key_clone.clone())));
        
        let service = APIKeyDomainService::new(Arc::new(mock_repo));
        
        let result = service.validate_and_get_key(token.as_str()).await;
        
        assert!(result.is_ok());
        let validated_key = result.unwrap();
        assert_eq!(validated_key.id, api_key.id);
        assert_eq!(validated_key.name, "Test API Key");
    }
    
    #[tokio::test]
    async fn test_validate_and_get_key_invalid_token() {
        let mock_repo = MockAPIKeyRepository::new();
        let service = APIKeyDomainService::new(Arc::new(mock_repo));
        
        let result = service.validate_and_get_key("invalid_token").await;
        
        assert!(result.is_err());
        match result {
            Err(PlatformError::ValidationError(_)) => {},
            _ => panic!("Expected ValidationError"),
        }
    }
    
    #[tokio::test]
    async fn test_validate_and_get_key_not_found() {
        let mut mock_repo = MockAPIKeyRepository::new();
        
        let token = APIKeyToken::generate().unwrap();
        
        // Expect find_by_key_hash to return None
        mock_repo
            .expect_find_by_key_hash()
            .times(1)
            .returning(|_| Ok(None));
        
        let service = APIKeyDomainService::new(Arc::new(mock_repo));
        
        let result = service.validate_and_get_key(token.as_str()).await;
        
        assert!(result.is_err());
        match result {
            Err(PlatformError::AuthenticationFailed(msg)) => {
                assert_eq!(msg, "Invalid API key");
            },
            _ => panic!("Expected AuthenticationFailed"),
        }
    }
    
    #[tokio::test]
    async fn test_validate_and_get_key_disabled() {
        let mut mock_repo = MockAPIKeyRepository::new();
        
        // Create a disabled API key
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        let permission_scope = PermissionScope::empty();
        
        let mut api_key = APIKey::new(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            key_hash.clone(),
            permission_scope,
            None,
        ).unwrap();
        
        api_key.disable();
        
        let api_key_clone = api_key.clone();
        
        // Expect find_by_key_hash to return the disabled key
        mock_repo
            .expect_find_by_key_hash()
            .times(1)
            .withf(move |hash| hash == &key_hash)
            .returning(move |_| Ok(Some(api_key_clone.clone())));
        
        let service = APIKeyDomainService::new(Arc::new(mock_repo));
        
        let result = service.validate_and_get_key(token.as_str()).await;
        
        assert!(result.is_err());
        match result {
            Err(PlatformError::AuthenticationFailed(msg)) => {
                assert_eq!(msg, "API key is disabled");
            },
            _ => panic!("Expected AuthenticationFailed"),
        }
    }
    
    #[tokio::test]
    async fn test_validate_and_get_key_expired() {
        let mut mock_repo = MockAPIKeyRepository::new();
        
        // Create an expired API key
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        let permission_scope = PermissionScope::empty();
        
        // Set expiration to 1 day ago
        let expires_at = Utc::now() - chrono::Duration::days(1);
        
        // Create with future date first, then manually set to past
        let mut api_key = APIKey::new(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            key_hash.clone(),
            permission_scope,
            Some(Utc::now() + chrono::Duration::days(1)),
        ).unwrap();
        
        // Manually set expired date
        api_key.expires_at = Some(expires_at);
        
        let api_key_clone = api_key.clone();
        
        // Expect find_by_key_hash to return the expired key
        mock_repo
            .expect_find_by_key_hash()
            .times(1)
            .withf(move |hash| hash == &key_hash)
            .returning(move |_| Ok(Some(api_key_clone.clone())));
        
        let service = APIKeyDomainService::new(Arc::new(mock_repo));
        
        let result = service.validate_and_get_key(token.as_str()).await;
        
        assert!(result.is_err());
        match result {
            Err(PlatformError::AuthenticationFailed(msg)) => {
                assert_eq!(msg, "API key has expired");
            },
            _ => panic!("Expected AuthenticationFailed"),
        }
    }
    
    #[tokio::test]
    async fn test_check_resource_permission_allowed() {
        let mock_repo = MockAPIKeyRepository::new();
        let service = APIKeyDomainService::new(Arc::new(mock_repo));
        
        // Create API key with agent permission
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
        ).unwrap();
        
        let result = service.check_resource_permission(
            &api_key,
            ResourceType::Agent,
            agent_id.0,
        ).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
    
    #[tokio::test]
    async fn test_check_resource_permission_denied() {
        let mock_repo = MockAPIKeyRepository::new();
        let service = APIKeyDomainService::new(Arc::new(mock_repo));
        
        // Create API key with agent permission
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
        ).unwrap();
        
        // Try to access a different agent
        let other_agent_id = AgentId::new();
        let result = service.check_resource_permission(
            &api_key,
            ResourceType::Agent,
            other_agent_id.0,
        ).await;
        
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
    
    #[tokio::test]
    async fn test_check_resource_permission_different_resource_type() {
        let mock_repo = MockAPIKeyRepository::new();
        let service = APIKeyDomainService::new(Arc::new(mock_repo));
        
        // Create API key with agent permission only
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
        ).unwrap();
        
        // Try to access a flow (not in permission scope)
        let flow_id = FlowId::new();
        let result = service.check_resource_permission(
            &api_key,
            ResourceType::Flow,
            flow_id.0,
        ).await;
        
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
    
    #[tokio::test]
    async fn test_check_resource_permission_disabled_key() {
        let mock_repo = MockAPIKeyRepository::new();
        let service = APIKeyDomainService::new(Arc::new(mock_repo));
        
        // Create disabled API key
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let token = APIKeyToken::generate().unwrap();
        let key_hash = token.hash();
        
        let agent_id = AgentId::new();
        let permission_scope = PermissionScope::new(vec![agent_id.0], vec![], vec![], vec![]);
        
        let mut api_key = APIKey::new(
            tenant_id,
            user_id,
            "Test API Key".to_string(),
            key_hash,
            permission_scope,
            None,
        ).unwrap();
        
        api_key.disable();
        
        let result = service.check_resource_permission(
            &api_key,
            ResourceType::Agent,
            agent_id.0,
        ).await;
        
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
