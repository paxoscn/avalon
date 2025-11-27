import React, { useState } from 'react';

interface Agent {
  id: string;
  name: string;
  is_published: boolean;
  published_at?: string;
  is_creator: boolean;
}

interface AgentPublishButtonProps {
  agent: Agent;
  onPublishSuccess?: () => void;
  onUnpublishSuccess?: () => void;
}

export const AgentPublishButton: React.FC<AgentPublishButtonProps> = ({
  agent,
  onPublishSuccess,
  onUnpublishSuccess,
}) => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 只有创建者才能看到发布按钮
  if (!agent.is_creator) {
    return null;
  }

  const handlePublish = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`/api/agents/${agent.id}/publish`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error('发布失败');
      }

      onPublishSuccess?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : '发布失败');
    } finally {
      setLoading(false);
    }
  };

  const handleUnpublish = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`/api/agents/${agent.id}/unpublish`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error('取消发布失败');
      }

      onUnpublishSuccess?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : '取消发布失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="agent-publish-button">
      {agent.is_published ? (
        <button
          onClick={handleUnpublish}
          disabled={loading}
          className="btn btn-secondary"
        >
          {loading ? '处理中...' : '取消发布'}
        </button>
      ) : (
        <button
          onClick={handlePublish}
          disabled={loading}
          className="btn btn-primary"
        >
          {loading ? '处理中...' : '发布'}
        </button>
      )}
      
      {error && (
        <div className="error-message text-red-500 mt-2">
          {error}
        </div>
      )}
    </div>
  );
};

// 发布状态徽章组件
interface PublishStatusBadgeProps {
  agent: Agent;
}

export const PublishStatusBadge: React.FC<PublishStatusBadgeProps> = ({ agent }) => {
  if (!agent.is_published) {
    return (
      <span className="badge badge-draft bg-gray-200 text-gray-700 px-2 py-1 rounded">
        未发布
      </span>
    );
  }

  return (
    <span className="badge badge-published bg-green-100 text-green-700 px-2 py-1 rounded">
      已发布
      {agent.published_at && (
        <span className="text-xs ml-1">
          ({new Date(agent.published_at).toLocaleDateString()})
        </span>
      )}
    </span>
  );
};

// Agent卡片组件示例
interface AgentCardProps {
  agent: Agent;
  onRefresh: () => void;
}

export const AgentCard: React.FC<AgentCardProps> = ({ agent, onRefresh }) => {
  return (
    <div className="agent-card border rounded-lg p-4 shadow-sm">
      <div className="flex justify-between items-start mb-2">
        <h3 className="text-lg font-semibold">{agent.name}</h3>
        <PublishStatusBadge agent={agent} />
      </div>
      
      <div className="mt-4 flex gap-2">
        {agent.is_creator && (
          <>
            <button className="btn btn-sm btn-outline">
              编辑
            </button>
            <AgentPublishButton
              agent={agent}
              onPublishSuccess={onRefresh}
              onUnpublishSuccess={onRefresh}
            />
          </>
        )}
        
        {!agent.is_creator && agent.is_published && (
          <button className="btn btn-sm btn-primary">
            雇佣
          </button>
        )}
      </div>
    </div>
  );
};

// 发布确认对话框组件
interface PublishConfirmDialogProps {
  isOpen: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  agentName: string;
}

export const PublishConfirmDialog: React.FC<PublishConfirmDialogProps> = ({
  isOpen,
  onConfirm,
  onCancel,
  agentName,
}) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 max-w-md w-full">
        <h3 className="text-lg font-semibold mb-4">确认发布</h3>
        <p className="text-gray-600 mb-6">
          确定要发布 "{agentName}" 吗？发布后，其他用户将可以看到并雇佣这个Agent。
        </p>
        <div className="flex gap-3 justify-end">
          <button
            onClick={onCancel}
            className="btn btn-secondary"
          >
            取消
          </button>
          <button
            onClick={onConfirm}
            className="btn btn-primary"
          >
            确认发布
          </button>
        </div>
      </div>
    </div>
  );
};
