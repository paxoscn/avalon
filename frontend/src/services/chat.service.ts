import { apiClient } from './api';

export interface CreateSessionRequest {
  title?: string;
}

export interface AddMessageRequest {
  role: 'user' | 'assistant' | 'system';
  content: string;
  metadata?: Record<string, any>;
}

export interface MessageResponse {
  id: string;
  session_id: string;
  role: string;
  content: string;
  metadata?: Record<string, any>;
  created_at: string;
}

export interface SessionResponse {
  id: string;
  tenant_id: string;
  user_id: string;
  title?: string;
  created_at: string;
  updated_at: string;
}

export interface ChatRequest {
  agentId: string;
  message: string;
  sessionId?: string;
}

export interface ChatResponse {
  sessionId: string;
  message: MessageResponse;
  reply: MessageResponse;
}

class ChatService {
  /**
   * Create a new chat session
   */
  async createSession(title?: string): Promise<SessionResponse> {
    const response = await apiClient.post<SessionResponse>('/sessions', {
      title,
    });
    return response.data;
  }

  /**
   * Add a message to a session
   */
  async addMessage(
    sessionId: string,
    role: 'user' | 'assistant' | 'system',
    content: string,
    metadata?: Record<string, any>
  ): Promise<MessageResponse> {
    const response = await apiClient.post<MessageResponse>(
      `/sessions/${sessionId}/messages`,
      {
        role,
        content,
        metadata,
      }
    );
    return response.data;
  }

  /**
   * Send a chat message and get a response
   * This creates a session if needed, sends the user message,
   * and returns the assistant's response
   */
  async chat(request: ChatRequest): Promise<ChatResponse> {
    // Call the agent chat endpoint
    const response = await apiClient.post<{
      session_id: string;
      message_id: string;
      reply_id: string;
      reply: string;
      metadata?: Record<string, any>;
    }>(`/agents/${request.agentId}/chat`, {
      message: request.message,
      session_id: request.sessionId,
      stream: false,
    });

    // Convert the response to our format
    return {
      sessionId: response.data.session_id,
      message: {
        id: response.data.message_id,
        session_id: response.data.session_id,
        role: 'user',
        content: request.message,
        created_at: new Date().toISOString(),
      },
      reply: {
        id: response.data.reply_id,
        session_id: response.data.session_id,
        role: 'assistant',
        content: response.data.reply,
        metadata: response.data.metadata,
        created_at: new Date().toISOString(),
      },
    };
  }

  /**
   * Get messages from a session
   */
  async getSessionMessages(sessionId: string): Promise<MessageResponse[]> {
    const response = await apiClient.get<{ messages: MessageResponse[] }>(
      `/sessions/${sessionId}/messages`
    );
    return response.data.messages || [];
  }

  /**
   * Set context variable for a session
   */
  async setContext(
    sessionId: string,
    key: string,
    value: any
  ): Promise<void> {
    await apiClient.post(`/sessions/${sessionId}/context`, {
      key,
      value,
    });
  }

  /**
   * Get context variable from a session
   */
  async getContext(sessionId: string, key: string): Promise<any> {
    const response = await apiClient.get<{ value: any }>(
      `/sessions/${sessionId}/context/${key}`
    );
    return response.data.value;
  }
}

export const chatService = new ChatService();
