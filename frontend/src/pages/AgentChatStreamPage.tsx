import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import AgentChatStream from '../components/AgentChatStream';

interface Agent {
  id: string;
  name: string;
  avatar?: string;
  greeting?: string;
  system_prompt: string;
}

export const AgentChatStreamPage: React.FC = () => {
  const { agentId } = useParams<{ agentId: string }>();
  const navigate = useNavigate();
  const [agent, setAgent] = useState<Agent | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!agentId) {
      navigate('/agents');
      return;
    }

    fetchAgent();
  }, [agentId]);

  const fetchAgent = async () => {
    try {
      setLoading(true);
      const token = localStorage.getItem('token');
      const response = await fetch(`/api/agents/${agentId}`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to fetch agent');
      }

      const data = await response.json();
      setAgent(data);
    } catch (err: any) {
      setError(err.message || 'Failed to load agent');
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto"></div>
          <p className="mt-4 text-gray-600">Loading agent...</p>
        </div>
      </div>
    );
  }

  if (error || !agent) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-center">
          <div className="text-red-500 text-5xl mb-4">⚠️</div>
          <h2 className="text-2xl font-semibold text-gray-800 mb-2">Error</h2>
          <p className="text-gray-600 mb-4">{error || 'Agent not found'}</p>
          <button
            onClick={() => navigate('/agents')}
            className="px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
          >
            Back to Agents
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col">
      {/* Navigation Bar */}
      <nav className="bg-white border-b px-6 py-3 flex items-center">
        <button
          onClick={() => navigate('/agents')}
          className="text-gray-600 hover:text-gray-800 mr-4"
        >
          ← Back
        </button>
        <h1 className="text-lg font-semibold text-gray-800">Chat with Agent</h1>
      </nav>

      {/* Chat Component */}
      <div className="flex-1 overflow-hidden">
        <AgentChatStream
          agentId={agent.id}
          agentName={agent.name}
          greeting={agent.greeting}
        />
      </div>
    </div>
  );
};

export default AgentChatStreamPage;
