import React, { useState, useRef, useEffect } from 'react';
import { useAgentChatStream } from '../hooks/useAgentChatStream';

interface AgentChatStreamProps {
  agentId: string;
  agentName?: string;
  greeting?: string;
}

export const AgentChatStream: React.FC<AgentChatStreamProps> = ({
  agentId,
  agentName = 'Agent',
  greeting,
}) => {
  const [inputMessage, setInputMessage] = useState('');
  const [expandedReasonings, setExpandedReasonings] = useState<Set<string>>(new Set());
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const toggleReasoning = (messageId: string) => {
    setExpandedReasonings((prev) => {
      const next = new Set(prev);
      if (next.has(messageId)) {
        next.delete(messageId);
      } else {
        next.add(messageId);
      }
      return next;
    });
  };

  const {
    messages,
    currentResponse,
    currentReasoning,
    isStreaming,
    sessionId,
    sendMessage,
    cancelStream,
    clearMessages,
  } = useAgentChatStream({
    agentId,
    onChunk: (chunk) => {
      console.log('Received chunk:', chunk);
    },
    onComplete: (message) => {
      console.log('Message complete:', message);
      // 完成后，思考过程默认收起（不自动展开）
      // 用户可以通过按钮手动展开查看
    },
    onError: (error) => {
      console.error('Stream error:', error);
      alert(`Error: ${error}`);
    },
  });

  // 自动滚动到底部
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, currentResponse, currentReasoning]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputMessage.trim() || isStreaming) return;

    await sendMessage(inputMessage);
    setInputMessage('');
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  return (
    <div className="flex flex-col h-full bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b px-6 py-4 flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-gray-800">{agentName}</h2>
          {sessionId && (
            <p className="text-sm text-gray-500">Session: {sessionId.slice(0, 8)}...</p>
          )}
        </div>
        <button
          onClick={clearMessages}
          className="px-4 py-2 text-sm text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded-lg transition-colors"
        >
          Clear Chat
        </button>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
        {/* Greeting */}
        {greeting && messages.length === 0 && !isStreaming && (
          <div className="flex items-start space-x-3 animate-fade-in">
            <div className="flex-shrink-0 w-8 h-8 bg-gradient-to-br from-blue-400 to-blue-600 rounded-full flex items-center justify-center text-white font-semibold shadow-md">
              🤖
            </div>
            <div className="flex-1 bg-gradient-to-br from-blue-50 to-indigo-50 border border-blue-200 rounded-lg shadow-sm p-4">
              <p className="text-gray-800 leading-relaxed">{greeting}</p>
            </div>
          </div>
        )}

        {/* Message List */}
        {messages.map((message) => (
          <div
            key={message.id}
            className={`flex items-start space-x-3 ${
              message.role === 'User' ? 'flex-row-reverse space-x-reverse' : ''
            }`}
          >
            <div
              className={`flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center text-white font-semibold shadow-md ${
                message.role === 'User' 
                  ? 'bg-gradient-to-br from-green-400 to-green-600' 
                  : 'bg-gradient-to-br from-blue-400 to-blue-600'
              }`}
            >
              {message.role === 'User' ? '👤' : '🤖'}
            </div>
            <div className="flex-1">
              <div
                className={`rounded-lg shadow-sm p-4 border ${
                  message.role === 'User'
                    ? 'bg-gradient-to-br from-green-50 to-emerald-50 border-green-200 text-gray-800'
                    : 'bg-white border-gray-100 text-gray-800'
                }`}
              >
                <p className="whitespace-pre-wrap leading-relaxed">{message.content}</p>
                
                {/* 思考过程按钮 */}
                {message.reasoning && message.role === 'Assistant' && (
                  <div className="mt-3">
                    <button
                      onClick={() => toggleReasoning(message.id)}
                      className="flex items-center space-x-2 text-sm text-amber-700 hover:text-amber-800 font-medium transition-colors"
                    >
                      <svg 
                        className={`w-4 h-4 transition-transform ${expandedReasonings.has(message.id) ? 'rotate-90' : ''}`}
                        fill="none" 
                        stroke="currentColor" 
                        viewBox="0 0 24 24"
                      >
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                      </svg>
                      <span>💭 思考过程</span>
                    </button>
                    
                    {/* 展开的思考过程 */}
                    {expandedReasonings.has(message.id) && (
                      <div className="mt-2 p-3 bg-gradient-to-r from-amber-50 to-orange-50 border border-amber-200 rounded-lg">
                        <p className="text-sm text-amber-900 whitespace-pre-wrap leading-relaxed">
                          {message.reasoning}
                        </p>
                      </div>
                    )}
                  </div>
                )}
                
                {message.metadata && (
                  <div className="mt-3 pt-3 border-t border-gray-200 flex items-center space-x-4 text-xs text-gray-500">
                    {message.metadata.model && (
                      <span className="flex items-center space-x-1">
                        <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
                        </svg>
                        <span>{message.metadata.model}</span>
                      </span>
                    )}
                    {message.metadata.tokens_used && (
                      <span className="flex items-center space-x-1">
                        <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 20l4-16m2 16l4-16M6 9h14M4 15h14" />
                        </svg>
                        <span>{message.metadata.tokens_used} tokens</span>
                      </span>
                    )}
                  </div>
                )}
              </div>
            </div>
          </div>
        ))}

        {/* Streaming Response */}
        {isStreaming && (
          <div className="flex items-start space-x-3 animate-fade-in">
            <div className="flex-shrink-0 w-8 h-8 bg-gradient-to-br from-blue-400 to-blue-600 rounded-full flex items-center justify-center text-white font-semibold shadow-md">
              🤖
            </div>
            <div className="flex-1 space-y-3">
              {/* 思考过程 - 只在有思考内容时显示 */}
              {currentReasoning && (
                <div className="bg-gradient-to-r from-amber-50 to-orange-50 border border-amber-200 rounded-lg p-4 shadow-sm">
                  <div className="flex items-center space-x-2 mb-2">
                    <svg className="w-5 h-5 text-amber-600 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    <span className="text-sm font-semibold text-amber-700">💭 思考过程</span>
                  </div>
                  <div className="text-sm text-amber-900 whitespace-pre-wrap leading-relaxed">
                    {currentReasoning}
                    <span className="inline-block w-1 h-4 bg-amber-600 animate-pulse ml-1 align-middle"></span>
                  </div>
                </div>
              )}
              
              {/* 回复内容 - 只在思考过程结束后显示 */}
              {!currentReasoning && currentResponse && (
                <div className="bg-white rounded-lg shadow-sm p-4 border border-gray-100">
                  <div className="text-gray-800 whitespace-pre-wrap leading-relaxed">
                    {currentResponse}
                    <span className="inline-block w-1 h-4 bg-blue-500 animate-pulse ml-1 align-middle"></span>
                  </div>
                </div>
              )}
              
              {/* 加载状态 - 既没有思考也没有回复时 */}
              {!currentReasoning && !currentResponse && (
                <div className="bg-gradient-to-r from-amber-50 to-orange-50 border border-amber-200 rounded-lg p-4 shadow-sm">
                  <div className="flex items-center space-x-2">
                    <svg className="w-5 h-5 text-amber-600 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    <span className="text-sm font-semibold text-amber-700">💭 正在思考...</span>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="bg-white border-t px-6 py-4">
        <form onSubmit={handleSubmit} className="flex items-end space-x-3">
          <div className="flex-1">
            <textarea
              value={inputMessage}
              onChange={(e) => setInputMessage(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Type your message..."
              disabled={isStreaming}
              rows={1}
              className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none disabled:bg-gray-100 disabled:cursor-not-allowed"
              style={{ minHeight: '48px', maxHeight: '120px' }}
            />
          </div>
          {isStreaming ? (
            <button
              type="button"
              onClick={cancelStream}
              className="px-6 py-3 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors font-medium"
            >
              Stop
            </button>
          ) : (
            <button
              type="submit"
              disabled={!inputMessage.trim()}
              className="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors font-medium disabled:bg-gray-300 disabled:cursor-not-allowed"
            >
              Send
            </button>
          )}
        </form>
      </div>
    </div>
  );
};

export default AgentChatStream;
