use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QuerySelect, PaginatorTrait};
use std::sync::Arc;
use crate::domain::entities::Tenant;
use crate::domain::repositories::TenantRepository;
use crate::domain::value_objects::{TenantId, TenantName};
use crate::infrastructure::database::entities;
use crate::error::{Result, PlatformError};

pub struct TenantRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl TenantRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_domain(entity: entities::tenant::Model) -> Result<Tenant> {
        let name = TenantName::new(entity.name)
            .map_err(|e| PlatformError::ValidationError(e))?;
        
        Ok(Tenant {
            id: TenantId::from_uuid(entity.id),
            name,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        })
    }

    fn domain_to_active_model(tenant: &Tenant) -> entities::tenant::ActiveModel {
        use sea_orm::ActiveValue::Set;
        
        entities::tenant::ActiveModel {
            id: Set(tenant.id.0),
            name: Set(tenant.name.0.clone()),
            created_at: Set(tenant.created_at),
            updated_at: Set(tenant.updated_at),
        }
    }
}

#[async_trait]
impl TenantRepository for TenantRepositoryImpl {
    async fn find_by_id(&self, id: TenantId) -> Result<Option<Tenant>> {
        let tenant = entities::tenant::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await?;
        println!("id: {}, tenant: {:?}", id.0, tenant);

        match tenant {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Tenant>> {
        let tenant = entities::tenant::Entity::find()
            .filter(entities::tenant::Column::Name.eq(name))
            .one(self.db.as_ref())
            .await?;

        match tenant {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> Result<Vec<Tenant>> {
        let tenants = entities::tenant::Entity::find()
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in tenants {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn save(&self, tenant: &Tenant) -> Result<()> {
        let active_model = Self::domain_to_active_model(tenant);
        
        // Check if tenant exists
        let existing = entities::tenant::Entity::find_by_id(tenant.id.0)
            .one(self.db.as_ref())
            .await?;

        if existing.is_some() {
            // Update existing tenant
            entities::tenant::Entity::update(active_model)
                .exec(self.db.as_ref())
                .await?;
        } else {
            // Insert new tenant
            entities::tenant::Entity::insert(active_model)
                .exec(self.db.as_ref())
                .await?;
        }

        Ok(())
    }

    async fn delete(&self, id: TenantId) -> Result<()> {
        entities::tenant::Entity::delete_by_id(id.0)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn name_exists(&self, name: &str) -> Result<bool> {
        let count = entities::tenant::Entity::find()
            .filter(entities::tenant::Column::Name.eq(name))
            .count(self.db.as_ref())
            .await?;

        Ok(count > 0)
    }

    async fn count(&self) -> Result<u64> {
        let count = entities::tenant::Entity::find()
            .count(self.db.as_ref())
            .await?;

        Ok(count)
    }

    async fn find_paginated(&self, offset: u64, limit: u64) -> Result<Vec<Tenant>> {
        let tenants = entities::tenant::Entity::find()
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in tenants {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }
}