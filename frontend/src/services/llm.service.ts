import { apiClient } from './api';
import type { LLMConfig, LLMTestResult } from '../types';

export interface CreateLLMConfigRequest {
  name: string;
  provider: 'openai' | 'claude' | 'local';
  model_name: string;
  parameters: LLMParametersConfig;
  credentials: LLMCredentialsConfig;
  isDefault?: boolean;
}

export interface UpdateLLMConfigRequest {
  name?: string;
  provider?: 'openai' | 'claude' | 'local';
  model_name: string;
  parameters: LLMParametersConfig;
  credentials: LLMCredentialsConfig;
  isDefault?: boolean;
}

export interface LLMProviderConfig {
  apiKey?: string;
  apiUrl?: string;
  model?: string;
  temperature?: number;
  maxTokens?: number;
  topP?: number;
  frequencyPenalty?: number;
  presencePenalty?: number;
  [key: string]: any;
}

export interface LLMParametersConfig {
  temperature?: number;
  max_tokens?: number;
  top_p?: number;
  frequency_penalty?: number;
  presence_penalty?: number;
  [key: string]: any;
}

export interface LLMCredentialsConfig {
  api_key?: string;
  api_base?: string;
  organization?: string;
  [key: string]: any;
}

export interface TestLLMRequest {
  user_prompt: string;
  system_prompt?: string;
}

class LLMService {
  async listConfigs(): Promise<LLMConfig[]> {
    const response = await apiClient.get<{ data: LLMConfig[] }>('/config/llm');
    return response.data.data;
  }

  async getConfig(id: string): Promise<LLMConfig> {
    const response = await apiClient.get<LLMConfig>(`/config/llm/${id}`);
    return response.data;
  }

  async createConfig(request: CreateLLMConfigRequest): Promise<LLMConfig> {
    const response = await apiClient.post<{ config: LLMConfig }>('/config/llm', request);
    return response.data.config;
  }

  async updateConfig(id: string, request: UpdateLLMConfigRequest): Promise<LLMConfig> {
    const response = await apiClient.put<{ config: LLMConfig }>(`/config/llm/${id}`, request);
    return response.data.config;
  }

  async deleteConfig(id: string): Promise<void> {
    await apiClient.delete(`/config/llm/${id}`);
  }

  async testConfig(id: string, request: TestLLMRequest): Promise<LLMTestResult> {
    const response = await apiClient.post<{
      success: boolean;
      response_time_ms: number;
      error_message?: string;
      response?: string;
      usage?: {
        prompt_tokens: number;
        completion_tokens: number;
        total_tokens: number;
      };
    }>(`/config/llm/${id}/test`, request);
    
    // Map backend response to frontend format
    return {
      success: response.data.success,
      response: response.data.response,
      executionTime: response.data.response_time_ms,
      error: response.data.error_message,
      usage: response.data.usage ? {
        promptTokens: response.data.usage.prompt_tokens,
        completionTokens: response.data.usage.completion_tokens,
        totalTokens: response.data.usage.total_tokens,
      } : undefined,
    };
  }

  async testConnection(id: string): Promise<{ success: boolean; message?: string }> {
    const response = await apiClient.post<{ success: boolean; message?: string }>(
      `/config/llm/${id}/test-connection`
    );
    return response.data;
  }

  async setDefault(id: string): Promise<LLMConfig> {
    const response = await apiClient.post<{ config: LLMConfig }>(`/config/llm/${id}/set-default`);
    return response.data.config;
  }
}

export const llmService = new LLMService();
