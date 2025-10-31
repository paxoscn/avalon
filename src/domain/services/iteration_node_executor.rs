use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;

use crate::domain::services::execution_engine::{
    ExecutionState, NodeExecutionResult, NodeExecutionStatus, NodeExecutor,
};
use crate::domain::value_objects::{FlowNode, NodeType};
use crate::error::Result;

/// Iteration node executor - iterates over an array and executes a sub-flow for each item
pub struct IterationNodeExecutor;

impl IterationNodeExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IterationNodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeExecutor for IterationNodeExecutor {
    async fn execute(
        &self,
        node: &FlowNode,
        state: &mut ExecutionState,
    ) -> Result<NodeExecutionResult> {
        let started_at = Utc::now();

        // Extract iterator_selector: [node_id, variable_name]
        let iterator_selector = node
            .data
            .get("iterator_selector")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "Iteration node missing 'iterator_selector' field".to_string(),
                )
            })?;

        if iterator_selector.len() != 2 {
            let completed_at = Utc::now();
            let execution_time_ms = completed_at
                .signed_duration_since(started_at)
                .num_milliseconds();
            return Ok(NodeExecutionResult {
                node_id: node.id.clone(),
                status: NodeExecutionStatus::Failed,
                output: None,
                error: Some("iterator_selector must have exactly 2 elements [node_id, variable_name]".to_string()),
                started_at,
                completed_at,
                execution_time_ms,
            });
        }

        let iterator_node_id = iterator_selector[0].as_str().ok_or_else(|| {
            crate::error::PlatformError::ValidationError(
                "iterator_selector[0] must be a string".to_string(),
            )
        })?;
        let iterator_var_name = iterator_selector[1].as_str().ok_or_else(|| {
            crate::error::PlatformError::ValidationError(
                "iterator_selector[1] must be a string".to_string(),
            )
        })?;

        // Extract output_selector: [node_id, variable_name]
        let output_selector = node
            .data
            .get("output_selector")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "Iteration node missing 'output_selector' field".to_string(),
                )
            })?;

        if output_selector.len() != 2 {
            let completed_at = Utc::now();
            let execution_time_ms = completed_at
                .signed_duration_since(started_at)
                .num_milliseconds();
            return Ok(NodeExecutionResult {
                node_id: node.id.clone(),
                status: NodeExecutionStatus::Failed,
                output: None,
                error: Some("output_selector must have exactly 2 elements [node_id, variable_name]".to_string()),
                started_at,
                completed_at,
                execution_time_ms,
            });
        }

        let output_node_id = output_selector[0].as_str().ok_or_else(|| {
            crate::error::PlatformError::ValidationError(
                "output_selector[0] must be a string".to_string(),
            )
        })?;
        let output_var_name = output_selector[1].as_str().ok_or_else(|| {
            crate::error::PlatformError::ValidationError(
                "output_selector[1] must be a string".to_string(),
            )
        })?;

        // Extract start_node_id
        let start_node_id = node
            .data
            .get("start_node_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(
                    "Iteration node missing 'start_node_id' field".to_string(),
                )
            })?;

        // Get the iterator array from state
        let iterator_var_key = format!("#{}.{}#", iterator_node_id, iterator_var_name);
        let iterator_array = state
            .get_variable(&iterator_var_key)
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::error::PlatformError::ValidationError(format!(
                    "Iterator variable '{}' not found or not an array",
                    iterator_var_key
                ))
            })?
            .clone();

        // Initialize output array
        let output_array: Vec<Value> = Vec::new();

        // Store iteration metadata
        let iteration_count = iterator_array.len();
        let iteration_meta_key = format!("#{}.iteration_count#", node.id);
        state.set_variable(iteration_meta_key, serde_json::json!(iteration_count));

        // Store the iteration configuration in the state for the execution engine to use
        let iteration_config = serde_json::json!({
            "iterator_array": iterator_array,
            "start_node_id": start_node_id,
            "output_node_id": output_node_id,
            "output_var_name": output_var_name,
            "current_index": 0,
            "total_count": iteration_count,
            "output_array": output_array,
        });

        let iteration_config_key = format!("#{}.iteration_config#", node.id);
        state.set_variable(iteration_config_key, iteration_config);

        let output = serde_json::json!({
            "message": "Iteration prepared",
            "iteration_count": iteration_count,
            "start_node_id": start_node_id,
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
        matches!(node_type, NodeType::Iteration)
    }
}
