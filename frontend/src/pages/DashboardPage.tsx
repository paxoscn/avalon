import { useTranslation } from 'react-i18next';
import { useEffect, useState } from 'react';
import { Card } from '../components/common/Card';
import { useAuthStore } from '../stores/authStore';
import { dashboardService, type DashboardStats } from '../services/dashboard.service';

export const DashboardPage: React.FC = () => {
  const { t } = useTranslation();
  const user = useAuthStore((state) => state.user);
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        setLoading(true);
        const data = await dashboardService.getStats();
        setStats(data);
        setError(null);
      } catch (err) {
        console.error('Failed to fetch dashboard stats:', err);
        setError('Failed to load statistics');
      } finally {
        setLoading(false);
      }
    };

    fetchStats();
  }, []);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-gray-900">{t('dashboard.title')}</h1>
        <p className="text-gray-600 mt-2">
          {t('dashboard.welcomeBack', { name: user?.nickname || user?.username })}
        </p>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-6">
        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">{t('dashboard.agents')}</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">
              {loading ? '...' : stats?.agents_count ?? 0}
            </p>
          </div>
        </Card>

        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">{t('dashboard.totalFlows')}</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">
              {loading ? '...' : stats?.flows_count ?? 0}
            </p>
          </div>
        </Card>

        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">{t('dashboard.mcpTools')}</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">
              {loading ? '...' : stats?.mcp_tools_count ?? 0}
            </p>
          </div>
        </Card>

        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">{t('dashboard.knowledgeBases')}</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">
              {loading ? '...' : stats?.knowledge_bases_count ?? 0}
            </p>
          </div>
        </Card>

        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">{t('dashboard.sessions')}</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">
              {loading ? '...' : stats?.sessions_count ?? 0}
            </p>
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
