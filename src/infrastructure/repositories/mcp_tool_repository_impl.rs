use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;

use crate::domain::{
    entities::{MCPTool, MCPToolVersion, VersionDiff},
    repositories::{
        mcp_tool_repository::{MCPToolQueryOptions, MCPToolQueryResult, MCPToolRepository},
        mcp_tool_version_repository::MCPToolVersionRepository,
    },
    value_objects::{
        ids::{MCPToolId, TenantId, UserId},
        tool_config::ToolConfig,
    },
};
use crate::error::PlatformError;
use crate::infrastructure::database::entities::{mcp_tool, mcp_tool_version};
use crate::infrastructure::repositories::MCPToolVersionRepositoryImpl;

/// MCP工具仓储实现
pub struct MCPToolRepositoryImpl {
    db: Arc<DatabaseConnection>,
    version_repo: MCPToolVersionRepositoryImpl,
}

impl MCPToolRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        let version_repo = MCPToolVersionRepositoryImpl::new(db.clone());
        Self { db, version_repo }
    }

    /// 将领域实体转换为数据库实体
    fn domain_to_db_entity(&self, tool: &MCPTool) -> mcp_tool::ActiveModel {
        mcp_tool::ActiveModel {
            id: Set(tool.id.0),
            tenant_id: Set(tool.tenant_id.0),
            name: Set(tool.name.clone()),
            description: Set(tool.description.clone()),
            current_version: Set(tool.current_version),
            status: Set(self.domain_status_to_db(&tool.status)),
            created_by: Set(tool.created_by.0),
            created_at: Set(tool.created_at),
            updated_at: Set(tool.updated_at),
        }
    }

    /// 将数据库实体转换为领域实体
    fn db_entity_to_domain(&self, model: mcp_tool::Model) -> Result<MCPTool, PlatformError> {
        // 注意：这里我们需要从版本表中获取配置信息
        // 为了简化，我们先使用默认配置，实际实现中应该查询版本表
        let config = ToolConfig::default();

        let mut tool = MCPTool::new(
            TenantId::from_uuid(model.tenant_id),
            model.name,
            model.description,
            config,
            UserId::from_uuid(model.created_by),
        );

        // 更新从数据库获取的字段
        tool.id = MCPToolId::from_uuid(model.id);
        tool.current_version = model.current_version;
        tool.status = self.db_status_to_domain(&model.status);
        tool.created_at = model.created_at;
        tool.updated_at = model.updated_at;

        Ok(tool)
    }

    /// 将领域状态转换为数据库状态
    fn domain_status_to_db(&self, status: &crate::domain::entities::MCPToolStatus) -> mcp_tool::ToolStatus {
        match status {
            crate::domain::entities::MCPToolStatus::Active => mcp_tool::ToolStatus::Active,
            crate::domain::entities::MCPToolStatus::Inactive => mcp_tool::ToolStatus::Inactive,
            crate::domain::entities::MCPToolStatus::Testing => mcp_tool::ToolStatus::Inactive, // 映射到Inactive
        }
    }

    /// 将数据库状态转换为领域状态
    fn db_status_to_domain(&self, status: &mcp_tool::ToolStatus) -> crate::domain::entities::MCPToolStatus {
        match status {
            mcp_tool::ToolStatus::Active => crate::domain::entities::MCPToolStatus::Active,
            mcp_tool::ToolStatus::Inactive => crate::domain::entities::MCPToolStatus::Inactive,
        }
    }

    /// 创建工具版本记录
    async fn create_tool_version(&self, tool: &MCPTool, change_log: Option<String>) -> Result<(), PlatformError> {
        let version = MCPToolVersion::new(
            tool.id,
            tool.current_version,
            tool.config.clone(),
            change_log,
            tool.created_by,
        );

        self.version_repo.save(&version).await
    }

    /// 获取工具的最新配置
    async fn get_tool_config(&self, tool_id: MCPToolId, version: i32) -> Result<ToolConfig, PlatformError> {
        match self.version_repo.find_by_tool_and_version(tool_id, version).await? {
            Some(version_model) => Ok(version_model.config),
            None => Ok(ToolConfig::default()),
        }
    }

    /// 完整的领域实体转换（包含配置）
    async fn db_entity_to_domain_with_config(&self, model: mcp_tool::Model) -> Result<MCPTool, PlatformError> {
        let config = self.get_tool_config(MCPToolId::from_uuid(model.id), model.current_version).await?;

        let mut tool = MCPTool::new(
            TenantId::from_uuid(model.tenant_id),
            model.name,
            model.description,
            config,
            UserId::from_uuid(model.created_by),
        );

        // 更新从数据库获取的字段
        tool.id = MCPToolId::from_uuid(model.id);
        tool.current_version = model.current_version;
        tool.status = self.db_status_to_domain(&model.status);
        tool.created_at = model.created_at;
        tool.updated_at = model.updated_at;

        Ok(tool)
    }
}

