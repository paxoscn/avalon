use crate::domain::entities::LLMConfig;
use crate::domain::repositories::LLMConfigRepository;
use crate::domain::value_objects::{ConfigId, TenantId, ModelConfig};
use crate::error::{PlatformError, Result};
use crate::infrastructure::database::entities;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Set, PaginatorTrait, TransactionTrait,
};
use std::sync::Arc;
use sea_orm::prelude::Expr;
use serde_json;


pub struct LLMConfigRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl LLMConfigRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_domain(entity: entities::llm_config::Model) -> Result<LLMConfig> {
        let model_config: ModelConfig = serde_json::from_value(entity.config)
            .map_err(|e| PlatformError::InternalError(format!("Failed to deserialize model config: {}", e)))?;

        Ok(LLMConfig {
            id: ConfigId::from_uuid(entity.id),
            tenant_id: TenantId::from_uuid(entity.tenant_id),
            name: entity.name,
            description: None, // Not stored in database entity yet
            model_config,
            is_default: entity.is_default,
            is_active: true, // Not stored in database entity yet, assume active
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        })
    }

    fn domain_to_active_model(config: &LLMConfig) -> Result<entities::llm_config::ActiveModel> {
        let config_json = serde_json::to_value(&config.model_config)
            .map_err(|e| PlatformError::InternalError(format!("Failed to serialize model config: {}", e)))?;

        Ok(entities::llm_config::ActiveModel {
            id: Set(config.id.0),
            tenant_id: Set(config.tenant_id.0),
            name: Set(config.name.clone()),
            provider: Set(config.provider_name()),
            config: Set(config_json),
            is_default: Set(config.is_default),
            created_at: Set(config.created_at),
            updated_at: Set(config.updated_at),
        })
    }
}

