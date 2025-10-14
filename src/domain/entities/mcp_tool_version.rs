use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{
    ids::{MCPToolId, MCPToolVersionId, UserId},
    tool_config::ToolConfig,
};

/// MCP工具版本领域实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MCPToolVersion {
    pub id: MCPToolVersionId,
    pub tool_id: MCPToolId,
    pub version: i32,
    pub config: ToolConfig,
    pub change_log: Option<String>,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
}

impl MCPToolVersion {
    /// 创建新的工具版本
    pub fn new(
        tool_id: MCPToolId,
        version: i32,
        config: ToolConfig,
        change_log: Option<String>,
        created_by: UserId,
    ) -> Self {
        Self {
            id: MCPToolVersionId::new(),
            tool_id,
            version,
            config,
            change_log,
            created_by,
            created_at: Utc::now(),
        }
    }

    /// 验证版本号
    pub fn validate_version(&self) -> Result<(), String> {
        if self.version <= 0 {
            return Err("Version must be positive".to_string());
        }
        Ok(())
    }

    /// 验证配置
    pub fn validate_config(&self) -> Result<(), String> {
        self.config.validate()
    }

    /// 检查是否为初始版本
    pub fn is_initial_version(&self) -> bool {
        self.version == 1
    }

    /// 获取版本描述
    pub fn get_version_description(&self) -> String {
        match &self.change_log {
            Some(log) => format!("v{}: {}", self.version, log),
            None => format!("v{}", self.version),
        }
    }

    /// 比较两个版本
    pub fn compare_version(&self, other: &MCPToolVersion) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }

    /// 检查是否比另一个版本更新
    pub fn is_newer_than(&self, other: &MCPToolVersion) -> bool {
        self.version > other.version
    }

    /// 检查是否比另一个版本更旧
    pub fn is_older_than(&self, other: &MCPToolVersion) -> bool {
        self.version < other.version
    }
}

/// 版本比较结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionDiff {
    pub from_version: i32,
    pub to_version: i32,
    pub config_changed: bool,
    pub changes: Vec<String>,
}

impl VersionDiff {
    /// 创建版本差异
    pub fn new(from: &MCPToolVersion, to: &MCPToolVersion) -> Self {
        let config_changed = from.config != to.config;
        let mut changes = Vec::new();

        if config_changed {
            changes.push("Configuration updated".to_string());
        }

        if let (Some(from_log), Some(to_log)) = (&from.change_log, &to.change_log) {
            if from_log != to_log {
                changes.push(format!("Change log updated: {}", to_log));
            }
        } else if to.change_log.is_some() {
            changes.push("Change log added".to_string());
        }

        Self {
            from_version: from.version,
            to_version: to.version,
            config_changed,
            changes,
        }
    }

    /// 检查是否有重大变更
    pub fn has_major_changes(&self) -> bool {
        self.config_changed
    }

    /// 获取变更摘要
    pub fn get_summary(&self) -> String {
        if self.changes.is_empty() {
            "No changes detected".to_string()
        } else {
            self.changes.join("; ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::tool_config::{HTTPToolConfig, HttpMethod};

    #[test]
    fn test_new_mcp_tool_version() {
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

        assert_eq!(version.tool_id, tool_id);
        assert_eq!(version.version, 1);
        assert_eq!(version.change_log, Some("Initial version".to_string()));
        assert_eq!(version.created_by, user_id);
        assert!(version.is_initial_version());
    }

    #[test]
    fn test_validate_version() {
        let tool_id = MCPToolId::new();
        let user_id = UserId::new();
        let config = ToolConfig::default();

        // Valid version
        let version = MCPToolVersion::new(tool_id, 1, config.clone(), None, user_id);
        assert!(version.validate_version().is_ok());

        // Invalid version
        let version = MCPToolVersion::new(tool_id, 0, config, None, user_id);
        assert!(version.validate_version().is_err());
    }

    #[test]
    fn test_version_comparison() {
        let tool_id = MCPToolId::new();
        let user_id = UserId::new();
        let config = ToolConfig::default();

        let version1 = MCPToolVersion::new(tool_id, 1, config.clone(), None, user_id);
        let version2 = MCPToolVersion::new(tool_id, 2, config, None, user_id);

        assert!(version2.is_newer_than(&version1));
        assert!(version1.is_older_than(&version2));
        assert_eq!(version1.compare_version(&version2), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_version_diff() {
        let tool_id = MCPToolId::new();
        let user_id = UserId::new();
        let config1 = ToolConfig::HTTP(HTTPToolConfig::new(
            "https://api.example.com".to_string(),
            HttpMethod::GET,
        ));
        let config2 = ToolConfig::HTTP(HTTPToolConfig::new(
            "https://api.example.com/v2".to_string(),
            HttpMethod::POST,
        ));

        let version1 = MCPToolVersion::new(
            tool_id,
            1,
            config1,
            Some("Initial version".to_string()),
            user_id,
        );
        let version2 = MCPToolVersion::new(
            tool_id,
            2,
            config2,
            Some("Updated API endpoint".to_string()),
            user_id,
        );

        let diff = VersionDiff::new(&version1, &version2);

        assert_eq!(diff.from_version, 1);
        assert_eq!(diff.to_version, 2);
        assert!(diff.config_changed);
        assert!(diff.has_major_changes());
        assert!(!diff.changes.is_empty());
    }

    #[test]
    fn test_get_version_description() {
        let tool_id = MCPToolId::new();
        let user_id = UserId::new();
        let config = ToolConfig::default();

        let version_with_log = MCPToolVersion::new(
            tool_id,
            1,
            config.clone(),
            Some("Initial version".to_string()),
            user_id,
        );
        assert_eq!(version_with_log.get_version_description(), "v1: Initial version");

        let version_without_log = MCPToolVersion::new(tool_id, 2, config, None, user_id);
        assert_eq!(version_without_log.get_version_description(), "v2");
    }
}