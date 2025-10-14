use crate::domain::{
    entities::{Tenant, User},
    events::{
        PasswordChangedEvent, TokenRefreshedEvent, UserAuthenticatedEvent,
        UserAuthenticationFailedEvent, UserLoggedOutEvent,
    },
    value_objects::{
        HashedPassword, JwtToken, LoginCredentials, Password, SessionInfo, TenantId, TokenClaims,
        UserId, Username,
    },
};
use crate::error::PlatformError;
use async_trait::async_trait;
use chrono::Duration;
use uuid::Uuid;

/// Authentication domain service interface
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthenticationDomainService: Send + Sync {
    /// Hash a password using bcrypt
    async fn hash_password(&self, password: &Password) -> Result<HashedPassword, PlatformError>;

    /// Verify a password against its hash
    async fn verify_password(
        &self,
        password: &Password,
        hash: &HashedPassword,
    ) -> Result<bool, PlatformError>;

    /// Generate a JWT token for a user
    async fn generate_token(
        &self,
        user: &User,
        expires_in: Duration,
    ) -> Result<JwtToken, PlatformError>;

    /// Validate and decode a JWT token
    async fn validate_token(&self, token: &JwtToken) -> Result<TokenClaims, PlatformError>;

    /// Refresh a JWT token
    async fn refresh_token(&self, token: &JwtToken) -> Result<JwtToken, PlatformError>;

    /// Revoke a JWT token
    async fn revoke_token(&self, token: &JwtToken) -> Result<(), PlatformError>;

    /// Check if a token is revoked
    async fn is_token_revoked(&self, token_id: Uuid) -> Result<bool, PlatformError>;

    /// Authenticate user with credentials
    async fn authenticate_user(
        &self,
        credentials: &LoginCredentials,
        user: &User,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(SessionInfo, UserAuthenticatedEvent), PlatformError>;

    /// Handle authentication failure
    fn create_authentication_failed_event(
        &self,
        credentials: &LoginCredentials,
        reason: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> UserAuthenticationFailedEvent;

    /// Create logout event
    fn create_logout_event(
        &self,
        user: &User,
        token_id: Uuid,
        ip_address: Option<String>,
    ) -> UserLoggedOutEvent;

    /// Create token refresh event
    fn create_token_refresh_event(
        &self,
        user: &User,
        old_token_id: Uuid,
        new_token_id: Uuid,
        ip_address: Option<String>,
    ) -> TokenRefreshedEvent;

    /// Create password change event
    fn create_password_change_event(&self, user: &User, changed_by: UserId)
        -> PasswordChangedEvent;
}

/// Default implementation of authentication domain service
pub struct AuthenticationDomainServiceImpl {
    jwt_secret: String,
    bcrypt_cost: u32,
}

impl AuthenticationDomainServiceImpl {
    pub fn new(jwt_secret: String, bcrypt_cost: Option<u32>) -> Self {
        Self {
            jwt_secret,
            bcrypt_cost: bcrypt_cost.unwrap_or(12),
        }
    }
}

#[async_trait]
impl AuthenticationDomainService for AuthenticationDomainServiceImpl {
    async fn hash_password(&self, password: &Password) -> Result<HashedPassword, PlatformError> {
        let hash = bcrypt::hash(password.as_str(), self.bcrypt_cost)
            .map_err(|e| PlatformError::InternalError(format!("Failed to hash password: {}", e)))?;

        HashedPassword::new(hash).map_err(|e| PlatformError::ValidationError(e))
    }

    async fn verify_password(
        &self,
        password: &Password,
        hash: &HashedPassword,
    ) -> Result<bool, PlatformError> {
        // println!(
        //     "hashed {}",
        //     bcrypt::hash(password.as_str(), 12).unwrap().as_str()
        // );
        bcrypt::verify(password.as_str(), hash.as_str())
            .map_err(|e| PlatformError::InternalError(format!("Failed to verify password: {}", e)))
    }

    async fn generate_token(
        &self,
        user: &User,
        expires_in: Duration,
    ) -> Result<JwtToken, PlatformError> {
        let claims = TokenClaims::new(
            user.id.0,
            user.tenant_id.0,
            user.username.0.clone(),
            user.nickname.clone(),
            expires_in,
        );

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| PlatformError::InternalError(format!("Failed to generate token: {}", e)))?;

        JwtToken::new(token).map_err(|e| PlatformError::ValidationError(e))
    }

    async fn validate_token(&self, token: &JwtToken) -> Result<TokenClaims, PlatformError> {
        let token_data = jsonwebtoken::decode::<TokenClaims>(
            token.as_str(),
            &jsonwebtoken::DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|e| PlatformError::AuthenticationFailed(format!("Invalid token: {}", e)))?;

        let claims = token_data.claims;

        // Check if token is expired
        if claims.is_expired() {
            return Err(PlatformError::AuthenticationFailed(
                "Token has expired".to_string(),
            ));
        }

        // Check if token is revoked
        if self.is_token_revoked(claims.jti).await? {
            return Err(PlatformError::AuthenticationFailed(
                "Token has been revoked".to_string(),
            ));
        }

        Ok(claims)
    }

    async fn refresh_token(&self, token: &JwtToken) -> Result<JwtToken, PlatformError> {
        let claims = self.validate_token(token).await?;

        // Create new token with extended expiration
        let new_claims = TokenClaims::new(
            claims.sub,
            claims.tenant_id,
            claims.username,
            claims.nickname,
            Duration::hours(24), // Default 24 hours
        );

        let new_token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &new_claims,
            &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| PlatformError::InternalError(format!("Failed to refresh token: {}", e)))?;

        // Revoke old token
        self.revoke_token(token).await?;

        JwtToken::new(new_token).map_err(|e| PlatformError::ValidationError(e))
    }

    async fn revoke_token(&self, token: &JwtToken) -> Result<(), PlatformError> {
        // Extract token ID from claims
        let claims = jsonwebtoken::decode::<TokenClaims>(
            token.as_str(),
            &jsonwebtoken::DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|e| PlatformError::AuthenticationFailed(format!("Invalid token: {}", e)))?;

        // TODO: Store revoked token ID in Redis or database
        // For now, this is a placeholder implementation
        tracing::info!("Token {} revoked", claims.claims.jti);

        Ok(())
    }

    async fn is_token_revoked(&self, token_id: Uuid) -> Result<bool, PlatformError> {
        // TODO: Check if token ID is in revoked tokens store (Redis/database)
        // For now, return false (no tokens are revoked)
        Ok(false)
    }

    async fn authenticate_user(
        &self,
        credentials: &LoginCredentials,
        user: &User,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(SessionInfo, UserAuthenticatedEvent), PlatformError> {
        // Verify tenant matches
        if user.tenant_id.0 != credentials.tenant_id {
            return Err(PlatformError::AuthenticationFailed(
                "Invalid tenant".to_string(),
            ));
        }

        // Verify username matches
        if user.username.0 != credentials.username {
            return Err(PlatformError::AuthenticationFailed(
                "Invalid username".to_string(),
            ));
        }

        // Verify password
        let password = Password::new(credentials.password.clone())
            .map_err(|e| PlatformError::ValidationError(e))?;

        let hashed_password = HashedPassword::new(user.password_hash.clone())
            .map_err(|e| PlatformError::ValidationError(e))?;

        if !self.verify_password(&password, &hashed_password).await? {
            return Err(PlatformError::AuthenticationFailed(
                "Invalid password".to_string(),
            ));
        }

        // Generate token
        let token = self.generate_token(user, Duration::hours(24)).await?;

        // Extract token claims for session info
        let claims = self.validate_token(&token).await?;

        // Create session info
        let session_info = SessionInfo::new(
            user.id.0,
            user.tenant_id.0,
            user.username.0.clone(),
            user.nickname.clone(),
            token,
            claims.expires_at(),
        );

        // Create authentication event
        let auth_event = UserAuthenticatedEvent::new(
            user.id.0,
            user.tenant_id.0,
            user.username.0.clone(),
            ip_address,
            user_agent,
            1, // Version
        );

        Ok((session_info, auth_event))
    }

    fn create_authentication_failed_event(
        &self,
        credentials: &LoginCredentials,
        reason: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> UserAuthenticationFailedEvent {
        UserAuthenticationFailedEvent::new(
            credentials.tenant_id,
            credentials.username.clone(),
            reason,
            ip_address,
            user_agent,
            1, // Version
        )
    }

    fn create_logout_event(
        &self,
        user: &User,
        token_id: Uuid,
        ip_address: Option<String>,
    ) -> UserLoggedOutEvent {
        UserLoggedOutEvent::new(
            user.id.0,
            user.tenant_id.0,
            user.username.0.clone(),
            token_id,
            ip_address,
            1, // Version
        )
    }

    fn create_token_refresh_event(
        &self,
        user: &User,
        old_token_id: Uuid,
        new_token_id: Uuid,
        ip_address: Option<String>,
    ) -> TokenRefreshedEvent {
        TokenRefreshedEvent::new(
            user.id.0,
            user.tenant_id.0,
            old_token_id,
            new_token_id,
            ip_address,
            1, // Version
        )
    }

    fn create_password_change_event(
        &self,
        user: &User,
        changed_by: UserId,
    ) -> PasswordChangedEvent {
        PasswordChangedEvent::new(
            user.id.0,
            user.tenant_id.0,
            user.username.0.clone(),
            changed_by.0,
            1, // Version
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::TenantName;

    #[tokio::test]
    async fn test_password_hashing_and_verification() {
        let service = AuthenticationDomainServiceImpl::new("test_secret".to_string(), Some(4));
        let password = Password::new("test_password_123".to_string()).unwrap();

        let hash = service.hash_password(&password).await.unwrap();
        assert!(service.verify_password(&password, &hash).await.unwrap());

        let wrong_password = Password::new("wrong_password".to_string()).unwrap();
        assert!(!service
            .verify_password(&wrong_password, &hash)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_token_generation_and_validation() {
        let service = AuthenticationDomainServiceImpl::new("test_secret".to_string(), Some(4));

        let user = User::new(
            UserId::new(),
            TenantId::new(),
            Username::new("testuser".to_string()).unwrap(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        )
        .unwrap();

        let token = service
            .generate_token(&user, Duration::hours(1))
            .await
            .unwrap();
        let claims = service.validate_token(&token).await.unwrap();

        assert_eq!(claims.sub, user.id.0);
        assert_eq!(claims.tenant_id, user.tenant_id.0);
        assert_eq!(claims.username, user.username.0);
        assert_eq!(claims.nickname, user.nickname);
    }

    #[tokio::test]
    async fn test_authentication_success() {
        let service = AuthenticationDomainServiceImpl::new("test_secret".to_string(), Some(4));

        let password = Password::new("test_password_123".to_string()).unwrap();
        let hash = service.hash_password(&password).await.unwrap();

        let tenant_id = TenantId::new();
        let user = User::new(
            UserId::new(),
            tenant_id,
            Username::new("testuser".to_string()).unwrap(),
            hash.0,
            Some("Test User".to_string()),
        )
        .unwrap();

        let credentials = LoginCredentials::new(
            tenant_id.0,
            "testuser".to_string(),
            "test_password_123".to_string(),
        )
        .unwrap();

        let result = service
            .authenticate_user(
                &credentials,
                &user,
                Some("127.0.0.1".to_string()),
                Some("test-agent".to_string()),
            )
            .await;

        assert!(result.is_ok());
        let (session_info, auth_event) = result.unwrap();
        assert_eq!(session_info.user_id, user.id.0);
        assert_eq!(auth_event.user_id, user.id.0);
    }

    #[tokio::test]
    async fn test_authentication_failure_wrong_password() {
        let service = AuthenticationDomainServiceImpl::new("test_secret".to_string(), Some(4));

        let password = Password::new("correct_password".to_string()).unwrap();
        let hash = service.hash_password(&password).await.unwrap();

        let tenant_id = TenantId::new();
        let user = User::new(
            UserId::new(),
            tenant_id,
            Username::new("testuser".to_string()).unwrap(),
            hash.0,
            Some("Test User".to_string()),
        )
        .unwrap();

        let credentials = LoginCredentials::new(
            tenant_id.0,
            "testuser".to_string(),
            "wrong_password".to_string(),
        )
        .unwrap();

        let result = service
            .authenticate_user(
                &credentials,
                &user,
                Some("127.0.0.1".to_string()),
                Some("test-agent".to_string()),
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::AuthenticationFailed(msg) => {
                assert_eq!(msg, "Invalid password");
            }
            _ => panic!("Expected AuthenticationFailed error"),
        }
    }
}
