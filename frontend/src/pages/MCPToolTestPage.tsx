import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { mcpService } from '../services/mcp.service';
import type { MCPTool, MCPToolVersion, TestToolResponse } from '../types';
import { Button, Card, Input, Loader, Alert } from '../components/common';

export function MCPToolTestPage() {
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
        
        // Initialize parameters with default values
        const initialParams: Record<string, any> = {};
        latestVersion.config.HTTP.parameters.forEach((param: any) => {
          if (param.default_value !== undefined) {
            initialParams[param.name] = param.default_value;
          } else {
            initialParams[param.name] = '';
          }
        });
        setParameters(initialParams);
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load tool');
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
      setError(err.response?.data?.error || 'Tool test failed');
      setTestResult({
        result: null,
        executionTime: 0,
        success: false,
        error: err.response?.data?.error || 'Tool test failed',
      });
    } finally {
      setTesting(false);
    }
  };

  const handleParameterChange = (name: string, value: any) => {
    setParameters({ ...parameters, [name]: value });
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
        Tool not found or has no versions configured.
      </Alert>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-semibold text-gray-900">Test MCP Tool</h1>
          <p className="mt-2 text-sm text-gray-600">
            Test the tool with different parameters
          </p>
        </div>
        <Link to={`/mcp/tools/${id}`}>
          <Button variant="secondary">Back to Configuration</Button>
        </Link>
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      <Card>
        <h2 className="text-lg font-medium text-gray-900 mb-4">Tool Information</h2>
        <div className="space-y-2 text-sm">
          <div className="flex">
            <span className="font-medium w-32">Name:</span>
            <span className="text-gray-600">{tool.name}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">Version:</span>
            <span className="text-gray-600">{tool.current_version}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">Endpoint:</span>
            <span className="text-gray-600 break-all">{version.config.HTTP.endpoint}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">Method:</span>
            <span className="text-gray-600">{version.config.HTTP.method}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">Status:</span>
            <span
              className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                tool.status.toLowerCase() === 'active'
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
        <h2 className="text-lg font-medium text-gray-900 mb-4">Test Parameters</h2>
        <div className="space-y-4">
          {version.config.HTTP.parameters.length === 0 ? (
            <p className="text-sm text-gray-500">This tool has no parameters.</p>
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
                {param.type === 'boolean' ? (
                  <select
                    value={parameters[param.name]?.toString() || 'false'}
                    onChange={(e) =>
                      handleParameterChange(param.name, e.target.value === 'true')
                    }
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  >
                    <option value="true">true</option>
                    <option value="false">false</option>
                  </select>
                ) : param.type === 'number' ? (
                  <Input
                    type="number"
                    value={parameters[param.name] || ''}
                    onChange={(e) =>
                      handleParameterChange(
                        param.name,
                        e.target.value ? parseFloat(e.target.value) : ''
                      )
                    }
                    required={param.required}
                  />
                ) : param.type === 'object' || param.type === 'array' ? (
                  <textarea
                    value={
                      typeof parameters[param.name] === 'string'
                        ? parameters[param.name]
                        : JSON.stringify(parameters[param.name] || {}, null, 2)
                    }
                    onChange={(e) => {
                      try {
                        const parsed = JSON.parse(e.target.value);
                        handleParameterChange(param.name, parsed);
                      } catch {
                        handleParameterChange(param.name, e.target.value);
                      }
                    }}
                    rows={4}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                    placeholder={param.type === 'object' ? '{}' : '[]'}
                  />
                ) : (
                  <Input
                    type="text"
                    value={parameters[param.name] || ''}
                    onChange={(e) => handleParameterChange(param.name, e.target.value)}
                    required={param.required}
                  />
                )}
              </div>
            ))
          )}
        </div>

        <div className="mt-6">
          <Button onClick={handleTest} disabled={testing || tool.status.toLowerCase() !== 'active'}>
            {testing ? 'Testing...' : 'Run Test'}
          </Button>
          {tool.status.toLowerCase() !== 'active' && (
            <p className="mt-2 text-sm text-amber-600">
              Tool must be active to run tests
            </p>
          )}
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

          <div>
            <h3 className="text-sm font-medium text-gray-700 mb-2">Response:</h3>
            <pre className="p-4 bg-gray-50 rounded-lg overflow-x-auto text-sm">
              {JSON.stringify(testResult.result, null, 2)}
            </pre>
          </div>
        </Card>
      )}
    </div>
  );
}
