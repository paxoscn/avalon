import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { vectorService, type CreateVectorConfigRequest, type UpdateVectorConfigRequest } from '../services/vector.service';
import type { VectorConfig } from '../types';
import { Button, Card, Input, Loader, Alert } from '../components/common';

export function VectorConfigDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const isNew = id === 'new';

  const [config, setConfig] = useState<VectorConfig | null>(null);
  const [loading, setLoading] = useState(!isNew);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [formData, setFormData] = useState({
    name: '',
    provider: 'pinecone' as 'pinecone' | 'weaviate' | 'chromadb' | 'qdrant' | 'milvus',
    apiKey: '',
    apiUrl: '',
    environment: '',
    indexName: '',
    dimension: 1536,
    metric: 'cosine' as 'cosine' | 'euclidean' | 'dotproduct',
    isDefault: false,
  });

  useEffect(() => {
    if (!isNew && id) {
      loadConfig();
    }
  }, [id, isNew]);

  const loadConfig = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await vectorService.getConfig(id!);
      setConfig(data);
      setFormData({
        name: data.name,
        provider: data.provider,
        apiKey: data.config.apiKey || '',
        apiUrl: data.config.apiUrl || '',
        environment: data.config.environment || '',
        indexName: data.config.indexName || '',
        dimension: data.config.dimension || 1536,
        metric: data.config.metric || 'cosine',
        isDefault: data.isDefault,
      });
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load configuration');
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError(null);
    setSuccess(null);

    try {
      const configData = {
        apiKey: formData.apiKey || undefined,
        apiUrl: formData.apiUrl || undefined,
        environment: formData.environment || undefined,
        indexName: formData.indexName,
        dimension: formData.dimension,
        metric: formData.metric,
      };

      if (isNew) {
        const request: CreateVectorConfigRequest = {
          name: formData.name,
          provider: formData.provider,
          config: configData,
          isDefault: formData.isDefault,
        };
        const newConfig = await vectorService.createConfig(request);
        setSuccess('Configuration created successfully');
        setTimeout(() => navigate(`/config/vector/${newConfig.id}`), 1500);
      } else {
        const request: UpdateVectorConfigRequest = {
          name: formData.name,
          provider: formData.provider,
          config: configData,
          isDefault: formData.isDefault,
        };
        await vectorService.updateConfig(id!, request);
        setSuccess('Configuration updated successfully');
        await loadConfig();
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to save configuration');
    } finally {
      setSaving(false);
    }
  };

  const getProviderDefaults = (provider: string) => {
    switch (provider) {
      case 'pinecone':
        return {
          apiUrl: 'https://api.pinecone.io',
          dimension: 1536,
        };
      case 'weaviate':
        return {
          apiUrl: 'http://localhost:8080',
          dimension: 1536,
        };
      case 'chromadb':
        return {
          apiUrl: 'http://localhost:8000',
          dimension: 1536,
        };
      case 'qdrant':
        return {
          apiUrl: 'http://localhost:6333',
          dimension: 1536,
        };
      case 'milvus':
        return {
          apiUrl: 'localhost:19530',
          dimension: 1536,
        };
      default:
        return {};
    }
  };

  const handleProviderChange = (provider: 'pinecone' | 'weaviate' | 'chromadb' | 'qdrant' | 'milvus') => {
    const defaults = getProviderDefaults(provider);
    setFormData({
      ...formData,
      provider,
      apiUrl: defaults.apiUrl || '',
      dimension: defaults.dimension || 1536,
    });
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader size="lg" />
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-semibold text-gray-900">
            {isNew ? 'Add Vector Database Configuration' : 'Edit Vector Database Configuration'}
          </h1>
          <p className="mt-2 text-sm text-gray-600">
            {isNew
              ? 'Configure a new vector database provider'
              : 'Update vector database configuration'}
          </p>
        </div>
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {success && (
        <Alert type="success" onClose={() => setSuccess(null)}>
          {success}
        </Alert>
      )}

      <form onSubmit={handleSubmit} className="space-y-6">
        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Basic Information</h2>
          <div className="space-y-4">
            <Input
              label="Configuration Name"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
              placeholder="My Pinecone Config"
            />

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Provider
              </label>
              <select
                value={formData.provider}
                onChange={(e) => handleProviderChange(e.target.value as any)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                disabled={!isNew}
              >
                <option value="pinecone">Pinecone</option>
                <option value="weaviate">Weaviate</option>
                <option value="chromadb">ChromaDB</option>
                <option value="qdrant">Qdrant</option>
                <option value="milvus">Milvus</option>
              </select>
              {!isNew && (
                <p className="mt-1 text-xs text-gray-500">
                  Provider cannot be changed after creation
                </p>
              )}
            </div>

            <label className="flex items-center">
              <input
                type="checkbox"
                checked={formData.isDefault}
                onChange={(e) => setFormData({ ...formData, isDefault: e.target.checked })}
                className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span className="ml-2 text-sm text-gray-700">Set as default configuration</span>
            </label>
          </div>
        </Card>

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Connection Settings</h2>
          <div className="space-y-4">
            <Input
              label="API URL"
              type="url"
              value={formData.apiUrl}
              onChange={(e) => setFormData({ ...formData, apiUrl: e.target.value })}
              placeholder="https://api.pinecone.io"
            />

            <Input
              label="API Key"
              type="password"
              value={formData.apiKey}
              onChange={(e) => setFormData({ ...formData, apiKey: e.target.value })}
              placeholder="Enter API key"
              required={formData.provider === 'pinecone'}
            />

            {formData.provider === 'pinecone' && (
              <Input
                label="Environment"
                value={formData.environment}
                onChange={(e) => setFormData({ ...formData, environment: e.target.value })}
                placeholder="us-west1-gcp"
              />
            )}

            <Input
              label="Index Name"
              value={formData.indexName}
              onChange={(e) => setFormData({ ...formData, indexName: e.target.value })}
              required
              placeholder="my-index"
            />
          </div>
        </Card>

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Index Configuration</h2>
          <div className="space-y-4">
            <Input
              label="Vector Dimension"
              type="number"
              value={formData.dimension}
              onChange={(e) =>
                setFormData({ ...formData, dimension: parseInt(e.target.value) || 0 })
              }
              min="1"
              max="10000"
              required
            />
            <p className="text-xs text-gray-500 -mt-2">
              Must match the dimension of your embedding model (e.g., 1536 for OpenAI text-embedding-ada-002)
            </p>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Distance Metric
              </label>
              <select
                value={formData.metric}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    metric: e.target.value as any,
                  })
                }
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              >
                <option value="cosine">Cosine Similarity</option>
                <option value="euclidean">Euclidean Distance</option>
                <option value="dotproduct">Dot Product</option>
              </select>
              <p className="text-xs text-gray-500 mt-1">
                Cosine is recommended for most use cases
              </p>
            </div>
          </div>
        </Card>

        <div className="flex items-center gap-3">
          <Button type="submit" disabled={saving}>
            {saving ? 'Saving...' : isNew ? 'Create Configuration' : 'Update Configuration'}
          </Button>
          <Button
            type="button"
            variant="secondary"
            onClick={() => navigate('/config/vector')}
          >
            Cancel
          </Button>
        </div>
      </form>
    </div>
  );
}
