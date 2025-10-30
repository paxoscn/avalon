use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use serde_json::Value;
use std::sync::Arc;

use crate::domain::services::execution_engine::{
    ExecutionState, NodeExecutionResult, NodeExecutionStatus, NodeExecutor,
};
use crate::domain::value_objects::{FlowNode, NodeType};
use crate::domain::ConfigId;
use crate::error::Result;

/// Start node executor - saves flow parameters for later access
pub struct StartNodeExecutor;

impl StartNodeExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StartNodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeExecutor for StartNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Extract variables from node data and save them with node ID prefix
        // Expected format: {"variables": [{"variable": "user_input", "default": "Hello World"}]}
        // Priority: 1. Value from input_data (already in state.variables)
        //           2. Default value from node definition
        if let Some(variables) = node.data.get("variables") {
            if let Some(vars_array) = variables.as_array() {
                for var_item in vars_array {
                    if let Some(var_obj) = var_item.as_object() {
                        if let (Some(var_name), Some(default_value)) = (
                            var_obj.get("variable").and_then(|v| v.as_str()),
                            var_obj.get("default"),
                        ) {
                            // First check if value exists in state.variables (from input_data)
                            // If not found, use the default value from node definition
                            let value = state
                                .get_variable(var_name)
                                .cloned()
                                .unwrap_or_else(|| default_value.clone());

                            // Store variables with format: #node_id.variable_name#
                            let prefixed_key = format!("#{}.{}#", node.id, var_name);
                            state.set_variable(prefixed_key, value);
                        }
                    }
                }
            }
        }

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Success,
            output: Some(serde_json::json!({"message": "Flow started"})),
            error: None,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::Start)
    }
}

/// End node executor - marks flow as complete
pub struct EndNodeExecutor;

impl EndNodeExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EndNodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeExecutor for EndNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Extract outputs configuration from node data
        // Expected format: {"outputs": [{"value_selector": ["node_id", "variable_name"], "value_type": "string", "variable": "output_name"}]}
        let mut final_outputs = serde_json::Map::new();

        if let Some(outputs) = node.data.get("outputs") {
            if let Some(outputs_array) = outputs.as_array() {
                for output_config in outputs_array {
                    if let Some(config_obj) = output_config.as_object() {
                        // Extract the output variable name
                        let output_var_name = config_obj
                            .get("variable")
                            .and_then(|v| v.as_str())
                            .unwrap_or("output");

                        // Extract value_selector: [node_id, variable_name]
                        if let Some(value_selector) = config_obj.get("value_selector") {
                            if let Some(selector_array) = value_selector.as_array() {
                                if selector_array.len() >= 2 {
                                    // Get node_id and variable_name from selector
                                    let node_id = selector_array[0].as_str().unwrap_or("");
                                    let var_name = selector_array[1].as_str().unwrap_or("");

                                    // Construct the variable key: #node_id.variable_name#
                                    let var_key = format!("#{}.{}#", node_id, var_name);

                                    // Get the value from state
                                    if let Some(value) = state.get_variable(&var_key) {
                                        final_outputs
                                            .insert(output_var_name.to_string(), value.clone());
                                    } else {
                                        // If not found with # prefix, try without it
                                        // This handles cases where the variable might be stored differently
                                        if let Some(value) = state.get_variable(var_name) {
                                            final_outputs
                                                .insert(output_var_name.to_string(), value.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        state
            .variables
            .insert("outputs".to_string(), json!(final_outputs));

        let output = serde_json::json!({
            "message": "Flow completed",
            "final_variables": state.variables,
        });

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Success,
            output: Some(output),
            error: None,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::End)
    }
}

/// Variable node executor - sets or updates variables
pub struct VariableNodeExecutor;

impl VariableNodeExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VariableNodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeExecutor for VariableNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Extract variable assignments from node data
        // Expected format: {"assignments": [{"name": "var1", "value": "value1"}]}
        if let Some(assignments) = node.data.get("assignments").and_then(|v| v.as_array()) {
            for assignment in assignments {
                if let Some(obj) = assignment.as_object() {
                    if let (Some(name), Some(value)) =
                        (obj.get("name").and_then(|v| v.as_str()), obj.get("value"))
                    {
                        // Support variable references in value
                        let resolved_value = self.resolve_value(value, state);
                        state.set_variable(name.to_string(), resolved_value);
                    }
                }
            }
        }

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Success,
            output: Some(serde_json::json!({"variables_updated": true})),
            error: None,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::Variable)
    }
}

impl VariableNodeExecutor {
    fn resolve_value(&self, value: &Value, state: &ExecutionState) -> Value {
        // If value is a string starting with $, treat it as a variable reference
        if let Some(s) = value.as_str() {
            if let Some(var_name) = s.strip_prefix('$') {
                if let Some(var_value) = state.get_variable(var_name) {
                    return var_value.clone();
                }
            }

            // Also support {{variable}} syntax for consistency
            if s.starts_with("{{") && s.ends_with("}}") {
                let var_name = &s[2..s.len() - 2];
                if let Some(var_value) = state.get_variable(var_name) {
                    return var_value.clone();
                }
            }
        }
        value.clone()
    }
}

/// Condition node executor - evaluates conditions
pub struct ConditionNodeExecutor;

impl ConditionNodeExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConditionNodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeExecutor for ConditionNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        _state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Condition evaluation is handled by the execution engine
        // This executor just marks the node as executed
        let output = serde_json::json!({
            "message": "Condition evaluated",
            "condition": node.data.get("condition"),
        });

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Success,
            output: Some(output),
            error: None,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::Condition)
    }
}

/// Loop node executor - manages loop iterations
pub struct LoopNodeExecutor;

impl LoopNodeExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoopNodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeExecutor for LoopNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Increment loop counter
        let iteration = state.increment_loop_counter(&node.id);

        let output = serde_json::json!({
            "message": "Loop iteration",
            "iteration": iteration,
        });

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Success,
            output: Some(output),
            error: None,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::Loop)
    }
}

