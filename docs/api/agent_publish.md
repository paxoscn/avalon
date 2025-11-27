# Agent发布API文档

## 概述

Agent发布功能允许创建者控制其Agent的可见性。只有已发布的Agent才会出现在公共市场中供其他用户查看和雇佣。

## API端点

### 1. 发布Agent

发布一个Agent，使其对其他用户可见。

**端点**
```
POST /api/agents/{agent_id}/publish
```

**权限**
- 需要认证
- 仅Agent创建者可以执行此操作

**路径参数**
| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| agent_id | UUID | 是 | Agent的唯一标识符 |

**请求头**
```
Authorization: Bearer {token}
Content-Type: application/json
```

**响应**

成功 (204 No Content)
```
无响应体
```

错误响应
```json
{
  "error": "Agent not found",
  "code": "AGENT_NOT_FOUND"
}
```

```json
{
  "error": "Only the creator can modify this agent",
  "code": "AGENT_UNAUTHORIZED"
}
```

```json
{
  "error": "Agent is already published",
  "code": "AGENT_VALIDATION_ERROR"
}
```

**示例**

```bash
curl -X POST "http://localhost:8080/api/agents/123e4567-e89b-12d3-a456-426614174000/publish" \
  -H "Authorization: Bearer your_token_here"
```

---

### 2. 取消发布Agent

取消发布一个Agent，使其从公共市场中移除。

**端点**
```
POST /api/agents/{agent_id}/unpublish
```

**权限**
- 需要认证
- 仅Agent创建者可以执行此操作

**路径参数**
| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| agent_id | UUID | 是 | Agent的唯一标识符 |

**请求头**
```
Authorization: Bearer {token}
Content-Type: application/json
```

**响应**

成功 (204 No Content)
```
无响应体
```

错误响应
```json
{
  "error": "Agent not found",
  "code": "AGENT_NOT_FOUND"
}
```

```json
{
  "error": "Only the creator can modify this agent",
  "code": "AGENT_UNAUTHORIZED"
}
```

```json
{
  "error": "Agent is not published",
  "code": "AGENT_VALIDATION_ERROR"
}
```

**示例**

```bash
curl -X POST "http://localhost:8080/api/agents/123e4567-e89b-12d3-a456-426614174000/unpublish" \
  -H "Authorization: Bearer your_token_here"
```

---

### 3. 获取Agent详情

获取Agent的详细信息，包括发布状态。

**端点**
```
GET /api/agents/{agent_id}
```

**响应字段变更**

新增字段：
- `is_published` (boolean): Agent是否已发布
- `published_at` (string|null): 发布时间（ISO 8601格式）

