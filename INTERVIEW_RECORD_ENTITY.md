# 面试记录实体实现

## 概述

已成功创建面试记录（Interview Record）实体的完整实现，包括领域层、基础设施层和数据库迁移。

## 创建的文件

### 1. 领域层

- **src/domain/entities/interview_record.rs**
  - 定义了 `InterviewRecord` 领域实体
  - 定义了 `InterviewStatus` 枚举（Pending, InProgress, Passed, Failed, Cancelled）
  - 提供了业务方法：`start()`, `complete()`, `cancel()`, `set_questions()`, `set_answers()`
  - 提供了查询方法：`is_completed()`, `is_passed()`, `duration_seconds()`

- **src/domain/repositories/interview_record_repository.rs**
  - 定义了 `InterviewRecordRepository` trait
  - 包含 CRUD 操作和各种查询方法

### 2. 基础设施层

- **src/infrastructure/database/entities/interview_record.rs**
  - SeaORM 数据库实体定义
  - 包含与 Agent、Tenant、User 的关系定义
  - 定义了多个索引以优化查询性能

- **src/infrastructure/repositories/interview_record_repository_impl.rs**
  - `InterviewRecordRepository` trait 的具体实现
  - 提供领域实体与数据库实体之间的转换
  - 实现了所有仓储方法

### 3. 数据库迁移

- **src/infrastructure/database/migrations/m20241127_000003_create_interview_records.rs**
  - 创建 `interview_records` 表
  - 定义外键关系（Agent, Tenant, User）
  - 创建索引以优化查询

## 数据模型

### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| id | UUID | 主键 |
| agent_id | UUID | 关联的 Agent ID |
| tenant_id | UUID | 租户 ID |
| user_id | UUID (可选) | 用户 ID |
| session_id | UUID (可选) | 会话 ID |
| status | String | 面试状态 |
| score | Integer (可选) | 面试分数 |
| feedback | Text (可选) | 反馈信息 |
| questions | JSON (可选) | 面试问题 |
| answers | JSON (可选) | 面试答案 |
| started_at | Timestamp (可选) | 开始时间 |
| completed_at | Timestamp (可选) | 完成时间 |
| created_at | Timestamp | 创建时间 |
| updated_at | Timestamp | 更新时间 |

### 索引

- `idx_interview_records_agent` - Agent ID 索引
- `idx_interview_records_tenant` - Tenant ID 索引
- `idx_interview_records_user` - User ID 索引
- `idx_interview_records_session` - Session ID 索引
- `idx_interview_records_status` - 状态索引
- `idx_interview_records_created_at` - 创建时间索引

## 使用示例

```rust
// 创建新的面试记录
let mut record = InterviewRecord::new(agent_id, tenant_id, Some(user_id));

// 开始面试
record.start(session_id);

// 设置问题和答案
record.set_questions(questions_json);
record.set_answers(answers_json);

// 完成面试
record.complete(InterviewStatus::Passed, Some(85), Some("表现优秀".to_string()));

// 保存到数据库
let saved_record = repository.create(&record).await?;

// 查询面试记录
let records = repository.find_by_agent(&agent_id).await?;
let passed_count = repository.count_passed_by_agent(&agent_id).await?;
```

## 下一步

要使用这个实体，需要：

1. 运行数据库迁移：`cargo run --bin migrate`
2. 在依赖注入容器中注册 `InterviewRecordRepositoryImpl`
3. 在应用服务层创建相关的业务逻辑
4. 在 API 层添加相应的端点

## 相关实体

- Agent - 面试所属的 Agent
- Tenant - 租户信息
- User - 参与面试的用户
- Session - 关联的会话（如果有）
