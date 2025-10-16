import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { llmService, type CreateLLMConfigRequest, type UpdateLLMConfigRequest } from '../services/llm.service';
import type { LLMConfig } from '../types';
import { Button, Card, Input, Loader, Alert } from '../components/common';

export function LLMConfigDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const isNew = id === 'new';

  const [config, setConfig] = useState<LLMConfig | null>(null);
  const [loading, setLoading] = useState(!isNew);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [formData, setFormData] = useState({
    name: '',
    provider: 'openai' as 'openai' | 'claude' | 'local',
    apiKey: '',
    apiUrl: '',
    model: '',
    temperature: 0.7,
    maxTokens: 2000,
    topP: 1.0,
    frequencyPenalty: 0,
    presencePenalty: 0,
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
      const data = await llmService.getConfig(id!);
      setConfig(data);
      setFormData({
        name: data.name,
        provider: data.provider,
        apiKey: data.config.model_config.credentials.api_key || '',
        apiUrl: data.config.model_config.credentials.api_base || '',
        model: data.model_name || '',
        temperature: data.config.model_config.parameters.temperature ?? 0.7,
        maxTokens: data.config.model_config.parameters.max_tokens ?? 2000,
        topP: data.config.model_config.parameters.top_p ?? 1.0,
        frequencyPenalty: data.config.model_config.parameters.frequency_penalty ?? 0,
        presencePenalty: data.config.model_config.parameters.presence_penalty ?? 0,
        isDefault: data.is_default,
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
        model: formData.model,
        temperature: formData.temperature,
        maxTokens: formData.maxTokens,
        topP: formData.topP,
        frequencyPenalty: formData.frequencyPenalty,
        presencePenalty: formData.presencePenalty,
      };

      const parameters = {
        temperature: formData.temperature,
        max_tokens: formData.maxTokens,
        top_p: formData.topP,
        frequency_penalty: formData.frequencyPenalty,
        presence_penalty: formData.presencePenalty,
        stop_sequences: [],
        custom_parameters: {},
      };

      const credentials = {
        api_key: formData.apiKey || undefined,
        api_base: formData.apiUrl || undefined,
        organization: '',
        custom_headers: {},
      };

      if (isNew) {
        const request: CreateLLMConfigRequest = {
          name: formData.name,
          provider: formData.provider,
          model_name: formData.model,
          parameters: parameters,
          credentials: credentials,
          isDefault: formData.isDefault,
        };
        const newConfig = await llmService.createConfig(request);
        setSuccess('Configuration created successfully');
        setTimeout(() => navigate(`/config/llm/${newConfig.id}`), 1500);
      } else {
        const request: UpdateLLMConfigRequest = {
          name: formData.name,
          provider: formData.provider,
          model_name: formData.model,
          parameters: parameters,
          credentials: credentials,
          isDefault: formData.isDefault,
        };
        await llmService.updateConfig(id!, request);
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
      case 'openai':
        return {
          apiUrl: 'https://api.openai.com/v1',
          model: 'gpt-4',
        };
      case 'claude':
        return {
          apiUrl: 'https://api.anthropic.com/v1',
          model: 'claude-3-opus-20240229',
        };
      case 'local':
        return {
          apiUrl: 'http://localhost:11434',
          model: 'llama2',
        };
      default:
        return {};
    }
  };

  const handleProviderChange = (provider: 'openai' | 'claude' | 'local') => {
    const defaults = getProviderDefaults(provider);
    setFormData({
      ...formData,
      provider,
      apiUrl: defaults.apiUrl || '',
      model: defaults.model || '',
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
            {isNew ? 'Add LLM Configuration' : 'Edit LLM Configuration'}
          </h1>
          <p className="mt-2 text-sm text-gray-600">
            {isNew
              ? 'Configure a new large language model provider'
              : 'Update LLM provider configuration'}
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
              placeholder="My OpenAI Config"
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
                <option value="openai">OpenAI</option>
                <option value="claude">Claude (Anthropic)</option>
                <option value="local">Local LLM</option>
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
              placeholder="https://api.openai.com/v1"
            />

            <Input
              label="API Key"
              type="password"
              value={formData.apiKey}
              onChange={(e) => setFormData({ ...formData, apiKey: e.target.value })}
              placeholder="sk-..."
              required={formData.provider !== 'local'}
            />

            <Input
              label="Model"
              value={formData.model}
              onChange={(e) => setFormData({ ...formData, model: e.target.value })}
              required
              placeholder="gpt-4"
            />
          </div>
        </Card>

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Model Parameters</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Temperature: {formData.temperature}
              </label>
              <input
                type="range"
                min="0"
                max="2"
                step="0.1"
                value={formData.temperature}
                onChange={(e) =>
                  setFormData({ ...formData, temperature: parseFloat(e.target.value) })
                }
                className="w-full"
              />
              <p className="text-xs text-gray-500 mt-1">
                Controls randomness. Lower is more focused, higher is more creative.
              </p>
            </div>

            <Input
              label="Max Tokens"
              type="number"
              value={formData.maxTokens}
              onChange={(e) =>
                setFormData({ ...formData, maxTokens: parseInt(e.target.value) || 0 })
              }
              min="1"
              max="100000"
            />

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Top P: {formData.topP}
              </label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={formData.topP}
                onChange={(e) =>
                  setFormData({ ...formData, topP: parseFloat(e.target.value) })
                }
                className="w-full"
              />
              <p className="text-xs text-gray-500 mt-1">
                Nucleus sampling. Alternative to temperature.
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Frequency Penalty: {formData.frequencyPenalty}
              </label>
              <input
                type="range"
                min="-2"
                max="2"
                step="0.1"
                value={formData.frequencyPenalty}
                onChange={(e) =>
                  setFormData({ ...formData, frequencyPenalty: parseFloat(e.target.value) })
                }
                className="w-full"
              />
              <p className="text-xs text-gray-500 mt-1">
                Reduces repetition of token sequences.
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Presence Penalty: {formData.presencePenalty}
              </label>
              <input
                type="range"
                min="-2"
                max="2"
                step="0.1"
                value={formData.presencePenalty}
                onChange={(e) =>
                  setFormData({ ...formData, presencePenalty: parseFloat(e.target.value) })
                }
                className="w-full"
              />
              <p className="text-xs text-gray-500 mt-1">
                Encourages talking about new topics.
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
            onClick={() => navigate('/config/llm')}
          >
            Cancel
          </Button>
        </div>
      </form>
    </div>
  );
}
