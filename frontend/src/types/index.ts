export interface User {
  id: string;
  tenant_id: string;
  username: string;
  nickname?: string;
}

export interface AuthResponse {
  token: string;
  user: User;
  expiresAt: string;
}

export interface LoginRequest {
  tenant_id: string;
  username: string;
  password: string;
}

export interface ApiError {
  error: string;
  timestamp: string;
}

export interface Flow {
  id: string;
  tenant_id: string;
  name: string;
  description?: string;
  currentVersion: number;
  status: 'draft' | 'active' | 'archived';
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

export interface FlowVersion {
  id: string;
  flowId: string;
  version: number;
  definition: Record<string, any>;
  changeLog?: string;
  createdBy: string;
  createdAt: string;
}

export interface FlowExecution {
  id: string;
  flowId: string;
  flowVersion: number;
  tenant_id: string;
  userId: string;
  sessionId?: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  inputData?: Record<string, any>;
  outputData?: Record<string, any>;
  errorMessage?: string;
  startedAt: string;
  completedAt?: string;
  executionTimeMs?: number;
}

export interface ExecuteFlowRequest {
  variables?: Record<string, any>;
  sessionId?: string;
}

export interface ImportDifyRequest {
  dsl: string;
  name: string;
}

export interface ValidationResult {
  valid: boolean;
  errors?: string[];
  warnings?: string[];
}

export interface MCPTool {
  id: string;
  tenant_id: string;
  name: string;
  description?: string;
  currentVersion: number;
  status: 'active' | 'inactive';
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

export interface MCPToolVersion {
  id: string;
  toolId: string;
  version: number;
  config: MCPToolConfig;
  changeLog?: string;
  createdBy: string;
  createdAt: string;
}

export interface MCPToolConfig {
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

export interface TestToolRequest {
  parameters: Record<string, any>;
}

export interface TestToolResponse {
  result: any;
  executionTime: number;
  success: boolean;
  error?: string;
}

export interface AuditLog {
  id: string;
  tenant_id: string;
  userId?: string;
  action: string;
  resourceType: string;
  resourceId?: string;
  details?: Record<string, any>;
  ipAddress?: string;
  userAgent?: string;
  createdAt: string;
}

export interface LLMConfig {
  id: string;
  tenant_id: string;
  name: string;
  provider: 'openai' | 'claude' | 'local';
  config: Record<string, any>;
  isDefault: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface LLMTestResult {
  success: boolean;
  response?: string;
  executionTime: number;
  error?: string;
  usage?: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
}

export interface VectorConfig {
  id: string;
  tenant_id: string;
  name: string;
  provider: 'pinecone' | 'weaviate' | 'chromadb' | 'qdrant' | 'milvus';
  config: Record<string, any>;
  isDefault: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface VectorTestResult {
  success: boolean;
  message?: string;
  executionTime: number;
  error?: string;
  indexInfo?: {
    dimension?: number;
    count?: number;
    [key: string]: any;
  };
}

export interface ChatSession {
  id: string;
  tenant_id: string;
  userId: string;
  title?: string;
  context?: Record<string, any>;
  createdAt: string;
  updatedAt: string;
}

export interface ChatMessage {
  id: string;
  sessionId: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  metadata?: Record<string, any>;
  createdAt: string;
}

export interface SessionStats {
  totalSessions: number;
  totalMessages: number;
  averageMessagesPerSession: number;
  activeUsers: number;
  sessionsByUser: Array<{
    userId: string;
    count: number;
  }>;
  sessionTrend: Array<{
    date: string;
    count: number;
  }>;
}
