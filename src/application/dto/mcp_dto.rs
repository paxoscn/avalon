use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{
    entities::MCPToolStatus,
    value_objects::{
        ids::TenantId,
        tool_config::ToolConfig,
    },
};

/// 创建MCP工具请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMCPToolRequest {
    pub name: String,
    pub description: Option<String>,
    pub config: ToolConfig,
}

/// 更新MCP工具请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMCPToolRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<ToolConfig>,
    pub change_log: Option<String>,
}

/// MCP工具响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub current_version: i32,
    pub status: MCPToolStatus,
    pub config: ToolConfig,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// MCP工具列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolListResponse {
    pub tools: Vec<MCPToolResponse>,
    pub total: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

/// 调用MCP工具请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallMCPToolRequest {
    pub parameters: Value,
    pub session_id: Option<String>,
    pub metadata: Option<HashMap<String, Value>>,
}

/// 调用MCP工具响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallMCPToolResponse {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, Value>,
}

/// 测试MCP工具请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMCPToolRequest {
    pub test_parameters: Option<Value>,
}

/// 测试MCP工具响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMCPToolResponse {
    pub success: bool,
    pub message: String,
    pub execution_time_ms: u64,
    pub details: Option<Value>,
}

/// MCP工具版本响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolVersionResponse {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub version: i32,
    pub config: ToolConfig,
    pub change_log: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// MCP工具统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolStatsResponse {
    pub total_tools: usize,
    pub active_tools: usize,
    pub inactive_tools: usize,
    pub tools_by_type: HashMap<String, usize>,
}

/// 工具列表查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolListQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<MCPToolStatus>,
    pub search: Option<String>,
}

/// 回退版本请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackVersionRequest {
    pub target_version: i32,
    pub change_log: Option<String>,
}

/// 批量操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchToolOperationRequest {
    pub tool_ids: Vec<Uuid>,
    pub operation: BatchToolOperation,
}

/// 批量操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchToolOperation {
    Activate,
    Deactivate,
    Delete,
}

/// 批量操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchToolOperationResponse {
    pub success_count: usize,
    pub failure_count: usize,
    pub results: Vec<BatchToolOperationResult>,
}

/// 批量操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchToolOperationResult {
    pub tool_id: Uuid,
    pub success: bool,
    pub error: Option<String>,
}

/// 工具配置验证请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateToolConfigRequest {
    pub config: ToolConfig,
}

/// 工具配置验证响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateToolConfigResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// 工具搜索请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchToolsRequest {
    pub query: String,
    pub filters: Option<ToolSearchFilters>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// 工具搜索过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchFilters {
    pub status: Option<MCPToolStatus>,
    pub tool_type: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

/// 工具搜索响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchToolsResponse {
    pub tools: Vec<MCPToolResponse>,
    pub total: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
    pub search_time_ms: u64,
}

/// 工具使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStats {
    pub tool_id: Uuid,
    pub tool_name: String,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub average_execution_time_ms: f64,
    pub last_called_at: Option<DateTime<Utc>>,
}

/// 工具使用统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStatsResponse {
    pub stats: Vec<ToolUsageStats>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_calls: u64,
    pub total_tools: usize,
}

/// 导出工具配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportToolsRequest {
    pub tool_ids: Option<Vec<Uuid>>,
    pub include_versions: bool,
    pub format: ExportFormat,
}

/// 导出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Yaml,
}

/// 导出工具配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportToolsResponse {
    pub data: String,
    pub format: ExportFormat,
    pub tool_count: usize,
    pub exported_at: DateTime<Utc>,
}

/// 导入工具配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportToolsRequest {
    pub data: String,
    pub format: ExportFormat,
    pub overwrite_existing: bool,
}

/// 导入工具配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportToolsResponse {
    pub imported_count: usize,
    pub skipped_count: usize,
    pub error_count: usize,
    pub results: Vec<ImportToolResult>,
}

/// 导入工具结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportToolResult {
    pub tool_name: String,
    pub success: bool,
    pub tool_id: Option<Uuid>,
    pub error: Option<String>,
}

impl Default for MCPToolListQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
            status: None,
            search: None,
        }
    }
}

impl Default for TestMCPToolRequest {
    fn default() -> Self {
        Self {
            test_parameters: None,
        }
    }
}

impl Default for CallMCPToolRequest {
    fn default() -> Self {
        Self {
            parameters: serde_json::json!({}),
            session_id: None,
            metadata: None,
        }
    }
}