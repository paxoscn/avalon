# 调试：看不到思考过程

## 快速检查清单

### 1. 检查浏览器控制台

打开浏览器控制台（F12），发送一条消息，查看日志：

```javascript
// 应该看到这些日志
Received chunk: {type: "content", reasoning_content: "...", ...}
Reasoning update: ...
Content update: ...
```

**如果没有看到 "Reasoning update:" 日志**：
→ 后端没有返回 `reasoning_content`
→ 查看下面的"后端检查"部分

**如果看到了 "Reasoning update:" 日志**：
→ 数据已经到达前端
→ 查看下面的"前端检查"部分

### 2. 检查网络请求

在浏览器开发者工具的 Network 标签中：

1. 找到 `/api/agents/{id}/chat/stream` 请求
2. 查看 Response 标签
3. 查找 SSE 数据流

**期望看到**：
```
data: {"type":"content","reasoning_content":"正在分析...","content":null}
data: {"type":"content","reasoning_content":"检索知识...","content":null}
data: {"type":"content","content":"根据分析...","reasoning_content":null}
```

**如果没有 reasoning_content 字段**：
→ 后端没有返回这个字段
→ 检查模型配置

### 3. 检查使用的模型

```sql
-- 查看 Agent 使用的模型
SELECT id, name, llm_config->>'model' as model 
FROM agents 
WHERE id = 'your-agent-id';
```

**支持 reasoning_content 的模型**：
- ✅ `o1-preview`
- ✅ `o1-mini`
- ✅ `o1`

**不支持的模型**：
- ❌ `gpt-4`
- ❌ `gpt-4-turbo`
- ❌ `gpt-3.5-turbo`
- ❌ Claude 系列

## 详细调试步骤

### 步骤 1：确认使用的组件

你使用的是哪个组件？

#### AgentChatStream（桌面版）
```tsx
import AgentChatStream from '../components/AgentChatStream';

<AgentChatStream
  agentId="agent-123"
  agentName="AI 助手"
/>
```

#### MobileChatPreview（移动版）
```tsx
import { MobileChatPreview } from '../components/common/MobileChatPreview';

<MobileChatPreview
  agentId="agent-123"
  agentName="AI 助手"
/>
```

### 步骤 2：添加调试日志

#### 在 AgentChatStream 中
```tsx
const {
  currentReasoning,
  // ...
} = useAgentChatStream({
  agentId,
  onChunk: (chunk) => {
    console.log('=== Chunk received ===', chunk);
    if (chunk.reasoning_content) {
      console.log('🧠 Reasoning:', chunk.reasoning_content);
    }
    if (chunk.content) {
      console.log('💬 Content:', chunk.content);
    }
  },
});

// 在渲染部分添加日志
console.log('currentReasoning:', currentReasoning);
console.log('currentResponse:', currentResponse);
console.log('isStreaming:', isStreaming);
```

#### 在 MobileChatPreview 中
```tsx
// 在 chatService.chatStream 中
onReasoning: (chunk) => {
  console.log('🧠 Reasoning chunk:', chunk);
  fullReasoning += chunk;
  setCurrentReasoning(fullReasoning);
  console.log('📊 Current reasoning state:', fullReasoning);
},

// 在渲染部分添加日志
console.log('isTyping:', isTyping);
console.log('currentReasoning:', currentReasoning);
```

### 步骤 3：检查状态

在浏览器控制台中运行：

```javascript
// 检查 React 组件状态（需要 React DevTools）
// 找到 AgentChatStream 或 MobileChatPreview 组件
// 查看 hooks 中的状态：
// - currentReasoning
// - currentResponse
// - isStreaming
```

### 步骤 4：测试演示页面

```bash
open frontend/src/examples/chat-demo.html
```

这个页面有模拟的 reasoning_content，如果这个页面能正常显示，说明 UI 是正常的，问题在于数据源。

## 常见问题

### 问题 1：模型不支持

**症状**：
- 控制台没有 "Reasoning update:" 日志
- Network 请求中没有 `reasoning_content` 字段

**解决方案**：
```sql
UPDATE agents 
SET llm_config = jsonb_set(llm_config, '{model}', '"o1-preview"')
WHERE id = 'your-agent-id';
```

### 问题 2：前端没有处理

**症状**：
- 控制台有 "Received chunk:" 日志
- 但没有 "Reasoning update:" 日志

**检查**：
```typescript
// 确认 useAgentChatStream.ts 中有这段代码
if (data.reasoning_content) {
  accumulatedReasoning += data.reasoning_content;
  setCurrentReasoning(accumulatedReasoning);
  console.log('Reasoning update:', data.reasoning_content);
}
```

