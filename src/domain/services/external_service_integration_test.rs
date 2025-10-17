/// Integration tests for external service calls in flow execution
/// This module tests the integration of LLM, Vector Search, and MCP Tool nodes
#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::collections::HashMap;
    use serde_json::{json, Value};
    use async_trait::async_trait;
    
    use crate::domain::services::{
        execution_engine::{ExecutionEngine, ExecutionEngineImpl, NodeExecutor, ExecutionState},
        node_executors::*,
        llm_service::{LLMDomainService, ChatResponse, TokenUsage, FinishReason, LLMError, ChatRequest, ChatStreamChunk, ValidationResult, ModelInfo, ConnectionTestResult},
        vector_service::{VectorStoreDomainService},
        mcp_tool_service::{MCPToolDomainService, ToolCallContext, ToolCallResult, PermissionCheckResult, ConfigValidationResult},
    };
    use crate::domain::value_objects::{
        FlowNode, NodeType, NodePosition, FlowDefinition, FlowEdge, FlowMetadata,
        ChatMessage, ModelConfig, ModelProvider, ModelParameters, ModelCredentials,
        SearchQuery, SearchResult, TenantId, UserId, FlowExecutionId,
        tool_config::ToolConfig,
    };
    use crate::domain::entities::{FlowExecution, MCPTool};
    use crate::domain::repositories::mcp_tool_repository::MCPToolRepository;
    use crate::error::{Result, PlatformError};

    // Mock LLM Service
    struct MockLLMService;

    #[async_trait]
    impl LLMDomainService for MockLLMService {
        async fn chat_completion(
            &self,
            _config: &ModelConfig,
            messages: Vec<ChatMessage>,
            _tenant_id: uuid::Uuid,
        ) -> std::result::Result<ChatResponse, LLMError> {
            // Return a mock response based on the last message
            let last_message = messages.last().map(|m| m.content.clone()).unwrap_or_default();
            Ok(ChatResponse {
                content: format!("Mock LLM response to: {}", last_message),
                model_used: "mock-model".to_string(),
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                },
                finish_reason: FinishReason::Stop,
                metadata: None,
            })
        }

        async fn generate_embedding(
            &self,
            _config: &ModelConfig,
            _text: &str,
            _tenant_id: uuid::Uuid,
        ) -> std::result::Result<Vec<f32>, LLMError> {
            Ok(vec![0.1, 0.2, 0.3, 0.4])
        }

        async fn stream_chat_completion(
            &self,
            _config: &ModelConfig,
            _messages: Vec<ChatMessage>,
            _tenant_id: uuid::Uuid,
        ) -> std::result::Result<Box<dyn futures::Stream<Item = std::result::Result<ChatStreamChunk, LLMError>> + Send + Unpin>, LLMError> {
            unimplemented!()
        }

        fn validate_config(&self, _config: &ModelConfig) -> std::result::Result<ValidationResult, LLMError> {
            Ok(ValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
            })
        }

        fn supports_streaming(&self, _config: &ModelConfig) -> bool {
            false
        }

        async fn get_available_models(&self, _provider: &str) -> std::result::Result<Vec<ModelInfo>, LLMError> {
            Ok(Vec::new())
        }

        async fn test_connection(&self, _config: &ModelConfig) -> std::result::Result<ConnectionTestResult, LLMError> {
            Ok(ConnectionTestResult {
                success: true,
                response_time_ms: 100,
                error_message: None,
                model_info: None,
            })
        }

        fn estimate_token_count(&self, _messages: &[ChatMessage], _model: &str) -> std::result::Result<u32, LLMError> {
            Ok(100)
        }
    }

    // Mock Vector Service
    struct MockVectorService;

    #[async_trait]
    impl VectorStoreDomainService for MockVectorService {
        async fn store_vector(&self, _record: crate::domain::value_objects::VectorRecord) -> Result<()> {
            Ok(())
        }

        async fn store_vectors_batch(&self, _records: Vec<crate::domain::value_objects::VectorRecord>) -> Result<()> {
            Ok(())
        }

        async fn search_vectors(
            &self,
            query: SearchQuery,
            _tenant_id: TenantId,
        ) -> Result<Vec<SearchResult>> {
            // Return mock search results
            Ok(vec![
                SearchResult {
                    id: "result1".to_string(),
                    score: 0.95,
                    vector: None,
                    metadata: Some(HashMap::new()),
                },
                SearchResult {
                    id: "result2".to_string(),
                    score: 0.85,
                    vector: None,
                    metadata: Some(HashMap::new()),
                },
            ])
        }

        async fn delete_vectors(&self, _ids: Vec<String>, _tenant_id: TenantId) -> Result<()> {
            Ok(())
        }

        async fn execute_batch(
            &self,
            _operation: crate::domain::value_objects::BatchOperation,
            _tenant_id: TenantId,
        ) -> Result<()> {
            Ok(())
        }

        async fn manage_index(
            &self,
            _config: crate::domain::value_objects::IndexConfig,
            _tenant_id: TenantId,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_stats(&self, _tenant_id: TenantId) -> Result<crate::domain::value_objects::VectorStats> {
            Ok(crate::domain::value_objects::VectorStats {
                total_vectors: 100,
                dimension: 4,
                index_fullness: 0.5,
                namespace_stats: HashMap::new(),
            })
        }

        fn validate_vector_record(&self, _record: &crate::domain::value_objects::VectorRecord) -> Result<()> {
            Ok(())
        }

        fn validate_search_query(&self, _query: &SearchQuery) -> Result<()> {
            Ok(())
        }

        fn validate_tenant_namespace_access(
            &self,
            _tenant_id: TenantId,
            _namespace: Option<&str>,
        ) -> Result<()> {
            Ok(())
        }

        fn generate_tenant_namespace(&self, tenant_id: TenantId, namespace: Option<&str>) -> String {
            match namespace {
                Some(ns) => format!("{}_{}", tenant_id, ns),
                None => tenant_id.to_string(),
            }
        }
    }

    // Mock MCP Service
    struct MockMCPService;

    #[async_trait]
    impl MCPToolDomainService for MockMCPService {
        async fn validate_tool_config(&self, _config: &ToolConfig) -> Result<ConfigValidationResult> {
            Ok(ConfigValidationResult {
                valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
            })
        }

        async fn check_tool_permission(
            &self,
            _tool: &MCPTool,
            _context: &ToolCallContext,
        ) -> Result<PermissionCheckResult> {
            Ok(PermissionCheckResult {
                allowed: true,
                reason: None,
            })
        }

        async fn check_call_permission(
            &self,
            _tool: &MCPTool,
            _context: &ToolCallContext,
            _parameters: &Value,
        ) -> Result<PermissionCheckResult> {
            Ok(PermissionCheckResult {
                allowed: true,
                reason: None,
            })
        }

        async fn validate_call_parameters(
            &self,
            _tool: &MCPTool,
            _parameters: &Value,
        ) -> Result<()> {
            Ok(())
        }

        async fn test_tool_connection(&self, _config: &ToolConfig) -> Result<ToolCallResult> {
            Ok(ToolCallResult {
                success: true,
                result: Some(json!({"status": "ok"})),
                error: None,
                execution_time_ms: 50,
                metadata: HashMap::new(),
            })
        }

        fn create_call_context(
            &self,
            tenant_id: TenantId,
            user_id: UserId,
            request_id: String,
        ) -> ToolCallContext {
            ToolCallContext {
                tenant_id,
                user_id,
                session_id: None,
                request_id,
                metadata: HashMap::new(),
            }
        }

        fn can_execute_tool(&self, _tool: &MCPTool) -> bool {
            true
        }

        async fn validate_tool_name_uniqueness(
            &self,
            _tenant_id: TenantId,
            _name: &str,
            _exclude_tool_id: Option<crate::domain::value_objects::ids::MCPToolId>,
        ) -> Result<bool> {
            Ok(true)
        }
    }

    // Mock Tool Repository
    struct MockToolRepository;

    #[async_trait]
    impl MCPToolRepository for MockToolRepository {
        async fn find_by_id(&self, _id: crate::domain::value_objects::ids::MCPToolId) -> Result<Option<MCPTool>> {
            // Return a mock tool
            let tenant_id = TenantId::new();
            let user_id = UserId::new();
            Ok(Some(MCPTool::new(
                tenant_id,
                "mock-tool".to_string(),
                Some("Mock tool for testing".to_string()),
                ToolConfig::default(),
                user_id,
            )))
        }

        async fn find_by_tenant_and_name(&self, _tenant_id: TenantId, _name: &str) -> Result<Option<MCPTool>> {
            Ok(None)
        }

        async fn find_by_options(&self, _options: crate::domain::repositories::mcp_tool_repository::MCPToolQueryOptions) -> Result<crate::domain::repositories::mcp_tool_repository::MCPToolQueryResult> {
            Ok(crate::domain::repositories::mcp_tool_repository::MCPToolQueryResult {
                tools: Vec::new(),
                total_count: 0,
            })
        }

        async fn find_by_tenant_id(&self, _tenant_id: TenantId) -> Result<Vec<MCPTool>> {
            Ok(Vec::new())
        }

        async fn find_by_created_by(&self, _created_by: UserId) -> Result<Vec<MCPTool>> {
            Ok(Vec::new())
        }

        async fn save(&self, _tool: &MCPTool) -> Result<()> {
            Ok(())
        }

        async fn update(&self, _tool: &MCPTool) -> Result<()> {
            Ok(())
        }

        async fn delete(&self, _id: crate::domain::value_objects::ids::MCPToolId) -> Result<()> {
            Ok(())
        }

        async fn exists_by_tenant_and_name(&self, _tenant_id: TenantId, _name: &str, _exclude_id: Option<crate::domain::value_objects::ids::MCPToolId>) -> Result<bool> {
            Ok(false)
        }

        async fn count_by_tenant(&self, _tenant_id: TenantId) -> Result<u64> {
            Ok(1)
        }

        async fn find_active_by_tenant(&self, _tenant_id: TenantId) -> Result<Vec<MCPTool>> {
            Ok(Vec::new())
        }

        async fn get_version_history(&self, _tool_id: crate::domain::value_objects::ids::MCPToolId) -> Result<Vec<crate::domain::entities::MCPToolVersion>> {
            Ok(Vec::new())
        }

        async fn rollback_to_version(&self, _tool_id: crate::domain::value_objects::ids::MCPToolId, _target_version: i32, _created_by: UserId, _change_log: Option<String>) -> Result<MCPTool> {
            let tenant_id = TenantId::new();
            let user_id = UserId::new();
            Ok(MCPTool::new(
                tenant_id,
                "mock-tool".to_string(),
                Some("Mock tool for testing".to_string()),
                ToolConfig::default(),
                user_id,
            ))
        }

        async fn compare_versions(&self, _tool_id: crate::domain::value_objects::ids::MCPToolId, _from_version: i32, _to_version: i32) -> Result<crate::domain::entities::VersionDiff> {
            Ok(crate::domain::entities::VersionDiff {
                from_version: 1,
                to_version: 2,
                config_changed: false,
                changes: Vec::new(),
            })
        }

        async fn create_version(&self, _tool: &MCPTool, _change_log: Option<String>) -> Result<crate::domain::entities::MCPToolVersion> {
            let tenant_id = TenantId::new();
            let user_id = UserId::new();
            let tool_id = crate::domain::value_objects::ids::MCPToolId::new();
            Ok(crate::domain::entities::MCPToolVersion::new(
                tool_id,
                1,
                ToolConfig::default(),
                None,
                user_id,
            ))
        }
    }

    #[tokio::test]
    async fn test_llm_node_execution_with_tenant_isolation() {
        // Setup
        let llm_service = Arc::new(MockLLMService);
        let llm_executor = LLMChatNodeExecutor::new(llm_service);

        let tenant_id = TenantId::new();
        let user_id = UserId::new();

        // Create LLM node
        let node = FlowNode {
            id: "llm1".to_string(),
            node_type: NodeType::Llm,
            data: json!({
                "model_config": {
                    "provider": "open_a_i",
                    "model_name": "gpt-3.5-turbo",
                    "parameters": {
                        "temperature": 0.7,
                        "max_tokens": 100,
                        "custom_parameters": {}
                    },
                    "credentials": {
                        "api_key": "test-key",
                        "custom_headers": {}
                    }
                },
                "messages": [
                    {
                        "role": "user",
                        "content": "Hello, {{user_input}}"
                    }
                ],
                "output_variable": "llm_response"
            }),
            position: NodePosition { x: 0.0, y: 0.0 },
        };

        // Create execution state with context
        let mut variables = HashMap::new();
        variables.insert("user_input".to_string(), json!("world"));
        
        let mut state = ExecutionState::with_context(
            FlowExecutionId::new(),
            tenant_id.0,
            user_id.0,
            None,
            variables,
        );

        // Execute node
        let result = llm_executor.execute(&node, &mut state).await.unwrap();

        // Verify
        if result.status != crate::domain::services::execution_engine::NodeExecutionStatus::Success {
            eprintln!("Node execution failed with error: {:?}", result.error);
        }
        assert_eq!(result.status, crate::domain::services::execution_engine::NodeExecutionStatus::Success);
        assert!(result.output.is_some());
        
        // Check that response was stored in state
        let llm_response = state.variables.get("llm_response");
        assert!(llm_response.is_some());
        assert!(llm_response.unwrap().as_str().unwrap().contains("Mock LLM response"));
    }

    #[tokio::test]
    async fn test_vector_search_node_with_tenant_isolation() {
        // Setup
        let vector_service = Arc::new(MockVectorService);
        let vector_executor = VectorSearchNodeExecutor::new(vector_service);

        let tenant_id = TenantId::new();
        let user_id = UserId::new();

        // Create vector search node
        let node = FlowNode {
            id: "vector1".to_string(),
            node_type: NodeType::VectorSearch,
            title: "Vector Search".to_string(),
            data: json!({
                "query_vector": [0.1, 0.2, 0.3, 0.4],
                "top_k": 5,
                "namespace": "documents",
                "output_variable": "search_results"
            }),
            position: NodePosition { x: 0.0, y: 0.0 },
        };

        // Create execution state with context
        let variables = HashMap::new();
        let mut state = ExecutionState::with_context(
            FlowExecutionId::new(),
            tenant_id.0,
            user_id.0,
            None,
            variables,
        );

        // Execute node
        let result = vector_executor.execute(&node, &mut state).await.unwrap();

        // Verify
        assert_eq!(result.status, crate::domain::services::execution_engine::NodeExecutionStatus::Success);
        assert!(result.output.is_some());
        
        // Check that results were stored in state
        let search_results = state.variables.get("search_results");
        assert!(search_results.is_some());
        assert!(search_results.unwrap().is_array());
    }

    #[tokio::test]
    async fn test_mcp_tool_node_with_permission_check() {
        // Setup
        let mcp_service = Arc::new(MockMCPService);
        let tool_repository = Arc::new(MockToolRepository);
        let mcp_executor = MCPToolNodeExecutor::new(mcp_service, tool_repository);

        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let tool_id = crate::domain::value_objects::ids::MCPToolId::new();

        // Create MCP tool node
        let node = FlowNode {
            id: "mcp1".to_string(),
            node_type: NodeType::McpTool,
            title: "MCP Tool".to_string(),
            data: json!({
                "tool_id": tool_id.to_string(),
                "parameters": {
                    "input": "test data"
                },
                "output_variable": "tool_result"
            }),
            position: NodePosition { x: 0.0, y: 0.0 },
        };

        // Create execution state with context
        let variables = HashMap::new();
        let mut state = ExecutionState::with_context(
            FlowExecutionId::new(),
            tenant_id.0,
            user_id.0,
            None,
            variables,
        );

        // Execute node
        let result = mcp_executor.execute(&node, &mut state).await.unwrap();

        // Verify
        assert_eq!(result.status, crate::domain::services::execution_engine::NodeExecutionStatus::Success);
        assert!(result.output.is_some());
        
        // Check that result was stored in state
        let tool_result = state.variables.get("tool_result");
        assert!(tool_result.is_some());
    }

    #[tokio::test]
    async fn test_context_propagation_through_execution() {
        // This test verifies that tenant_id and user_id are properly propagated
        // through the execution state and available to all node executors
        
        let tenant_id = TenantId::new();
        let user_id = UserId::new();
        let session_id = Some(crate::domain::value_objects::ids::SessionId::new().0);

        let variables = HashMap::new();
        let state = ExecutionState::with_context(
            FlowExecutionId::new(),
            tenant_id.0,
            user_id.0,
            session_id,
            variables,
        );

        // Verify context is in state variables
        assert_eq!(
            state.variables.get("tenant_id").and_then(|v| v.as_str()),
            Some(tenant_id.to_string().as_str())
        );
        assert_eq!(
            state.variables.get("user_id").and_then(|v| v.as_str()),
            Some(user_id.to_string().as_str())
        );
        assert!(state.variables.get("session_id").is_some());
    }
}
