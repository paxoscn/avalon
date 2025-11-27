# 思考过程流式显示行为

## 设计说明

思考过程在流式响应时自动展开并逐字显示，完成后自动收起到按钮中。

## 用户体验流程

### 1. 发送消息
```
用户输入消息 → 点击发送
```

### 2. 流式响应开始

#### 桌面版 (AgentChatStream)
```
🤖 [思考过程卡片 - 自动展开]
   💭 思考过程
   ─────────────────
   正在分析问题...█
```

#### 移动版 (MobileChatPreview)
```
┌─────────────────────────────┐
│ 💭 思考中                   │
│ 正在分析问题...█            │
└─────────────────────────────┘
```

### 3. 思考过程完成，开始回复
```
🤖 [回复内容 - 实时显示]
   根据分析...█
```

### 4. 完成后 - 思考过程自动收起

#### 桌面版
```
🤖 [回复消息]
   根据分析结果，我建议...
   
   ▶ 💭 思考过程  ← 默认收起，可点击展开
   ─────────────────
   📦 o1-preview | 🔢 150 tokens
```

#### 移动版
```
┌─────────────────────────────┐
│ 根据分析结果，我建议...     │
│                             │
│ ▶ 💭 思考过程               │
│                             │
│ 9:41                        │
└─────────────────────────────┘
```

### 5. 用户可以手动展开查看
```
点击 "💭 思考过程" 按钮
    ↓
展开显示完整的思考过程
    ↓
再次点击可以收起
```

## 技术实现

### 桌面版 (AgentChatStream.tsx)

#### 流式响应时
```tsx
{isStreaming && (
  <div className="flex items-start space-x-3">
    {/* 思考过程 - 自动展开 */}
    {currentReasoning && (
      <div className="bg-gradient-to-r from-amber-50 to-orange-50">
        <div className="flex items-center">
          <svg className="animate-spin">...</svg>
          <span>💭 思考过程</span>
        </div>
        <div>
          {currentReasoning}
          <span className="animate-pulse">█</span>
        </div>
      </div>
    )}
    
    {/* 回复内容 */}
    {currentResponse && (
      <div className="bg-white">
        {currentResponse}
        <span className="animate-pulse">█</span>
      </div>
    )}
  </div>
)}
```

#### 完成后
```tsx
{message.reasoning && (
  <div className="mt-3">
    {/* 默认收起 */}
    <button onClick={() => toggleReasoning(message.id)}>
      <svg className={expandedReasonings.has(message.id) ? 'rotate-90' : ''}>
        →
      </svg>
      <span>💭 思考过程</span>
    </button>
    
    {/* 点击后展开 */}
    {expandedReasonings.has(message.id) && (
      <div className="mt-2 p-3 bg-gradient-to-r from-amber-50 to-orange-50">
        <p>{message.reasoning}</p>
      </div>
    )}
  </div>
)}
```

### 移动版 (MobileChatPreview.tsx)

#### 流式响应时
```tsx
{isTyping && currentReasoning && (
  <div className="flex justify-start">
    <div className="bg-gradient-to-r from-amber-50 to-orange-50">
      <div className="flex items-center">
        <svg className="animate-spin">...</svg>
        <span>💭 思考中</span>
      </div>
      <p>
        {currentReasoning}
        <span className="animate-pulse">█</span>
      </p>
    </div>
  </div>
)}
```

#### 完成后
```tsx
{message.reasoning && message.role === 'assistant' && (
  <div className="mt-2">
    {/* 默认收起 */}
    <button onClick={() => toggleReasoning(message.id)}>
      <svg className={expandedReasonings.has(message.id) ? 'rotate-90' : ''}>
        →
      </svg>
      <span>💭 思考过程</span>
    </button>
    
    {/* 点击后展开 */}
    {expandedReasonings.has(message.id) && (
      <div className="mt-2 p-2 bg-gradient-to-r from-amber-50 to-orange-50">
        <p>{message.reasoning}</p>
      </div>
    )}
  </div>
)}
```

## 状态管理

### 流式响应状态

**桌面版**：
```typescript
const [currentReasoning, setCurrentReasoning] = useState('');

// 流式响应时
if (data.reasoning_content) {
  accumulatedReasoning += data.reasoning_content;
  setCurrentReasoning(accumulatedReasoning);  // 实时显示
}

// 完成时
setCurrentReasoning('');  // 清空临时状态
```

