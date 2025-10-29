# Design Document

## Overview

本设计文档描述了MCP工具增强功能的技术实现方案。主要包括四个核心功能模块：

1. **参数位置标识系统** - 支持body/header/path三种参数位置
2. **路径参数处理系统** - 支持在endpoint URL中使用占位符
3. **响应模板转换系统** - 使用高性能模板引擎将JSON转换为文本
4. **MCP Server实现** - 提供标准MCP协议接口

## Architecture

### 系统架构图

```
┌─────────────────────────────────────────────────────────────┐
│                     MCP Server Layer                         │
│  ┌──────────────────┐         ┌──────────────────┐         │
│  │  tools/list API  │         │  tools/call API  │         │
│  └────────┬─────────┘         └────────┬─────────┘         │
└───────────┼──────────────────────────────┼──────────────────┘
            │                              │
┌───────────┼──────────────────────────────┼──────────────────┐
│           │    Application Service Layer │                  │
│           ▼                              ▼                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │      MCP Application Service                        │   │
│  └─────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────┘
            │
┌───────────┼──────────────────────────────────────────────┐
│           │    Infrastructure Layer                      │
│           ▼                                              │
│  ┌──────────────────┐      ┌──────────────────┐        │
│  │ HTTP Executor    │      │ Template Engine  │        │
│  │ - Path Params    │      │ - Handlebars     │        │
│  │ - Header Params  │      │ - Template Cache │        │
│  │ - Body Params    │      └──────────────────┘        │
│  └──────────────────┘                                   │
└──────────────────────────────────────────────────────────┘
            │
┌───────────┼──────────────────────────────────────────────┐
│           │    Domain Layer                              │
│           ▼                                              │
│  ┌──────────────────────────────────────────────────┐   │
│  │  ParameterSchema (with position field)           │   │
│  │  HTTPToolConfig (with response_template field)   │   │
│  └──────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```


### 技术栈选择

- **模板引擎**: Handlebars-rust - 高性能、编译型模板引擎，支持变量、循环和条件
- **HTTP客户端**: reqwest - 已在项目中使用
- **URL处理**: url crate - 用于URL编码和解析
- **缓存**: 内存缓存（HashMap + RwLock）- 用于缓存编译后的模板

## Components and Interfaces

### 1. 参数位置枚举 (ParameterPosition)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterPosition {
    Body,    // 请求体参数
    Header,  // HTTP头参数
    Path,    // 路径参数
}
```

### 2. 增强的ParameterSchema

```rust
pub struct ParameterSchema {
    pub name: String,
    pub parameter_type: ParameterType,
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub enum_values: Option<Vec<serde_json::Value>>,
    pub position: ParameterPosition,  // 新增字段
}
```

### 3. 增强的HTTPToolConfig

```rust
pub struct HTTPToolConfig {
    pub endpoint: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub parameters: Vec<ParameterSchema>,
    pub timeout_seconds: Option<u64>,
    pub retry_count: Option<u32>,
    pub response_template: Option<String>,  // 新增字段
}
```


### 4. HTTP工具执行器 (HTTPToolExecutor)

负责执行HTTP工具调用，处理不同位置的参数。

```rust
pub struct HTTPToolExecutor {
    client: reqwest::Client,
}

impl HTTPToolExecutor {
    // 执行HTTP工具调用
    pub async fn execute(
        &self,
        config: &HTTPToolConfig,
        parameters: &Value,
    ) -> Result<Value, MCPError>;
    
    // 构建请求URL（处理路径参数）
    fn build_url(
        &self,
        endpoint: &str,
        path_params: &HashMap<String, String>,
    ) -> Result<String, MCPError>;
    
    // 提取不同位置的参数
    fn extract_parameters(
        &self,
        config: &HTTPToolConfig,
        parameters: &Value,
    ) -> Result<ParameterGroups, MCPError>;
}

struct ParameterGroups {
    path_params: HashMap<String, String>,
    header_params: HashMap<String, String>,
    body_params: Value,
}
```

### 5. 响应模板引擎 (ResponseTemplateEngine)

负责将JSON响应转换为文本格式。

```rust
pub struct ResponseTemplateEngine {
    handlebars: Arc<RwLock<Handlebars<'static>>>,
    template_cache: Arc<RwLock<HashMap<String, String>>>,
}

