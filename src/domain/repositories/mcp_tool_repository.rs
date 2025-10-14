use async_trait::async_trait;

use crate::domain::{
    entities::{MCPTool, MCPToolVersion, VersionDiff},
    value_objects::ids::{MCPToolId, TenantId, UserId},
};
use crate::error::PlatformError;

/// MCP工具查询选项
#[derive(Debug, Clone, Default)]
pub struct MCPToolQueryOptions {
    pub tenant_id: Option<TenantId>,
    pub status: Option<String>,
    pub created_by: Option<UserId>,
    pub name_contains: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl MCPToolQueryOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tenant_id(mut self, tenant_id: TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_created_by(mut self, created_by: UserId) -> Self {
        self.created_by = Some(created_by);
        self
    }

    pub fn with_name_contains(mut self, name_contains: String) -> Self {
        self.name_contains = Some(name_contains);
        self
    }

    pub fn with_pagination(mut self, limit: u64, offset: u64) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

/// MCP工具查询结果
#[derive(Debug, Clone)]
pub struct MCPToolQueryResult {
    pub tools: Vec<MCPTool>,
    pub total_count: u64,
}

/// MCP工具仓储接口
#[async_trait]
pub trait MCPToolRepository: Send + Sync {
    /// 根据ID查找工具
    async fn find_by_id(&self, id: MCPToolId) -> Result<Option<MCPTool>, PlatformError>;

    /// 根据租户ID和名称查找工具
    async fn find_by_tenant_and_name(
        &self,
        tenant_id: TenantId,
        name: &str,
    ) -> Result<Option<MCPTool>, PlatformError>;

    /// 查询工具列表
    async fn find_by_options(&self, options: MCPToolQueryOptions) -> Result<MCPToolQueryResult, PlatformError>;

    /// 根据租户ID查找所有工具
    async fn find_by_tenant_id(&self, tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError>;

    /// 根据创建者查找工具
    async fn find_by_created_by(&self, created_by: UserId) -> Result<Vec<MCPTool>, PlatformError>;

    /// 保存工具
    async fn save(&self, tool: &MCPTool) -> Result<(), PlatformError>;

    /// 更新工具
    async fn update(&self, tool: &MCPTool) -> Result<(), PlatformError>;

    /// 删除工具
    async fn delete(&self, id: MCPToolId) -> Result<(), PlatformError>;

    /// 检查工具名称是否存在
    async fn exists_by_tenant_and_name(
        &self,
        tenant_id: TenantId,
        name: &str,
        exclude_id: Option<MCPToolId>,
    ) -> Result<bool, PlatformError>;

    /// 统计租户的工具数量
    async fn count_by_tenant(&self, tenant_id: TenantId) -> Result<u64, PlatformError>;

    /// 获取活跃工具列表
    async fn find_active_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<MCPTool>, PlatformError>;

    /// 获取工具的版本历史
    async fn get_version_history(&self, tool_id: MCPToolId) -> Result<Vec<MCPToolVersion>, PlatformError>;

    /// 回退工具到指定版本
    async fn rollback_to_version(
        &self,
        tool_id: MCPToolId,
        target_version: i32,
        created_by: UserId,
        change_log: Option<String>,
    ) -> Result<MCPTool, PlatformError>;

    /// 比较两个版本的差异
    async fn compare_versions(
        &self,
        tool_id: MCPToolId,
        from_version: i32,
        to_version: i32,
    ) -> Result<VersionDiff, PlatformError>;

    /// 创建新版本
    async fn create_version(
        &self,
        tool: &MCPTool,
        change_log: Option<String>,
    ) -> Result<MCPToolVersion, PlatformError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_options_builder() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();

        let options = MCPToolQueryOptions::new()
            .with_tenant_id(tenant_id)
            .with_status("active".to_string())
            .with_created_by(user_id)
            .with_name_contains("test".to_string())
            .with_pagination(10, 0);

        assert_eq!(options.tenant_id, Some(tenant_id));
        assert_eq!(options.status, Some("active".to_string()));
        assert_eq!(options.created_by, Some(user_id));
        assert_eq!(options.name_contains, Some("test".to_string()));
        assert_eq!(options.limit, Some(10));
        assert_eq!(options.offset, Some(0));
    }
}