**移动版**：
```typescript
const [currentReasoning, setCurrentReasoning] = useState('');

// 流式响应时
onReasoning: (chunk) => {
  fullReasoning += chunk;
  setCurrentReasoning(fullReasoning);  // 实时显示
}

// 完成时
onDone: (data) => {
  setCurrentReasoning('');  // 清空临时状态
}
```

### 展开/收起状态

```typescript
const [expandedReasonings, setExpandedReasonings] = useState<Set<string>>(new Set());

const toggleReasoning = (messageId: string) => {
  setExpandedReasonings((prev) => {
    const next = new Set(prev);
    if (next.has(messageId)) {
      next.delete(messageId);  // 收起
    } else {
      next.add(messageId);     // 展开
    }
    return next;
  });
};
```

## 视觉效果

### 流式响应时

**思考过程卡片**：
- 琥珀色渐变背景
- 旋转的加载图标
- 闪烁的打字机光标
- 实时更新的文字

**回复内容卡片**：
- 白色背景
- 闪烁的打字机光标
- 实时更新的文字

### 完成后

**收起状态**：
- 只显示 "▶ 💭 思考过程" 按钮
- 箭头向右
- 琥珀色文字

**展开状态**：
- 显示 "▼ 💭 思考过程" 按钮
- 箭头向下（旋转 90 度）
- 显示完整的思考内容
- 琥珀色渐变背景

## 动画效果

### 1. 流式响应时
- 加载图标旋转（1s 循环）
- 打字机光标闪烁（1s 循环）
- 文字逐字出现

### 2. 展开/收起
- 箭头旋转动画（0.3s）
- 内容淡入/淡出（可选）

### 3. 完成时
- 思考过程卡片消失
- 按钮出现在消息中

## 优势

### 1. 清晰的视觉反馈
- 流式响应时，用户可以看到 AI 正在思考
- 完成后，界面更简洁

### 2. 更好的可读性
- 思考过程不会永久占据空间
- 用户可以选择性查看

### 3. 更流畅的体验
- 实时显示思考过程
- 自动收起避免干扰

### 4. 灵活的交互
- 用户可以随时展开查看
- 可以对比多个消息的思考过程

## 用户场景

### 场景 1：快速浏览
```
用户发送消息
  ↓
看到思考过程（了解 AI 在做什么）
  ↓
看到回复内容
  ↓
继续对话（思考过程已收起，界面简洁）
```

### 场景 2：深入了解
```
用户发送消息
  ↓
看到思考过程
  ↓
看到回复内容
  ↓
点击 "💭 思考过程" 按钮
  ↓
查看完整的思考过程
  ↓
理解 AI 的推理逻辑
```

### 场景 3：对比分析
```
用户发送多个问题
  ↓
展开第一个回复的思考过程
  ↓
展开第二个回复的思考过程
  ↓
对比不同的推理方式
```

## 测试清单

- [ ] 流式响应时思考过程自动展开
- [ ] 思考过程实时逐字显示
- [ ] 打字机光标正常闪烁
- [ ] 加载图标正常旋转
- [ ] 完成后思考过程自动收起
- [ ] 按钮正确显示
- [ ] 点击按钮可以展开
- [ ] 展开后显示完整内容
- [ ] 再次点击可以收起
- [ ] 箭头旋转动画正常
- [ ] 多个消息可以独立控制
- [ ] 移动端和桌面端都正常工作

## 相关文件

- `frontend/src/components/AgentChatStream.tsx` - 桌面版聊天组件
- `frontend/src/components/common/MobileChatPreview.tsx` - 移动版聊天组件
- `frontend/src/hooks/useAgentChatStream.ts` - 聊天 Hook
- `frontend/src/services/chat.service.ts` - 聊天服务

## 总结

新的设计提供了最佳的用户体验：

1. **流式响应时**：思考过程自动展开，实时显示，让用户了解 AI 的工作状态
2. **完成后**：思考过程自动收起，保持界面简洁
3. **按需查看**：用户可以通过按钮随时展开查看详细的思考过程

这种设计平衡了透明度和简洁性，既让用户了解 AI 的思考过程，又不会让界面过于拥挤。
