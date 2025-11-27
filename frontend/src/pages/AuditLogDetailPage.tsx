import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { auditService } from '../services/audit.service';
import type { AuditLog } from '../types';
import { Card, Button, Loader, Alert } from '../components/common';

export function AuditLogDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [log, setLog] = useState<AuditLog | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (id) {
      loadAuditLog();
    }
  }, [id]);

  const loadAuditLog = async () => {
    if (!id) return;
    
    try {
      setLoading(true);
      setError(null);
      const data = await auditService.getAuditLogById(id);
      setLog(data);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load audit log');
    } finally {
      setLoading(false);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  if (loading) {
    return (
      <div className="flex justify-center items-center py-12">
        <Loader />
      </div>
    );
  }

  if (error || !log) {
    return (
      <div className="space-y-4">
        <Alert type="error">{error || 'Audit log not found'}</Alert>
        <Button onClick={() => navigate('/audit/logs')}>Back to Audit Logs</Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-semibold text-gray-900">Audit Log Details</h1>
        <Button variant="secondary" onClick={() => navigate('/audit/logs')}>
          Back to List
        </Button>
      </div>

      <Card>
        <div className="p-6 space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Log ID
              </label>
              <p className="text-sm text-gray-900 font-mono">{log.id}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Timestamp
              </label>
              <p className="text-sm text-gray-900">{formatDate(log.created_at)}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Action
              </label>
              <p className="text-sm text-gray-900 font-semibold">{log.action}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Resource Type
              </label>
              <p className="text-sm text-gray-900">{log.resourceType}</p>
            </div>

            {log.resourceId && (
              <div>
                <label className="block text-sm font-medium text-gray-500 mb-1">
                  Resource ID
                </label>
                <p className="text-sm text-gray-900 font-mono">{log.resourceId}</p>
              </div>
            )}

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                User ID
              </label>
              <p className="text-sm text-gray-900 font-mono">{log.userId || 'System'}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Tenant ID
              </label>
              <p className="text-sm text-gray-900 font-mono">{log.tenant_id}</p>
            </div>

            {log.ipAddress && (
              <div>
                <label className="block text-sm font-medium text-gray-500 mb-1">
                  IP Address
                </label>
                <p className="text-sm text-gray-900">{log.ipAddress}</p>
              </div>
            )}
          </div>

          {log.userAgent && (
            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                User Agent
              </label>
              <p className="text-sm text-gray-900 break-all">{log.userAgent}</p>
            </div>
          )}

          {log.details && Object.keys(log.details).length > 0 && (
            <div>
              <label className="block text-sm font-medium text-gray-500 mb-2">
                Additional Details
              </label>
              <div className="bg-gray-50 rounded-lg p-4">
                <pre className="text-xs text-gray-900 overflow-x-auto">
                  {JSON.stringify(log.details, null, 2)}
                </pre>
              </div>
            </div>
          )}
        </div>
      </Card>
    </div>
  );
}
