import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { sessionService } from '../services/session.service';
import type { ChatSession, ChatMessage } from '../types';
import { Card, Button, Loader, Alert } from '../components/common';

export function SessionDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [session, setSession] = useState<ChatSession | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);

  useEffect(() => {
    if (id) {
      loadSessionDetails();
    }
  }, [id]);

  const loadSessionDetails = async () => {
    if (!id) return;
    
    try {
      setLoading(true);
      setError(null);
      
      const [sessionData, messagesData] = await Promise.all([
        sessionService.getSessionById(id),
        sessionService.getSessionMessages(id),
      ]);
      
      setSession(sessionData);
      setMessages(messagesData.messages);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load session details');
    } finally {
      setLoading(false);
    }
  };

  const handleExport = async (format: 'json' | 'txt') => {
    if (!id) return;
    
    try {
      setExporting(true);
      const blob = await sessionService.exportSession(id, format);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `session-${id}.${format}`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to export session');
    } finally {
      setExporting(false);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  const getRoleColor = (role: string) => {
    switch (role) {
      case 'user':
        return 'bg-blue-100 text-blue-800';
      case 'assistant':
        return 'bg-green-100 text-green-800';
      case 'system':
        return 'bg-gray-100 text-gray-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getRoleIcon = (role: string) => {
    switch (role) {
      case 'user':
        return '👤';
      case 'assistant':
        return '🤖';
      case 'system':
        return '⚙️';
      default:
        return '💬';
    }
  };

  if (loading) {
    return (
      <div className="flex justify-center items-center py-12">
        <Loader />
      </div>
    );
  }

  if (error || !session) {
    return (
      <div className="space-y-4">
        <Alert type="error">{error || 'Session not found'}</Alert>
        <Button onClick={() => navigate('/sessions')}>Back to Session History</Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-semibold text-gray-900">Session Details</h1>
        <div className="flex gap-2">
          <Button
            variant="secondary"
            onClick={() => handleExport('json')}
            disabled={exporting}
          >
            {exporting ? 'Exporting...' : 'Export JSON'}
          </Button>
          <Button
            variant="secondary"
            onClick={() => handleExport('txt')}
            disabled={exporting}
          >
            {exporting ? 'Exporting...' : 'Export Text'}
          </Button>
          <Button variant="secondary" onClick={() => navigate('/sessions')}>
            Back to List
          </Button>
        </div>
      </div>

      <Card>
        <div className="p-6 space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Session ID
              </label>
              <p className="text-sm text-gray-900 font-mono">{session.id}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Title
              </label>
              <p className="text-sm text-gray-900">{session.title || 'Untitled Session'}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                User ID
              </label>
              <p className="text-sm text-gray-900 font-mono">{session.userId}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Tenant ID
              </label>
              <p className="text-sm text-gray-900 font-mono">{session.tenant_id}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Created At
              </label>
              <p className="text-sm text-gray-900">{formatDate(session.created_at)}</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-500 mb-1">
                Last Updated
              </label>
              <p className="text-sm text-gray-900">{formatDate(session.updated_at)}</p>
            </div>
          </div>

          {session.context && Object.keys(session.context).length > 0 && (
            <div>
              <label className="block text-sm font-medium text-gray-500 mb-2">
                Session Context
              </label>
              <div className="bg-gray-50 rounded-lg p-4">
                <pre className="text-xs text-gray-900 overflow-x-auto">
                  {JSON.stringify(session.context, null, 2)}
                </pre>
              </div>
            </div>
          )}
        </div>
      </Card>

      <Card>
        <div className="p-6">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-lg font-medium text-gray-900">
              Conversation ({messages.length} messages)
            </h2>
          </div>

          {messages.length === 0 ? (
            <p className="text-gray-500 text-center py-8">No messages in this session</p>
          ) : (
            <div className="space-y-4">
              {messages.map((message) => (
                <div
                  key={message.id}
                  className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
                >
                  <div
                    className={`max-w-3xl ${
                      message.role === 'user'
                        ? 'bg-blue-50 border-blue-200'
                        : message.role === 'assistant'
                        ? 'bg-green-50 border-green-200'
                        : 'bg-gray-50 border-gray-200'
                    } border rounded-lg p-4`}
                  >
                    <div className="flex items-center gap-2 mb-2">
                      <span className="text-lg">{getRoleIcon(message.role)}</span>
                      <span className={`px-2 py-1 text-xs font-medium rounded-full ${getRoleColor(message.role)}`}>
                        {message.role}
                      </span>
                      <span className="text-xs text-gray-500 ml-auto">
                        {formatDate(message.created_at)}
                      </span>
                    </div>
                    
                    <div className="text-sm text-gray-900 whitespace-pre-wrap break-words">
                      {message.content}
                    </div>

                    {message.metadata && Object.keys(message.metadata).length > 0 && (
                      <details className="mt-3">
                        <summary className="text-xs text-gray-500 cursor-pointer hover:text-gray-700">
                          View metadata
                        </summary>
                        <div className="mt-2 bg-white rounded p-2">
                          <pre className="text-xs text-gray-700 overflow-x-auto">
                            {JSON.stringify(message.metadata, null, 2)}
                          </pre>
                        </div>
                      </details>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </Card>

      <Card>
        <div className="p-6">
          <h2 className="text-lg font-medium text-gray-900 mb-4">Session Analytics</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-gray-50 rounded-lg p-4">
              <p className="text-sm font-medium text-gray-500">Total Messages</p>
              <p className="text-2xl font-semibold text-gray-900 mt-1">{messages.length}</p>
            </div>
            <div className="bg-gray-50 rounded-lg p-4">
              <p className="text-sm font-medium text-gray-500">User Messages</p>
              <p className="text-2xl font-semibold text-blue-600 mt-1">
                {messages.filter(m => m.role === 'user').length}
              </p>
            </div>
            <div className="bg-gray-50 rounded-lg p-4">
              <p className="text-sm font-medium text-gray-500">Assistant Messages</p>
              <p className="text-2xl font-semibold text-green-600 mt-1">
                {messages.filter(m => m.role === 'assistant').length}
              </p>
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
}
