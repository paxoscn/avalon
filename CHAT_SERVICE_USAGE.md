# Chat Service 使用指南

## 概述

`chat.service.ts` 现在完全支持思考过程（reasoning_content）的流式传输。

## 更新内容

### 1. ChatStreamChunk 接口

添加了 `reasoning_content` 字段：

```typescript
export interface ChatStreamChunk {
  type: 'content' | 'done' | 'error';
  content?: string;
  reasoning_content?: string;  // 新增：思考过程
  session_id?: string;
  message_id?: string;
  reply_id?: string;
  metadata?: Record<string, any>;
  finish_reason?: string;
  error?: string;
}
```

### 2. ChatStreamCallbacks 接口

添加了 `onReasoning` 回调：

```typescript
export interface ChatStreamCallbacks {
  onContent?: (content: string) => void;
  onReasoning?: (reasoning: string) => void;  // 新增：思考过程回调
  onDone?: (data: { ... }) => void;
  onError?: (error: string) => void;
}
```

### 3. chatStream 方法

更新了流处理逻辑，支持 `reasoning_content`：

```typescript
if (data.type === 'content') {
  if (data.content) {
    callbacks.onContent?.(data.content);
  }
  if (data.reasoning_content) {
    callbacks.onReasoning?.(data.reasoning_content);  // 新增
  }
}
```

## 使用示例

### 基础用法

```typescript
import { chatService } from './services/chat.service';

// 发送消息并接收流式响应
await chatService.chatStream(
  {
    agentId: 'agent-123',
    message: '请帮我分析一下这个问题',
    sessionId: 'session-456', // 可选
  },
  {
    onContent: (content) => {
      console.log('收到内容:', content);
      // 更新 UI 显示回复内容
    },
    onReasoning: (reasoning) => {
      console.log('收到思考过程:', reasoning);
      // 更新 UI 显示思考过程
    },
    onDone: (data) => {
      console.log('完成:', data);
      // 保存消息 ID，更新 UI 状态
    },
    onError: (error) => {
      console.error('错误:', error);
      // 显示错误信息
    },
  }
);
```

### React 组件中使用

```typescript
import React, { useState } from 'react';
import { chatService } from '../services/chat.service';

function ChatComponent({ agentId }: { agentId: string }) {
  const [content, setContent] = useState('');
  const [reasoning, setReasoning] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);

  const sendMessage = async (message: string) => {
    setIsStreaming(true);
    setContent('');
    setReasoning('');

    try {
      await chatService.chatStream(
        { agentId, message },
        {
          onContent: (chunk) => {
            setContent((prev) => prev + chunk);
          },
          onReasoning: (chunk) => {
            setReasoning((prev) => prev + chunk);
          },
          onDone: (data) => {
            console.log('消息完成:', data);
            setIsStreaming(false);
          },
          onError: (error) => {
            console.error('错误:', error);
            setIsStreaming(false);
          },
        }
      );
    } catch (error) {
      console.error('发送失败:', error);
      setIsStreaming(false);
    }
  };

  return (
    <div>
      {/* 思考过程 */}
      {reasoning && (
        <div className="thinking-box">
          <h4>💭 思考过程</h4>
          <p>{reasoning}</p>
        </div>
      )}

      {/* 回复内容 */}
      {content && (
        <div className="content-box">
          <p>{content}</p>
        </div>
      )}

      {/* 输入框 */}
      <input
        type="text"
        disabled={isStreaming}
        onKeyPress={(e) => {
          if (e.key === 'Enter') {
            sendMessage(e.currentTarget.value);
            e.currentTarget.value = '';
          }
        }}
      />
    </div>
  );
}
```

### 累积内容的完整示例

```typescript
import { chatService } from '../services/chat.service';

async function chatWithAccumulation(agentId: string, message: string) {
  let accumulatedContent = '';
  let accumulatedReasoning = '';

  await chatService.chatStream(
    { agentId, message },
    {
      onContent: (chunk) => {
        accumulatedContent += chunk;
        console.log('当前内容:', accumulatedContent);
      },
      onReasoning: (chunk) => {
        accumulatedReasoning += chunk;
        console.log('当前思考:', accumulatedReasoning);
      },
      onDone: (data) => {
        console.log('最终内容:', accumulatedContent);
        console.log('最终思考:', accumulatedReasoning);
        console.log('元数据:', data.metadata);
      },
      onError: (error) => {
        console.error('错误:', error);
      },
    }
  );

  return {
    content: accumulatedContent,
    reasoning: accumulatedReasoning,
  };
}
```

