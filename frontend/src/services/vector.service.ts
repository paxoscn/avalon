import { apiClient } from './api';
import type { VectorConfig, VectorTestResult } from '../types';

export interface CreateVectorConfigRequest {
  name: string;
  provider: 'pinecone' | 'weaviate' | 'chromadb' | 'qdrant' | 'milvus';
  config: VectorProviderConfig;
  isDefault?: boolean;
}

export interface UpdateVectorConfigRequest {
  name?: string;
  provider?: 'pinecone' | 'weaviate' | 'chromadb' | 'qdrant' | 'milvus';
  config?: VectorProviderConfig;
  isDefault?: boolean;
}

export interface VectorProviderConfig {
  apiKey?: string;
  apiUrl?: string;
  environment?: string;
  indexName?: string;
  dimension?: number;
  metric?: 'cosine' | 'euclidean' | 'dotproduct';
  [key: string]: any;
}

export interface UpsertVectorRequest {
  vectors: VectorRecord[];
}

export interface VectorRecord {
  id: string;
  vector: number[];
  metadata?: Record<string, any>;
}

export interface QueryVectorRequest {
  vector: number[];
  topK: number;
  filter?: Record<string, any>;
}

export interface QueryVectorResponse {
  results: VectorSearchResult[];
  executionTime: number;
}

export interface VectorSearchResult {
  id: string;
  score: number;
  metadata?: Record<string, any>;
}

class VectorService {
  async listConfigs(): Promise<VectorConfig[]> {
    const response = await apiClient.get<{ data: VectorConfig[] }>('/config/vector');
    return response.data.data;
  }

  async getConfig(id: string): Promise<VectorConfig> {
    const response = await apiClient.get<VectorConfig>(`/config/vector/${id}`);
    return response.data;
  }

  async createConfig(request: CreateVectorConfigRequest): Promise<VectorConfig> {
    const response = await apiClient.post<{ config: VectorConfig }>('/config/vector', request);
    return response.data.config;
  }

  async updateConfig(id: string, request: UpdateVectorConfigRequest): Promise<VectorConfig> {
    const response = await apiClient.put<{ config: VectorConfig }>(`/config/vector/${id}`, request);
    return response.data.config;
  }

  async deleteConfig(id: string): Promise<void> {
    await apiClient.delete(`/config/vector/${id}`);
  }

  async testConnection(id: string): Promise<VectorTestResult> {
    const response = await apiClient.post<VectorTestResult>(`/config/vector/${id}/test-connection`);
    return response.data;
  }

  async setDefault(id: string): Promise<VectorConfig> {
    const response = await apiClient.post<{ config: VectorConfig }>(`/config/vector/${id}/set-default`);
    return response.data.config;
  }

  async upsertVectors(id: string, request: UpsertVectorRequest): Promise<{ success: boolean }> {
    const response = await apiClient.post<{ success: boolean }>(`/vector/${id}/upsert`, request);
    return response.data;
  }

  async queryVectors(id: string, request: QueryVectorRequest): Promise<QueryVectorResponse> {
    const response = await apiClient.post<QueryVectorResponse>(`/vector/${id}/query`, request);
    return response.data;
  }

  async getIndexInfo(id: string): Promise<{ dimension?: number; count?: number; [key: string]: any }> {
    const response = await apiClient.get<{ dimension?: number; count?: number; [key: string]: any }>(
      `/vector/${id}/info`
    );
    return response.data;
  }
}

export const vectorService = new VectorService();
