import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { mcpService } from '../services/mcp.service';
import type { MCPTool, MCPToolVersion, TestToolResponse } from '../types';
import { Button, Card, Input, Loader, Alert } from '../components/common';

export function MCPToolTestPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();

  const [tool, setTool] = useState<MCPTool | null>(null);
  const [version, setVersion] = useState<MCPToolVersion | null>(null);
  const [loading, setLoading] = useState(true);
  const [testing, setTesting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<TestToolResponse | null>(null);

  const [parameters, setParameters] = useState<Record<string, any>>({});

  useEffect(() => {
    if (id) {
      loadTool();
    }
  }, [id]);

  const loadTool = async () => {
    try {
      setLoading(true);
      setError(null);

      const toolData = await mcpService.getTool(id!);
      setTool(toolData);

      const versions = await mcpService.getToolVersions(id!);
      if (versions.length > 0) {
        const latestVersion = versions[0];
        setVersion(latestVersion);

        // Initialize parameters with default values, properly typed
        // const initialParams: Record<string, any> = {};
        latestVersion.config.HTTP.parameters.forEach((param: any) => {
          if (param.default_value !== undefined) {
            // initialParams[param.name] = param.default_value;
            handleParameterChange(param.name, param.default_value, param.parameter_type.toLowerCase());
          } else {
            let value;
            // Set appropriate default based on type
            switch (param.parameter_type.toLowerCase()) {
              case 'number':
              case 'integer':
                value = '';
                break;
              case 'boolean':
                value = false;
                break;
              case 'object':
                value = {};
                break;
              case 'array':
                value = [];
                break;
              default:
                value = '';
            }
            handleParameterChange(param.name, value, param.parameter_type.toLowerCase());
          }
        });
        // setParameters(initialParams);
      }
    } catch (err: any) {
      setError(err.response?.data?.error || t('mcpTools.errors.loadToolFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleTest = async () => {
    setTesting(true);
    setError(null);
    setTestResult(null);

    try {
      const result = await mcpService.testTool(id!, parameters);
      setTestResult(result);
    } catch (err: any) {
      setError(err.response?.data?.error || t('mcpTools.errors.testFailed'));
      setTestResult({
        result: null,
        executionTime: 0,
        success: false,
        error: err.response?.data?.error || t('mcpTools.errors.testFailed'),
      });
    } finally {
      setTesting(false);
    }
  };

  const handleParameterChange = (name: string, value: any, paramType?: string) => {
    let convertedValue = value;

    // Convert value to the correct type based on parameter type
    if (paramType) {
      switch (paramType) {
        case 'number':
          convertedValue = value === '' ? '' : parseFloat(value);
          break;
        case 'integer':
          convertedValue = value === '' ? '' : parseInt(value, 10);
          break;
        case 'boolean':
          convertedValue = value === 'true' || value === true;
          break;
        case 'string':
          convertedValue = String(value);
          break;
        case 'object':
        case 'array':
          // Already handled in the textarea onChange
          convertedValue = value;
          break;
        default:
          convertedValue = value;
      }
    }

    setParameters({ ...parameters, [name]: convertedValue });
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader size="lg" />
      </div>
    );
  }

  if (!tool || !version) {
    return (
      <Alert type="error">
        {t('mcpTools.errors.toolNotFound')}
      </Alert>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-semibold text-gray-900">{t('mcpTools.test.title')}</h1>
          <p className="mt-2 text-sm text-gray-600">
            {t('mcpTools.test.description')}
          </p>
        </div>
        <Link to={`/mcp/tools/${id}`}>
          <Button variant="secondary">{t('mcpTools.test.backToConfig')}</Button>
        </Link>
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      <Card>
        <h2 className="text-lg font-medium text-gray-900 mb-4">{t('mcpTools.test.toolInfo')}</h2>
        <div className="space-y-2 text-sm">
          <div className="flex">
            <span className="font-medium w-32">{t('mcpTools.test.name')}:</span>
            <span className="text-gray-600">{tool.name}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">{t('mcpTools.version')}:</span>
            <span className="text-gray-600">{tool.current_version}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">{t('mcpTools.test.endpoint')}:</span>
            <span className="text-gray-600 break-all">{version.config.HTTP.endpoint}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">{t('mcpTools.test.method')}:</span>
            <span className="text-gray-600">{version.config.HTTP.method}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">{t('mcpTools.test.status')}:</span>
            <span
              className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${tool.status.toLowerCase() === 'active'
                  ? 'bg-green-100 text-green-800'
                  : 'bg-gray-100 text-gray-800'
                }`}
            >
              {tool.status}
            </span>
          </div>
        </div>
      </Card>

      <Card>
        <h2 className="text-lg font-medium text-gray-900 mb-4">{t('mcpTools.test.testParameters')}</h2>
        <div className="space-y-4">
          {version.config.HTTP.parameters.length === 0 ? (
            <p className="text-sm text-gray-500">{t('mcpTools.test.noParameters')}</p>
          ) : (
            version.config.HTTP.parameters.map((param: any) => (
              <div key={param.name}>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  {param.name}
                  {param.required && <span className="text-red-500 ml-1">*</span>}
                </label>
                {param.description && (
                  <p className="text-xs text-gray-500 mb-2">{param.description}</p>
                )}
                {param.parameter_type.toLowerCase() === 'boolean' ? (
                  <select
                    value={parameters[param.name]?.toString() || 'false'}
                    onChange={(e) =>
                      handleParameterChange(param.name, e.target.value, 'boolean')
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  >
                    <option value="true">true</option>
                    <option value="false">false</option>
                  </select>
                ) : param.parameter_type.toLowerCase() === 'number' || param.parameter_type.toLowerCase() === 'integer' ? (
                  <Input
                    type="number"
                    value={parameters[param.name] || ''}
                    onChange={(e) =>
                      handleParameterChange(param.name, e.target.value, param.parameter_type.toLowerCase())
                    }
                    required={param.required}
                    step={param.parameter_type.toLowerCase() === 'integer' ? '1' : 'any'}
                  />
                ) : param.parameter_type.toLowerCase() === 'object' || param.parameter_type.toLowerCase() === 'array' ? (
                  <textarea
                    value={
                      typeof parameters[param.name] === 'string'
                        ? parameters[param.name]
                        : JSON.stringify(parameters[param.name] || {}, null, 2)
                    }
                    onChange={(e) => {
                      try {
                        const parsed = JSON.parse(e.target.value);
                        handleParameterChange(param.name, parsed, param.parameter_type.toLowerCase());
                      } catch {
                        handleParameterChange(param.name, e.target.value, param.parameter_type.toLowerCase());
                      }
                    }}
                    rows={4}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                    placeholder={param.parameter_type.toLowerCase() === 'object' ? '{}' : '[]'}
                  />
                ) : (
                  <Input
                    type="text"
                    value={parameters[param.name] || ''}
                    onChange={(e) => handleParameterChange(param.name, e.target.value, 'string')}
                    required={param.required}
                  />
                )}
              </div>
            ))
          )}
        </div>

        <div className="mt-6">
          <Button onClick={handleTest} disabled={testing || tool.status.toLowerCase() !== 'active'}>
            {testing ? t('mcpTools.test.testing') : t('mcpTools.test.runTest')}
          </Button>
          {tool.status.toLowerCase() !== 'active' && (
            <p className="mt-2 text-sm text-amber-600">
              {t('mcpTools.test.mustBeActive')}
            </p>
          )}
        </div>
      </Card>

      {testResult && (
        <Card>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-medium text-gray-900">{t('mcpTools.test.testResult')}</h2>
            <div className="flex items-center gap-4">
              <span
                className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${testResult.success
                    ? 'bg-green-100 text-green-800'
                    : 'bg-red-100 text-red-800'
                  }`}
              >
                {testResult.success ? t('mcpTools.test.success') : t('mcpTools.test.failed')}
              </span>
              <span className="text-sm text-gray-600">
                {testResult.executionTime}{t('mcpTools.test.executionTime')}
              </span>
            </div>
          </div>

          {testResult.error && (
            <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-sm text-red-800">{testResult.error}</p>
            </div>
          )}

          <div>
            <h3 className="text-sm font-medium text-gray-700 mb-2">{t('mcpTools.test.response')}</h3>
            <pre className="p-4 bg-gray-50 rounded-lg overflow-x-auto text-sm">
              {JSON.stringify(testResult.result, null, 2)}
            </pre>
          </div>
        </Card>
      )}
    </div>
  );
}
