use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QuerySelect, PaginatorTrait};
use std::sync::Arc;
use crate::domain::entities::User;
use crate::domain::repositories::UserRepository;
use crate::domain::value_objects::{UserId, TenantId, Username};
use crate::infrastructure::database::entities;
use crate::error::{Result, PlatformError};

pub struct UserRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl UserRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub fn entity_to_domain(entity: entities::user::Model) -> Result<User> {
        let username = Username::new(entity.username)
            .map_err(|e| PlatformError::ValidationError(e))?;
        
        User::new(
            TenantId::from_uuid(entity.tenant_id),
            username,
            entity.password_hash,
            entity.nickname,
        ).map_err(|e| PlatformError::ValidationError(e))
    }

    pub fn domain_to_active_model(user: &User) -> entities::user::ActiveModel {
        use sea_orm::ActiveValue::Set;
        
        entities::user::ActiveModel {
            id: Set(user.id.0),
            tenant_id: Set(user.tenant_id.0),
            username: Set(user.username.0.clone()),
            nickname: Set(user.nickname.clone()),
            password_hash: Set(user.password_hash.clone()),
            created_at: Set(user.created_at),
            updated_at: Set(user.updated_at),
        }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>> {
        let user = entities::user::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await?;

        match user {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant_and_username(&self, tenant_id: TenantId, username: &str) -> Result<Option<User>> {
        let user = entities::user::Entity::find()
            .filter(entities::user::Column::TenantId.eq(tenant_id.0))
            .filter(entities::user::Column::Username.eq(username))
            .one(self.db.as_ref())
            .await?;

        match user {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<User>> {
        let users = entities::user::Entity::find()
            .filter(entities::user::Column::TenantId.eq(tenant_id.0))
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in users {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn save(&self, user: &User) -> Result<()> {
        let active_model = Self::domain_to_active_model(user);
        
        // Check if user exists
        let existing = entities::user::Entity::find_by_id(user.id.0)
            .one(self.db.as_ref())
            .await?;

        if existing.is_some() {
            // Update existing user
            entities::user::Entity::update(active_model)
                .exec(self.db.as_ref())
                .await?;
        } else {
            // Insert new user
            entities::user::Entity::insert(active_model)
                .exec(self.db.as_ref())
                .await?;
        }

        Ok(())
    }

    async fn delete(&self, id: UserId) -> Result<()> {
        entities::user::Entity::delete_by_id(id.0)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn username_exists(&self, tenant_id: TenantId, username: &str) -> Result<bool> {
        let count = entities::user::Entity::find()
            .filter(entities::user::Column::TenantId.eq(tenant_id.0))
            .filter(entities::user::Column::Username.eq(username))
            .count(self.db.as_ref())
            .await?;

        Ok(count > 0)
    }

    async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64> {
        let count = entities::user::Entity::find()
            .filter(entities::user::Column::TenantId.eq(tenant_id.0))
            .count(self.db.as_ref())
            .await?;

        Ok(count)
    }

    async fn find_by_tenant_paginated(
        &self, 
        tenant_id: TenantId, 
        offset: u64, 
        limit: u64
    ) -> Result<Vec<User>> {
        let users = entities::user::Entity::find()
            .filter(entities::user::Column::TenantId.eq(tenant_id.0))
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in users {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }
}