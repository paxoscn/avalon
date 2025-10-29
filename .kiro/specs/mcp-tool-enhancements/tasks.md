# Implementation Plan

- [x] 1. 扩展领域模型以支持参数位置和响应模板
  - 在`src/domain/value_objects/tool_config.rs`中添加`ParameterPosition`枚举
  - 为`ParameterSchema`添加`position`字段，默认值为`Body`
  - 为`HTTPToolConfig`添加`response_template`字段（可选）
  - 更新`HTTPToolConfig::validate()`方法，添加路径参数一致性验证和header命名验证
  - _Requirements: 1.1, 1.2, 1.3, 2.3, 2.4, 7.2, 7.3, 7.4_

- [x] 2. 实现HTTP工具执行器以处理不同位置的参数
- [x] 2.1 创建参数提取和分组逻辑
  - 在`src/infrastructure/mcp/http_converter.rs`中实现`ParameterGroups`结构
  - 实现`extract_parameters`方法，按position分组参数
  - 实现路径参数的URL编码
  - _Requirements: 1.4, 1.5, 2.5_

- [x] 2.2 实现URL构建逻辑
  - 实现`build_url`方法，替换路径参数占位符
  - 处理特殊字符的URL编码
  - 验证所有路径参数都已提供
  - _Requirements: 2.1, 2.2, 2.5_

- [x] 2.3 更新HTTP请求执行逻辑
  - 修改`HTTPToMCPConverter::execute_tool`方法
  - 将header参数添加到HTTP请求头
  - 将body参数构建为请求体
  - 使用构建的URL发起请求
  - _Requirements: 1.4, 2.2_


- [x] 3. 实现响应模板引擎
- [x] 3.1 添加Handlebars依赖并创建模板引擎结构
  - 在`Cargo.toml`中添加`handlebars = "5.1"`和`percent-encoding = "2.3"`依赖
  - 创建`src/infrastructure/mcp/template_engine.rs`文件
  - 实现`ResponseTemplateEngine`结构，包含Handlebars实例和模板缓存
  - _Requirements: 3.2, 6.1_

- [x] 3.2 实现模板渲染逻辑
  - 实现`render`方法，支持变量替换和循环
  - 实现`compile_template`方法，编译并缓存模板
  - 实现`clear_cache`方法，清除指定工具的模板缓存
  - 处理模板渲染错误，返回原始JSON和错误信息
  - _Requirements: 3.2, 3.3, 3.4, 3.6, 3.7, 6.2, 6.3_

- [x] 3.3 集成模板引擎到HTTP执行器
  - 在`HTTPToMCPConverter`中添加`ResponseTemplateEngine`实例
  - 在执行工具后，检查是否配置了response_template
  - 如果配置了模板，使用模板引擎渲染响应
  - 如果未配置或渲染失败，返回原始JSON
  - _Requirements: 3.5, 3.6, 3.7_

- [x] 4. 实现MCP Server接口
- [x] 4.1 创建MCP协议数据结构
  - 创建`src/infrastructure/mcp/mcp_protocol.rs`文件
  - 定义`MCPToolDescriptor`、`MCPToolListResponse`、`MCPToolCallResponse`等结构
  - 实现JSON Schema转换逻辑
  - _Requirements: 4.2, 4.3_

- [x] 4.2 实现MCP Server Handler
  - 创建`src/infrastructure/mcp/mcp_server_handler.rs`文件
  - 实现`MCPServerHandler`结构
  - 实现`handle_list_tools`方法，返回租户的工具列表
  - 实现`handle_call_tool`方法，执行工具调用
  - 实现`tool_to_mcp_format`方法，转换工具为MCP格式
  - 实现`parameters_to_json_schema`方法，生成JSON Schema
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 5.1, 5.2, 5.3_


- [x] 4.3 创建HTTP路由和处理器
  - 在`src/presentation/handlers/`中创建`mcp_server_handlers.rs`
  - 实现`list_mcp_tools`处理器函数（GET /api/v1/mcp/tools）
  - 实现`call_mcp_tool`处理器函数（POST /api/v1/mcp/tools/call）
  - 添加租户认证和权限验证
  - _Requirements: 5.1, 5.2, 5.6_

