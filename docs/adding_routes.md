# 添加路由到应用程序

## 概述

本文档说明如何将所有路由模块集成到主应用程序中。

## 当前状态

目前 `src/presentation/server.rs` 中只包含了认证路由。其他路由模块已经定义但尚未集成。

## 可用的路由模块

以下路由模块已在 `src/presentation/routes/` 中定义：

1. **auth_routes** - 认证和授权 ✅ (已集成)
2. **flow_routes** - 流程管理
3. **config_routes** - LLM 和向量数据库配置
4. **session_audit_routes** - 会话、审计和执行历史
5. **mcp_routes** - MCP 工具管理
6. **vector_config_routes** - 向量配置
7. **vector_storage_routes** - 向量存储操作

## 如何集成所有路由

### 步骤 1: 在 server.rs 中创建所需的服务

```rust
// 在 create_app() 方法中添加

// 创建所有需要的 repositories
let flow_repository = Arc::new(FlowRepositoryImpl::new(self.database.connection()));
let llm_config_repository = Arc::new(LLMConfigRepositoryImpl::new(self.database.connection()));
let vector_config_repository = Arc::new(VectorConfigRepositoryImpl::new(self.database.connection()));
let session_repository = Arc::new(SessionRepositoryImpl::new(self.database.connection()));
let audit_repository = Arc::new(AuditRepositoryImpl::new(self.database.connection()));
let mcp_repository = Arc::new(MCPRepositoryImpl::new(self.database.connection()));

// 创建 application services
let flow_service: Arc<dyn FlowApplicationService> = Arc::new(
    FlowApplicationServiceImpl::new(flow_repository)
);

let llm_service: Arc<dyn LLMApplicationService> = Arc::new(
    LLMApplicationServiceImpl::new(llm_config_repository)
);

let vector_service: Arc<VectorApplicationService> = Arc::new(
    VectorApplicationServiceImpl::new(vector_config_repository)
);

let session_service: Arc<SessionApplicationService> = Arc::new(
    SessionApplicationServiceImpl::new(session_repository)
);

let audit_service: Arc<AuditApplicationService> = Arc::new(
    AuditApplicationServiceImpl::new(audit_repository)
);

let execution_history_service: Arc<ExecutionHistoryApplicationService> = Arc::new(
    ExecutionHistoryApplicationServiceImpl::new(/* ... */)
);

let mcp_service: Arc<dyn MCPApplicationService> = Arc::new(
    MCPApplicationServiceImpl::new(mcp_repository)
);

let vector_storage_service: Arc<VectorStorageApplicationService> = Arc::new(
    VectorStorageApplicationServiceImpl::new(/* ... */)
);
```

### 步骤 2: 导入路由创建函数

```rust
use crate::presentation::routes::{
    create_app_router,
    flow_routes,
    llm_config_routes,
    vector_config_routes,
    session_routes,
    audit_routes,
    execution_history_routes,
    create_mcp_api_routes,
    create_vector_config_routes,
    create_vector_storage_routes,
};
```

### 步骤 3: 组合所有路由

```rust
fn create_app(&self) -> Router {
    // ... 创建所有服务 (见步骤 1)

    // 配置 CORS
    let cors = self.create_cors_layer();

    // 创建基础路由器
    let app = Router::new()
        // 认证路由 (公开和受保护)
        .merge(create_app_router(auth_service.clone()))
        
        // API 路由 (需要认证)
        .nest("/api", Router::new()
            // 流程管理
            .merge(flow_routes(flow_service))
            
            // 配置管理
            .merge(llm_config_routes(llm_service))
            .merge(vector_config_routes(vector_service.clone()))
            
            // 会话和审计
            .merge(session_routes(session_service))
            .merge(audit_routes(audit_service))
            .merge(execution_history_routes(execution_history_service))
            
            // MCP 工具
            .merge(create_mcp_api_routes(auth_service.clone(), mcp_service))
            
            // 向量操作
            .merge(create_vector_config_routes().with_state(vector_service.clone()))
            .merge(create_vector_storage_routes().with_state(vector_storage_service))
        );

    // 应用 CORS
    app.layer(cors)
}
```

## 完整示例

以下是完整的 `create_app()` 方法示例：

```rust
fn create_app(&self) -> Router {
    // 创建 repositories
    let user_repository = Arc::new(UserRepositoryImpl::new(self.database.connection()));
    let tenant_repository = Arc::new(TenantRepositoryImpl::new(self.database.connection()));
    let flow_repository = Arc::new(FlowRepositoryImpl::new(self.database.connection()));
    // ... 其他 repositories

    // 创建 domain services
    let auth_domain_service = Arc::new(AuthenticationDomainServiceImpl::new(
        self.config.jwt_secret.clone(),
        Some(self.config.bcrypt_cost),
    ));

    // 创建 application services
    let auth_service: Arc<dyn AuthApplicationService> = Arc::new(
        AuthApplicationServiceImpl::new(
            user_repository,
            tenant_repository,
            auth_domain_service,
            None,
        )
    );
    
    // TODO: 创建其他 application services
    // let flow_service = ...
    // let llm_service = ...
    // 等等

    // 配置 CORS
    let cors = self.create_cors_layer();

    // 创建路由器
    let app = Router::new()
        .merge(create_app_router(auth_service.clone()))
        // TODO: 添加其他路由
        // .nest("/api", Router::new()
        //     .merge(flow_routes(flow_service))
        //     .merge(llm_config_routes(llm_service))
        //     // ... 等等
        // )
        .layer(cors);

    app
}
```

