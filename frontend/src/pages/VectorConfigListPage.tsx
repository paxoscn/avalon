import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { vectorService } from '../services/vector.service';
import type { VectorConfig } from '../types';
import { Button, Card, Loader, Alert } from '../components/common';

export function VectorConfigListPage() {
  const [configs, setConfigs] = useState<VectorConfig[]>([]);
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
      
      // 添加模拟数据
      const mockConfigs: VectorConfig[] = [
        {
          id: 'mock-vector-1',
          name: '商品知识库',
          provider: 'pinecone',
          config: {
            indexName: 'product-knowledge',
            dimension: 1536,
            apiKey: '***',
            environment: 'production'
          },
          isDefault: true,
          created_at: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
        },
        {
          id: 'mock-vector-2',
          name: '高德POI知识库',
          provider: 'qdrant',
          config: {
            indexName: 'amap-poi-data',
            dimension: 768,
            url: 'https://qdrant.example.com',
            apiKey: '***'
          },
          isDefault: false,
          created_at: new Date(Date.now() - 45 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000).toISOString()
        },
        {
          id: 'mock-vector-3',
          name: '活动知识库',
          provider: 'weaviate',
          config: {
            indexName: 'marketing-campaigns',
            dimension: 1536,
            url: 'https://weaviate.example.com',
            apiKey: '***'
          },
          isDefault: false,
          created_at: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString()
        },
        {
          id: 'mock-vector-4',
          name: '优惠券知识库',
          provider: 'chromadb',
          config: {
            indexName: 'coupon-database',
            dimension: 384,
            host: 'localhost',
            port: 8000
          },
          isDefault: false,
          created_at: new Date(Date.now() - 20 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString()
        },
        {
          id: 'mock-vector-5',
          name: '门店信息知识库',
          provider: 'milvus',
          config: {
            indexName: 'store-information',
            dimension: 1536,
            host: 'milvus.example.com',
            port: 19530
          },
          isDefault: false,
          created_at: new Date(Date.now() - 40 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 4 * 24 * 60 * 60 * 1000).toISOString()
        },
        {
          id: 'mock-vector-6',
          name: '客户反馈知识库',
          provider: 'pinecone',
          config: {
            indexName: 'customer-feedback',
            dimension: 768,
            apiKey: '***',
            environment: 'production'
          },
          isDefault: false,
          created_at: new Date(Date.now() - 25 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString()
        },
        {
          id: 'mock-vector-7',
          name: '政策法规知识库',
          provider: 'qdrant',
          config: {
            indexName: 'policy-regulations',
            dimension: 1536,
            url: 'https://qdrant.example.com',
            apiKey: '***'
          },
          isDefault: false,
          created_at: new Date(Date.now() - 50 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
        }
      ];
      
      const data = await vectorService.listConfigs();
      // 将模拟数据添加到实际数据前面
      setConfigs([...mockConfigs, ...data]);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load vector configurations');
    } finally {
      setLoading(false);
    }
  };

  const handleTestConnection = async (id: string) => {
    try {
      setTestingId(id);
      setError(null);
      const result = await vectorService.testConnection(id);
      if (result.success) {
        const info = result.indexInfo
          ? `\nDimension: ${result.indexInfo.dimension || 'N/A'}\nVector Count: ${result.indexInfo.count || 'N/A'}`
          : '';
        alert(`Connection test successful!${info}`);
      } else {
        alert(`Connection test failed: ${result.message || result.error}`);
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'Connection test failed');
    } finally {
      setTestingId(null);
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      await vectorService.setDefault(id);
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
      await vectorService.deleteConfig(id);
      await loadConfigs();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to delete configuration');
    }
  };

  const getProviderIcon = (provider: string) => {
    switch (provider) {
      case 'pinecone':
        return '🌲';
      case 'weaviate':
        return '🕸️';
      case 'chromadb':
        return '🎨';
      case 'qdrant':
        return '🔷';
      case 'milvus':
        return '🦅';
      default:
        return '📊';
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
          <h1 className="text-3xl font-semibold text-gray-900">知识库配置</h1>
          <p className="mt-2 text-sm text-gray-600">
            管理企业知识库配置
          </p>
        </div>
        <Link to="/config/vector/new">
          <Button>Add Configuration</Button>
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
                d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"
              />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-gray-900">No configurations</h3>
            <p className="mt-1 text-sm text-gray-500">
              Get started by adding a new vector database configuration.
            </p>
            <div className="mt-6">
              <Link to="/config/vector/new">
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
                  {config.isDefault && (
                    <span className="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                      Default
                    </span>
                  )}
                </div>

                <div className="text-sm text-gray-500">
                  <div>Index: {config.config.indexName || 'N/A'}</div>
                  <div>Dimension: {config.config.dimension || 'N/A'}</div>
                  <div>Created: {new Date(config.created_at).toLocaleDateString()}</div>
                </div>

                <div className="flex items-center gap-2 pt-4 border-t border-gray-200">
                  <Link to={`/config/vector/${config.id}`} className="flex-1">
                    <Button variant="secondary" className="w-full">
                      Configure
                    </Button>
                  </Link>
                  <Link to={`/config/vector/${config.id}/test`} className="flex-1">
                    <Button variant="secondary" className="w-full">
                      Test
                    </Button>
                  </Link>
                </div>

                <div className="flex items-center gap-2">
                  <Button
                    variant="secondary"
                    onClick={() => handleTestConnection(config.id)}
                    disabled={testingId === config.id}
                    className="flex-1"
                  >
                    {testingId === config.id ? 'Testing...' : 'Test Connection'}
                  </Button>
                  {!config.isDefault && (
                    <Button
                      variant="secondary"
                      onClick={() => handleSetDefault(config.id)}
                      className="flex-1"
                    >
                      Set Default
                    </Button>
                  )}
                </div>

                {!config.isDefault && (
                  <Button
                    variant="secondary"
                    onClick={() => handleDelete(config.id)}
                    className="w-full text-red-600 hover:text-red-700"
                  >
                    Delete
                  </Button>
                )}
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
