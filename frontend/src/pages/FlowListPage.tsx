import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { flowService } from '../services/flow.service';
import type { Flow } from '../types';
import { Button, Card, Table, Loader, Alert } from '../components/common';

export const FlowListPage = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [flows, setFlows] = useState<Flow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<string>('all');
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const limit = 10;

  useEffect(() => {
    loadFlows();
  }, [page, filter]);

  const loadFlows = async () => {
    try {
      setLoading(true);
      setError(null);
      const params: any = { page, limit };
      if (filter !== 'all') {
        params.status = filter;
      }
      const response = await flowService.getFlows(params);
      setFlows(response.flows);
      setTotal(response.total);
    } catch (err: any) {
      setError(err.response?.data?.error || t('flows.errors.loadFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleViewDetails = (flowId: string) => {
    navigate(`/flows/${flowId}`);
  };

  const handleExecute = async (flowId: string) => {
    try {
      const result = await flowService.executeFlow(flowId, {});
      navigate(`/flows/${flowId}/executions/${result.id}`);
    } catch (err: any) {
      setError(err.response?.data?.error || t('flows.errors.executeFailed'));
    }
  };

  const handleToggleStatus = async (flowId: string, currentStatus: string) => {
    try {
      if (currentStatus === 'Active') {
        await flowService.deactivateFlow(flowId);
      } else {
        await flowService.activateFlow(flowId);
      }
      await loadFlows();
    } catch (err: any) {
      setError(err.response?.data?.error || t('flows.errors.updateStatusFailed'));
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Active':
        return 'text-green-600 bg-green-100';
      case 'Draft':
        return 'text-yellow-600 bg-yellow-100';
      case 'Archived':
        return 'text-gray-600 bg-gray-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  };

  const columns = [
    {
      key: 'name',
      header: t('flows.columns.name'),
      render: (flow: Flow) => (
        <div>
          <div className="font-medium text-gray-900">{flow.name}</div>
          {flow.description && (
            <div className="text-sm text-gray-500">{flow.description}</div>
          )}
        </div>
      ),
    },
    {
      key: 'status',
      header: t('flows.columns.status'),
      render: (flow: Flow) => (
        <span
          className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${getStatusColor(
            flow.status
          )}`}
        >
          {t(`flows.status.${flow.status.toLowerCase()}`)}
        </span>
      ),
    },
    {
      key: 'current_version',
      header: t('flows.columns.version'),
      render: (flow: Flow) => <span className="text-gray-900">v{flow.current_version}</span>,
    },
    {
      key: 'updated_at',
      header: t('flows.columns.lastUpdated'),
      render: (flow: Flow) => (
        <span className="text-gray-500">
          {new Date(flow.updated_at).toLocaleDateString()}
        </span>
      ),
    },
    {
      key: 'actions',
      header: t('common.actions'),
      render: (flow: Flow) => (
        <div className="flex gap-2">
          <Button size="sm" variant="secondary" onClick={() => handleViewDetails(flow.id)}>
            {t('flows.actions.view')}
          </Button>
          {flow.status !== 'Archived' && (
            <Button
              size="sm"
              variant={flow.status === 'Active' ? 'secondary' : 'primary'}
              onClick={() => handleToggleStatus(flow.id, flow.status)}
            >
              {flow.status === 'Active' ? t('flows.actions.deactivate') : t('flows.actions.activate')}
            </Button>
          )}
          {flow.status === 'Active' && (
            <Button size="sm" onClick={() => handleExecute(flow.id)}>
              {t('flows.actions.execute')}
            </Button>
          )}
        </div>
      ),
    },
  ];

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold text-gray-900">{t('flows.title')}</h1>
        <div className="flex gap-3">
          <Button variant="secondary" onClick={() => navigate('/flows/import')}>
            {t('flows.importDSL')}
          </Button>
          <Button onClick={() => navigate('/flows/new')}>{t('flows.createFlow')}</Button>
        </div>
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      <Card>
        <div className="mb-4 flex gap-2">
          {['all', 'active', 'draft', 'archived'].map((status) => (
            <button
              key={status}
              onClick={() => {
                setFilter(status);
                setPage(1);
              }}
              className={`px-4 py-2 text-sm font-medium rounded-lg transition-colors ${filter === status
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                }`}
            >
              {t(`flows.filters.${status}`)}
            </button>
          ))}
        </div>

        <Table data={flows} columns={columns} keyExtractor={(item) => { return item.id }} />

        {total > limit && (
          <div className="mt-4 flex items-center justify-between">
            <div className="text-sm text-gray-700">
              {t('flows.pagination.showing', {
                from: (page - 1) * limit + 1,
                to: Math.min(page * limit, total),
                total: total
              })}
            </div>
            <div className="flex gap-2">
              <Button
                size="sm"
                variant="secondary"
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                disabled={page === 1}
              >
                {t('common.previous')}
              </Button>
              <Button
                size="sm"
                variant="secondary"
                onClick={() => setPage((p) => p + 1)}
                disabled={page * limit >= total}
              >
                {t('common.next')}
              </Button>
            </div>
          </div>
        )}
      </Card>
    </div>
  );
};
