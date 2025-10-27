import { useTranslation } from 'react-i18next';
import { Card } from '../components/common/Card';
import { EmbeddedChat } from '../components/common';
import { useAuthStore } from '../stores/authStore';

/**
 * Dashboard 页面示例 - 带嵌入式聊天助手
 * 展示如何在任意页面中集成 EmbeddedChat 组件
 */
export const DashboardWithChatPage: React.FC = () => {
  const { t } = useTranslation();
  const user = useAuthStore((state) => state.user);

  // 处理聊天消息
  const handleSendMessage = async (message: string): Promise<string> => {
    // TODO: 调用真实的 API
    // 这里可以集成后端的聊天服务
    await new Promise((resolve) => setTimeout(resolve, 1000));
    
    // 模拟智能回复
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
      <div>
        <h1 className="text-3xl font-bold text-gray-900">{t('dashboard.title')}</h1>
        <p className="text-gray-600 mt-2">
          {t('dashboard.welcomeBack', { name: user?.nickname || user?.username })}
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">{t('dashboard.totalFlows')}</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">0</p>
          </div>
        </Card>

        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">{t('dashboard.activeExecutions')}</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">0</p>
          </div>
        </Card>

        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">{t('dashboard.mcpTools')}</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">0</p>
          </div>
        </Card>

        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">{t('dashboard.sessions')}</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">0</p>
          </div>
        </Card>
      </div>

      <Card title={t('dashboard.recentActivity')} subtitle={t('dashboard.latestActions')}>
        <div className="text-center py-8 text-gray-500">
          {t('dashboard.noRecentActivity')}
        </div>
      </Card>

      {/* 嵌入式聊天助手 */}
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