- [x] 4.4 注册MCP Server路由
  - 在`src/presentation/routes/`中创建或更新MCP路由配置
  - 将新的MCP Server接口添加到路由表
  - 配置认证中间件
  - _Requirements: 5.1, 5.2_

- [x] 5. 更新应用服务层
- [x] 5.1 更新MCP应用服务
  - 在`MCPApplicationServiceImpl`中集成`MCPServerHandler`
  - 添加`list_tools_for_mcp`方法，返回MCP格式的工具列表
  - 添加`call_tool_via_mcp`方法，处理MCP格式的调用请求
  - _Requirements: 4.1, 4.4, 4.5, 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 5.2 更新工具配置验证
  - 在工具创建和更新时，验证新的配置字段
  - 验证路径参数与endpoint的一致性
  - 验证模板语法（尝试编译）
  - _Requirements: 7.1, 7.2, 7.3, 7.5_

- [x] 5.3 实现模板缓存清除逻辑
  - 在工具配置更新时，清除对应的模板缓存
  - 在工具删除时，清除对应的模板缓存
  - _Requirements: 6.4_


- [x] 6. 更新错误处理
- [x] 6.1 扩展MCPError枚举
  - 在`src/infrastructure/mcp/error_handling.rs`中添加新的错误类型
  - 添加`PathParameterMissing`、`PathParameterInvalid`错误
  - 添加`TemplateRenderError`、`TemplateSyntaxError`错误
  - 添加`ParameterPositionMismatch`错误
  - 实现错误消息的Display trait
  - _Requirements: 3.7, 7.5_

- [x] 6.2 更新错误转换逻辑
  - 将模板引擎错误转换为MCPError
  - 将URL解析错误转换为MCPError
  - 确保错误信息清晰且可操作
  - _Requirements: 3.7, 7.5_

- [x] 7. 编写测试
- [x] 7.1 编写参数位置相关的单元测试
  - 测试ParameterPosition的序列化和反序列化
  - 测试参数提取和分组逻辑
  - 测试路径参数URL编码
  - _Requirements: 1.1, 1.4, 2.5_

- [x] 7.2 编写URL构建的单元测试
  - 测试简单路径参数替换
  - 测试多个路径参数替换
  - 测试特殊字符的URL编码
  - 测试缺失路径参数的错误处理
  - _Requirements: 2.1, 2.2, 2.5_

- [x] 7.3 编写模板引擎的单元测试
  - 测试简单变量替换
  - 测试循环渲染
  - 测试条件渲染
  - 测试模板缓存功能
  - 测试渲染性能（<1ms）
  - 测试模板语法错误处理
  - _Requirements: 3.2, 3.3, 3.4, 3.7, 6.1, 6.2, 6.5_

- [x] 7.4 编写MCP Server的单元测试
  - 测试工具到MCP格式的转换
  - 测试ParameterSchema到JSON Schema的转换
  - 测试不同position的参数转换
  - _Requirements: 4.2, 4.3_


- [x] 7.5 编写配置验证的单元测试
  - 测试路径参数与endpoint一致性验证
  - 测试header参数命名规范验证
  - 测试模板语法验证
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [x] 7.6 编写集成测试
  - 测试创建包含路径参数的工具
  - 测试调用工具并验证参数正确传递到各个位置
  - 测试响应模板正确渲染
  - 测试MCP Server的tools/list接口
  - 测试MCP Server的tools/call接口
  - 测试错误场景（缺失参数、模板错误等）
  - _Requirements: 1.4, 2.2, 3.6, 4.1, 4.4, 4.5, 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 8. 更新文档
  - 更新API文档，添加MCP Server接口说明
  - 添加参数位置配置示例
  - 添加响应模板配置示例和语法说明
  - 更新用户指南，说明如何使用新功能
  - _Requirements: All_
