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
  current_version: number;
  status: 'Draft' | 'Active' | 'Archived';
  created_by: string;
  created_at: string;
  updated_at: string;
}

export interface FlowVersion {
  id: string;
  flowId: string;
  version: number;
  definition: Record<string, any>;
  changeLog?: string;
  created_by: string;
  created_at: string;
}

export interface FlowExecution {
  id: string;
  flowId: string;
  flow_version: number;
  tenant_id: string;
  userId: string;
  sessionId?: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  input_data?: Record<string, any>;
  output_data?: Record<string, any>;
  error_message?: string;
  started_at: string;
  completed_at?: string;
  execution_time_ms?: number;
}

export interface ExecuteFlowRequest {
  variables?: Record<string, any>;
  sessionId?: string;
}

export interface ImportDslRequest {
  dsl: string;
  name: string;
}

export interface ValidationResult {
  is_valid: boolean;
  errors?: string[];
  warnings?: string[];
}

export interface MCPTool {
  id: string;
  tenant_id: string;
  name: string;
  description?: string;
  current_version: number;
  status: 'active' | 'inactive';
  created_by: string;
  created_at: string;
  updated_at: string;
}

export interface MCPToolVersion {
  id: string;
  toolId: string;
  version: number;
  config: MCPToolConfig;
  changeLog?: string;
  created_by: string;
  created_at: string;
}

export interface MCPToolConfig {
  HTTP: HTTPMCPToolConfig;
}

export interface HTTPMCPToolConfig {
  endpoint: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  headers?: Record<string, string>;
  parameters: ParameterSchema[];
  timeout_seconds?: number;
  retry_count?: number;
  response_template?: string;
}

export interface ParameterSchema {
  name: string;
  parameter_type: string;
  description?: string;
  required: boolean;
  default_value?: any;
  enum_values?: any[];
  position?: 'body' | 'header' | 'path';
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
  created_at: string;
}

export interface LLMConfig {
  id: string;
  tenant_id: string;
  name: string;
  provider: 'openai' | 'claude' | 'local';
  model_name: string;
  config: Record<string, any>;
  is_default: boolean;
  created_at: string;
  updated_at: string;
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
  created_at: string;
  updated_at: string;
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
  created_at: string;
  updated_at: string;
}

export interface ChatMessage {
  id: string;
  sessionId: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  metadata?: Record<string, any>;
  created_at: string;
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

export interface Agent {
  id: string;
  tenant_id: string;
  name: string;
  avatar?: string;
  greeting?: string;
  knowledge_base_ids: string[];
  mcp_tool_ids: string[];
  flow_ids: string[];
  system_prompt: string;
  additional_settings?: string;
  preset_questions: string[];
  source_agent_id?: string;
  creator_id: string;
  created_at: string;
  updated_at: string;
}

export interface AgentEmployment {
  id: string;
  agent_id: string;
  user_id: string;
  tenant_id: string;
  employed_at: string;
}

export interface AgentAllocation {
  id: string;
  agent_id: string;
  user_id: string;
  tenant_id: string;
  allocated_at: string;
}
