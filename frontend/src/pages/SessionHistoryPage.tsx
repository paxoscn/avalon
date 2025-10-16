import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { sessionService, type SessionFilters, type SessionStats } from '../services/session.service';
import type { ChatSession } from '../types';
import { Card, Button, Input, Loader, Alert } from '../components/common';

export function SessionHistoryPage() {
  const navigate = useNavigate();
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [stats, setStats] = useState<SessionStats | null>(null);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  const [filters, setFilters] = useState<SessionFilters>({
    page: 1,
    limit: 20,
  });

  const [searchTerm, setSearchTerm] = useState('');
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');

  useEffect(() => {
    loadSessions();
    loadStats();
  }, [filters]);

  const loadSessions = async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await sessionService.getSessions(filters);
      setSessions(response.sessions);
      setTotal(response.total);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load session history');
    } finally {
      setLoading(false);
    }
  };

  const loadStats = async () => {
    try {
      const statsData = await sessionService.getSessionStats({
        startDate: filters.startDate,
        endDate: filters.endDate,
      });
      setStats(statsData);
    } catch (err: any) {
      console.error('Failed to load stats:', err);
    }
  };

  const handleSearch = () => {
    setFilters({
      ...filters,
      search: searchTerm || undefined,
      startDate: startDate || undefined,
      endDate: endDate || undefined,
      page: 1,
    });
  };

  const handleReset = () => {
    setSearchTerm('');
    setStartDate('');
    setEndDate('');
    setFilters({ page: 1, limit: 20 });
  };

  const handlePageChange = (newPage: number) => {
    setFilters({ ...filters, page: newPage });
  };

  const handleDelete = async (sessionId: string) => {
    if (!confirm('Are you sure you want to delete this session?')) {
      return;
    }

    try {
      await sessionService.deleteSession(sessionId);
      await loadSessions();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to delete session');
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  const totalPages = Math.ceil(total / (filters.limit || 20));

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-semibold text-gray-900">Session History</h1>
      </div>

      {error && <Alert variant="error" message={error} onClose={() => setError(null)} />}

      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <Card>
            <div className="p-6">
              <p className="text-sm font-medium text-gray-500">Total Sessions</p>
              <p className="text-3xl font-semibold text-gray-900 mt-2">
                {stats.totalSessions}
              </p>
            </div>
          </Card>
          <Card>
            <div className="p-6">
              <p className="text-sm font-medium text-gray-500">Total Messages</p>
              <p className="text-3xl font-semibold text-blue-600 mt-2">
                {stats.totalMessages}
              </p>
            </div>
          </Card>
          <Card>
            <div className="p-6">
              <p className="text-sm font-medium text-gray-500">Avg Messages/Session</p>
              <p className="text-3xl font-semibold text-green-600 mt-2">
                {stats.averageMessagesPerSession.toFixed(1)}
              </p>
            </div>
          </Card>
          <Card>
            <div className="p-6">
              <p className="text-sm font-medium text-gray-500">Active Users</p>
              <p className="text-3xl font-semibold text-purple-600 mt-2">
                {stats.activeUsers}
              </p>
            </div>
          </Card>
        </div>
      )}

      <Card>
        <div className="p-6 space-y-4">
          <h2 className="text-lg font-medium text-gray-900">Filters</h2>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Search
              </label>
              <Input
                type="text"
                placeholder="Search by title or content..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Start Date
              </label>
              <Input
                type="datetime-local"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                End Date
              </label>
              <Input
                type="datetime-local"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
              />
            </div>
          </div>

          <div className="flex gap-2">
            <Button onClick={handleSearch}>Apply Filters</Button>
            <Button variant="secondary" onClick={handleReset}>Reset</Button>
          </div>
        </div>
      </Card>

      <Card>
        {loading ? (
          <div className="flex justify-center items-center py-12">
            <Loader />
          </div>
        ) : sessions.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-gray-500">No session history found</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Title
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    User
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Created At
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Last Updated
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {sessions.map((session) => (
                  <tr key={session.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm font-medium text-gray-900">
                        {session.title || 'Untitled Session'}
                      </div>
                      <div className="text-xs text-gray-500 font-mono">{session.id}</div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {session.userId}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {formatDate(session.created_at)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {formatDate(session.updated_at)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm space-x-2">
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => navigate(`/sessions/${session.id}`)}
                      >
                        View
                      </Button>
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => handleDelete(session.id)}
                      >
                        Delete
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      {!loading && totalPages > 1 && (
        <div className="flex justify-between items-center">
          <p className="text-sm text-gray-700">
            Showing {((filters.page || 1) - 1) * (filters.limit || 20) + 1} to{' '}
            {Math.min((filters.page || 1) * (filters.limit || 20), total)} of {total} results
          </p>
          <div className="flex gap-2">
            <Button
              variant="secondary"
              onClick={() => handlePageChange((filters.page || 1) - 1)}
              disabled={filters.page === 1}
            >
              Previous
            </Button>
            <span className="px-4 py-2 text-sm text-gray-700">
              Page {filters.page} of {totalPages}
            </span>
            <Button
              variant="secondary"
              onClick={() => handlePageChange((filters.page || 1) + 1)}
              disabled={filters.page === totalPages}
            >
              Next
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
