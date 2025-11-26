# SSE聊天接口实现文档

## 概述

已成功实现基于SSE（Server-Sent Events）协议的流式聊天接口，允许客户端实时接收Agent的响应流。

## 实现内容

### 1. 数据传输对象 (DTO)

在 `src/application/dto/agent_dto.rs` 中添加：

```rust
/// Agent chat stream chunk DTO (for SSE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatStreamChunk {
    #[serde(rename = "type")]
    pub chunk_type: String,        // "content" | "done" | "error"
    pub content: Option<String>,   // 流式内容片段
    pub session_id: Option<Uuid>,  // 会话ID
    pub message_id: Option<Uuid>,  // 用户消息ID
    pub reply_id: Option<Uuid>,    // 助手回复ID
    pub metadata: Option<serde_json::Value>, // 元数据（模型、token等）
    pub finish_reason: Option<String>, // 完成原因
    pub error: Option<String>,     // 错误信息
}
```

### 2. 应用服务层

在 `src/application/services/agent_application_service.rs` 中：

#### 接口定义
```rust
async fn chat_stream(
    &self,
    agent_id: AgentId,
    message: String,
    session_id: Option<SessionId>,
    user_id: UserId,
    tenant_id: TenantId,
) -> Result<Box<dyn Stream<Item = Result<AgentChatStreamChunk>> + Send + Unpin>>;
```

#### 实现特性
- 自动创建或使用现有会话
- 构建包含系统提示和问候语的对话历史
- 调用LLM服务的流式接口
- 实时转换并发送响应块
- 在流结束时保存完整消息到会话
- 自动记录统计信息（消息数、token使用量）

### 3. 处理器层

在 `src/presentation/handlers/agent_handlers.rs` 中添加：

```rust
pub async fn chat_with_agent_stream(
    State(service): State<Arc<dyn AgentApplicationService>>,
    user: AuthenticatedUser,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<AgentChatRequest>,
) -> Result<impl IntoResponse>
```

特性：
- 使用Axum的SSE响应类型
- 自动转换流为SSE事件
- 15秒心跳保持连接
- 错误自动转换为SSE事件

### 4. 路由配置

在 `src/presentation/routes/agent_routes.rs` 中添加：

```rust
.route("/agents/{agent_id}/chat/stream", post(agent_handlers::chat_with_agent_stream))
```

## API使用说明

### 端点

```
POST /api/agents/{agent_id}/chat/stream
```

### 请求头

```
Authorization: Bearer <token>
Content-Type: application/json
Accept: text/event-stream
```

### 请求体

```json
{
  "message": "你好，请介绍一下你自己",
  "session_id": "optional-session-uuid",
  "stream": true
}
```

### 响应格式

SSE流，每个事件包含JSON数据：

#### 内容块（持续发送）
```json
{
  "type": "content",
  "content": "你好",
  "session_id": "uuid",
  "message_id": "uuid",
  "reply_id": null,
  "metadata": null,
  "finish_reason": null,
  "error": null
}
```

#### 完成块（最后一个）
```json
{
  "type": "done",
  "content": null,
  "session_id": "uuid",
  "message_id": "uuid",
  "reply_id": "uuid",
  "metadata": {
    "model": "gpt-4",
    "tokens_used": 150,
    "finish_reason": "Stop"
  },
  "finish_reason": "Stop",
  "error": null
}
```

#### 错误块
```json
{
  "type": "error",
  "content": null,
  "session_id": "uuid",
  "message_id": "uuid",
  "reply_id": null,
  "metadata": null,
  "finish_reason": null,
  "error": "错误描述"
}
```

## 客户端示例

### JavaScript/TypeScript

```typescript
async function chatWithAgentStream(agentId: string, message: string, sessionId?: string) {
  const response = await fetch(`/api/agents/${agentId}/chat/stream`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
      'Accept': 'text/event-stream',
    },
    body: JSON.stringify({
      message,
      session_id: sessionId,
      stream: true,
    }),
  });

  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const chunk = decoder.decode(value);
    const lines = chunk.split('\n');

    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = JSON.parse(line.slice(6));
        
        if (data.type === 'content') {
          // 显示内容
          console.log('Content:', data.content);
        } else if (data.type === 'done') {
          // 完成
          console.log('Done:', data.metadata);
        } else if (data.type === 'error') {
          // 错误
          console.error('Error:', data.error);
        }
      }
    }
  }
}
```