/// Answer node executor - processes answer strings with variable references
pub struct AnswerNodeExecutor;

impl AnswerNodeExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Resolves variable references in the answer string
    /// Supports format: {{#node_id.variable_name#}}
    fn resolve_answer(&self, answer: &str, state: &ExecutionState) -> String {
        let mut result = answer.to_string();

        // Replace {{#node_id.variable_name#}} with actual values from state
        for (key, value) in &state.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            if result.contains(&placeholder) {
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Array(arr) => {
                        serde_json::to_string(arr).unwrap_or_else(|_| "[]".to_string())
                    }
                    Value::Object(obj) => {
                        serde_json::to_string(obj).unwrap_or_else(|_| "{}".to_string())
                    }
                    Value::Null => "null".to_string(),
                };
                result = result.replace(&placeholder, &value_str);
            }
        }

        result
    }
}

impl Default for AnswerNodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeExecutor for AnswerNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Extract answer template from node data
        let answer_template = node
            .data
            .get("answer")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Resolve variable references in the answer
        let resolved_answer = self.resolve_answer(answer_template, state);

        // Store the resolved answer in state with format: #node_id.answer#
        let answer_var = format!("#{}.answer#", node.id);
        state.set_variable(answer_var, serde_json::json!(resolved_answer.clone()));
        
        state.set_variable("outputs".to_string(), json!({ "answer": resolved_answer.clone() }));

        let output = serde_json::json!({
            "answer": resolved_answer,
        });

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Success,
            output: Some(output),
            error: None,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::Answer)
    }
}

/// Code node executor - executes code snippets (placeholder for now)
pub struct CodeNodeExecutor;

impl CodeNodeExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CodeNodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeExecutor for CodeNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // For now, this is a placeholder
        // In a real implementation, this would execute code in a sandboxed environment
        let code = node.data.get("code").and_then(|v| v.as_str()).unwrap_or("");

        // Simple variable extraction for demonstration
        // In production, you'd use a proper code execution engine
        let output = serde_json::json!({
            "message": "Code execution placeholder",
            "code_length": code.len(),
            "variables": state.variables,
        });

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Success,
            output: Some(output),
            error: None,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::Code)
    }
}

/// HTTP Request node executor - makes HTTP requests (placeholder for now)
pub struct HttpRequestNodeExecutor;

impl HttpRequestNodeExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HttpRequestNodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeExecutor for HttpRequestNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        _state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Placeholder for HTTP request execution
        // In production, this would make actual HTTP requests
        let url = node.data.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let method = node
            .data
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");

        let output = serde_json::json!({
            "message": "HTTP request placeholder",
            "url": url,
            "method": method,
        });

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Success,
            output: Some(output),
            error: None,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::HttpRequest)
    }
}

/// LLM Chat node executor - integrates with LLM services
pub struct LLMChatNodeExecutor {
    llm_service: Arc<dyn crate::domain::services::llm_service::LLMDomainService>,
    llm_config_repository:
        Arc<dyn crate::domain::repositories::llm_config_repository::LLMConfigRepository>,
}

impl LLMChatNodeExecutor {
    pub fn new(
        llm_service: Arc<dyn crate::domain::services::llm_service::LLMDomainService>,
        llm_config_repository: Arc<
            dyn crate::domain::repositories::llm_config_repository::LLMConfigRepository,
        >,
    ) -> Self {
        Self {
            llm_service,
            llm_config_repository,
        }
    }