impl ResponseTemplateEngine {
    pub fn new() -> Self;
    
    // 渲染模板
    pub fn render(
        &self,
        template: &str,
        data: &Value,
    ) -> Result<String, TemplateError>;
    
    // 编译并缓存模板
    fn compile_template(
        &self,
        template: &str,
    ) -> Result<String, TemplateError>;
    
    // 清除模板缓存
    pub fn clear_cache(&self);
}
```


### 6. MCP Server Handler

提供标准MCP协议接口。

```rust
pub struct MCPServerHandler {
    tool_repository: Arc<dyn MCPToolRepository>,
    proxy_service: Arc<dyn MCPProxyService>,
}

impl MCPServerHandler {
    // 处理tools/list请求
    pub async fn handle_list_tools(
        &self,
        tenant_id: TenantId,
        page: Option<u64>,
        limit: Option<u64>,
    ) -> Result<MCPToolListResponse, PlatformError>;
    
    // 处理tools/call请求
    pub async fn handle_call_tool(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
        tool_name: String,
        arguments: Value,
    ) -> Result<MCPToolCallResponse, PlatformError>;
    
    // 将工具转换为MCP格式
    fn tool_to_mcp_format(&self, tool: &MCPTool) -> MCPToolDescriptor;
    
    // 将ParameterSchema转换为JSON Schema
    fn parameters_to_json_schema(
        &self,
        parameters: &[ParameterSchema],
    ) -> Value;
}
```

### 7. MCP协议数据结构

```rust
// MCP工具描述符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolDescriptor {
    pub name: String,
    pub description: Option<String>,
    pub inputSchema: Value,  // JSON Schema格式
}

// MCP工具列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolListResponse {
    pub tools: Vec<MCPToolDescriptor>,
}

// MCP工具调用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolCallResponse {
    pub content: Vec<MCPContent>,
    pub isError: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPContent {
    #[serde(rename = "type")]
    pub content_type: String,  // "text" or "resource"
    pub text: Option<String>,
}
```


## Data Models

### 数据库迁移

不需要新的数据库表，只需要更新现有的`mcp_tools`表的`config`字段（JSONB类型）来存储新的配置结构。

### 配置示例

```json
{
  "HTTP": {
    "endpoint": "https://api.example.com/users/{userId}/orders/{orderId}",
    "method": "GET",
    "headers": {
      "Content-Type": "application/json"
    },
    "parameters": [
      {
        "name": "userId",
        "parameter_type": "String",
        "description": "User ID",
        "required": true,
        "position": "path"
      },
      {
        "name": "orderId",
        "parameter_type": "String",
        "description": "Order ID",
        "required": true,
        "position": "path"
      },
      {
        "name": "Authorization",
        "parameter_type": "String",
        "description": "Auth token",
        "required": true,
        "position": "header"
      },
      {
        "name": "includeDetails",
        "parameter_type": "Boolean",
        "description": "Include order details",
        "required": false,
        "position": "body",
        "default_value": false
      }
    ],
    "timeout_seconds": 30,
    "retry_count": 3,
    "response_template": "Order ID: {{ .orderId }}\nStatus: {{ .status }}\nItems:\n{{- range $index, $item := .items }}\n- {{ .name }}: ${{ .price }}\n{{- end }}"
  }
}
```


## Error Handling

### 错误类型扩展

```rust
pub enum MCPError {
    // 现有错误类型...
    
