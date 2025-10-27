# 手机端聊天界面组件使用指南

## 概述

本项目提供了一套通用的手机端聊天界面组件，可以在多个场景中使用：

1. **MobileChatPreview** - 手机端聊天预览组件（用于实时预览）
2. **EmbeddedChat** - 可嵌入式聊天组件（可最小化/最大化）
3. **AgentChatPage** - 独立聊天页面示例

## 组件特性

### MobileChatPreview

完整的手机端聊天界面，包含：
- 手机状态栏（时间、信号、电量）
- 聊天头部（头像、名称、在线状态）
- 消息列表（用户消息、助手消息）
- 预设问题快速选择
- 输入框和发送按钮
- 打字动画效果
- 手机底部指示器

### EmbeddedChat

可嵌入任何页面的浮动聊天组件：
- 浮动按钮触发
- 支持最小化/最大化
- 可配置位置（四个角落）
- 优雅的动画效果

## 使用方法

### 1. 在 Agent 编辑页面中使用（实时预览）

已在 `AgentDetailPage.tsx` 中实现，编辑 Agent 时右侧会实时显示手机端预览效果。

```tsx
import { MobileChatPreview } from '../components/common';

<MobileChatPreview
  agentName={formData.name || 'AI 助手'}
  agentAvatar={formData.avatar}
  systemPrompt={formData.systemPrompt}
  presetQuestions={formData.presetQuestions}
/>
```

### 2. 创建独立的聊天页面

参考 `AgentChatPage.tsx`：

```tsx
import { MobileChatPreview } from '../components/common';

export function AgentChatPage() {
  const handleSendMessage = async (message: string): Promise<string> => {
    // 调用后端 API 处理消息
    const response = await chatService.sendMessage(agentId, message);
    return response.content;
  };

  return (
    <MobileChatPreview
      agentName={agent.name}
      agentAvatar={agent.avatar}
      systemPrompt={agent.system_prompt}
      presetQuestions={agent.preset_questions}
      onSendMessage={handleSendMessage}
      className="h-[700px]"
    />
  );
}
```

### 3. 在任意页面嵌入聊天组件

```tsx
import { EmbeddedChat } from '../components/common';

export function DashboardPage() {
  const handleSendMessage = async (message: string): Promise<string> => {
    // 处理消息
    return await chatService.sendMessage(agentId, message);
  };

  return (
    <div>
      {/* 页面内容 */}
      <h1>Dashboard</h1>
      
      {/* 嵌入式聊天组件 */}
      <EmbeddedChat
        agentId="agent-123"
        agentName="AI 助手"
        agentAvatar="https://example.com/avatar.png"
        systemPrompt="我是您的智能助手"
        presetQuestions={['帮我分析数据', '生成报告', '查看统计']}
        onSendMessage={handleSendMessage}
        position="bottom-right"
      />
    </div>
  );
}
```

### 4. 在 Agent 列表中添加聊天按钮

可以在 `AgentListPage.tsx` 中为每个 Agent 添加"聊天"按钮：

```tsx
<Button
  variant="secondary"
  onClick={() => navigate(`/agents/${agent.id}/chat`)}
>
  开始对话
</Button>
```

## API 接口

### MobileChatPreview Props

```typescript
interface MobileChatPreviewProps {
  agentName: string;              // Agent 名称
  agentAvatar?: string;           // Agent 头像 URL
  systemPrompt?: string;          // 系统提示词（显示在欢迎界面）
  presetQuestions?: string[];     // 预设问题列表
  onSendMessage?: (message: string) => Promise<string>;  // 消息发送处理函数
  className?: string;             // 自定义样式类
}
```

### EmbeddedChat Props

```typescript
interface EmbeddedChatProps {
  agentId: string;                // Agent ID
  agentName: string;              // Agent 名称
  agentAvatar?: string;           // Agent 头像 URL
  systemPrompt?: string;          // 系统提示词
  presetQuestions?: string[];     // 预设问题列表
  onSendMessage?: (message: string) => Promise<string>;  // 消息发送处理函数
  position?: 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left';  // 位置
  className?: string;             // 自定义样式类
}
```

## 消息处理

### 模拟模式

如果不提供 `onSendMessage` 函数，组件会使用模拟响应：

```tsx
<MobileChatPreview
  agentName="测试助手"
  // 不提供 onSendMessage，将使用模拟响应
/>
```

### 真实 API 集成

提供 `onSendMessage` 函数来处理真实的消息发送：

