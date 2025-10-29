import { apiClient } from './api';
import type { FlowExecution } from '../types';

export interface ExecutionFilters {
  flowId?: string;
  userId?: string;
  status?: string;
  startDate?: string;
  endDate?: string;
  page?: number;
  limit?: number;
}

export interface ExecutionListResponse {
  executions: FlowExecution[];
  total: number;
  page: number;
  limit: number;
}

export interface ExecutionStep {
  id: string;
  executionId: string;
  stepName: string;
  stepType: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'skipped';
  input_data?: Record<string, any>;
  output_data?: Record<string, any>;
  error_message?: string;
  started_at: string;
  completed_at?: string;
  execution_time_ms?: number;
}

export interface ExecutionStats {
  totalExecutions: number;
  successRate: number;
  averageExecutionTime: number;
  executionsByStatus: Record<string, number>;
  executionsByFlow: Array<{
    flowId: string;
    flowName: string;
    count: number;
  }>;
  executionTrend: Array<{
    date: string;
    total: number;
    successful: number;
    failed: number;
  }>;
}

export interface PerformanceMetrics {
  executionId: string;
  totalTime: number;
  stepMetrics: Array<{
    stepName: string;
    executionTime: number;
    percentage: number;
  }>;
  bottlenecks: Array<{
    stepName: string;
    executionTime: number;
    reason: string;
  }>;
}

class ExecutionService {
  async getExecutions(filters: ExecutionFilters = {}): Promise<ExecutionListResponse> {
    const response = await apiClient.get<ExecutionListResponse>('/executions', {
      params: filters,
    });
    return response.data;
  }

  async getExecutionById(id: string): Promise<FlowExecution> {
    const response = await apiClient.get<FlowExecution>(`/executions/${id}`);
    return response.data;
  }

  async getExecutionSteps(executionId: string): Promise<ExecutionStep[]> {
    const response = await apiClient.get<{ steps: ExecutionStep[] }>(
      `/executions/${executionId}/steps`
    );
    return response.data.steps;
  }

  async getExecutionStats(filters?: { startDate?: string; endDate?: string }): Promise<ExecutionStats> {
    const response = await apiClient.get<ExecutionStats>('/executions/stats', {
      params: filters,
    });
    return response.data;
  }

  async getPerformanceMetrics(executionId: string): Promise<PerformanceMetrics> {
    const response = await apiClient.get<PerformanceMetrics>(
      `/executions/${executionId}/performance`
    );
    return response.data;
  }

  async cancelExecution(executionId: string): Promise<void> {
    await apiClient.post(`/executions/${executionId}/cancel`);
  }

  async retryExecution(executionId: string): Promise<{ executionId: string }> {
    const response = await apiClient.post<{ executionId: string }>(
      `/executions/${executionId}/retry`
    );
    return response.data;
  }
}

export const executionService = new ExecutionService();
