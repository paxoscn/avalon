# SSE聊天接口快速入门

## 快速开始

### 1. 后端已实现的功能

✅ SSE流式聊天端点：`POST /api/agents/{agent_id}/chat/stream`
✅ 自动会话管理
✅ 实时流式响应
✅ 统计数据追踪
✅ 错误处理

### 2. 测试接口

#### 方法1：使用测试脚本

```bash
# 设置环境变量
export API_BASE_URL="http://localhost:8080/api"
export AGENT_ID="your-agent-uuid"
export TOKEN="your-jwt-token"

# 运行测试
./test_sse_chat.sh
```

#### 方法2：使用cURL

```bash
curl -N -X POST "http://localhost:8080/api/agents/{agent_id}/chat/stream" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "message": "你好，请介绍一下你自己",
    "stream": true
  }'
```

#### 方法3：使用HTML示例

1. 打开 `examples/sse_chat_example.html` 文件
2. 配置API URL、Agent ID和Token
3. 点击"Save Config"
4. 开始聊天！

### 3. 前端集成

#### React Hook使用

```typescript
import { useAgentChatStream } from './hooks/useAgentChatStream';

function ChatComponent({ agentId }: { agentId: string }) {
  const { messages, currentResponse, isStreaming, sendMessage } = 
    useAgentChatStream({ agentId });

  return (
    <div>
      {messages.map(msg => (
        <div key={msg.id}>{msg.content}</div>
      ))}
      {isStreaming && <div>{currentResponse}</div>}
      <button onClick={() => sendMessage('Hello')}>Send</button>
    </div>
  );
}
```

#### 完整页面组件

```typescript
import AgentChatStream from './components/AgentChatStream';

function App() {
  return <AgentChatStream agentId="your-agent-id" />;
}
```

### 4. API响应格式

#### 内容块
```json
{
  "type": "content",
  "content": "你好",
  "session_id": "uuid",
  "message_id": "uuid"
}
```

#### 完成块
```json
{
  "type": "done",
  "session_id": "uuid",
  "message_id": "uuid",
  "reply_id": "uuid",
  "metadata": {
    "model": "gpt-4",
    "tokens_used": 150
  }
}
```

#### 错误块
```json
{
  "type": "error",
  "error": "错误描述"
}
```

### 5. 与非流式接口对比

| 特性 | 流式 `/chat/stream` | 非流式 `/chat` |
|------|-------------------|---------------|
| 响应方式 | SSE流 | JSON |
| 用户体验 | 实时显示 | 等待完整响应 |
| 首字节延迟 | 低 | 高 |
| 适用场景 | 长文本生成 | 短文本、API调用 |

### 6. 常见问题

#### Q: 如何取消正在进行的流？
A: 使用AbortController或关闭SSE连接

```typescript
const controller = new AbortController();
fetch(url, { signal: controller.signal });
// 取消
controller.abort();
```

#### Q: 如何处理网络中断？
A: 实现自动重连机制

```typescript
const { sendMessage, cancelStream } = useAgentChatStream({
  agentId,
  onError: (error) => {
    // 重试逻辑
    setTimeout(() => sendMessage(lastMessage), 3000);
  }
});
```

#### Q: 如何保持会话？
A: 保存并传递session_id

```typescript
const [sessionId, setSessionId] = useState<string>();

// 首次请求后保存session_id
// 后续请求传递相同的session_id
```

### 7. 性能优化建议

1. **使用连接池**：复用HTTP连接
2. **实现背压**：控制流速度
3. **批量更新UI**：使用requestAnimationFrame
4. **虚拟滚动**：处理大量消息
5. **懒加载历史**：按需加载旧消息

### 8. 安全注意事项

1. **Token管理**：安全存储JWT token
2. **输入验证**：验证用户输入
3. **速率限制**：防止滥用
4. **CORS配置**：正确配置跨域
5. **HTTPS**：生产环境使用HTTPS

### 9. 监控和调试

#### 浏览器开发者工具
- Network标签查看SSE连接
- Console查看事件日志
- Performance分析性能

#### 服务器日志
```rust
// 在handler中添加日志
log::info!("SSE stream started for agent: {}", agent_id);
log::debug!("Chunk sent: {:?}", chunk);
```

### 10. 下一步

- [ ] 实现消息历史加载
- [ ] 添加文件上传支持
- [ ] 实现多模态内容
- [ ] 添加工具调用支持
- [ ] 优化大规模并发

## 文件清单

- ✅ `src/application/dto/agent_dto.rs` - DTO定义
- ✅ `src/application/services/agent_application_service.rs` - 服务实现
- ✅ `src/presentation/handlers/agent_handlers.rs` - HTTP处理器
- ✅ `src/presentation/routes/agent_routes.rs` - 路由配置
- ✅ `frontend/src/hooks/useAgentChatStream.ts` - React Hook
- ✅ `frontend/src/components/AgentChatStream.tsx` - React组件
- ✅ `frontend/src/pages/AgentChatStreamPage.tsx` - 页面组件
- ✅ `examples/sse_chat_example.html` - 独立HTML示例
- ✅ `test_sse_chat.sh` - 测试脚本
- ✅ `SSE_CHAT_IMPLEMENTATION.md` - 详细文档

## 支持

如有问题，请查看：
1. `SSE_CHAT_IMPLEMENTATION.md` - 完整实现文档
2. `examples/sse_chat_example.html` - 可运行的示例
3. 项目issue tracker
