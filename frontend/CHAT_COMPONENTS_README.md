# 手机端聊天界面组件

## 📱 概述

本项目实现了一套完整的手机端聊天界面组件系统，可以在多个场景中使用：

- ✅ **实时预览**：在 Agent 编辑页面右侧显示手机端预览
- ✅ **独立页面**：专门的全屏聊天界面
- ✅ **嵌入式组件**：可在任意页面添加浮动聊天助手

## 🚀 快速开始

### 查看效果

1. 启动前端服务：
```bash
cd frontend
npm run dev
```

2. 访问 Agent 编辑页面查看实时预览：
```
http://localhost:5173/agents/new
```

3. 填写表单，右侧会实时显示手机端聊天界面

### 基础使用

```tsx
import { MobileChatPreview } from '../components/common';

<MobileChatPreview
  agentName="AI 助手"
  agentAvatar="https://example.com/avatar.png"
  systemPrompt="我是您的智能助手"
  presetQuestions={['问题1', '问题2', '问题3']}
/>
```

## 📦 组件列表

### 1. MobileChatPreview
完整的手机端聊天界面，包含状态栏、聊天头部、消息列表、输入框等。

**特点**：
- 真实的手机界面模拟
- 支持预设问题
- 打字动画效果
- 自动滚动到最新消息
- 渐变色主题设计

**Props**：
```typescript
interface MobileChatPreviewProps {
  agentName: string;              // Agent 名称
  agentAvatar?: string;           // Agent 头像 URL
  systemPrompt?: string;          // 系统提示词
  presetQuestions?: string[];     // 预设问题列表
  onSendMessage?: (message: string) => Promise<string>;  // 消息处理函数
  className?: string;             // 自定义样式
}
```

### 2. EmbeddedChat
可嵌入任何页面的浮动聊天组件。

**特点**：
- 浮动按钮触发
- 支持最小化/最大化
- 可配置位置（四个角落）
- 优雅的动画效果

**Props**：
```typescript
interface EmbeddedChatProps {
  agentId: string;                // Agent ID
  agentName: string;              // Agent 名称
  agentAvatar?: string;           // Agent 头像 URL
  systemPrompt?: string;          // 系统提示词
  presetQuestions?: string[];     // 预设问题列表
  onSendMessage?: (message: string) => Promise<string>;  // 消息处理函数
  position?: 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left';
  className?: string;             // 自定义样式
}
```

## 📄 文档

| 文档 | 说明 |
|------|------|
| [QUICK_START_CHAT.md](./QUICK_START_CHAT.md) | 5 分钟快速开始指南 |
| [MOBILE_CHAT_COMPONENT.md](./MOBILE_CHAT_COMPONENT.md) | 完整的组件使用指南 |
| [CHAT_INTEGRATION_EXAMPLES.md](./CHAT_INTEGRATION_EXAMPLES.md) | 5 种集成场景的完整示例 |
| [MOBILE_CHAT_SUMMARY.md](./MOBILE_CHAT_SUMMARY.md) | 实现总结和技术细节 |

## 🎯 使用场景

### 场景 1：Agent 编辑页面实时预览
✅ **已实现**：`AgentDetailPage.tsx`

编辑 Agent 配置时，右侧实时显示手机端预览效果。

### 场景 2：独立聊天页面
✅ **已实现**：`AgentChatPage.tsx`

为每个 Agent 创建专门的全屏聊天页面。

```tsx
// 路由配置
<Route path="/agents/:id/chat" element={<AgentChatPage />} />
```

### 场景 3：Dashboard 嵌入式助手
✅ **已实现**：`DashboardWithChatPage.tsx`

在 Dashboard 页面添加浮动聊天助手。

```tsx
<EmbeddedChat
  agentId="assistant-id"
  agentName="智能助手"
  position="bottom-right"
  onSendMessage={handleSendMessage}
/>
```

### 场景 4：Agent 列表添加聊天入口
在 Agent 列表中添加"开始对话"按钮：

```tsx
<Link to={`/agents/${agent.id}/chat`}>
  <Button>开始对话</Button>
</Link>
```

### 场景 5：多 Agent 切换聊天
在一个页面中管理多个 Agent 的对话。