    fn extract_messages(
        &self,
        node: &FlowNode,
        state: &ExecutionState,
    ) -> Result<Vec<crate::domain::value_objects::ChatMessage>> {
        let default_messages_data = Vec::new();
        let messages_data = node
            .data
            .get("prompt_template")
            .and_then(|v| v.as_array())
            .unwrap_or(&default_messages_data);

        let mut messages = Vec::new();

        for msg in messages_data {
            if let Some(obj) = msg.as_object() {
                let role = obj.get("role").and_then(|v| v.as_str()).ok_or_else(|| {
                    crate::error::PlatformError::ValidationError(
                        "Message missing 'role' field".to_string(),
                    )
                })?;

                let content = obj.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
                    crate::error::PlatformError::ValidationError(
                        "Message missing 'text' field".to_string(),
                    )
                })?;

                // Resolve variable references in content
                let resolved_content = self.resolve_template(content, state);

                if resolved_content.len() < 1 {
                    continue;
                }

                let message = match role {
                    "user" => crate::domain::value_objects::ChatMessage::new_user_message(
                        resolved_content,
                    ),
                    "assistant" => {
                        crate::domain::value_objects::ChatMessage::new_assistant_message(
                            resolved_content,
                        )
                    }
                    "system" => crate::domain::value_objects::ChatMessage::new_system_message(
                        resolved_content,
                    ),
                    _ => {
                        return Err(crate::error::PlatformError::ValidationError(format!(
                            "Unknown message role: {}",
                            role
                        )))
                    }
                };

                messages.push(message);
            }
        }

        Ok(messages)
    }

    fn resolve_template(&self, template: &str, state: &ExecutionState) -> String {
        let mut result = template.to_string();

        // Replace {{variable_name}} with actual values from state
        // This supports both regular variables and node-prefixed parameters like {{#node_id.param#}}
        for (key, value) in &state.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            if result.contains(&placeholder) {
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    v => v.to_string(),
                };
                result = result.replace(&placeholder, &value_str);
            }
        }

        result
    }

    async fn extract_model_config(
        &self,
        node: &FlowNode,
        _state: &ExecutionState,
    ) -> Result<crate::domain::value_objects::ModelConfig> {
        let config_data = node.data.get("model").ok_or_else(|| {
            crate::error::PlatformError::ValidationError(
                "LLM node missing 'model' field".to_string(),
            )
        })?;

        let llm_config_id = config_data
            .get("llm_config_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "LLM node missing 'model.llm_config_id' field".to_string(),
                )
            })?;

        // // Extract tenant_id from state and convert to TenantId
        // let tenant_id_uuid = self.extract_tenant_id(state)?;
        // let tenant_id = crate::domain::value_objects::TenantId::from_uuid(tenant_id_uuid);

        // Query database for matching config
        // First try to find by provider
        let config = self
            .llm_config_repository
            .find_by_id(ConfigId::from_string(llm_config_id).map_err(|e| {
                crate::error::PlatformError::ValidationError(format!(
                    "Invalid UUID: {}. Error: {}",
                    llm_config_id, e
                ))
            })?)
            .await?
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(format!(
                    "LLM config not found: {}",
                    llm_config_id
                ))
            })?;

        // TODO Merging config from both the flow and the LLM config.
        Ok(config.model_config.clone())
    }

    fn extract_tenant_id(&self, state: &ExecutionState) -> Result<uuid::Uuid> {
        state
            .variables
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "Missing or invalid tenant_id in execution context".to_string(),
                )
            })
    }
}

#[async_trait]
impl NodeExecutor for LLMChatNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Extract configuration and messages
        let model_config = match self.extract_model_config(node, state).await {
            Ok(config) => config,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        let messages = match self.extract_messages(node, state) {
            Ok(msgs) => msgs,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        let tenant_id = match self.extract_tenant_id(state) {
            Ok(id) => id,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Call LLM service
        let response = match self
            .llm_service
            .chat_completion(&model_config, messages, tenant_id)
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(format!("LLM call failed: {}", e)),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Store response in state variables
        let output_var = node
            .data
            .get("output_variable")
            .and_then(|v| v.as_str())
            .unwrap_or("llm_response");

        state.set_variable(output_var.to_string(), serde_json::json!(response.content));

        // Also store response.content in #node_id.text# for easy access in subsequent nodes
        let text_var = format!("#{}.text#", node.id);
        state.set_variable(text_var, serde_json::json!(response.content));

        let output = serde_json::json!({
            "content": response.content,
            "model_used": response.model_used,
            "usage": {
                "prompt_tokens": response.usage.prompt_tokens,
                "completion_tokens": response.usage.completion_tokens,
                "total_tokens": response.usage.total_tokens,
            },
            "finish_reason": format!("{:?}", response.finish_reason),
        });

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Success,
            output: Some(output),
            error: None,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::Llm)
    }
}

/// Vector Search node executor - integrates with vector database services
pub struct VectorSearchNodeExecutor {
    vector_service: Arc<dyn crate::domain::services::vector_service::VectorStoreDomainService>,
}

impl VectorSearchNodeExecutor {
    pub fn new(
        vector_service: Arc<dyn crate::domain::services::vector_service::VectorStoreDomainService>,
    ) -> Self {
        Self { vector_service }
    }

