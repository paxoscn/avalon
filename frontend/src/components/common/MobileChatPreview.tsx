import { useState, useRef, useEffect } from 'react';

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

export interface MobileChatPreviewProps {
  agentName: string;
  agentAvatar?: string;
  greeting?: string;
  systemPrompt?: string;
  presetQuestions?: string[];
  onSendMessage?: (message: string) => Promise<string>;
  className?: string;
}

export function MobileChatPreview({
  agentName,
  agentAvatar,
  greeting,
  systemPrompt,
  presetQuestions = [],
  onSendMessage,
  className = '',
}: MobileChatPreviewProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isTyping, setIsTyping] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSendMessage = async (content: string) => {
    if (!content.trim()) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: content.trim(),
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInputValue('');
    setIsTyping(true);

    try {
      let responseContent = '';
      
      if (onSendMessage) {
        responseContent = await onSendMessage(content);
      } else {
        // 模拟响应
        await new Promise((resolve) => setTimeout(resolve, 1000));
        responseContent = `这是一个模拟回复。我是 ${agentName}，收到了您的消息："${content}"`;
      }

      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: responseContent,
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, assistantMessage]);
    } catch (error) {
      console.error('Failed to send message:', error);
      const error_message: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: '抱歉，发送消息时出现错误。',
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, error_message]);
    } finally {
      setIsTyping(false);
    }
  };

  const handlePresetQuestionClick = (question: string) => {
    handleSendMessage(question);
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage(inputValue);
    }
  };

  const filteredPresetQuestions = presetQuestions.filter((q) => q.trim() !== '');

  return (
    <div className={`flex flex-col bg-white rounded-2xl shadow-xl overflow-hidden ${className}`}>
      {/* 手机顶部状态栏 */}
      <div className="bg-gray-900 text-white px-4 py-2 flex items-center justify-between text-xs">
        <span>9:41</span>
        <div className="flex items-center gap-1">
          <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path d="M2 11a1 1 0 011-1h2a1 1 0 011 1v5a1 1 0 01-1 1H3a1 1 0 01-1-1v-5zM8 7a1 1 0 011-1h2a1 1 0 011 1v9a1 1 0 01-1 1H9a1 1 0 01-1-1V7zM14 4a1 1 0 011-1h2a1 1 0 011 1v12a1 1 0 01-1 1h-2a1 1 0 01-1-1V4z" />
          </svg>
          <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M17.778 8.222c-4.296-4.296-11.26-4.296-15.556 0A1 1 0 01.808 6.808c5.076-5.077 13.308-5.077 18.384 0a1 1 0 01-1.414 1.414zM14.95 11.05a7 7 0 00-9.9 0 1 1 0 01-1.414-1.414 9 9 0 0112.728 0 1 1 0 01-1.414 1.414zM12.12 13.88a3 3 0 00-4.242 0 1 1 0 01-1.415-1.415 5 5 0 017.072 0 1 1 0 01-1.415 1.415zM9 16a1 1 0 011-1h.01a1 1 0 110 2H10a1 1 0 01-1-1z" clipRule="evenodd" />
          </svg>
          <svg className="w-5 h-3" fill="currentColor" viewBox="0 0 24 24">
            <rect x="1" y="4" width="21" height="13" rx="2" />
            <path d="M23 9v6" strokeWidth="2" stroke="currentColor" />
          </svg>
        </div>
      </div>

      {/* 聊天头部 */}
      <div className="bg-gradient-to-r from-blue-500 to-purple-600 text-white px-4 py-3 flex items-center gap-3 shadow-md">
        <button className="p-1 hover:bg-white/20 rounded-full transition-colors">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>
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
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold text-base truncate">{agentName}</h3>
          <p className="text-xs text-white/80">在线</p>
        </div>
        <button className="p-1 hover:bg-white/20 rounded-full transition-colors">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
          </svg>
        </button>
      </div>

      {/* 聊天消息区域 */}
      <div className="flex-1 overflow-y-auto bg-gray-50 p-4 space-y-4" style={{ maxHeight: '500px' }}>
        {messages.length === 0 && (
          <div className="text-center py-8">
            <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gradient-to-br from-blue-400 to-purple-500 text-white text-2xl font-bold mb-3">
              {agentName.charAt(0).toUpperCase()}
            </div>
            <h4 className="text-lg font-semibold text-gray-900 mb-2">开始对话</h4>
            <p className="text-sm text-gray-500 mb-4">
              {greeting || systemPrompt || `我是 ${agentName}，很高兴为您服务`}
            </p>
            {filteredPresetQuestions.length > 0 && (
              <div className="space-y-2">
                <p className="text-xs text-gray-400 mb-2">快速开始：</p>
                {filteredPresetQuestions.map((question, index) => (
                  <button
                    key={index}
                    onClick={() => handlePresetQuestionClick(question)}
                    className="block w-full text-left px-4 py-2 bg-white rounded-lg text-sm text-gray-700 hover:bg-blue-50 hover:text-blue-600 transition-colors shadow-sm"
                  >
                    {question}
                  </button>
                ))}
              </div>
            )}
          </div>
        )}

        {messages.map((message) => (
          <div
            key={message.id}
            className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            <div
              className={`max-w-[75%] rounded-2xl px-4 py-2 ${
                message.role === 'user'
                  ? 'bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-br-sm'
                  : 'bg-white text-gray-800 shadow-sm rounded-bl-sm'
              }`}
            >
              <p className="text-sm whitespace-pre-wrap break-words">{message.content}</p>
              <p
                className={`text-xs mt-1 ${
                  message.role === 'user' ? 'text-white/70' : 'text-gray-400'
                }`}
              >
                {message.timestamp.toLocaleTimeString('zh-CN', {
                  hour: '2-digit',
                  minute: '2-digit',
                })}
              </p>
            </div>
          </div>
        ))}

        {isTyping && (
          <div className="flex justify-start">
            <div className="bg-white rounded-2xl rounded-bl-sm px-4 py-3 shadow-sm">
              <div className="flex gap-1">
                <span className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0ms' }}></span>
                <span className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '150ms' }}></span>
                <span className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '300ms' }}></span>
              </div>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* 输入区域 */}
      <div className="bg-white border-t border-gray-200 p-3">
        <div className="flex items-end gap-2">
          <button className="p-2 text-gray-400 hover:text-gray-600 transition-colors">
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
          </button>
          <div className="flex-1 bg-gray-100 rounded-full px-4 py-2 flex items-center gap-2">
            <input
              type="text"
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="输入消息..."
              className="flex-1 bg-transparent outline-none text-sm text-gray-800 placeholder-gray-400"
              disabled={isTyping}
            />
            <button className="text-gray-400 hover:text-gray-600 transition-colors">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </button>
          </div>
          <button
            onClick={() => handleSendMessage(inputValue)}
            disabled={!inputValue.trim() || isTyping}
            className="p-2 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-full hover:shadow-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
            </svg>
          </button>
        </div>
      </div>

      {/* 手机底部指示器 */}
      <div className="bg-white h-5 flex items-center justify-center">
        <div className="w-32 h-1 bg-gray-300 rounded-full"></div>
      </div>
    </div>
  );
}
