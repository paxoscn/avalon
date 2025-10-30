use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entities::FlowExecution;
use crate::domain::value_objects::{FlowDefinition, FlowExecutionId, FlowNode, NodeType};
use crate::error::{PlatformError, Result};

/// Node execution result
#[derive(Debug, Clone)]
pub struct NodeExecutionResult {
    pub node_id: String,
    pub status: NodeExecutionStatus,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub execution_time_ms: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeExecutionStatus {
    Success,
    Failed,
    Skipped,
}

/// Execution state that tracks the current state of flow execution
#[derive(Debug, Clone)]
pub struct ExecutionState {
    pub execution_id: FlowExecutionId,
    pub current_node: Option<String>,
    pub variables: HashMap<String, Value>,
    pub node_results: HashMap<String, NodeExecutionResult>,
    pub visited_nodes: Vec<String>,
    pub loop_counters: HashMap<String, usize>,
}

impl ExecutionState {
    pub fn new(execution_id: FlowExecutionId, initial_variables: HashMap<String, Value>) -> Self {
        Self {
            execution_id,
            current_node: None,
            variables: initial_variables,
            node_results: HashMap::new(),
            visited_nodes: Vec::new(),
            loop_counters: HashMap::new(),
        }
    }

    /// Create execution state with tenant and user context
    pub fn with_context(
        execution_id: FlowExecutionId,
        tenant_id: uuid::Uuid,
        user_id: uuid::Uuid,
        session_id: Option<uuid::Uuid>,
        initial_variables: HashMap<String, Value>,
    ) -> Self {
        let mut variables = initial_variables;

        // Inject context into variables for use by node executors
        variables.insert(
            "tenant_id".to_string(),
            Value::String(tenant_id.to_string()),
        );
        variables.insert("user_id".to_string(), Value::String(user_id.to_string()));

        if let Some(sid) = session_id {
            variables.insert("session_id".to_string(), Value::String(sid.to_string()));
        }

        Self {
            execution_id,
            current_node: None,
            variables,
            node_results: HashMap::new(),
            visited_nodes: Vec::new(),
            loop_counters: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn record_node_result(&mut self, result: NodeExecutionResult) {
        self.visited_nodes.push(result.node_id.clone());
        self.node_results.insert(result.node_id.clone(), result);
    }

    pub fn increment_loop_counter(&mut self, loop_id: &str) -> usize {
        let counter = self.loop_counters.entry(loop_id.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    pub fn get_loop_counter(&self, loop_id: &str) -> usize {
        self.loop_counters.get(loop_id).copied().unwrap_or(0)
    }

    pub fn reset_loop_counter(&mut self, loop_id: &str) {
        self.loop_counters.insert(loop_id.to_string(), 0);
    }
}

/// Node executor trait for executing different types of nodes
#[async_trait]
pub trait NodeExecutor: Send + Sync {
    /// Execute a node with the given state
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult>;

    /// Check if this executor can handle the given node type
    fn can_handle(&self, node_type: &NodeType) -> bool;
}

/// Execution engine trait
#[async_trait]
pub trait ExecutionEngine: Send + Sync {
    /// Execute a flow with the given definition and initial state
    async fn execute(
        &self,
        execution: &mut FlowExecution,
        definition: &FlowDefinition,
        initial_variables: HashMap<String, Value>,
    ) -> Result<ExecutionState>;

    /// Execute a single node
    async fn execute_node(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult>;

    /// Get the next nodes to execute based on current node and state
    fn get_next_nodes(
        &self,
        current_node: &FlowNode,
        definition: &FlowDefinition,
        state: &ExecutionState,
    ) -> Result<Vec<String>>;

    /// Evaluate a condition for conditional branching
    fn evaluate_condition(&self, condition: &Value, state: &ExecutionState) -> Result<bool>;
}

/// Default implementation of ExecutionEngine
pub struct ExecutionEngineImpl {
    node_executors: Vec<Arc<dyn NodeExecutor>>,
    max_iterations: usize,
}

impl ExecutionEngineImpl {
    pub fn new(node_executors: Vec<Arc<dyn NodeExecutor>>) -> Self {
        Self {
            node_executors,
            max_iterations: 1000, // Prevent infinite loops
        }
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    fn find_executor(&self, node_type: &NodeType) -> Option<&Arc<dyn NodeExecutor>> {
        self.node_executors.iter().find(|e| e.can_handle(node_type))
    }

    fn find_node_by_id<'a>(
        &self,
        node_id: &str,
        definition: &'a FlowDefinition,
    ) -> Option<&'a FlowNode> {
        definition
            .workflow
            .graph
            .nodes
            .iter()
            .find(|n| n.id == node_id)
    }

    fn get_outgoing_edges<'a>(
        &self,
        node_id: &str,
        definition: &'a FlowDefinition,
    ) -> Vec<&'a crate::domain::value_objects::FlowEdge> {
        definition
            .workflow
            .graph
            .edges
            .iter()
            .filter(|e| e.source == node_id)
            .collect()
    }
}

#[async_trait]
impl ExecutionEngine for ExecutionEngineImpl {
    async fn execute(
        &self,
        execution: &mut FlowExecution,
        definition: &FlowDefinition,
        initial_variables: HashMap<String, Value>,
    ) -> Result<ExecutionState> {
        // Mark execution as running
        execution.start();

        // Initialize execution state with tenant and user context for isolation
        let mut state = ExecutionState::with_context(
            execution.id,
            execution.tenant_id.0,
            execution.user_id.0,
            execution.session_id.map(|sid| sid.0),
            initial_variables,
        );

        // Find start nodes
        let start_nodes = definition.get_start_nodes();
        if start_nodes.is_empty() {
            execution.fail("No start node found in flow definition".to_string());
            return Err(PlatformError::ValidationError(
                "No start node found".to_string(),
            ));
        }

        // Use the first start node
        let start_node = start_nodes[0];
        let mut current_nodes = vec![start_node.id.clone()];
        let mut iteration_count = 0;

        // Execute nodes until we reach an end node or max iterations
        while !current_nodes.is_empty() && iteration_count < self.max_iterations {
            iteration_count += 1;
            let mut next_nodes = Vec::new();

            for node_id in current_nodes {
                let node = match self.find_node_by_id(&node_id, definition) {
                    Some(n) => n,
                    None => {
                        let error = format!("Node not found: {}", node_id);
                        execution.fail(error.clone());
                        return Err(PlatformError::ValidationError(error));
                    }
                };

                // Execute the node
                state.current_node = Some(node_id.clone());
                let result = self.execute_node(node, &mut state).await?;
                state.record_node_result(result.clone());

                // Check if this is an end or answer node
                if node.node_type == NodeType::End || node.node_type == NodeType::Answer {
                    // Collect final output from state
                    let output = serde_json::json!({
                        "variables": state.variables,
                        "visited_nodes": state.visited_nodes,
                    });
                    execution.complete(output);
                    return Ok(state);
                }

                // Check if node execution failed
                if result.status == NodeExecutionStatus::Failed {
                    let error = result
                        .error
                        .unwrap_or_else(|| "Node execution failed".to_string());
                    execution.fail(error.clone());
                    return Err(PlatformError::InternalError(error));
                }

                // Get next nodes to execute
                let next = self.get_next_nodes(node, definition, &state)?;
                next_nodes.extend(next);
            }

            current_nodes = next_nodes;
        }

        // Check if we hit max iterations
        if iteration_count >= self.max_iterations {
            let error = format!(
                "Flow execution exceeded maximum iterations: {}",
                self.max_iterations
            );
            execution.fail(error.clone());
            return Err(PlatformError::InternalError(error));
        }

        // If we exit the loop without reaching an end node, it's an error
        let error = "Flow execution completed without reaching an end node".to_string();
        execution.fail(error.clone());
        Err(PlatformError::InternalError(error))
    }

    async fn execute_node(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Find appropriate executor for this node type
        let executor = match self.find_executor(&node.node_type) {
            Some(e) => e,
            None => {
                return Ok(NodeExecutionResult {
                    node_id: node.id.clone(),
                    status: NodeExecutionStatus::Failed,
                    output: None,
                    error: Some(format!(
                        "No executor found for node type: {:?}",
                        node.node_type
                    )),
                    started_at,
                    completed_at: Utc::now(),
                    execution_time_ms: 0,
                });
            }
        };

        // Execute the node
        let result = executor.execute(node, state).await?;
        Ok(result)
    }

    fn get_next_nodes(
        &self,
        current_node: &FlowNode,
        definition: &FlowDefinition,
        state: &ExecutionState,
    ) -> Result<Vec<String>> {
        let mut next_nodes = Vec::new();

        match current_node.node_type {
            NodeType::Condition => {
                // For condition nodes, evaluate the condition and choose the appropriate branch
                let edges = self.get_outgoing_edges(&current_node.id, definition);

                for edge in edges {
                    // Check if this edge has a condition in the source_handle
                    let should_follow = if let Some(ref handle) = edge.source_handle {
                        // Handle can be "true" or "false" for condition branches
                        match handle.as_str() {
                            "true" => {
                                // Evaluate condition from node data
                                if let Some(condition) = current_node.data.get("condition") {
                                    self.evaluate_condition(condition, state)?
                                } else {
                                    false
                                }
                            }
                            "false" => {
                                // Evaluate negated condition
                                if let Some(condition) = current_node.data.get("condition") {
                                    !self.evaluate_condition(condition, state)?
                                } else {
                                    true
                                }
                            }
                            _ => true, // Default case
                        }
                    } else {
                        true // No condition, follow the edge
                    };

                    if should_follow {
                        next_nodes.push(edge.target.clone());
                    }
                }
            }
            NodeType::Loop => {
                // For loop nodes, check if we should continue looping or exit
                let loop_id = &current_node.id;
                let max_iterations = current_node
                    .data
                    .get("max_iterations")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;

                let current_iteration = state.get_loop_counter(loop_id);

                if current_iteration < max_iterations {
                    // Continue looping - find the loop body edge
                    let edges = self.get_outgoing_edges(&current_node.id, definition);
                    for edge in edges {
                        if edge.source_handle.as_deref() == Some("loop") {
                            next_nodes.push(edge.target.clone());
                            break;
                        }
                    }
                } else {
                    // Exit loop - find the exit edge
                    let edges = self.get_outgoing_edges(&current_node.id, definition);
                    for edge in edges {
                        if edge.source_handle.as_deref() == Some("exit")
                            || edge.source_handle.is_none()
                        {
                            next_nodes.push(edge.target.clone());
                            break;
                        }
                    }
                    // Reset loop counter
                    // Note: We don't reset here to allow nested loops to work correctly
                }
            }
            _ => {
                // For other node types, follow all outgoing edges
                let edges = self.get_outgoing_edges(&current_node.id, definition);
                for edge in edges {
                    next_nodes.push(edge.target.clone());
                }
            }
        }

        Ok(next_nodes)
    }

    fn evaluate_condition(&self, condition: &Value, state: &ExecutionState) -> Result<bool> {
        // Simple condition evaluation
        // Supports: {"variable": "var_name", "operator": "==", "value": "expected_value"}

        if let Some(obj) = condition.as_object() {
            let variable_name = obj
                .get("variable")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    PlatformError::ValidationError("Condition missing 'variable' field".to_string())
                })?;

            let operator = obj
                .get("operator")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    PlatformError::ValidationError("Condition missing 'operator' field".to_string())
                })?;

