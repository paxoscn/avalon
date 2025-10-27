# 聊天组件集成示例

本文档提供了在不同场景下集成聊天组件的完整示例。

## 目录

1. [Agent 编辑页面实时预览](#1-agent-编辑页面实时预览)
2. [独立聊天页面](#2-独立聊天页面)
3. [Dashboard 嵌入式助手](#3-dashboard-嵌入式助手)
4. [Agent 列表添加聊天入口](#4-agent-列表添加聊天入口)
5. [多 Agent 切换聊天](#5-多-agent-切换聊天)

---

## 1. Agent 编辑页面实时预览

**场景**：在编辑 Agent 配置时，右侧实时显示手机端聊天界面预览。

**已实现文件**：`frontend/src/pages/AgentDetailPage.tsx`

### 关键代码

```tsx
import { MobileChatPreview } from '../components/common';

export function AgentDetailPage() {
  const [formData, setFormData] = useState({
    name: '',
    avatar: '',
    systemPrompt: '',
    presetQuestions: ['', '', ''],
    // ...
  });

  return (
    <div className="flex gap-6">
      {/* 左侧编辑表单 */}
      <div className="flex-1 space-y-6">
        <form onSubmit={handleSubmit}>
          {/* 表单内容 */}
        </form>
      </div>

      {/* 右侧手机预览 */}
      <div className="w-96 sticky top-6 self-start">
        <div className="mb-3 text-center">
          <h3 className="text-sm font-medium text-gray-700">实时预览</h3>
          <p className="text-xs text-gray-500">查看手机端聊天界面效果</p>
        </div>
        <MobileChatPreview
          agentName={formData.name || 'AI 助手'}
          agentAvatar={formData.avatar}
          systemPrompt={formData.systemPrompt}
          presetQuestions={formData.presetQuestions}
        />
      </div>
    </div>
  );
}
```

### 特点

- ✅ 实时同步：编辑表单时，预览立即更新
- ✅ 粘性定位：预览区域固定在视口中
- ✅ 模拟模式：无需后端即可预览效果

---

## 2. 独立聊天页面

**场景**：为每个 Agent 创建专门的聊天页面，用户可以全屏与 Agent 对话。

**文件**：`frontend/src/pages/AgentChatPage.tsx`

### 完整代码

```tsx
import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { agentService } from '../services/agent.service';
import type { Agent } from '../types';
import { Loader, Alert, MobileChatPreview } from '../components/common';

export function AgentChatPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [agent, setAgent] = useState<Agent | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (id) {
      loadAgent();
    }
  }, [id]);

  const loadAgent = async () => {
    try {
      setLoading(true);
      const data = await agentService.getAgent(id!);
      setAgent(data);
    } catch (err: any) {
      setError(err.response?.data?.error || '加载 Agent 失败');
    } finally {
      setLoading(false);
    }
  };

  const handleSendMessage = async (message: string): Promise<string> => {
    // TODO: 调用真实的聊天 API
    // const response = await chatService.sendMessage(id!, message);
    // return response.content;
    
    await new Promise((resolve) => setTimeout(resolve, 1000));
    return `收到您的消息："${message}"。这是一个模拟回复。`;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader size="lg" />
      </div>
    );
  }

  if (error || !agent) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Alert type="error">{error || 'Agent 不存在'}</Alert>
      </div>
    );
  }

  return (
    <div className="h-screen bg-gray-100 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="mb-4 flex items-center justify-between">
          <button
            onClick={() => navigate('/agents')}
            className="text-sm text-gray-600 hover:text-gray-900 flex items-center gap-1"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
            返回列表
          </button>
          <h2 className="text-lg font-semibold text-gray-900">与 {agent.name} 对话</h2>
        </div>
        
        <MobileChatPreview
          agentName={agent.name}
          agentAvatar={agent.avatar}
          systemPrompt={agent.system_prompt}
          presetQuestions={agent.preset_questions}
          onSendMessage={handleSendMessage}
          className="h-[700px]"
        />
      </div>
    </div>
  );
}
```

### 路由配置

在 `App.tsx` 或路由配置文件中添加：

```tsx
import { AgentChatPage } from './pages/AgentChatPage';

<Route path="/agents/:id/chat" element={<AgentChatPage />} />
```

### 特点

- ✅ 全屏体验：专注的聊天界面
- ✅ 真实 API：支持接入后端聊天服务
- ✅ 导航友好：可返回 Agent 列表

---

## 3. Dashboard 嵌入式助手

**场景**：在 Dashboard 页面添加浮动聊天按钮，用户可随时唤起智能助手。

**文件**：`frontend/src/pages/DashboardWithChatPage.tsx`

### 完整代码

```tsx
import { useTranslation } from 'react-i18next';
import { Card } from '../components/common/Card';
import { EmbeddedChat } from '../components/common';
import { useAuthStore } from '../stores/authStore';

export const DashboardWithChatPage: React.FC = () => {
  const { t } = useTranslation();
  const user = useAuthStore((state) => state.user);

  const handleSendMessage = async (message: string): Promise<string> => {
    await new Promise((resolve) => setTimeout(resolve, 1000));
    
    // 智能回复逻辑
    if (message.includes('流程') || message.includes('flow')) {
      return '您可以在左侧菜单中点击"Flows"来查看和管理所有工作流程。需要创建新流程吗？';
    } else if (message.includes('工具') || message.includes('tool')) {
      return '在"MCP Tools"页面，您可以配置和测试各种工具。目前支持 HTTP、数据库等多种工具类型。';
    } else if (message.includes('帮助') || message.includes('help')) {
      return '我可以帮您：\n1. 创建和管理工作流\n2. 配置 MCP 工具\n3. 查看执行历史\n4. 管理 Agent\n\n请告诉我您需要什么帮助？';
    } else {
      return `收到您的消息："${message}"。我是您的智能助手，可以帮您快速了解和使用系统功能。`;
    }
  };

  return (
    <div className="space-y-6">
      {/* Dashboard 内容 */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900">{t('dashboard.title')}</h1>
        <p className="text-gray-600 mt-2">
          {t('dashboard.welcomeBack', { name: user?.nickname || user?.username })}
        </p>
      </div>

      {/* 统计卡片 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">{t('dashboard.totalFlows')}</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">0</p>
          </div>
        </Card>
        {/* 更多卡片... */}
      </div>

      {/* 嵌入式聊天助手 - 浮动在右下角 */}
      <EmbeddedChat
        agentId="dashboard-assistant"
        agentName="智能助手"
        systemPrompt="我是您的智能助手，可以帮您快速了解和使用系统功能。"
        presetQuestions={[
          '如何创建工作流？',
          '如何配置 MCP 工具？',
          '如何查看执行历史？',
        ]}
        onSendMessage={handleSendMessage}
        position="bottom-right"
      />
    </div>
  );
};
```

### 特点

- ✅ 非侵入式：浮动按钮不影响页面布局
- ✅ 可最小化：用户可以随时收起/展开
- ✅ 智能回复：根据关键词提供相关帮助
- ✅ 位置可配：支持四个角落定位

---

## 4. Agent 列表添加聊天入口

**场景**：在 Agent 列表页面，为每个 Agent 添加"开始对话"按钮。

### 修改 AgentListPage.tsx

```tsx
// 在 action buttons 部分添加聊天按钮
{activeTab === 'employed' && (
  <div className="flex items-center gap-2 pt-4 border-t border-gray-200">
    <Link to={`/agents/${agent.id}/chat`} className="flex-1">
      <Button variant="primary" className="w-full">
        开始对话
      </Button>
    </Link>
    <Button
      variant="secondary"
      onClick={() => handleTune(agent.id)}
      className="flex-1"
    >
      调整设置
    </Button>
  </div>
)}
```

### 特点

- ✅ 快速访问：直接从列表进入聊天
- ✅ 上下文保持：知道是哪个 Agent
- ✅ 用户友好：清晰的操作入口

---

## 5. 多 Agent 切换聊天

**场景**：在一个页面中支持与多个 Agent 对话，可以切换。

### 实现代码

```tsx
import { useState, useEffect } from 'react';
import { agentService } from '../services/agent.service';
import type { Agent } from '../types';
import { MobileChatPreview, Loader } from '../components/common';

export function MultiAgentChatPage() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadAgents();
  }, []);

  const loadAgents = async () => {
    try {
      const response = await agentService.listEmployedAgents();
      setAgents(response.items);
      if (response.items.length > 0) {
        setSelectedAgent(response.items[0]);
      }
    } catch (err) {
      console.error('Failed to load agents:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSendMessage = async (message: string): Promise<string> => {
    if (!selectedAgent) return '请先选择一个 Agent';
    
    // TODO: 调用真实 API
    await new Promise((resolve) => setTimeout(resolve, 1000));
    return `${selectedAgent.name} 回复：收到您的消息 "${message}"`;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader size="lg" />
      </div>
    );
  }

  return (
    <div className="h-screen bg-gray-100 flex">
      {/* 左侧 Agent 列表 */}
      <div className="w-64 bg-white border-r border-gray-200 overflow-y-auto">
        <div className="p-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">我的 Agents</h2>
          <p className="text-xs text-gray-500 mt-1">选择一个开始对话</p>
        </div>
        <div className="p-2 space-y-2">
          {agents.map((agent) => (
            <button
              key={agent.id}
              onClick={() => setSelectedAgent(agent)}
              className={`w-full p-3 rounded-lg text-left transition-colors ${
                selectedAgent?.id === agent.id
                  ? 'bg-blue-50 border-2 border-blue-500'
                  : 'bg-gray-50 border-2 border-transparent hover:bg-gray-100'
              }`}
            >
              <div className="flex items-center gap-3">
                {agent.avatar ? (
                  <img
                    src={agent.avatar}
                    alt={agent.name}
                    className="w-10 h-10 rounded-full object-cover"
                  />
                ) : (
                  <div className="w-10 h-10 rounded-full bg-gradient-to-br from-blue-400 to-purple-500 flex items-center justify-center text-white font-bold">
                    {agent.name.charAt(0).toUpperCase()}
                  </div>
                )}
                <div className="flex-1 min-w-0">
                  <h3 className="text-sm font-medium text-gray-900 truncate">
                    {agent.name}
                  </h3>
                  <p className="text-xs text-gray-500 truncate">
                    {agent.system_prompt}
                  </p>
                </div>
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* 右侧聊天界面 */}
      <div className="flex-1 flex items-center justify-center p-4">
        {selectedAgent ? (
          <div className="w-full max-w-md">
            <MobileChatPreview
              agentName={selectedAgent.name}
              agentAvatar={selectedAgent.avatar}
              systemPrompt={selectedAgent.system_prompt}
              presetQuestions={selectedAgent.preset_questions}
              onSendMessage={handleSendMessage}
              className="h-[700px]"
            />
          </div>
        ) : (
          <div className="text-center text-gray-500">
            <p>请从左侧选择一个 Agent 开始对话</p>
          </div>
        )}
      </div>
    </div>
  );
}
```

### 路由配置

```tsx
<Route path="/chat" element={<MultiAgentChatPage />} />
```

### 特点

- ✅ 多 Agent 支持：在一个页面管理多个对话
- ✅ 快速切换：点击即可切换 Agent
- ✅ 上下文独立：每个 Agent 的对话独立保存

---

## 后端 API 集成

### 创建聊天服务

```typescript
// frontend/src/services/chat.service.ts
import { apiClient } from './api';

export interface SendMessageRequest {
  agent_id: string;
  session_id?: string;
  message: string;
}

export interface SendMessageResponse {
  message_id: string;
  content: string;
  session_id: string;
  created_at: string;
}

class ChatService {
  async sendMessage(request: SendMessageRequest): Promise<SendMessageResponse> {
    const response = await apiClient.post<SendMessageResponse>(
      `/agents/${request.agent_id}/chat`,
      {
        session_id: request.session_id,
        message: request.message,
      }
    );
    return response.data;
  }

  async createSession(agentId: string): Promise<{ session_id: string }> {
    const response = await apiClient.post<{ session_id: string }>(
      `/agents/${agentId}/sessions`
    );
    return response.data;
  }

  async getMessages(agentId: string, sessionId: string) {
    const response = await apiClient.get(
      `/agents/${agentId}/sessions/${sessionId}/messages`
    );
    return response.data;
  }
}

export const chatService = new ChatService();
```

### 在组件中使用

```tsx
import { chatService } from '../services/chat.service';

const [sessionId, setSessionId] = useState<string | null>(null);

// 初始化会话
useEffect(() => {
  if (agentId) {
    chatService.createSession(agentId).then((res) => {
      setSessionId(res.session_id);
    });
  }
}, [agentId]);

// 发送消息
const handleSendMessage = async (message: string): Promise<string> => {
  if (!sessionId) {
    throw new Error('Session not initialized');
  }

  const response = await chatService.sendMessage({
    agent_id: agentId,
    session_id: sessionId,
    message: message,
  });

  return response.content;
};
```

---

## 最佳实践

### 1. 会话管理

```tsx
// 使用 localStorage 保存会话 ID
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

```tsx
const handleSendMessage = async (message: string): Promise<string> => {
  try {
    const response = await chatService.sendMessage({
      agent_id: agentId,
      session_id: sessionId,
      message: message,
    });
    return response.content;
  } catch (error: any) {
    console.error('Failed to send message:', error);
    
    // 返回友好的错误消息
    if (error.response?.status === 429) {
      return '抱歉，请求过于频繁，请稍后再试。';
    } else if (error.response?.status === 500) {
      return '服务器错误，请稍后再试。';
    } else {
      return '发送消息失败，请检查网络连接。';
    }
  }
};
```

### 3. 加载历史消息

```tsx
useEffect(() => {
  if (sessionId && agentId) {
    loadHistoryMessages();
  }
}, [sessionId, agentId]);

const loadHistoryMessages = async () => {
  try {
    const messages = await chatService.getMessages(agentId, sessionId);
    // 将历史消息设置到组件状态
    setMessages(messages);
  } catch (error) {
    console.error('Failed to load history:', error);
  }
};
```

### 4. 实时通知（WebSocket）

```tsx
useEffect(() => {
  if (!sessionId) return;

  const ws = new WebSocket(`ws://localhost:8080/ws/chat/${sessionId}`);
  
  ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    // 添加新消息到列表
    setMessages((prev) => [...prev, message]);
  };

  return () => ws.close();
}, [sessionId]);
```

---

## 总结

本文档提供了 5 种常见的聊天组件集成场景：

1. ✅ **实时预览** - Agent 编辑页面
2. ✅ **独立页面** - 专门的聊天界面
3. ✅ **嵌入式助手** - Dashboard 浮动助手
4. ✅ **列表入口** - 从 Agent 列表快速进入
5. ✅ **多 Agent 切换** - 管理多个对话

所有示例都提供了完整的代码，可以直接使用或根据需求定制。
