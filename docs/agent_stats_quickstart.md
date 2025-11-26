# Agent每日统计功能 - 快速开始

## 1. 运行数据库迁移

首先，运行数据库迁移来创建 `agent_daily_stats` 表：

```bash
cargo run --bin agent-platform -- migrate
```

或者使用 sea-orm-cli：

```bash
sea-orm-cli migrate up
```

## 2. 在代码中使用

### 初始化Repository

```rust
use std::sync::Arc;
use sea_orm::DatabaseConnection;
use agent_platform::infrastructure::repositories::AgentDailyStatsRepositoryImpl;

// 假设你已经有了数据库连接
let db: Arc<DatabaseConnection> = /* ... */;
let stats_repo = AgentDailyStatsRepositoryImpl::new(db);
```

### 记录面试统计

```rust
use chrono::Utc;
use agent_platform::domain::value_objects::{AgentId, TenantId};

let agent_id = AgentId::new();
let tenant_id = TenantId::new();
let today = Utc::now().date_naive();

// 获取或创建今天的统计记录
let mut stats = stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;

// 记录面试
stats.increment_interview();

// 如果面试通过
stats.increment_interview_passed();

// 保存
stats_repo.update(&stats).await?;
```

### 记录雇佣

```rust
let mut stats = stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
stats.increment_employment();
stats_repo.update(&stats).await?;
```

### 记录会话统计

```rust
let mut stats = stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;

// 新会话
stats.increment_session();

// 添加消息和Token
stats.add_messages(10);
stats.add_tokens(1500);

// 添加收益（单位：元）
stats.add_revenue(0.05);

stats_repo.update(&stats).await?;
```

### 查询统计数据

```rust
use chrono::Duration;

// 查询今天的统计
let today_stats = stats_repo.find_by_agent_and_date(&agent_id, today).await?;

// 查询最近7天的统计
let end_date = Utc::now().date_naive();
let start_date = end_date - Duration::days(7);
let week_stats = stats_repo.find_by_agent_and_date_range(
    &agent_id, 
    start_date, 
    end_date
).await?;

// 计算总计
let total_revenue: f64 = week_stats.iter().map(|s| s.revenue).sum();
let total_sessions: i64 = week_stats.iter().map(|s| s.session_count).sum();
```

### 查询租户统计

```rust
// 查询租户今天所有Agent的统计
let tenant_stats = stats_repo.find_by_tenant_and_date(&tenant_id, today).await?;

// 计算租户总收益
let total_revenue: f64 = tenant_stats.iter().map(|s| s.revenue).sum();
println!("租户今日总收益: {:.4}", total_revenue);
```

## 3. 集成到现有业务逻辑

### 在面试流程中集成

```rust
// 在 interview_handler.rs 或类似文件中
async fn handle_interview(
    agent_id: AgentId,
    tenant_id: TenantId,
    stats_repo: &dyn AgentDailyStatsRepository,
) -> Result<()> {
    // ... 面试逻辑 ...
    
    // 记录统计
    let today = Utc::now().date_naive();
    let mut stats = stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
    stats.increment_interview();
    
    // 如果面试通过
    if interview_passed {
        stats.increment_interview_passed();
    }
    
    stats_repo.update(&stats).await?;
    
    Ok(())
}
```

### 在雇佣流程中集成

```rust
async fn handle_employment(
    agent_id: AgentId,
    tenant_id: TenantId,
    stats_repo: &dyn AgentDailyStatsRepository,
) -> Result<()> {
    // ... 雇佣逻辑 ...
    
    // 记录统计
    let today = Utc::now().date_naive();
    let mut stats = stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
    stats.increment_employment();
    stats_repo.update(&stats).await?;
    
    Ok(())
}
```

### 在聊天会话中集成

```rust
async fn handle_chat_session(
    agent_id: AgentId,
    tenant_id: TenantId,
    message_count: i64,
    token_count: i64,
    stats_repo: &dyn AgentDailyStatsRepository,
) -> Result<()> {
    // ... 聊天逻辑 ...
    
    // 记录统计
    let today = Utc::now().date_naive();
    let mut stats = stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
    
    stats.increment_session();
    stats.add_messages(message_count);
    stats.add_tokens(token_count);
    
    // 计算收益（示例：每1000个token收费0.01元）
    let revenue = (token_count as f64 / 1000.0) * 0.01;
    stats.add_revenue(revenue);
    
    stats_repo.update(&stats).await?;
    
    Ok(())
}
```

## 4. 添加API端点（可选）

### 查询Agent统计

```rust
use axum::{extract::Path, Json};

async fn get_agent_stats(
    Path((agent_id, date)): Path<(String, String)>,
    Extension(stats_repo): Extension<Arc<dyn AgentDailyStatsRepository>>,
) -> Result<Json<AgentDailyStats>> {
    let agent_id = AgentId::from_uuid(uuid::Uuid::parse_str(&agent_id)?);
    let date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")?;
    
    let stats = stats_repo.find_by_agent_and_date(&agent_id, date)
        .await?
        .ok_or_else(|| PlatformError::NotFound("Stats not found".to_string()))?;
    
    Ok(Json(stats))
}
```

### 查询Agent统计范围

```rust
async fn get_agent_stats_range(
    Path(agent_id): Path<String>,
    Query(params): Query<DateRangeParams>,
    Extension(stats_repo): Extension<Arc<dyn AgentDailyStatsRepository>>,
) -> Result<Json<Vec<AgentDailyStats>>> {
    let agent_id = AgentId::from_uuid(uuid::Uuid::parse_str(&agent_id)?);
    
    let stats = stats_repo.find_by_agent_and_date_range(
        &agent_id,
        params.start_date,
        params.end_date,
    ).await?;
    
    Ok(Json(stats))
}
```

## 5. 常见问题

### Q: 如何处理并发更新？

A: 使用数据库事务和乐观锁：

```rust
// 在事务中更新
let txn = db.begin().await?;
let mut stats = stats_repo.get_or_create(&agent_id, &tenant_id, today).await?;
stats.increment_session();
stats_repo.update(&stats).await?;
txn.commit().await?;
```

### Q: 如何提高性能？

A: 考虑以下优化：
1. 使用Redis缓存当天的统计数据
2. 使用消息队列异步更新统计
3. 批量更新而不是每次操作都更新

### Q: 如何归档历史数据？

A: 定期运行归档任务：

```sql
-- 归档90天前的数据到历史表
INSERT INTO agent_daily_stats_archive 
SELECT * FROM agent_daily_stats 
WHERE stat_date < DATE_SUB(CURDATE(), INTERVAL 90 DAY);

-- 删除已归档的数据
DELETE FROM agent_daily_stats 
WHERE stat_date < DATE_SUB(CURDATE(), INTERVAL 90 DAY);
```

## 6. 下一步

- 查看 [完整实现文档](../AGENT_DAILY_STATS_IMPLEMENTATION.md)
- 查看 [使用示例](../examples/agent_daily_stats_usage.rs)
- 查看 [SQL查询示例](./agent_daily_stats_schema.sql)
- 查看 [总结文档](../AGENT_STATS_SUMMARY.md)
