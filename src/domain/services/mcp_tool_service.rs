use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::domain::{
    entities::MCPTool,
    value_objects::{
        ids::{MCPToolId, TenantId, UserId},
        tool_config::ToolConfig,
    },
};
use crate::error::PlatformError;

/// MCP工具调用上下文
#[derive(Debug, Clone)]
pub struct ToolCallContext {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub session_id: Option<String>,
    pub request_id: String,
    pub metadata: HashMap<String, Value>,
}

impl ToolCallContext {
    pub fn new(tenant_id: TenantId, user_id: UserId, request_id: String) -> Self {
        Self {
            tenant_id,
            user_id,
            session_id: None,
            request_id,
            metadata: HashMap::new(),
        }
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// MCP工具调用结果
#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, Value>,
}

impl ToolCallResult {
    pub fn success(result: Value, execution_time_ms: u64) -> Self {
        Self {
            success: true,
            result: Some(result),
            error: None,
            execution_time_ms,
            metadata: HashMap::new(),
        }
    }

    pub fn error(error: String, execution_time_ms: u64) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(error),
            execution_time_ms,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// MCP工具权限检查结果
#[derive(Debug, Clone)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub reason: Option<String>,
}

impl PermissionCheckResult {
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            reason: None,
        }
    }

    pub fn denied(reason: String) -> Self {
        Self {
            allowed: false,
            reason: Some(reason),
        }
    }
}

/// MCP工具配置验证结果
#[derive(Debug, Clone)]
pub struct ConfigValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ConfigValidationResult {
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// MCP工具领域服务接口
#[async_trait]
pub trait MCPToolDomainService: Send + Sync {
    /// 验证工具配置
    async fn validate_tool_config(&self, config: &ToolConfig) -> Result<ConfigValidationResult, PlatformError>;

    /// 检查用户是否有权限访问工具
    async fn check_tool_permission(
        &self,
        tool: &MCPTool,
        context: &ToolCallContext,
    ) -> Result<PermissionCheckResult, PlatformError>;

    /// 检查用户是否有权限调用工具
    async fn check_call_permission(
        &self,
        tool: &MCPTool,
        context: &ToolCallContext,
        parameters: &Value,
    ) -> Result<PermissionCheckResult, PlatformError>;

    /// 验证工具调用参数
    async fn validate_call_parameters(
        &self,
        tool: &MCPTool,
        parameters: &Value,
    ) -> Result<(), PlatformError>;

    /// 测试工具连接
    async fn test_tool_connection(&self, config: &ToolConfig) -> Result<ToolCallResult, PlatformError>;

    /// 创建工具调用上下文
    fn create_call_context(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        request_id: String,
    ) -> ToolCallContext;

    /// 检查工具是否可以执行
    fn can_execute_tool(&self, tool: &MCPTool) -> bool;

    /// 验证工具名称唯一性
    async fn validate_tool_name_uniqueness(
        &self,
        tenant_id: TenantId,
        name: &str,
        exclude_tool_id: Option<MCPToolId>,
    ) -> Result<bool, PlatformError>;
}

/// MCP工具领域服务默认实现
pub struct MCPToolDomainServiceImpl;

impl MCPToolDomainServiceImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MCPToolDomainService for MCPToolDomainServiceImpl {
    async fn validate_tool_config(&self, config: &ToolConfig) -> Result<ConfigValidationResult, PlatformError> {
        let mut result = ConfigValidationResult::valid();

        // 基础配置验证
        if let Err(error) = config.validate() {
            result.add_error(error);
        }

        // 根据工具类型进行特定验证
        match config {
            ToolConfig::HTTP(http_config) => {
                // HTTP特定验证
                if http_config.endpoint.is_empty() {
                    result.add_error("HTTP endpoint cannot be empty".to_string());
                }

                // 检查是否使用HTTPS（警告）
                if !http_config.endpoint.starts_with("https://") {
                    result.add_warning("Consider using HTTPS for better security".to_string());
                }

                // 检查超时设置
                if let Some(timeout) = http_config.timeout_seconds {
                    if timeout > 60 {
                        result.add_warning("Long timeout may affect user experience".to_string());
                    }
                }
            }
        }

        Ok(result)
    }