参考：[CHAT_INTEGRATION_EXAMPLES.md](./CHAT_INTEGRATION_EXAMPLES.md#5-多-agent-切换聊天)

## 🔌 后端集成

### 创建聊天服务

```typescript
// frontend/src/services/chat.service.ts
class ChatService {
  async sendMessage(agentId: string, message: string): Promise<string> {
    const response = await apiClient.post(`/agents/${agentId}/chat`, {
      message,
    });
    return response.data.content;
  }
}
```

### 在组件中使用

```tsx
const handleSendMessage = async (message: string): Promise<string> => {
  try {
    const response = await chatService.sendMessage(agentId, message);
    return response;
  } catch (error) {
    console.error('Failed to send message:', error);
    return '抱歉，发送消息失败。';
  }
};

<MobileChatPreview
  agentName="AI 助手"
  onSendMessage={handleSendMessage}
/>
```

## 🎨 自定义样式

### 修改高度

```tsx
<MobileChatPreview
  agentName="AI 助手"
  className="h-[600px]"
/>
```

### 修改位置（EmbeddedChat）

```tsx
<EmbeddedChat
  agentId="assistant-id"
  agentName="智能助手"
  position="bottom-left"  // 左下角
/>
```

## 📁 文件结构

```
frontend/src/
├── components/common/
│   ├── MobileChatPreview.tsx       # 手机端聊天预览组件
│   ├── EmbeddedChat.tsx            # 嵌入式聊天组件
│   └── index.ts                    # 导出
├── pages/
│   ├── AgentDetailPage.tsx         # Agent 编辑页面（含预览）
│   ├── AgentChatPage.tsx           # 独立聊天页面
│   └── DashboardWithChatPage.tsx   # Dashboard 示例
└── docs/
    ├── QUICK_START_CHAT.md         # 快速开始
    ├── MOBILE_CHAT_COMPONENT.md    # 组件指南
    ├── CHAT_INTEGRATION_EXAMPLES.md # 集成示例
    ├── MOBILE_CHAT_SUMMARY.md      # 实现总结
    └── CHAT_COMPONENTS_README.md   # 本文档
```

## 🛠️ 技术栈

- **React 18** - UI 框架
- **TypeScript** - 类型安全
- **Tailwind CSS** - 样式和动画
- **React Router** - 路由管理

## ✨ 特性

- ✅ 完整的手机界面模拟（状态栏、底部指示器）
- ✅ 流畅的动画效果（打字动画、消息滚动）
- ✅ 支持预设问题快速选择
- ✅ 实时预览（编辑即时更新）
- ✅ 灵活的集成方式（预览、独立、嵌入）
- ✅ 完整的 TypeScript 类型定义
- ✅ 响应式设计
- ✅ 易于扩展和定制

## 🔮 未来扩展

可以考虑添加的功能：

- 语音输入
- 文件上传
- 富文本消息（Markdown、代码高亮）
- 消息撤回
- 消息搜索
- 多语言支持（i18n）
- 主题定制
- 表情符号
- 消息引用
- 实时通知（WebSocket）

## 📝 最佳实践

### 1. 会话管理

使用 localStorage 保存会话 ID：

```tsx
const getOrCreateSession = async (agentId: string) => {
  const key = `chat_session_${agentId}`;
  let sessionId = localStorage.getItem(key);
  
  if (!sessionId) {
    const response = await chatService.createSession(agentId);
    sessionId = response.session_id;
    localStorage.setItem(key, sessionId);
  }
  
  return sessionId;
};
```

### 2. 错误处理

提供友好的错误消息：

```tsx
const handleSendMessage = async (message: string): Promise<string> => {
  try {
    return await chatService.sendMessage(agentId, message);
  } catch (error: any) {
    if (error.response?.status === 429) {
      return '请求过于频繁，请稍后再试。';
    } else if (error.response?.status === 500) {
      return '服务器错误，请稍后再试。';
    } else {
      return '发送消息失败，请检查网络连接。';
    }
  }
};
```

### 3. 性能优化

- 使用防抖处理输入
- 实现消息虚拟滚动
- 缓存 Agent 信息
- 分页加载历史消息

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT

---

**快速链接**：
- [快速开始](./QUICK_START_CHAT.md)
- [完整文档](./MOBILE_CHAT_COMPONENT.md)
- [集成示例](./CHAT_INTEGRATION_EXAMPLES.md)
- [实现总结](./MOBILE_CHAT_SUMMARY.md)
