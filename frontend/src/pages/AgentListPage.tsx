import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { agentService } from '../services/agent.service';
import type { Agent } from '../types';
import { Button, Card, Loader, Alert } from '../components/common';

export function AgentListPage() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);

  useEffect(() => {
    loadAgents();
  }, [page]);

  const loadAgents = async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await agentService.listAgents({ page, page_size: 12 });
      setAgents(response.items);
      setTotalPages(response.total_pages);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load agents');
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this agent?')) {
      return;
    }

    try {
      await agentService.deleteAgent(id);
      await loadAgents();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to delete agent');
    }
  };

  const handleCopy = async (id: string) => {
    try {
      await agentService.copyAgent(id);
      await loadAgents();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to copy agent');
    }
  };

  const handleEmploy = async (id: string) => {
    try {
      await agentService.employAgent(id);
      alert('Agent employed successfully!');
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to employ agent');
    }
  };

  if (loading && page === 1) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader size="lg" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-semibold text-gray-900">Agents</h1>
          <p className="mt-2 text-sm text-gray-600">
            Manage AI agents with custom capabilities and knowledge
          </p>
        </div>
        <Link to="/agents/new">
          <Button>Create Agent</Button>
        </Link>
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {agents.length === 0 ? (
        <Card>
          <div className="text-center py-12">
            <svg
              className="mx-auto h-12 w-12 text-gray-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
              />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-gray-900">No agents</h3>
            <p className="mt-1 text-sm text-gray-500">
              Get started by creating a new AI agent.
            </p>
            <div className="mt-6">
              <Link to="/agents/new">
                <Button>Create Agent</Button>
              </Link>
            </div>
          </div>
        </Card>
      ) : (
        <>
          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
            {agents.map((agent) => (
              <Card key={agent.id} className="hover:shadow-lg transition-shadow">
                <div className="space-y-4">
                  <div className="flex items-start justify-between">
                    <div className="flex items-center gap-3 flex-1 min-w-0">
                      {agent.avatar ? (
                        <img
                          src={agent.avatar}
                          alt={agent.name}
                          className="w-12 h-12 rounded-full object-cover"
                        />
                      ) : (
                        <div className="w-12 h-12 rounded-full bg-gradient-to-br from-blue-400 to-purple-500 flex items-center justify-center text-white text-xl font-bold">
                          {agent.name.charAt(0).toUpperCase()}
                        </div>
                      )}
                      <div className="flex-1 min-w-0">
                        <h3 className="text-lg font-medium text-gray-900 truncate">
                          {agent.name}
                        </h3>
                        <p className="text-sm text-gray-500">
                          {new Date(agent.created_at).toLocaleDateString()}
                        </p>
                      </div>
                    </div>
                  </div>

                  <div className="text-sm text-gray-600">
                    <p className="line-clamp-2">{agent.system_prompt}</p>
                  </div>

                  <div className="flex flex-wrap gap-2 text-xs">
                    {agent.knowledge_base_ids != null && agent.knowledge_base_ids.length > 0 && (
                      <span className="px-2 py-1 bg-blue-100 text-blue-700 rounded">
                        {agent.knowledge_base_ids.length} KB
                      </span>
                    )}
                    {agent.mcp_tool_ids != null && agent.mcp_tool_ids.length > 0 && (
                      <span className="px-2 py-1 bg-green-100 text-green-700 rounded">
                        {agent.mcp_tool_ids.length} Tools
                      </span>
                    )}
                    {agent.flow_ids != null && agent.flow_ids.length > 0 && (
                      <span className="px-2 py-1 bg-purple-100 text-purple-700 rounded">
                        {agent.flow_ids.length} Flows
                      </span>
                    )}
                  </div>

                  <div className="flex items-center gap-2 pt-4 border-t border-gray-200">
                    <Link to={`/agents/${agent.id}`} className="flex-1">
                      <Button variant="secondary" className="w-full">
                        Edit
                      </Button>
                    </Link>
                    <Button
                      variant="secondary"
                      onClick={() => handleEmploy(agent.id)}
                      className="flex-1"
                    >
                      Employ
                    </Button>
                  </div>

                  <div className="flex items-center gap-2">
                    <Button
                      variant="secondary"
                      onClick={() => handleCopy(agent.id)}
                      className="flex-1"
                    >
                      Copy
                    </Button>
                    <Button
                      variant="secondary"
                      onClick={() => handleDelete(agent.id)}
                      className="flex-1 text-red-600 hover:text-red-700"
                    >
                      Delete
                    </Button>
                  </div>
                </div>
              </Card>
            ))}
          </div>

          {totalPages > 1 && (
            <div className="flex items-center justify-center gap-2 mt-6">
              <Button
                variant="secondary"
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                disabled={page === 1}
              >
                Previous
              </Button>
              <span className="text-sm text-gray-600">
                Page {page} of {totalPages}
              </span>
              <Button
                variant="secondary"
                onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                disabled={page === totalPages}
              >
                Next
              </Button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
