import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { llmService, type TestLLMRequest } from '../services/llm.service';
import type { LLMConfig, LLMTestResult } from '../types';
import { Button, Card, Loader, Alert } from '../components/common';

export function LLMConfigTestPage() {
  const { id } = useParams<{ id: string }>();
  
  const [config, setConfig] = useState<LLMConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [testing, setTesting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<LLMTestResult | null>(null);
  
  const [prompt, setPrompt] = useState('');
  const [systemPrompt, setSystemPrompt] = useState('You are a helpful assistant.');

  useEffect(() => {
    if (id) {
      loadConfig();
    }
  }, [id]);

  const loadConfig = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await llmService.getConfig(id!);
      setConfig(data);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load configuration');
    } finally {
      setLoading(false);
    }
  };

  const handleTest = async () => {
    if (!prompt.trim()) {
      setError('Please enter a prompt');
      return;
    }

    setTesting(true);
    setError(null);
    setTestResult(null);

    try {
      const request: TestLLMRequest = {
        user_prompt: prompt.trim(),
        system_prompt: systemPrompt.trim() || undefined,
      };
      const result = await llmService.testConfig(id!, request);
      setTestResult(result);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Test failed');
      setTestResult({
        success: false,
        executionTime: 0,
        error: err.response?.data?.error || 'Test failed',
      });
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
    <div className="max-w-4xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-semibold text-gray-900">Test LLM Configuration</h1>
          <p className="mt-2 text-sm text-gray-600">
            Test the LLM with custom prompts
          </p>
        </div>
        <Link to={`/config/llm/${id}`}>
          <Button variant="secondary">Back to Configuration</Button>
        </Link>
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      <Card>
        <h2 className="text-lg font-medium text-gray-900 mb-4">Configuration Details</h2>
        <div className="space-y-2 text-sm">
          <div className="flex">
            <span className="font-medium w-32">Name:</span>
            <span className="text-gray-600">{config.name}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">Provider:</span>
            <span className="text-gray-600 capitalize">{config.provider}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">Model:</span>
            <span className="text-gray-600">{config.model_name || 'N/A'}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">Temperature:</span>
            <span className="text-gray-600">{config.config.model_config.parameters.temperature ?? 0.7}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">Max Tokens:</span>
            <span className="text-gray-600">{config.config.model_config.parameters.max_tokens ?? 2000}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">Default:</span>
            <span
              className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                config.is_default
                  ? 'bg-blue-100 text-blue-800'
                  : 'bg-gray-100 text-gray-800'
              }`}
            >
              {config.is_default ? 'Yes' : 'No'}
            </span>
          </div>
        </div>
      </Card>

      <Card>
        <h2 className="text-lg font-medium text-gray-900 mb-4">Test Prompts</h2>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              System Prompt
            </label>
            <textarea
              value={systemPrompt}
              onChange={(e) => setSystemPrompt(e.target.value)}
              rows={3}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="You are a helpful assistant."
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              User Prompt <span className="text-red-500">*</span>
            </label>
            <textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              rows={6}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Enter your test prompt here..."
            />
          </div>

          <Button onClick={handleTest} disabled={testing || !prompt.trim()}>
            {testing ? 'Testing...' : 'Run Test'}
          </Button>
        </div>
      </Card>

      {testResult && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-medium text-gray-900">Test Result</h2>
            <div className="flex items-center gap-4">
              <span
                className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${
                  testResult.success
                    ? 'bg-green-100 text-green-800'
                    : 'bg-red-100 text-red-800'
                }`}
              >
                {testResult.success ? 'Success' : 'Failed'}
              </span>
              <span className="text-sm text-gray-600">
                {testResult.executionTime}ms
              </span>
            </div>
          </div>

          {testResult.error && (
            <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-sm text-red-800">{testResult.error}</p>
            </div>
          )}

          {testResult.response && (
            <div className="mb-4">
              <h3 className="text-sm font-medium text-gray-700 mb-2">Response:</h3>
              <div className="p-4 bg-gray-50 rounded-lg">
                <p className="text-sm text-gray-800 whitespace-pre-wrap">
                  {testResult.response}
                </p>
              </div>
            </div>
          )}

          {testResult.usage && (
            <div>
              <h3 className="text-sm font-medium text-gray-700 mb-2">Token Usage:</h3>
              <div className="grid grid-cols-3 gap-4 text-sm">
                <div className="p-3 bg-gray-50 rounded-lg">
                  <div className="text-gray-600">Prompt Tokens</div>
                  <div className="text-lg font-semibold text-gray-900">
                    {testResult.usage.promptTokens}
                  </div>
                </div>
                <div className="p-3 bg-gray-50 rounded-lg">
                  <div className="text-gray-600">Completion Tokens</div>
                  <div className="text-lg font-semibold text-gray-900">
                    {testResult.usage.completionTokens}
                  </div>
                </div>
                <div className="p-3 bg-gray-50 rounded-lg">
                  <div className="text-gray-600">Total Tokens</div>
                  <div className="text-lg font-semibold text-gray-900">
                    {testResult.usage.totalTokens}
                  </div>
                </div>
              </div>
            </div>
          )}
        </Card>
      )}
    </div>
  );
}