### 使用EventSource（更简单）

```typescript
function chatWithAgentStream(agentId: string, message: string, sessionId?: string) {
  // 注意：EventSource不支持POST，需要使用fetch-event-source库
  // 或者使用上面的fetch方式
  
  const eventSource = new EventSource(
    `/api/agents/${agentId}/chat/stream?message=${encodeURIComponent(message)}`
  );

  eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);
    
    if (data.type === 'content') {
      console.log('Content:', data.content);
    } else if (data.type === 'done') {
      console.log('Done:', data.metadata);
      eventSource.close();
    } else if (data.type === 'error') {
      console.error('Error:', data.error);
      eventSource.close();
    }
  };

  eventSource.onerror = (error) => {
    console.error('SSE Error:', error);
    eventSource.close();
  };
}
```

### React示例

```tsx
import { useState, useEffect } from 'react';

function ChatComponent({ agentId }: { agentId: string }) {
  const [message, setMessage] = useState('');
  const [response, setResponse] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);

  const sendMessage = async () => {
    setIsStreaming(true);
    setResponse('');

    const res = await fetch(`/api/agents/${agentId}/chat/stream`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
        'Accept': 'text/event-stream',
      },
      body: JSON.stringify({ message }),
    });

    const reader = res.body.getReader();
    const decoder = new TextDecoder();

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const chunk = decoder.decode(value);
      const lines = chunk.split('\n');

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = JSON.parse(line.slice(6));
          
          if (data.type === 'content' && data.content) {
            setResponse(prev => prev + data.content);
          } else if (data.type === 'done') {
            setIsStreaming(false);
          } else if (data.type === 'error') {
            console.error(data.error);
            setIsStreaming(false);
          }
        }
      }
    }
  };

  return (
    <div>
      <input 
        value={message} 
        onChange={(e) => setMessage(e.target.value)}
        disabled={isStreaming}
      />
      <button onClick={sendMessage} disabled={isStreaming}>
        发送
      </button>
      <div>{response}</div>
    </div>
  );
}
```

### cURL测试

```bash
curl -N -X POST "http://localhost:8080/api/agents/{agent_id}/chat/stream" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "message": "你好",
    "stream": true
  }'
```

## 技术特性

### 1. 流式处理
- 使用Rust的异步流（Stream trait）
- 零拷贝数据传输
- 低延迟响应

### 2. 会话管理
- 自动创建新会话或使用现有会话
- 完整的消息历史记录
- 支持多轮对话

### 3. 统计追踪
- 自动记录会话数
- 追踪消息数量
- 统计token使用量

### 4. 错误处理
- 流中的错误自动转换为错误事件
- 不会中断整个连接
- 客户端可以优雅处理错误

### 5. 连接保持
- 15秒心跳保持连接活跃
- 防止代理服务器超时
- 自动重连机制（客户端实现）

## 与非流式接口的对比

| 特性 | 流式接口 | 非流式接口 |
|------|---------|-----------|
| 端点 | `/agents/{id}/chat/stream` | `/agents/{id}/chat` |
| 响应类型 | SSE流 | JSON |
| 用户体验 | 实时显示 | 等待完整响应 |
| 延迟 | 低（首字节快） | 高（等待完成） |
| 适用场景 | 长文本生成 | 短文本、API调用 |

## 注意事项

1. **浏览器兼容性**：SSE在所有现代浏览器中都支持
2. **代理配置**：某些代理可能需要配置以支持流式响应
3. **超时设置**：确保服务器和客户端的超时设置足够长
4. **错误重试**：建议客户端实现自动重连机制
5. **资源清理**：确保在组件卸载时关闭SSE连接

## 测试建议

1. 测试长文本生成的流式效果
2. 测试网络中断后的重连
3. 测试并发多个流式连接
4. 测试会话持久化
5. 测试统计数据的准确性

## 未来改进

1. 支持中断生成（取消请求）
2. 支持流式工具调用
3. 添加流式进度指示
4. 支持多模态内容流式传输
5. 优化大规模并发性能
