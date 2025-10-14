use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    Set, TransactionTrait, PaginatorTrait,
};
use sea_orm::prelude::Expr;
use std::collections::HashMap;

use crate::domain::entities::VectorConfigEntity;
use crate::domain::repositories::VectorConfigRepository;
use crate::domain::value_objects::{TenantId, ConfigId};
use crate::error::PlatformError;
use crate::infrastructure::database::entities::vector_config;
use crate::infrastructure::vector::VectorProvider;

/// SeaORM implementation of VectorConfigRepository
pub struct VectorConfigRepositoryImpl {
    db: DatabaseConnection,
}

impl VectorConfigRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
    
    fn entity_to_domain(entity: vector_config::Model) -> Result<VectorConfigEntity, PlatformError> {
        let provider = VectorProvider::from_str(&entity.provider)?;
        
        let connection_params: HashMap<String, String> = serde_json::from_value(entity.config)
            .map_err(|e| PlatformError::SerializationError(e))?;
        
        Ok(VectorConfigEntity {
            id: ConfigId::from_uuid(entity.id),
            tenant_id: TenantId::from_uuid(entity.tenant_id),
            name: entity.name,
            provider,
            connection_params,
            is_default: entity.is_default,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        })
    }
    
    fn domain_to_active_model(config: &VectorConfigEntity) -> Result<vector_config::ActiveModel, PlatformError> {
        let config_json = serde_json::to_value(&config.connection_params)
            .map_err(|e| PlatformError::SerializationError(e))?;
        
        Ok(vector_config::ActiveModel {
            id: Set(config.id.0),
            tenant_id: Set(config.tenant_id.0),
            name: Set(config.name.clone()),
            provider: Set(config.provider.as_str().to_string()),
            config: Set(config_json),
            is_default: Set(config.is_default),
            created_at: Set(config.created_at),
            updated_at: Set(config.updated_at),
        })
    }
}