### 问题 3：UI 没有渲染

**症状**：
- 控制台有 "Reasoning update:" 日志
- 但界面上看不到

**检查**：
```tsx
// 确认 AgentChatStream.tsx 中有这段代码
{isStreaming && (
  <div>
    {currentReasoning && (
      <div className="bg-gradient-to-r from-amber-50 to-orange-50">
        💭 思考过程
        {currentReasoning}
      </div>
    )}
  </div>
)}
```

### 问题 4：CSS 样式问题

**症状**：
- 元素存在但不可见

**检查**：
```bash
# 在浏览器开发者工具中
# 1. 找到思考过程的 div 元素
# 2. 检查 computed styles
# 3. 确认没有 display: none 或 opacity: 0
```

## 测试用例

### 测试 1：基础功能

```typescript
// 1. 发送消息
await sendMessage("你好");

// 2. 检查控制台
// 应该看到：
// Received chunk: {...}
// Reasoning update: ... (如果模型支持)
// Content update: ...

// 3. 检查界面
// 应该看到思考过程卡片（如果模型支持）
```

### 测试 2：模拟数据

```typescript
// 在 useAgentChatStream.ts 中临时添加
if (data.type === 'content') {
  // 模拟 reasoning_content
  if (!data.reasoning_content && Math.random() > 0.5) {
    data.reasoning_content = '这是模拟的思考过程...';
  }
  
  if (data.reasoning_content) {
    accumulatedReasoning += data.reasoning_content;
    setCurrentReasoning(accumulatedReasoning);
    console.log('Reasoning update:', data.reasoning_content);
  }
}
```

### 测试 3：强制显示

```typescript
// 在组件中临时添加
const [currentReasoning, setCurrentReasoning] = useState('测试思考过程');

// 这样可以验证 UI 是否正常
```

## 完整的调试流程

```
1. 打开浏览器控制台
   ↓
2. 发送消息
   ↓
3. 查看 Network 请求
   - 有 reasoning_content？
     - 是 → 继续步骤 4
     - 否 → 检查模型配置
   ↓
4. 查看控制台日志
   - 有 "Received chunk:"？
     - 是 → 继续步骤 5
     - 否 → 检查网络连接
   ↓
5. 查看控制台日志
   - 有 "Reasoning update:"？
     - 是 → 继续步骤 6
     - 否 → 检查前端代码
   ↓
6. 查看界面
   - 看到思考过程卡片？
     - 是 → 功能正常！
     - 否 → 检查 UI 渲染代码
```

## 快速修复

### 修复 1：确保模型正确

```sql
UPDATE agents 
SET llm_config = jsonb_set(llm_config, '{model}', '"o1-preview"')
WHERE id = 'your-agent-id';
```

### 修复 2：添加调试日志

在 `useAgentChatStream.ts` 中：

```typescript
if (data.type === 'content') {
  console.log('📦 Data:', data);
  
  if (data.content) {
    console.log('💬 Content:', data.content);
    accumulatedContent += data.content;
    setCurrentResponse(accumulatedContent);
  }
  
  if (data.reasoning_content) {
    console.log('🧠 Reasoning:', data.reasoning_content);
    accumulatedReasoning += data.reasoning_content;
    setCurrentReasoning(accumulatedReasoning);
  }
}
```

### 修复 3：强制显示测试

在组件中临时添加：

```tsx
// 测试 UI 是否正常
const [testReasoning] = useState('这是测试的思考过程');

{isStreaming && (
  <div>
    {/* 测试显示 */}
    <div className="bg-gradient-to-r from-amber-50 to-orange-50 border border-amber-200 rounded-lg p-4">
      <div className="flex items-center space-x-2 mb-2">
        <span className="text-sm font-semibold text-amber-700">💭 测试思考过程</span>
      </div>
      <div className="text-sm text-amber-900">
        {testReasoning}
      </div>
    </div>
    
    {/* 实际的思考过程 */}
    {currentReasoning && (
      <div className="bg-gradient-to-r from-amber-50 to-orange-50">
        💭 实际思考过程
        {currentReasoning}
      </div>
    )}
  </div>
)}
```

## 联系支持

如果以上步骤都无法解决问题，请提供：

1. **浏览器控制台的完整日志**
2. **Network 请求的 Response 数据**
3. **使用的模型名称**
4. **使用的组件名称**（AgentChatStream 或 MobileChatPreview）
5. **React DevTools 中的组件状态截图**

## 相关文档

- `DEBUG_REASONING_CONTENT.md` - 详细的调试指南
- `REASONING_CONTENT_SUMMARY.md` - 功能总结
- `check_reasoning_support.sh` - 自动检查脚本
