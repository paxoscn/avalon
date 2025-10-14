use async_trait::async_trait;
use chrono::Duration;
use uuid::Uuid;
use std::sync::Arc;

use crate::{
    domain::{
        entities::{User, Tenant},
        repositories::{UserRepository, TenantRepository},
        services::AuthenticationDomainService,
        value_objects::{LoginCredentials, Password, JwtToken, TokenClaims},
        events::{
            UserAuthenticatedEvent, UserAuthenticationFailedEvent, 
            UserLoggedOutEvent, TokenRefreshedEvent, PasswordChangedEvent
        },
    },
    application::dto::{
        LoginRequest, LoginResponse, UserInfo, RefreshTokenRequest, 
        RefreshTokenResponse, LogoutRequest, LogoutResponse,
        ChangePasswordRequest, ChangePasswordResponse, AuthContext, TenantContext
    },
    error::{PlatformError, Result},
};

/// Authentication application service interface
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthApplicationService: Send + Sync {
    /// Authenticate user and return session information
    async fn login(
        &self,
        request: LoginRequest,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(LoginResponse, UserAuthenticatedEvent)>;

    /// Refresh authentication token
    async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
        ip_address: Option<String>,
    ) -> Result<(RefreshTokenResponse, TokenRefreshedEvent)>;

    /// Logout user and revoke token
    async fn logout(
        &self,
        request: LogoutRequest,
        ip_address: Option<String>,
    ) -> Result<(LogoutResponse, UserLoggedOutEvent)>;

    /// Change user password
    async fn change_password(
        &self,
        request: ChangePasswordRequest,
        auth_context: AuthContext,
    ) -> Result<(ChangePasswordResponse, PasswordChangedEvent)>;

    /// Validate token and return authentication context
    async fn validate_token(&self, token: &str) -> Result<AuthContext>;

    /// Get tenant context for authenticated user
    async fn get_tenant_context(&self, auth_context: &AuthContext) -> Result<TenantContext>;

    /// Check if user has permission for tenant operation
    async fn check_tenant_permission(
        &self,
        auth_context: &AuthContext,
        tenant_id: &Uuid,
        permission: &str,
    ) -> Result<bool>;
}

/// Default implementation of authentication application service
pub struct AuthApplicationServiceImpl {
    user_repository: Arc<dyn UserRepository>,
    tenant_repository: Arc<dyn TenantRepository>,
    auth_domain_service: Arc<dyn AuthenticationDomainService>,
    default_token_expiry: Duration,
}

impl AuthApplicationServiceImpl {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        tenant_repository: Arc<dyn TenantRepository>,
        auth_domain_service: Arc<dyn AuthenticationDomainService>,
        default_token_expiry: Option<Duration>,
    ) -> Self {
        Self {
            user_repository,
            tenant_repository,
            auth_domain_service,
            default_token_expiry: default_token_expiry.unwrap_or(Duration::hours(24)),
        }
    }
}

#[async_trait]
impl AuthApplicationService for AuthApplicationServiceImpl {
    async fn login(
        &self,
        request: LoginRequest,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(LoginResponse, UserAuthenticatedEvent)> {
        // Validate request
        let credentials = LoginCredentials::new(
            request.tenant_id,
            request.username.clone(),
            request.password.clone(),
        )
        .map_err(|e| PlatformError::ValidationError(e))?;

        // Check if tenant exists
        let tenant = self.tenant_repository
            .find_by_id(request.tenant_id.into())
            .await?
            .ok_or_else(|| PlatformError::AuthenticationFailed("Invalid tenant".to_string()))?;

        // Find user by tenant and username
        let user = self.user_repository
            .find_by_tenant_and_username(request.tenant_id.into(), &request.username)
            .await?
            .ok_or_else(|| PlatformError::AuthenticationFailed("Invalid credentials".to_string()))?;

        // Authenticate user using domain service
        let (session_info, auth_event) = self.auth_domain_service
            .authenticate_user(&credentials, &user, ip_address, user_agent)
            .await?;

        // Create response
        let user_info = UserInfo {
            id: user.id.0,
            tenant_id: user.tenant_id.0,
            username: user.username.0.clone(),
            nickname: user.nickname.clone(),
            created_at: user.created_at,
        };

        let response = LoginResponse {
            token: session_info.token.0,
            user: user_info,
            expires_at: session_info.expires_at,
        };

        Ok((response, auth_event))
    }

    async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
        ip_address: Option<String>,
    ) -> Result<(RefreshTokenResponse, TokenRefreshedEvent)> {
        // Validate current token
        let current_token = JwtToken::new(request.token)
            .map_err(|e| PlatformError::ValidationError(e))?;

        let current_claims = self.auth_domain_service
            .validate_token(&current_token)
            .await?;

        // Find user to ensure they still exist
        let user = self.user_repository
            .find_by_id(current_claims.sub.into())
            .await?
            .ok_or_else(|| PlatformError::AuthenticationFailed("User not found".to_string()))?;

        // Generate new token
        let new_token = self.auth_domain_service
            .refresh_token(&current_token)
            .await?;

        // Get new token claims
        let new_claims = self.auth_domain_service
            .validate_token(&new_token)
            .await?;

        // Create refresh event
        let refresh_event = self.auth_domain_service
            .create_token_refresh_event(
                &user,
                current_claims.jti,
                new_claims.jti,
                ip_address,
            );

        let response = RefreshTokenResponse {
            token: new_token.0,
            expires_at: new_claims.expires_at(),
        };

        Ok((response, refresh_event))
    }

