import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { agentService } from '../services/agent.service';
import type { Agent } from '../types';
import { Loader, Alert, MobileChatPreview } from '../components/common';

/**
 * 独立的 Agent 聊天页面
 * 展示如何在独立页面中使用 MobileChatPreview 组件
 */
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
      setError(null);
      const data = await agentService.getAgent(id!);
      setAgent(data);
    } catch (err: any) {
      setError(err.response?.data?.error || '加载 Agent 失败');
    } finally {
      setLoading(false);
    }
  };

  const handleSendMessage = async (message: string): Promise<string> => {
    // TODO: 实现真实的消息发送逻辑
    // 这里可以调用后端 API 来处理消息
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
        <Alert type="error">
          {error || 'Agent 不存在'}
        </Alert>
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
