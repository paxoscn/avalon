# 路由集成状态

## 概述

本文档记录了 Agent Platform 中路由的当前集成状态。

## 已集成的路由 ✅

### 认证路由 (`src/presentation/routes/auth_routes.rs`)

**状态**: ✅ 已完全集成到 `src/presentation/server.rs`

**可用端点**:
- `GET /api/health` - 健康检查（公开）
- `POST /api/auth/login` - 用户登录（公开）
- `POST /api/auth/refresh` - 刷新令牌（公开）
- `POST /api/auth/logout` - 用户登出（公开）
- `GET /api/auth/me` - 获取当前用户信息（需要认证）
- `POST /api/auth/change-password` - 修改密码（需要认证）

**依赖**:
- ✅ `AuthApplicationService` - 已实现
- ✅ `AuthApplicationServiceImpl` - 已实现
- ✅ `UserRepository` - 已实现
- ✅ `TenantRepository` - 已实现
- ✅ `AuthenticationDomainService` - 已实现

## 待集成的路由 ⚠️

以下路由模块已定义但尚未集成到 `server.rs`。每个模块都需要额外的依赖才能正常工作。

### 1. 流程管理路由 (`flow_routes.rs`)

**状态**: ⚠️ 已定义，待集成

**缺少的依赖**:
- `FlowExecutionRepository` - 需要实现
- `FlowDomainService` - 需要实现
- `ExecutionEngine` - 需要实现

**端点数量**: 15+

### 2. LLM 配置路由 (`config_routes.rs` - llm_config_routes)

**状态**: ⚠️ 已定义，待集成

**缺少的依赖**:
- `LLMProviderRegistry` - 需要实现

**端点数量**: 8

### 3. 向量配置路由 (`config_routes.rs` - vector_config_routes)

**状态**: ⚠️ 已定义，待集成

**缺少的依赖**:
- `VectorDomainService` - 需要实现
- `VectorStoreFactory` - 需要实现

**端点数量**: 9

### 4. 会话管理路由 (`session_audit_routes.rs` - session_routes)

**状态**: ⚠️ 已定义，待集成

**缺少的依赖**:
- `ChatSessionRepository` - 需要实现
- `MessageRepository` - 需要实现
- `SessionDomainService` - 需要实现

**端点数量**: 8

### 5. 审计日志路由 (`session_audit_routes.rs` - audit_routes)

**状态**: ⚠️ 已定义，待集成

**缺少的依赖**:
- `AuditService` (domain service) - 需要实现

**端点数量**: 2

### 6. 执行历史路由 (`session_audit_routes.rs` - execution_history_routes)

**状态**: ⚠️ 已定义，待集成

**缺少的依赖**:
- `ExecutionHistoryService` (domain service) - 需要实现

**端点数量**: 2

### 7. MCP 工具路由 (`mcp_routes.rs`)

**状态**: ⚠️ 已定义，待集成

**缺少的依赖**:
- `MCPToolDomainService` - 需要实现
- `MCPProxyService` - 需要实现

**端点数量**: 10+

### 8. 向量存储路由 (`vector_storage_routes.rs`)

**状态**: ⚠️ 已定义，待集成

**缺少的依赖**:
- `VectorStoreRegistry` - 需要实现
- `VectorApplicationService` (完整版本) - 需要实现

**端点数量**: 8

## 集成优先级建议

基于功能重要性和依赖复杂度，建议按以下顺序集成：

### 高优先级
1. **LLM 配置路由** - 只需要 `LLMProviderRegistry`
2. **审计日志路由** - 只需要 `AuditService`
3. **执行历史路由** - 只需要 `ExecutionHistoryService`

### 中优先级
4. **向量配置路由** - 需要 `VectorDomainService` 和 `VectorStoreFactory`
5. **会话管理路由** - 需要多个 repositories 和 domain service

### 低优先级
6. **流程管理路由** - 需要执行引擎等复杂依赖
7. **MCP 工具路由** - 需要代理服务等
8. **向量存储路由** - 依赖向量配置路由

## 如何集成新路由

### 步骤 1: 实现缺失的依赖

对于每个路由模块，首先实现所有缺失的依赖：

```rust
// 示例：实现 LLMProviderRegistry
pub struct LLMProviderRegistry {
    // 实现细节
}

impl LLMProviderRegistry {
    pub fn new() -> Self {
        // 初始化
    }
}
```

### 步骤 2: 在 server.rs 中添加导入

```rust
use crate::{
    application::services::{
        // ... 现有的 services
        LLMApplicationService, LLMApplicationServiceImpl,
    },
    // ... 其他导入
    presentation::routes::{
        // ... 现有的 routes
        llm_config_routes,
    },
};
```

### 步骤 3: 创建 service 实例

在 `create_app()` 方法中：

```rust
// 创建 repositories
let llm_config_repository = Arc::new(LLMConfigRepositoryImpl::new(
    self.database.connection()
));

// 创建 domain services
let llm_domain_service: Arc<dyn LLMDomainService> = 
    Arc::new(LLMDomainServiceImpl::new());

// 创建 provider registry
let provider_registry = Arc::new(LLMProviderRegistry::new());

// 创建 application service
let llm_service: Arc<dyn LLMApplicationService> =
    Arc::new(LLMApplicationServiceImpl::new(
        llm_config_repository,
        llm_domain_service,
        provider_registry,
    ));
```

### 步骤 4: 添加路由

```rust
let app = Router::new()
    .merge(create_app_router(auth_service.clone()))
    .nest("/api", Router::new()
        // 添加新路由
        .merge(llm_config_routes(llm_service))
    )
    .layer(cors);
```

## 测试策略

每个集成的路由都应该有：

1. **单元测试** - 测试 handler 函数
2. **集成测试** - 测试完整的 HTTP 请求/响应
3. **认证测试** - 验证认证中间件工作正常
4. **多租户测试** - 验证租户隔离

## 当前代码位置

- **路由定义**: `src/presentation/routes/`
- **路由集成**: `src/presentation/server.rs` (第 52-72 行)
- **Application Services**: `src/application/services/`
- **Domain Services**: `src/domain/services/`
- **Repositories**: `src/infrastructure/repositories/`

## 相关文档

- [添加路由指南](adding_routes.md) - 详细的集成步骤
- [路由集成总结](routes_integration_summary.md) - 架构概述
- [TODO 列表](../TODO_ROUTES.md) - 待办事项清单

## 更新日志

- **2024-01-01**: 初始版本，认证路由已集成
- **2024-01-01**: 添加 CORS 配置
- **2024-01-01**: 文档化所有待集成路由的依赖

---

**最后更新**: 2024-01-01  
**维护者**: Agent Platform Team