            let expected_value = obj.get("value").ok_or_else(|| {
                PlatformError::ValidationError("Condition missing 'value' field".to_string())
            })?;

            let actual_value = state.get_variable(variable_name).ok_or_else(|| {
                PlatformError::ValidationError(format!("Variable not found: {}", variable_name))
            })?;

            let result = match operator {
                "==" | "eq" => actual_value == expected_value,
                "!=" | "ne" => actual_value != expected_value,
                ">" | "gt" => {
                    if let (Some(a), Some(b)) = (actual_value.as_f64(), expected_value.as_f64()) {
                        a > b
                    } else {
                        false
                    }
                }
                "<" | "lt" => {
                    if let (Some(a), Some(b)) = (actual_value.as_f64(), expected_value.as_f64()) {
                        a < b
                    } else {
                        false
                    }
                }
                ">=" | "gte" => {
                    if let (Some(a), Some(b)) = (actual_value.as_f64(), expected_value.as_f64()) {
                        a >= b
                    } else {
                        false
                    }
                }
                "<=" | "lte" => {
                    if let (Some(a), Some(b)) = (actual_value.as_f64(), expected_value.as_f64()) {
                        a <= b
                    } else {
                        false
                    }
                }
                "contains" => {
                    if let (Some(a), Some(b)) = (actual_value.as_str(), expected_value.as_str()) {
                        a.contains(b)
                    } else {
                        false
                    }
                }
                _ => {
                    return Err(PlatformError::ValidationError(format!(
                        "Unknown operator: {}",
                        operator
                    )));
                }
            };

            Ok(result)
        } else {
            Err(PlatformError::ValidationError(
                "Condition must be a JSON object".to_string(),
            ))
        }
    }
}
