import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { agentService } from '../services/agent.service';
import type { AgentUsageStats, Agent } from '../types';
import { Button, Card, Loader, Alert } from '../components/common';

export function AgentStatsPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [agent, setAgent] = useState<Agent | null>(null);
  const [stats, setStats] = useState<AgentUsageStats[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [summary, setSummary] = useState<any>(null);

  // Date range state
  const [startDate, setStartDate] = useState(() => {
    const date = new Date();
    date.setDate(date.getDate() - 30);
    return date.toISOString().split('T')[0];
  });
  const [endDate, setEndDate] = useState(() => {
    return new Date().toISOString().split('T')[0];
  });

  useEffect(() => {
    if (id) {
      loadAgent();
    }
  }, [id]);

  useEffect(() => {
    if (id) {
      loadStats();
    }
  }, [id, page, startDate, endDate]);

  const loadAgent = async () => {
    if (!id) return;
    try {
      const data = await agentService.getAgent(id);
      setAgent(data);
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.loadAgentFailed'));
    }
  };

  const loadStats = async () => {
    if (!id) return;
    try {
      setLoading(true);
      setError(null);
      const response = await agentService.getAgentUsageStats(id, {
        start_date: startDate,
        end_date: endDate,
        page,
        page_size: 20,
      });
      setStats(response.items);
      setTotalPages(response.total_pages);
      setSummary(response.summary);
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.stats.loadFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = () => {
    setPage(1);
    loadStats();
  };

  if (!agent) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader size="lg" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <div className="flex items-center gap-2">
            <Button variant="secondary" onClick={() => navigate('/agents')}>
              {t('agents.stats.backToList')}
            </Button>
          </div>
          <h1 className="text-3xl font-semibold text-gray-900 mt-4">
            {t('agents.stats.title')}
          </h1>
          <p className="mt-2 text-sm text-gray-600">
            {agent.name} - {t('agents.stats.description')}
          </p>
        </div>
      </div>

      {/* Date Range Filter */}
      <Card>
        <div className="space-y-4">
          <h3 className="text-lg font-medium text-gray-900">
            {t('agents.stats.dateRange')}
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {t('agents.stats.startDate')}
              </label>
              <input
                type="date"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {t('agents.stats.endDate')}
              </label>
              <input
                type="date"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div className="flex items-end">
              <Button onClick={handleSearch} className="w-full">
                {t('agents.stats.search')}
              </Button>
            </div>
          </div>
        </div>
      </Card>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {/* Summary Cards */}
      {summary && (
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
          <Card>
            <div className="space-y-2">
              <p className="text-sm text-gray-600">{t('agents.stats.totalSessions')}</p>
              <p className="text-3xl font-bold text-gray-900">
                {summary.total_sessions.toLocaleString()}
              </p>
            </div>
          </Card>
          <Card>
            <div className="space-y-2">
              <p className="text-sm text-gray-600">{t('agents.stats.totalMessages')}</p>
              <p className="text-3xl font-bold text-gray-900">
                {summary.total_messages.toLocaleString()}
              </p>
            </div>
          </Card>
          <Card>
            <div className="space-y-2">
              <p className="text-sm text-gray-600">{t('agents.stats.totalTokens')}</p>
              <p className="text-3xl font-bold text-gray-900">
                {summary.total_tokens.toLocaleString()}
              </p>
            </div>
          </Card>
          <Card>
            <div className="space-y-2">
              <p className="text-sm text-gray-600">{t('agents.stats.uniqueUsers')}</p>
              <p className="text-3xl font-bold text-gray-900">
                {summary.unique_users.toLocaleString()}
              </p>
            </div>
          </Card>
        </div>
      )}

      {/* Stats Table */}
      <Card>
        <div className="space-y-4">
          <h3 className="text-lg font-medium text-gray-900">
            {t('agents.stats.dailyStats')}
          </h3>
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader size="lg" />
            </div>
          ) : stats.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-gray-500">{t('agents.stats.noData')}</p>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      {t('agents.stats.date')}
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      {t('agents.stats.sessions')}
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      {t('agents.stats.messages')}
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      {t('agents.stats.tokens')}
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      {t('agents.stats.users')}
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      {t('agents.stats.avgDuration')}
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {stats.map((stat, index) => (
                    <tr key={index} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {new Date(stat.date).toLocaleDateString()}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {stat.total_sessions.toLocaleString()}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {stat.total_messages.toLocaleString()}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {stat.total_tokens.toLocaleString()}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {stat.unique_users.toLocaleString()}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {stat.avg_session_duration_seconds
                          ? `${Math.round(stat.avg_session_duration_seconds)}s`
                          : '-'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </Card>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-center gap-2">
          <Button
            variant="secondary"
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page === 1}
          >
            {t('common.previous')}
          </Button>
          <span className="text-sm text-gray-600">
            {t('common.page')} {page} {t('common.of')} {totalPages}
          </span>
          <Button
            variant="secondary"
            onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
            disabled={page === totalPages}
          >
            {t('common.next')}
          </Button>
        </div>
      )}
    </div>
  );
}
