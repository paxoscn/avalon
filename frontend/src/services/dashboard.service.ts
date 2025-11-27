import { apiClient } from './api';

export interface DashboardStats {
  agents_count: number;
  flows_count: number;
  mcp_tools_count: number;
  knowledge_bases_count: number;
  sessions_count: number;
}

class DashboardService {
  async getStats(): Promise<DashboardStats> {
    const response = await apiClient.get<DashboardStats>('/dashboard/stats');
    return response.data;
  }
}

export const dashboardService = new DashboardService();