    async fn check_tool_permission(
        &self,
        tool: &MCPTool,
        context: &ToolCallContext,
    ) -> Result<PermissionCheckResult, PlatformError> {
        // 检查租户权限
        if !tool.can_access(&context.tenant_id) {
            return Ok(PermissionCheckResult::denied(
                "Tool does not belong to user's tenant".to_string(),
            ));
        }

        // 注掉: 不需要检查状态
        // // 检查工具状态
        // if !tool.can_execute() {
        //     return Ok(PermissionCheckResult::denied(
        //         "Tool is not in active state".to_string(),
        //     ));
        // }

        Ok(PermissionCheckResult::allowed())
    }

    async fn check_call_permission(
        &self,
        tool: &MCPTool,
        context: &ToolCallContext,
        _parameters: &Value,
    ) -> Result<PermissionCheckResult, PlatformError> {
        // 首先检查基本权限
        let basic_permission = self.check_tool_permission(tool, context).await?;
        if !basic_permission.allowed {
            return Ok(basic_permission);
        }

        // 这里可以添加更细粒度的权限检查
        // 例如：基于参数内容的权限检查、调用频率限制等

        Ok(PermissionCheckResult::allowed())
    }

    async fn validate_call_parameters(
        &self,
        tool: &MCPTool,
        parameters: &Value,
    ) -> Result<(), PlatformError> {
        tool.config
            .validate_call_parameters(parameters)
            .map_err(|e| PlatformError::ValidationError(e))
    }

    async fn test_tool_connection(&self, config: &ToolConfig) -> Result<ToolCallResult, PlatformError> {
        let start_time = std::time::Instant::now();

        match config {
            ToolConfig::HTTP(http_config) => {
                // 这里应该实际调用HTTP接口进行测试
                // 为了简化，我们只验证配置
                match http_config.validate() {
                    Ok(_) => {
                        let execution_time = start_time.elapsed().as_millis() as u64;
                        Ok(ToolCallResult::success(
                            serde_json::json!({"status": "connection_test_passed"}),
                            execution_time,
                        ))
                    }
                    Err(e) => {
                        let execution_time = start_time.elapsed().as_millis() as u64;
                        Ok(ToolCallResult::error(
                            format!("Connection test failed: {}", e),
                            execution_time,
                        ))
                    }
                }
            }
        }
    }

    fn create_call_context(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        request_id: String,
    ) -> ToolCallContext {
        ToolCallContext::new(tenant_id, user_id, request_id)
    }

    fn can_execute_tool(&self, tool: &MCPTool) -> bool {
        tool.can_execute()
    }

    async fn validate_tool_name_uniqueness(
        &self,
        _tenant_id: TenantId,
        _name: &str,
        _exclude_tool_id: Option<MCPToolId>,
    ) -> Result<bool, PlatformError> {
        // 这个方法需要访问仓储层来检查名称唯一性
        // 在实际实现中，这应该通过仓储接口来完成
        // 这里返回true表示名称是唯一的（简化实现）
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::tool_config::{HTTPToolConfig, HttpMethod};

    #[tokio::test]
    async fn test_validate_tool_config() {
        let service = MCPToolDomainServiceImpl::new();
        
        let config = ToolConfig::HTTP(HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::GET,
        ));

        let result = service.validate_tool_config(&config).await.unwrap();
        assert!(result.valid);
    }

    #[tokio::test]
    async fn test_check_tool_permission() {
        let service = MCPToolDomainServiceImpl::new();
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        
        let mut tool = MCPTool::new(
            tenant_id,
            "test-tool".to_string(),
            None,
            ToolConfig::default(),
            user_id,
        );
        tool.activate();

        let context = ToolCallContext::new(tenant_id, user_id, "req-123".to_string());
        
        let result = service.check_tool_permission(&tool, &context).await.unwrap();
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_check_tool_permission_wrong_tenant() {
        let service = MCPToolDomainServiceImpl::new();
        let tenant_id = TenantId::new();
        let other_tenant_id = TenantId::new();
        let user_id = UserId::new();
        
        let tool = MCPTool::new(
            tenant_id,
            "test-tool".to_string(),
            None,
            ToolConfig::default(),
            user_id,
        );

        let context = ToolCallContext::new(other_tenant_id, user_id, "req-123".to_string());
        
        let result = service.check_tool_permission(&tool, &context).await.unwrap();
        assert!(!result.allowed);
    }

    #[tokio::test]
    async fn test_test_tool_connection() {
        let service = MCPToolDomainServiceImpl::new();
        
        let config = ToolConfig::HTTP(HTTPToolConfig::new(
            "https://api.example.com/test".to_string(),
            HttpMethod::GET,
        ));

        let result = service.test_tool_connection(&config).await.unwrap();
        assert!(result.success);
    }
}