#[async_trait]
impl LLMConfigRepository for LLMConfigRepositoryImpl {
    async fn find_by_id(&self, id: ConfigId) -> Result<Option<LLMConfig>> {
        let entity = entities::llm_config::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await
            .map_err(PlatformError::DatabaseError)?;

        match entity {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<LLMConfig>> {
        let entities = entities::llm_config::Entity::find()
            .filter(entities::llm_config::Column::TenantId.eq(tenant_id.0))
            .order_by_asc(entities::llm_config::Column::Name)
            .all(self.db.as_ref())
            .await
            .map_err(PlatformError::DatabaseError)?;

        let mut configs = Vec::new();
        for entity in entities {
            configs.push(Self::entity_to_domain(entity)?);
        }
        Ok(configs)
    }

    async fn find_active_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<LLMConfig>> {
        // Since we don't have is_active in the database yet, return all configs
        // In the future, we can add a filter for is_active
        self.find_by_tenant(tenant_id).await
    }

    async fn find_default_by_tenant(&self, tenant_id: TenantId) -> Result<Option<LLMConfig>> {
        let entity = entities::llm_config::Entity::find()
            .filter(entities::llm_config::Column::TenantId.eq(tenant_id.0))
            .filter(entities::llm_config::Column::IsDefault.eq(true))
            .one(self.db.as_ref())
            .await
            .map_err(PlatformError::DatabaseError)?;

        match entity {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant_and_name(&self, tenant_id: TenantId, name: &str) -> Result<Option<LLMConfig>> {
        let entity = entities::llm_config::Entity::find()
            .filter(entities::llm_config::Column::TenantId.eq(tenant_id.0))
            .filter(entities::llm_config::Column::Name.eq(name))
            .one(self.db.as_ref())
            .await
            .map_err(PlatformError::DatabaseError)?;

        match entity {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, config: &LLMConfig) -> Result<()> {
        let active_model = Self::domain_to_active_model(config)?;
        
        // Check if the config already exists
        let existing = entities::llm_config::Entity::find_by_id(config.id.0)
            .one(self.db.as_ref())
            .await
            .map_err(PlatformError::DatabaseError)?;

        if existing.is_some() {
            // Update existing
            active_model
                .update(self.db.as_ref())
                .await
                .map_err(PlatformError::DatabaseError)?;
        } else {
            // Insert new
            active_model
                .insert(self.db.as_ref())
                .await
                .map_err(PlatformError::DatabaseError)?;
        }

        Ok(())
    }

    async fn delete(&self, id: ConfigId) -> Result<()> {
        entities::llm_config::Entity::delete_by_id(id.0)
            .exec(self.db.as_ref())
            .await
            .map_err(PlatformError::DatabaseError)?;

        Ok(())
    }

    async fn name_exists(&self, tenant_id: TenantId, name: &str) -> Result<bool> {
        let count = entities::llm_config::Entity::find()
            .filter(entities::llm_config::Column::TenantId.eq(tenant_id.0))
            .filter(entities::llm_config::Column::Name.eq(name))
            .count(self.db.as_ref())
            .await
            .map_err(PlatformError::DatabaseError)?;

        Ok(count > 0)
    }

    async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64> {
        let count = entities::llm_config::Entity::find()
            .filter(entities::llm_config::Column::TenantId.eq(tenant_id.0))
            .count(self.db.as_ref())
            .await
            .map_err(PlatformError::DatabaseError)?;

        Ok(count)
    }

    async fn find_by_tenant_and_provider(&self, tenant_id: TenantId, provider: &str) -> Result<Vec<LLMConfig>> {
        let entities = entities::llm_config::Entity::find()
            .filter(entities::llm_config::Column::TenantId.eq(tenant_id.0))
            .filter(entities::llm_config::Column::Provider.eq(provider))
            .order_by_asc(entities::llm_config::Column::Name)
            .all(self.db.as_ref())
            .await
            .map_err(PlatformError::DatabaseError)?;

        let mut configs = Vec::new();
        for entity in entities {
            configs.push(Self::entity_to_domain(entity)?);
        }
        Ok(configs)
    }

    async fn set_as_default(&self, tenant_id: TenantId, config_id: ConfigId) -> Result<()> {
        // Start a transaction to ensure atomicity
        let txn = self.db.begin().await
            .map_err(PlatformError::DatabaseError)?;

        // First, unset all default flags for this tenant
        entities::llm_config::Entity::update_many()
            .filter(entities::llm_config::Column::TenantId.eq(tenant_id.0))
            .col_expr(entities::llm_config::Column::IsDefault, Expr::value(false))
            .exec(&txn)
            .await
            .map_err(PlatformError::DatabaseError)?;

        // Then set the specified config as default
        entities::llm_config::Entity::update_many()
            .filter(entities::llm_config::Column::TenantId.eq(tenant_id.0))
            .filter(entities::llm_config::Column::Id.eq(config_id.0))
            .col_expr(entities::llm_config::Column::IsDefault, Expr::value(true))
            .exec(&txn)
            .await
            .map_err(PlatformError::DatabaseError)?;

        txn.commit().await
            .map_err(PlatformError::DatabaseError)?;

        Ok(())
    }

    async fn find_by_tenant_paginated(
        &self,
        tenant_id: TenantId,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<LLMConfig>> {
        let entities = entities::llm_config::Entity::find()
            .filter(entities::llm_config::Column::TenantId.eq(tenant_id.0))
            .order_by_asc(entities::llm_config::Column::Name)
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await
            .map_err(PlatformError::DatabaseError)?;

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
    use crate::domain::value_objects::{ModelProvider, ModelParameters, ModelCredentials};
    use sea_orm::{Database, DatabaseBackend, MockDatabase, MockExecResult};

    fn create_test_config() -> LLMConfig {
        let model_config = ModelConfig {
            provider: ModelProvider::OpenAI,
            model_name: "gpt-3.5-turbo".to_string(),
            parameters: ModelParameters::default(),
            credentials: ModelCredentials::default(),
        };

        LLMConfig::new(
            TenantId::new(),
            "Test Config".to_string(),
            model_config,
        )
    }

    #[tokio::test]
    async fn test_entity_to_domain_conversion() {
        let model_config = ModelConfig {
            provider: ModelProvider::OpenAI,
            model_name: "gpt-3.5-turbo".to_string(),
            parameters: ModelParameters::default(),
            credentials: ModelCredentials::default(),
        };

        let entity = entities::llm_config::Model {
            id: uuid::Uuid::new_v4(),
            tenant_id: uuid::Uuid::new_v4(),
            name: "Test Config".to_string(),
            provider: "openai".to_string(),
            config: serde_json::to_value(&model_config).unwrap(),
            is_default: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let domain_config = LLMConfigRepositoryImpl::entity_to_domain(entity.clone()).unwrap();

        assert_eq!(domain_config.id.0, entity.id);
        assert_eq!(domain_config.tenant_id.0, entity.tenant_id);
        assert_eq!(domain_config.name, entity.name);
        assert_eq!(domain_config.is_default, entity.is_default);
        assert_eq!(domain_config.model_config.model_name, "gpt-3.5-turbo");
    }

    #[tokio::test]
    async fn test_domain_to_active_model_conversion() {
        let config = create_test_config();
        let active_model = LLMConfigRepositoryImpl::domain_to_active_model(&config).unwrap();

        assert_eq!(active_model.id.unwrap(), config.id.0);
        assert_eq!(active_model.tenant_id.unwrap(), config.tenant_id.0);
        assert_eq!(active_model.name.unwrap(), config.name);
        assert_eq!(active_model.is_default.unwrap(), config.is_default);
        assert_eq!(active_model.provider.unwrap(), "openai");
    }
}