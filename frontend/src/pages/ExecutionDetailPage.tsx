import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { executionService, type ExecutionStep, type PerformanceMetrics } from '../services/execution.service';
import type { FlowExecution } from '../types';
import { Card, Button, Loader, Alert } from '../components/common';

export function ExecutionDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [execution, setExecution] = useState<FlowExecution | null>(null);
  const [steps, setSteps] = useState<ExecutionStep[]>([]);
  const [metrics, setMetrics] = useState<PerformanceMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'timeline' | 'performance' | 'data'>('timeline');

  useEffect(() => {
    if (id) {
      loadExecutionDetails();
    }
  }, [id]);

  const loadExecutionDetails = async () => {
    if (!id) return;
    
    try {
      setLoading(true);
      setError(null);
      
      const [executionData, stepsData] = await Promise.all([
        executionService.getExecutionById(id),
        executionService.getExecutionSteps(id),
      ]);
      
      setExecution(executionData);
      setSteps(stepsData);

      if (executionData.status === 'completed' || executionData.status === 'failed') {
        try {
          const metricsData = await executionService.getPerformanceMetrics(id);
          setMetrics(metricsData);
        } catch (err) {
          console.error('Failed to load performance metrics:', err);
        }
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load execution details');
    } finally {
      setLoading(false);
    }
  };

  const handleRetry = async () => {
    if (!id) return;
    
    try {
      const result = await executionService.retryExecution(id);
      navigate(`/executions/${result.executionId}`);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to retry execution');
    }
  };

  const handleCancel = async () => {
    if (!id) return;
    
    try {
      await executionService.cancelExecution(id);
      await loadExecutionDetails();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to cancel execution');
    }
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
      case 'skipped':
        return 'bg-yellow-100 text-yellow-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  if (loading) {
    return (
      <div className="flex justify-center items-center py-12">
        <Loader />
      </div>
    );
  }

  if (error || !execution) {
    return (
      <div className="space-y-4">
        <Alert variant="error" message={error || 'Execution not found'} />
        <Button onClick={() => navigate('/executions')}>Back to Execution History</Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-semibold text-gray-900">Execution Details</h1>
        <div className="flex gap-2">
          {execution.status === 'running' && (
            <Button variant="secondary" onClick={handleCancel}>
              Cancel Execution
            </Button>
          )}
          {(execution.status === 'failed' || execution.status === 'cancelled') && (
            <Button onClick={handleRetry}>Retry Execution</Button>
          )}
          <Button variant="secondary" onClick={() => navigate('/executions')}>
            Back to List
          </Button>
        </div>
      </div>

      <Card>
        <div className="p-6 space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Execution ID
              </label>
              <p className="text-sm text-gray-900 font-mono">{execution.id}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Status
              </label>
              <span className={`inline-block px-3 py-1 text-sm font-medium rounded-full ${getStatusColor(execution.status)}`}>
                {execution.status}
              </span>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Duration
              </label>
              <p className="text-sm text-gray-900">{formatDuration(execution.executionTimeMs)}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Flow ID
              </label>
              <p className="text-sm text-gray-900 font-mono">{execution.flowId}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Flow Version
              </label>
              <p className="text-sm text-gray-900">v{execution.flowVersion}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                User ID
              </label>
              <p className="text-sm text-gray-900 font-mono">{execution.userId}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Started At
              </label>
              <p className="text-sm text-gray-900">{formatDate(execution.startedAt)}</p>
            </div>

            {execution.completedAt && (
              <div>
                <label className="block text-sm font-medium text-gray-500 mb-1">
                  Completed At
                </label>
                <p className="text-sm text-gray-900">{formatDate(execution.completedAt)}</p>
              </div>
            )}

            {execution.sessionId && (
              <div>
                <label className="block text-sm font-medium text-gray-500 mb-1">
                  Session ID
                </label>
                <p className="text-sm text-gray-900 font-mono">{execution.sessionId}</p>
              </div>
            )}
          </div>

          {execution.errorMessage && (
            <div>
              <label className="block text-sm font-medium text-red-500 mb-1">
                Error Message
              </label>
              <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                <p className="text-sm text-red-900">{execution.errorMessage}</p>
              </div>
            </div>
          )}
        </div>
      </Card>

      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          <button
            onClick={() => setActiveTab('timeline')}
            className={`py-4 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'timeline'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Timeline
          </button>
          <button
            onClick={() => setActiveTab('performance')}
            className={`py-4 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'performance'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Performance
          </button>
          <button
            onClick={() => setActiveTab('data')}
            className={`py-4 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'data'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Input/Output
          </button>
        </nav>
      </div>

      {activeTab === 'timeline' && (
        <Card>
          <div className="p-6">
            <h2 className="text-lg font-medium text-gray-900 mb-4">Execution Timeline</h2>
            {steps.length === 0 ? (
              <p className="text-gray-500">No execution steps available</p>
            ) : (
              <div className="space-y-4">
                {steps.map((step, index) => (
                  <div key={step.id} className="flex">
                    <div className="flex flex-col items-center mr-4">
                      <div className={`w-8 h-8 rounded-full flex items-center justify-center ${
                        step.status === 'completed' ? 'bg-green-500' :
                        step.status === 'failed' ? 'bg-red-500' :
                        step.status === 'running' ? 'bg-blue-500' :
                        'bg-gray-300'
                      }`}>
                        <span className="text-white text-sm font-medium">{index + 1}</span>
                      </div>
                      {index < steps.length - 1 && (
                        <div className="w-0.5 h-full bg-gray-300 mt-2"></div>
                      )}
                    </div>
                    <div className="flex-1 pb-8">
                      <div className="bg-gray-50 rounded-lg p-4">
                        <div className="flex justify-between items-start mb-2">
                          <div>
                            <h3 className="text-sm font-medium text-gray-900">{step.stepName}</h3>
                            <p className="text-xs text-gray-500">{step.stepType}</p>
                          </div>
                          <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(step.status)}`}>
                            {step.status}
                          </span>
                        </div>
                        <div className="grid grid-cols-2 gap-4 text-xs text-gray-600 mt-2">
                          <div>
                            <span className="font-medium">Started:</span> {formatDate(step.startedAt)}
                          </div>
                          {step.completedAt && (
                            <div>
                              <span className="font-medium">Completed:</span> {formatDate(step.completedAt)}
                            </div>
                          )}
                          {step.executionTimeMs && (
                            <div>
                              <span className="font-medium">Duration:</span> {formatDuration(step.executionTimeMs)}
                            </div>
                          )}
                        </div>
                        {step.errorMessage && (
                          <div className="mt-2 text-xs text-red-600">
                            <span className="font-medium">Error:</span> {step.errorMessage}
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </Card>
      )}

      {activeTab === 'performance' && (
        <Card>
          <div className="p-6">
            <h2 className="text-lg font-medium text-gray-900 mb-4">Performance Analysis</h2>
            {!metrics ? (
              <p className="text-gray-500">Performance metrics not available</p>
            ) : (
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm font-medium text-gray-700 mb-3">Step Execution Times</h3>
                  <div className="space-y-2">
                    {metrics.stepMetrics.map((metric) => (
                      <div key={metric.stepName}>
                        <div className="flex justify-between text-sm mb-1">
                          <span className="text-gray-700">{metric.stepName}</span>
                          <span className="text-gray-900 font-medium">
                            {formatDuration(metric.executionTime)} ({metric.percentage.toFixed(1)}%)
                          </span>
                        </div>
                        <div className="w-full bg-gray-200 rounded-full h-2">
                          <div
                            className="bg-blue-600 h-2 rounded-full"
                            style={{ width: `${metric.percentage}%` }}
                          ></div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                {metrics.bottlenecks.length > 0 && (
                  <div>
                    <h3 className="text-sm font-medium text-gray-700 mb-3">Performance Bottlenecks</h3>
                    <div className="space-y-2">
                      {metrics.bottlenecks.map((bottleneck, index) => (
                        <div key={index} className="bg-yellow-50 border border-yellow-200 rounded-lg p-3">
                          <div className="flex justify-between items-start">
                            <div>
                              <p className="text-sm font-medium text-gray-900">{bottleneck.stepName}</p>
                              <p className="text-xs text-gray-600 mt-1">{bottleneck.reason}</p>
                            </div>
                            <span className="text-sm font-medium text-yellow-800">
                              {formatDuration(bottleneck.executionTime)}
                            </span>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        </Card>
      )}

      {activeTab === 'data' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card>
            <div className="p-6">
              <h2 className="text-lg font-medium text-gray-900 mb-4">Input Data</h2>
              {execution.inputData ? (
                <div className="bg-gray-50 rounded-lg p-4">
                  <pre className="text-xs text-gray-900 overflow-x-auto">
                    {JSON.stringify(execution.inputData, null, 2)}
                  </pre>
                </div>
              ) : (
                <p className="text-gray-500">No input data</p>
              )}
            </div>
          </Card>

          <Card>
            <div className="p-6">
              <h2 className="text-lg font-medium text-gray-900 mb-4">Output Data</h2>
              {execution.outputData ? (
                <div className="bg-gray-50 rounded-lg p-4">
                  <pre className="text-xs text-gray-900 overflow-x-auto">
                    {JSON.stringify(execution.outputData, null, 2)}
                  </pre>
                </div>
              ) : (
                <p className="text-gray-500">No output data</p>
              )}
            </div>
          </Card>
        </div>
      )}
    </div>
  );
}
