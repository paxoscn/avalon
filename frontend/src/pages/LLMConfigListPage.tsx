import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { llmService } from '../services/llm.service';
import type { LLMConfig } from '../types';
import { Button, Card, Loader, Alert } from '../components/common';

export function LLMConfigListPage() {
  const [configs, setConfigs] = useState<LLMConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [testingId, setTestingId] = useState<string | null>(null);

  useEffect(() => {
    loadConfigs();
  }, []);

  const loadConfigs = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await llmService.listConfigs();
      setConfigs(data);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load LLM configurations');
    } finally {
      setLoading(false);
    }
  };

  const handleTestConnection = async (id: string) => {
    try {
      setTestingId(id);
      setError(null);
      const result = await llmService.testConnection(id);
      if (result.success) {
        alert('Connection test successful!');
      } else {
        alert(`Connection test failed: ${result.message}`);
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'Connection test failed');
    } finally {
      setTestingId(null);
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      await llmService.setDefault(id);
      await loadConfigs();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to set default configuration');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this configuration?')) {
      return;
    }

    try {
      await llmService.deleteConfig(id);
      await loadConfigs();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to delete configuration');
    }
  };

  const getProviderIcon = (provider: string) => {
    switch (provider) {
      case 'openai':
        return '🤖';
      case 'claude':
        return '🧠';
      case 'local':
        return '💻';
      default:
        return '🔧';
    }
  };

  if (loading) {
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
          <h1 className="text-3xl font-semibold text-gray-900">大模型配置</h1>
          <p className="mt-2 text-sm text-gray-600">
            管理大模型供应商
          </p>
        </div>
        <Link to="/config/llm/new">
          <Button>增加配置</Button>
        </Link>
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {configs.length === 0 ? (
        <Card>
          <div className="text-center py-12">
            <svg
              className="mx-auto h-12 w-12 text-gray-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
              />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-gray-900">No configurations</h3>
            <p className="mt-1 text-sm text-gray-500">
              Get started by adding a new LLM configuration.
            </p>
            <div className="mt-6">
              <Link to="/config/llm/new">
                <Button>Add Configuration</Button>
              </Link>
            </div>
          </div>
        </Card>
      ) : (
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          {configs.map((config) => (
            <Card key={config.id} className="hover:shadow-lg transition-shadow">
              <div className="space-y-4">
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-3 flex-1 min-w-0">
                    <span className="text-3xl">{getProviderIcon(config.provider)}</span>
                    <div className="flex-1 min-w-0">
                      <h3 className="text-lg font-medium text-gray-900 truncate">
                        {config.name}
                      </h3>
                      <p className="text-sm text-gray-500 capitalize">{config.provider}</p>
                    </div>
                  </div>
                  {config.is_default && (
                    <span className="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                      Default
                    </span>
                  )}
                </div>

                <div className="text-sm text-gray-500">
                  <div>Model: {config.model_name || 'N/A'}</div>
                  <div>Created: {new Date(config.created_at).toLocaleDateString()}</div>
                </div>

                <div className="flex items-center gap-2 pt-4 border-t border-gray-200">
                  <Link to={`/config/llm/${config.id}`} className="flex-1">
                    <Button variant="secondary" className="w-full">
                      Configure
                    </Button>
                  </Link>
                  <Link to={`/config/llm/${config.id}/test`} className="flex-1">
                    <Button variant="secondary" className="w-full">
                      Test
                    </Button>
                  </Link>
                </div>

                <div className="flex items-center gap-2">
                  {/* <Button
                    variant="secondary"
                    onClick={() => handleTestConnection(config.id)}
                    disabled={testingId === config.id}
                    className="flex-1"
                  >
                    {testingId === config.id ? 'Testing...' : 'Test Connection'}
                  </Button> */}
                  {!config.is_default && (
                    <Button
                      variant="secondary"
                      onClick={() => handleSetDefault(config.id)}
                      className="flex-1"
                    >
                      Set Default
                    </Button>
                  )}
                  {!config.is_default && (
                    <Button
                      variant="secondary"
                      onClick={() => handleDelete(config.id)}
                      className="flex-1 text-red-600 hover:text-red-700"
                    >
                      Delete
                    </Button>
                  )}
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
