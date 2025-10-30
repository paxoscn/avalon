import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { flowService } from '../services/flow.service';
import type { FlowExecution } from '../types';
import { Button, Card, Loader, Alert } from '../components/common';

export const FlowExecutionPage = () => {
  const { flowId, executionId } = useParams<{ flowId: string; executionId: string }>();
  const navigate = useNavigate();
  const [execution, setExecution] = useState<FlowExecution | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [polling, setPolling] = useState(true);

  useEffect(() => {
    if (executionId) {
      loadExecution();
    }
  }, [executionId]);

  useEffect(() => {
    if (!polling || !executionId) return;

    const interval = setInterval(() => {
      loadExecution(true);
    }, 2000);

    return () => clearInterval(interval);
  }, [polling, executionId]);

  const loadExecution = async (silent = false) => {
    if (!executionId) return;

    try {
      if (!silent) {
        setLoading(true);
        setError(null);
      }
      const data = await flowService.getExecutionById(executionId);
      setExecution(data);

      if (data.status.toLowerCase() === 'completed' || data.status.toLowerCase() === 'failed' || data.status.toLowerCase() === 'cancelled') {
        setPolling(false);
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load execution details');
      setPolling(false);
    } finally {
      if (!silent) {
        setLoading(false);
      }
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'text-green-600 bg-green-100';
      case 'running':
        return 'text-blue-600 bg-blue-100';
      case 'failed':
        return 'text-red-600 bg-red-100';
      case 'pending':
        return 'text-yellow-600 bg-yellow-100';
      case 'cancelled':
        return 'text-gray-600 bg-gray-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader />
      </div>
    );
  }

  if (!execution) {
    return (
      <div className="text-center py-12">
        <p className="text-gray-500">Execution not found</p>
        <Button className="mt-4" onClick={() => navigate(`/flows/${flowId}`)}>
          Back to Flow
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <Button variant="secondary" size="sm" onClick={() => navigate(`/flows/${flowId}`)}>
            ← Back to Flow
          </Button>
          <h1 className="text-2xl font-semibold text-gray-900 mt-2">Execution Details</h1>
          <p className="text-gray-600 mt-1">ID: {execution.id}</p>
        </div>
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <Card>
          <h3 className="text-sm font-medium text-gray-500 mb-2">Status</h3>
          <div className="flex items-center gap-2">
            <span
              className={`inline-flex px-3 py-1 text-sm font-semibold rounded-full ${getStatusColor(
                execution.status
              )}`}
            >
              {execution.status}
            </span>
            {(execution.status.toLowerCase() === 'running' || execution.status.toLowerCase() === 'pending') && (
              <div className="animate-spin h-4 w-4 border-2 border-blue-600 border-t-transparent rounded-full" />
            )}
          </div>
        </Card>

        <Card>
          <h3 className="text-sm font-medium text-gray-500 mb-2">Flow Version</h3>
          <p className="text-2xl font-semibold text-gray-900">v{execution.flow_version}</p>
        </Card>

        <Card>
          <h3 className="text-sm font-medium text-gray-500 mb-2">Started At</h3>
          <p className="text-sm text-gray-900">{new Date(execution.started_at).toLocaleString()}</p>
        </Card>

        <Card>
          <h3 className="text-sm font-medium text-gray-500 mb-2">Execution Time</h3>
          <p className="text-2xl font-semibold text-gray-900">
            {execution.execution_time_ms ? `${execution.execution_time_ms}ms` : '-'}
          </p>
        </Card>
      </div>

      {execution.input_data && (
        <Card>
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Input Data</h2>
          <pre className="bg-gray-50 p-4 rounded-lg overflow-x-auto text-sm">
            {JSON.stringify(execution.input_data, null, 2)}
          </pre>
        </Card>
      )}

      {execution.output_data && (
        <Card>
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Output Data</h2>
          <pre className="bg-gray-50 p-4 rounded-lg overflow-x-auto text-sm">
            {JSON.stringify(execution.output_data, null, 2)}
          </pre>
        </Card>
      )}

      {execution.error_message && (
        <Card>
          <h2 className="text-lg font-semibold text-red-600 mb-4">Error</h2>
          <div className="bg-red-50 p-4 rounded-lg">
            <p className="text-red-800 font-mono text-sm">{execution.error_message}</p>
          </div>
        </Card>
      )}

      {execution.completed_at && (
        <Card>
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Timeline</h2>
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-gray-600">Started:</span>
              <span className="text-gray-900">{new Date(execution.started_at).toLocaleString()}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-600">Completed:</span>
              <span className="text-gray-900">
                {new Date(execution.completed_at).toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-600">Duration:</span>
              <span className="text-gray-900">{execution.execution_time_ms}ms</span>
            </div>
          </div>
        </Card>
      )}
    </div>
  );
};
