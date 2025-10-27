import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { agentService, type CreateAgentRequest, type UpdateAgentRequest } from '../services/agent.service';
import { llmService } from '../services/llm.service';
import { mcpService } from '../services/mcp.service';
import { flowService } from '../services/flow.service';
import type { Agent, VectorConfig, MCPTool, Flow } from '../types';
import { Button, Card, Input, Loader, Alert, MobileChatPreview } from '../components/common';

export function AgentDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { t } = useTranslation();
  const isNew = id === 'new';

  const [agent, setAgent] = useState<Agent | null>(null);
  const [loading, setLoading] = useState(!isNew);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [availableKnowledgeBases, setAvailableKnowledgeBases] = useState<VectorConfig[]>([]);
  const [availableTools, setAvailableTools] = useState<MCPTool[]>([]);
  const [availableFlows, setAvailableFlows] = useState<Flow[]>([]);

  const [formData, setFormData] = useState({
    name: '',
    avatar: '',
    systemPrompt: '',
    additionalSettings: '',
    presetQuestions: ['', '', ''],
    knowledgeBaseIds: [] as string[],
    mcpToolIds: [] as string[],
    flowIds: [] as string[],
  });

  useEffect(() => {
    loadResources();
    if (!isNew && id) {
      loadAgent();
    }
  }, [id, isNew]);

  const loadAgent = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await agentService.getAgent(id!);
      setAgent(data);
      setFormData({
        name: data.name,
        avatar: data.avatar || '',
        systemPrompt: data.system_prompt,
        additionalSettings: data.additional_settings || '',
        presetQuestions: [
          ...data.preset_questions,
          ...Array(3 - data.preset_questions.length).fill(''),
        ].slice(0, 3),
        knowledgeBaseIds: data.knowledge_base_ids,
        mcpToolIds: data.mcp_tool_ids,
        flowIds: data.flow_ids,
      });
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.loadAgentFailed'));
    } finally {
      setLoading(false);
    }
  };

  const loadResources = async () => {
    try {
      const [kbs, tools, flowsResponse] = await Promise.all([
        llmService.listConfigs().catch(() => []),
        mcpService.listTools().catch(() => []),
        flowService.getFlows().catch(() => ({ flows: [], total: 0 })),
      ]);
      setAvailableKnowledgeBases(kbs as any);
      setAvailableTools(tools);console.log(flowsResponse.flows);
      setAvailableFlows(flowsResponse.flows || []);
    } catch (err) {
      console.error('Failed to load resources:', err);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError(null);
    setSuccess(null);

    try {
      const presetQuestions = formData.presetQuestions.filter((q) => q.trim() !== '');

      if (isNew) {
        const request: CreateAgentRequest = {
          name: formData.name,
          avatar: formData.avatar || undefined,
          system_prompt: formData.systemPrompt,
          additional_settings: formData.additionalSettings || undefined,
          preset_questions: presetQuestions,
          knowledge_base_ids: formData.knowledgeBaseIds,
          mcp_tool_ids: formData.mcpToolIds,
          flow_ids: formData.flowIds,
        };
        const newAgent = await agentService.createAgent(request);
        setSuccess(t('agents.success.created'));
        setTimeout(() => navigate(`/agents/${newAgent.id}`), 1500);
      } else {
        const request: UpdateAgentRequest = {
          name: formData.name,
          avatar: formData.avatar || undefined,
          system_prompt: formData.systemPrompt,
          additional_settings: formData.additionalSettings || undefined,
          preset_questions: presetQuestions,
        };
        await agentService.updateAgent(id!, request);
        setSuccess(t('agents.success.updated'));
        await loadAgent();
      }
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.saveFailed'));
    } finally {
      setSaving(false);
    }
  };

  const handleToggleKnowledgeBase = async (configId: string) => {
    if (isNew) {
      setFormData((prev) => ({
        ...prev,
        knowledgeBaseIds: prev.knowledgeBaseIds != null && prev.knowledgeBaseIds.includes(configId)
          ? prev.knowledgeBaseIds.filter((id) => id !== configId)
          : [...prev.knowledgeBaseIds, configId],
      }));
      return;
    }

    try {
      if (formData.knowledgeBaseIds != null && formData.knowledgeBaseIds.includes(configId)) {
        await agentService.removeKnowledgeBase(id!, configId);
      } else {
        await agentService.addKnowledgeBase(id!, configId);
      }
      await loadAgent();
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.updateKnowledgeBaseFailed'));
    }
  };

  const handleToggleTool = async (toolId: string) => {
    if (isNew) {
      setFormData((prev) => ({
        ...prev,
        mcpToolIds: prev.mcpToolIds.includes(toolId)
          ? prev.mcpToolIds.filter((id) => id !== toolId)
          : [...prev.mcpToolIds, toolId],
      }));
      return;
    }

    try {
      if (formData.mcpToolIds.includes(toolId)) {
        await agentService.removeMcpTool(id!, toolId);
      } else {
        await agentService.addMcpTool(id!, toolId);
      }
      await loadAgent();
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.updateToolFailed'));
    }
  };

  const handleToggleFlow = async (flowId: string) => {
    if (isNew) {
      setFormData((prev) => ({
        ...prev,
        flowIds: prev.flowIds.includes(flowId)
          ? prev.flowIds.filter((id) => id !== flowId)
          : [...prev.flowIds, flowId],
      }));
      return;
    }

    try {
      if (formData.flowIds.includes(flowId)) {
        await agentService.removeFlow(id!, flowId);
      } else {
        await agentService.addFlow(id!, flowId);
      }
      await loadAgent();
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.updateFlowFailed'));
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
    <div className="flex gap-6">
      {/* 左侧编辑表单 */}
      <div className="flex-1 space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-semibold text-gray-900">
              {isNew ? t('agents.detail.createTitle') : t('agents.detail.editTitle')}
            </h1>
            <p className="mt-2 text-sm text-gray-600">
              {isNew ? t('agents.detail.createDescription') : t('agents.detail.editDescription')}
            </p>
          </div>
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

        <form onSubmit={handleSubmit} className="space-y-6">
          <Card>
            <h2 className="text-lg font-medium text-gray-900 mb-4">{t('agents.detail.basicInfo')}</h2>
          <div className="space-y-4">
            <Input
              label={t('agents.detail.agentName')}
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
              placeholder={t('agents.detail.agentNamePlaceholder')}
            />

            <Input
              label={t('agents.detail.avatarUrl')}
              value={formData.avatar}
              onChange={(e) => setFormData({ ...formData, avatar: e.target.value })}
              placeholder={t('agents.detail.avatarUrlPlaceholder')}
            />

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {t('agents.detail.systemPrompt')} *
              </label>
              <textarea
                value={formData.systemPrompt}
                onChange={(e) => setFormData({ ...formData, systemPrompt: e.target.value })}
                required
                rows={6}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder={t('agents.detail.systemPromptPlaceholder')}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {t('agents.detail.additionalSettings')}
              </label>
              <textarea
                value={formData.additionalSettings}
                onChange={(e) => setFormData({ ...formData, additionalSettings: e.target.value })}
                rows={4}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                placeholder={t('agents.detail.additionalSettingsPlaceholder')}
              />
            </div>
            </div>
          </Card>

          <Card>
            <h2 className="text-lg font-medium text-gray-900 mb-4">{t('agents.detail.presetQuestions')}</h2>
          <p className="text-sm text-gray-600 mb-4">
            {t('agents.detail.presetQuestionsDescription')}
          </p>
          <div className="space-y-3">
            {formData.presetQuestions.map((question, index) => (
              <Input
                key={index}
                label={`${t('agents.detail.question')} ${index + 1}`}
                value={question}
                onChange={(e) => {
                  const newQuestions = [...formData.presetQuestions];
                  newQuestions[index] = e.target.value;
                  setFormData({ ...formData, presetQuestions: newQuestions });
                }}
                placeholder={`${t('agents.detail.questionPlaceholder')} ${index + 1}`}
              />
            ))}
            </div>
          </Card>

          <Card>
            <h2 className="text-lg font-medium text-gray-900 mb-4">{t('agents.detail.knowledgeBases')}</h2>
          <p className="text-sm text-gray-600 mb-4">
            {t('agents.detail.knowledgeBasesDescription')}
          </p>
          <div className="space-y-2">
            {availableKnowledgeBases.length === 0 ? (
              <p className="text-sm text-gray-500">{t('agents.detail.noKnowledgeBases')}</p>
            ) : (
              availableKnowledgeBases.map((kb) => (
                <label key={kb.id} className="flex items-center p-3 border rounded-lg hover:bg-gray-50 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.knowledgeBaseIds != null && formData.knowledgeBaseIds.includes(kb.id)}
                    onChange={() => handleToggleKnowledgeBase(kb.id)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <span className="ml-3 text-sm font-medium text-gray-900">{kb.name}</span>
                </label>
              ))
            )}
            </div>
          </Card>

          <Card>
            <h2 className="text-lg font-medium text-gray-900 mb-4">{t('agents.detail.mcpTools')}</h2>
          <p className="text-sm text-gray-600 mb-4">
            {t('agents.detail.mcpToolsDescription')}
          </p>
          <div className="space-y-2">
            {availableTools.length === 0 ? (
              <p className="text-sm text-gray-500">{t('agents.detail.noTools')}</p>
            ) : (
              availableTools.map((tool) => (
                <label key={tool.id} className="flex items-center p-3 border rounded-lg hover:bg-gray-50 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.mcpToolIds != null && formData.mcpToolIds.includes(tool.id)}
                    onChange={() => handleToggleTool(tool.id)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <div className="ml-3 flex-1">
                    <span className="text-sm font-medium text-gray-900">{tool.name}</span>
                    {tool.description && (
                      <p className="text-xs text-gray-500">{tool.description}</p>
                    )}
                  </div>
                </label>
              ))
            )}
            </div>
          </Card>

          <Card>
            <h2 className="text-lg font-medium text-gray-900 mb-4">{t('agents.detail.flows')}</h2>
          <p className="text-sm text-gray-600 mb-4">
            {t('agents.detail.flowsDescription')}
          </p>
          <div className="space-y-2">
            {availableFlows.length === 0 ? (
              <p className="text-sm text-gray-500">{t('agents.detail.noFlows')}</p>
            ) : (
              availableFlows.map((flow) => (
                <label key={flow.id} className="flex items-center p-3 border rounded-lg hover:bg-gray-50 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.flowIds != null && formData.flowIds.includes(flow.id)}
                    onChange={() => handleToggleFlow(flow.id)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <div className="ml-3 flex-1">
                    <span className="text-sm font-medium text-gray-900">{flow.name}</span>
                    {flow.description && (
                      <p className="text-xs text-gray-500">{flow.description}</p>
                    )}
                  </div>
                </label>
              ))
            )}
            </div>
          </Card>

          <div className="flex items-center gap-3">
            <Button type="submit" disabled={saving}>
              {saving ? t('agents.detail.saving') : isNew ? t('agents.createAgent') : t('agents.detail.updateAgent')}
            </Button>
            <Button
              type="button"
              variant="secondary"
              onClick={() => navigate('/agents')}
            >
              {t('common.cancel')}
            </Button>
          </div>
        </form>
      </div>

      {/* 右侧手机预览 */}
      <div className="w-96 sticky top-6 self-start">
        <div className="mb-3 text-center">
          <h3 className="text-sm font-medium text-gray-700">{t('agents.detail.preview')}</h3>
          <p className="text-xs text-gray-500">{t('agents.detail.previewDescription')}</p>
        </div>
        <MobileChatPreview
          agentName={formData.name || t('agents.detail.defaultAgentName')}
          agentAvatar={formData.avatar}
          systemPrompt={formData.systemPrompt}
          presetQuestions={formData.presetQuestions}
        />
      </div>
    </div>
  );
}
