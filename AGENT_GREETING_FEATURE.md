# Agent 问候语功能

## 概述
为 Agent 实体添加了 `greeting`（问候语）属性，允许为每个 Agent 设置自定义的问候语。

## 修改内容

### 后端修改

1. **Domain 实体** (`src/domain/entities/agent.rs`)
   - 在 `Agent` 结构体中添加 `greeting: Option<String>` 字段
   - 添加 `update_greeting()` 方法用于更新问候语
   - 在 `new()` 和 `copy_from()` 方法中初始化 greeting 字段

2. **DTO** (`src/application/dto/agent_dto.rs`)
   - `CreateAgentDto`: 添加 `greeting: Option<String>`
   - `UpdateAgentDto`: 添加 `greeting: Option<String>`
   - `AgentDto`: 添加 `greeting: Option<String>`
   - `AgentCardDto`: 添加 `greeting: Option<String>`
   - `AgentDetailDto`: 添加 `greeting: Option<String>`

3. **应用服务** (`src/application/services/agent_application_service.rs`)
   - 在 `create_agent()` 中处理 greeting 字段
   - 在 `update_agent()` 中支持更新 greeting
   - 在所有 DTO 转换方法中包含 greeting 字段

4. **数据库实体** (`src/infrastructure/database/entities/agent.rs`)
   - 在 `Model` 结构体中添加 `greeting: Option<String>` 字段

5. **Repository 实现** (`src/infrastructure/repositories/agent_repository_impl.rs`)
   - 在 `entity_to_domain()` 中映射 greeting 字段
   - 在 `domain_to_active_model()` 中包含 greeting 字段

6. **数据库迁移**
   - 创建新迁移文件 `m20241027_000001_add_greeting_to_agents.rs`
   - 添加 `greeting` 列到 `agents` 表（TEXT 类型，可为空）
   - 在 `migrator.rs` 中注册新迁移

### 前端修改

1. **类型定义** (`frontend/src/types/index.ts`)
   - 在 `Agent` 接口中添加 `greeting?: string`

2. **服务** (`frontend/src/services/agent.service.ts`)
   - 在 `CreateAgentRequest` 中添加 `greeting?: string`
   - 在 `UpdateAgentRequest` 中添加 `greeting?: string`

3. **编辑页面** (`frontend/src/pages/AgentDetailPage.tsx`)
   - 在表单数据中添加 `greeting` 字段
   - 添加问候语输入框（textarea）
   - 在创建和更新请求中包含 greeting 字段
   - 将 greeting 传递给预览组件

4. **预览组件** (`frontend/src/components/common/MobileChatPreview.tsx`)
   - 在组件 props 中添加 `greeting?: string`
   - 在空消息状态下优先显示 greeting，其次是 systemPrompt
   - 更新界面以展示问候语

5. **国际化** (`frontend/src/i18n/locales/`)
   - 添加中文翻译：`agents.detail.greeting`、`greetingPlaceholder`、`greetingDescription`
   - 添加英文翻译：`agents.detail.greeting`、`greetingPlaceholder`、`greetingDescription`

## 使用方式

### 在编辑界面设置问候语

1. 进入 Agent 创建或编辑页面
2. 在"基本信息"部分找到"问候语（可选）"字段
3. 输入自定义的欢迎消息，例如："您好！我是您的专属客服助手，有什么可以帮您的吗？"
4. 右侧预览区域会实时显示问候语效果
5. 保存后，用户首次与 Agent 对话时会看到这条问候语

### 创建 Agent 时设置问候语（API）

```typescript
const agent = await agentService.createAgent({
  name: "客服助手",
  greeting: "您好！我是您的专属客服助手，有什么可以帮您的吗？",
  system_prompt: "你是一个友好的客服助手...",
  // ... 其他字段
});
```

### 更新 Agent 的问候语

```typescript
await agentService.updateAgent(agentId, {
  greeting: "欢迎回来！我是您的智能助手，随时为您服务。"
});
```

### 获取 Agent 信息

```typescript
const agent = await agentService.getAgent(agentId);
console.log(agent.greeting); // 输出问候语
```

## 数据库迁移

运行以下命令应用数据库迁移：

```bash
# 开发环境
cargo run --bin migrator up

# 或者通过应用启动时自动迁移
cargo run
```

## 注意事项

- `greeting` 字段是可选的（`Option<String>`），可以为空
- 问候语没有长度限制（使用 TEXT 类型）
- 复制 Agent 时会同时复制问候语
- 更新问候语会自动更新 `updated_at` 时间戳

## API 示例

### 创建 Agent
```json
POST /api/agents
{
  "name": "客服助手",
  "greeting": "您好！我是您的专属客服助手",
  "system_prompt": "你是一个友好的客服助手",
  "preset_questions": [],
  "knowledge_base_ids": [],
  "mcp_tool_ids": [],
  "flow_ids": []
}
```

### 更新 Agent
```json
PUT /api/agents/{id}
{
  "greeting": "欢迎回来！我是您的智能助手"
}
```

### 响应示例
```json
{
  "id": "...",
  "name": "客服助手",
  "greeting": "您好！我是您的专属客服助手",
  "avatar": null,
  "system_prompt": "...",
  "created_at": "2024-10-27T00:00:00Z",
  "updated_at": "2024-10-27T00:00:00Z"
}
```