    // 新增错误类型
    PathParameterMissing(String),
    PathParameterInvalid(String),
    TemplateRenderError(String),
    TemplateSyntaxError(String),
    ParameterPositionMismatch(String),
}
```

### 错误处理策略

1. **路径参数错误**: 在调用前验证，返回详细的错误信息
2. **模板渲染错误**: 返回原始JSON响应并附带错误信息
3. **参数位置不匹配**: 在配置保存时验证，拒绝无效配置

## Testing Strategy

### 单元测试

1. **ParameterSchema测试**
   - 测试position字段的序列化/反序列化
   - 测试参数验证逻辑

2. **HTTPToolExecutor测试**
   - 测试路径参数替换
   - 测试header参数设置
   - 测试body参数构建
   - 测试URL编码

3. **ResponseTemplateEngine测试**
   - 测试简单变量替换
   - 测试循环渲染
   - 测试条件渲染
   - 测试模板缓存
   - 测试性能（<1ms）

4. **MCPServerHandler测试**
   - 测试工具列表转换
   - 测试JSON Schema生成
   - 测试工具调用


### 集成测试

1. **端到端工具调用测试**
   - 创建包含路径参数的工具
   - 调用工具并验证参数正确传递
   - 验证响应模板正确渲染

2. **MCP Server接口测试**
   - 测试tools/list接口返回正确格式
   - 测试tools/call接口执行工具
   - 测试错误处理

3. **配置验证测试**
   - 测试路径参数与endpoint一致性验证
   - 测试header参数命名规范验证

## Implementation Details

### 路径参数处理流程

1. 解析endpoint，提取所有`{paramName}`占位符
2. 从parameters中提取position为path的参数
3. 验证占位符与path参数的一致性
4. 在调用时，将参数值进行URL编码后替换到endpoint中

### 响应模板处理流程

1. 检查HTTPToolConfig是否配置了response_template
2. 如果没有，直接返回JSON响应
3. 如果有，检查模板缓存
4. 如果缓存未命中，编译模板并缓存
5. 使用编译后的模板渲染JSON数据
6. 如果渲染失败，返回原始JSON和错误信息

### MCP Server接口实现

#### tools/list接口

- 路径: `GET /api/v1/mcp/tools`
- 查询参数: `tenant_id`, `page`, `limit`
- 响应格式: 符合MCP协议的工具列表

#### tools/call接口

- 路径: `POST /api/v1/mcp/tools/call`
- 请求体:
  ```json
  {
    "name": "tool-name",
    "arguments": { ... }
  }
  ```
- 响应格式: 符合MCP协议的调用结果


### 配置验证增强

在`HTTPToolConfig::validate()`方法中添加以下验证：

1. **路径参数一致性验证**
   - 提取endpoint中的所有占位符
   - 检查每个占位符是否有对应的path参数
   - 检查每个path参数是否在endpoint中有占位符

2. **Header参数命名验证**
   - 验证header参数名称符合HTTP规范
   - 允许的字符: 字母、数字、连字符
   - 常见header名称: Authorization, Content-Type, Accept等

3. **模板语法验证**
   - 在保存配置时尝试编译模板
   - 如果编译失败，返回语法错误

### 性能优化

1. **模板缓存策略**
   - 使用工具ID作为缓存键
   - 工具配置更新时清除对应缓存
   - 使用RwLock实现并发安全的缓存访问

2. **模板预编译**
   - 在工具注册时预编译模板
   - 避免首次调用时的编译开销

3. **参数提取优化**
   - 一次遍历提取所有位置的参数
   - 避免重复遍历参数列表

## Security Considerations

1. **路径参数注入防护**
   - 对路径参数进行URL编码
   - 验证参数值不包含路径遍历字符（../, ./）

2. **Header注入防护**
   - 验证header值不包含换行符
   - 限制header值的长度

3. **模板注入防护**
   - 使用Handlebars的安全模式
   - 禁用危险的helper函数
   - 限制模板复杂度（嵌套深度、循环次数）

4. **权限控制**
   - MCP Server接口需要租户认证
   - 工具调用需要用户权限验证


## Dependencies

需要添加的新依赖：

```toml
[dependencies]
# 模板引擎
handlebars = "5.1"

# URL编码（已有url crate，可能需要升级）
percent-encoding = "2.3"
```

## Migration Strategy

### 向后兼容性

1. **position字段默认值**: 对于现有的参数配置，如果没有position字段，默认为`Body`
2. **response_template字段**: 可选字段，现有工具不受影响
3. **配置迁移**: 不需要数据迁移，JSONB字段自动支持新结构

### 部署步骤

1. 部署新版本代码
2. 现有工具继续正常工作（使用默认position=Body）
3. 用户可以逐步更新工具配置以使用新功能

## Monitoring and Observability

### 指标收集

1. **模板渲染性能**
   - 渲染时间分布
   - 缓存命中率
   - 模板编译次数

2. **参数处理性能**
   - 参数提取时间
   - URL构建时间

3. **MCP Server指标**
   - tools/list调用次数
   - tools/call调用次数
   - 调用成功率

### 日志记录

1. 记录模板渲染错误
2. 记录路径参数替换过程
3. 记录MCP Server接口调用

