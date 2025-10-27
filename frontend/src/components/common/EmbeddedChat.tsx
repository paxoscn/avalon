import { useState } from 'react';
import { MobileChatPreview } from './MobileChatPreview';

export interface EmbeddedChatProps {
  agentId: string;
  agentName: string;
  agentAvatar?: string;
  systemPrompt?: string;
  presetQuestions?: string[];
  onSendMessage?: (message: string) => Promise<string>;
  position?: 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left';
  className?: string;
}

/**
 * 可嵌入任何页面的聊天组件
 * 支持最小化/最大化，可拖动位置
 */
export function EmbeddedChat({
  agentId,
  agentName,
  agentAvatar,
  systemPrompt,
  presetQuestions = [],
  onSendMessage,
  position = 'bottom-right',
  className = '',
}: EmbeddedChatProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [isMinimized, setIsMinimized] = useState(false);

  const positionClasses = {
    'bottom-right': 'bottom-4 right-4',
    'bottom-left': 'bottom-4 left-4',
    'top-right': 'top-4 right-4',
    'top-left': 'top-4 left-4',
  };

  if (!isOpen) {
    return (
      <button
        onClick={() => setIsOpen(true)}
        className={`fixed ${positionClasses[position]} z-50 w-14 h-14 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-full shadow-lg hover:shadow-xl transition-all flex items-center justify-center group ${className}`}
      >
        <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
        </svg>
        <span className="absolute -top-1 -right-1 w-3 h-3 bg-red-500 rounded-full animate-pulse"></span>
      </button>
    );
  }

  return (
    <div
      className={`fixed ${positionClasses[position]} z-50 transition-all ${
        isMinimized ? 'w-80' : 'w-96'
      } ${className}`}
    >
      {isMinimized ? (
        <div className="bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-lg shadow-xl p-4 flex items-center justify-between cursor-pointer hover:shadow-2xl transition-shadow">
          <div className="flex items-center gap-3" onClick={() => setIsMinimized(false)}>
            {agentAvatar ? (
              <img
                src={agentAvatar}
                alt={agentName}
                className="w-10 h-10 rounded-full object-cover border-2 border-white"
              />
            ) : (
              <div className="w-10 h-10 rounded-full bg-white/20 flex items-center justify-center text-lg font-bold border-2 border-white">
                {agentName.charAt(0).toUpperCase()}
              </div>
            )}
            <div>
              <h3 className="font-semibold text-sm">{agentName}</h3>
              <p className="text-xs text-white/80">点击展开</p>
            </div>
          </div>
          <button
            onClick={(e) => {
              e.stopPropagation();
              setIsOpen(false);
            }}
            className="p-1 hover:bg-white/20 rounded-full transition-colors"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      ) : (
        <div className="relative">
          <div className="absolute -top-10 right-0 flex items-center gap-2">
            <button
              onClick={() => setIsMinimized(true)}
              className="p-2 bg-white rounded-full shadow-md hover:shadow-lg transition-all text-gray-600 hover:text-gray-900"
              title="最小化"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12H4" />
              </svg>
            </button>
            <button
              onClick={() => setIsOpen(false)}
              className="p-2 bg-white rounded-full shadow-md hover:shadow-lg transition-all text-gray-600 hover:text-gray-900"
              title="关闭"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
          <MobileChatPreview
            agentName={agentName}
            agentAvatar={agentAvatar}
            systemPrompt={systemPrompt}
            presetQuestions={presetQuestions}
            onSendMessage={onSendMessage}
            className="shadow-2xl"
          />
        </div>
      )}
    </div>
  );
}
