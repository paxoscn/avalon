import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { mcpService } from '../services/mcp.service';
import type { MCPTool, MCPToolVersion } from '../types';
import { Button, Card, Loader, Alert } from '../components/common';

export function MCPToolVersionsPage() {
  const { id } = useParams<{ id: string }>();
  
  const [tool, setTool] = useState<MCPTool | null>(null);
  const [versions, setVersions] = useState<MCPToolVersion[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [rollingBack, setRollingBack] = useState(false);

  useEffect(() => {
    if (id) {
      loadData();
    }
  }, [id]);

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const [toolData, versionsData] = await Promise.all([
        mcpService.getTool(id!),
        mcpService.getToolVersions(id!),
      ]);
      
      setTool(toolData);
      setVersions(versionsData);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load versions');
    } finally {
      setLoading(false);
    }
  };

  const handleRollback = async (version: number) => {
    if (!confirm(`Are you sure you want to rollback to version ${version}?`)) {
      return;
    }

    try {
      setRollingBack(true);
      setError(null);
      setSuccess(null);
      
      await mcpService.rollbackTool(id!, version);
      setSuccess(`Successfully rolled back to version ${version}`);
      await loadData();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to rollback version');
    } finally {
      setRollingBack(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader size="lg" />
      </div>
    );
  }

  if (!tool) {
    return <Alert type="error">Tool not found</Alert>;
  }

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-semibold text-gray-900">Tool Versions</h1>
          <p className="mt-2 text-sm text-gray-600">
            Version history for {tool.name}
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

      {success && (
        <Alert type="success" onClose={() => setSuccess(null)}>
          {success}
        </Alert>
      )}

      <Card>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-medium text-gray-900">Current Version</h2>
          <span className="text-2xl font-bold text-blue-600">
            v{tool.current_version}
          </span>
        </div>
      </Card>

      {versions.length === 0 ? (
        <Card>
          <p className="text-center text-gray-500 py-8">No versions found</p>
        </Card>
      ) : (
        <div className="space-y-4">
          {versions.map((version) => (
            <Card key={version.id} className="hover:shadow-md transition-shadow">
              <div className="space-y-4">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3">
                      <h3 className="text-lg font-medium text-gray-900">
                        Version {version.version}
                      </h3>
                      {version.version === tool.current_version && (
                        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                          Current
                        </span>
                      )}
                    </div>
                    <p className="mt-1 text-sm text-gray-500">
                      Created {new Date(version.created_at).toLocaleString()}
                    </p>
                  </div>
                  {version.version !== tool.current_version && (
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => handleRollback(version.version)}
                      disabled={rollingBack}
                    >
                      Rollback
                    </Button>
                  )}
                </div>

                {version.changeLog && (
                  <div className="p-3 bg-gray-50 rounded-lg">
                    <h4 className="text-sm font-medium text-gray-700 mb-1">
                      Change Log:
                    </h4>
                    <p className="text-sm text-gray-600">{version.changeLog}</p>
                  </div>
                )}

                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="font-medium text-gray-700">Endpoint:</span>
                    <p className="text-gray-600 break-all mt-1">
                      {version.config.endpoint}
                    </p>
                  </div>
                  <div>
                    <span className="font-medium text-gray-700">Method:</span>
                    <p className="text-gray-600 mt-1">{version.config.method}</p>
                  </div>
                </div>

                {version.config.headers && Object.keys(version.config.headers).length > 0 && (
                  <div>
                    <h4 className="text-sm font-medium text-gray-700 mb-2">Headers:</h4>
                    <div className="space-y-1">
                      {Object.entries(version.config.headers).map(([key, value]) => (
                        <div
                          key={key}
                          className="text-sm text-gray-600 font-mono bg-gray-50 px-3 py-1 rounded"
                        >
                          {key}: {value}
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {version.config.parameters.length > 0 && (
                  <div>
                    <h4 className="text-sm font-medium text-gray-700 mb-2">
                      Parameters ({version.config.parameters.length}):
                    </h4>
                    <div className="space-y-2">
                      {version.config.parameters.map((param, idx) => (
                        <div
                          key={idx}
                          className="p-3 bg-gray-50 rounded-lg text-sm"
                        >
                          <div className="flex items-center gap-2">
                            <span className="font-medium text-gray-900">
                              {param.name}
                            </span>
                            <span className="text-gray-500">({param.type})</span>
                            {param.required && (
                              <span className="text-red-500 text-xs">required</span>
                            )}
                          </div>
                          {param.description && (
                            <p className="text-gray-600 mt-1">{param.description}</p>
                          )}
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
