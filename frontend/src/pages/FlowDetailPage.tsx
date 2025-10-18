import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { flowService } from '../services/flow.service';
import type { Flow, FlowExecution } from '../types';
import { Button, Card, Loader, Alert, Modal } from '../components/common';

export const FlowDetailPage = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [flow, setFlow] = useState<Flow | null>(null);
  const [executions, setExecutions] = useState<FlowExecution[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showExecuteModal, setShowExecuteModal] = useState(false);
  const [executing, setExecuting] = useState(false);
  const [executionVariables, setExecutionVariables] = useState('{}');
  const [showCurlModal, setShowCurlModal] = useState(false);

  useEffect(() => {
    if (id) {
      loadFlowDetails();
    }
  }, [id]);

  const loadFlowDetails = async () => {
    if (!id) return;

    try {
      setLoading(true);
      setError(null);
      const [flowData, executionsData] = await Promise.all([
        flowService.getFlowById(id),
        flowService.getFlowExecutions(id),
      ]);
      setFlow(flowData);
      setExecutions(executionsData);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load flow details');
    } finally {
      setLoading(false);
    }
  };

  const handleExecute = async () => {
    if (!id) return;

    try {
      setExecuting(true);
      let variables = {};
      try {
        variables = JSON.parse(executionVariables);
      } catch {
        setError('Invalid JSON format for variables');
        return;
      }

      const result = await flowService.executeFlow(id, { variables });
      setShowExecuteModal(false);
      setExecutionVariables('{}');
      navigate(`/flows/${id}/executions/${result.executionId}`);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to execute flow');
    } finally {
      setExecuting(false);
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

  if (!flow) {
    return (
      <div className="text-center py-12">
        <p className="text-gray-500">Flow not found</p>
        <Button className="mt-4" onClick={() => navigate('/flows')}>
          Back to Flows
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <Button variant="secondary" size="sm" onClick={() => navigate('/flows')}>
            ← Back
          </Button>
          <h1 className="text-2xl font-semibold text-gray-900 mt-2">{flow.name}</h1>
          {flow.description && <p className="text-gray-600 mt-1">{flow.description}</p>}
        </div>
        <div className="flex gap-3">
          <Button variant="secondary" onClick={() => navigate(`/flows/${id}/versions`)}>
            Version History
          </Button>
          <Button variant="secondary" onClick={() => setShowCurlModal(true)}>
            Show cURL Command
          </Button>
          {flow.status === 'active' && (
            <Button onClick={() => setShowExecuteModal(true)}>Execute Flow</Button>
          )}
        </div>
      </div>

      {error && (
        <Alert variant="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card>
          <h3 className="text-sm font-medium text-gray-500 mb-2">Status</h3>
          <span
            className={`inline-flex px-3 py-1 text-sm font-semibold rounded-full ${
              flow.status === 'active'
                ? 'text-green-600 bg-green-100'
                : flow.status === 'draft'
                ? 'text-yellow-600 bg-yellow-100'
                : 'text-gray-600 bg-gray-100'
            }`}
          >
            {flow.status}
          </span>
        </Card>

        <Card>
          <h3 className="text-sm font-medium text-gray-500 mb-2">Current Version</h3>
          <p className="text-2xl font-semibold text-gray-900">v{flow.current_version}</p>
        </Card>

        <Card>
          <h3 className="text-sm font-medium text-gray-500 mb-2">Last Updated</h3>
          <p className="text-lg text-gray-900">
            {new Date(flow.updated_at).toLocaleDateString()}
          </p>
        </Card>
      </div>

      <Card>
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Recent Executions</h2>
        {executions.length === 0 ? (
          <p className="text-gray-500 text-center py-8">No executions yet</p>
        ) : (
          <div className="space-y-3">
            {executions.slice(0, 10).map((execution) => (
              <div
                key={execution.id}
                className="flex items-center justify-between p-4 bg-gray-50 rounded-lg hover:bg-gray-100 cursor-pointer transition-colors"
                onClick={() => navigate(`/flows/${id}/executions/${execution.id}`)}
              >
                <div className="flex-1">
                  <div className="flex items-center gap-3">
                    <span
                      className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${getStatusColor(
                        execution.status
                      )}`}
                    >
                      {execution.status}
                    </span>
                    <span className="text-sm text-gray-600">
                      {new Date(execution.startedAt).toLocaleString()}
                    </span>
                  </div>
                  {execution.errorMessage && (
                    <p className="text-sm text-red-600 mt-1">{execution.errorMessage}</p>
                  )}
                </div>
                <div className="text-right">
                  {execution.executionTimeMs && (
                    <p className="text-sm text-gray-600">{execution.executionTimeMs}ms</p>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </Card>

      <Modal
        isOpen={showExecuteModal}
        onClose={() => setShowExecuteModal(false)}
        title="Execute Flow"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Input Variables (JSON)
            </label>
            <textarea
              value={executionVariables}
              onChange={(e) => setExecutionVariables(e.target.value)}
              className="w-full h-32 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
              placeholder='{"key": "value"}'
            />
          </div>
          <div className="flex gap-3 justify-end">
            <Button variant="secondary" onClick={() => setShowExecuteModal(false)}>
              Cancel
            </Button>
            <Button onClick={handleExecute} disabled={executing}>
              {executing ? 'Executing...' : 'Execute'}
            </Button>
          </div>
        </div>
      </Modal>

      <Modal
        isOpen={showCurlModal}
        onClose={() => setShowCurlModal(false)}
        title="cURL Command"
      >
        <div className="space-y-4">
          <p className="text-sm text-gray-600">
            Use this command to execute the flow from the command line:
          </p>
          <div className="relative">
            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm font-mono">
{`curl -v -X POST ${import.meta.env.VITE_API_BASE_URL}/flows/${id}/execute \\
  -H "Content-Type: application/json" \\
  -d '{
    "variables": {}
  }'`}
            </pre>
            <Button
              size="sm"
              variant="secondary"
              className="absolute top-2 right-2"
              onClick={() => {
                const curlCommand = `curl -v -X POST ${import.meta.env.VITE_API_BASE_URL}/flows/${id}/execute -H "Content-Type: application/json" -d '{"variables": {}}'`;
                navigator.clipboard.writeText(curlCommand);
              }}
            >
              Copy
            </Button>
          </div>
          <div className="text-sm text-gray-600">
            <p className="font-medium mb-2">With custom variables:</p>
            <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto font-mono">
{`curl -v -X POST ${import.meta.env.VITE_API_BASE_URL}/flows/${id}/execute \\
  -H "Content-Type: application/json" \\
  -d '{
    "variables": {
      "key1": "value1",
      "key2": "value2"
    }
  }'`}
            </pre>
          </div>
        </div>
      </Modal>
    </div>
  );
};