    async fn logout(
        &self,
        request: LogoutRequest,
        ip_address: Option<String>,
    ) -> Result<(LogoutResponse, UserLoggedOutEvent)> {
        // Validate token
        let token = JwtToken::new(request.token)
            .map_err(|e| PlatformError::ValidationError(e))?;

        let claims = self.auth_domain_service
            .validate_token(&token)
            .await?;

        // Find user
        let user = self.user_repository
            .find_by_id(claims.sub.into())
            .await?
            .ok_or_else(|| PlatformError::AuthenticationFailed("User not found".to_string()))?;

        // Revoke token
        self.auth_domain_service
            .revoke_token(&token)
            .await?;

        // Create logout event
        let logout_event = self.auth_domain_service
            .create_logout_event(&user, claims.jti, ip_address);

        let response = LogoutResponse {
            success: true,
            message: "Successfully logged out".to_string(),
        };

        Ok((response, logout_event))
    }

    async fn change_password(
        &self,
        request: ChangePasswordRequest,
        auth_context: AuthContext,
    ) -> Result<(ChangePasswordResponse, PasswordChangedEvent)> {
        // Find user
        let mut user = self.user_repository
            .find_by_id(auth_context.user_id.into())
            .await?
            .ok_or_else(|| PlatformError::NotFound("User not found".to_string()))?;

        // Validate current password
        let current_password = Password::new(request.current_password)
            .map_err(|e| PlatformError::ValidationError(e))?;

        let current_hash = crate::domain::value_objects::HashedPassword::new(user.password_hash.clone())
            .map_err(|e| PlatformError::ValidationError(e))?;

        if !self.auth_domain_service
            .verify_password(&current_password, &current_hash)
            .await? {
            return Err(PlatformError::AuthenticationFailed("Current password is incorrect".to_string()));
        }

        // Hash new password
        let new_password = Password::new(request.new_password)
            .map_err(|e| PlatformError::ValidationError(e))?;

        let new_hash = self.auth_domain_service
            .hash_password(&new_password)
            .await?;

        // Update user password
        user.update_password(new_hash.0)?;

        // Save user
        self.user_repository.save(&user).await?;

        // Create password change event
        let password_change_event = self.auth_domain_service
            .create_password_change_event(&user, auth_context.user_id.into());

        let response = ChangePasswordResponse {
            success: true,
            message: "Password changed successfully".to_string(),
        };

        Ok((response, password_change_event))
    }

    async fn validate_token(&self, token: &str) -> Result<AuthContext> {
        let jwt_token = JwtToken::new(token.to_string())
            .map_err(|e| PlatformError::ValidationError(e))?;

        let claims = self.auth_domain_service
            .validate_token(&jwt_token)
            .await?;

        // Verify user still exists
        let user = self.user_repository
            .find_by_id(claims.sub.into())
            .await?
            .ok_or_else(|| PlatformError::AuthenticationFailed("User not found".to_string()))?;

        Ok(AuthContext::new(
            user.id.0,
            user.tenant_id.0,
            user.username.0,
            user.nickname,
            claims.jti,
            None, // IP address not available from token
            None, // User agent not available from token
        ))
    }

    async fn get_tenant_context(&self, auth_context: &AuthContext) -> Result<TenantContext> {
        // Find tenant
        let tenant = self.tenant_repository
            .find_by_id(auth_context.tenant_id.into())
            .await?
            .ok_or_else(|| PlatformError::NotFound("Tenant not found".to_string()))?;

        // For now, all authenticated users have basic permissions
        // This can be extended with a proper permission system later
        let permissions = vec![
            "read".to_string(),
            "write".to_string(),
            "execute".to_string(),
        ];

        Ok(TenantContext::new(
            tenant.id.0,
            tenant.name.0,
            auth_context.user_id,
            auth_context.username.clone(),
            permissions,
        ))
    }

