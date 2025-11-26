# Agent每日用量统计功能实现

## 概述

为已雇佣的Agent添加了按天统计用量的功能，用于跟踪和分析Agent的使用情况和收益。

## 数据库表结构

### agent_daily_stats 表

| 字段名 | 类型 | 说明 |
|--------|------|------|
| id | UUID | 主键 |
| agent_id | UUID | Agent ID (外键) |
| tenant_id | UUID | 租户ID (外键) |
| stat_date | DATE | 统计日期 |
| interview_count | BIGINT | 被面试次数 |
| interview_passed_count | BIGINT | 面试通过次数 |
| employment_count | BIGINT | 被雇佣次数 |
| session_count | BIGINT | 总会话数 |
| message_count | BIGINT | 总消息数 |
| token_count | BIGINT | 总Token消耗数 |
| revenue | DECIMAL(20,6) | 总收益 |
| created_at | TIMESTAMP | 创建时间 |
| updated_at | TIMESTAMP | 更新时间 |

### 索引

- `idx_agent_daily_stats_agent_date`: (agent_id, stat_date) - 唯一索引
- `idx_agent_daily_stats_tenant_date`: (tenant_id, stat_date)
- `idx_agent_daily_stats_stat_date`: (stat_date)

## 实现的文件

### 1. 数据库迁移
- `src/infrastructure/database/migrations/m20241126_000001_create_agent_daily_stats.rs`

### 2. 领域实体
- `src/domain/entities/agent_daily_stats.rs`

### 3. 数据库实体
- `src/infrastructure/database/entities/agent_daily_stats.rs`

### 4. Repository接口
- `src/domain/repositories/agent_daily_stats_repository.rs`

### 5. Repository实现
- `src/infrastructure/repositories/agent_daily_stats_repository_impl.rs`

## 核心功能

### AgentDailyStats 实体方法

```rust
// 创建新的统计记录
AgentDailyStats::new(agent_id, tenant_id, stat_date)

// 增加计数
stats.increment_interview()           // 增加面试次数
stats.increment_interview_passed()    // 增加面试通过次数
stats.increment_employment()          // 增加雇佣次数
stats.increment_session()             // 增加会话数

// 添加数值
stats.add_messages(count)             // 添加消息数
stats.add_tokens(count)               // 添加Token数
stats.add_revenue(amount)             // 添加收益

// 计算统计指标
stats.get_interview_pass_rate()       // 获取面试通过率
stats.get_employment_rate()           // 获取雇佣率
stats.get_average_messages_per_session()  // 获取平均每会话消息数
stats.get_average_tokens_per_message()    // 获取平均每消息Token数
stats.get_average_revenue_per_session()   // 获取平均每会话收益
```

### Repository 方法

```rust
// 创建统计记录
repository.create(&stats)

// 更新统计记录
repository.update(&stats)

// 查询单个Agent的某天统计
repository.find_by_agent_and_date(&agent_id, stat_date)

// 查询单个Agent的日期范围统计
repository.find_by_agent_and_date_range(&agent_id, start_date, end_date)

// 查询租户的某天所有Agent统计
repository.find_by_tenant_and_date(&tenant_id, stat_date)

// 查询租户的日期范围所有Agent统计
repository.find_by_tenant_and_date_range(&tenant_id, start_date, end_date)

// 获取或创建统计记录（如果不存在则创建）
repository.get_or_create(&agent_id, &tenant_id, stat_date)
```

## 使用示例

### 1. 记录面试统计

```rust
use chrono::Utc;

let today = Utc::now().date_naive();
let mut stats = repository.get_or_create(&agent_id, &tenant_id, today).await?;

// 记录面试
stats.increment_interview();

// 如果面试通过
stats.increment_interview_passed();

repository.update(&stats).await?;
```

### 2. 记录雇佣统计

```rust
let today = Utc::now().date_naive();
let mut stats = repository.get_or_create(&agent_id, &tenant_id, today).await?;

stats.increment_employment();
repository.update(&stats).await?;
```

### 3. 记录会话和消息统计

```rust
let today = Utc::now().date_naive();
let mut stats = repository.get_or_create(&agent_id, &tenant_id, today).await?;

// 新会话
stats.increment_session();

// 添加消息和Token
stats.add_messages(10);
stats.add_tokens(1500);

// 添加收益
stats.add_revenue(0.05);

repository.update(&stats).await?;
```

### 4. 查询统计数据

```rust
use chrono::{Utc, Duration};

// 查询今天的统计
let today = Utc::now().date_naive();
let today_stats = repository.find_by_agent_and_date(&agent_id, today).await?;

// 查询最近7天的统计
let end_date = Utc::now().date_naive();
let start_date = end_date - Duration::days(7);
let week_stats = repository.find_by_agent_and_date_range(
    &agent_id, 
    start_date, 
    end_date
).await?;

// 计算总计
let total_sessions: i64 = week_stats.iter().map(|s| s.session_count).sum();
let total_revenue: f64 = week_stats.iter().map(|s| s.revenue).sum();
```

### 5. 查询租户所有Agent统计

```rust
// 查询租户今天所有Agent的统计
let today = Utc::now().date_naive();
let tenant_stats = repository.find_by_tenant_and_date(&tenant_id, today).await?;

// 计算租户总收益
let total_revenue: f64 = tenant_stats.iter().map(|s| s.revenue).sum();
```

## 运行迁移

```bash
# 运行数据库迁移
cargo run --bin agent-platform -- migrate

# 或者使用sea-orm-cli
sea-orm-cli migrate up
```

## 后续工作建议

1. **创建Application Service**: 封装统计逻辑，提供更高级的API
2. **添加API端点**: 提供HTTP接口查询统计数据
3. **实现定时任务**: 自动聚合和清理历史数据
4. **添加缓存**: 使用Redis缓存热点统计数据
5. **实现报表功能**: 生成日报、周报、月报
6. **添加告警功能**: 当某些指标异常时发送通知
7. **数据可视化**: 在前端展示统计图表

## 注意事项

1. 统计数据应该在事务中更新，确保数据一致性
2. 对于高并发场景，考虑使用消息队列异步更新统计
3. 定期归档历史数据，避免表过大影响性能
4. 收益字段使用DECIMAL类型，避免浮点数精度问题
5. 唯一索引(agent_id, stat_date)确保每个Agent每天只有一条记录
