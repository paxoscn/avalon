import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { agentService } from '../services/agent.service';
import type { Agent } from '../types';
import { Button, Card, Loader, Alert, MobileChatPreview } from '../components/common';

export function AgentTunePage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { t } = useTranslation();

  const [agent, setAgent] = useState<Agent | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [additionalRequirements, setAdditionalRequirements] = useState('');

  useEffect(() => {
    if (id) {
      loadAgent();
    }
  }, [id]);

  const loadAgent = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await agentService.getAgent(id!);
      setAgent(data);
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.loadAgentFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!agent || !additionalRequirements.trim()) {
      setError(t('agents.tune.requirementsRequired'));
      return;
    }

    setSaving(true);
    setError(null);
    setSuccess(null);

    try {
      const updatedSystemPrompt = `${agent.system_prompt}\n\n${additionalRequirements.trim()}`;
      
      await agentService.updateAgent(id!, {
        system_prompt: updatedSystemPrompt,
      });

      setSuccess(t('agents.tune.success'));
      setTimeout(() => navigate('/agents'), 1500);
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.tune.saveFailed'));
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader size="lg" />
      </div>
    );
  }

  if (!agent) {
    return (
      <div className="flex items-center justify-center h-64">
        <Alert type="error">{t('agents.errors.loadAgentFailed')}</Alert>
      </div>
    );
  }

  return (
    <div className="flex gap-6">
      {/* 左侧调优表单 */}
      <div className="flex-1 space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-semibold text-gray-900">
              {t('agents.tune.title')}
            </h1>
            <p className="mt-2 text-sm text-gray-600">
              {t('agents.tune.description')}
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

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">
            {t('agents.tune.agentInfo')}
          </h2>
          <div className="space-y-3">
            <div className="flex items-center gap-3">
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
              <div>
                <h3 className="text-lg font-medium text-gray-900">{agent.name}</h3>
                <p className="text-sm text-gray-500">
                  {t('agents.tune.createdAt')}: {new Date(agent.created_at).toLocaleDateString()}
                </p>
              </div>
            </div>
          </div>
        </Card>

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">
            {t('agents.tune.currentPrompt')}
          </h2>
          <div className="bg-gray-50 p-4 rounded-lg border border-gray-200">
            <pre className="text-sm text-gray-700 whitespace-pre-wrap font-sans">
              {agent.system_prompt}
            </pre>
          </div>
        </Card>

        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">
            {t('agents.tune.additionalRequirements')} *
          </h2>
          <p className="text-sm text-gray-600 mb-4">
            {t('agents.tune.requirementsDescription')}
          </p>
          <textarea
            value={additionalRequirements}
            onChange={(e) => setAdditionalRequirements(e.target.value)}
            rows={12}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            placeholder={t('agents.tune.requirementsPlaceholder')}
          />
        </Card>

        <div className="flex items-center gap-3">
          <Button onClick={handleSave} disabled={saving || !additionalRequirements.trim()}>
            {saving ? t('agents.tune.saving') : t('agents.tune.saveChanges')}
          </Button>
          <Button
            variant="secondary"
            onClick={() => navigate('/agents')}
          >
            {t('common.cancel')}
          </Button>
        </div>
      </div>

      {/* 右侧手机预览 */}
      <div className="w-96 sticky top-6 self-start">
        <div className="mb-3 text-center">
          <h3 className="text-sm font-medium text-gray-700">{t('agents.tune.preview')}</h3>
          <p className="text-xs text-gray-500">{t('agents.tune.previewDescription')}</p>
        </div>
        <MobileChatPreview
          agentId={id}
          agentName={agent.name}
          agentAvatar={agent.avatar}
          greeting={agent.greeting}
          systemPrompt={
            additionalRequirements.trim()
              ? `${agent.system_prompt}\n\n${additionalRequirements.trim()}`
              : agent.system_prompt
          }
          presetQuestions={agent.preset_questions}
          className="h-[700px]"
        />
      </div>
    </div>
  );
}
