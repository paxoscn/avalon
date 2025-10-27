# 快速开始 - 手机端聊天界面

## 5 分钟快速集成

### 步骤 1：查看实时预览效果

Agent 编辑页面已经集成了实时预览功能，无需任何配置即可查看效果。

1. 启动前端服务：
```bash
cd frontend
npm run dev
```

2. 访问 Agent 编辑页面：
```
http://localhost:5173/agents/new
```

3. 填写表单，右侧会实时显示手机端预览效果

### 步骤 2：添加独立聊天页面（可选）

如果需要为 Agent 创建专门的聊天页面：

1. 在路由配置中添加（`App.tsx` 或路由文件）：
```tsx
import { AgentChatPage } from './pages/AgentChatPage';

<Route path="/agents/:id/chat" element={<AgentChatPage />} />
```

2. 在 Agent 列表中添加"开始对话"按钮：
```tsx
<Link to={`/agents/${agent.id}/chat`}>
  <Button>开始对话</Button>
</Link>
```

3. 访问聊天页面：
```
http://localhost:5173/agents/{agent-id}/chat
```

### 步骤 3：添加嵌入式助手（可选）

在任意页面添加浮动聊天助手：

```tsx
import { EmbeddedChat } from '../components/common';

export function YourPage() {
  const handleSendMessage = async (message: string): Promise<string> => {
    // 处理消息
    return '这是回复';
  };

  return (
    <div>
      {/* 页面内容 */}
      
      {/* 嵌入式聊天 */}
      <EmbeddedChat
        agentId="your-agent-id"
        agentName="智能助手"
        systemPrompt="我可以帮助您..."
        presetQuestions={['问题1', '问题2', '问题3']}
        onSendMessage={handleSendMessage}
        position="bottom-right"
      />
    </div>
  );
}
```

## 组件导入

所有组件都已导出，可以直接使用：

```tsx
import { 
  MobileChatPreview,  // 手机端聊天预览
  EmbeddedChat        // 嵌入式聊天
} from '../components/common';
```

## 基础用法

### 1. 仅预览（无后端）

```tsx
<MobileChatPreview
  agentName="AI 助手"
  agentAvatar="https://example.com/avatar.png"
  systemPrompt="我是您的智能助手"
  presetQuestions={['问题1', '问题2', '问题3']}
/>
```

### 2. 连接后端 API

```tsx
const handleSendMessage = async (message: string): Promise<string> => {
  const response = await fetch('/api/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message }),
  });
  const data = await response.json();
  return data.response;
};

<MobileChatPreview
  agentName="AI 助手"
  onSendMessage={handleSendMessage}
/>
```

## 常见问题

### Q: 如何修改聊天界面的高度？
```tsx
<MobileChatPreview
  agentName="AI 助手"
  className="h-[600px]"  // 自定义高度
/>
```

### Q: 如何改变嵌入式聊天的位置？
```tsx
<EmbeddedChat
  position="bottom-left"  // 可选：bottom-right, bottom-left, top-right, top-left
/>
```

### Q: 如何处理发送消息失败？
```tsx
const handleSendMessage = async (message: string): Promise<string> => {
  try {
    const response = await chatService.sendMessage(message);
    return response.content;
  } catch (error) {
    console.error('Failed to send message:', error);
    return '抱歉，发送消息失败，请稍后再试。';
  }
};
```

### Q: 如何加载历史消息？
目前组件不支持初始化历史消息，但可以通过修改组件添加此功能。参考 `CHAT_INTEGRATION_EXAMPLES.md` 中的最佳实践部分。

## 下一步

- 📖 查看 [MOBILE_CHAT_COMPONENT.md](./MOBILE_CHAT_COMPONENT.md) 了解完整功能
- 💡 查看 [CHAT_INTEGRATION_EXAMPLES.md](./CHAT_INTEGRATION_EXAMPLES.md) 获取更多示例
- 🚀 查看 [MOBILE_CHAT_SUMMARY.md](./MOBILE_CHAT_SUMMARY.md) 了解实现细节

## 需要帮助？

如果遇到问题，请检查：
1. 组件是否正确导入
2. Props 是否正确传递
3. 浏览器控制台是否有错误信息
4. 后端 API 是否正常工作

祝使用愉快！🎉
