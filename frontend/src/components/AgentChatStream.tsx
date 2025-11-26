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
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const {
    messages,
    currentResponse,
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
    },
    onError: (error) => {
      console.error('Stream error:', error);
      alert(`Error: ${error}`);
    },
  });

  // 自动滚动到底部
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, currentResponse]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputMessage.trim() || isStreaming) return;

    await sendMessage(inputMessage);
    setInputMessage('');
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
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
        {greeting && messages.length === 0 && (
          <div className="flex items-start space-x-3">
            <div className="flex-shrink-0 w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center text-white font-semibold">
              A
            </div>
            <div className="flex-1 bg-white rounded-lg shadow-sm p-4">
              <p className="text-gray-800">{greeting}</p>
            </div>
          </div>
        )}

        {/* Message List */}
        {messages.map((message) => (
          <div
            key={message.id}
            className={`flex items-start space-x-3 ${
              message.role === 'user' ? 'flex-row-reverse space-x-reverse' : ''
            }`}
          >
            <div
              className={`flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center text-white font-semibold ${
                message.role === 'user' ? 'bg-green-500' : 'bg-blue-500'
              }`}
            >
              {message.role === 'user' ? 'U' : 'A'}
            </div>
            <div
              className={`flex-1 rounded-lg shadow-sm p-4 ${
                message.role === 'user'
                  ? 'bg-green-50 text-gray-800'
                  : 'bg-white text-gray-800'
              }`}
            >
              <p className="whitespace-pre-wrap">{message.content}</p>
              {message.metadata && (
                <div className="mt-2 pt-2 border-t border-gray-200 text-xs text-gray-500">
                  {message.metadata.model && (
                    <span className="mr-3">Model: {message.metadata.model}</span>
                  )}
                  {message.metadata.tokens_used && (
                    <span>Tokens: {message.metadata.tokens_used}</span>
                  )}
                </div>
              )}
            </div>
          </div>
        ))}

        {/* Streaming Response */}
        {isStreaming && currentResponse && (
          <div className="flex items-start space-x-3">
            <div className="flex-shrink-0 w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center text-white font-semibold">
              A
            </div>
            <div className="flex-1 bg-white rounded-lg shadow-sm p-4">
              <p className="whitespace-pre-wrap">{currentResponse}</p>
              <span className="inline-block w-2 h-4 bg-blue-500 animate-pulse ml-1"></span>
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
              onKeyPress={handleKeyPress}
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
