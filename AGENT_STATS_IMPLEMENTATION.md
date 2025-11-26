# Agent用量统计功能实现

## 功能概述
在Agent列表页面的"已创建"标签下，为每个Agent卡片添加了用量统计按钮，点击后可以查看该Agent的详细使用统计数据。

## 实现内容

### 1. 类型定义 (frontend/src/types/index.ts)
- `AgentUsageStats`: 单条统计记录
- `AgentUsageStatsParams`: 查询参数（日期范围、分页）
- `AgentUsageStatsResponse`: API响应结构

### 2. API服务 (frontend/src/services/agent.service.ts)
- `getAgentUsageStats()`: 获取Agent用量统计数据

### 3. 统计页面 (frontend/src/pages/AgentStatsPage.tsx)
功能特性：
- 日期范围筛选（默认最近30天）
- 汇总统计卡片：总会话数、总消息数、总Token数、独立用户数
- 每日统计表格：展示每天的详细数据
- 分页支持

### 4. 路由配置 (frontend/src/router.tsx)
- 新增路由：`/agents/:id/stats`

### 5. 列表页面更新 (frontend/src/pages/AgentListPage.tsx)
- 在"已创建"标签的Agent卡片中添加"用量统计"按钮
- 按钮位于删除按钮左侧

### 6. 国际化 (frontend/src/i18n/locales/)
- 中文翻译 (zh.json)
- 英文翻译 (en.json)

## API接口要求

后端需要实现以下接口：

```
GET /agents/{agent_id}/stats
```

查询参数：
- `start_date`: 开始日期 (YYYY-MM-DD)
- `end_date`: 结束日期 (YYYY-MM-DD)
- `page`: 页码
- `page_size`: 每页数量

响应格式：
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

## 使用流程

1. 进入Agent列表页面
2. 切换到"已创建"标签
3. 在任意Agent卡片上点击"用量统计"按钮
4. 进入统计页面，可以：
   - 选择日期范围查询
   - 查看汇总统计
   - 浏览每日详细数据
   - 翻页查看更多历史数据
