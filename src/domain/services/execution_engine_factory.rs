use std::sync::Arc;

use crate::domain::services::{
    execution_engine::{ExecutionEngine, ExecutionEngineImpl, NodeExecutor},
    node_executors::*,
    llm_service::LLMDomainService,
    vector_service::VectorStoreDomainService,
    mcp_tool_service::MCPToolDomainService,
};
use crate::domain::repositories::{
    mcp_tool_repository::MCPToolRepository,
    llm_config_repository::LLMConfigRepository,
};

/// Factory for creating execution engines with all necessary node executors
pub struct ExecutionEngineFactory;

impl ExecutionEngineFactory {
    /// Create a new execution engine with all standard node executors
    pub fn create_with_services(
        llm_service: Arc<dyn LLMDomainService>,
        llm_config_repository: Arc<dyn LLMConfigRepository>,
        vector_service: Arc<dyn VectorStoreDomainService>,
        mcp_service: Arc<dyn MCPToolDomainService>,
        tool_repository: Arc<dyn MCPToolRepository>,
    ) -> Arc<dyn ExecutionEngine> {
        let mut executors: Vec<Arc<dyn NodeExecutor>> = Vec::new();

        // Add basic node executors
        executors.push(Arc::new(StartNodeExecutor::new()));
        executors.push(Arc::new(EndNodeExecutor::new()));
        executors.push(Arc::new(VariableNodeExecutor::new()));
        executors.push(Arc::new(ConditionNodeExecutor::new()));
        executors.push(Arc::new(LoopNodeExecutor::new()));
        executors.push(Arc::new(CodeNodeExecutor::new()));
        executors.push(Arc::new(HttpRequestNodeExecutor::new()));
        executors.push(Arc::new(AnswerNodeExecutor::new()));

        // Add service-integrated node executors
        executors.push(Arc::new(LLMChatNodeExecutor::new(llm_service, llm_config_repository)));
        executors.push(Arc::new(VectorSearchNodeExecutor::new(vector_service)));
        executors.push(Arc::new(MCPToolNodeExecutor::new(mcp_service, tool_repository)));

        Arc::new(ExecutionEngineImpl::new(executors))
    }

    /// Create a basic execution engine without external service integrations
    /// Useful for testing or when external services are not available
    pub fn create_basic() -> Arc<dyn ExecutionEngine> {
        let mut executors: Vec<Arc<dyn NodeExecutor>> = Vec::new();

        executors.push(Arc::new(StartNodeExecutor::new()));
        executors.push(Arc::new(EndNodeExecutor::new()));
        executors.push(Arc::new(VariableNodeExecutor::new()));
        executors.push(Arc::new(ConditionNodeExecutor::new()));
        executors.push(Arc::new(LoopNodeExecutor::new()));
        executors.push(Arc::new(CodeNodeExecutor::new()));
        executors.push(Arc::new(HttpRequestNodeExecutor::new()));
        executors.push(Arc::new(AnswerNodeExecutor::new()));

        Arc::new(ExecutionEngineImpl::new(executors))
    }

    /// Create an execution engine with custom node executors
    pub fn create_with_executors(executors: Vec<Arc<dyn NodeExecutor>>) -> Arc<dyn ExecutionEngine> {
        Arc::new(ExecutionEngineImpl::new(executors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_basic_engine() {
        let engine = ExecutionEngineFactory::create_basic();
        assert!(Arc::strong_count(&engine) == 1);
    }

    #[test]
    fn test_create_with_custom_executors() {
        let executors: Vec<Arc<dyn NodeExecutor>> = vec![
            Arc::new(StartNodeExecutor::new()),
            Arc::new(EndNodeExecutor::new()),
        ];
        let engine = ExecutionEngineFactory::create_with_executors(executors);
        assert!(Arc::strong_count(&engine) == 1);
    }
}
