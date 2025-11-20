import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { agentService } from '../services/agent.service';
import type { Agent } from '../types';
import { Button, Card, Loader, Alert } from '../components/common';

type TabType = 'created' | 'employed' | 'allocated' | 'visible';

export function AgentListPage() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<TabType>('created');
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);

  useEffect(() => {
    setPage(1);
  }, [activeTab]);

  useEffect(() => {
    loadAgents();
  }, [page, activeTab]);

  const loadAgents = async () => {
    try {
      setLoading(true);
      setError(null);
      let response;

      switch (activeTab) {
        case 'created':
          response = await agentService.listCreatedAgents({ page, page_size: 12 });
          break;
        case 'employed':
          // 添加模拟数据
          const mockEmployedAgents: Agent[] = [
            {
              id: 'mock-1',
              name: '门店巡检专家',
              system_prompt: '我是一位专业的门店巡检专家，擅长门店运营标准检查、陈列规范审核、服务质量评估等工作。我会帮助您发现门店运营中的问题并提供改进建议。',
              avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=inspector&backgroundColor=b6e3f4',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              knowledge_base_ids: ['kb-1', 'kb-2'],
              mcp_tool_ids: ['tool-1'],
              flow_ids: ['flow-1'],
              owner_id: 'user-1',
              is_public: false
            },
            {
              id: 'mock-2',
              name: '数据分析师',
              system_prompt: '我是一位资深数据分析师，精通数据挖掘、统计分析、可视化呈现等技能。我可以帮助您从海量数据中提取有价值的洞察，支持业务决策。',
              avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=analyst&backgroundColor=c0aede',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              knowledge_base_ids: ['kb-3'],
              mcp_tool_ids: ['tool-2', 'tool-3'],
              flow_ids: [],
              owner_id: 'user-1',
              is_public: false
            },
            {
              id: 'mock-3',
              name: '门店选址专家',
              system_prompt: '我是门店选址领域的专家，拥有丰富的商圈分析、人流量评估、竞争对手分析经验。我会综合考虑地理位置、人口密度、消费能力等因素，为您推荐最佳的门店位置。',
              avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=location&backgroundColor=ffdfbf',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              knowledge_base_ids: ['kb-4', 'kb-5'],
              mcp_tool_ids: ['tool-4'],
              flow_ids: ['flow-2', 'flow-3'],
              owner_id: 'user-1',
              is_public: false
            },
            {
              id: 'mock-4',
              name: 'A股投资助理',
              system_prompt: '我是您的A股投资助理，熟悉中国股市政策、行业动态、财务分析等知识。我可以帮助您分析个股基本面、追踪市场热点、评估投资风险，为您的投资决策提供参考。',
              avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=stock&backgroundColor=d1f4dd',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              knowledge_base_ids: ['kb-6'],
              mcp_tool_ids: ['tool-5', 'tool-6', 'tool-7'],
              flow_ids: ['flow-4'],
              owner_id: 'user-1',
              is_public: false
            },
            {
              id: 'mock-5',
              name: '择校专家',
              system_prompt: '我是教育领域的择校专家，深入了解各类学校的教学特色、师资力量、升学率等信息。我会根据孩子的特点和家庭需求，为您提供个性化的择校建议和规划方案。',
              avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=education&backgroundColor=ffd5dc',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              knowledge_base_ids: ['kb-7', 'kb-8'],
              mcp_tool_ids: ['tool-8'],
              flow_ids: [],
              owner_id: 'user-1',
              is_public: false
            }
          ];
          
          response = await agentService.listEmployedAgents({ page, page_size: 12 });
          // 将模拟数据添加到实际数据前面
          response.items = [...mockEmployedAgents, ...response.items];
          break;
        case 'allocated':
          response = await agentService.listAllocatedAgents({ page, page_size: 12 });
          break;
        case 'visible':
          response = await agentService.listAgents({ page, page_size: 12 });
          break;
      }

      setAgents(response.items);
      setTotalPages(response.total_pages);
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.loadFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm(t('agents.confirmDelete'))) {
      return;
    }

    try {
      await agentService.deleteAgent(id);
      await loadAgents();
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.deleteFailed'));
    }
  };

  const handleCopy = async (id: string) => {
    try {
      await agentService.copyAgent(id);
      await loadAgents();
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.copyFailed'));
    }
  };

  const handleEmploy = async (id: string) => {
    try {
      await agentService.employAgent(id);
      alert(t('agents.employSuccess'));
      await loadAgents();
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.employFailed'));
    }
  };

  const handleFire = async (id: string) => {
    if (!confirm(t('agents.confirmFire'))) {
      return;
    }

    try {
      await agentService.terminateEmployment(id);
      await loadAgents();
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.fireFailed'));
    }
  };

  const handleAllocate = async (id: string) => {
    try {
      await agentService.allocateAgent(id);
      alert(t('agents.allocateSuccess'));
      await loadAgents();
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.allocateFailed'));
    }
  };

  const handleUnallocate = async (id: string) => {
    if (!confirm(t('agents.confirmUnallocate'))) {
      return;
    }

    try {
      await agentService.terminateAllocation(id);
      await loadAgents();
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.unallocateFailed'));
    }
  };

  const handleTune = (id: string) => {
    // Navigate to agent detail page for tuning
    window.location.href = `/agents/${id}`;
  };

  const handleInterview = (id: string) => {
    // Navigate to agent detail page for interview
    window.location.href = `/agents/${id}`;
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
          <h1 className="text-3xl font-semibold text-gray-900">{t('agents.title')}</h1>
          <p className="mt-2 text-sm text-gray-600">
            {t('agents.description')}
          </p>
        </div>
        <Link to="/agents/new">
          <Button>{t('agents.createAgent')}</Button>
        </Link>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          <button
            onClick={() => setActiveTab('created')}
            className={`
              whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm
              ${activeTab === 'created'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }
            `}
          >
            {t('agents.tabs.created')}
          </button>
          <button
            onClick={() => setActiveTab('employed')}
            className={`
              whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm
              ${activeTab === 'employed'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }
            `}
          >
            {t('agents.tabs.employed')}
          </button>
          <button
            onClick={() => setActiveTab('allocated')}
            className={`
              whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm
              ${activeTab === 'allocated'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }
            `}
          >
            {t('agents.tabs.allocated')}
          </button>
          <button
            onClick={() => setActiveTab('visible')}
            className={`
              whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm
              ${activeTab === 'visible'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }
            `}
          >
            {t('agents.tabs.visible')}
          </button>
        </nav>
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
            <h3 className="mt-2 text-sm font-medium text-gray-900">{t('agents.noAgents')}</h3>
            <p className="mt-1 text-sm text-gray-500">
              {t('agents.getStarted')}
            </p>
            <div className="mt-6">
              <Link to="/agents/new">
                <Button>{t('agents.createAgent')}</Button>
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

                  {/* Action buttons based on active tab */}
                  {activeTab === 'created' && (
                    <>
                      <div className="flex items-center gap-2 pt-4 border-t border-gray-200">
                        <Link to={`/agents/${agent.id}`} className="flex-1">
                          <Button variant="secondary" className="w-full">
                            {t('agents.actions.edit')}
                          </Button>
                        </Link>
                        <Button
                          variant="secondary"
                          onClick={() => handleCopy(agent.id)}
                          className="flex-1"
                        >
                          {t('agents.actions.copy')}
                        </Button>
                      </div>
                      <div className="flex items-center gap-2">
                        <Button
                          variant="secondary"
                          onClick={() => handleDelete(agent.id)}
                          className="w-full text-red-600 hover:text-red-700"
                        >
                          {t('agents.actions.delete')}
                        </Button>
                      </div>
                    </>
                  )}

                  {activeTab === 'employed' && (
                    <>
                      <div className="flex items-center gap-2 pt-4 border-t border-gray-200">
                        <Button
                          variant="secondary"
                          onClick={() => handleTune(agent.id)}
                          className="flex-1"
                        >
                          {t('agents.actions.tune')}
                        </Button>
                        <Button
                          variant="secondary"
                          onClick={() => handleAllocate(agent.id)}
                          className="flex-1"
                        >
                          {t('agents.actions.allocate')}
                        </Button>
                      </div>
                      <div className="flex items-center gap-2">
                        <Button
                          variant="secondary"
                          onClick={() => handleFire(agent.id)}
                          className="flex-1 text-red-600 hover:text-red-700"
                        >
                          {t('agents.actions.fire')}
                        </Button>
                      </div>
                    </>
                  )}

                  {activeTab === 'allocated' && (
                    <div className="flex items-center gap-2 pt-4 border-t border-gray-200">
                      <Button
                        variant="secondary"
                        onClick={() => handleTune(agent.id)}
                        className="flex-1"
                      >
                        {t('agents.actions.tune')}
                      </Button>
                      <Button
                        variant="secondary"
                        onClick={() => handleUnallocate(agent.id)}
                        className="flex-1 text-red-600 hover:text-red-700"
                      >
                        {t('agents.actions.unallocate')}
                      </Button>
                    </div>
                  )}

                  {activeTab === 'visible' && (
                    <div className="flex items-center gap-2 pt-4 border-t border-gray-200">
                      <Button
                        variant="secondary"
                        onClick={() => handleInterview(agent.id)}
                        className="flex-1"
                      >
                        {t('agents.actions.interview')}
                      </Button>
                      <Button
                        variant="secondary"
                        onClick={() => handleEmploy(agent.id)}
                        className="flex-1"
                      >
                        {t('agents.actions.employ')}
                      </Button>
                    </div>
                  )}
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
                {t('common.previous')}
              </Button>
              <span className="text-sm text-gray-600">
                {t('common.page')} {page} {t('common.of')} {totalPages}
              </span>
              <Button
                variant="secondary"
                onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                disabled={page === totalPages}
              >
                {t('common.next')}
              </Button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
