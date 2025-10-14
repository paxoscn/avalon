import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { vectorService, type QueryVectorRequest, type UpsertVectorRequest } from '../services/vector.service';
import type { VectorConfig } from '../types';
import { Button, Card, Input, Loader, Alert } from '../components/common';

export function VectorConfigTestPage() {
  const { id } = useParams<{ id: string }>();
  
  const [config, setConfig] = useState<VectorConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [testing, setTesting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [indexInfo, setIndexInfo] = useState<any>(null);
  
  const [activeTab, setActiveTab] = useState<'query' | 'upsert' | 'info'>('info');
  
  // Query state
  const [queryVector, setQueryVector] = useState('');
  const [topK, setTopK] = useState(5);
  const [queryResults, setQueryResults] = useState<any>(null);
  
  // Upsert state
  const [upsertData, setUpsertData] = useState('');

  useEffect(() => {
    if (id) {
      loadConfig();
      loadIndexInfo();
    }
  }, [id]);

  const loadConfig = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await vectorService.getConfig(id!);
      setConfig(data);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load configuration');
    } finally {
      setLoading(false);
    }
  };

  const loadIndexInfo = async () => {
    try {
      const info = await vectorService.getIndexInfo(id!);
      setIndexInfo(info);
    } catch (err: any) {
      console.error('Failed to load index info:', err);
    }
  };

  const handleQuery = async () => {
    if (!queryVector.trim()) {
      setError('Please enter a query vector');
      return;
    }

    try {
      const vector = JSON.parse(queryVector);
      if (!Array.isArray(vector) || !vector.every((v) => typeof v === 'number')) {
        setError('Query vector must be an array of numbers');
        return;
      }

      setTesting(true);
      setError(null);
      setQueryResults(null);

      const request: QueryVectorRequest = {
        vector,
        topK,
      };

      const result = await vectorService.queryVectors(id!, request);
      setQueryResults(result);
      setSuccess(`Found ${result.results.length} results in ${result.executionTime}ms`);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Query failed');
    } finally {
      setTesting(false);
    }
  };

  const handleUpsert = async () => {
    if (!upsertData.trim()) {
      setError('Please enter vector data');
      return;
    }

    try {
      const vectors = JSON.parse(upsertData);
      if (!Array.isArray(vectors)) {
        setError('Data must be an array of vector records');
        return;
      }

      setTesting(true);
      setError(null);

      const request: UpsertVectorRequest = { vectors };
      await vectorService.upsertVectors(id!, request);
      setSuccess(`Successfully upserted ${vectors.length} vectors`);
      await loadIndexInfo();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Upsert failed');
    } finally {
      setTesting(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader size="lg" />
      </div>
    );
  }

  if (!config) {
    return <Alert type="error">Configuration not found</Alert>;
  }

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-semibold text-gray-900">Test Vector Database</h1>
          <p className="mt-2 text-sm text-gray-600">
            Test queries and manage vectors
          </p>
        </div>
        <Link to={`/config/vector/${id}`}>
          <Button variant="secondary">Back to Configuration</Button>
        </Link>
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

      <Card>
        <h2 className="text-lg font-medium text-gray-900 mb-4">Configuration Details</h2>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span className="font-medium text-gray-700">Name:</span>
            <p className="text-gray-600 mt-1">{config.name}</p>
          </div>
          <div>
            <span className="font-medium text-gray-700">Provider:</span>
            <p className="text-gray-600 mt-1 capitalize">{config.provider}</p>
          </div>
          <div>
            <span className="font-medium text-gray-700">Index:</span>
            <p className="text-gray-600 mt-1">{config.config.indexName || 'N/A'}</p>
          </div>
          <div>
            <span className="font-medium text-gray-700">Dimension:</span>
            <p className="text-gray-600 mt-1">{config.config.dimension || 'N/A'}</p>
          </div>
          <div>
            <span className="font-medium text-gray-700">Metric:</span>
            <p className="text-gray-600 mt-1 capitalize">{config.config.metric || 'N/A'}</p>
          </div>
          <div>
            <span className="font-medium text-gray-700">Default:</span>
            <span
              className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                config.isDefault
                  ? 'bg-blue-100 text-blue-800'
                  : 'bg-gray-100 text-gray-800'
              }`}
            >
              {config.isDefault ? 'Yes' : 'No'}
            </span>
          </div>
        </div>
      </Card>

      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          <button
            onClick={() => setActiveTab('info')}
            className={`py-4 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'info'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Index Info
          </button>
          <button
            onClick={() => setActiveTab('query')}
            className={`py-4 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'query'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Query Vectors
          </button>
          <button
            onClick={() => setActiveTab('upsert')}
            className={`py-4 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'upsert'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Upsert Vectors
          </button>
        </nav>
      </div>

      {activeTab === 'info' && (
        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Index Information</h2>
          {indexInfo ? (
            <div className="space-y-3">
              {Object.entries(indexInfo).map(([key, value]) => (
                <div key={key} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                  <span className="font-medium text-gray-700 capitalize">
                    {key.replace(/([A-Z])/g, ' $1').trim()}:
                  </span>
                  <span className="text-gray-900">{JSON.stringify(value)}</span>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-gray-500">Loading index information...</p>
          )}
          <div className="mt-4">
            <Button onClick={loadIndexInfo} variant="secondary">
              Refresh Info
            </Button>
          </div>
        </Card>
      )}

      {activeTab === 'query' && (
        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Query Vectors</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Query Vector (JSON array)
              </label>
              <textarea
                value={queryVector}
                onChange={(e) => setQueryVector(e.target.value)}
                rows={6}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                placeholder="[0.1, 0.2, 0.3, ...]"
              />
              <p className="text-xs text-gray-500 mt-1">
                Enter a {config.config.dimension}-dimensional vector as a JSON array
              </p>
            </div>

            <Input
              label="Top K Results"
              type="number"
              value={topK}
              onChange={(e) => setTopK(parseInt(e.target.value) || 5)}
              min="1"
              max="100"
            />

            <Button onClick={handleQuery} disabled={testing}>
              {testing ? 'Querying...' : 'Run Query'}
            </Button>
          </div>

          {queryResults && (
            <div className="mt-6">
              <h3 className="text-sm font-medium text-gray-700 mb-3">
                Results ({queryResults.results.length}):
              </h3>
              <div className="space-y-2">
                {queryResults.results.map((result: any, idx: number) => (
                  <div key={idx} className="p-3 bg-gray-50 rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <span className="font-medium text-gray-900">ID: {result.id}</span>
                      <span className="text-sm text-gray-600">Score: {result.score.toFixed(4)}</span>
                    </div>
                    {result.metadata && (
                      <pre className="text-xs text-gray-600 overflow-x-auto">
                        {JSON.stringify(result.metadata, null, 2)}
                      </pre>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}
        </Card>
      )}

      {activeTab === 'upsert' && (
        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Upsert Vectors</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Vector Data (JSON array)
              </label>
              <textarea
                value={upsertData}
                onChange={(e) => setUpsertData(e.target.value)}
                rows={12}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                placeholder={`[
  {
    "id": "vec1",
    "vector": [0.1, 0.2, ...],
    "metadata": { "text": "example" }
  }
]`}
              />
              <p className="text-xs text-gray-500 mt-1">
                Enter an array of vector records with id, vector, and optional metadata
              </p>
            </div>

            <Button onClick={handleUpsert} disabled={testing}>
              {testing ? 'Upserting...' : 'Upsert Vectors'}
            </Button>
          </div>
        </Card>
      )}
    </div>
  );
}
