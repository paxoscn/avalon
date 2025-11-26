# Agent统计功能使用指南

## 概述

本指南介绍如何使用Agent统计功能来跟踪面试、雇佣、会话和消息交互等关键指标。

## 功能特性

- ✅ 自动统计面试次数和通过率
- ✅ 自动统计雇佣次数
- ✅ 自动统计会话创建
- ✅ 自动统计消息数量和Token使用
- ✅ 按天聚合统计数据
- ✅ 支持日期范围查询
- ✅ 提供统计摘要和详细数据

## API端点

### 1. 开始面试

记录用户开始与Agent面试的行为。

**请求**
```http
POST /api/agents/{agent_id}/interview/start
Authorization: Bearer {token}
```

**响应**
```http
HTTP/1.1 204 No Content
```

**说明**
- 每次调用会增加当天的 `interview_count`
- 可以在用户点击"开始面试"按钮时调用

### 2. 完成面试

记录面试结果（通过或失败）。

**请求**
```http
POST /api/agents/{agent_id}/interview/complete
Authorization: Bearer {token}
Content-Type: application/json

{
  "passed": true
}
```

**响应**
```http
HTTP/1.1 204 No Content
```

**说明**
- 如果 `passed` 为 `true`，会增加当天的 `interview_passed_count`
- 如果 `passed` 为 `false`，只记录面试完成，不增加通过计数

### 3. 雇佣Agent

雇佣Agent时自动记录统计。

**请求**
```http
POST /api/agents/{agent_id}/employ
Authorization: Bearer {token}
```

**响应**
```json
{
  "id": "uuid",
  "name": "Agent Name",
  "employer_id": "user_uuid",
  ...
}
```

**说明**
- 自动增加当天的 `employment_count`
- 无需额外调用统计API

### 4. 聊天交互

与Agent聊天时自动记录统计。

**请求**
```http
POST /api/agents/{agent_id}/chat
Authorization: Bearer {token}
Content-Type: application/json

{
  "message": "Hello, how are you?",
  "session_id": null
}
```

**响应**
```json
{
  "session_id": "uuid",
  "message_id": "uuid",
  "reply_id": "uuid",
  "reply": "I'm doing well, thank you!",
  "metadata": {
    "model": "gpt-4",
    "tokens_used": 150
  }
}
```

**说明**
- 如果 `session_id` 为 `null`，会创建新会话并增加 `session_count`
- 每次对话增加 `message_count` 2次（用户消息 + 助手回复）
- 自动记录 `token_count`

### 5. 查询统计数据

获取Agent的使用统计数据。

**请求**
```http
GET /api/agents/{agent_id}/stats?start_date=2024-01-01&end_date=2024-12-31&page=1&page_size=20
Authorization: Bearer {token}
```

**响应**
```json
{
  "items": [
    {
      "agent_id": "uuid",
      "agent_name": "My Agent",
      "date": "2024-11-26",
      "total_sessions": 10,
      "total_messages": 50,
      "total_tokens": 5000,
      "unique_users": 5,
      "avg_session_duration_seconds": null
    }
  ],
  "page": 1,
  "page_size": 20,
  "total": 30,
  "total_pages": 2,
  "summary": {
    "total_sessions": 300,
    "total_messages": 1500,
    "total_tokens": 150000,
    "unique_users": 50
  }
}
```

**查询参数**
- `start_date`: 开始日期 (YYYY-MM-DD)，默认30天前
- `end_date`: 结束日期 (YYYY-MM-DD)，默认今天
- `page`: 页码，默认1
- `page_size`: 每页大小，默认20，最大100

## 使用场景

### 场景1: 面试流程

```javascript
// 1. 用户点击"开始面试"
await fetch(`/api/agents/${agentId}/interview/start`, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`
  }
});

// 2. 进行面试对话...
await fetch(`/api/agents/${agentId}/chat`, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    message: "Tell me about yourself",
    session_id: null
  })
});

// 3. 面试结束，提交结果
await fetch(`/api/agents/${agentId}/interview/complete`, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    passed: true
  })
});

// 4. 如果通过，雇佣Agent
await fetch(`/api/agents/${agentId}/employ`, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
```

### 场景2: 查看Agent表现

```javascript
// 获取最近30天的统计数据
const response = await fetch(
  `/api/agents/${agentId}/stats?start_date=2024-11-01&end_date=2024-11-30`,
  {
    headers: {
      'Authorization': `Bearer ${token}`
    }
  }
);

const stats = await response.json();

// 计算转化率
const interviewPassRate = stats.summary.interview_passed_count / 
                          stats.summary.interview_count * 100;
const employmentRate = stats.summary.employment_count / 
                       stats.summary.interview_passed_count * 100;

console.log(`面试通过率: ${interviewPassRate.toFixed(2)}%`);
console.log(`雇佣转化率: ${employmentRate.toFixed(2)}%`);
```

### 场景3: 持续对话

```javascript
let sessionId = null;

// 第一次对话 - 创建新会话
const firstResponse = await fetch(`/api/agents/${agentId}/chat`, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    message: "Hello!",
    session_id: null
  })
});

const firstData = await firstResponse.json();
sessionId = firstData.session_id;

// 后续对话 - 使用现有会话
const secondResponse = await fetch(`/api/agents/${agentId}/chat`, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    message: "How can you help me?",
    session_id: sessionId
  })
});
```

## 统计指标说明

### 基础指标

| 指标 | 说明 | 更新时机 |
|------|------|----------|
| interview_count | 面试次数 | 调用 start_interview |
| interview_passed_count | 通过面试次数 | 调用 complete_interview(passed=true) |
| employment_count | 雇佣次数 | 调用 employ_agent |
| session_count | 会话次数 | 创建新会话时 |
| message_count | 消息数量 | 每次对话 +2 |
| token_count | Token使用量 | 每次对话累加 |

### 计算指标

| 指标 | 计算公式 | 说明 |
|------|----------|------|
| 面试通过率 | interview_passed_count / interview_count * 100% | 衡量Agent质量 |
| 雇佣转化率 | employment_count / interview_passed_count * 100% | 衡量吸引力 |
| 平均会话消息数 | message_count / session_count | 衡量交互深度 |
| 平均消息Token数 | token_count / message_count | 衡量响应复杂度 |

## 权限说明

- **开始面试**: 任何租户内用户
- **完成面试**: 任何租户内用户
- **雇佣Agent**: 任何租户内用户
- **查看统计**: 仅Agent创建者

## 注意事项

1. **数据聚合**: 统计数据按天聚合，同一天的多次操作会累加
2. **异步处理**: 统计操作是异步的，不会影响主业务流程
3. **容错处理**: 统计失败不会导致主业务失败
4. **时区**: 使用UTC时区进行日期计算
5. **权限**: 只有Agent创建者可以查看统计数据

## 最佳实践

1. **面试流程**: 在用户开始面试时调用 `start_interview`，结束时调用 `complete_interview`
2. **会话管理**: 保存 `session_id` 以便后续对话使用同一会话
3. **定期查询**: 定期查询统计数据以监控Agent表现
4. **数据分析**: 结合多个指标综合评估Agent质量

## 故障排查

### 统计数据未更新

1. 检查API调用是否成功（返回204或200）
2. 确认使用正确的 `agent_id`
3. 检查数据库连接是否正常
4. 查看服务器日志是否有错误

### 统计数据不准确

1. 确认查询的日期范围正确
2. 检查时区设置
3. 验证是否有并发操作导致的竞态条件

## 相关文档

- [Agent管理API文档](./agent_api.md)
- [会话管理文档](./session_management.md)
- [统计数据模型](./data_models.md)
