import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { flowService } from '../services/flow.service';
import type { Flow } from '../types';
import { Button, Card, Table, Loader, Alert } from '../components/common';

export const FlowListPage = () => {
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
      setError(err.response?.data?.error || 'Failed to load flows');
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
      navigate(`/flows/${flowId}/executions/${result.executionId}`);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to execute flow');
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'text-green-600 bg-green-100';
      case 'draft':
        return 'text-yellow-600 bg-yellow-100';
      case 'archived':
        return 'text-gray-600 bg-gray-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  };

  const columns = [
    {
      key: 'name',
      label: 'Name',
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
      label: 'Status',
      render: (flow: Flow) => (
        <span
          className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${getStatusColor(
            flow.status
          )}`}
        >
          {flow.status}
        </span>
      ),
    },
    {
      key: 'current_version',
      label: 'Version',
      render: (flow: Flow) => <span className="text-gray-900">v{flow.current_version}</span>,
    },
    {
      key: 'updated_at',
      label: 'Last Updated',
      render: (flow: Flow) => (
        <span className="text-gray-500">
          {new Date(flow.updated_at).toLocaleDateString()}
        </span>
      ),
    },
    {
      key: 'actions',
      label: 'Actions',
      render: (flow: Flow) => (
        <div className="flex gap-2">
          <Button size="sm" variant="secondary" onClick={() => handleViewDetails(flow.id)}>
            View
          </Button>
          {flow.status === 'active' && (
            <Button size="sm" onClick={() => handleExecute(flow.id)}>
              Execute
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
        <h1 className="text-2xl font-semibold text-gray-900">Agent Flows</h1>
        <div className="flex gap-3">
          <Button variant="secondary" onClick={() => navigate('/flows/import')}>
            Import DSL
          </Button>
          <Button onClick={() => navigate('/flows/new')}>Create Flow</Button>
        </div>
      </div>

      {error && (
        <Alert variant="error" onClose={() => setError(null)}>
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
              className={`px-4 py-2 text-sm font-medium rounded-lg transition-colors ${
                filter === status
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              {status.charAt(0).toUpperCase() + status.slice(1)}
            </button>
          ))}
        </div>

        <Table data={flows} columns={columns} />

        {total > limit && (
          <div className="mt-4 flex items-center justify-between">
            <div className="text-sm text-gray-700">
              Showing {(page - 1) * limit + 1} to {Math.min(page * limit, total)} of {total}{' '}
              results
            </div>
            <div className="flex gap-2">
              <Button
                size="sm"
                variant="secondary"
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                disabled={page === 1}
              >
                Previous
              </Button>
              <Button
                size="sm"
                variant="secondary"
                onClick={() => setPage((p) => p + 1)}
                disabled={page * limit >= total}
              >
                Next
              </Button>
            </div>
          </div>
        )}
      </Card>
    </div>
  );
};
