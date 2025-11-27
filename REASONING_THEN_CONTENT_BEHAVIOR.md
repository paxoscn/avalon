# 思考过程优先显示行为

## 设计原则

**思考过程结束前，不显示回复内容气泡**

这样可以让用户清楚地看到 AI 的思考过程，然后再看到最终的回复内容，避免两者同时显示造成混乱。

## 用户体验流程

### 阶段 1：发送消息
```
用户输入消息 → 点击发送
```

### 阶段 2：思考过程（独占显示）
```
🤖 [思考过程卡片]
   💭 思考过程
   ─────────────────
   正在分析问题...
   检索相关知识库...
   整理回答思路...█
```

**此时不显示回复内容气泡**

### 阶段 3：思考结束，开始显示回复
```
思考过程卡片消失
    ↓
🤖 [回复内容气泡出现]
   根据分析结果...█
```

### 阶段 4：回复完成
```
🤖 [完整回复]
   根据分析结果，我建议...
   
   ▶ 💭 思考过程  ← 可点击查看
   ─────────────────
   📦 o1-preview | 🔢 150 tokens
```

## 技术实现

### 桌面版 (AgentChatStream.tsx)

#### 显示逻辑
```tsx
{isStreaming && (
  <div className="flex items-start space-x-3">
    <div className="flex-1 space-y-3">
      {/* 1. 思考过程 - 有内容就显示 */}
      {currentReasoning && (
        <div className="bg-gradient-to-r from-amber-50 to-orange-50">
          💭 思考过程
          {currentReasoning}█
        </div>
      )}
      
      {/* 2. 回复内容 - 只在思考过程结束后显示 */}
      {!currentReasoning && currentResponse && (
        <div className="bg-white">
          {currentResponse}█
        </div>
      )}
      
      {/* 3. 加载状态 - 既没有思考也没有回复时 */}
      {!currentReasoning && !currentResponse && (
        <div className="bg-gradient-to-r from-amber-50 to-orange-50">
          💭 正在思考...
        </div>
      )}
    </div>
  </div>
)}
```

#### 关键条件
- **思考过程显示**：`currentReasoning` 有值
- **回复内容显示**：`!currentReasoning && currentResponse` 有值
- **加载状态显示**：`!currentReasoning && !currentResponse`

### 移动版 (MobileChatPreview.tsx)

#### 数据流
```typescript
let fullContent = '';
let fullReasoning = '';

await chatService.chatStream(
  { agentId, message, sessionId },
  {
    onContent: (chunk) => {
      fullContent += chunk;
      // 内容累积，但不立即显示
      // 等思考过程结束后再显示
    },
    onReasoning: (chunk) => {
      fullReasoning += chunk;
      setCurrentReasoning(fullReasoning);  // 实时显示思考过程
    },
    onDone: (data) => {
      setCurrentReasoning('');  // 清空思考过程
      
      // 思考过程结束后，添加完整的消息
      const assistantMessage = {
        id: data.replyId,
        role: 'assistant',
        content: fullContent,        // 完整的回复内容
        reasoning: fullReasoning,    // 完整的思考过程
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, assistantMessage]);
    },
  }
);
```

#### 显示逻辑
```tsx
{/* 流式响应时的思考过程 */}
{isTyping && currentReasoning && (
  <div className="flex justify-start">
    <div className="bg-gradient-to-r from-amber-50 to-orange-50">
      💭 思考中
      {currentReasoning}█
    </div>
  </div>
)}

{/* 加载状态 */}
{isTyping && !currentReasoning && (
  <div className="flex justify-start">
    <div className="bg-white">
      <div className="flex gap-1">
        <span className="animate-bounce">●</span>
        <span className="animate-bounce">●</span>
        <span className="animate-bounce">●</span>
      </div>
    </div>
  </div>
)}

{/* 消息列表 - 只显示完成的消息 */}
{messages.map((message) => (
  <div key={message.id}>
    {message.content}
    {message.reasoning && (
      <button onClick={() => toggleReasoning(message.id)}>
        💭 思考过程
      </button>
    )}
  </div>
))}
```

## 状态转换

### 桌面版状态流

