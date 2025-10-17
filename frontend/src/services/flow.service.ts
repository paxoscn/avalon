import { apiClient } from './api';
import type {
  Flow,
  FlowVersion,
  FlowExecution,
  ExecuteFlowRequest,
  ImportDslRequest,
  ValidationResult,
} from '../types';

export interface FlowListParams {
  page?: number;
  limit?: number;
  status?: string;
}

export interface FlowListResponse {
  flows: Flow[];
  total: number;
}

export interface ExecuteFlowResponse {
  executionId: string;
  status: string;
}

export interface ImportDslResponse {
  flow: Flow;
  validation: ValidationResult;
}

export interface RollbackResponse {
  flow: Flow;
  success: boolean;
}

class FlowService {
  async getFlows(params?: FlowListParams): Promise<FlowListResponse> {
    const response = await apiClient.get<FlowListResponse>('/flows', { params });
    return response.data;
  }

  async getFlowById(id: string): Promise<Flow> {
    const response = await apiClient.get<Flow>(`/flows/${id}`);
    return response.data;
  }

  async createFlow(data: Partial<Flow>): Promise<Flow> {
    const response = await apiClient.post<Flow>('/flows', data);
    return response.data;
  }

  async updateFlow(id: string, data: Partial<Flow>): Promise<Flow> {
    const response = await apiClient.put<Flow>(`/flows/${id}`, data);
    return response.data;
  }

  async deleteFlow(id: string): Promise<void> {
    await apiClient.delete(`/flows/${id}`);
  }

  async executeFlow(id: string, data: ExecuteFlowRequest): Promise<ExecuteFlowResponse> {
    const response = await apiClient.post<ExecuteFlowResponse>(`/flows/${id}/execute`, data);
    return response.data;
  }

  async getFlowVersions(id: string): Promise<FlowVersion[]> {
    const response = await apiClient.get<{ versions: FlowVersion[] }>(`/flows/${id}/versions`);
    return response.data.versions;
  }

  async rollbackFlow(id: string, targetVersion: number): Promise<RollbackResponse> {
    const response = await apiClient.post<RollbackResponse>(`/flows/${id}/rollback`, {
      target_version: targetVersion,
    });
    return response.data;
  }

  async importDsl(data: ImportDslRequest): Promise<ImportDslResponse> {
    const response = await apiClient.post<ImportDslResponse>('/flows/import-dsl', data);
    return response.data;
  }

  async getFlowExecutions(flowId: string): Promise<FlowExecution[]> {
    const response = await apiClient.get<{ executions: FlowExecution[] }>(
      `/flows/${flowId}/executions`
    );
    return response.data.executions;
  }

  async getExecutionById(executionId: string): Promise<FlowExecution> {
    const response = await apiClient.get<FlowExecution>(`/executions/${executionId}`);
    return response.data;
  }
}

export const flowService = new FlowService();