```tsx
const handleSendMessage = async (message: string): Promise<string> => {
  try {
    // 调用后端 API
    const response = await fetch('/api/chat', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        agent_id: agentId,
        message: message,
        session_id: sessionId,
      }),
    });
    
    const data = await response.json();
    return data.response;
  } catch (error) {
    console.error('Failed to send message:', error);
    throw error;
  }
};

<MobileChatPreview
  agentName="AI 助手"
  onSendMessage={handleSendMessage}
/>
```

## 样式定制

### 自定义高度

```tsx
<MobileChatPreview
  agentName="AI 助手"
  className="h-[600px]"  // 自定义高度
/>
```

### 自定义位置（EmbeddedChat）

```tsx
<EmbeddedChat
  agentId="agent-123"
  agentName="AI 助手"
  position="bottom-left"  // 左下角
/>
```

## 路由配置

如果要使用独立的聊天页面，需要在路由中添加：

```tsx
// App.tsx 或路由配置文件
import { AgentChatPage } from './pages/AgentChatPage';

<Route path="/agents/:id/chat" element={<AgentChatPage />} />
```

## 后端集成建议

### 1. 创建聊天会话 API

```rust
// POST /api/agents/:id/chat/sessions
pub async fn create_chat_session(
    agent_id: String,
    user_id: String,
) -> Result<ChatSession>
```

### 2. 发送消息 API

```rust
// POST /api/agents/:id/chat/messages
pub async fn send_message(
    agent_id: String,
    session_id: String,
    message: String,
) -> Result<ChatMessage>
```

### 3. 获取历史消息 API

```rust
// GET /api/agents/:id/chat/sessions/:session_id/messages
pub async fn get_messages(
    agent_id: String,
    session_id: String,
    page: u32,
) -> Result<Vec<ChatMessage>>
```

## 最佳实践

1. **会话管理**：为每个聊天创建独立的会话 ID，便于追踪和恢复对话
2. **错误处理**：在 `onSendMessage` 中妥善处理错误，提供友好的错误提示
3. **加载状态**：组件内置了打字动画，无需额外处理加载状态
4. **消息持久化**：考虑将消息保存到后端，支持跨设备同步
5. **性能优化**：对于长对话，考虑实现消息分页加载

## 示例场景

### 场景 1：Agent 编辑时实时预览
✅ 已实现在 `AgentDetailPage.tsx`

### 场景 2：独立聊天页面
✅ 已实现在 `AgentChatPage.tsx`

### 场景 3：Dashboard 嵌入式助手
```tsx
// DashboardPage.tsx
<EmbeddedChat
  agentId="default-assistant"
  agentName="智能助手"
  position="bottom-right"
  onSendMessage={handleSendMessage}
/>
```

### 场景 4：客服支持页面
```tsx
// SupportPage.tsx
<MobileChatPreview
  agentName="客服助手"
  systemPrompt="我是您的专属客服，有什么可以帮您？"
  presetQuestions={[
    '如何创建 Agent？',
    '如何配置知识库？',
    '如何使用工具？'
  ]}
  onSendMessage={handleSupportMessage}
  className="h-full"
/>
```

## 未来扩展

可以考虑添加的功能：

1. **语音输入**：集成语音识别功能
2. **文件上传**：支持发送图片、文档等
3. **富文本消息**：支持 Markdown、代码高亮等
4. **消息撤回**：允许用户撤回已发送的消息
5. **消息搜索**：在历史消息中搜索
6. **多语言支持**：集成 i18n
7. **主题定制**：支持自定义颜色主题
8. **表情符号**：添加表情选择器
9. **消息引用**：支持引用回复
10. **实时通知**：WebSocket 实时推送

## 技术栈

- React 18
- TypeScript
- Tailwind CSS
- React Router

## 文件结构

```
frontend/src/
├── components/
│   └── common/
│       ├── MobileChatPreview.tsx    # 手机端聊天预览组件
│       ├── EmbeddedChat.tsx         # 嵌入式聊天组件
│       └── index.ts                 # 导出
├── pages/
│   ├── AgentDetailPage.tsx          # Agent 编辑页面（含预览）
│   └── AgentChatPage.tsx            # 独立聊天页面
└── services/
    └── agent.service.ts             # Agent 服务
```

## 总结

这套聊天组件提供了灵活的使用方式，可以满足不同场景的需求：

- ✅ 实时预览（Agent 编辑页面）
- ✅ 独立页面（专门的聊天界面）
- ✅ 嵌入式组件（任意页面浮动显示）
- ✅ 通用设计（可复用于多个场景）

组件设计遵循了 React 最佳实践，易于集成和定制。
