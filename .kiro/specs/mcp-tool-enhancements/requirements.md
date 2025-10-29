# Requirements Document

## Introduction

本文档定义了MCP工具增强功能的需求，主要包括：参数位置标识（body/header/path）、路径参数支持、响应模板转换以及MCP Server能力实现。这些增强将使MCP工具能够更灵活地与各种HTTP API集成，并提供标准的MCP协议接口供外部系统调用。

## Glossary

- **MCP Tool System**: 管理和执行MCP工具的系统
- **Parameter Schema**: 定义工具参数的结构和约束
- **HTTP Tool Config**: HTTP类型工具的配置信息
- **Response Template**: 用于将JSON响应转换为文本格式的模板
- **MCP Server**: 实现MCP协议的服务端，提供工具列表和调用接口
- **Path Parameter**: 嵌入在URL路径中的参数，如`/users/{userId}`
- **Template Engine**: 处理响应模板的引擎，支持变量替换和循环

## Requirements

### Requirement 1

**User Story:** 作为API集成开发者，我希望能够指定参数的位置（body/header/path），以便正确地将参数传递给不同类型的HTTP API

#### Acceptance Criteria

1. WHEN定义工具参数时，THE MCP Tool System SHALL允许指定参数位置为body、header或path之一
2. THE Parameter Schema SHALL包含position字段，用于标识参数位置
3. WHEN验证参数配置时，THE MCP Tool System SHALL确保position字段的值为有效的枚举值
4. WHEN调用HTTP工具时，THE MCP Tool System SHALL根据参数的position字段将参数放置到正确的HTTP请求位置
5. THE MCP Tool System SHALL为body类型参数构建请求体JSON对象

### Requirement 2

**User Story:** 作为API集成开发者，我希望能够在endpoint URL中使用路径参数占位符，以便支持RESTful API的路径参数

#### Acceptance Criteria

1. WHEN position为path时，THE HTTP Tool Config SHALL允许在endpoint属性中使用`{参数名}`格式的占位符
2. WHEN调用工具时，THE MCP Tool System SHALL将路径参数值替换到endpoint URL中的对应占位符
3. THE MCP Tool System SHALL验证所有endpoint中的路径参数占位符都有对应的参数定义
4. THE MCP Tool System SHALL验证所有position为path的参数在endpoint中都有对应的占位符
5. WHEN路径参数值包含特殊字符时，THE MCP Tool System SHALL进行URL编码

### Requirement 3

**User Story:** 作为API集成开发者，我希望能够定义响应模板，以便将JSON响应转换为易读的文本格式

#### Acceptance Criteria

1. THE HTTP Tool Config SHALL包含response_template字段，用于定义响应转换模板
2. THE Template Engine SHALL支持变量访问语法（如`{{ .data.name }}`）
3. THE Template Engine SHALL支持循环语法（如`{{- range $index, $item := .list }}`）
4. THE Template Engine SHALL支持条件判断语法（如`{{- if .condition }}`）
5. WHEN response_template为空时，THE MCP Tool System SHALL返回原始JSON响应
6. WHEN response_template不为空时，THE MCP Tool System SHALL使用模板引擎处理JSON响应并返回文本结果
7. WHEN模板处理失败时，THE MCP Tool System SHALL返回错误信息并包含原始响应

### Requirement 4

**User Story:** 作为外部系统开发者，我希望能够通过标准MCP协议访问工具列表，以便集成到我的应用中

#### Acceptance Criteria

1. THE MCP Server SHALL提供`tools/list`接口，返回租户可用的工具列表
2. THE MCP Server SHALL为每个工具返回符合MCP协议的工具描述，包括name、description和inputSchema
3. THE MCP Server SHALL将Parameter Schema转换为JSON Schema格式的inputSchema
4. THE MCP Server SHALL支持按租户ID过滤工具列表
5. THE MCP Server SHALL支持分页查询工具列表

### Requirement 5

**User Story:** 作为外部系统开发者，我希望能够通过标准MCP协议调用工具，以便在我的应用中执行工具功能

#### Acceptance Criteria

1. THE MCP Server SHALL提供`tools/call`接口，接受工具名称和参数
2. WHEN接收到工具调用请求时，THE MCP Server SHALL验证工具名称和参数
3. THE MCP Server SHALL执行工具调用并返回结果
4. WHEN工具执行成功时，THE MCP Server SHALL返回包含content的成功响应
5. WHEN工具执行失败时，THE MCP Server SHALL返回包含错误信息的失败响应
6. THE MCP Server SHALL记录工具调用的审计日志

### Requirement 6

**User Story:** 作为系统管理员，我希望模板引擎具有高性能，以便在高并发场景下快速处理响应

#### Acceptance Criteria

1. THE Template Engine SHALL使用编译型模板引擎（如Handlebars或Tera）
2. THE MCP Tool System SHALL缓存已编译的模板，避免重复编译
3. WHEN模板首次使用时，THE MCP Tool System SHALL编译并缓存模板
4. WHEN模板配置更新时，THE MCP Tool System SHALL清除对应的模板缓存
5. THE Template Engine SHALL在1ms内完成简单模板的渲染（不包括网络请求时间）

### Requirement 7

**User Story:** 作为API集成开发者，我希望系统能够验证参数配置的一致性，以便及早发现配置错误

#### Acceptance Criteria

1. WHEN保存工具配置时，THE MCP Tool System SHALL验证endpoint中的路径参数与参数定义的一致性
2. THE MCP Tool System SHALL验证position为path的参数必须在endpoint中有对应占位符
3. THE MCP Tool System SHALL验证endpoint中的占位符必须有对应的path参数定义
4. THE MCP Tool System SHALL验证position为header的参数名称符合HTTP header命名规范
5. WHEN验证失败时，THE MCP Tool System SHALL返回详细的错误信息
