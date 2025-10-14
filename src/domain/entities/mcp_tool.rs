use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


use crate::domain::value_objects::{
    ids::{TenantId, UserId, MCPToolId},
    tool_config::ToolConfig,
};

/// MCP工具状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MCPToolStatus {
    Active,
    Inactive,
    Testing,
}

/// MCP工具领域实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MCPTool {
    pub id: MCPToolId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub current_version: i32,
    pub status: MCPToolStatus,
    pub config: ToolConfig,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MCPTool {
    /// 创建新的MCP工具
    pub fn new(
        tenant_id: TenantId,
        name: String,
        description: Option<String>,
        config: ToolConfig,
        created_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: MCPToolId::new(),
            tenant_id,
            name,
            description,
            current_version: 1,
            status: MCPToolStatus::Testing,
            config,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }

    /// 验证工具名称
    pub fn validate_name(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Tool name cannot be empty".to_string());
        }
        if self.name.len() > 255 {
            return Err("Tool name cannot exceed 255 characters".to_string());
        }
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Tool name can only contain alphanumeric characters, underscores, and hyphens".to_string());
        }
        Ok(())
    }

    /// 检查工具是否可以执行
    pub fn can_execute(&self) -> bool {
        matches!(self.status, MCPToolStatus::Active)
    }

    /// 激活工具
    pub fn activate(&mut self) {
        self.status = MCPToolStatus::Active;
        self.updated_at = Utc::now();
    }

    /// 停用工具
    pub fn deactivate(&mut self) {
        self.status = MCPToolStatus::Inactive;
        self.updated_at = Utc::now();
    }

    /// 更新配置
    pub fn update_config(&mut self, config: ToolConfig) {
        self.config = config;
        self.current_version += 1;
        self.updated_at = Utc::now();
    }

    /// 更新工具名称
    pub fn update_name(&mut self, name: String) -> Result<(), String> {
        if name.is_empty() {
            return Err("Tool name cannot be empty".to_string());
        }
        if name.len() > 255 {
            return Err("Tool name cannot exceed 255 characters".to_string());
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Tool name can only contain alphanumeric characters, underscores, and hyphens".to_string());
        }
        
        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 更新工具描述
    pub fn update_description(&mut self, description: Option<String>) -> Result<(), String> {
        if let Some(ref desc) = description {
            if desc.len() > 1000 {
                return Err("Description cannot exceed 1000 characters".to_string());
            }
        }
        
        self.description = description;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 检查用户是否有权限访问此工具
    pub fn can_access(&self, user_tenant_id: &TenantId) -> bool {
        &self.tenant_id == user_tenant_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mcp_tool() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let config = ToolConfig::default();
        
        let tool = MCPTool::new(
            tenant_id,
            "test-tool".to_string(),
            Some("Test tool description".to_string()),
            config,
            user_id,
        );

        assert_eq!(tool.tenant_id, tenant_id);
        assert_eq!(tool.name, "test-tool");
        assert_eq!(tool.current_version, 1);
        assert_eq!(tool.status, MCPToolStatus::Testing);
        assert_eq!(tool.created_by, user_id);
    }

    #[test]
    fn test_validate_name() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let config = ToolConfig::default();

        // Valid name
        let tool = MCPTool::new(tenant_id, "valid-tool_name".to_string(), None, config.clone(), user_id);
        assert!(tool.validate_name().is_ok());

        // Empty name
        let tool = MCPTool::new(tenant_id, "".to_string(), None, config.clone(), user_id);
        assert!(tool.validate_name().is_err());

        // Invalid characters
        let tool = MCPTool::new(tenant_id, "invalid@tool".to_string(), None, config.clone(), user_id);
        assert!(tool.validate_name().is_err());
    }

    #[test]
    fn test_can_execute() {
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let config = ToolConfig::default();
        
        let mut tool = MCPTool::new(tenant_id, "test-tool".to_string(), None, config, user_id);
        
        // Initially in testing status
        assert!(!tool.can_execute());
        
        // After activation
        tool.activate();
        assert!(tool.can_execute());
        
        // After deactivation
        tool.deactivate();
        assert!(!tool.can_execute());
    }

    #[test]
    fn test_can_access() {
        let tenant_id = TenantId::new();
        let other_tenant_id = TenantId::new();
        let user_id = UserId::new();
        let config = ToolConfig::default();
        
        let tool = MCPTool::new(tenant_id, "test-tool".to_string(), None, config, user_id);
        
        assert!(tool.can_access(&tenant_id));
        assert!(!tool.can_access(&other_tenant_id));
    }
}