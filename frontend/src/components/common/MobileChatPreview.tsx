import { useState, useRef, useEffect } from 'react';
import { chatService } from '../../services/chat.service';

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  reasoning?: string;
  timestamp: Date;
}

export interface MobileChatPreviewProps {
  agentId?: string;
  agentName: string;
  agentAvatar?: string;
  greeting?: string;
  systemPrompt?: string;
  presetQuestions?: string[];
  onSendMessage?: (message: string) => Promise<string>;
  onFirstMessage?: () => Promise<void>;
  className?: string;
}

export function MobileChatPreview({
  agentId,
  agentName,
  agentAvatar,
  greeting,
  systemPrompt,
  presetQuestions = [],
  onSendMessage,
  onFirstMessage,
  className = '',
}: MobileChatPreviewProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isTyping, setIsTyping] = useState(false);
  const [currentResponse, setCurrentResponse] = useState('');
  const [currentReasoning, setCurrentReasoning] = useState('');
  const [sessionId, setSessionId] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);
  const [isFirstMessage, setIsFirstMessage] = useState(true);
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

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSendMessage = async (content: string) => {
    if (!content.trim()) return;

    // Call onFirstMessage callback if this is the first message
    if (isFirstMessage && onFirstMessage) {
      try {
        await onFirstMessage();
      } catch (error) {
        console.error('Failed to record first message:', error);
        // Continue with sending the message even if tracking fails
      }
      setIsFirstMessage(false);
    }

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: content.trim(),
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInputValue('');
    setIsTyping(true);
    setError(null);

    try {
      let responseContent = '';
      
      // Use custom onSendMessage if provided
      if (onSendMessage) {
        responseContent = await onSendMessage(content);
        
        const assistantMessage: ChatMessage = {
          id: (Date.now() + 1).toString(),
          role: 'assistant',
          content: responseContent,
          timestamp: new Date(),
        };
        setMessages((prev) => [...prev, assistantMessage]);
      } 
      // Use real chat service with SSE streaming if agentId is provided
      else if (agentId) {
        let fullContent = '';
        let fullReasoning = '';
        
        await chatService.chatStream(
          {
            agentId,
            message: content.trim(),
            sessionId,
          },
          {
            onContent: (replyId, chunk) => {
              console.log("Content chunk:", chunk, replyId);
              
              fullContent += chunk;
              setCurrentResponse(fullContent);

              const assistantMessage: ChatMessage = {
                id: replyId,
                role: 'assistant',
                content: fullContent,
                reasoning: fullReasoning || undefined,
                timestamp: new Date(),
              };
              setMessages((prev) => {
                if (prev.length > 0 && prev[prev.length - 1].id === replyId) {
                  prev[prev.length - 1].content = fullContent;
                  return prev
                } else {
                  return [...prev, assistantMessage]
                }
              });
            },
            onReasoning: (chunk) => {
              console.log("Reasoning chunk:", chunk);
              fullReasoning += chunk;
              // 流式响应时，显示在临时状态中
              setCurrentReasoning(fullReasoning);
            },
            onDone: (data) => {
              console.log("???");
              // Update session ID if this is the first message
              if (!sessionId) {
                setSessionId(data.sessionId);
              }
              
              // 清空临时的思考过程显示
              setCurrentReasoning('');
              
              // 思考过程结束后，添加完整的消息到列表
              const assistantMessage: ChatMessage = {
                id: data.replyId,
                role: 'assistant',
                content: fullContent,
                reasoning: fullReasoning || undefined,
                timestamp: new Date(),
              };
              setMessages((prev) => [...prev, assistantMessage]);
            },
            onError: (errorMsg) => {
              setError(errorMsg);
              setCurrentReasoning('');
              // 添加错误消息
              const errorMessage: ChatMessage = {
                id: `error-${Date.now()}`,
                role: 'assistant',
                content: `抱歉，${errorMsg}`,
                timestamp: new Date(),
              };
              setMessages((prev) => [...prev, errorMessage]);
            },
          }
        );
      } 
      // Fallback to simulation
      else {
        await new Promise((resolve) => setTimeout(resolve, 1000));
        responseContent = `这是一个模拟回复。我是 ${agentName}，收到了您的消息："${content}"`;
        
        const assistantMessage: ChatMessage = {
          id: (Date.now() + 1).toString(),
          role: 'assistant',
          content: responseContent,
          timestamp: new Date(),
        };
        setMessages((prev) => [...prev, assistantMessage]);
      }
    } catch (error) {
      console.error('Failed to send message:', error);
      const errorMessage = error instanceof Error ? error.message : '发送消息时出现错误';
      setError(errorMessage);
      
      const error_message: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: `抱歉，${errorMessage}`,
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
    <div className={`flex flex-col bg-white rounded-2xl shadow-xl overflow-hidden h-[max(600px,80vh)] ${className}`}>
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
      <div className="flex-1 overflow-y-auto bg-gray-50 p-4 space-y-4">
        {error && (
          <div className="bg-red-50 border border-red-200 rounded-lg p-3 mb-2">
            <p className="text-sm text-red-600">{error}</p>
          </div>
        )}
        
        {messages.length === 0 && (
          <div className="text-center py-8">
            {agentAvatar ? (
              <img
                src={agentAvatar}
                alt={agentName}
                className="inline-flex w-16 h-16 rounded-full items-center justify-center object-cover border-2 border-white"
              />
            ) : (
              <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gradient-to-br from-blue-400 to-purple-500 text-white text-2xl font-bold mb-3">
                {agentName.charAt(0).toUpperCase()}
              </div>
            )}
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
              {/* 思考过程按钮 */}
              {message.reasoning && message.role === 'assistant' && (
                <div className="mt-2">
                  <button
                    onClick={() => toggleReasoning(message.id)}
                    className="flex items-center space-x-1 text-xs text-amber-700 hover:text-amber-800 font-medium transition-colors"
                  >
                    <svg 
                      className={`w-3 h-3 transition-transform ${expandedReasonings.has(message.id) ? 'rotate-90' : ''}`}
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
                    <div className="mt-2 p-2 bg-gradient-to-r from-amber-50 to-orange-50 border border-amber-200 rounded-lg">
                      <p className="text-xs text-amber-900 whitespace-pre-wrap break-words leading-relaxed">
                        {message.reasoning}
                      </p>
                    </div>
                  )}
                </div>
              )}
              
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

        {/* 流式响应时的思考过程 */}
        {isTyping && currentReasoning && (
          <div className="flex justify-start">
            <div className="max-w-[75%] bg-gradient-to-r from-amber-50 to-orange-50 border border-amber-200 rounded-2xl rounded-bl-sm px-3 py-2 shadow-sm">
              <div className="flex items-center space-x-1 mb-1">
                <svg className="w-3 h-3 text-amber-600 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                <span className="text-xs font-semibold text-amber-700">💭 思考中</span>
              </div>
              <p className="text-xs text-amber-900 whitespace-pre-wrap break-words leading-relaxed">
                {currentReasoning}
                <span className="inline-block w-0.5 h-3 bg-amber-600 animate-pulse ml-0.5 align-middle"></span>
              </p>
            </div>
          </div>
        )}

        {isTyping && !currentReasoning && (
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
