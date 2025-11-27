# MobileChatPreview 思考过程功能更新

## 更新内容

为 `MobileChatPreview.tsx` 组件添加了思考过程（reasoning_content）支持。

## 主要更改

### 1. ChatMessage 接口更新

```typescript
export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  reasoning?: string;  // 新增：思考过程
  timestamp: Date;
}
```

### 2. 状态管理

添加了展开/收起状态管理：

```typescript
const [expandedReasonings, setExpandedReasonings] = useState<Set<string>>(new Set());

const toggleReasoning = (messageId: string) => {
  setExpandedReasonings((prev) => {
    const next = new Set(prev);
    if (next.has(messageId)) {
      next.delete(messageId);
    } else {
      next.add(messageId);
    }
    return next;
  });
};
```

### 3. 流式响应处理

添加了 `onReasoning` 回调：

```typescript
let fullContent = '';
let fullReasoning = '';

await chatService.chatStream(
  { agentId, message: content.trim(), sessionId },
  {
    onContent: (chunk) => {
      fullContent += chunk;
      // 更新消息内容
    },
    onReasoning: (chunk) => {
      fullReasoning += chunk;
      // 更新思考过程
    },
    onDone: (data) => {
      // 完成处理
    },
  }
);
```

### 4. UI 渲染

添加了思考过程按钮和展开内容：

```tsx
{message.reasoning && message.role === 'assistant' && (
  <div className="mt-2">
    <button onClick={() => toggleReasoning(message.id)}>
      <svg className={expandedReasonings.has(message.id) ? 'rotate-90' : ''}>
        {/* 箭头图标 */}
      </svg>
      <span>💭 思考过程</span>
    </button>
    
    {expandedReasonings.has(message.id) && (
      <div className="mt-2 p-2 bg-gradient-to-r from-amber-50 to-orange-50">
        <p>{message.reasoning}</p>
      </div>
    )}
  </div>
)}
```

## UI 效果

### 移动端消息气泡（收起状态）

```
┌─────────────────────────────┐
│ 根据您的问题，我的理解是... │
│                             │
│ ▶ 💭 思考过程               │
│                             │
│ 9:41                        │
└─────────────────────────────┘
```

### 移动端消息气泡（展开状态）

```
┌─────────────────────────────┐
│ 根据您的问题，我的理解是... │
│                             │
│ ▼ 💭 思考过程               │
│ ┌─────────────────────────┐ │
│ │ 正在分析您的问题...      │ │
│ │ 检索相关知识库...        │ │
│ └─────────────────────────┘ │
│                             │
│ 9:41                        │
└─────────────────────────────┘
```

## 样式特点

### 移动端优化

- **按钮大小**：`text-xs` - 适合移动端的小字体
- **图标大小**：`w-3 h-3` - 更小的图标
- **内边距**：`p-2` - 紧凑的内边距
- **字体大小**：`text-xs` - 小字体以节省空间

### 颜色方案

- **按钮颜色**：`text-amber-700` / `hover:text-amber-800`
- **背景渐变**：`from-amber-50 to-orange-50`
- **边框颜色**：`border-amber-200`
- **文字颜色**：`text-amber-900`

## 功能特点

### 1. 实时更新
- 流式响应时，思考过程实时更新
- 内容和思考过程独立更新

### 2. 可展开/收起
- 点击按钮展开或收起思考过程
- 箭头图标旋转表示状态
- 每个消息独立控制

### 3. 移动端友好
- 紧凑的布局
- 适合触摸的按钮大小
- 响应式设计

### 4. 性能优化
- 只在需要时渲染展开的内容
- 使用 Set 高效管理展开状态

## 使用示例

```tsx
<MobileChatPreview
  agentId="agent-123"
  agentName="AI 助手"
  greeting="你好！我是 AI 助手"
  presetQuestions={[
    "你能做什么？",
    "如何使用？"
  ]}
/>
```

## 兼容性

- ✅ 向后兼容：没有 reasoning 的消息不显示按钮
- ✅ 移动端优化：适合小屏幕显示
- ✅ 触摸友好：按钮大小适合触摸操作
- ✅ 性能优化：按需渲染

## 测试清单

- [ ] 有 reasoning 的消息显示按钮
- [ ] 没有 reasoning 的消息不显示按钮
- [ ] 点击按钮可以展开/收起
- [ ] 箭头图标正确旋转
- [ ] 展开内容样式正确
- [ ] 流式响应时实时更新
- [ ] 多个消息可以独立控制
- [ ] 移动端显示正常
- [ ] 触摸操作流畅

## 与桌面版对比

| 特性 | 桌面版 (AgentChatStream) | 移动版 (MobileChatPreview) |
|------|-------------------------|---------------------------|
| 按钮大小 | `text-sm` | `text-xs` |
| 图标大小 | `w-4 h-4` | `w-3 h-3` |
| 内边距 | `p-3` | `p-2` |
| 字体大小 | `text-sm` | `text-xs` |
| 布局 | 宽松 | 紧凑 |

## 相关文件

- `frontend/src/components/common/MobileChatPreview.tsx` - 移动端聊天组件
- `frontend/src/components/AgentChatStream.tsx` - 桌面端聊天组件
- `frontend/src/services/chat.service.ts` - 聊天服务
- `frontend/src/hooks/useAgentChatStream.ts` - 聊天 Hook

## 总结

`MobileChatPreview` 现在完全支持思考过程功能：

- ✅ 实时流式更新
- ✅ 可展开/收起
- ✅ 移动端优化
- ✅ 性能优化
- ✅ 向后兼容

与桌面版保持一致的功能，但针对移动端进行了优化。
