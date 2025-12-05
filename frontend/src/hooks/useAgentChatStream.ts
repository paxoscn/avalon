import { useState, useCallback, useRef } from 'react';

export interface ChatStreamChunk {
  type: 'content' | 'done' | 'error';
  content?: string;
  reasoning_content?: string;
  session_id?: string;
  message_id?: string;
  reply_id?: string;
  metadata?: {
    model?: string;
    tokens_used?: number;
    finish_reason?: string;
  };
  finish_reason?: string;
  error?: string;
}

export interface ChatMessage {
  id: string;
  role: 'User' | 'Assistant';
  content: string;
  reasoning?: string;
  timestamp: Date;
  metadata?: any;
}

export interface UseAgentChatStreamOptions {
  agentId: string;
  sessionId?: string;
  onChunk?: (chunk: ChatStreamChunk) => void;
  onComplete?: (message: ChatMessage) => void;
  onError?: (error: string) => void;
}

export function useAgentChatStream({
  agentId,
  sessionId: initialSessionId,
  onChunk,
  onComplete,
  onError,
}: UseAgentChatStreamOptions) {
  const [isStreaming, setIsStreaming] = useState(false);
  const [currentResponse, setCurrentResponse] = useState('');
  const [currentReasoning, setCurrentReasoning] = useState('');
  const [sessionId, setSessionId] = useState<string | undefined>(initialSessionId);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const abortControllerRef = useRef<AbortController | null>(null);

  const sendMessage = useCallback(
    async (message: string) => {
      console.log('???');
      if (isStreaming) {
        console.warn('Already streaming, please wait...');
        return;
      }

      setIsStreaming(true);
      setCurrentResponse('');
      setCurrentReasoning('');

      // 添加用户消息到列表
      const userMessage: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'User',
        content: message,
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, userMessage]);

      // 创建AbortController用于取消请求
      abortControllerRef.current = new AbortController();

      try {
        const token = localStorage.getItem('token'); // 根据实际情况调整
        const response = await fetch(`/api/agents/${agentId}/chat/stream`, {
          method: 'POST',
          headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': 'application/json',
            Accept: 'text/event-stream',
          },
          body: JSON.stringify({
            message,
            session_id: sessionId,
            stream: true,
          }),
          signal: abortControllerRef.current.signal,
        });

        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const reader = response.body?.getReader();
        if (!reader) {
          throw new Error('No response body');
        }

        const decoder = new TextDecoder();
        let buffer = '';
        let accumulatedContent = '';
        let accumulatedReasoning = '';

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split('\n');
          buffer = lines.pop() || '';

          for (const line of lines) {
            if (line.startsWith('data: ')) {
              try {
                const data: ChatStreamChunk = JSON.parse(line.slice(6));

                // 调用回调
                onChunk?.(data);

                // 更新session ID
                if (data.session_id && !sessionId) {
                  setSessionId(data.session_id);
                }

                if (data.type === 'content') {
                  if (data.content) {
                    accumulatedContent += data.content;
                    setCurrentResponse(accumulatedContent);
                    console.log('Content update:', data.content);
                  }
                  
                  if (data.reasoning_content) {
                    accumulatedReasoning += data.reasoning_content;
                    setCurrentReasoning(accumulatedReasoning);
                    console.log('Reasoning update:', data.reasoning_content);
                  }
                } else if (data.type === 'done') {
                  // 完成
                  const assistantMessage: ChatMessage = {
                    id: data.reply_id || crypto.randomUUID(),
                    role: 'Assistant',
                    content: accumulatedContent,
                    reasoning: accumulatedReasoning || undefined,
                    timestamp: new Date(),
                    metadata: data.metadata,
                  };

                  setMessages((prev) => [...prev, assistantMessage]);
                  onComplete?.(assistantMessage);
                  setIsStreaming(false);
                  setCurrentResponse('');
                  setCurrentReasoning('');
                } else if (data.type === 'error') {
                  const error_message = data.error || 'Unknown error';
                  onError?.(error_message);
                  setIsStreaming(false);
                  setCurrentResponse('');
                  setCurrentReasoning('');
                }
              } catch (e) {
                console.error('Failed to parse SSE data:', e);
              }
            } else if (line.startsWith(':')) {
              // 心跳消息，忽略
              continue;
            }
          }
        }
      } catch (error: any) {
        if (error.name === 'AbortError') {
          console.log('Request was cancelled');
        } else {
          console.error('Stream error:', error);
          onError?.(error.message || 'Stream error');
        }
        setIsStreaming(false);
        setCurrentResponse('');
        setCurrentReasoning('');
      } finally {
        abortControllerRef.current = null;
      }
    },
    [agentId, sessionId, isStreaming, onChunk, onComplete, onError]
  );

  const cancelStream = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      setIsStreaming(false);
      setCurrentResponse('');
      setCurrentReasoning('');
    }
  }, []);

  const clearMessages = useCallback(() => {
    setMessages([]);
    setCurrentResponse('');
    setCurrentReasoning('');
    setSessionId(undefined);
  }, []);

  return {
    messages,
    currentResponse,
    currentReasoning,
    isStreaming,
    sessionId,
    sendMessage,
    cancelStream,
    clearMessages,
  };
}
