import { apiClient } from './api';
import type { MCPTool, MCPToolVersion, MCPToolConfig, TestToolRequest, TestToolResponse } from '../types';

export interface CreateMCPToolRequest {
  name: string;
  description?: string;
  endpoint: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  headers?: Record<string, string>;
  parameters: ParameterSchema[];
  responseSchema?: Record<string, any>;
}

export interface ParameterSchema {
  name: string;
  type: string;
  description?: string;
  required: boolean;
  defaultValue?: any;
}

export interface UpdateMCPToolRequest {
  name?: string;
  description?: string;
  endpoint?: string;
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  headers?: Record<string, string>;
  parameters?: ParameterSchema[];
  responseSchema?: Record<string, any>;
  changeLog?: string;
}

class MCPService {
  async listTools(): Promise<MCPTool[]> {
    const response = await apiClient.get<{ tools: MCPTool[] }>('/mcp/tools');
    return response.data.tools;
  }

  async getTool(id: string): Promise<MCPTool> {
    const response = await apiClient.get<MCPTool>(`/mcp/tools/${id}`);
    return response.data;
  }

  async createTool(request: CreateMCPToolRequest): Promise<MCPTool> {
    const response = await apiClient.post<{ tool: MCPTool }>('/mcp/tools', request);
    return response.data.tool;
  }

  async updateTool(id: string, request: UpdateMCPToolRequest): Promise<MCPTool> {
    const response = await apiClient.put<{ tool: MCPTool }>(`/mcp/tools/${id}`, request);
    return response.data.tool;
  }

  async deleteTool(id: string): Promise<void> {
    await apiClient.delete(`/mcp/tools/${id}`);
  }

  async testTool(id: string, parameters: Record<string, any>): Promise<TestToolResponse> {
    const response = await apiClient.post<TestToolResponse>(`/mcp/tools/${id}/call`, { parameters });
    return response.data;
  }

  async getToolVersions(id: string): Promise<MCPToolVersion[]> {
    const response = await apiClient.get<{ versions: MCPToolVersion[] }>(`/mcp/tools/${id}/versions`);
    return response.data.versions;
  }

  async getToolVersion(id: string, version: number): Promise<MCPToolVersion> {
    const response = await apiClient.get<MCPToolVersion>(`/mcp/tools/${id}/versions/${version}`);
    return response.data;
  }

  async rollbackTool(id: string, targetVersion: number): Promise<MCPTool> {
    const response = await apiClient.post<{ tool: MCPTool }>(`/mcp/tools/${id}/rollback`, { targetVersion });
    return response.data.tool;
  }

  async toggleToolStatus(id: string, status: 'active' | 'inactive'): Promise<MCPTool> {
    const response = await apiClient.patch<{ tool: MCPTool }>(`/mcp/tools/${id}/status`, { status });
    return response.data.tool;
  }
}

export const mcpService = new MCPService();
