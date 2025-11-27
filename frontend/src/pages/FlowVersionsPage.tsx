import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { flowService } from '../services/flow.service';
import type { Flow, FlowVersion } from '../types';
import { Button, Card, Loader, Alert, Modal } from '../components/common';

export const FlowVersionsPage = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [flow, setFlow] = useState<Flow | null>(null);
  const [versions, setVersions] = useState<FlowVersion[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedVersions, setSelectedVersions] = useState<[number, number] | null>(null);
  const [showRollbackModal, setShowRollbackModal] = useState(false);
  const [rollbackVersion, setRollbackVersion] = useState<number | null>(null);
  const [rolling, setRolling] = useState(false);

  useEffect(() => {
    if (id) {
      loadVersions();
    }
  }, [id]);

  const loadVersions = async () => {
    if (!id) return;

    try {
      setLoading(true);
      setError(null);
      const [flowData, versionsData] = await Promise.all([
        flowService.getFlowById(id),
        flowService.getFlowVersions(id),
      ]);
      setFlow(flowData);
      setVersions(versionsData.sort((a, b) => b.version - a.version));
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load versions');
    } finally {
      setLoading(false);
    }
  };

  const handleRollback = async () => {
    if (!id || !rollbackVersion) return;

    try {
      setRolling(true);
      await flowService.rollbackFlow(id, rollbackVersion);
      setShowRollbackModal(false);
      setRollbackVersion(null);
      await loadVersions();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to rollback version');
    } finally {
      setRolling(false);
    }
  };

  const handleCompare = (v1: number, v2: number) => {
    setSelectedVersions([Math.min(v1, v2), Math.max(v1, v2)]);
  };

  const getVersionDiff = () => {
    if (!selectedVersions) return null;

    const [v1, v2] = selectedVersions;
    const version1 = versions.find((v) => v.version === v1);
    const version2 = versions.find((v) => v.version === v2);

    if (!version1 || !version2) return null;

    return { version1, version2 };
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader />
      </div>
    );
  }

  if (!flow) {
    return (
      <div className="text-center py-12">
        <p className="text-gray-500">Flow not found</p>
        <Button className="mt-4" onClick={() => navigate('/flows')}>
          Back to Flows
        </Button>
      </div>
    );
  }

  const diff = getVersionDiff();

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <Button variant="secondary" size="sm" onClick={() => navigate(`/flows/${id}`)}>
            ← Back to Flow
          </Button>
          <h1 className="text-2xl font-semibold text-gray-900 mt-2">Version History</h1>
          <p className="text-gray-600 mt-1">{flow.name}</p>
        </div>
        {selectedVersions && (
          <Button variant="secondary" onClick={() => setSelectedVersions(null)}>
            Clear Comparison
          </Button>
        )}
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {diff && (
        <Card>
          <h2 className="text-lg font-semibold text-gray-900 mb-4">
            Comparing v{diff.version1.version} with v{diff.version2.version}
          </h2>
          <div className="grid grid-cols-2 gap-6">
            <div>
              <h3 className="text-sm font-medium text-gray-700 mb-2">
                Version {diff.version1.version}
              </h3>
              <div className="bg-gray-50 p-4 rounded-lg">
                <p className="text-xs text-gray-500 mb-2">
                  {new Date(diff.version1.created_at).toLocaleString()}
                </p>
                {diff.version1.changeLog && (
                  <p className="text-sm text-gray-700 mb-3">{diff.version1.changeLog}</p>
                )}
                <pre className="text-xs overflow-x-auto">
                  {JSON.stringify(diff.version1.definition, null, 2)}
                </pre>
              </div>
            </div>
            <div>
              <h3 className="text-sm font-medium text-gray-700 mb-2">
                Version {diff.version2.version}
              </h3>
              <div className="bg-gray-50 p-4 rounded-lg">
                <p className="text-xs text-gray-500 mb-2">
                  {new Date(diff.version2.created_at).toLocaleString()}
                </p>
                {diff.version2.changeLog && (
                  <p className="text-sm text-gray-700 mb-3">{diff.version2.changeLog}</p>
                )}
                <pre className="text-xs overflow-x-auto">
                  {JSON.stringify(diff.version2.definition, null, 2)}
                </pre>
              </div>
            </div>
          </div>
        </Card>
      )}

      <Card>
        <h2 className="text-lg font-semibold text-gray-900 mb-4">All Versions</h2>
        <div className="space-y-3">
          {versions.map((version) => (
            <div
              key={version.id}
              className={`p-4 rounded-lg border-2 transition-colors ${
                version.version === flow.current_version
                  ? 'border-blue-500 bg-blue-50'
                  : 'border-gray-200 bg-white hover:border-gray-300'
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-3 mb-2">
                    <h3 className="text-lg font-semibold text-gray-900">
                      Version {version.version}
                    </h3>
                    {version.version === flow.current_version && (
                      <span className="inline-flex px-2 py-1 text-xs font-semibold text-blue-600 bg-blue-100 rounded-full">
                        Current
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-gray-600 mb-2">
                    {new Date(version.created_at).toLocaleString()}
                  </p>
                  {version.changeLog && (
                    <p className="text-sm text-gray-700 mb-3">{version.changeLog}</p>
                  )}
                  <details className="mt-3">
                    <summary className="text-sm text-blue-600 cursor-pointer hover:text-blue-700">
                      View Definition
                    </summary>
                    <pre className="mt-2 p-3 bg-gray-50 rounded text-xs overflow-x-auto">
                      {JSON.stringify(version.definition, null, 2)}
                    </pre>
                  </details>
                </div>
                <div className="flex gap-2 ml-4">
                  {version.version !== flow.current_version && (
                    <Button
                      size="sm"
                      variant="secondary"
                      onClick={() => {
                        setRollbackVersion(version.version);
                        setShowRollbackModal(true);
                      }}
                    >
                      Rollback
                    </Button>
                  )}
                  {!selectedVersions && (
                    <Button
                      size="sm"
                      variant="secondary"
                      onClick={() => handleCompare(version.version, flow.current_version)}
                      disabled={version.version === flow.current_version}
                    >
                      Compare
                    </Button>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      </Card>

      <Modal
        isOpen={showRollbackModal}
        onClose={() => setShowRollbackModal(false)}
        title="Confirm Rollback"
      >
        <div className="space-y-4">
          <p className="text-gray-700">
            Are you sure you want to rollback to version {rollbackVersion}? This will create a new
            version with the configuration from version {rollbackVersion}.
          </p>
          <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
            <p className="text-sm text-yellow-800">
              <strong>Note:</strong> This action will not delete any versions. A new version will
              be created based on the selected version.
            </p>
          </div>
          <div className="flex gap-3 justify-end">
            <Button variant="secondary" onClick={() => setShowRollbackModal(false)}>
              Cancel
            </Button>
            <Button onClick={handleRollback} disabled={rolling}>
              {rolling ? 'Rolling back...' : 'Confirm Rollback'}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
};