#[async_trait]
impl MCPToolRepository for MCPToolRepositoryImpl {
    async fn find_by_id(&self, id: MCPToolId) -> Result<Option<MCPTool>, PlatformError> {
        let model = mcp_tool::Entity::find_by_id(id.0)
            .one(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        match model {
            Some(model) => {
                let tool = self.db_entity_to_domain_with_config(model).await?;
                Ok(Some(tool))
            }
            None => Ok(None),
        }
    }

    async fn find_by_tenant_and_name(
        &self,
        tenant_id: TenantId,
        name: &str,
    ) -> Result<Option<MCPTool>, PlatformError> {
        let model = mcp_tool::Entity::find()
            .filter(mcp_tool::Column::TenantId.eq(tenant_id.0))
            .filter(mcp_tool::Column::Name.eq(name))
            .one(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        match model {
            Some(model) => {
                let tool = self.db_entity_to_domain_with_config(model).await?;
                Ok(Some(tool))
            }
            None => Ok(None),
        }
    }

    async fn find_by_options(&self, options: MCPToolQueryOptions) -> Result<MCPToolQueryResult, PlatformError> {
        let mut query = mcp_tool::Entity::find();

        // 应用过滤条件
        if let Some(tenant_id) = options.tenant_id {
            query = query.filter(mcp_tool::Column::TenantId.eq(tenant_id.0));
        }

        if let Some(status) = &options.status {
            let db_status = match status.as_str() {
                "active" => mcp_tool::ToolStatus::Active,
                "inactive" => mcp_tool::ToolStatus::Inactive,
                _ => return Err(PlatformError::ValidationError(format!("Invalid status: {}", status))),
            };
            query = query.filter(mcp_tool::Column::Status.eq(db_status));
        }

        if let Some(created_by) = options.created_by {
            query = query.filter(mcp_tool::Column::CreatedBy.eq(created_by.0));
        }

        if let Some(name_contains) = &options.name_contains {
            query = query.filter(mcp_tool::Column::Name.contains(name_contains));
        }

        // 排序
        query = query.order_by_desc(mcp_tool::Column::UpdatedAt);

        // 获取总数
        let total_count = query.clone().count(&*self.db).await.map_err(PlatformError::DatabaseError)?;

        // 应用分页
        if let (Some(limit), Some(offset)) = (options.limit, options.offset) {
            query = query.limit(limit).offset(offset);
        }

        // 执行查询
        let models = query.all(&*self.db).await.map_err(PlatformError::DatabaseError)?;

        // 转换为领域实体
        let mut tools = Vec::new();
        for model in models {
            let tool = self.db_entity_to_domain_with_config(model).await?;
            tools.push(tool);
        }

        Ok(MCPToolQueryResult { tools, total_count })
    }

    async fn find_by_tenant_id(&self, tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError> {
        let models = mcp_tool::Entity::find()
            .filter(mcp_tool::Column::TenantId.eq(tenant_id.0))
            .order_by_desc(mcp_tool::Column::UpdatedAt)
            .all(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        let mut tools = Vec::new();
        for model in models {
            let tool = self.db_entity_to_domain_with_config(model).await?;
            tools.push(tool);
        }

        Ok(tools)
    }

    async fn find_by_created_by(&self, created_by: UserId) -> Result<Vec<MCPTool>, PlatformError> {
        let models = mcp_tool::Entity::find()
            .filter(mcp_tool::Column::CreatedBy.eq(created_by.0))
            .order_by_desc(mcp_tool::Column::UpdatedAt)
            .all(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        let mut tools = Vec::new();
        for model in models {
            let tool = self.db_entity_to_domain_with_config(model).await?;
            tools.push(tool);
        }

        Ok(tools)
    }

    async fn save(&self, tool: &MCPTool) -> Result<(), PlatformError> {
        let active_model = self.domain_to_db_entity(tool);

        // 插入工具记录
        active_model.insert(&*self.db).await.map_err(PlatformError::DatabaseError)?;

        // 创建版本记录
        self.create_tool_version(tool, Some("Initial version".to_string())).await?;

        Ok(())
    }

    async fn update(&self, tool: &MCPTool) -> Result<(), PlatformError> {
        let active_model = self.domain_to_db_entity(tool);

        // 更新工具记录
        active_model.update(&*self.db).await.map_err(PlatformError::DatabaseError)?;

        // 如果配置有变化，创建新版本记录
        // 这里简化处理，实际应该检查配置是否真的有变化
        self.create_tool_version(tool, Some("Configuration updated".to_string())).await?;

        Ok(())
    }

    async fn delete(&self, id: MCPToolId) -> Result<(), PlatformError> {
        // 删除工具（级联删除版本记录）
        mcp_tool::Entity::delete_by_id(id.0)
            .exec(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        Ok(())
    }

    async fn exists_by_tenant_and_name(
        &self,
        tenant_id: TenantId,
        name: &str,
        exclude_id: Option<MCPToolId>,
    ) -> Result<bool, PlatformError> {
        let mut query = mcp_tool::Entity::find()
            .filter(mcp_tool::Column::TenantId.eq(tenant_id.0))
            .filter(mcp_tool::Column::Name.eq(name));

        if let Some(exclude_id) = exclude_id {
            query = query.filter(mcp_tool::Column::Id.ne(exclude_id.0));
        }

        let count = query.count(&*self.db).await.map_err(PlatformError::DatabaseError)?;

        Ok(count > 0)
    }

    async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64, PlatformError> {
        let count = mcp_tool::Entity::find()
            .filter(mcp_tool::Column::TenantId.eq(tenant_id.0))
            .count(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        Ok(count)
    }

    async fn find_active_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError> {
        let models = mcp_tool::Entity::find()
            .filter(mcp_tool::Column::TenantId.eq(tenant_id.0))
            .filter(mcp_tool::Column::Status.eq(mcp_tool::ToolStatus::Active))
            .order_by_desc(mcp_tool::Column::UpdatedAt)
            .all(&*self.db)
            .await
            .map_err(PlatformError::DatabaseError)?;

        let mut tools = Vec::new();
        for model in models {
            let tool = self.db_entity_to_domain_with_config(model).await?;
            tools.push(tool);
        }

        Ok(tools)
    }

    async fn get_version_history(&self, tool_id: MCPToolId) -> Result<Vec<MCPToolVersion>, PlatformError> {
        self.version_repo.get_version_history(tool_id).await
    }

    async fn rollback_to_version(
        &self,
        tool_id: MCPToolId,
        target_version: i32,
        created_by: UserId,
        change_log: Option<String>,
    ) -> Result<MCPTool, PlatformError> {
        // 执行版本回退
        let new_version = self.version_repo.rollback_to_version(
            tool_id,
            target_version,
            created_by,
            change_log,
        ).await?;

        // 更新工具的当前版本号
        let mut tool = self.find_by_id(tool_id).await?
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        tool.current_version = new_version.version;
        tool.config = new_version.config.clone();
        tool.updated_at = new_version.created_at;

        // 更新工具记录（不创建新版本，因为版本已经在rollback_to_version中创建）
        let active_model = self.domain_to_db_entity(&tool);
        active_model.update(&*self.db).await.map_err(PlatformError::DatabaseError)?;

        Ok(tool)
    }

    async fn compare_versions(
        &self,
        tool_id: MCPToolId,
        from_version: i32,
        to_version: i32,
    ) -> Result<VersionDiff, PlatformError> {
        self.version_repo.compare_versions(tool_id, from_version, to_version).await
    }

    async fn create_version(
        &self,
        tool: &MCPTool,
        change_log: Option<String>,
    ) -> Result<MCPToolVersion, PlatformError> {
        // 获取下一个版本号
        let next_version = self.version_repo.get_next_version_number(tool.id).await?;

        // 创建新版本
        let version = MCPToolVersion::new(
            tool.id,
            next_version,
            tool.config.clone(),
            change_log,
            tool.created_by,
        );

        // 保存版本
        self.version_repo.save(&version).await?;

        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::tool_config::{HTTPToolConfig, HttpMethod};
    use sea_orm::{Database, DatabaseBackend, MockDatabase, MockExecResult};

    #[tokio::test]
    async fn test_domain_to_db_entity_conversion() {
        let db = Arc::new(MockDatabase::new(DatabaseBackend::MySql).into_connection());
        let repo = MCPToolRepositoryImpl::new(db);

        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let config = ToolConfig::HTTP(HTTPToolConfig::new(
            "https://api.example.com".to_string(),
            HttpMethod::GET,
        ));

        let tool = MCPTool::new(
            tenant_id,
            "test-tool".to_string(),
            Some("Test tool".to_string()),
            config,
            user_id,
        );

        let active_model = repo.domain_to_db_entity(&tool);

        assert_eq!(active_model.id.unwrap(), tool.id.0);
        assert_eq!(active_model.tenant_id.unwrap(), tenant_id.0);
        assert_eq!(active_model.name.unwrap(), "test-tool");
        assert_eq!(active_model.created_by.unwrap(), user_id.0);
    }

    #[tokio::test]
    async fn test_status_conversion() {
        let db = Arc::new(MockDatabase::new(DatabaseBackend::MySql).into_connection());
        let repo = MCPToolRepositoryImpl::new(db);

        // Test domain to DB conversion
        let active_status = crate::domain::entities::MCPToolStatus::Active;
        let db_status = repo.domain_status_to_db(&active_status);
        assert_eq!(db_status, mcp_tool::ToolStatus::Active);

        // Test DB to domain conversion
        let domain_status = repo.db_status_to_domain(&mcp_tool::ToolStatus::Active);
        assert_eq!(domain_status, crate::domain::entities::MCPToolStatus::Active);
    }
}