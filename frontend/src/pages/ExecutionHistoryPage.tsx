import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { executionService, type ExecutionFilters, type ExecutionStats } from '../services/execution.service';
import type { FlowExecution } from '../types';
import { Card, Button, Input, Loader, Alert } from '../components/common';

export function ExecutionHistoryPage() {
  const navigate = useNavigate();
  const [executions, setExecutions] = useState<FlowExecution[]>([]);
  const [stats, setStats] = useState<ExecutionStats | null>(null);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  const [filters, setFilters] = useState<ExecutionFilters>({
    page: 1,
    limit: 20,
  });

  const [selectedStatus, setSelectedStatus] = useState('');
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');

  useEffect(() => {
    loadExecutions();
    loadStats();
  }, [filters]);

  const loadExecutions = async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await executionService.getExecutions(filters);
      setExecutions(response.executions);
      setTotal(response.total);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load execution history');
    } finally {
      setLoading(false);
    }
  };

  const loadStats = async () => {
    try {
      const statsData = await executionService.getExecutionStats({
        startDate: filters.startDate,
        endDate: filters.endDate,
      });
      setStats(statsData);
    } catch (err: any) {
      console.error('Failed to load stats:', err);
    }
  };

  const handleSearch = () => {
    setFilters({
      ...filters,
      status: selectedStatus || undefined,
      startDate: startDate || undefined,
      endDate: endDate || undefined,
      page: 1,
    });
  };

  const handleReset = () => {
    setSelectedStatus('');
    setStartDate('');
    setEndDate('');
    setFilters({ page: 1, limit: 20 });
  };

  const handlePageChange = (newPage: number) => {
    setFilters({ ...filters, page: newPage });
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  const formatDuration = (ms?: number) => {
    if (!ms) return '-';
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'bg-green-100 text-green-800';
      case 'running':
        return 'bg-blue-100 text-blue-800';
      case 'failed':
        return 'bg-red-100 text-red-800';
      case 'cancelled':
        return 'bg-gray-100 text-gray-800';
      default:
        return 'bg-yellow-100 text-yellow-800';
    }
  };

  const totalPages = Math.ceil(total / (filters.limit || 20));

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-semibold text-gray-900">Execution History</h1>
      </div>

      {error && <Alert type="error" onClose={() => setError(null)}>{error}</Alert>}

      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <Card>
            <div className="p-6">
              <p className="text-sm font-medium text-gray-500">Total Executions</p>
              <p className="text-3xl font-semibold text-gray-900 mt-2">
                {stats.totalExecutions}
              </p>
            </div>
          </Card>
          <Card>
            <div className="p-6">
              <p className="text-sm font-medium text-gray-500">Success Rate</p>
              <p className="text-3xl font-semibold text-green-600 mt-2">
                {stats.successRate.toFixed(1)}%
              </p>
            </div>
          </Card>
          <Card>
            <div className="p-6">
              <p className="text-sm font-medium text-gray-500">Avg Execution Time</p>
              <p className="text-3xl font-semibold text-blue-600 mt-2">
                {formatDuration(stats.averageExecutionTime)}
              </p>
            </div>
          </Card>
          <Card>
            <div className="p-6">
              <p className="text-sm font-medium text-gray-500">Failed</p>
              <p className="text-3xl font-semibold text-red-600 mt-2">
                {stats.executionsByStatus.failed || 0}
              </p>
            </div>
          </Card>
        </div>
      )}

      <Card>
        <div className="p-6 space-y-4">
          <h2 className="text-lg font-medium text-gray-900">Filters</h2>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Status
              </label>
              <select
                value={selectedStatus}
                onChange={(e) => setSelectedStatus(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              >
                <option value="">All Status</option>
                <option value="pending">Pending</option>
                <option value="running">Running</option>
                <option value="completed">Completed</option>
                <option value="failed">Failed</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Start Date
              </label>
              <Input
                type="datetime-local"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                End Date
              </label>
              <Input
                type="datetime-local"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
              />
            </div>
          </div>

          <div className="flex gap-2">
            <Button onClick={handleSearch}>Apply Filters</Button>
            <Button variant="secondary" onClick={handleReset}>Reset</Button>
          </div>
        </div>
      </Card>

      <Card>
        {loading ? (
          <div className="flex justify-center items-center py-12">
            <Loader />
          </div>
        ) : executions.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-gray-500">No execution history found</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Started At
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Flow
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Duration
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    User
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {executions.map((execution) => (
                  <tr key={execution.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {formatDate(execution.started_at)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      <div>
                        <div className="font-medium">{execution.flowId}</div>
                        <div className="text-xs text-gray-500">v{execution.flow_version}</div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(execution.status)}`}>
                        {execution.status}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {formatDuration(execution.execution_time_ms)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {execution.userId}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm">
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => navigate(`/executions/${execution.id}`)}
                      >
                        View Details
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      {!loading && totalPages > 1 && (
        <div className="flex justify-between items-center">
          <p className="text-sm text-gray-700">
            Showing {((filters.page || 1) - 1) * (filters.limit || 20) + 1} to{' '}
            {Math.min((filters.page || 1) * (filters.limit || 20), total)} of {total} results
          </p>
          <div className="flex gap-2">
            <Button
              variant="secondary"
              onClick={() => handlePageChange((filters.page || 1) - 1)}
              disabled={filters.page === 1}
            >
              Previous
            </Button>
            <span className="px-4 py-2 text-sm text-gray-700">
              Page {filters.page} of {totalPages}
            </span>
            <Button
              variant="secondary"
              onClick={() => handlePageChange((filters.page || 1) + 1)}
              disabled={filters.page === totalPages}
            >
              Next
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