    fn extract_search_query(
        &self,
        node: &FlowNode,
        state: &ExecutionState,
    ) -> Result<crate::domain::value_objects::SearchQuery> {
        // Get query vector - can be from a variable or directly specified
        let vector = if let Some(var_name) = node
            .data
            .get("query_vector_variable")
            .and_then(|v| v.as_str())
        {
            // Get vector from state variable
            state
                .variables
                .get(var_name)
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    crate::error::PlatformError::ValidationError(format!(
                        "Variable '{}' not found or not an array",
                        var_name
                    ))
                })?
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect::<Vec<f32>>()
        } else if let Some(vec_data) = node.data.get("query_vector").and_then(|v| v.as_array()) {
            // Get vector directly from node data
            vec_data
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect::<Vec<f32>>()
        } else {
            return Err(crate::error::PlatformError::ValidationError(
                "Vector search node missing query vector".to_string(),
            ));
        };

        let top_k = node
            .data
            .get("top_k")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let namespace = node
            .data
            .get("namespace")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut query = crate::domain::value_objects::SearchQuery::new(vector, top_k)
            .map_err(|e| crate::error::PlatformError::ValidationError(e))?;

        query.namespace = namespace;

        // Add filters if specified
        if let Some(filter_data) = node.data.get("filter") {
            let filter: crate::domain::value_objects::SearchFilter =
                serde_json::from_value(filter_data.clone()).map_err(|e| {
                    crate::error::PlatformError::ValidationError(format!("Invalid filter: {}", e))
                })?;
            query.filter = Some(filter);
        }

        Ok(query)
    }

    fn extract_tenant_id(
        &self,
        state: &ExecutionState,
    ) -> Result<crate::domain::value_objects::ids::TenantId> {
        state
            .variables
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .map(crate::domain::value_objects::ids::TenantId::from)
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "Missing or invalid tenant_id in execution context".to_string(),
                )
            })
    }
}

#[async_trait]
impl NodeExecutor for VectorSearchNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Extract search query
        let query = match self.extract_search_query(node, state) {
            Ok(q) => q,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        let tenant_id = match self.extract_tenant_id(state) {
            Ok(id) => id,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Perform vector search with tenant isolation
        let results = match self.vector_service.search_vectors(query, tenant_id).await {
            Ok(res) => res,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(format!("Vector search failed: {}", e)),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Store results in state variables
        let output_var = node
            .data
            .get("output_variable")
            .and_then(|v| v.as_str())
            .unwrap_or("search_results");

        let results_json: Vec<Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "score": r.score,
                    "metadata": r.metadata,
                })
            })
            .collect();

        state.set_variable(output_var.to_string(), serde_json::json!(results_json));

        let output = serde_json::json!({
            "results_count": results.len(),
            "results": results_json,
        });

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Success,
            output: Some(output),
            error: None,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::VectorSearch)
    }
}

/// MCP Tool node executor - integrates with MCP tool services
pub struct MCPToolNodeExecutor {
    mcp_service: Arc<dyn crate::domain::services::mcp_tool_service::MCPToolDomainService>,
    tool_repository: Arc<dyn crate::domain::repositories::mcp_tool_repository::MCPToolRepository>,
}

impl MCPToolNodeExecutor {
    pub fn new(
        mcp_service: Arc<dyn crate::domain::services::mcp_tool_service::MCPToolDomainService>,
        tool_repository: Arc<
            dyn crate::domain::repositories::mcp_tool_repository::MCPToolRepository,
        >,
    ) -> Self {
        Self {
            mcp_service,
            tool_repository,
        }
    }

    fn extract_tool_parameters(&self, node: &FlowNode, state: &ExecutionState) -> Result<Value> {
        let params_data = node.data.get("parameters").ok_or_else(|| {
            crate::error::PlatformError::ValidationError(
                "MCP tool node missing 'parameters' field".to_string(),
            )
        })?;

        // Resolve variable references in parameters
        let resolved_params = self.resolve_parameters(params_data, state);
        Ok(resolved_params)
    }

    fn resolve_parameters(&self, params: &Value, state: &ExecutionState) -> Value {
        match params {
            Value::Object(map) => {
                let mut resolved = serde_json::Map::new();
                for (key, value) in map {
                    resolved.insert(key.clone(), self.resolve_parameters(value, state));
                }
                Value::Object(resolved)
            }
            Value::Array(arr) => {
                let resolved: Vec<Value> = arr
                    .iter()
                    .map(|v| self.resolve_parameters(v, state))
                    .collect();
                Value::Array(resolved)
            }
            Value::String(s) => {
                // Check if it's a variable reference with {{...}} syntax
                if let Some(var_name) = s.strip_prefix("{{").and_then(|s| s.strip_suffix("}}")) {
                    let trimmed = var_name.trim();
                    state
                        .variables
                        .get(trimmed)
                        .cloned()
                        .unwrap_or_else(|| Value::String(s.clone()))
                } else {
                    Value::String(s.clone())
                }
            }
            other => other.clone(),
        }
    }

    fn extract_context(
        &self,
        state: &ExecutionState,
    ) -> Result<crate::domain::services::mcp_tool_service::ToolCallContext> {
        let tenant_id = state
            .variables
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .map(crate::domain::value_objects::ids::TenantId::from)
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "Missing or invalid tenant_id".to_string(),
                )
            })?;

        let user_id = state
            .variables
            .get("user_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .map(crate::domain::value_objects::ids::UserId::from)
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "Missing or invalid user_id".to_string(),
                )
            })?;

        let request_id = uuid::Uuid::new_v4().to_string();

        let mut context = crate::domain::services::mcp_tool_service::ToolCallContext::new(
            tenant_id, user_id, request_id,
        );

        // Add session_id if available
        if let Some(session_id) = state.variables.get("session_id").and_then(|v| v.as_str()) {
            context = context.with_session_id(session_id.to_string());
        }

        Ok(context)
    }
}

