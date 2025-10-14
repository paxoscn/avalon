# TODO: 集成所有路由

## 当前状态

✅ **已完成**:
- 认证路由 (auth_routes) 已集成到 server.rs
- CORS 配置已添加
- 所有路由模块已定义

⚠️ **待完成**:
- 其他路由模块尚未集成到主应用程序中

## 需要完成的工作

### 1. 实现缺失的 Application Services

以下 application services 需要实现或验证：

- [ ] `FlowApplicationService` 和 `FlowApplicationServiceImpl`
- [ ] `LLMApplicationService` 和 `LLMApplicationServiceImpl`
- [ ] `VectorApplicationService` 和 `VectorApplicationServiceImpl`
- [ ] `SessionApplicationService` 和 `SessionApplicationServiceImpl`
- [ ] `AuditApplicationService` 和 `AuditApplicationServiceImpl`
- [ ] `ExecutionHistoryApplicationService` 和 `ExecutionHistoryApplicationServiceImpl`
- [ ] `MCPApplicationService` 和 `MCPApplicationServiceImpl`
- [ ] `VectorStorageApplicationService` 和 `VectorStorageApplicationServiceImpl`

### 2. 实现缺失的 Repositories

以下 repositories 需要实现或验证：

- [ ] `FlowRepositoryImpl`
- [ ] `LLMConfigRepositoryImpl`
- [ ] `VectorConfigRepositoryImpl`
- [ ] `SessionRepositoryImpl`
- [ ] `AuditRepositoryImpl`
- [ ] `ExecutionHistoryRepositoryImpl`
- [ ] `MCPRepositoryImpl`
- [ ] `VectorStorageRepositoryImpl`

### 3. 更新 server.rs

在 `src/presentation/server.rs` 的 `create_app()` 方法中：

1. 创建所有需要的 repositories
2. 创建所有需要的 application services
3. 将所有路由集成到主路由器中

参考 `docs/adding_routes.md` 获取详细说明。

### 4. 测试

- [ ] 为每个路由端点添加集成测试
- [ ] 验证认证中间件正常工作
- [ ] 测试 CORS 配置
- [ ] 测试多租户隔离

### 5. 文档

- [ ] 更新 API 文档
- [ ] 添加端点使用示例
- [ ] 更新 README

## 快速开始

要集成所有路由，请按照以下步骤操作：

1. 阅读 `docs/adding_routes.md`
2. 实现缺失的 services 和 repositories
3. 更新 `src/presentation/server.rs` 中的 `create_app()` 方法
4. 运行测试验证所有端点

## 当前可用的路由

✅ **已集成到 server.rs**：

- `GET /api/health` - 健康检查
- `POST /api/auth/login` - 用户登录
- `POST /api/auth/refresh` - 刷新令牌
- `POST /api/auth/logout` - 用户登出
- `GET /api/auth/me` - 获取当前用户信息 (需要认证)
- `POST /api/auth/change-password` - 修改密码 (需要认证)

⚠️ **已定义但未集成**：

所有其他路由模块已定义但需要额外的依赖才能集成。参见下面的"需要完成的工作"部分。

## 路由模块状态

| 模块 | 状态 | 说明 |
|------|------|------|
| auth_routes | ✅ 已集成 | 认证和授权路由 |
| flow_routes | ⚠️ 已定义未集成 | 流程管理路由 |
| config_routes | ⚠️ 已定义未集成 | LLM 和向量配置路由 |
| session_audit_routes | ⚠️ 已定义未集成 | 会话、审计和执行历史路由 |
| mcp_routes | ⚠️ 已定义未集成 | MCP 工具管理路由 |
| vector_config_routes | ⚠️ 已定义未集成 | 向量配置路由 |
| vector_storage_routes | ⚠️ 已定义未集成 | 向量存储操作路由 |

## 注意事项

- 所有路由模块已经定义并导出
- 路由函数签名已经正确
- 需要的只是创建相应的 services 并在 server.rs 中集成
- 当前的警告（unused imports）是正常的，因为路由还没有被使用

## 参考文档

- [添加路由指南](docs/adding_routes.md)
- [API 文档](docs/api_documentation.md)
- [用户指南](docs/user_guide.md)
