use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::domain::value_objects::{UserId, TenantId, Username};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub tenant_id: TenantId,
    pub username: Username,
    pub nickname: Option<String>,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        tenant_id: TenantId,
        username: Username,
        password_hash: String,
        nickname: Option<String>,
    ) -> Result<Self, String> {
        let now = Utc::now();
        
        // Validate password hash
        if password_hash.trim().is_empty() {
            return Err("Password hash cannot be empty".to_string());
        }

        // Validate nickname if provided
        if let Some(ref nick) = nickname {
            if nick.len() > 255 {
                return Err("Nickname cannot exceed 255 characters".to_string());
            }
        }

        Ok(User {
            id: UserId::new(),
            tenant_id,
            username,
            nickname,
            password_hash,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_nickname(&mut self, nickname: Option<String>) -> Result<(), String> {
        if let Some(ref nick) = nickname {
            if nick.len() > 255 {
                return Err("Nickname cannot exceed 255 characters".to_string());
            }
        }
        
        self.nickname = nickname;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_password(&mut self, password_hash: String) -> Result<(), String> {
        if password_hash.trim().is_empty() {
            return Err("Password hash cannot be empty".to_string());
        }
        
        self.password_hash = password_hash;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn belongs_to_tenant(&self, tenant_id: &TenantId) -> bool {
        &self.tenant_id == tenant_id
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.password_hash.trim().is_empty() {
            return Err("Password hash cannot be empty".to_string());
        }

        if let Some(ref nickname) = self.nickname {
            if nickname.len() > 255 {
                return Err("Nickname cannot exceed 255 characters".to_string());
            }
        }

        Ok(())
    }
}