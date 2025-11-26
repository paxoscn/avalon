use async_trait::async_trait;
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QuerySelect, 
    PaginatorTrait, QueryOrder, Set, ActiveModelTrait
};
use std::sync::Arc;
use chrono::Utc;
use crate::domain::entities::APIKey;
use crate::domain::repositories::{APIKeyRepository, QueryOptions, APIKeyQueryResult};
use crate::domain::value_objects::{APIKeyId, TenantId, UserId, PermissionScope};
use crate::infrastructure::database::entities;
use crate::error::{Result, PlatformError};

pub struct APIKeyRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl APIKeyRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Convert database entity to domain entity
    fn entity_to_domain(entity: entities::api_key::Model) -> Result<APIKey> {
        // Deserialize permission_scope from JSON
        let permission_scope: PermissionScope = serde_json::from_value(entity.permission_scope.clone())
            .map_err(|e| PlatformError::ValidationError(format!("Failed to deserialize permission scope: {}", e)))?;

        Ok(APIKey {
            id: APIKeyId::from_uuid(entity.id),
            tenant_id: TenantId::from_uuid(entity.tenant_id),
            user_id: UserId::from_uuid(entity.user_id),
            name: entity.name,
            key_hash: entity.key_hash,
            permission_scope,
            enabled: entity.enabled,
            expires_at: entity.expires_at,
            last_used_at: entity.last_used_at,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        })
    }

    /// Convert domain entity to active model for insert/update
    fn domain_to_active_model(api_key: &APIKey) -> Result<entities::api_key::ActiveModel> {
        // Serialize permission_scope to JSON
        let permission_scope_json = serde_json::to_value(&api_key.permission_scope)
            .map_err(|e| PlatformError::ValidationError(format!("Failed to serialize permission scope: {}", e)))?;

        Ok(entities::api_key::ActiveModel {
            id: Set(api_key.id.0),
            tenant_id: Set(api_key.tenant_id.0),
            user_id: Set(api_key.user_id.0),
            name: Set(api_key.name.clone()),
            key_hash: Set(api_key.key_hash.clone()),
            permission_scope: Set(permission_scope_json),
            enabled: Set(api_key.enabled),
            expires_at: Set(api_key.expires_at),
            last_used_at: Set(api_key.last_used_at),
            created_at: Set(api_key.created_at),
            updated_at: Set(api_key.updated_at),
        })
    }
}

#[async_trait]
impl APIKeyRepository for APIKeyRepositoryImpl {
    async fn save(&self, api_key: &APIKey) -> Result<()> {
        let active_model = Self::domain_to_active_model(api_key)?;
        
        // Check if API key exists
        let existing = entities::api_key::Entity::find_by_id(api_key.id.0)
            .one(self.db.as_ref())
            .await?;

        if existing.is_some() {
            // Update existing API key
            entities::api_key::Entity::update(active_model)
                .exec(self.db.as_ref())
                .await?;
        } else {
            // Insert new API key
            entities::api_key::Entity::insert(active_model)
                .exec(self.db.as_ref())
                .await?;
        }

        Ok(())
    }

    async fn find_by_id(&self, id: APIKeyId) -> Result<Option<APIKey>> {
        let api_key = entities::api_key::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await?;

        match api_key {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_key_hash(&self, key_hash: &str) -> Result<Option<APIKey>> {
        let api_key = entities::api_key::Entity::find()
            .filter(entities::api_key::Column::KeyHash.eq(key_hash))
            .one(self.db.as_ref())
            .await?;

        match api_key {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant(&self, tenant_id: TenantId, options: QueryOptions) -> Result<APIKeyQueryResult> {
        let mut query = entities::api_key::Entity::find()
            .filter(entities::api_key::Column::TenantId.eq(tenant_id.0));

        // Apply enabled filter if specified
        if let Some(enabled) = options.enabled_filter {
            query = query.filter(entities::api_key::Column::Enabled.eq(enabled));
        }

        // Apply expiration filter
        if !options.include_expired {
            let now = Utc::now();
            query = query.filter(
                entities::api_key::Column::ExpiresAt.is_null()
                    .or(entities::api_key::Column::ExpiresAt.gt(now))
            );
        }

        // Get total count
        let total = query.clone().count(self.db.as_ref()).await?;

        // Apply pagination and ordering
        let offset = options.offset.unwrap_or(0);
        let limit = options.limit.unwrap_or(50);
        
        let api_keys = query
            .order_by_desc(entities::api_key::Column::CreatedAt)
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut items = Vec::new();
        for entity in api_keys {
            items.push(Self::entity_to_domain(entity)?);
        }

        Ok(APIKeyQueryResult {
            items,
            total,
            offset,
            limit,
        })
    }

    async fn find_by_user(&self, user_id: UserId, options: QueryOptions) -> Result<APIKeyQueryResult> {
        let mut query = entities::api_key::Entity::find()
            .filter(entities::api_key::Column::UserId.eq(user_id.0));

        // Apply enabled filter if specified
        if let Some(enabled) = options.enabled_filter {
            query = query.filter(entities::api_key::Column::Enabled.eq(enabled));
        }

        // Apply expiration filter
        if !options.include_expired {
            let now = Utc::now();
            query = query.filter(
                entities::api_key::Column::ExpiresAt.is_null()
                    .or(entities::api_key::Column::ExpiresAt.gt(now))
            );
        }

        // Get total count
        let total = query.clone().count(self.db.as_ref()).await?;

        // Apply pagination and ordering
        let offset = options.offset.unwrap_or(0);
        let limit = options.limit.unwrap_or(50);
        
        let api_keys = query
            .order_by_desc(entities::api_key::Column::CreatedAt)
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut items = Vec::new();
        for entity in api_keys {
            items.push(Self::entity_to_domain(entity)?);
        }

        Ok(APIKeyQueryResult {
            items,
            total,
            offset,
            limit,
        })
    }

    async fn update(&self, api_key: &APIKey) -> Result<()> {
        let active_model = Self::domain_to_active_model(api_key)?;
        
        entities::api_key::Entity::update(active_model)
            .exec(self.db.as_ref())
            .await?;

        Ok(())
    }

    async fn delete(&self, id: APIKeyId) -> Result<()> {
        entities::api_key::Entity::delete_by_id(id.0)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64> {
        let count = entities::api_key::Entity::find()
            .filter(entities::api_key::Column::TenantId.eq(tenant_id.0))
            .count(self.db.as_ref())
            .await?;

        Ok(count)
    }
}
