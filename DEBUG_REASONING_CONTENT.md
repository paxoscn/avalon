# 调试 reasoning_content 不显示的问题

## 问题描述

思考过程（reasoning_content）没有显示在聊天界面中。

## 可能的原因

### 1. 模型不支持 reasoning_content

**最常见的原因**：只有特定的 OpenAI 模型支持 `reasoning_content` 字段。

#### 支持的模型
- ✅ `o1-preview`
- ✅ `o1-mini`
- ✅ `o1` (如果可用)

#### 不支持的模型
- ❌ `gpt-4`
- ❌ `gpt-4-turbo`
- ❌ `gpt-3.5-turbo`
- ❌ Claude 系列
- ❌ 其他模型

**解决方案**：确保你的 Agent 使用的是支持推理的模型。

### 2. 模型配置问题

检查你的 Agent 配置：

```sql
-- 查看 Agent 使用的模型
SELECT id, name, llm_config 
FROM agents 
WHERE id = 'your-agent-id';
```

确保 `llm_config` 中的模型是 `o1-preview` 或 `o1-mini`。

### 3. API 请求参数

某些模型需要特定的参数才会返回 reasoning_content。

## 调试步骤

### 步骤 1: 检查浏览器控制台

打开浏览器控制台（F12），发送一条消息，查看日志：

```javascript
// 应该看到这些日志
Received chunk: {type: "content", reasoning_content: "...", ...}
Reasoning update: ...
Content update: ...
```

**如果没有看到 "Reasoning update:"**：
- 说明后端没有返回 `reasoning_content`
- 继续下一步

**如果看到了 "Reasoning update:"**：
- 说明数据已经到达前端
- 检查 UI 渲染逻辑

### 步骤 2: 检查网络请求

在浏览器开发者工具的 Network 标签中：

1. 找到 `/api/agents/{id}/chat/stream` 请求
2. 查看 Response 标签
3. 查找 SSE 数据流

**期望看到**：
```
data: {"type":"content","reasoning_content":"正在分析...","content":null,...}
data: {"type":"content","reasoning_content":"检索知识...","content":null,...}
data: {"type":"content","content":"根据分析...","reasoning_content":null,...}
```

**如果没有 reasoning_content 字段**：
- 说明后端没有返回这个字段
- 检查模型配置

### 步骤 3: 检查后端日志

查看后端日志，看是否有 reasoning_content：

```bash
# 如果使用 cargo run
cargo run | grep reasoning

# 或者查看日志文件
tail -f logs/app.log | grep reasoning
```

### 步骤 4: 测试 OpenAI API

直接测试 OpenAI API 是否返回 reasoning_content：

```bash
curl https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "o1-preview",
    "messages": [
      {"role": "user", "content": "解释一下量子纠缠"}
    ],
    "stream": true
  }'
```

**期望响应**：
```
data: {"choices":[{"delta":{"reasoning_content":"首先..."}}]}
data: {"choices":[{"delta":{"content":"量子纠缠是..."}}]}
```

## 解决方案

### 方案 1: 使用支持推理的模型

更新 Agent 配置，使用 `o1-preview` 或 `o1-mini`：

```sql
UPDATE agents 
SET llm_config = jsonb_set(
  llm_config, 
  '{model}', 
  '"o1-preview"'
)
WHERE id = 'your-agent-id';
```

### 方案 2: 检查 LLM 配置

确保 LLM 配置正确：

```json
{
  "provider": "openai",
  "model": "o1-preview",
  "api_key": "sk-...",
  "temperature": 1.0,
  "max_tokens": 4000
}
```

### 方案 3: 添加调试日志

在后端添加更多日志：

```rust
// src/infrastructure/llm/streaming.rs
let reasoning_content = choice
    .get("delta")
    .and_then(|d| d.get("reasoning_content"))
    .and_then(|c| c.as_str())
    .map(|s| {
        tracing::debug!("Received reasoning_content: {}", s);
        s.to_string()
    });
```