#[async_trait]
impl NodeExecutor for MCPToolNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Extract tool ID or name
        let tool_id_str = node
            .data
            .get("tool_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "MCP tool node missing 'tool_id' field".to_string(),
                )
            });

        let tool_id = match tool_id_str {
            Ok(id_str) => match uuid::Uuid::parse_str(id_str) {
                Ok(uuid) => Ok(crate::domain::value_objects::ids::MCPToolId::from_uuid(
                    uuid,
                )),
                Err(e) => Err(crate::error::PlatformError::ValidationError(format!(
                    "Invalid tool_id: {}",
                    e
                ))),
            },
            Err(e) => Err(e),
        };

        let tool_id = match tool_id {
            Ok(id) => id,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Get tool from repository
        let tool = match self.tool_repository.find_by_id(tool_id).await {
            Ok(Some(t)) => t,
            Ok(None) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(format!("Tool not found: {}", tool_id)),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(format!("Failed to retrieve tool: {}", e)),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Extract call context with tenant isolation
        let context = match self.extract_context(state) {
            Ok(ctx) => ctx,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Check permissions
        let permission = match self
            .mcp_service
            .check_tool_permission(&tool, &context)
            .await
        {
            Ok(perm) => perm,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(format!("Permission check failed: {}", e)),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        if !permission.allowed {
            let completed_at = Utc::now();
            let execution_time_ms = completed_at
                .signed_duration_since(started_at)
                .num_milliseconds();
            return Ok(NodeExecutionResult {
                node_id: node.id.clone(),
                status: NodeExecutionStatus::Failed,
                output: None,
                error: Some(
                    permission
                        .reason
                        .unwrap_or_else(|| "Permission denied".to_string()),
                ),
                started_at,
                completed_at,
                execution_time_ms,
            });
        }

        // Extract and resolve parameters
        let parameters = match self.extract_tool_parameters(node, state) {
            Ok(params) => params,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Validate parameters
        if let Err(e) = self
            .mcp_service
            .validate_call_parameters(&tool, &parameters)
            .await
        {
            let completed_at = Utc::now();
            let execution_time_ms = completed_at
                .signed_duration_since(started_at)
                .num_milliseconds();
            return Ok(NodeExecutionResult {
                node_id: node.id.clone(),
                status: NodeExecutionStatus::Failed,
                output: None,
                error: Some(format!("Parameter validation failed: {}", e)),
                started_at,
                completed_at,
                execution_time_ms,
            });
        }

        // Call the tool (this would be implemented in the infrastructure layer)
        // For now, we'll create a placeholder result
        let tool_result = crate::domain::services::mcp_tool_service::ToolCallResult::success(
            serde_json::json!({
                "message": "Tool executed successfully",
                "tool_id": tool_id.to_string(),
                "parameters": parameters,
            }),
            Utc::now()
                .signed_duration_since(started_at)
                .num_milliseconds() as u64,
        );

        // Store result in state variables
        let output_var = node
            .data
            .get("output_variable")
            .and_then(|v| v.as_str())
            .unwrap_or("tool_result");

        if let Some(result_data) = &tool_result.result {
            state.set_variable(output_var.to_string(), result_data.clone());
        }

        let output = serde_json::json!({
            "success": tool_result.success,
            "result": tool_result.result,
            "error": tool_result.error,
            "execution_time_ms": tool_result.execution_time_ms,
        });

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        let status = if tool_result.success {
            NodeExecutionStatus::Success
        } else {
            NodeExecutionStatus::Failed
        };

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status,
            output: Some(output),
            error: tool_result.error,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::McpTool)
    }
}

/// Parameter Extractor node executor - uses LLM to extract structured parameters
pub struct ParameterExtractorNodeExecutor {
    llm_service: Arc<dyn crate::domain::services::llm_service::LLMDomainService>,
    llm_config_repository:
        Arc<dyn crate::domain::repositories::llm_config_repository::LLMConfigRepository>,
}

impl ParameterExtractorNodeExecutor {
    pub fn new(
        llm_service: Arc<dyn crate::domain::services::llm_service::LLMDomainService>,
        llm_config_repository: Arc<
            dyn crate::domain::repositories::llm_config_repository::LLMConfigRepository,
        >,
    ) -> Self {
        Self {
            llm_service,
            llm_config_repository,
        }
    }

    async fn extract_model_config(
        &self,
        node: &FlowNode,
        _state: &ExecutionState,
    ) -> Result<crate::domain::value_objects::ModelConfig> {
        let config_data = node.data.get("model").ok_or_else(|| {
            crate::error::PlatformError::ValidationError(
                "Parameter extractor node missing 'model' field".to_string(),
            )
        })?;

        let llm_config_id = config_data
            .get("llm_config_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "Parameter extractor node missing 'model.llm_config_id' field".to_string(),
                )
            })?;

        let config = self
            .llm_config_repository
            .find_by_id(ConfigId::from_string(llm_config_id).map_err(|e| {
                crate::error::PlatformError::ValidationError(format!(
                    "Invalid UUID: {}. Error: {}",
                    llm_config_id, e
                ))
            })?)
            .await?
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(format!(
                    "LLM config not found: {}",
                    llm_config_id
                ))
            })?;

        Ok(config.model_config.clone())
    }

    fn extract_tenant_id(&self, state: &ExecutionState) -> Result<uuid::Uuid> {
        state
            .variables
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "Missing or invalid tenant_id in execution context".to_string(),
                )
            })
    }

    fn resolve_query_content(&self, query_path: &[String], state: &ExecutionState) -> String {
        if query_path.len() != 2 {
            return String::new();
        }

        let node_id = &query_path[0];
        let var_name = &query_path[1];
        let var_key = format!("#{}.{}#", node_id, var_name);

        state
            .get_variable(&var_key)
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => serde_json::to_string(v).ok(),
            })
            .unwrap_or_default()
    }
}