#[async_trait]
impl VectorConfigRepository for VectorConfigRepositoryImpl {
    async fn find_by_id(&self, id: ConfigId) -> Result<Option<VectorConfigEntity>, PlatformError> {
        let entity = vector_config::Entity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        match entity {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }
    
    async fn find_by_tenant_and_name(
        &self, 
        tenant_id: TenantId, 
        name: &str
    ) -> Result<Option<VectorConfigEntity>, PlatformError> {
        let entity = vector_config::Entity::find()
            .filter(vector_config::Column::TenantId.eq(tenant_id.0))
            .filter(vector_config::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        match entity {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }
    
    async fn find_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<VectorConfigEntity>, PlatformError> {
        let entities = vector_config::Entity::find()
            .filter(vector_config::Column::TenantId.eq(tenant_id.0))
            .order_by_asc(vector_config::Column::Name)
            .all(&self.db)
            .await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        let mut configs = Vec::new();
        for entity in entities {
            configs.push(Self::entity_to_domain(entity)?);
        }
        
        Ok(configs)
    }
    
    async fn find_default_by_tenant(&self, tenant_id: TenantId) -> Result<Option<VectorConfigEntity>, PlatformError> {
        let entity = vector_config::Entity::find()
            .filter(vector_config::Column::TenantId.eq(tenant_id.0))
            .filter(vector_config::Column::IsDefault.eq(true))
            .one(&self.db)
            .await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        match entity {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }
    
    async fn save(&self, config: &VectorConfigEntity) -> Result<(), PlatformError> {
        let active_model = Self::domain_to_active_model(config)?;
        
        // Check if the record exists
        let existing = vector_config::Entity::find_by_id(config.id.0)
            .one(&self.db)
            .await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        if existing.is_some() {
            // Update existing record
            active_model.update(&self.db).await
                .map_err(|e| PlatformError::DatabaseError(e))?;
        } else {
            // Insert new record
            active_model.insert(&self.db).await
                .map_err(|e| PlatformError::DatabaseError(e))?;
        }
        
        Ok(())
    }
    
    async fn delete(&self, id: ConfigId) -> Result<(), PlatformError> {
        vector_config::Entity::delete_by_id(id.0)
            .exec(&self.db)
            .await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        Ok(())
    }
    
    async fn set_as_default(&self, id: ConfigId, tenant_id: TenantId) -> Result<(), PlatformError> {
        // Start a transaction
        let txn = self.db.begin().await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        // First, unset all defaults for this tenant
        vector_config::Entity::update_many()
            .filter(vector_config::Column::TenantId.eq(tenant_id.0))
            .col_expr(vector_config::Column::IsDefault, Expr::value(false))
            .exec(&txn)
            .await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        // Then set the specified config as default
        vector_config::Entity::update_many()
            .filter(vector_config::Column::Id.eq(id.0))
            .filter(vector_config::Column::TenantId.eq(tenant_id.0))
            .col_expr(vector_config::Column::IsDefault, Expr::value(true))
            .exec(&txn)
            .await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        // Commit the transaction
        txn.commit().await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        Ok(())
    }
    
    async fn exists_by_tenant_and_name(
        &self, 
        tenant_id: TenantId, 
        name: &str
    ) -> Result<bool, PlatformError> {
        let count = vector_config::Entity::find()
            .filter(vector_config::Column::TenantId.eq(tenant_id.0))
            .filter(vector_config::Column::Name.eq(name))
            .count(&self.db)
            .await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        Ok(count > 0)
    }
    
    async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64, PlatformError> {
        let count = vector_config::Entity::find()
            .filter(vector_config::Column::TenantId.eq(tenant_id.0))
            .count(&self.db)
            .await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        Ok(count)
    }
    
    async fn find_by_tenant_and_provider(
        &self, 
        tenant_id: TenantId, 
        provider: &str
    ) -> Result<Vec<VectorConfigEntity>, PlatformError> {
        let entities = vector_config::Entity::find()
            .filter(vector_config::Column::TenantId.eq(tenant_id.0))
            .filter(vector_config::Column::Provider.eq(provider))
            .order_by_asc(vector_config::Column::Name)
            .all(&self.db)
            .await
            .map_err(|e| PlatformError::DatabaseError(e))?;
        
        let mut configs = Vec::new();
        for entity in entities {
            configs.push(Self::entity_to_domain(entity)?);
        }
        
        Ok(configs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_config() -> VectorConfigEntity {
        let mut params = HashMap::new();
        params.insert("api_key".to_string(), "test-key".to_string());
        params.insert("environment".to_string(), "test-env".to_string());
        params.insert("index_name".to_string(), "test-index".to_string());
        
        VectorConfigEntity::new(
            TenantId::new(),
            "Test Config".to_string(),
            VectorProvider::Pinecone,
            params,
        )
    }

    #[tokio::test]
    async fn test_entity_to_domain_conversion() {
        let entity = vector_config::Model {
            id: uuid::Uuid::new_v4(),
            tenant_id: uuid::Uuid::new_v4(),
            name: "Test Config".to_string(),
            provider: "pinecone".to_string(),
            config: serde_json::json!({
                "api_key": "test-key",
                "environment": "test-env",
                "index_name": "test-index"
            }),
            is_default: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        let domain_config = VectorConfigRepositoryImpl::entity_to_domain(entity.clone()).unwrap();
        
        assert_eq!(domain_config.name, entity.name);
        assert_eq!(domain_config.provider, VectorProvider::Pinecone);
        assert_eq!(domain_config.is_default, entity.is_default);
        assert_eq!(domain_config.connection_params.len(), 3);
    }

    #[tokio::test]
    async fn test_domain_to_active_model_conversion() {
        let config = create_test_config();
        let active_model = VectorConfigRepositoryImpl::domain_to_active_model(&config).unwrap();
        
        match active_model.name {
            Set(name) => assert_eq!(name, config.name),
            _ => panic!("Expected Set value for name"),
        }
        
        match active_model.provider {
            Set(provider) => assert_eq!(provider, "pinecone"),
            _ => panic!("Expected Set value for provider"),
        }
    }
}