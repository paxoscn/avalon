import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { llmService, type CreateLLMConfigRequest, type UpdateLLMConfigRequest } from '../services/llm.service';
import type { LLMConfig } from '../types';
import { Button, Card, Input, Loader, Alert } from '../components/common';

export function LLMConfigDetailPage() {
  const { t } = useTranslation();
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
      setError(err.response?.data?.error || t('llmConfig.errors.loadConfigFailed'));
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
        setSuccess(t('llmConfig.success.created'));
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
        setSuccess(t('llmConfig.success.updated'));
        await loadConfig();
      }
    } catch (err: any) {
      setError(err.response?.data?.error || t('llmConfig.errors.saveFailed'));
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
            {isNew ? t('llmConfig.detail.addTitle') : t('llmConfig.detail.editTitle')}
          </h1>
          <p className="mt-2 text-sm text-gray-600">
            {isNew
              ? t('llmConfig.detail.addDescription')
              : t('llmConfig.detail.editDescription')}
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
          <h2 className="text-lg font-medium text-gray-900 mb-4">{t('llmConfig.detail.basicInfo')}</h2>
          <div className="space-y-4">
            <Input
              label={t('llmConfig.detail.configName')}
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
              placeholder={t('llmConfig.detail.configNamePlaceholder')}
            />

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {t('llmConfig.detail.provider')}
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
                  {t('llmConfig.detail.providerCannotChange')}
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
              <span className="ml-2 text-sm text-gray-700">{t('llmConfig.detail.setAsDefault')}</span>
            </label>
          </div>
        </Card>

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">{t('llmConfig.detail.connectionSettings')}</h2>
          <div className="space-y-4">
            <Input
              label={t('llmConfig.detail.apiUrl')}
              type="url"
              value={formData.apiUrl}
              onChange={(e) => setFormData({ ...formData, apiUrl: e.target.value })}
              placeholder={t('llmConfig.detail.apiUrlPlaceholder')}
            />

            <Input
              label={t('llmConfig.detail.apiKey')}
              type="password"
              value={formData.apiKey}
              onChange={(e) => setFormData({ ...formData, apiKey: e.target.value })}
              placeholder={t('llmConfig.detail.apiKeyPlaceholder')}
              required={formData.provider !== 'local'}
            />

            <Input
              label={t('llmConfig.detail.model')}
              value={formData.model}
              onChange={(e) => setFormData({ ...formData, model: e.target.value })}
              required
              placeholder={t('llmConfig.detail.modelPlaceholder')}
            />
          </div>
        </Card>

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">{t('llmConfig.detail.modelParameters')}</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {t('llmConfig.detail.temperature', { value: formData.temperature })}
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
                {t('llmConfig.detail.temperatureHelp')}
              </p>
            </div>

            <Input
              label={t('llmConfig.detail.maxTokens')}
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
                {t('llmConfig.detail.topP', { value: formData.topP })}
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
                {t('llmConfig.detail.topPHelp')}
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {t('llmConfig.detail.frequencyPenalty', { value: formData.frequencyPenalty })}
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
                {t('llmConfig.detail.frequencyPenaltyHelp')}
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {t('llmConfig.detail.presencePenalty', { value: formData.presencePenalty })}
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
                {t('llmConfig.detail.presencePenaltyHelp')}
              </p>
            </div>
          </div>
        </Card>

        <div className="flex items-center gap-3">
          <Button type="submit" disabled={saving}>
            {saving ? t('llmConfig.detail.saving') : isNew ? t('llmConfig.detail.createConfig') : t('llmConfig.detail.updateConfig')}
          </Button>
          <Button
            type="button"
            variant="secondary"
            onClick={() => navigate('/config/llm')}
          >
            {t('common.cancel')}
          </Button>
        </div>
      </form>
    </div>
  );
}
