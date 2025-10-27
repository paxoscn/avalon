import { useTranslation } from 'react-i18next';
import { Card } from '../components/common/Card';
import { useAuthStore } from '../stores/authStore';

export const DashboardPage: React.FC = () => {
  const { t } = useTranslation();
  const user = useAuthStore((state) => state.user);

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
    </div>
  );
};