#[async_trait]
impl NodeExecutor for ParameterExtractorNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Extract model configuration
        let model_config = match self.extract_model_config(node, state).await {
            Ok(config) => config,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Extract system instruction
        let instruction = node
            .data
            .get("instruction")
            .and_then(|v| v.as_str())
            .unwrap_or("Extract parameters from the following text and return them as a JSON array of strings.");

        // Extract query paths and build user prompt
        let query_paths = node
            .data
            .get("query")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "Parameter extractor node missing 'query' field".to_string(),
                )
            });

        let query_paths = match query_paths {
            Ok(paths) => paths,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Build user prompt from query paths
        let path: Vec<String> = query_paths
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        let content = self.resolve_query_content(&path, state);

        // Build messages for LLM
        let messages = vec![
            crate::domain::value_objects::ChatMessage::new_system_message(format!(
                "{}. You must respond with a valid JSON array of strings only, no other text.",
                instruction
            )),
            crate::domain::value_objects::ChatMessage::new_user_message(content),
        ];

        let tenant_id = match self.extract_tenant_id(state) {
            Ok(id) => id,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Call LLM service
        let response = match self
            .llm_service
            .chat_completion(&model_config, messages, tenant_id)
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                let completed_at = Utc::now();
                let execution_time_ms = completed_at
                    .signed_duration_since(started_at)
                    .num_milliseconds();
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(format!("LLM call failed: {}", e)),
                    started_at,
                    completed_at,
                    execution_time_ms,
                });
            }
        };

        // Parse response as JSON array
        let extracted_params: Vec<String> = match serde_json::from_str(&response.content) {
            Ok(params) => params,
            Err(_) => {
                // If parsing fails, try to extract JSON array from the response
                if let Some(start) = response.content.find('[') {
                    if let Some(end) = response.content.rfind(']') {
                        let json_str = &response.content[start..=end];
                        serde_json::from_str(json_str).unwrap_or_else(|_| vec![response.content.clone()])
                    } else {
                        vec![response.content.clone()]
                    }
                } else {
                    vec![response.content.clone()]
                }
            }
        };

        // Extract output parameter name
        let parameters = node
            .data
            .get("parameters")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|p| p.as_object())
            .and_then(|obj| obj.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("extracted_parameters");

        // Store result in state with node ID prefix
        let output_var = format!("#{}.{}#", node.id, parameters);
        state.set_variable(output_var, serde_json::json!(extracted_params));

        let output = serde_json::json!({
            "extracted_parameters": extracted_params,
            "parameter_name": parameters,
            "model_used": response.model_used,
        });

        let completed_at = Utc::now();
        let execution_time_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds();

        Ok(NodeExecutionResult {
            node_id: node.id.clone(),
            status: NodeExecutionStatus::Success,
            output: Some(output),
            error: None,
            started_at,
            completed_at,
            execution_time_ms,
        })
    }

    fn can_handle(&self, node_type: &NodeType) -> bool {
        matches!(node_type, NodeType::ParameterExtractor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::NodePosition;
    use std::collections::HashMap;

    fn create_test_state() -> ExecutionState {
        let mut variables = HashMap::new();
        variables.insert("test_var".to_string(), serde_json::json!("test_value"));
        ExecutionState::new(
            crate::domain::value_objects::FlowExecutionId::new(),
            variables,
        )
    }

    #[tokio::test]
    async fn test_start_node_executor() {
        let executor = StartNodeExecutor::new();
        let node = FlowNode {
            id: "start".to_string(),
            node_type: NodeType::Start,
            // title: "Start".to_string(),
            data: serde_json::json!({}),
            position: NodePosition { x: 0.0, y: 0.0 },
        };
        let mut state = create_test_state();

        let result = executor.execute(&node, &mut state).await.unwrap();
        assert_eq!(result.status, NodeExecutionStatus::Success);
        assert_eq!(result.node_id, "start");
    }

    #[tokio::test]
    async fn test_start_node_with_parameters() {
        let executor = StartNodeExecutor::new();
        let node = FlowNode {
            id: "start_1".to_string(),
            node_type: NodeType::Start,
            data: serde_json::json!({
                "variables": [
                    {"variable": "user_input", "default": "Hello World"},
                    {"variable": "max_tokens", "default": 100},
                    {"variable": "temperature", "default": 0.7}
                ]
            }),
            position: NodePosition { x: 0.0, y: 0.0 },
        };
        let mut state = create_test_state();

        let result = executor.execute(&node, &mut state).await.unwrap();
        assert_eq!(result.status, NodeExecutionStatus::Success);

        // Verify variables are stored with node ID prefix
        assert_eq!(
            state.get_variable("#start_1.user_input#"),
            Some(&serde_json::json!("Hello World"))
        );
        assert_eq!(
            state.get_variable("#start_1.max_tokens#"),
            Some(&serde_json::json!(100))
        );
        assert_eq!(
            state.get_variable("#start_1.temperature#"),
            Some(&serde_json::json!(0.7))
        );
    }

    #[tokio::test]
    async fn test_start_node_with_input_data_override() {
        let executor = StartNodeExecutor::new();
        let node = FlowNode {
            id: "start_1".to_string(),
            node_type: NodeType::Start,
            data: serde_json::json!({
                "variables": [
                    {"variable": "user_input", "default": "Default Question"},
                    {"variable": "max_tokens", "default": 100},
                    {"variable": "temperature", "default": 0.7}
                ]
            }),
            position: NodePosition { x: 0.0, y: 0.0 },
        };

        // Create state with input_data that overrides some variables
        let mut initial_variables = HashMap::new();
        initial_variables.insert(
            "user_input".to_string(),
            serde_json::json!("Overridden Question from input_data"),
        );
        initial_variables.insert("max_tokens".to_string(), serde_json::json!(200));
        // temperature is not provided, should use default

        let mut state = ExecutionState::new(
            crate::domain::value_objects::FlowExecutionId::new(),
            initial_variables,
        );

        let result = executor.execute(&node, &mut state).await.unwrap();
        assert_eq!(result.status, NodeExecutionStatus::Success);

        // Verify that input_data values override defaults
        assert_eq!(
            state.get_variable("#start_1.user_input#"),
            Some(&serde_json::json!("Overridden Question from input_data"))
        );
        assert_eq!(
            state.get_variable("#start_1.max_tokens#"),
            Some(&serde_json::json!(200))
        );
        // temperature should use default since not in input_data
        assert_eq!(
            state.get_variable("#start_1.temperature#"),
            Some(&serde_json::json!(0.7))
        );
    }

    #[tokio::test]
    async fn test_variable_node_executor() {
        let executor = VariableNodeExecutor::new();
        let node = FlowNode {
            id: "var1".to_string(),
            node_type: NodeType::Variable,
            // title: "Set Variable".to_string(),
            data: serde_json::json!({
                "assignments": [
                    {"name": "new_var", "value": "new_value"}
                ]
            }),
            position: NodePosition { x: 0.0, y: 0.0 },
        };
        let mut state = create_test_state();

        let result = executor.execute(&node, &mut state).await.unwrap();
        assert_eq!(result.status, NodeExecutionStatus::Success);
        assert_eq!(
            state.get_variable("new_var"),
            Some(&serde_json::json!("new_value"))
        );
    }

    #[tokio::test]
    async fn test_variable_node_with_reference() {
        let executor = VariableNodeExecutor::new();
        let node = FlowNode {
            id: "var1".to_string(),
            node_type: NodeType::Variable,
            // title: "Set Variable".to_string(),
            data: serde_json::json!({
                "assignments": [
                    {"name": "copied_var", "value": "$test_var"}
                ]
            }),
            position: NodePosition { x: 0.0, y: 0.0 },
        };
        let mut state = create_test_state();

        let result = executor.execute(&node, &mut state).await.unwrap();
        assert_eq!(result.status, NodeExecutionStatus::Success);
        assert_eq!(
            state.get_variable("copied_var"),
            Some(&serde_json::json!("test_value"))
        );
    }

    #[tokio::test]
    async fn test_loop_node_executor() {
        let executor = LoopNodeExecutor::new();
        let node = FlowNode {
            id: "loop1".to_string(),
            node_type: NodeType::Loop,
            // title: "Loop".to_string(),
            data: serde_json::json!({"max_iterations": 5}),
            position: NodePosition { x: 0.0, y: 0.0 },
        };
        let mut state = create_test_state();

        // Execute loop multiple times
        for i in 1..=3 {
            let result = executor.execute(&node, &mut state).await.unwrap();
            assert_eq!(result.status, NodeExecutionStatus::Success);
            assert_eq!(state.get_loop_counter("loop1"), i);
        }
    }

    #[tokio::test]
    async fn test_end_node_with_outputs() {
        let executor = EndNodeExecutor::new();

        // Create state with some variables including LLM node output
        let mut variables = HashMap::new();
        variables.insert(
            "#llm_1.text#".to_string(),
            serde_json::json!("This is the LLM response"),
        );
        variables.insert(
            "#start_1.user_input#".to_string(),
            serde_json::json!("User question"),
        );
        variables.insert("other_var".to_string(), serde_json::json!("other value"));

        let mut state = ExecutionState::new(
            crate::domain::value_objects::FlowExecutionId::new(),
            variables,
        );

        // End node configured to output specific variables
        let node = FlowNode {
            id: "end_1".to_string(),
            node_type: NodeType::End,
            data: serde_json::json!({
                "outputs": [
                    {
                        "value_selector": ["llm_1", "text"],
                        "value_type": "string",
                        "variable": "text"
                    },
                    {
                        "value_selector": ["start_1", "user_input"],
                        "value_type": "string",
                        "variable": "question"
                    }
                ]
            }),
            position: NodePosition { x: 0.0, y: 0.0 },
        };

        let result = executor.execute(&node, &mut state).await.unwrap();
        assert_eq!(result.status, NodeExecutionStatus::Success);

        // Verify the output contains only the specified variables
        let output = result.output.unwrap();
        let outputs = output.get("outputs").unwrap();

        assert_eq!(
            outputs.get("text"),
            Some(&serde_json::json!("This is the LLM response"))
        );
        assert_eq!(
            outputs.get("question"),
            Some(&serde_json::json!("User question"))
        );

        // Verify other_var is not in outputs
        assert_eq!(outputs.get("other_var"), None);
    }

    #[tokio::test]
    async fn test_end_node_without_outputs_config() {
        let executor = EndNodeExecutor::new();

        let mut variables = HashMap::new();
        variables.insert("var1".to_string(), serde_json::json!("value1"));
        variables.insert("var2".to_string(), serde_json::json!("value2"));

        let mut state = ExecutionState::new(
            crate::domain::value_objects::FlowExecutionId::new(),
            variables,
        );

        // End node without outputs configuration (backward compatibility)
        let node = FlowNode {
            id: "end_1".to_string(),
            node_type: NodeType::End,
            data: serde_json::json!({}),
            position: NodePosition { x: 0.0, y: 0.0 },
        };

        let result = executor.execute(&node, &mut state).await.unwrap();
        assert_eq!(result.status, NodeExecutionStatus::Success);

        // Should return all variables
        let output = result.output.unwrap();
        assert!(output.get("final_variables").is_some());
    }

    #[tokio::test]
    async fn test_answer_node_executor() {
        let executor = AnswerNodeExecutor::new();

        // Create state with some variables
        let mut variables = HashMap::new();
        variables.insert(
            "#1761621778329.checking_items#".to_string(),
            serde_json::json!("Item 1, Item 2, Item 3"),
        );
        variables.insert(
            "#start_1.user_name#".to_string(),
            serde_json::json!("Alice"),
        );

        let mut state = ExecutionState::new(
            crate::domain::value_objects::FlowExecutionId::new(),
            variables,
        );

        // Answer node with variable references
        let node = FlowNode {
            id: "answer_1".to_string(),
            node_type: NodeType::Answer,
            data: serde_json::json!({
                "answer": "Hello {{#start_1.user_name#}}, here are your items: {{#1761621778329.checking_items#}}"
            }),
            position: NodePosition { x: 0.0, y: 0.0 },
        };

        let result = executor.execute(&node, &mut state).await.unwrap();
        assert_eq!(result.status, NodeExecutionStatus::Success);

        // Verify the resolved answer is stored in state
        assert_eq!(
            state.get_variable("#answer_1.answer#"),
            Some(&serde_json::json!(
                "Hello Alice, here are your items: Item 1, Item 2, Item 3"
            ))
        );

        // Verify the output contains the resolved answer
        let output = result.output.unwrap();
        assert_eq!(
            output.get("answer"),
            Some(&serde_json::json!(
                "Hello Alice, here are your items: Item 1, Item 2, Item 3"
            ))
        );
    }

    #[tokio::test]
    async fn test_answer_node_with_no_variables() {
        let executor = AnswerNodeExecutor::new();
        let mut state = create_test_state();

        // Answer node with plain text (no variable references)
        let node = FlowNode {
            id: "answer_2".to_string(),
            node_type: NodeType::Answer,
            data: serde_json::json!({
                "answer": "This is a plain text answer without any variables."
            }),
            position: NodePosition { x: 0.0, y: 0.0 },
        };

        let result = executor.execute(&node, &mut state).await.unwrap();
        assert_eq!(result.status, NodeExecutionStatus::Success);

        // Verify the answer is stored as-is
        assert_eq!(
            state.get_variable("#answer_2.answer#"),
            Some(&serde_json::json!(
                "This is a plain text answer without any variables."
            ))
        );
    }

    #[tokio::test]
    async fn test_answer_node_with_missing_variable() {
        let executor = AnswerNodeExecutor::new();
        let mut state = create_test_state();

        // Answer node referencing a non-existent variable
        let node = FlowNode {
            id: "answer_3".to_string(),
            node_type: NodeType::Answer,
            data: serde_json::json!({
                "answer": "Hello {{#missing_node.missing_var#}}, this variable doesn't exist."
            }),
            position: NodePosition { x: 0.0, y: 0.0 },
        };

        let result = executor.execute(&node, &mut state).await.unwrap();
        assert_eq!(result.status, NodeExecutionStatus::Success);

        // The placeholder should remain unreplaced if variable doesn't exist
        let output = result.output.unwrap();
        assert_eq!(
            output.get("answer"),
            Some(&serde_json::json!(
                "Hello {{#missing_node.missing_var#}}, this variable doesn't exist."
            ))
        );
    }
}
