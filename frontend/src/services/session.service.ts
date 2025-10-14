import { apiClient } from './api';
import type { ChatSession, ChatMessage, SessionStats } from '../types';

export interface SessionFilters {
  userId?: string;
  startDate?: string;
  endDate?: string;
  search?: string;
  page?: number;
  limit?: number;
}

export interface SessionListResponse {
  sessions: ChatSession[];
  total: number;
  page: number;
  limit: number;
}

export interface MessageListResponse {
  messages: ChatMessage[];
  total: number;
}

class SessionService {
  async getSessions(filters: SessionFilters = {}): Promise<SessionListResponse> {
    const response = await apiClient.get<SessionListResponse>('/sessions', {
      params: filters,
    });
    return response.data;
  }

  async getSessionById(id: string): Promise<ChatSession> {
    const response = await apiClient.get<ChatSession>(`/sessions/${id}`);
    return response.data;
  }

  async getSessionMessages(sessionId: string): Promise<MessageListResponse> {
    const response = await apiClient.get<MessageListResponse>(`/sessions/${sessionId}/messages`);
    return response.data;
  }

  async getSessionStats(filters?: { startDate?: string; endDate?: string }): Promise<SessionStats> {
    const response = await apiClient.get<SessionStats>('/sessions/stats', {
      params: filters,
    });
    return response.data;
  }

  async deleteSession(id: string): Promise<void> {
    await apiClient.delete(`/sessions/${id}`);
  }

  async exportSession(sessionId: string, format: 'json' | 'txt' = 'json'): Promise<Blob> {
    const response = await apiClient.get(`/sessions/${sessionId}/export`, {
      params: { format },
      responseType: 'blob',
    });
    return response.data;
  }

  async searchSessions(query: string): Promise<ChatSession[]> {
    const response = await apiClient.get<{ sessions: ChatSession[] }>('/sessions/search', {
      params: { q: query },
    });
    return response.data.sessions;
  }
}

export const sessionService = new SessionService();