**响应示例**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "tenant_id": "123e4567-e89b-12d3-a456-426614174001",
  "name": "客服助手",
  "avatar": "https://example.com/avatar.png",
  "greeting": "你好！我是客服助手",
  "system_prompt": "你是一个专业的客服助手...",
  "additional_settings": null,
  "preset_questions": ["如何使用？", "价格是多少？"],
  "knowledge_bases": [],
  "mcp_tools": [],
  "flows": [],
  "source_agent": null,
  "creator": {
    "id": "123e4567-e89b-12d3-a456-426614174002",
    "username": "user123",
    "nickname": "张三"
  },
  "employer": null,
  "is_employer": false,
  "is_allocated": false,
  "is_creator": true,
  "is_fired": false,
  "fired_at": null,
  "is_published": true,
  "published_at": "2024-11-27T10:30:00Z",
  "created_at": "2024-11-20T08:00:00Z",
  "updated_at": "2024-11-27T10:30:00Z"
}
```

---

### 4. 列表查询变更

#### 公共Agent列表

**端点**
```
GET /api/agents
```

**行为变更**
- 现在只返回已发布的Agents (`is_published = true`)
- 不返回已被雇佣的Agents (`employer_id IS NULL`)

**响应示例**

```json
{
  "items": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "name": "客服助手",
      "avatar": "https://example.com/avatar.png",
      "greeting": "你好！",
      "system_prompt_preview": "你是一个专业的客服助手...",
      "creator_name": "张三",
      "is_employer": false,
      "is_allocated": false,
      "is_creator": false,
      "is_fired": false,
      "fired_at": null,
      "is_published": true,
      "published_at": "2024-11-27T10:30:00Z",
      "created_at": "2024-11-20T08:00:00Z"
    }
  ],
  "total": 10,
  "page": 1,
  "limit": 20,
  "total_pages": 1
}
```

#### 我创建的Agent列表

**端点**
```
GET /api/agents/created
```

**行为**
- 返回当前用户创建的所有Agents（包括未发布的）
- 不包括通过复制或雇佣创建的Agents

---

## 数据模型

### Agent对象

```typescript
interface Agent {
  id: string;
  tenant_id: string;
  name: string;
  avatar?: string;
  greeting?: string;
  system_prompt: string;
  additional_settings?: string;
  preset_questions: string[];
  source_agent_id?: string;
  creator_id: string;
  employer_id?: string;
  fired_at?: string;
  is_published: boolean;        // 新增
  published_at?: string;         // 新增
  created_at: string;
  updated_at: string;
}
```

### AgentCard对象

```typescript
interface AgentCard {
  id: string;
  name: string;
  avatar?: string;
  greeting?: string;
  system_prompt_preview: string;
  creator_name: string;
  is_employer: boolean;
  is_allocated: boolean;
  is_creator: boolean;
  is_fired: boolean;
  fired_at?: string;
  is_published: boolean;        // 新增
  published_at?: string;         // 新增
  created_at: string;
}
```

---

## 业务规则

### 发布规则

1. **权限控制**
   - 只有Agent的创建者可以发布或取消发布
   - 其他用户无法修改Agent的发布状态

2. **默认状态**
   - 新创建的Agent默认为未发布状态 (`is_published = false`)
   - 需要创建者手动发布

3. **可见性**
   - 未发布的Agent只有创建者可以看到
   - 已发布的Agent会出现在公共市场列表中

4. **雇佣限制**
   - 只能雇佣已发布的Agent
   - 雇佣创建的副本默认为未发布状态

### 状态转换

```
[创建] -> 未发布 (is_published = false, published_at = null)
   |
   v [发布]
已发布 (is_published = true, published_at = timestamp)
   |
   v [取消发布]
未发布 (is_published = false, published_at = null)
```

---

## 错误代码

| 错误代码 | HTTP状态码 | 描述 |
|---------|-----------|------|
| AGENT_NOT_FOUND | 404 | Agent不存在 |
| AGENT_UNAUTHORIZED | 403 | 无权限操作此Agent |
| AGENT_VALIDATION_ERROR | 400 | Agent状态验证失败 |
| AUTHENTICATION_REQUIRED | 401 | 需要认证 |

---

## 使用场景

### 场景1：创建并发布Agent

```bash
# 1. 创建Agent
curl -X POST "http://localhost:8080/api/agents" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "我的助手",
    "system_prompt": "你是一个助手",
    "preset_questions": [],
    "knowledge_base_ids": [],
    "mcp_tool_ids": [],
    "flow_ids": []
  }'

# 响应: { "id": "agent-id", "is_published": false, ... }

# 2. 发布Agent
curl -X POST "http://localhost:8080/api/agents/agent-id/publish" \
  -H "Authorization: Bearer $TOKEN"

# 响应: 204 No Content
```

### 场景2：查看市场上的Agents

```bash
# 获取已发布的Agents
curl -X GET "http://localhost:8080/api/agents?page=1&limit=20" \
  -H "Authorization: Bearer $TOKEN"

# 响应: 只包含已发布的Agents
```

### 场景3：管理自己的Agents

```bash
# 获取我创建的所有Agents（包括未发布的）
curl -X GET "http://localhost:8080/api/agents/created?page=1&limit=20" \
  -H "Authorization: Bearer $TOKEN"

# 响应: 包含所有我创建的Agents
```

---

## 注意事项

1. **向后兼容性**
   - 现有的Agents默认为未发布状态
   - 需要手动发布才能在市场中显示

2. **性能优化**
   - 添加了 `is_published` 索引以提升查询性能
   - 建议在大量数据时使用分页

3. **数据一致性**
   - 发布状态变更会自动更新 `updated_at` 时间戳
   - `published_at` 记录首次发布时间

4. **安全性**
   - 所有操作都需要认证
   - 权限检查在应用层和领域层都有实现
