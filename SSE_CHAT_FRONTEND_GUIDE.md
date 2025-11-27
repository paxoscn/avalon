# SSE聊天前端集成指南

## 概述

前端聊天界面已成功改为使用SSE（Server-Sent Events）协议进行实时流式通信。用户现在可以看到AI助手的回复逐字显示，提供更好的用户体验。

## 修改内容

### 1. 聊天服务 (`frontend/src/services/chat.service.ts`)

#### 新增接口定义

```typescript
// SSE流式响应块
export interface ChatStreamChunk {
  type: 'content' | 'done' | 'error';
  content?: string;
  session_id?: string;
  message_id?: string;
  reply_id?: string;
  metadata?: Record<string, any>;
  finish_reason?: string;
  error?: string;
}

// SSE流式回调
export interface ChatStreamCallbacks {
  onContent?: (content: string) => void;
  onDone?: (data: { sessionId: string; messageId: string; replyId: string; metadata?: Record<string, any> }) => void;
  onError?: (error: string) => void;
}
```

#### 新增流式方法

```typescript
async chatStream(
  request: ChatRequest,
  callbacks: ChatStreamCallbacks
): Promise<void>
```

**功能特性：**
- 使用原生 `fetch` API 发送请求
- 支持 SSE 流式响应
- 实时解析并回调内容块
- 自动处理连接错误
- 支持会话管理

**使用示例：**

```typescript
await chatService.chatStream(
  {
    agentId: 'agent-uuid',
    message: '你好',
    sessionId: 'optional-session-uuid',
  },
  {
    onContent: (chunk) => {
      console.log('收到内容:', chunk);
      // 实时更新UI
    },
    onDone: (data) => {
      console.log('完成:', data);
      // 保存会话ID等信息
    },
    onError: (error) => {
      console.error('错误:', error);
      // 显示错误信息
    },
  }
);
```

### 2. 聊天预览组件 (`frontend/src/components/common/MobileChatPreview.tsx`)

#### 修改内容

组件已更新为使用 `chatService.chatStream()` 方法：

**主要变化：**

1. **创建临时消息**：在开始流式传输前创建一个空的助手消息
2. **实时更新内容**：通过 `onContent` 回调逐步更新消息内容
3. **完成时更新ID**：通过 `onDone` 回调更新消息的最终ID和会话信息
4. **错误处理**：通过 `onError` 回调显示错误信息

**代码片段：**

```typescript
// 创建临时消息用于流式显示
const tempMessageId = `temp-${Date.now()}`;
const assistantMessage: ChatMessage = {
  id: tempMessageId,
  role: 'assistant',
  content: '',
  timestamp: new Date(),
};
setMessages((prev) => [...prev, assistantMessage]);

let fullContent = '';

await chatService.chatStream(
  { agentId, message: content.trim(), sessionId },
  {
    onContent: (chunk) => {
      fullContent += chunk;
      // 实时更新消息内容
      setMessages((prev) =>
        prev.map((msg) =>
          msg.id === tempMessageId
            ? { ...msg, content: fullContent }
            : msg
        )
      );
    },
    onDone: (data) => {
      if (!sessionId) {
        setSessionId(data.sessionId);
      }
      // 更新为最终ID
      setMessages((prev) =>
        prev.map((msg) =>
          msg.id === tempMessageId
            ? { ...msg, id: data.replyId }
            : msg
        )
      );
    },
    onError: (errorMsg) => {
      setError(errorMsg);
      setMessages((prev) =>
        prev.map((msg) =>
          msg.id === tempMessageId
            ? { ...msg, content: `抱歉，${errorMsg}` }
            : msg
        )
      );
    },
  }
);
```

## 用户体验改进

### 之前（非流式）
- 用户发送消息后需要等待完整响应
- 显示"正在输入..."加载动画
- 响应完成后一次性显示全部内容
- 长文本响应时等待时间较长

### 现在（SSE流式）
- 用户发送消息后立即看到响应开始
- 文字逐字显示，类似真人打字效果
- 首字节延迟低，用户感知更快
- 更好的交互体验和参与感

## 测试工具

### HTML测试页面

位置：`frontend/src/examples/sse-chat-test.html`

**功能：**
- 独立的HTML页面，无需构建前端项目
- 可直接在浏览器中打开测试
- 支持配置API地址、Token、Agent ID
- 实时显示流式响应
- 显示元数据（会话ID、消息ID、token使用量等）
- 显示响应时间统计

**使用方法：**

1. 在浏览器中打开 `frontend/src/examples/sse-chat-test.html`
2. 填写必要信息：
   - API地址（默认：`http://localhost:3000/api`）
   - 认证Token
   - Agent ID
   - 消息内容
3. 点击"发送消息"按钮
4. 观察实时流式响应

### Shell测试脚本

位置：`test_sse_chat.sh`

**使用方法：**

```bash
# 设置环境变量
export API_URL="http://localhost:3000/api"
export TOKEN="your-auth-token"
export AGENT_ID="your-agent-id"

# 运行测试
./test_sse_chat.sh
```

## API端点

### 流式聊天端点

```
POST /api/agents/{agent_id}/chat/stream
```

**请求头：**
```
Authorization: Bearer <token>
Content-Type: application/json
Accept: text/event-stream
```

**请求体：**
```json
{
  "message": "你好",
  "session_id": "optional-uuid",
  "stream": true
}
```

**响应格式：**

SSE流，每个事件格式为：

```
data: {"type":"content","content":"你","session_id":"...","message_id":"..."}

data: {"type":"content","content":"好","session_id":"...","message_id":"..."}

data: {"type":"done","session_id":"...","message_id":"...","reply_id":"...","metadata":{...}}
```

## 兼容性

### 浏览器支持
- Chrome/Edge: ✅ 完全支持
- Firefox: ✅ 完全支持
- Safari: ✅ 完全支持
- IE11: ❌ 不支持（需要polyfill）

### 后端要求
- 已实现SSE流式接口（`chat_with_agent_stream`）
- 支持15秒心跳保持连接
- 正确的CORS配置

## 故障排查

### 问题1：连接立即断开

**可能原因：**
- Token无效或过期
- Agent ID不存在
- CORS配置问题

**解决方法：**
- 检查浏览器控制台的网络请求
- 验证Token是否有效
- 检查后端CORS配置

### 问题2：内容不显示

**可能原因：**
- SSE数据格式解析错误
- 回调函数未正确设置

**解决方法：**
- 检查浏览器控制台的错误日志
- 使用测试页面验证后端响应格式
- 确认回调函数正确实现

### 问题3：响应缓慢

**可能原因：**
- LLM服务响应慢
- 网络延迟
- 代理服务器缓冲

**解决方法：**
- 检查LLM服务状态
- 测试网络延迟
- 配置代理服务器禁用缓冲（nginx: `proxy_buffering off;`）

## 性能优化建议

1. **连接复用**：同一会话中复用连接
2. **错误重试**：实现自动重连机制
3. **取消请求**：支持用户中断生成
4. **内容缓存**：缓存历史消息避免重复请求
5. **批量更新**：使用 `requestAnimationFrame` 优化UI更新频率

## 后续改进计划

- [ ] 支持中断生成（取消请求）
- [ ] 添加重连机制
- [ ] 支持多模态内容流式传输
- [ ] 优化大规模并发性能
- [ ] 添加流式进度指示器
- [ ] 支持流式工具调用显示

## 相关文档

- [SSE聊天实现文档](./SSE_CHAT_IMPLEMENTATION.md)
- [SSE聊天快速开始](./SSE_CHAT_QUICKSTART.md)
- [后端API文档](./docs/api.md)
