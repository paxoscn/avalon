use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::{
        entities::{MCPTool, MCPToolVersion},
        repositories::{MCPToolRepository, MCPToolVersionRepository},
        services::mcp_tool_service::{
            MCPToolDomainService, ToolCallContext, 
            ConfigValidationResult
        },
        value_objects::{
            ids::{MCPToolId, TenantId, UserId},
            tool_config::ToolConfig,
        },
    },
    application::dto::{
        CreateMCPToolRequest, UpdateMCPToolRequest, MCPToolResponse,
        CallMCPToolRequest, CallMCPToolResponse,
        TestMCPToolRequest, TestMCPToolResponse, MCPToolVersionResponse,
        MCPToolStatsResponse,
    },
    error::{PlatformError, Result},
    infrastructure::mcp::{
        MCPProxyService,
        mcp_server_handler::MCPServerHandler,
        mcp_protocol::{MCPToolListResponse, MCPToolCallResponse},
        template_engine::ResponseTemplateEngine,
    },
};

/// MCP工具管理应用服务接口
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MCPApplicationService: Send + Sync {
    /// 创建MCP工具
    async fn create_tool(
        &self,
        request: CreateMCPToolRequest,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<MCPToolResponse>;

    /// 更新MCP工具
    async fn update_tool(
        &self,
        tool_id: MCPToolId,
        request: UpdateMCPToolRequest,
        user_id: UserId,
    ) -> Result<MCPToolResponse>;

    /// 删除MCP工具
    async fn delete_tool(
        &self,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<()>;

    /// 获取MCP工具详情
    async fn get_tool(
        &self,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<MCPToolResponse>;

    /// 获取租户的MCP工具列表
    async fn list_tools(
        &self,
        tenant_id: TenantId,
        page: u64,
        limit: u64,
    ) -> Result<(Vec<MCPTool>, u64)>;

    /// 调用MCP工具
    async fn call_tool(
        &self,
        tool_id: MCPToolId,
        request: CallMCPToolRequest,
        context: ToolCallContext,
    ) -> Result<CallMCPToolResponse>;

    /// 测试MCP工具连接
    async fn test_tool(
        &self,
        tool_id: MCPToolId,
        request: TestMCPToolRequest,
        user_id: UserId,
    ) -> Result<TestMCPToolResponse>;

    /// 激活MCP工具
    async fn activate_tool(
        &self,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<MCPToolResponse>;

    /// 停用MCP工具
    async fn deactivate_tool(
        &self,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<MCPToolResponse>;

    /// 获取工具版本列表
    async fn list_tool_versions(
        &self,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<Vec<MCPToolVersionResponse>>;

    /// 回退到指定版本
    async fn rollback_tool_version(
        &self,
        tool_id: MCPToolId,
        target_version: i32,
        user_id: UserId,
    ) -> Result<MCPToolResponse>;

    /// 获取工具统计信息
    async fn get_tool_stats(
        &self,
        tenant_id: TenantId,
    ) -> Result<MCPToolStatsResponse>;

    /// 验证工具配置
    async fn validate_tool_config(
        &self,
        config: &ToolConfig,
    ) -> Result<ConfigValidationResult>;

    /// 获取MCP格式的工具列表（用于MCP Server接口）
    async fn list_tools_for_mcp(
        &self,
        tenant_id: TenantId,
        page: Option<u64>,
        limit: Option<u64>,
    ) -> Result<MCPToolListResponse>;

    /// 通过MCP格式调用工具（用于MCP Server接口）
    async fn call_tool_via_mcp(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        tool_name: String,
        arguments: serde_json::Value,
    ) -> Result<MCPToolCallResponse>;
}

/// MCP工具管理应用服务实现
pub struct MCPApplicationServiceImpl {
    tool_repository: Arc<dyn MCPToolRepository>,
    version_repository: Arc<dyn MCPToolVersionRepository>,
    domain_service: Arc<dyn MCPToolDomainService>,
    proxy_service: Arc<dyn MCPProxyService>,
    mcp_server_handler: Arc<MCPServerHandler>,
    template_engine: Arc<ResponseTemplateEngine>,
}

impl MCPApplicationServiceImpl {
    pub fn new(
        tool_repository: Arc<dyn MCPToolRepository>,
        version_repository: Arc<dyn MCPToolVersionRepository>,
        domain_service: Arc<dyn MCPToolDomainService>,
        proxy_service: Arc<dyn MCPProxyService>,
    ) -> Self {
        let mcp_server_handler = Arc::new(MCPServerHandler::new(
            tool_repository.clone(),
            proxy_service.clone(),
        ));
        let template_engine = Arc::new(ResponseTemplateEngine::new());

        Self {
            tool_repository,
            version_repository,
            domain_service,
            proxy_service,
            mcp_server_handler,
            template_engine,
        }
    }

    /// 验证用户对工具的访问权限
    async fn validate_tool_access(
        &self,
        tool: &MCPTool,
        user_id: UserId,
    ) -> Result<()> {
        // 检查工具是否属于用户的租户
        let context = self.domain_service.create_call_context(
            tool.tenant_id,
            user_id,
            Uuid::new_v4().to_string(),
        );

        let permission_result = self.domain_service
            .check_tool_permission(tool, &context)
            .await?;

        if !permission_result.allowed {
            return Err(PlatformError::AuthorizationFailed(
                permission_result.reason.unwrap_or_else(|| "Access denied".to_string())
            ));
        }

        Ok(())
    }

    /// 创建工具版本
    async fn create_tool_version(
        &self,
        tool: &MCPTool,
        change_log: Option<String>,
        user_id: UserId,
    ) -> Result<MCPToolVersion> {
        let version = MCPToolVersion::new(
            tool.id,
            tool.current_version,
            tool.config.clone(),
            change_log,
            user_id,
        );

        self.version_repository.save(&version).await?;
        Ok(version)
    }

    /// 将领域实体转换为响应DTO
    fn tool_to_response(&self, tool: &MCPTool) -> MCPToolResponse {
        MCPToolResponse {
            id: tool.id.0,
            tenant_id: tool.tenant_id.0,
            name: tool.name.clone(),
            description: tool.description.clone(),
            current_version: tool.current_version,
            status: tool.status.clone(),
            config: tool.config.clone(),
            created_by: tool.created_by.0,
            created_at: tool.created_at,
            updated_at: tool.updated_at,
        }
    }
}

#[async_trait]
impl MCPApplicationService for MCPApplicationServiceImpl {
    async fn create_tool(
        &self,
        request: CreateMCPToolRequest,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<MCPToolResponse> {
        // 验证配置（包括路径参数一致性和header命名规范）
        let validation_result = self.domain_service
            .validate_tool_config(&request.config)
            .await?;

        if !validation_result.valid {
            return Err(PlatformError::ValidationError(
                validation_result.errors.join(", ")
            ));
        }

        // 验证响应模板语法（如果配置了模板）
        if let ToolConfig::HTTP(ref http_config) = request.config {
            if let Some(ref template) = http_config.response_template {
                self.template_engine.validate_template(template)
                    .map_err(|e| PlatformError::ValidationError(
                        format!("Invalid response template: {}", e)
                    ))?;
            }
        }

        // 检查名称唯一性
        let is_unique = self.domain_service
            .validate_tool_name_uniqueness(tenant_id, &request.name, None)
            .await?;

        if !is_unique {
            return Err(PlatformError::ValidationError(
                "Tool name already exists in this tenant".to_string()
            ));
        }

        // 创建工具
        let tool = MCPTool::new(
            tenant_id,
            request.name,
            request.description,
            request.config,
            user_id,
        );

        // 保存工具
        self.tool_repository.save(&tool).await?;

        // 注掉. 在tool_repository中已经保存
        // // 创建初始版本
        // self.create_tool_version(&tool, Some("Initial version".to_string()), user_id).await?;

        // 注册到代理服务
        self.proxy_service.register_tool(tool.clone()).await?;

        Ok(self.tool_to_response(&tool))
    }

    async fn update_tool(
        &self,
        tool_id: MCPToolId,
        request: UpdateMCPToolRequest,
        user_id: UserId,
    ) -> Result<MCPToolResponse> {
        // 获取工具
        let mut tool = self.tool_repository
            .find_by_id(tool_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 验证访问权限
        self.validate_tool_access(&tool, user_id).await?;

        // 更新工具信息
        if let Some(name) = request.name {
            // 检查名称唯一性（排除当前工具）
            let is_unique = self.domain_service
                .validate_tool_name_uniqueness(tool.tenant_id, &name, Some(tool_id))
                .await?;

            if !is_unique {
                return Err(PlatformError::ValidationError(
                    "Tool name already exists in this tenant".to_string()
                ));
            }

            tool.update_name(name)?;
        }

        if let Some(description) = request.description {
            tool.update_description(Some(description))?;
        }

        if let Some(config) = request.config {
            // 验证新配置（包括路径参数一致性和header命名规范）
            let validation_result = self.domain_service
                .validate_tool_config(&config)
                .await?;

            if !validation_result.valid {
                return Err(PlatformError::ValidationError(
                    validation_result.errors.join(", ")
                ));
            }

            // 验证响应模板语法（如果配置了模板）
            if let ToolConfig::HTTP(ref http_config) = config {
                if let Some(ref template) = http_config.response_template {
                    self.template_engine.validate_template(template)
                        .map_err(|e| PlatformError::ValidationError(
                            format!("Invalid response template: {}", e)
                        ))?;
                }
            }

            tool.update_config(config);

            // 清除模板缓存（配置更新时）
            self.template_engine.clear_cache(&tool.id.to_string());

            // 注掉: 在tool_repository中处理
            // // 创建新版本
            // self.create_tool_version(&tool, request.change_log, user_id).await?;
        }

        // 保存工具
        self.tool_repository.update(&tool).await?;

        // 更新代理服务
        self.proxy_service.register_tool(tool.clone()).await?;

        Ok(self.tool_to_response(&tool))
    }

    async fn delete_tool(
        &self,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<()> {
        // 获取工具
        let tool = self.tool_repository
            .find_by_id(tool_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 验证访问权限
        self.validate_tool_access(&tool, user_id).await?;

        // 清除模板缓存（工具删除时）
        self.template_engine.clear_cache(&tool_id.to_string());

        // 从代理服务注销
        self.proxy_service.unregister_tool(tool_id).await?;

        // 删除工具（级联删除版本）
        self.tool_repository.delete(tool_id).await?;

        Ok(())
    }

    async fn get_tool(
        &self,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<MCPToolResponse> {
        // 获取工具
        let tool = self.tool_repository
            .find_by_id(tool_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 验证访问权限
        self.validate_tool_access(&tool, user_id).await?;

        Ok(self.tool_to_response(&tool))
    }

    async fn list_tools(
        &self,
        tenant_id: TenantId,
        page: u64,
        limit: u64,
    ) -> Result<(Vec<MCPTool>, u64)> {
        let offset = page * limit;

        let query_options = crate::domain::repositories::mcp_tool_repository::MCPToolQueryOptions::new()
            .with_tenant_id(tenant_id)
            .with_pagination(limit, offset);

        let query_result = self.tool_repository
            .find_by_options(query_options)
            .await?;

        Ok((query_result.tools, query_result.total_count))
    }

    async fn call_tool(
        &self,
        tool_id: MCPToolId,
        request: CallMCPToolRequest,
        context: ToolCallContext,
    ) -> Result<CallMCPToolResponse> {
        // 获取工具
        let tool = self.tool_repository
            .find_by_id(tool_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 验证调用权限
        let permission_result = self.domain_service
            .check_call_permission(&tool, &context, &request.parameters)
            .await?;

        if !permission_result.allowed {
            return Err(PlatformError::AuthorizationFailed(
                permission_result.reason.unwrap_or_else(|| "Call not allowed".to_string())
            ));
        }

        // 验证参数
        self.domain_service
            .validate_call_parameters(&tool, &request.parameters)
            .await?;

        // 调用工具
        let result = self.proxy_service
            .call_tool(tool_id, request.parameters, context)
            .await?;

        Ok(CallMCPToolResponse {
            success: result.success,
            result: result.result,
            error: result.error,
            execution_time_ms: result.execution_time_ms,
            metadata: result.metadata,
        })
    }

    async fn test_tool(
        &self,
        tool_id: MCPToolId,
        _request: TestMCPToolRequest,
        user_id: UserId,
    ) -> Result<TestMCPToolResponse> {
        // 获取工具
        let tool = self.tool_repository
            .find_by_id(tool_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 验证访问权限
        self.validate_tool_access(&tool, user_id).await?;

        // 测试连接
        let result = self.proxy_service
            .test_tool_connection(tool_id)
            .await?;

        Ok(TestMCPToolResponse {
            success: result.success,
            message: if result.success {
                "Tool connection test passed".to_string()
            } else {
                result.error.unwrap_or_else(|| "Connection test failed".to_string())
            },
            execution_time_ms: result.execution_time_ms,
            details: result.result,
        })
    }

    async fn activate_tool(
        &self,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<MCPToolResponse> {
        // 获取工具
        let mut tool = self.tool_repository
            .find_by_id(tool_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 验证访问权限
        self.validate_tool_access(&tool, user_id).await?;

        // 激活工具
        tool.activate();

        // 保存工具
        self.tool_repository.save(&tool).await?;

        // 更新代理服务
        self.proxy_service.register_tool(tool.clone()).await?;

        Ok(self.tool_to_response(&tool))
    }

    async fn deactivate_tool(
        &self,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<MCPToolResponse> {
        // 获取工具
        let mut tool = self.tool_repository
            .find_by_id(tool_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 验证访问权限
        self.validate_tool_access(&tool, user_id).await?;

        // 停用工具
        tool.deactivate();

        // 保存工具
        self.tool_repository.save(&tool).await?;

        // 更新代理服务
        self.proxy_service.register_tool(tool.clone()).await?;

        Ok(self.tool_to_response(&tool))
    }

    async fn list_tool_versions(
        &self,
        tool_id: MCPToolId,
        user_id: UserId,
    ) -> Result<Vec<MCPToolVersionResponse>> {
        // 获取工具
        let tool = self.tool_repository
            .find_by_id(tool_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 验证访问权限
        self.validate_tool_access(&tool, user_id).await?;

        // 获取版本列表
        let versions = self.version_repository
            .find_by_tool_id(tool_id)
            .await?;

        let version_responses: Vec<MCPToolVersionResponse> = versions
            .iter()
            .map(|version| MCPToolVersionResponse {
                id: version.id.0,
                tool_id: version.tool_id.0,
                version: version.version,
                config: version.config.clone(),
                change_log: version.change_log.clone(),
                created_by: version.created_by.0,
                created_at: version.created_at,
            })
            .collect();

        Ok(version_responses)
    }

    async fn rollback_tool_version(
        &self,
        tool_id: MCPToolId,
        target_version: i32,
        user_id: UserId,
    ) -> Result<MCPToolResponse> {
        // 获取工具
        let mut tool = self.tool_repository
            .find_by_id(tool_id)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Tool not found".to_string()))?;

        // 验证访问权限
        self.validate_tool_access(&tool, user_id).await?;

        // 获取目标版本
        let target_version_entity = self.version_repository
            .find_by_tool_and_version(tool_id, target_version)
            .await?
            .ok_or_else(|| PlatformError::NotFound("Target version not found".to_string()))?;

        // 回退配置
        tool.update_config(target_version_entity.config);

        // 保存工具
        self.tool_repository.save(&tool).await?;

        // 创建回退版本记录
        let change_log = format!("Rolled back to version {}", target_version);
        self.create_tool_version(&tool, Some(change_log), user_id).await?;

        // 更新代理服务
        self.proxy_service.register_tool(tool.clone()).await?;

        Ok(self.tool_to_response(&tool))
    }

    async fn get_tool_stats(
        &self,
        tenant_id: TenantId,
    ) -> Result<MCPToolStatsResponse> {
        let stats = self.proxy_service
            .get_tool_stats(tenant_id)
            .await?;

        Ok(MCPToolStatsResponse {
            total_tools: stats.total_tools,
            active_tools: stats.active_tools,
            inactive_tools: stats.inactive_tools,
            tools_by_type: stats.tools_by_type,
        })
    }

    async fn validate_tool_config(
        &self,
        config: &ToolConfig,
    ) -> Result<ConfigValidationResult> {
        self.domain_service.validate_tool_config(config).await
    }

    async fn list_tools_for_mcp(
        &self,
        tenant_id: TenantId,
        page: Option<u64>,
        limit: Option<u64>,
    ) -> Result<MCPToolListResponse> {
        self.mcp_server_handler
            .handle_list_tools(tenant_id, page, limit)
            .await
    }

    async fn call_tool_via_mcp(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        tool_name: String,
        arguments: serde_json::Value,
    ) -> Result<MCPToolCallResponse> {
        self.mcp_server_handler
            .handle_call_tool(tenant_id, user_id, tool_name, arguments)
            .await
    }
}

// Tests are in a separate file: mcp_application_service_test.rs