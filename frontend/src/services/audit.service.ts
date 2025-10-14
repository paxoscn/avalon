import { apiClient } from './api';
import type { AuditLog } from '../types';

export interface AuditLogFilters {
  userId?: string;
  action?: string;
  resourceType?: string;
  resourceId?: string;
  startDate?: string;
  endDate?: string;
  page?: number;
  limit?: number;
}

export interface AuditLogListResponse {
  logs: AuditLog[];
  total: number;
  page: number;
  limit: number;
}

export interface AuditStats {
  totalActions: number;
  actionsByType: Record<string, number>;
  actionsByUser: Record<string, number>;
  recentActivity: Array<{
    date: string;
    count: number;
  }>;
}

class AuditService {
  async getAuditLogs(filters: AuditLogFilters = {}): Promise<AuditLogListResponse> {
    const response = await apiClient.get<AuditLogListResponse>('/audit/logs', {
      params: filters,
    });
    return response.data;
  }

  async getAuditLogById(id: string): Promise<AuditLog> {
    const response = await apiClient.get<AuditLog>(`/audit/logs/${id}`);
    return response.data;
  }

  async getAuditStats(filters?: { startDate?: string; endDate?: string }): Promise<AuditStats> {
    const response = await apiClient.get<AuditStats>('/audit/stats', {
      params: filters,
    });
    return response.data;
  }

  async exportAuditLogs(filters: AuditLogFilters = {}, format: 'csv' | 'json' = 'csv'): Promise<Blob> {
    const response = await apiClient.get('/audit/logs/export', {
      params: { ...filters, format },
      responseType: 'blob',
    });
    return response.data;
  }
}

export const auditService = new AuditService();
