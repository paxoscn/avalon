use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;

use crate::domain::{
    entities::{MCPToolVersion, VersionDiff},
    repositories::mcp_tool_version_repository::{
        MCPToolVersionQueryOptions, MCPToolVersionQueryResult, MCPToolVersionRepository,
    },
    value_objects::{
        ids::{MCPToolId, MCPToolVersionId, UserId},
        tool_config::ToolConfig,
    },
};
use crate::error::PlatformError;
use crate::infrastructure::database::entities::mcp_tool_version;

/// MCP工具版本仓储实现
pub struct MCPToolVersionRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl MCPToolVersionRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// 将领域实体转换为数据库实体
    fn domain_to_db_entity(&self, version: &MCPToolVersion) -> Result<mcp_tool_version::ActiveModel, PlatformError> {
        let config_json = serde_json::to_value(&version.config).map_err(|e| {
            PlatformError::InternalError(format!("Failed to serialize tool config: {}", e))
        })?;

        Ok(mcp_tool_version::ActiveModel {
            id: Set(version.id.0),
            tool_id: Set(version.tool_id.0),
            version: Set(version.version),
            config: Set(config_json),
            change_log: Set(version.change_log.clone()),
            created_by: Set(version.created_by.0),
            created_at: Set(version.created_at),
        })
    }

    /// 将数据库实体转换为领域实体
    fn db_entity_to_domain(&self, model: mcp_tool_version::Model) -> Result<MCPToolVersion, PlatformError> {
        let config: ToolConfig = serde_json::from_value(model.config).map_err(|e| {
            PlatformError::InternalError(format!("Failed to deserialize tool config: {}", e))
        })?;

        Ok(MCPToolVersion {
            id: MCPToolVersionId::from_uuid(model.id),
            tool_id: MCPToolId::from_uuid(model.tool_id),
            version: model.version,
            config,
            change_log: model.change_log,
            created_by: UserId::from_uuid(model.created_by),
            created_at: model.created_at,
        })
    }
}

