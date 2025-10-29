import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { mcpService, type CreateMCPToolRequest, type UpdateMCPToolRequest, type ParameterSchema } from '../services/mcp.service';
import type { MCPTool } from '../types';
import { Button, Card, Input, Loader, Alert } from '../components/common';

export function MCPToolDetailPage() {
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
      setError(err.response?.data?.error || 'Failed to load tool');
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
        setSuccess('Tool created successfully');
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
        setSuccess('Tool updated successfully');
        await loadTool();
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to save tool');
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
            {isNew ? 'Create MCP Tool' : 'Configure MCP Tool'}
          </h1>
          <p className="mt-2 text-sm text-gray-600">
            {isNew
              ? 'Configure a new HTTP endpoint as an MCP tool'
              : 'Update tool configuration (creates a new version)'}
          </p>
        </div>
        {!isNew && (
          <Link to={`/mcp/tools/${id}/versions`}>
            <Button variant="secondary">View Versions</Button>
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
          <h2 className="text-lg font-medium text-gray-900 mb-4">Basic Information</h2>
          <div className="space-y-4">
            <Input
              label="Tool Name"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
              placeholder="my-api-tool"
            />

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Description
              </label>
              <textarea
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="Describe what this tool does..."
              />
            </div>
          </div>
        </Card>

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">HTTP Configuration</h2>
          <div className="space-y-4">
            <Input
              label="Endpoint URL"
              type="url"
              value={formData.endpoint}
              onChange={(e) => setFormData({ ...formData, endpoint: e.target.value })}
              required
              placeholder="https://api.example.com/endpoint"
              helpText="Use {paramName} for path parameters, e.g., /users/{userId}"
            />

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                HTTP Method
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
                label="Timeout (seconds)"
                type="number"
                min="1"
                max="300"
                value={formData.timeoutSeconds}
                onChange={(e) => setFormData({ ...formData, timeoutSeconds: parseInt(e.target.value) || 30 })}
                helpText="1-300 seconds"
              />
              <Input
                label="Retry Count"
                type="number"
                min="0"
                max="10"
                value={formData.retryCount}
                onChange={(e) => setFormData({ ...formData, retryCount: parseInt(e.target.value) || 0 })}
                helpText="0-10 retries"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Response Template (Optional)
              </label>
              <textarea
                value={formData.responseTemplate}
                onChange={(e) => setFormData({ ...formData, responseTemplate: e.target.value })}
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                placeholder="Use Handlebars syntax: {{ data.result }}"
              />
              <p className="mt-1 text-xs text-gray-500">
                Transform the API response using Handlebars template syntax
              </p>
            </div>
          </div>
        </Card>

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Headers</h2>
          <div className="space-y-4">
            <div className="flex gap-2">
              <Input
                placeholder="Header name"
                value={headerKey}
                onChange={(e) => setHeaderKey(e.target.value)}
                className="flex-1"
              />
              <Input
                placeholder="Header value"
                value={headerValue}
                onChange={(e) => setHeaderValue(e.target.value)}
                className="flex-1"
              />
              <Button type="button" onClick={addHeader} variant="secondary">
                Add
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
                      Remove
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </Card>

        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-medium text-gray-900">Parameters</h2>
            <Button type="button" onClick={addParameter} variant="secondary" size="sm">
              Add Parameter
            </Button>
          </div>

          <div className="space-y-4">
            {formData.parameters.map((param, index) => (
              <div key={index} className="p-4 border border-gray-200 rounded-lg space-y-3">
                <div className="grid grid-cols-3 gap-3">
                  <Input
                    label="Name"
                    value={param.name}
                    onChange={(e) => updateParameter(index, 'name', e.target.value)}
                    required
                    placeholder="parameterName"
                  />
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Type
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
                      Position
                    </label>
                    <select
                      value={param.position || 'body'}
                      onChange={(e) => updateParameter(index, 'position', e.target.value)}
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    >
                      <option value="body">Body</option>
                      <option value="header">Header</option>
                      <option value="path">Path</option>
                    </select>
                  </div>
                </div>

                <Input
                  label="Description"
                  value={param.description || ''}
                  onChange={(e) => updateParameter(index, 'description', e.target.value)}
                  placeholder="Parameter description"
                />

                <div className="grid grid-cols-2 gap-3">
                  <Input
                    label="Default Value (JSON)"
                    value={param.default_value ? JSON.stringify(param.default_value) : ''}
                    onChange={(e) => {
                      try {
                        const value = e.target.value ? JSON.parse(e.target.value) : undefined;
                        updateParameter(index, 'default_value', value);
                      } catch {
                        // Invalid JSON, ignore
                      }
                    }}
                    placeholder='e.g., "default" or 123'
                  />
                  <Input
                    label="Enum Values (JSON array)"
                    value={param.enum_values ? JSON.stringify(param.enum_values) : ''}
                    onChange={(e) => {
                      try {
                        const value = e.target.value ? JSON.parse(e.target.value) : undefined;
                        updateParameter(index, 'enum_values', value);
                      } catch {
                        // Invalid JSON, ignore
                      }
                    }}
                    placeholder='e.g., ["option1", "option2"]'
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
                    <span className="ml-2 text-sm text-gray-700">Required</span>
                  </label>
                  <button
                    type="button"
                    onClick={() => removeParameter(index)}
                    className="text-red-600 hover:text-red-700 text-sm"
                  >
                    Remove Parameter
                  </button>
                </div>
              </div>
            ))}

            {formData.parameters.length === 0 && (
              <p className="text-sm text-gray-500 text-center py-4">
                No parameters defined. Click "Add Parameter" to add one.
              </p>
            )}
          </div>
        </Card>

        {!isNew && (
          <Card>
            <h2 className="text-lg font-medium text-gray-900 mb-4">Change Log</h2>
            <textarea
              value={formData.changeLog}
              onChange={(e) => setFormData({ ...formData, changeLog: e.target.value })}
              rows={3}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Describe the changes made in this version..."
            />
          </Card>
        )}

        <div className="flex items-center gap-3">
          <Button type="submit" disabled={saving}>
            {saving ? 'Saving...' : isNew ? 'Create Tool' : 'Update Tool'}
          </Button>
          <Button
            type="button"
            variant="secondary"
            onClick={() => navigate('/mcp/tools')}
          >
            Cancel
          </Button>
        </div>
      </form>
    </div>
  );
}
