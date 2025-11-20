import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { flowService } from '../services/flow.service';
import type { Flow } from '../types';
import { Button, Card, Table, Loader, Alert } from '../components/common';

export const FlowListPage = () => {
  const navigate = useNavigate();
  const [flows, setFlows] = useState<Flow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<string>('all');
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const limit = 10;

  useEffect(() => {
    loadFlows();
  }, [page, filter]);

  const loadFlows = async () => {
    try {
      setLoading(true);
      setError(null);
      const params: any = { page, limit };
      if (filter !== 'all') {
        params.status = filter;
      }
      
      // 添加模拟数据
      const mockFlows: Flow[] = [
        {
          id: 'mock-flow-1',
          name: 'AI巡检流程',
          description: '自动化门店巡检流程，包括图像识别、标准对比、问题记录和报告生成',
          status: 'Active',
          current_version: 2,
          created_at: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          is_public: false
        },
        {
          id: 'mock-flow-2',
          name: 'Text2SQL流程',
          description: '将自然语言查询转换为SQL语句，执行查询并返回结构化结果',
          status: 'Active',
          current_version: 3,
          created_at: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          is_public: false
        },
        {
          id: 'mock-flow-3',
          name: '销量报告流程',
          description: '自动收集销售数据，进行多维度分析，生成可视化报告并发送给相关人员',
          status: 'Active',
          current_version: 1,
          created_at: new Date(Date.now() - 10 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          is_public: false
        },
        {
          id: 'mock-flow-4',
          name: '客户反馈分析流程',
          description: '收集客户反馈，进行情感分析和主题提取，生成改进建议',
          status: 'Draft',
          current_version: 1,
          created_at: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          is_public: false
        },
        {
          id: 'mock-flow-5',
          name: '智能选址评估流程',
          description: '综合分析地理位置、人流量、竞争对手等因素，输出选址评分和建议',
          status: 'Active',
          current_version: 2,
          created_at: new Date(Date.now() - 20 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 4 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          is_public: false
        },
        {
          id: 'mock-flow-6',
          name: '股票筛选流程',
          description: '根据财务指标、技术指标和市场情绪筛选潜力股票',
          status: 'Draft',
          current_version: 1,
          created_at: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          is_public: false
        }
      ];
      
      const response = await flowService.getFlows(params);
      
      // 根据filter过滤模拟数据
      let filteredMockFlows = mockFlows;
      if (filter !== 'all') {
        filteredMockFlows = mockFlows.filter(flow => 
          flow.status.toLowerCase() === filter.toLowerCase()
        );
      }
      
      // 将模拟数据添加到实际数据前面
      setFlows([...filteredMockFlows, ...response.flows]);
      setTotal(response.total + filteredMockFlows.length);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load flows');
    } finally {
      setLoading(false);
    }
  };

  const handleViewDetails = (flowId: string) => {
    navigate(`/flows/${flowId}`);
  };

  const handleExecute = async (flowId: string) => {
    try {
      const result = await flowService.executeFlow(flowId, {});
      navigate(`/flows/${flowId}/executions/${result.executionId}`);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to execute flow');
    }
  };

  const handleToggleStatus = async (flowId: string, currentStatus: string) => {
    try {
      if (currentStatus === 'Active') {
        await flowService.deactivateFlow(flowId);
      } else {
        await flowService.activateFlow(flowId);
      }
      await loadFlows();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to update flow status');
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Active':
        return 'text-green-600 bg-green-100';
      case 'Draft':
        return 'text-yellow-600 bg-yellow-100';
      case 'Archived':
        return 'text-gray-600 bg-gray-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  };

  const columns = [
    {
      key: 'name',
      label: 'Name',
      render: (flow: Flow) => (
        <div>
          <div className="font-medium text-gray-900">{flow.name}</div>
          {flow.description && (
            <div className="text-sm text-gray-500">{flow.description}</div>
          )}
        </div>
      ),
    },
    {
      key: 'status',
      label: 'Status',
      render: (flow: Flow) => (
        <span
          className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${getStatusColor(
            flow.status
          )}`}
        >
          {flow.status}
        </span>
      ),
    },
    {
      key: 'current_version',
      label: 'Version',
      render: (flow: Flow) => <span className="text-gray-900">v{flow.current_version}</span>,
    },
    {
      key: 'updated_at',
      label: 'Last Updated',
      render: (flow: Flow) => (
        <span className="text-gray-500">
          {new Date(flow.updated_at).toLocaleDateString()}
        </span>
      ),
    },
    {
      key: 'actions',
      label: 'Actions',
      render: (flow: Flow) => (
        <div className="flex gap-2">
          <Button size="sm" variant="secondary" onClick={() => handleViewDetails(flow.id)}>
            View
          </Button>
          {flow.status !== 'Archived' && (
            <Button
              size="sm"
              variant={flow.status === 'Active' ? 'secondary' : 'primary'}
              onClick={() => handleToggleStatus(flow.id, flow.status)}
            >
              {flow.status === 'Active' ? 'Deactivate' : 'Activate'}
            </Button>
          )}
          {flow.status === 'Active' && (
            <Button size="sm" onClick={() => handleExecute(flow.id)}>
              Execute
            </Button>
          )}
        </div>
      ),
    },
  ];

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold text-gray-900">Agent Flows</h1>
        <div className="flex gap-3">
          <Button variant="secondary" onClick={() => navigate('/flows/import')}>
            Import DSL
          </Button>
          <Button onClick={() => navigate('/flows/new')}>Create Flow</Button>
        </div>
      </div>

      {error && (
        <Alert variant="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      <Card>
        <div className="mb-4 flex gap-2">
          {['All', 'Active', 'Draft', 'Archived'].map((status) => (
            <button
              key={status}
              onClick={() => {
                setFilter(status);
                setPage(1);
              }}
              className={`px-4 py-2 text-sm font-medium rounded-lg transition-colors ${filter === status
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                }`}
            >
              {status.charAt(0).toUpperCase() + status.slice(1)}
            </button>
          ))}
        </div>

        <Table data={flows} columns={columns} keyExtractor={(item) => { return item.id }} />

        {total > limit && (
          <div className="mt-4 flex items-center justify-between">
            <div className="text-sm text-gray-700">
              Showing {(page - 1) * limit + 1} to {Math.min(page * limit, total)} of {total}{' '}
              results
            </div>
            <div className="flex gap-2">
              <Button
                size="sm"
                variant="secondary"
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                disabled={page === 1}
              >
                Previous
              </Button>
              <Button
                size="sm"
                variant="secondary"
                onClick={() => setPage((p) => p + 1)}
                disabled={page * limit >= total}
              >
                Next
              </Button>
            </div>
          </div>
        )}
      </Card>
    </div>
  );
};