#[async_trait]
impl MCPToolVersionRepository for MCPToolVersionRepositoryImpl {
    async fn find_by_id(&self, id: MCPToolVersionId) -> Result<Option<MCPToolVersion>, PlatformError> {
        let model = mcp_tool_version::Entity::find_by_id(id.0)
            .one(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        match model {
            Some(model) => {
                let version = self.db_entity_to_domain(model)?;
                Ok(Some(version))
            }
            None => Ok(None),
        }
    }

    async fn find_by_tool_and_version(
        &self,
        tool_id: MCPToolId,
        version: i32,
    ) -> Result<Option<MCPToolVersion>, PlatformError> {
        let model = mcp_tool_version::Entity::find()
            .filter(mcp_tool_version::Column::ToolId.eq(tool_id.0))
            .filter(mcp_tool_version::Column::Version.eq(version))
            .one(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        match model {
            Some(model) => {
                let version = self.db_entity_to_domain(model)?;
                Ok(Some(version))
            }
            None => Ok(None),
        }
    }

    async fn find_by_options(&self, options: MCPToolVersionQueryOptions) -> Result<MCPToolVersionQueryResult, PlatformError> {
        let mut query = mcp_tool_version::Entity::find();

        // 应用过滤条件
        if let Some(tool_id) = options.tool_id {
            query = query.filter(mcp_tool_version::Column::ToolId.eq(tool_id.0));
        }

        if let Some(version) = options.version {
            query = query.filter(mcp_tool_version::Column::Version.eq(version));
        }

        if let Some(created_by) = options.created_by {
            query = query.filter(mcp_tool_version::Column::CreatedBy.eq(created_by.0));
        }

        // 排序（按版本号倒序）
        query = query.order_by_desc(mcp_tool_version::Column::Version);

        // 获取总数
        let total_count = query.clone().count(&*self.db).await.map_err(PlatformError::DatabaseError)?;

        // 应用分页
        if let (Some(limit), Some(offset)) = (options.limit, options.offset) {
            query = query.limit(limit).offset(offset);
        }

        // 执行查询
        let models = query.all(&*self.db).await.map_err(PlatformError::DatabaseError)?;

        // 转换为领域实体
        let mut versions = Vec::new();
        for model in models {
            let version = self.db_entity_to_domain(model)?;
            versions.push(version);
        }

        Ok(MCPToolVersionQueryResult { versions, total_count })
    }

    async fn find_by_tool_id(&self, tool_id: MCPToolId) -> Result<Vec<MCPToolVersion>, PlatformError> {
        let models = mcp_tool_version::Entity::find()
            .filter(mcp_tool_version::Column::ToolId.eq(tool_id.0))
            .order_by_desc(mcp_tool_version::Column::Version)
            .all(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        let mut versions = Vec::new();
        for model in models {
            let version = self.db_entity_to_domain(model)?;
            versions.push(version);
        }

        Ok(versions)
    }

    async fn find_latest_by_tool_id(&self, tool_id: MCPToolId) -> Result<Option<MCPToolVersion>, PlatformError> {
        let model = mcp_tool_version::Entity::find()
            .filter(mcp_tool_version::Column::ToolId.eq(tool_id.0))
            .order_by_desc(mcp_tool_version::Column::Version)
            .one(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        match model {
            Some(model) => {
                let version = self.db_entity_to_domain(model)?;
                Ok(Some(version))
            }
            None => Ok(None),
        }
    }

    async fn find_recent_by_tool_id(&self, tool_id: MCPToolId, limit: u64) -> Result<Vec<MCPToolVersion>, PlatformError> {
        let models = mcp_tool_version::Entity::find()
            .filter(mcp_tool_version::Column::ToolId.eq(tool_id.0))
            .order_by_desc(mcp_tool_version::Column::Version)
            .limit(limit)
            .all(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        let mut versions = Vec::new();
        for model in models {
            let version = self.db_entity_to_domain(model)?;
            versions.push(version);
        }

        Ok(versions)
    }

    async fn save(&self, version: &MCPToolVersion) -> Result<(), PlatformError> {
        let active_model = self.domain_to_db_entity(version)?;
        active_model.insert(&*self.db).await.map_err(PlatformError::DatabaseError)?;
        Ok(())
    }

    async fn update(&self, version: &MCPToolVersion) -> Result<(), PlatformError> {
        let active_model = self.domain_to_db_entity(version)?;
        active_model.update(&*self.db).await.map_err(PlatformError::DatabaseError)?;
        Ok(())
    }

    async fn delete(&self, id: MCPToolVersionId) -> Result<(), PlatformError> {
        mcp_tool_version::Entity::delete_by_id(id.0)
            .exec(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;
        Ok(())
    }

    async fn delete_by_tool_id(&self, tool_id: MCPToolId) -> Result<(), PlatformError> {
        mcp_tool_version::Entity::delete_many()
            .filter(mcp_tool_version::Column::ToolId.eq(tool_id.0))
            .exec(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;
        Ok(())
    }

    async fn exists_by_tool_and_version(
        &self,
        tool_id: MCPToolId,
        version: i32,
    ) -> Result<bool, PlatformError> {
        let count = mcp_tool_version::Entity::find()
            .filter(mcp_tool_version::Column::ToolId.eq(tool_id.0))
            .filter(mcp_tool_version::Column::Version.eq(version))
            .count(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        Ok(count > 0)
    }

    async fn get_next_version_number(&self, tool_id: MCPToolId) -> Result<i32, PlatformError> {
        let latest_version = mcp_tool_version::Entity::find()
            .filter(mcp_tool_version::Column::ToolId.eq(tool_id.0))
            .order_by_desc(mcp_tool_version::Column::Version)
            .one(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        match latest_version {
            Some(version) => Ok(version.version + 1),
            None => Ok(1),
        }
    }

    async fn count_by_tool_id(&self, tool_id: MCPToolId) -> Result<u64, PlatformError> {
        let count = mcp_tool_version::Entity::find()
            .filter(mcp_tool_version::Column::ToolId.eq(tool_id.0))
            .count(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        Ok(count)
    }

    async fn compare_versions(
        &self,
        tool_id: MCPToolId,
        from_version: i32,
        to_version: i32,
    ) -> Result<VersionDiff, PlatformError> {
        let from_model = self.find_by_tool_and_version(tool_id, from_version).await?
            .ok_or_else(|| PlatformError::NotFound(format!("Version {} not found", from_version)))?;

        let to_model = self.find_by_tool_and_version(tool_id, to_version).await?
            .ok_or_else(|| PlatformError::NotFound(format!("Version {} not found", to_version)))?;

        Ok(VersionDiff::new(&from_model, &to_model))
    }

    async fn get_version_history(&self, tool_id: MCPToolId) -> Result<Vec<MCPToolVersion>, PlatformError> {
        self.find_by_tool_id(tool_id).await
    }

    async fn rollback_to_version(
        &self,
        tool_id: MCPToolId,
        target_version: i32,
        created_by: UserId,
        change_log: Option<String>,
    ) -> Result<MCPToolVersion, PlatformError> {
        // 获取目标版本的配置
        let target_version_model = self.find_by_tool_and_version(tool_id, target_version).await?
            .ok_or_else(|| PlatformError::NotFound(format!("Target version {} not found", target_version)))?;

        // 获取下一个版本号
        let next_version = self.get_next_version_number(tool_id).await?;

        // 创建新版本记录（使用目标版本的配置）
        let rollback_log = change_log.unwrap_or_else(|| {
            format!("Rollback to version {}", target_version)
        });

        let new_version = MCPToolVersion::new(
            tool_id,
            next_version,
            target_version_model.config,
            Some(rollback_log),
            created_by,
        );

        // 保存新版本
        self.save(&new_version).await?;

        Ok(new_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::tool_config::{HTTPToolConfig, HttpMethod};
    use sea_orm::{DatabaseBackend, MockDatabase};

    #[tokio::test]
    async fn test_domain_to_db_entity_conversion() {
        let db = Arc::new(MockDatabase::new(DatabaseBackend::MySql).into_connection());
        let repo = MCPToolVersionRepositoryImpl::new(db);

        let tool_id = MCPToolId::new();
        let user_id = UserId::new();
        let config = ToolConfig::HTTP(HTTPToolConfig::new(
            "https://api.example.com".to_string(),
            HttpMethod::GET,
        ));

        let version = MCPToolVersion::new(
            tool_id,
            1,
            config,
            Some("Initial version".to_string()),
            user_id,
        );

        let active_model = repo.domain_to_db_entity(&version).unwrap();

        assert_eq!(active_model.id.unwrap(), version.id.0);
        assert_eq!(active_model.tool_id.unwrap(), tool_id.0);
        assert_eq!(active_model.version.unwrap(), 1);
        assert_eq!(active_model.created_by.unwrap(), user_id.0);
        assert_eq!(active_model.change_log.unwrap(), Some("Initial version".to_string()));
    }

    #[test]
    fn test_version_validation() {
        let tool_id = MCPToolId::new();
        let user_id = UserId::new();
        let config = ToolConfig::default();

        let version = MCPToolVersion::new(
            tool_id,
            1,
            config,
            Some("Test version".to_string()),
            user_id,
        );

        assert!(version.validate_version().is_ok());
        assert!(version.validate_config().is_ok());
        assert!(version.is_initial_version());
    }
}