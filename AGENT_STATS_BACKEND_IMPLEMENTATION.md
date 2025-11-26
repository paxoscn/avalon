# Agent用量统计后端接口实现

## 概述
实现了 `/agents/{agent_id}/stats` 接口，用于查询Agent的用量统计数据。

## 实现内容

### 1. DTO定义 (src/application/dto/agent_dto.rs)

添加了以下DTO：

- `AgentUsageStatsQuery`: 查询参数
  - `start_date`: 开始日期 (可选，默认30天前)
  - `end_date`: 结束日期 (可选，默认今天)
  - `page`: 页码 (可选，默认1)
  - `page_size`: 每页数量 (可选，默认20)

- `AgentUsageStatsDto`: 单条统计记录
  - `agent_id`: Agent ID
  - `agent_name`: Agent名称
  - `date`: 日期
  - `total_sessions`: 总会话数
  - `total_messages`: 总消息数
  - `total_tokens`: 总Token数
  - `unique_users`: 独立用户数
  - `avg_session_duration_seconds`: 平均会话时长（可选）

- `AgentUsageStatsSummaryDto`: 汇总统计
  - `total_sessions`: 总会话数
  - `total_messages`: 总消息数
  - `total_tokens`: 总Token数
  - `unique_users`: 独立用户数

- `AgentUsageStatsResponse`: 响应结构
  - `items`: 统计记录列表
  - `page`: 当前页码
  - `page_size`: 每页数量
  - `total`: 总记录数
  - `total_pages`: 总页数
  - `summary`: 汇总统计（可选）

### 2. 应用服务 (src/application/services/agent_application_service.rs)

#### 修改AgentApplicationServiceImpl结构体
- 添加 `db: Option<Arc<sea_orm::DatabaseConnection>>` 字段
- 添加 `with_db()` 方法用于设置数据库连接

#### 实现get_agent_usage_stats方法
功能：
1. 验证Agent存在且用户是创建者
2. 解析日期范围（默认最近30天）
3. 从 `agent_daily_stats` 表查询统计数据
4. 支持分页
5. 计算汇总统计
6. 返回格式化的响应

权限控制：
- 只有Agent的创建者可以查看统计数据

### 3. Handler (src/presentation/handlers/agent_handlers.rs)

添加 `get_agent_usage_stats` handler：
- 接收路径参数 `agent_id`
- 接收查询参数 `AgentUsageStatsQuery`
- 调用应用服务获取统计数据
- 返回JSON响应

### 4. 路由 (src/presentation/routes/agent_routes.rs)

添加路由：
```rust
.route("/agents/{agent_id}/stats", get(agent_handlers::get_agent_usage_stats))
```

### 5. 服务器配置 (src/presentation/server.rs)

在创建AgentApplicationService时添加数据库连接：
```rust
.with_db(self.database.connection())
```

## API接口

### 请求
```
GET /agents/{agent_id}/stats?start_date=2025-10-27&end_date=2025-11-26&page=1&page_size=20
```

### 响应
```json
{
  "items": [
    {
      "agent_id": "uuid",
      "agent_name": "Agent名称",
      "date": "2025-11-26",
      "total_sessions": 100,
      "total_messages": 500,
      "total_tokens": 50000,
      "unique_users": 20,
      "avg_session_duration_seconds": 300
    }
  ],
  "page": 1,
  "page_size": 20,
  "total": 30,
  "total_pages": 2,
  "summary": {
    "total_sessions": 3000,
    "total_messages": 15000,
    "total_tokens": 1500000,
    "unique_users": 500
  }
}
```

## 数据来源

统计数据来自 `agent_daily_stats` 表，该表包含以下字段：
- `id`: 主键
- `agent_id`: Agent ID
- `tenant_id`: 租户ID
- `stat_date`: 统计日期
- `interview_count`: 面试次数
- `interview_passed_count`: 面试通过次数
- `employment_count`: 雇佣次数
- `session_count`: 会话数
- `message_count`: 消息数
- `token_count`: Token数
- `revenue`: 收入
- `created_at`: 创建时间
- `updated_at`: 更新时间

## 注意事项

1. **权限控制**: 只有Agent的创建者可以查看统计数据
2. **日期范围**: 默认查询最近30天的数据
3. **分页**: 支持分页查询，默认每页20条
4. **汇总统计**: 返回日期范围内的汇总数据
5. **独立用户数**: 当前使用简化实现，后续可以优化为真实的独立用户统计

## 后续优化

1. 实现真实的独立用户数统计（需要关联chat_sessions表）
2. 实现平均会话时长计算
3. 添加更多统计维度（如按小时统计、按用户统计等）
4. 添加缓存机制提升查询性能
5. 支持导出统计数据
