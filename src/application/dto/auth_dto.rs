use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Login request DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginRequest {
    pub tenant_id: Uuid,
    pub username: String,
    pub password: String,
}

/// Login response DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
    pub expires_at: DateTime<Utc>,
}

/// User information DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Token refresh request DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub token: String,
}

/// Token refresh response DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Logout request DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub token: String,
}

/// Logout response DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

/// Change password request DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Change password response DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangePasswordResponse {
    pub success: bool,
    pub message: String,
}

/// Authentication context for requests
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
    pub token_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuthContext {
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        username: String,
        nickname: Option<String>,
        token_id: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            user_id,
            tenant_id,
            username,
            nickname,
            token_id,
            ip_address,
            user_agent,
        }
    }

    pub fn belongs_to_tenant(&self, tenant_id: &Uuid) -> bool {
        &self.tenant_id == tenant_id
    }

    pub fn is_user(&self, user_id: &Uuid) -> bool {
        &self.user_id == user_id
    }
}

/// Tenant context for multi-tenant operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub user_id: Uuid,
    pub username: String,
    pub permissions: Vec<String>,
}

impl TenantContext {
    pub fn new(
        tenant_id: Uuid,
        tenant_name: String,
        user_id: Uuid,
        username: String,
        permissions: Vec<String>,
    ) -> Self {
        Self {
            tenant_id,
            tenant_name,
            user_id,
            username,
            permissions,
        }
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    pub fn add_permission(&mut self, permission: String) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
    }

    pub fn remove_permission(&mut self, permission: &str) {
        self.permissions.retain(|p| p != permission);
    }
}