```
开始流式响应
    ↓
currentReasoning = ""
currentResponse = ""
    ↓
收到 reasoning_content
    ↓
currentReasoning = "正在分析..."  ← 显示思考过程
currentResponse = ""
    ↓
继续收到 reasoning_content
    ↓
currentReasoning = "正在分析...\n检索知识..."  ← 更新思考过程
currentResponse = ""
    ↓
思考过程结束，开始收到 content
    ↓
currentReasoning = ""  ← 清空（思考过程消失）
currentResponse = "根据..."  ← 显示回复内容
    ↓
继续收到 content
    ↓
currentReasoning = ""
currentResponse = "根据分析..."  ← 更新回复内容
    ↓
完成
    ↓
currentReasoning = ""  ← 清空
currentResponse = ""  ← 清空
消息添加到列表（包含 reasoning 和 content）
```

### 移动版状态流

```
开始流式响应
    ↓
currentReasoning = ""
fullContent = ""
fullReasoning = ""
    ↓
收到 reasoning_content
    ↓
fullReasoning += chunk
currentReasoning = fullReasoning  ← 显示思考过程
    ↓
收到 content
    ↓
fullContent += chunk  ← 累积但不显示
currentReasoning = fullReasoning  ← 继续显示思考过程
    ↓
完成
    ↓
currentReasoning = ""  ← 清空（思考过程消失）
添加消息到列表：
  - content: fullContent
  - reasoning: fullReasoning
    ↓
消息气泡出现（显示完整的回复内容）
```

## 视觉效果

### 阶段 1：思考中
```
┌─────────────────────────────┐
│ 💭 思考过程                 │
│ ─────────────────────────── │
│ 正在分析您的问题...         │
│ 检索相关知识库...           │
│ 整理回答思路...█            │
└─────────────────────────────┘

（没有回复内容气泡）
```

### 阶段 2：思考结束，显示回复
```
（思考过程卡片消失）

┌─────────────────────────────┐
│ 根据分析结果...█            │
└─────────────────────────────┘
```

### 阶段 3：回复完成
```
┌─────────────────────────────┐
│ 根据分析结果，我建议...     │
│                             │
│ ▶ 💭 思考过程               │
│ ─────────────────────────── │
│ 📦 o1-preview | 🔢 150 tokens│
└─────────────────────────────┘
```

## 优势

### 1. 清晰的信息层次
- 先看思考过程，了解 AI 在做什么
- 再看回复内容，获得最终答案
- 避免两者同时显示造成混乱

### 2. 更好的注意力引导
- 用户的注意力先集中在思考过程
- 思考结束后，自然转移到回复内容
- 不会被同时出现的两个气泡分散注意力

### 3. 更流畅的体验
- 思考过程有明确的开始和结束
- 回复内容的出现有明确的时机
- 整个流程更有节奏感

### 4. 更好的性能
- 回复内容在后台累积，不需要频繁更新 DOM
- 思考过程结束后一次性显示完整内容
- 减少不必要的渲染

## 边界情况

### 情况 1：没有思考过程
```
开始流式响应
    ↓
显示加载状态（💭 正在思考...）
    ↓
收到 content（没有 reasoning_content）
    ↓
立即显示回复内容
```

### 情况 2：只有思考过程，没有回复
```
开始流式响应
    ↓
显示思考过程
    ↓
完成（没有 content）
    ↓
添加消息到列表（只有 reasoning，没有 content）
```

### 情况 3：思考过程和回复同时到达
```
开始流式响应
    ↓
同时收到 reasoning_content 和 content
    ↓
优先显示思考过程
    ↓
回复内容在后台累积
    ↓
思考过程结束后显示回复内容
```

## 测试清单

- [ ] 思考过程进行时，不显示回复内容气泡
- [ ] 思考过程实时更新
- [ ] 思考过程结束后，回复内容气泡出现
- [ ] 回复内容实时更新
- [ ] 完成后，思考过程收起到按钮中
- [ ] 没有思考过程时，直接显示回复内容
- [ ] 只有思考过程时，正常显示
- [ ] 加载状态正确显示
- [ ] 移动端和桌面端都正常工作

## 相关文件

- `frontend/src/components/AgentChatStream.tsx` - 桌面版实现
- `frontend/src/components/common/MobileChatPreview.tsx` - 移动版实现
- `frontend/src/hooks/useAgentChatStream.ts` - 聊天 Hook
- `frontend/src/services/chat.service.ts` - 聊天服务

## 总结

新的设计确保了思考过程和回复内容的清晰分离：

1. **思考过程进行时**：只显示思考过程卡片，回复内容在后台累积
2. **思考过程结束后**：思考过程卡片消失，回复内容气泡出现
3. **回复完成后**：思考过程收起到按钮中，用户可以随时查看

这种设计提供了更清晰的信息层次和更流畅的用户体验。