## 路由端点概览

### 认证路由 (`/api`)
- `POST /api/auth/login` - 用户登录
- `POST /api/auth/refresh` - 刷新令牌
- `POST /api/auth/logout` - 用户登出
- `GET /api/auth/me` - 获取当前用户信息 (需要认证)
- `POST /api/auth/change-password` - 修改密码 (需要认证)
- `GET /api/health` - 健康检查

### 流程路由 (`/api/flows`)
- `POST /api/flows` - 创建流程
- `GET /api/flows` - 列出流程
- `GET /api/flows/{flow_id}` - 获取流程详情
- `PUT /api/flows/{flow_id}` - 更新流程
- `DELETE /api/flows/{flow_id}` - 删除流程
- `POST /api/flows/{flow_id}/execute` - 执行流程
- `POST /api/flows/{flow_id}/activate` - 激活流程
- `POST /api/flows/{flow_id}/archive` - 归档流程
- `POST /api/flows/import-dsl` - 导入 Dify DSL
- `GET /api/flows/{flow_id}/versions` - 获取版本历史
- `POST /api/flows/{flow_id}/rollback` - 回滚到指定版本

### LLM 配置路由 (`/api/llm-configs`)
- `POST /api/llm-configs` - 创建 LLM 配置
- `GET /api/llm-configs` - 列出 LLM 配置
- `GET /api/llm-configs/{config_id}` - 获取配置详情
- `PUT /api/llm-configs/{config_id}` - 更新配置
- `DELETE /api/llm-configs/{config_id}` - 删除配置
- `POST /api/llm-configs/{config_id}/test` - 测试连接
- `POST /api/llm-configs/{config_id}/set-default` - 设为默认

### 向量配置路由 (`/api/vector-configs`)
- `POST /api/vector-configs` - 创建向量配置
- `GET /api/vector-configs` - 列出向量配置
- `GET /api/vector-configs/{config_id}` - 获取配置详情
- `PUT /api/vector-configs/{config_id}` - 更新配置
- `DELETE /api/vector-configs/{config_id}` - 删除配置
- `POST /api/vector-configs/{config_id}/test` - 测试连接
- `GET /api/vector-configs/health` - 健康状态

### 会话路由 (`/api/sessions`)
- `POST /api/sessions` - 创建会话
- `GET /api/sessions` - 列出会话
- `GET /api/sessions/{session_id}` - 获取会话详情
- `PUT /api/sessions/{session_id}` - 更新会话
- `DELETE /api/sessions/{session_id}` - 删除会话
- `POST /api/sessions/{session_id}/messages` - 添加消息
- `POST /api/sessions/{session_id}/context` - 设置上下文
- `GET /api/sessions/{session_id}/context/{key}` - 获取上下文

### 审计路由 (`/api/audit`)
- `GET /api/audit/logs` - 查询审计日志
- `GET /api/audit/statistics` - 获取审计统计

### 执行历史路由 (`/api/execution-history`)
- `GET /api/execution-history` - 查询执行历史
- `GET /api/execution-history/{execution_id}` - 获取执行详情

### MCP 工具路由 (`/api/mcp`)
- `POST /api/mcp/tools` - 创建 MCP 工具
- `GET /api/mcp/tools` - 列出 MCP 工具
- `GET /api/mcp/tools/{tool_id}` - 获取工具详情
- `PUT /api/mcp/tools/{tool_id}` - 更新工具
- `DELETE /api/mcp/tools/{tool_id}` - 删除工具
- `POST /api/mcp/tools/{tool_id}/call` - 调用工具
- `POST /api/mcp/tools/{tool_id}/test` - 测试工具

## 认证中间件

大多数路由需要认证。认证中间件会：
1. 验证 JWT 令牌
2. 提取用户和租户信息
3. 将信息注入请求上下文

在路由中应用认证中间件：

```rust
Router::new()
    .route("/protected", get(handler))
    .route_layer(middleware::from_fn_with_state(
        auth_service.clone(),
        auth_middleware,
    ))
```

## 下一步

1. 实现所有缺失的 application services
2. 实现所有缺失的 repositories
3. 在 `server.rs` 中集成所有路由
4. 添加集成测试验证所有端点
5. 更新 API 文档

## 参考

- [Axum 路由文档](https://docs.rs/axum/latest/axum/routing/index.html)
- [Axum 中间件文档](https://docs.rs/axum/latest/axum/middleware/index.html)
- [Tower Service 文档](https://docs.rs/tower/latest/tower/trait.Service.html)
