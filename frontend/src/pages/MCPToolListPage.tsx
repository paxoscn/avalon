import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { mcpService } from '../services/mcp.service';
import type { MCPTool } from '../types';
import { Button, Card, Loader, Alert } from '../components/common';

export function MCPToolListPage() {
  const { t } = useTranslation();
  const [tools, setTools] = useState<MCPTool[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadTools();
  }, []);

  const loadTools = async () => {
    try {
      setLoading(true);
      setError(null);
      
      // 添加模拟数据
      const mockTools: MCPTool[] = [
        {
          id: 'mock-mcp-1',
          name: '会员系统MCP',
          description: '提供会员信息查询、积分管理、等级升级、优惠券发放等功能，支持会员全生命周期管理',
          status: 'active',
          current_version: '2.1.0',
          created_at: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          config: {}
        },
        {
          id: 'mock-mcp-2',
          name: '商品中心MCP',
          description: '商品信息管理工具，包括商品查询、库存查看、价格更新、SKU管理、分类管理等核心功能',
          status: 'active',
          current_version: '3.0.2',
          created_at: new Date(Date.now() - 45 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          config: {}
        },
        {
          id: 'mock-mcp-3',
          name: '门店中心MCP',
          description: '门店管理系统接口，支持门店信息查询、营业状态管理、员工排班、业绩统计等功能',
          status: 'active',
          current_version: '1.5.1',
          created_at: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          config: {}
        },
        {
          id: 'mock-mcp-4',
          name: '订单系统MCP',
          description: '订单全流程管理工具，包括订单创建、状态跟踪、退款处理、物流查询等功能',
          status: 'active',
          current_version: '2.3.0',
          created_at: new Date(Date.now() - 40 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          config: {}
        },
        {
          id: 'mock-mcp-5',
          name: '营销活动MCP',
          description: '营销活动管理平台，支持活动创建、规则配置、效果追踪、数据分析等功能',
          status: 'inactive',
          current_version: '1.2.0',
          created_at: new Date(Date.now() - 20 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 10 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          config: {}
        },
        {
          id: 'mock-mcp-6',
          name: '数据分析MCP',
          description: '数据查询和分析工具，提供多维度报表、趋势分析、数据导出等功能',
          status: 'active',
          current_version: '2.0.0',
          created_at: new Date(Date.now() - 35 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          config: {}
        },
        {
          id: 'mock-mcp-7',
          name: '消息通知MCP',
          description: '统一消息推送服务，支持短信、邮件、站内信、APP推送等多种通知方式',
          status: 'active',
          current_version: '1.8.0',
          created_at: new Date(Date.now() - 50 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 4 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          config: {}
        },
        {
          id: 'mock-mcp-8',
          name: '财务系统MCP',
          description: '财务数据管理工具，包括账单查询、对账管理、结算处理、财务报表等功能',
          status: 'inactive',
          current_version: '1.0.5',
          created_at: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 8 * 24 * 60 * 60 * 1000).toISOString(),
          owner_id: 'user-1',
          config: {}
        }
      ];
      
      const data = await mcpService.listTools();
      // 将模拟数据添加到实际数据前面
      setTools([...mockTools, ...data]);
    } catch (err: any) {
      setError(err.response?.data?.error || t('mcpTools.errors.loadFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleToggleStatus = async (tool: MCPTool) => {
    try {
      const newStatus = tool.status.toLowerCase() === 'active' ? 'inactive' : 'active';
      await mcpService.toggleToolStatus(tool.id, newStatus);
      await loadTools();
    } catch (err: any) {
      setError(err.response?.data?.error || t('mcpTools.errors.toggleFailed'));
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm(t('mcpTools.confirmDelete'))) {
      return;
    }

    try {
      await mcpService.deleteTool(id);
      await loadTools();
    } catch (err: any) {
      setError(err.response?.data?.error || t('mcpTools.errors.deleteFailed'));
    }
  };

  if (loading) {
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
          <h1 className="text-3xl font-semibold text-gray-900">{t('mcpTools.title')}</h1>
          <p className="mt-2 text-sm text-gray-600">
            {t('mcpTools.description')}
          </p>
        </div>
        <Link to="/mcp/tools/new">
          <Button>{t('mcpTools.createTool')}</Button>
        </Link>
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {tools.length === 0 ? (
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
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-gray-900">{t('mcpTools.noTools')}</h3>
            <p className="mt-1 text-sm text-gray-500">
              {t('mcpTools.getStarted')}
            </p>
            <div className="mt-6">
              <Link to="/mcp/tools/new">
                <Button>{t('mcpTools.createTool')}</Button>
              </Link>
            </div>
          </div>
        </Card>
      ) : (
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          {tools.map((tool) => (
            <Card key={tool.id} className="hover:shadow-lg transition-shadow">
              <div className="space-y-4">
                <div className="flex items-start justify-between">
                  <div className="flex-1 min-w-0">
                    <h3 className="text-lg font-medium text-gray-900 truncate">
                      {tool.name}
                    </h3>
                    {tool.description && (
                      <p className="mt-1 text-sm text-gray-500 line-clamp-2">
                        {tool.description}
                      </p>
                    )}
                  </div>
                  <span
                    className={`ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      tool.status.toLowerCase() === 'active'
                        ? 'bg-green-100 text-green-800'
                        : 'bg-gray-100 text-gray-800'
                    }`}
                  >
                    {tool.status}
                  </span>
                </div>

                <div className="text-sm text-gray-500">
                  <div>{t('mcpTools.version')}: {tool.current_version}</div>
                  <div>{t('mcpTools.created')}: {new Date(tool.created_at).toLocaleDateString()}</div>
                </div>

                <div className="flex items-center gap-2 pt-4 border-t border-gray-200">
                  <Link to={`/mcp/tools/${tool.id}`} className="flex-1">
                    <Button variant="secondary" className="w-full">
                      {t('mcpTools.configure')}
                    </Button>
                  </Link>
                  <Link to={`/mcp/tools/${tool.id}/test`} className="flex-1">
                    <Button variant="secondary" className="w-full">
                      {t('mcpTools.testIt')}
                    </Button>
                  </Link>
                </div>

                <div className="flex items-center gap-2">
                  <Button
                    variant="secondary"
                    onClick={() => handleToggleStatus(tool)}
                    className="flex-1"
                  >
                    {tool.status.toLowerCase() === 'active' ? t('mcpTools.deactivate') : t('mcpTools.activate')}
                  </Button>
                  <Button
                    variant="secondary"
                    onClick={() => handleDelete(tool.id)}
                    className="text-red-600 hover:text-red-700"
                  >
                    {t('common.delete')}
                  </Button>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
