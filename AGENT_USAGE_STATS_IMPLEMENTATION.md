# Agent用量统计实现总结

## 概述

在面试、通过面试、雇佣、开启会话、消息交互等关键节点实现了Agent用量统计功能。

## 实现的功能

### 1. 统计服务 (AgentStatsService)

创建了新的领域服务 `src/domain/services/agent_stats_service.rs`，提供以下统计方法：

- `record_interview()` - 记录面试次数
- `record_interview_passed()` - 记录通过面试次数
- `record_employment()` - 记录雇佣次数
- `record_session()` - 记录会话次数
- `record_messages()` - 记录消息数量
- `record_tokens()` - 记录Token使用量
- `record_revenue()` - 记录收入

### 2. 统计集成点

#### 2.1 面试统计
- **开始面试**: `POST /agents/{agent_id}/interview/start`
  - 调用 `start_interview()` 方法
  - 记录 `interview_count` +1

- **完成面试**: `POST /agents/{agent_id}/interview/complete`
  - 调用 `complete_interview()` 方法
  - 如果通过，记录 `interview_passed_count` +1

#### 2.2 雇佣统计
- **雇佣Agent**: `POST /agents/{agent_id}/employ`
  - 在 `employ_agent()` 方法中
  - 记录 `employment_count` +1

#### 2.3 会话统计
- **开启新会话**: 在 `chat()` 方法中
  - 当创建新会话时
  - 记录 `session_count` +1

#### 2.4 消息和Token统计
- **消息交互**: 在 `chat()` 方法中
  - 每次对话记录2条消息（用户+助手）
  - 记录 `message_count` +2
  - 记录 `token_count` += 实际使用的token数

## 数据模型

### AgentDailyStats 实体

```rust
pub struct AgentDailyStats {
    pub id: Uuid,
    pub agent_id: AgentId,
    pub tenant_id: TenantId,
    pub stat_date: NaiveDate,
    pub interview_count: i64,           // 面试次数
    pub interview_passed_count: i64,    // 通过面试次数
    pub employment_count: i64,          // 雇佣次数
    pub session_count: i64,             // 会话次数
    pub message_count: i64,             // 消息数量
    pub token_count: i64,               // Token使用量
    pub revenue: f64,                   // 收入
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## API端点

### 面试相关
- `POST /agents/{agent_id}/interview/start` - 开始面试
- `POST /agents/{agent_id}/interview/complete` - 完成面试
  - Body: `{ "passed": true/false }`

### 统计查询
- `GET /agents/{agent_id}/stats` - 获取Agent使用统计
  - Query参数:
    - `start_date`: 开始日期 (YYYY-MM-DD)
    - `end_date`: 结束日期 (YYYY-MM-DD)
    - `page`: 页码
    - `page_size`: 每页大小

## 配置

在应用启动时需要配置统计服务：

```rust
let agent_service = AgentApplicationServiceImpl::new(...)
    .with_stats_service(Arc::new(AgentStatsService::new(stats_repo)))
    .with_session_service(session_service)
    .with_llm_service(llm_service)
    .with_llm_config_repo(llm_config_repo)
    .with_db(db);
```

## 统计指标

系统会自动统计以下指标：

1. **面试转化率**: `interview_passed_count / interview_count * 100%`
2. **雇佣转化率**: `employment_count / interview_passed_count * 100%`
3. **平均会话消息数**: `message_count / session_count`
4. **平均消息Token数**: `token_count / message_count`
5. **平均会话收入**: `revenue / session_count`

## 文件变更

### 新增文件
- `src/domain/services/agent_stats_service.rs` - 统计服务实现

### 修改文件
- `src/domain/services/mod.rs` - 导出统计服务
- `src/application/services/agent_application_service.rs` - 集成统计功能
- `src/application/dto/agent_dto.rs` - 添加面试相关DTO
- `src/presentation/handlers/agent_handlers.rs` - 添加面试处理函数
- `src/presentation/routes/agent_routes.rs` - 添加面试路由

## 使用示例

### 1. 面试流程

```bash
# 开始面试
curl -X POST http://api/agents/{agent_id}/interview/start \
  -H "Authorization: Bearer {token}"

# 完成面试（通过）
curl -X POST http://api/agents/{agent_id}/interview/complete \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{"passed": true}'
```

### 2. 雇佣Agent

```bash
curl -X POST http://api/agents/{agent_id}/employ \
  -H "Authorization: Bearer {token}"
```

### 3. 聊天（自动统计会话和消息）

```bash
curl -X POST http://api/agents/{agent_id}/chat \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello",
    "session_id": null
  }'
```

### 4. 查询统计

```bash
curl -X GET "http://api/agents/{agent_id}/stats?start_date=2024-01-01&end_date=2024-12-31" \
  -H "Authorization: Bearer {token}"
```

## 注意事项

1. 统计数据按天聚合，使用 `stat_date` 字段
2. 所有统计操作都是异步的，不会阻塞主流程
3. 统计失败不会影响主业务逻辑（使用 `let _ =` 忽略错误）
4. 需要在应用启动时正确配置 `stats_service`
5. 统计数据存储在 `agent_daily_stats` 表中

## 后续优化建议

1. 添加唯一用户统计（需要记录user_id）
2. 添加会话时长统计
3. 实现收入计算逻辑
4. 添加实时统计缓存
5. 实现统计数据导出功能
