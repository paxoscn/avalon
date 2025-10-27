# Implementation Plan

- [x] 1. 创建Agent值对象和领域实体
  - 在 `src/domain/value_objects/ids.rs` 中添加 `AgentId` 值对象
  - 创建 `src/domain/entities/agent.rs` 实现Agent领域实体，包含所有业务逻辑方法
  - 创建 `src/domain/entities/agent_employment.rs` 实现雇佣关系实体
  - 在 `src/domain/entities/mod.rs` 中导出新实体
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 2. 定义Repository接口
  - 创建 `src/domain/repositories/agent_repository.rs` 定义 `AgentRepository` trait
  - 在同一文件中定义 `AgentEmploymentRepository` trait
  - 在 `src/domain/repositories/mod.rs` 中导出新接口
  - _Requirements: 1.1, 5.1, 5.2, 5.3, 5.4, 5.5, 6.1_

- [x] 3. 创建数据库迁移脚本
  - 创建 `src/infrastructure/database/migrations/m20231201_000014_create_agents.rs` 定义agents表结构
  - 创建 `src/infrastructure/database/migrations/m20231201_000015_create_agent_employments.rs` 定义agent_employments表结构
  - 在 `src/infrastructure/database/migrations/mod.rs` 中注册新迁移
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 5.1, 5.2, 5.3, 5.4_

- [x] 4. 实现数据库实体映射
  - 创建 `src/infrastructure/database/entities/agent.rs` 实现SeaORM实体模型
  - 创建 `src/infrastructure/database/entities/agent_employment.rs` 实现雇佣关系表模型
  - 在 `src/infrastructure/database/entities/mod.rs` 中导出新实体
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 5.1, 5.2_

- [x] 5. 实现Repository
  - 创建 `src/infrastructure/repositories/agent_repository_impl.rs` 实现 `AgentRepository` 和 `AgentEmploymentRepository`
  - 实现领域实体与数据库实体的转换方法
  - 实现所有CRUD操作和查询方法
  - 在 `src/infrastructure/repositories/mod.rs` 中导出实现
  - _Requirements: 1.1, 5.1, 5.2, 5.3, 5.4, 5.5, 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 6. 创建DTO定义
  - 创建 `src/application/dto/agent_dto.rs` 定义所有Agent相关的DTO
  - 实现 `CreateAgentDto`, `UpdateAgentDto`, `AgentDto`, `AgentCardDto`, `AgentDetailDto` 等
  - 在 `src/application/dto/mod.rs` 中导出新DTO
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

- [x] 7. 实现Application Service
  - 创建 `src/application/services/agent_application_service.rs` 实现业务逻辑编排
  - 实现CRUD操作方法（create, get, update, delete, list）
  - 实现复制功能（copy_agent）
  - 实现雇佣管理方法（employ, terminate, list_employed）
  - 实现资源管理方法（add/remove knowledge_base, mcp_tool, flow）
  - 在所有修改操作中添加权限验证逻辑
  - 在 `src/application/services/mod.rs` 中导出新服务
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5, 5.1, 5.2, 5.3, 5.4, 5.5, 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

- [x] 8. 实现API Handlers
  - 创建 `src/presentation/handlers/agent_handlers.rs` 实现所有HTTP处理器
  - 实现基本CRUD端点（create, get, update, delete, list）
  - 实现复制端点（copy_agent）
  - 实现雇佣管理端点（employ, terminate, list_employed）
  - 实现资源管理端点（add/remove knowledge_base, mcp_tool, flow）
  - 在 `src/presentation/handlers/mod.rs` 中导出新处理器
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 5.3, 5.4, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

- [x] 9. 配置路由
  - 在 `src/presentation/routes/` 中创建或更新路由配置
  - 注册所有Agent相关的API端点
  - 确保所有端点都应用了认证中间件
  - 在 `src/presentation/server.rs` 中集成Agent路由
  - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 4.1, 5.1, 5.2, 6.1, 7.1_

- [x] 10. 添加错误处理
  - 在 `src/error/mod.rs` 中添加Agent特定的错误类型
  - 实现错误类型的Display和转换逻辑
  - _Requirements: 3.1, 3.2_

- [x] 11. 在server中注册服务
  - 在 `src/presentation/server.rs` 中初始化 `AgentApplicationService`
  - 将服务注入到应用状态中
  - 确保Repository依赖正确注入
  - _Requirements: 所有需求_

- [x] 12. 运行数据库迁移
  - 执行 `cargo run --bin migrator` 应用新的数据库迁移
  - 验证表结构和索引创建成功
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 5.1, 5.2_

- [x] 13. 编写集成测试
  - 创建 `tests/agent_integration_tests.rs` 测试完整的API流程
  - 测试CRUD操作
  - 测试权限验证（创建者权限）
  - 测试雇佣关系管理
  - 测试复制功能
  - 测试资源关联管理
  - 测试分页功能
  - _Requirements: 所有需求_