### 方案 4: 模拟数据测试

如果模型不支持，可以先用模拟数据测试 UI：

```typescript
// 在 useAgentChatStream.ts 中添加模拟数据
if (data.type === 'content') {
  // 模拟 reasoning_content
  if (!data.reasoning_content && Math.random() > 0.5) {
    data.reasoning_content = '正在分析问题...';
  }
  
  if (data.content) {
    accumulatedContent += data.content;
    setCurrentResponse(accumulatedContent);
  }
  
  if (data.reasoning_content) {
    accumulatedReasoning += data.reasoning_content;
    setCurrentReasoning(accumulatedReasoning);
  }
}
```

## 验证 UI 是否正常

即使没有真实的 reasoning_content，也可以测试 UI：

### 方法 1: 使用演示页面

```bash
open frontend/src/examples/chat-demo.html
```

这个页面有模拟的 reasoning_content。

### 方法 2: 手动触发

在浏览器控制台中：

```javascript
// 假设你已经在聊天页面
// 手动设置 reasoning 状态
const event = new CustomEvent('test-reasoning', {
  detail: { reasoning: '这是测试的思考过程...' }
});
window.dispatchEvent(event);
```

## 常见问题

### Q: 为什么 GPT-4 不显示思考过程？

A: GPT-4 不支持 `reasoning_content` 字段。只有 o1 系列模型支持。

### Q: 如何知道模型是否支持？

A: 查看 OpenAI 文档或直接测试 API。通常模型名称中包含 "o1" 的支持推理。

### Q: 可以用其他方式显示思考过程吗？

A: 可以。你可以：
1. 在 system prompt 中要求模型在回复中包含思考过程
2. 使用特殊的格式标记（如 `<thinking>...</thinking>`）
3. 解析回复内容，提取思考部分

### Q: UI 已经准备好了，但是没有数据？

A: 是的，UI 已经完全准备好了。只要后端返回 `reasoning_content`，就会自动显示。

## 快速检查清单

- [ ] 使用的是 o1-preview 或 o1-mini 模型
- [ ] Agent 的 llm_config 配置正确
- [ ] 浏览器控制台有 "Received chunk:" 日志
- [ ] 网络请求中有 reasoning_content 字段
- [ ] 前端代码没有错误
- [ ] UI 组件正确渲染

## 推荐的测试流程

1. **先测试 UI**：打开 `chat-demo.html`，确认 UI 正常
2. **检查模型**：确认使用的是支持推理的模型
3. **查看日志**：检查浏览器控制台和后端日志
4. **测试 API**：直接调用 OpenAI API 验证
5. **调试代码**：添加更多日志，逐步排查

## 示例：完整的调试会话

```bash
# 1. 检查 Agent 配置
psql -d your_db -c "SELECT name, llm_config->>'model' as model FROM agents WHERE id = 'xxx';"

# 输出：
# name    | model
# --------|----------
# AI助手  | gpt-4      ← 问题：不支持 reasoning_content

# 2. 更新为支持的模型
psql -d your_db -c "UPDATE agents SET llm_config = jsonb_set(llm_config, '{model}', '\"o1-preview\"') WHERE id = 'xxx';"

# 3. 重启后端
cargo run

# 4. 测试聊天
# 打开浏览器，发送消息

# 5. 检查控制台
# 应该看到：
# Received chunk: {type: "content", reasoning_content: "...", ...}
# Reasoning update: ...
```

## 总结

**最可能的原因**：使用的模型不支持 `reasoning_content`。

**解决方案**：切换到 o1-preview 或 o1-mini 模型。

**验证方法**：查看浏览器控制台日志和网络请求。

如果按照以上步骤仍然无法解决，请提供：
1. 使用的模型名称
2. 浏览器控制台日志
3. 网络请求的 Response 数据
4. 后端日志（如果有）
