use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::domain::value_objects::{TenantId, TenantName};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub name: TenantName,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tenant {
    pub fn new(name: TenantName) -> Self {
        let now = Utc::now();
        Self {
            id: TenantId::new(),
            name,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_name(&mut self, name: TenantName) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn validate(&self) -> Result<(), String> {
        // Name validation is handled by TenantName value object
        Ok(())
    }
}