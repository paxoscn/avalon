import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { agentService } from '../services/agent.service';
import type { Agent, InterviewRecord } from '../types';
import { Button, Card, Loader, Alert, MobileChatPreview } from '../components/common';

export function AgentInterviewPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { t } = useTranslation();

  const [agent, setAgent] = useState<Agent | null>(null);
  const [interviewRecords, setInterviewRecords] = useState<InterviewRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingRecords, setLoadingRecords] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const [scores, setScores] = useState({
    professionalism: 5,
    communication: 5,
    knowledge: 5,
    responsiveness: 5,
    overall: 5,
  });

  const [feedback, setFeedback] = useState('');

  useEffect(() => {
    loadAgent();
    loadInterviewRecords();
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

  const loadInterviewRecords = async () => {
    try {
      setLoadingRecords(true);
      const records = await agentService.getInterviewRecords(id!);
      setInterviewRecords(records);
    } catch (err: any) {
      console.error('Failed to load interview records:', err);
      // Don't show error to user, just log it
    } finally {
      setLoadingRecords(false);
    }
  };

  const handleScoreChange = (category: keyof typeof scores, value: number) => {
    setScores((prev) => ({
      ...prev,
      [category]: value,
    }));
  };

  const handleInterviewResult = async (passed: boolean) => {
    if (!agent) return;

    try {
      setSubmitting(true);
      setError(null);
      
      // 保存面试结果
      await agentService.completeInterview(agent.id, passed);
      
      if (passed) {
        setSuccess(t('agents.interview.passSuccess'));
      } else {
        setSuccess(t('agents.interview.failSuccess'));
      }
      
      // Reload interview records
      await loadInterviewRecords();
      
      setTimeout(() => navigate('/agents'), 1500);
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.interviewFailed'));
    } finally {
      setSubmitting(false);
    }
  };

  const handleEmploy = async () => {
    if (!agent) return;

    try {
      setSubmitting(true);
      setError(null);
      
      // 先标记面试通过
      await agentService.completeInterview(agent.id, true);
      
      // 然后雇佣
      await agentService.employAgent(agent.id);
      
      setSuccess(t('agents.interview.employSuccess'));
      
      // Reload interview records
      await loadInterviewRecords();
      
      setTimeout(() => navigate('/agents'), 1500);
    } catch (err: any) {
      setError(err.response?.data?.error || t('agents.errors.employFailed'));
    } finally {
      setSubmitting(false);
    }
  };

  const handleFirstMessage = async () => {
    if (!agent) return;
    
    try {
      await agentService.startInterview(agent.id);
    } catch (err: any) {
      console.error('Failed to record interview start:', err);
      // Don't show error to user, just log it
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
      <div className="text-center py-12">
        <p className="text-gray-500">{t('agents.errors.loadAgentFailed')}</p>
      </div>
    );
  }

  const averageScore = (
    Object.values(scores).reduce((sum, score) => sum + score, 0) / Object.keys(scores).length
  ).toFixed(1);

  const getStatusBadge = (status: InterviewRecord['status']) => {
    const statusConfig = {
      pending: { text: t('agents.interview.statusPending'), className: 'bg-gray-100 text-gray-800' },
      in_progress: { text: t('agents.interview.statusInProgress'), className: 'bg-blue-100 text-blue-800' },
      passed: { text: t('agents.interview.statusPassed'), className: 'bg-green-100 text-green-800' },
      failed: { text: t('agents.interview.statusFailed'), className: 'bg-red-100 text-red-800' },
      cancelled: { text: t('agents.interview.statusCancelled'), className: 'bg-gray-100 text-gray-800' },
    };
    const config = statusConfig[status];
    return (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${config.className}`}>
        {config.text}
      </span>
    );
  };

  return (
    <div className="flex gap-6">
      {/* 左侧区域 - 分为两列 */}
      <div className="flex-1 flex gap-6">
        {/* 左子列 - 面试历史 */}
        <div className="w-80 space-y-6">
          <div>
            <h2 className="text-lg font-semibold text-gray-900 mb-4">
              {t('agents.interview.history')}
            </h2>
            
            {loadingRecords ? (
              <div className="flex items-center justify-center py-8">
                <Loader size="md" />
              </div>
            ) : interviewRecords.length === 0 ? (
              <Card>
                <div className="text-center py-8">
                  <p className="text-sm text-gray-500">{t('agents.interview.noHistory')}</p>
                </div>
              </Card>
            ) : (
              <div className="space-y-3 max-h-[calc(100vh-200px)] overflow-y-auto">
                {interviewRecords.map((record) => (
                  <Card key={record.id} className="hover:shadow-md transition-shadow">
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <span className="text-xs text-gray-500">
                          {new Date(record.created_at).toLocaleString()}
                        </span>
                        {getStatusBadge(record.status)}
                      </div>
                      
                      {record.score !== undefined && record.score !== null && (
                        <div className="flex items-center gap-2">
                          <span className="text-sm text-gray-600">{t('agents.interview.score')}:</span>
                          <span className="text-lg font-semibold text-blue-600">{record.score}</span>
                        </div>
                      )}
                      
                      {record.feedback && (
                        <div className="mt-2">
                          <p className="text-xs text-gray-500 mb-1">{t('agents.interview.feedback')}:</p>
                          <p className="text-sm text-gray-700 line-clamp-3">{record.feedback}</p>
                        </div>
                      )}
                      
                      {record.completed_at && (
                        <div className="text-xs text-gray-500 pt-2 border-t border-gray-100">
                          {t('agents.interview.completedAt')}: {new Date(record.completed_at).toLocaleString()}
                        </div>
                      )}
                    </div>
                  </Card>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* 右子列 - 面试评价操作 */}
        <div className="flex-1 space-y-6">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-semibold text-gray-900">
                {t('agents.interview.title')}
              </h1>
              <p className="mt-2 text-sm text-gray-600">
                {t('agents.interview.description')}
              </p>
            </div>
            <Button variant="secondary" onClick={() => navigate('/agents')}>
              {t('common.cancel')}
            </Button>
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

          {/* 数字人信息 */}
          <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">
            {t('agents.interview.agentInfo')}
          </h2>
          <div className="flex items-center gap-4">
            {agent.avatar ? (
              <img
                src={agent.avatar}
                alt={agent.name}
                className="w-16 h-16 rounded-full object-cover"
              />
            ) : (
              <div className="w-16 h-16 rounded-full bg-gradient-to-br from-blue-400 to-purple-500 flex items-center justify-center text-white text-2xl font-bold">
                {agent.name.charAt(0).toUpperCase()}
              </div>
            )}
            <div>
              <h3 className="text-xl font-medium text-gray-900">{agent.name}</h3>
              <p className="text-sm text-gray-500">
                {t('agents.interview.createdAt')}: {new Date(agent.created_at).toLocaleDateString()}
              </p>
            </div>
          </div>
        </Card>

        {/* 评分表 */}
        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">
            {t('agents.interview.evaluationScores')}
          </h2>
          <div className="space-y-6">
            {/* 专业性 */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <label className="text-sm font-medium text-gray-700">
                  {t('agents.interview.professionalism')}
                </label>
                <span className="text-lg font-semibold text-blue-600">
                  {scores.professionalism}
                </span>
              </div>
              <input
                type="range"
                min="1"
                max="10"
                value={scores.professionalism}
                onChange={(e) => handleScoreChange('professionalism', parseInt(e.target.value))}
                className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"
              />
              <div className="flex justify-between text-xs text-gray-500 mt-1">
                <span>1</span>
                <span>10</span>
              </div>
            </div>

            {/* 沟通能力 */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <label className="text-sm font-medium text-gray-700">
                  {t('agents.interview.communication')}
                </label>
                <span className="text-lg font-semibold text-blue-600">
                  {scores.communication}
                </span>
              </div>
              <input
                type="range"
                min="1"
                max="10"
                value={scores.communication}
                onChange={(e) => handleScoreChange('communication', parseInt(e.target.value))}
                className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"
              />
              <div className="flex justify-between text-xs text-gray-500 mt-1">
                <span>1</span>
                <span>10</span>
              </div>
            </div>

            {/* 知识储备 */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <label className="text-sm font-medium text-gray-700">
                  {t('agents.interview.knowledge')}
                </label>
                <span className="text-lg font-semibold text-blue-600">
                  {scores.knowledge}
                </span>
              </div>
              <input
                type="range"
                min="1"
                max="10"
                value={scores.knowledge}
                onChange={(e) => handleScoreChange('knowledge', parseInt(e.target.value))}
                className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"
              />
              <div className="flex justify-between text-xs text-gray-500 mt-1">
                <span>1</span>
                <span>10</span>
              </div>
            </div>

            {/* 响应速度 */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <label className="text-sm font-medium text-gray-700">
                  {t('agents.interview.responsiveness')}
                </label>
                <span className="text-lg font-semibold text-blue-600">
                  {scores.responsiveness}
                </span>
              </div>
              <input
                type="range"
                min="1"
                max="10"
                value={scores.responsiveness}
                onChange={(e) => handleScoreChange('responsiveness', parseInt(e.target.value))}
                className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"
              />
              <div className="flex justify-between text-xs text-gray-500 mt-1">
                <span>1</span>
                <span>10</span>
              </div>
            </div>

            {/* 综合评价 */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <label className="text-sm font-medium text-gray-700">
                  {t('agents.interview.overall')}
                </label>
                <span className="text-lg font-semibold text-blue-600">
                  {scores.overall}
                </span>
              </div>
              <input
                type="range"
                min="1"
                max="10"
                value={scores.overall}
                onChange={(e) => handleScoreChange('overall', parseInt(e.target.value))}
                className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"
              />
              <div className="flex justify-between text-xs text-gray-500 mt-1">
                <span>1</span>
                <span>10</span>
              </div>
            </div>

            {/* 平均分 */}
            <div className="pt-4 border-t border-gray-200">
              <div className="flex items-center justify-between">
                <span className="text-base font-medium text-gray-900">
                  {t('agents.interview.averageScore')}
                </span>
                <span className="text-2xl font-bold text-blue-600">
                  {averageScore}
                </span>
              </div>
            </div>
          </div>
        </Card>

        {/* 面试反馈 */}
        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">
            {t('agents.interview.feedback')}
          </h2>
          <textarea
            value={feedback}
            onChange={(e) => setFeedback(e.target.value)}
            rows={6}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            placeholder={t('agents.interview.feedbackPlaceholder')}
          />
          <p className="mt-2 text-xs text-gray-500">
            {t('agents.interview.feedbackDescription')}
          </p>
        </Card>

        {/* 面试结果决策 */}
        <Card>
          <div>
            <h3 className="text-base font-medium text-gray-900 mb-2">
              {t('agents.interview.decision')}
            </h3>
            <p className="text-sm text-gray-600 mb-4">
              {t('agents.interview.decisionDescription')}
            </p>
            
            <div className="flex gap-3">
              {/* 面试通过按钮 */}
              <Button
                onClick={() => handleInterviewResult(true)}
                disabled={submitting}
                className="flex-1 bg-green-600 hover:bg-green-700 text-white"
              >
                {submitting ? t('common.submitting') : t('agents.interview.pass')}
              </Button>
              
              {/* 面试不通过按钮 */}
              <Button
                onClick={() => handleInterviewResult(false)}
                disabled={submitting}
                variant="secondary"
                className="flex-1 bg-red-600 hover:bg-red-700 text-white border-red-600"
              >
                {submitting ? t('common.submitting') : t('agents.interview.fail')}
              </Button>
            </div>
            
            <div className="mt-4 pt-4 border-t border-gray-200">
              <p className="text-xs text-gray-500 mb-3">
                {t('agents.interview.employNote')}
              </p>
              <Button
                onClick={handleEmploy}
                disabled={submitting}
                className="w-full bg-blue-600 hover:bg-blue-700"
              >
                {submitting ? t('agents.interview.employing') : t('agents.interview.passAndEmploy')}
              </Button>
            </div>
          </div>
        </Card>
        </div>
      </div>

      {/* 右侧预览 */}
      <div className="w-96 sticky top-6 self-start">
        <div className="mb-3 text-center">
          <h3 className="text-sm font-medium text-gray-700">
            {t('agents.interview.preview')}
          </h3>
          <p className="text-xs text-gray-500">
            {t('agents.interview.previewDescription')}
          </p>
        </div>
        <MobileChatPreview
          agentId={agent.id}
          agentName={agent.name}
          agentAvatar={agent.avatar}
          greeting={agent.greeting}
          systemPrompt={agent.system_prompt}
          presetQuestions={agent.preset_questions}
          onFirstMessage={handleFirstMessage}
          className="h-[700px]"
        />
      </div>
    </div>
  );
}
