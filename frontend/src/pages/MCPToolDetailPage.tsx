import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { mcpService, type CreateMCPToolRequest, type UpdateMCPToolRequest, type ParameterSchema } from '../services/mcp.service';
import type { MCPTool } from '../types';
import { Button, Card, Input, Loader, Alert } from '../components/common';

export function MCPToolDetailPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const isNew = id === 'new';

  const [tool, setTool] = useState<MCPTool | null>(null);
  const [loading, setLoading] = useState(!isNew);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [formData, setFormData] = useState({
    name: '',
    description: '',
    endpoint: '',
    method: 'POST' as 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH',
    headers: {} as Record<string, string>,
    parameters: [] as ParameterSchema[],
    timeoutSeconds: 30,
    retryCount: 3,
    responseTemplate: '',
    changeLog: '',
  });

  const [headerKey, setHeaderKey] = useState('');
  const [headerValue, setHeaderValue] = useState('');

  useEffect(() => {
    if (!isNew && id) {
      loadTool();
    }
  }, [id, isNew]);

  const loadTool = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await mcpService.getTool(id!);
      setTool(data);
      
      // Load the latest version config
      const versions = await mcpService.getToolVersions(id!);
      if (versions.length > 0) {
        const latestVersion = versions[0];
        setFormData({
          name: data.name,
          description: data.description || '',
          endpoint: latestVersion.config.HTTP.endpoint,
          method: latestVersion.config.HTTP.method,
          headers: latestVersion.config.HTTP.headers || {},
          parameters: latestVersion.config.HTTP.parameters || [],
          timeoutSeconds: latestVersion.config.HTTP.timeout_seconds || 30,
          retryCount: latestVersion.config.HTTP.retry_count || 3,
          responseTemplate: latestVersion.config.HTTP.response_template || '',
          changeLog: '',
        });
      }
    } catch (err: any) {console.log("xxx", err);
      setError(err.response?.data?.error || t('mcpTools.errors.loadToolFailed'));
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
      if (isNew) {
        const request: CreateMCPToolRequest = {
          name: formData.name,
          description: formData.description || undefined,
          config: {
            HTTP: {
              endpoint: formData.endpoint,
              method: formData.method,
              headers: Object.keys(formData.headers).length > 0 ? formData.headers : undefined,
              parameters: formData.parameters,
              timeout_seconds: formData.timeoutSeconds,
              retry_count: formData.retryCount,
              response_template: formData.responseTemplate || undefined,
            },
          },
        };
        const newTool = await mcpService.createTool(request);
        setSuccess(t('mcpTools.success.created'));
        setTimeout(() => navigate(`/mcp/tools/${newTool.id}`), 1500);
      } else {
        const request: UpdateMCPToolRequest = {
          name: formData.name,
          description: formData.description || undefined,
          config: {
            HTTP: {
              endpoint: formData.endpoint,
              method: formData.method,
              headers: Object.keys(formData.headers).length > 0 ? formData.headers : undefined,
              parameters: formData.parameters,
              timeout_seconds: formData.timeoutSeconds,
              retry_count: formData.retryCount,
              response_template: formData.responseTemplate || undefined,
            },
          },
          changeLog: formData.changeLog || undefined,
        };
        await mcpService.updateTool(id!, request);
        setSuccess(t('mcpTools.success.updated'));
        await loadTool();
      }
    } catch (err: any) {
      setError(err.response?.data?.error || t('mcpTools.errors.saveFailed'));
    } finally {
      setSaving(false);
    }
  };

  const addHeader = () => {
    if (headerKey && headerValue) {
      setFormData({
        ...formData,
        headers: { ...formData.headers, [headerKey]: headerValue },
      });
      setHeaderKey('');
      setHeaderValue('');
    }
  };

  const removeHeader = (key: string) => {
    const newHeaders = { ...formData.headers };
    delete newHeaders[key];
    setFormData({ ...formData, headers: newHeaders });
  };

  const addParameter = () => {
    setFormData({
      ...formData,
      parameters: [
        ...formData.parameters,
        { name: '', parameter_type: 'String', description: '', required: false, position: 'body' },
      ],
    });
  };

  const updateParameter = (index: number, field: keyof ParameterSchema, value: any) => {
    const newParameters = [...formData.parameters];
    newParameters[index] = { ...newParameters[index], [field]: value };
    setFormData({ ...formData, parameters: newParameters });
  };

  const removeParameter = (index: number) => {
    setFormData({
      ...formData,
      parameters: formData.parameters.filter((_, i) => i !== index),
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
            {isNew ? t('mcpTools.detail.createTitle') : t('mcpTools.detail.editTitle')}
          </h1>
          <p className="mt-2 text-sm text-gray-600">
            {isNew
              ? t('mcpTools.detail.createDescription')
              : t('mcpTools.detail.editDescription')}
          </p>
        </div>
        {!isNew && (
          <Link to={`/mcp/tools/${id}/versions`}>
            <Button variant="secondary">{t('mcpTools.detail.viewVersions')}</Button>
          </Link>
        )}
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
          <h2 className="text-lg font-medium text-gray-900 mb-4">{t('mcpTools.detail.basicInfo')}</h2>
          <div className="space-y-4">
            <Input
              label={t('mcpTools.detail.toolName')}
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
              placeholder={t('mcpTools.detail.toolNamePlaceholder')}
            />

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {t('mcpTools.detail.description')}
              </label>
              <textarea
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder={t('mcpTools.detail.descriptionPlaceholder')}
              />
            </div>
          </div>
        </Card>

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">{t('mcpTools.detail.httpConfig')}</h2>
          <div className="space-y-4">
            <Input
              label={t('mcpTools.detail.endpointUrl')}
              type="url"
              value={formData.endpoint}
              onChange={(e) => setFormData({ ...formData, endpoint: e.target.value })}
              required
              placeholder={t('mcpTools.detail.endpointUrlPlaceholder')}
              helpText={t('mcpTools.detail.endpointUrlHelp')}
            />

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {t('mcpTools.detail.httpMethod')}
              </label>
              <select
                value={formData.method}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    method: e.target.value as any,
                  })
                }
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              >
                <option value="GET">GET</option>
                <option value="POST">POST</option>
                <option value="PUT">PUT</option>
                <option value="DELETE">DELETE</option>
                <option value="PATCH">PATCH</option>
              </select>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <Input
                label={t('mcpTools.detail.timeout')}
                type="number"
                min="1"
                max="300"
                value={formData.timeoutSeconds}
                onChange={(e) => setFormData({ ...formData, timeoutSeconds: parseInt(e.target.value) || 30 })}
                helpText={t('mcpTools.detail.timeoutHelp')}
              />
              <Input
                label={t('mcpTools.detail.retryCount')}
                type="number"
                min="0"
                max="10"
                value={formData.retryCount}
                onChange={(e) => setFormData({ ...formData, retryCount: parseInt(e.target.value) || 0 })}
                helpText={t('mcpTools.detail.retryCountHelp')}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {t('mcpTools.detail.responseTemplate')}
              </label>
              <textarea
                value={formData.responseTemplate}
                onChange={(e) => setFormData({ ...formData, responseTemplate: e.target.value })}
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                placeholder={t('mcpTools.detail.responseTemplatePlaceholder')}
              />
              <p className="mt-1 text-xs text-gray-500">
                {t('mcpTools.detail.responseTemplateHelp')}
              </p>
            </div>
          </div>
        </Card>

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">{t('mcpTools.detail.headers')}</h2>
          <div className="space-y-4">
            <div className="flex gap-2">
              <Input
                placeholder={t('mcpTools.detail.headerName')}
                value={headerKey}
                onChange={(e) => setHeaderKey(e.target.value)}
                className="flex-1"
              />
              <Input
                placeholder={t('mcpTools.detail.headerValue')}
                value={headerValue}
                onChange={(e) => setHeaderValue(e.target.value)}
                className="flex-1"
              />
              <Button type="button" onClick={addHeader} variant="secondary">
                {t('mcpTools.detail.add')}
              </Button>
            </div>

            {Object.entries(formData.headers).length > 0 && (
              <div className="space-y-2">
                {Object.entries(formData.headers).map(([key, value]) => (
                  <div
                    key={key}
                    className="flex items-center justify-between p-3 bg-gray-50 rounded-lg"
                  >
                    <div className="flex-1">
                      <span className="font-medium text-sm">{key}:</span>{' '}
                      <span className="text-sm text-gray-600">{value}</span>
                    </div>
                    <button
                      type="button"
                      onClick={() => removeHeader(key)}
                      className="text-red-600 hover:text-red-700 text-sm"
                    >
                      {t('mcpTools.detail.remove')}
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </Card>

        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-medium text-gray-900">{t('mcpTools.detail.parameters')}</h2>
            <Button type="button" onClick={addParameter} variant="secondary" size="sm">
              {t('mcpTools.detail.addParameter')}
            </Button>
          </div>

          <div className="space-y-4">
            {formData.parameters.map((param, index) => (
              <div key={index} className="p-4 border border-gray-200 rounded-lg space-y-3">
                <div className="grid grid-cols-3 gap-3">
                  <Input
                    label={t('mcpTools.detail.parameterName')}
                    value={param.name}
                    onChange={(e) => updateParameter(index, 'name', e.target.value)}
                    required
                    placeholder={t('mcpTools.detail.parameterNamePlaceholder')}
                  />
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      {t('mcpTools.detail.parameterType')}
                    </label>
                    <select
                      value={param.parameter_type}
                      onChange={(e) => updateParameter(index, 'parameter_type', e.target.value)}
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    >
                      <option value="String">String</option>
                      <option value="Number">Number</option>
                      <option value="Boolean">Boolean</option>
                      <option value="Object">Object</option>
                      <option value="Array">Array</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      {t('mcpTools.detail.parameterPosition')}
                    </label>
                    <select
                      value={param.position || 'body'}
                      onChange={(e) => updateParameter(index, 'position', e.target.value)}
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    >
                      <option value="body">{t('mcpTools.detail.body')}</option>
                      <option value="header">{t('mcpTools.detail.header')}</option>
                      <option value="path">{t('mcpTools.detail.path')}</option>
                    </select>
                  </div>
                </div>

                <Input
                  label={t('mcpTools.detail.parameterDescription')}
                  value={param.description || ''}
                  onChange={(e) => updateParameter(index, 'description', e.target.value)}
                  placeholder={t('mcpTools.detail.parameterDescriptionPlaceholder')}
                />

                <div className="grid grid-cols-2 gap-3">
                  <Input
                    label={t('mcpTools.detail.defaultValue')}
                    value={param.default_value ? JSON.stringify(param.default_value) : ''}
                    onChange={(e) => {
                      try {
                        const value = e.target.value ? JSON.parse(e.target.value) : undefined;
                        updateParameter(index, 'default_value', value);
                      } catch {
                        // Invalid JSON, ignore
                      }
                    }}
                    placeholder={t('mcpTools.detail.defaultValuePlaceholder')}
                  />
                  <Input
                    label={t('mcpTools.detail.enumValues')}
                    value={param.enum_values ? JSON.stringify(param.enum_values) : ''}
                    onChange={(e) => {
                      try {
                        const value = e.target.value ? JSON.parse(e.target.value) : undefined;
                        updateParameter(index, 'enum_values', value);
                      } catch {
                        // Invalid JSON, ignore
                      }
                    }}
                    placeholder={t('mcpTools.detail.enumValuesPlaceholder')}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      checked={param.required}
                      onChange={(e) => updateParameter(index, 'required', e.target.checked)}
                      className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                    />
                    <span className="ml-2 text-sm text-gray-700">{t('mcpTools.detail.required')}</span>
                  </label>
                  <button
                    type="button"
                    onClick={() => removeParameter(index)}
                    className="text-red-600 hover:text-red-700 text-sm"
                  >
                    {t('mcpTools.detail.removeParameter')}
                  </button>
                </div>
              </div>
            ))}

            {formData.parameters.length === 0 && (
              <p className="text-sm text-gray-500 text-center py-4">
                {t('mcpTools.detail.noParameters')}
              </p>
            )}
          </div>
        </Card>

        {!isNew && (
          <Card>
            <h2 className="text-lg font-medium text-gray-900 mb-4">{t('mcpTools.detail.changeLog')}</h2>
            <textarea
              value={formData.changeLog}
              onChange={(e) => setFormData({ ...formData, changeLog: e.target.value })}
              rows={3}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder={t('mcpTools.detail.changeLogPlaceholder')}
            />
          </Card>
        )}

        <div className="flex items-center gap-3">
          <Button type="submit" disabled={saving}>
            {saving ? t('mcpTools.detail.saving') : isNew ? t('mcpTools.createTool') : t('mcpTools.detail.updateTool')}
          </Button>
          <Button
            type="button"
            variant="secondary"
            onClick={() => navigate('/mcp/tools')}
          >
            {t('common.cancel')}
          </Button>
        </div>
      </form>
    </div>
  );
}
