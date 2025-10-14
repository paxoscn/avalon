use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

/// Password value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password(pub String);

impl Password {
    pub fn new(password: String) -> Result<Self, String> {
        if password.len() < 8 {
            return Err("Password must be at least 8 characters long".to_string());
        }
        if password.len() > 128 {
            return Err("Password cannot exceed 128 characters".to_string());
        }
        Ok(Password(password))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Hashed password value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashedPassword(pub String);

impl HashedPassword {
    pub fn new(hash: String) -> Result<Self, String> {
        if hash.trim().is_empty() {
            return Err("Password hash cannot be empty".to_string());
        }
        Ok(HashedPassword(hash))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// JWT token value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JwtToken(pub String);

impl JwtToken {
    pub fn new(token: String) -> Result<Self, String> {
        if token.trim().is_empty() {
            return Err("JWT token cannot be empty".to_string());
        }
        Ok(JwtToken(token))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Token claims containing user and tenant information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: Uuid,        // User ID
    pub tenant_id: Uuid,  // Tenant ID
    pub username: String, // Username
    pub nickname: Option<String>, // User nickname
    pub exp: i64,         // Expiration timestamp
    pub iat: i64,         // Issued at timestamp
    pub jti: Uuid,        // JWT ID for token revocation
}

impl TokenClaims {
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        username: String,
        nickname: Option<String>,
        expires_in: Duration,
    ) -> Self {
        let now = Utc::now();
        let exp = (now + expires_in).timestamp();
        
        Self {
            sub: user_id,
            tenant_id,
            username,
            nickname,
            exp,
            iat: now.timestamp(),
            jti: Uuid::new_v4(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.exp, 0).unwrap_or_else(|| Utc::now())
    }

    pub fn issued_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.iat, 0).unwrap_or_else(|| Utc::now())
    }
}

/// Authentication credentials for login
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub tenant_id: Uuid,
    pub username: String,
    pub password: String,
}

impl LoginCredentials {
    pub fn new(tenant_id: Uuid, username: String, password: String) -> Result<Self, String> {
        if username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if password.trim().is_empty() {
            return Err("Password cannot be empty".to_string());
        }
        
        Ok(Self {
            tenant_id,
            username: username.trim().to_string(),
            password,
        })
    }
}

/// Session information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInfo {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
    pub token: JwtToken,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl SessionInfo {
    pub fn new(
        user_id: Uuid,
        tenant_id: Uuid,
        username: String,
        nickname: Option<String>,
        token: JwtToken,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            tenant_id,
            username,
            nickname,
            token,
            expires_at,
            created_at: Utc::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}