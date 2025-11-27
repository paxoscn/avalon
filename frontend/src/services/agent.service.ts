import { apiClient } from './api';
import type { Agent, AgentUsageStatsParams, AgentUsageStatsResponse } from '../types';

export interface CreateAgentRequest {
  name: string;
  avatar?: string;
  greeting?: string;
  system_prompt: string;
  additional_settings?: string;
  preset_questions?: string[];
  knowledge_base_ids?: string[];
  mcp_tool_ids?: string[];
  flow_ids?: string[];
}

export interface UpdateAgentRequest {
  name?: string;
  avatar?: string;
  greeting?: string;
  system_prompt?: string;
  additional_settings?: string;
  preset_questions?: string[];
}

export interface ListAgentsParams {
  page?: number;
  page_size?: number;
}

export interface ListAgentsResponse {
  items: Agent[];
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
}

class AgentService {
  async listAgents(params?: ListAgentsParams): Promise<ListAgentsResponse> {
    const response = await apiClient.get<ListAgentsResponse>('/agents', { params });
    return response.data;
  }

  async getAgent(id: string): Promise<Agent> {
    const response = await apiClient.get<Agent>(`/agents/${id}`);
    return response.data;
  }

  async createAgent(request: CreateAgentRequest): Promise<Agent> {
    const response = await apiClient.post<{ agent: Agent }>('/agents', request);
    return response.data.agent;
  }

  async updateAgent(id: string, request: UpdateAgentRequest): Promise<Agent> {
    const response = await apiClient.put<{ agent: Agent }>(`/agents/${id}`, request);
    return response.data.agent;
  }

  async deleteAgent(id: string): Promise<void> {
    await apiClient.delete(`/agents/${id}`);
  }

  async copyAgent(id: string): Promise<Agent> {
    const response = await apiClient.post<{ agent: Agent }>(`/agents/${id}/copy`);
    return response.data.agent;
  }

  async employAgent(id: string): Promise<void> {
    await apiClient.post(`/agents/${id}/employ`);
  }

  async terminateEmployment(id: string): Promise<void> {
    await apiClient.delete(`/agents/${id}/employ`);
  }

  async listEmployedAgents(params?: ListAgentsParams): Promise<ListAgentsResponse> {
    const response = await apiClient.get<ListAgentsResponse>('/agents/employed', { params });
    return response.data;
  }

  async allocateAgent(id: string): Promise<void> {
    await apiClient.post(`/agents/${id}/allocate`);
  }

  async terminateAllocation(id: string): Promise<void> {
    await apiClient.delete(`/agents/${id}/allocate`);
  }

  async listAllocatedAgents(params?: ListAgentsParams): Promise<ListAgentsResponse> {
    const response = await apiClient.get<ListAgentsResponse>('/agents/allocated', { params });
    return response.data;
  }

  async listCreatedAgents(params?: ListAgentsParams): Promise<ListAgentsResponse> {
    const response = await apiClient.get<ListAgentsResponse>('/agents/created', { params });
    return response.data;
  }

  async addKnowledgeBase(agentId: string, configId: string): Promise<void> {
    await apiClient.post(`/agents/${agentId}/knowledge-bases/${configId}`);
  }

  async removeKnowledgeBase(agentId: string, configId: string): Promise<void> {
    await apiClient.delete(`/agents/${agentId}/knowledge-bases/${configId}`);
  }

  async addMcpTool(agentId: string, toolId: string): Promise<void> {
    await apiClient.post(`/agents/${agentId}/mcp-tools/${toolId}`);
  }

  async removeMcpTool(agentId: string, toolId: string): Promise<void> {
    await apiClient.delete(`/agents/${agentId}/mcp-tools/${toolId}`);
  }

  async addFlow(agentId: string, flowId: string): Promise<void> {
    await apiClient.post(`/agents/${agentId}/flows/${flowId}`);
  }

  async removeFlow(agentId: string, flowId: string): Promise<void> {
    await apiClient.delete(`/agents/${agentId}/flows/${flowId}`);
  }

  async getAgentUsageStats(agentId: string, params?: AgentUsageStatsParams): Promise<AgentUsageStatsResponse> {
    const response = await apiClient.get<AgentUsageStatsResponse>(`/agents/${agentId}/stats`, { params });
    return response.data;
  }

  async startInterview(id: string): Promise<void> {
    await apiClient.post(`/agents/${id}/interview/start`);
  }

  async completeInterview(id: string, passed: boolean): Promise<void> {
    await apiClient.post(`/agents/${id}/interview/complete`, { passed });
  }

  async publishAgent(id: string): Promise<void> {
    await apiClient.post(`/agents/${id}/publish`);
  }

  async unpublishAgent(id: string): Promise<void> {
    await apiClient.post(`/agents/${id}/unpublish`);
  }
}

export const agentService = new AgentService();