### 与 useAgentChatStream Hook 对比

#### chat.service.ts (底层服务)
```typescript
// 更底层，更灵活
await chatService.chatStream(
  { agentId, message },
  {
    onContent: (chunk) => { /* 处理内容 */ },
    onReasoning: (chunk) => { /* 处理思考 */ },
    onDone: (data) => { /* 完成 */ },
    onError: (error) => { /* 错误 */ },
  }
);
```

#### useAgentChatStream (React Hook)
```typescript
// 更高层，更易用，自动管理状态
const {
  messages,
  currentResponse,
  currentReasoning,
  isStreaming,
  sendMessage,
} = useAgentChatStream({ agentId });

// 直接使用状态
console.log(currentResponse);
console.log(currentReasoning);
```

## 数据流

```
用户发送消息
    ↓
chatService.chatStream()
    ↓
SSE 流开始
    ↓
收到 reasoning_content → onReasoning() 回调
    ↓
收到 content → onContent() 回调
    ↓
收到 done → onDone() 回调
    ↓
完成
```

## 错误处理

```typescript
try {
  await chatService.chatStream(
    { agentId, message },
    {
      onContent: (chunk) => {
        // 处理内容
      },
      onReasoning: (chunk) => {
        // 处理思考
      },
      onDone: (data) => {
        // 完成
      },
      onError: (error) => {
        // SSE 流中的错误
        console.error('流错误:', error);
      },
    }
  );
} catch (error) {
  // 网络错误或其他异常
  console.error('请求失败:', error);
}
```

## 最佳实践

### 1. 使用累加器

```typescript
let content = '';
let reasoning = '';

await chatService.chatStream(
  { agentId, message },
  {
    onContent: (chunk) => {
      content += chunk;  // 累加内容
      updateUI(content);
    },
    onReasoning: (chunk) => {
      reasoning += chunk;  // 累加思考
      updateThinkingUI(reasoning);
    },
  }
);
```

### 2. 分离关注点

```typescript
// 内容处理器
const handleContent = (chunk: string) => {
  setContent((prev) => prev + chunk);
};

// 思考处理器
const handleReasoning = (chunk: string) => {
  setReasoning((prev) => prev + chunk);
};

// 使用
await chatService.chatStream(
  { agentId, message },
  {
    onContent: handleContent,
    onReasoning: handleReasoning,
    onDone: handleDone,
    onError: handleError,
  }
);
```

### 3. 状态管理

```typescript
const [state, setState] = useState({
  content: '',
  reasoning: '',
  isStreaming: false,
  error: null,
});

await chatService.chatStream(
  { agentId, message },
  {
    onContent: (chunk) => {
      setState((prev) => ({
        ...prev,
        content: prev.content + chunk,
      }));
    },
    onReasoning: (chunk) => {
      setState((prev) => ({
        ...prev,
        reasoning: prev.reasoning + chunk,
      }));
    },
    onDone: () => {
      setState((prev) => ({
        ...prev,
        isStreaming: false,
      }));
    },
    onError: (error) => {
      setState((prev) => ({
        ...prev,
        error,
        isStreaming: false,
      }));
    },
  }
);
```

## 类型安全

所有接口都是完全类型化的：

```typescript
// ✅ 类型安全
const callbacks: ChatStreamCallbacks = {
  onContent: (content: string) => { /* ... */ },
  onReasoning: (reasoning: string) => { /* ... */ },
  onDone: (data) => {
    // data 的类型是自动推断的
    console.log(data.sessionId);
    console.log(data.metadata);
  },
};

// ❌ 类型错误
const badCallbacks: ChatStreamCallbacks = {
  onContent: (content: number) => { /* 错误：应该是 string */ },
};
```

## 总结

`chat.service.ts` 现在完全支持思考过程的流式传输：

- ✅ 添加了 `reasoning_content` 字段
- ✅ 添加了 `onReasoning` 回调
- ✅ 更新了流处理逻辑
- ✅ 完全类型安全
- ✅ 向后兼容（onReasoning 是可选的）

如果你不需要思考过程，可以不提供 `onReasoning` 回调，服务仍然正常工作。
