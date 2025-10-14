use async_trait::async_trait;

use crate::domain::{
    entities::{MCPToolVersion, VersionDiff},
    value_objects::ids::{MCPToolId, MCPToolVersionId, UserId},
};
use crate::error::PlatformError;

/// MCP工具版本查询选项
#[derive(Debug, Clone, Default)]
pub struct MCPToolVersionQueryOptions {
    pub tool_id: Option<MCPToolId>,
    pub version: Option<i32>,
    pub created_by: Option<UserId>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl MCPToolVersionQueryOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tool_id(mut self, tool_id: MCPToolId) -> Self {
        self.tool_id = Some(tool_id);
        self
    }

    pub fn with_version(mut self, version: i32) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_created_by(mut self, created_by: UserId) -> Self {
        self.created_by = Some(created_by);
        self
    }

    pub fn with_pagination(mut self, limit: u64, offset: u64) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

/// MCP工具版本查询结果
#[derive(Debug, Clone)]
pub struct MCPToolVersionQueryResult {
    pub versions: Vec<MCPToolVersion>,
    pub total_count: u64,
}

/// MCP工具版本仓储接口
#[async_trait]
pub trait MCPToolVersionRepository: Send + Sync {
    /// 根据ID查找版本
    async fn find_by_id(&self, id: MCPToolVersionId) -> Result<Option<MCPToolVersion>, PlatformError>;

    /// 根据工具ID和版本号查找版本
    async fn find_by_tool_and_version(
        &self,
        tool_id: MCPToolId,
        version: i32,
    ) -> Result<Option<MCPToolVersion>, PlatformError>;

    /// 查询版本列表
    async fn find_by_options(&self, options: MCPToolVersionQueryOptions) -> Result<MCPToolVersionQueryResult, PlatformError>;

    /// 根据工具ID查找所有版本
    async fn find_by_tool_id(&self, tool_id: MCPToolId) -> Result<Vec<MCPToolVersion>, PlatformError>;

    /// 获取工具的最新版本
    async fn find_latest_by_tool_id(&self, tool_id: MCPToolId) -> Result<Option<MCPToolVersion>, PlatformError>;

    /// 获取工具的指定数量的最新版本
    async fn find_recent_by_tool_id(&self, tool_id: MCPToolId, limit: u64) -> Result<Vec<MCPToolVersion>, PlatformError>;

    /// 保存版本
    async fn save(&self, version: &MCPToolVersion) -> Result<(), PlatformError>;

    /// 更新版本
    async fn update(&self, version: &MCPToolVersion) -> Result<(), PlatformError>;

    /// 删除版本
    async fn delete(&self, id: MCPToolVersionId) -> Result<(), PlatformError>;

    /// 删除工具的所有版本
    async fn delete_by_tool_id(&self, tool_id: MCPToolId) -> Result<(), PlatformError>;

    /// 检查版本是否存在
    async fn exists_by_tool_and_version(
        &self,
        tool_id: MCPToolId,
        version: i32,
    ) -> Result<bool, PlatformError>;

    /// 获取工具的下一个版本号
    async fn get_next_version_number(&self, tool_id: MCPToolId) -> Result<i32, PlatformError>;

    /// 统计工具的版本数量
    async fn count_by_tool_id(&self, tool_id: MCPToolId) -> Result<u64, PlatformError>;

    /// 比较两个版本
    async fn compare_versions(
        &self,
        tool_id: MCPToolId,
        from_version: i32,
        to_version: i32,
    ) -> Result<VersionDiff, PlatformError>;

    /// 获取版本历史（按版本号倒序）
    async fn get_version_history(&self, tool_id: MCPToolId) -> Result<Vec<MCPToolVersion>, PlatformError>;

    /// 回退到指定版本（创建新版本记录）
    async fn rollback_to_version(
        &self,
        tool_id: MCPToolId,
        target_version: i32,
        created_by: UserId,
        change_log: Option<String>,
    ) -> Result<MCPToolVersion, PlatformError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_query_options_builder() {
        let tool_id = MCPToolId::new();
        let user_id = UserId::new();

        let options = MCPToolVersionQueryOptions::new()
            .with_tool_id(tool_id)
            .with_version(1)
            .with_created_by(user_id)
            .with_pagination(10, 0);

        assert_eq!(options.tool_id, Some(tool_id));
        assert_eq!(options.version, Some(1));
        assert_eq!(options.created_by, Some(user_id));
        assert_eq!(options.limit, Some(10));
        assert_eq!(options.offset, Some(0));
    }
}