    async fn check_tenant_permission(
        &self,
        auth_context: &AuthContext,
        tenant_id: &Uuid,
        permission: &str,
    ) -> Result<bool> {
        // Check if user belongs to the tenant
        if !auth_context.belongs_to_tenant(tenant_id) {
            return Ok(false);
        }

        // Get tenant context
        let tenant_context = self.get_tenant_context(auth_context).await?;

        // Check permission
        Ok(tenant_context.has_permission(permission))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use crate::domain::{
        repositories::{MockUserRepository, MockTenantRepository},
        services::MockAuthenticationDomainService,
        value_objects::{UserId, TenantId, Username, TenantName, SessionInfo},
    };

    fn create_test_user() -> User {
        User::new(
            UserId::new(),
            TenantId::new(),
            Username::new("testuser".to_string()).unwrap(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        ).unwrap()
    }

    fn create_test_tenant() -> Tenant {
        Tenant::new(TenantName::new("Test Tenant".to_string()).unwrap())
    }

    #[tokio::test]
    async fn test_login_success() {
        let mut user_repo = MockUserRepository::new();
        let mut tenant_repo = MockTenantRepository::new();
        let mut auth_service = MockAuthenticationDomainService::new();

        let user = create_test_user();
        let tenant = create_test_tenant();
        let user_id = user.id;
        let tenant_id = tenant.id;

        // Setup mocks
        tenant_repo
            .expect_find_by_id()
            .with(eq(tenant_id))
            .times(1)
            .returning(move |_| Ok(Some(tenant.clone())));

        user_repo
            .expect_find_by_tenant_and_username()
            .with(eq(tenant_id), eq("testuser"))
            .times(1)
            .returning(move |_, _| Ok(Some(user.clone())));

        auth_service
            .expect_authenticate_user()
            .times(1)
            .returning(move |_, _, _, _| {
                let session_info = SessionInfo::new(
                    user_id.0,
                    tenant_id.0,
                    "testuser".to_string(),
                    Some("Test User".to_string()),
                    JwtToken::new("test_token".to_string()).unwrap(),
                    chrono::Utc::now() + Duration::hours(24),
                );
                let auth_event = UserAuthenticatedEvent::new(
                    user_id.0,
                    tenant_id.0,
                    "testuser".to_string(),
                    None,
                    None,
                    1,
                );
                Ok((session_info, auth_event))
            });

        let service = AuthApplicationServiceImpl::new(
            Arc::new(user_repo),
            Arc::new(tenant_repo),
            Arc::new(auth_service),
            None,
        );

        let request = LoginRequest {
            tenant_id: tenant_id.0,
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };

        let result = service.login(request, None, None).await;
        assert!(result.is_ok());

        let (response, event) = result.unwrap();
        assert_eq!(response.token, "test_token");
        assert_eq!(response.user.username, "testuser");
        assert_eq!(event.username, "testuser");
    }

    #[tokio::test]
    async fn test_login_invalid_tenant() {
        let mut user_repo = MockUserRepository::new();
        let mut tenant_repo = MockTenantRepository::new();
        let auth_service = MockAuthenticationDomainService::new();

        let tenant_id = TenantId::new();

        // Setup mocks - tenant not found
        tenant_repo
            .expect_find_by_id()
            .with(eq(tenant_id))
            .times(1)
            .returning(|_| Ok(None));

        let service = AuthApplicationServiceImpl::new(
            Arc::new(user_repo),
            Arc::new(tenant_repo),
            Arc::new(auth_service),
            None,
        );

        let request = LoginRequest {
            tenant_id: tenant_id.0,
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };

        let result = service.login(request, None, None).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            PlatformError::AuthenticationFailed(msg) => {
                assert_eq!(msg, "Invalid tenant");
            }
            _ => panic!("Expected AuthenticationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_validate_token_success() {
        let user_repo = MockUserRepository::new();
        let tenant_repo = MockTenantRepository::new();
        let mut auth_service = MockAuthenticationDomainService::new();

        let user = create_test_user();
        let user_id = user.id;
        let tenant_id = user.tenant_id;

        let claims = TokenClaims::new(
            user_id.0,
            tenant_id.0,
            "testuser".to_string(),
            Some("Test User".to_string()),
            Duration::hours(1),
        );

        auth_service
            .expect_validate_token()
            .times(1)
            .returning(move |_| Ok(claims.clone()));

        let mut user_repo_mock = MockUserRepository::new();
        user_repo_mock
            .expect_find_by_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(Some(user.clone())));

        let service = AuthApplicationServiceImpl::new(
            Arc::new(user_repo_mock),
            Arc::new(tenant_repo),
            Arc::new(auth_service),
            None,
        );

        let result = service.validate_token("test_token").await;
        assert!(result.is_ok());

        let auth_context = result.unwrap();
        assert_eq!(auth_context.user_id, user_id.0);
        assert_eq!(auth_context.tenant_id, tenant_id.0);
        assert_eq!(auth_context.username, "testuser");
    }
}