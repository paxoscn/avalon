# Implementation Plan

- [x] 1. 更新Agent值对象和领域实体
  - 更新 `src/domain/entities/agent.rs` ，包含employer_id和fired_at字段，以及employ()、fire()、is_employed()、is_fired()、is_employer()等业务逻辑方法
  - 废弃 `src/domain/entities/agent_employment.rs` 
  - 在 `src/domain/entities/mod.rs` 中更新导出
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 5.1, 5.4, 5.5, 5.6_

- [x] 2. 定义Repository接口
  - 更新 `src/domain/repositories/agent_repository.rs` 定义 `AgentRepository` trait，包含find_by_employer()等方法
  - _Requirements: 1.1, 5.1, 5.2, 5.3, 6.1, 6.7_

- [x] 3. 更新数据库迁移脚本
  - 创建 `src/infrastructure/database/migrations/m20241027_000003_refactor_agent.rs` ，添加employer_id和fired_at字段到agents表
  - 添加employer_id和fired_at的索引
  - 废弃agent_employments表（如果已存在，创建迁移删除该表）
  - 在 `src/infrastructure/database/migrations/mod.rs` 中注册新迁移
  - _Requirements: 1.1, 5.1, 5.4, 5.5, 5.6, 6.7_

- [x] 4. 更新数据库实体映射
  - 修改 `src/infrastructure/database/entities/agent.rs` 添加employer_id和fired_at字段，以及Employer关系
  - 废弃 `src/infrastructure/database/entities/agent_employment.rs`（如果存在）
  - 在 `src/infrastructure/database/entities/mod.rs` 中更新导出
  - _Requirements: 1.1, 5.1, 5.4, 5.5, 5.6_

- [x] 5. 更新Repository实现
  - 修改 `src/infrastructure/repositories/agent_repository_impl.rs` 实现 `AgentRepository`
  - 更新领域实体与数据库实体的转换方法，包含employer_id和fired_at字段
  - 实现find_by_employer()、find_by_tenant_active()、find_by_tenant_active_paginated()等新方法
  - 移除AgentEmploymentRepository相关实现
  - 在 `src/infrastructure/repositories/mod.rs` 中更新导出
  - _Requirements: 1.1, 5.1, 5.2, 5.3, 5.4, 5.5, 6.1, 6.2, 6.3, 6.4, 6.5, 6.7_

- [x] 6. 更新DTO定义
  - 修改 `src/application/dto/agent_dto.rs` 更新所有Agent相关的DTO
  - 在AgentDto中添加employer_id和fired_at字段
  - 在AgentCardDto中将is_employed改为is_employer，添加is_fired和fired_at字段
  - 在AgentDetailDto中将is_employed改为is_employer，添加employer、is_fired和fired_at字段
  - 在 `src/application/dto/mod.rs` 中确保导出
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 5.1, 5.4, 5.5, 5.6, 6.1, 6.2, 6.3, 6.4, 6.5, 6.7, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8_

- [x] 7. 更新Application Service
  - 修改 `src/application/services/agent_application_service.rs` 更新业务逻辑编排
  - 移除employment_repo依赖
  - 更新employ_agent()方法：复制Agent并设置employer_id，返回新创建的Agent
  - 实现fire_agent()方法：验证用户是employer，设置fired_at时间戳
  - 更新list_agents()和list_employed_agents()方法：添加include_fired参数，默认过滤已解雇的Agent
  - 更新get_agent()方法：返回employer信息和is_employer标志
  - 在解雇操作中添加权限验证（必须是employer）
  - 在 `src/application/services/mod.rs` 中确保导出
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8_

- [x] 8. 更新API Handlers
  - 修改 `src/presentation/handlers/agent_handlers.rs` 更新HTTP处理器
  - 更新employ_agent端点：返回新创建的Agent（Json<AgentDto>）
  - 将terminate_employment端点改名为fire_agent
  - 在list_agents和list_employed_agents端点中添加include_fired查询参数
  - 在 `src/presentation/handlers/mod.rs` 中确保导出
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 6.1, 6.2, 6.3, 6.4, 6.5, 6.7, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8_

- [x] 9. 更新路由配置
  - 在 `src/presentation/routes/` 中更新路由配置
  - 将DELETE /api/v1/agents/{id}/employ 改为 POST /api/v1/agents/{id}/fire
  - 确保所有端点都应用了认证中间件
  - 在 `src/presentation/server.rs` 中确保Agent路由正确集成
  - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 4.1, 5.1, 5.2, 5.5, 6.1, 7.1_

- [x] 10. 更新错误处理
  - 在 `src/error/mod.rs` 中更新Agent特定的错误类型
  - 添加AgentAlreadyFired和AgentNotEmployer错误类型
  - 移除AgentAlreadyEmployed和AgentNotEmployed错误类型（如果存在）
  - 实现错误类型的Display和转换逻辑
  - _Requirements: 3.1, 3.2, 5.5, 5.6_

- [x] 11. 更新server中的服务注册
  - 在 `src/presentation/server.rs` 中更新 `AgentApplicationService` 初始化
  - 移除employment_repo依赖注入
  - 确保其他Repository依赖正确注入
  - _Requirements: 所有需求_

- [x] 12. 运行数据库迁移
  - 执行 `cargo run --bin migrator` 应用新的数据库迁移
  - 验证employer_id和fired_at字段添加成功
  - 验证索引创建成功
  - 验证agent_employments表已删除（如果之前存在）
  - _Requirements: 1.1, 5.1, 5.4, 5.5, 5.6_

- [x] 13. 更新集成测试
  - 修改 `tests/agent_integration_tests.rs` 更新测试用例
  - 测试雇佣操作（验证返回新Agent副本，employer_id正确设置）
  - 测试解雇操作（验证fired_at时间戳设置，权限验证）
  - 测试列表过滤（验证默认不显示已解雇的Agent）
  - 测试权限验证（只有employer可以解雇）
  - 更新现有测试以适应新的雇佣模型
  - _Requirements: 所有需求_
