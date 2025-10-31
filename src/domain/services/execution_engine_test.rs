#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use serde_json::json;

    use crate::domain::entities::FlowExecution;
    use crate::domain::services::execution_engine::{ExecutionEngine, ExecutionEngineImpl, ExecutionState};
    use crate::domain::services::node_executors::*;
    use crate::domain::value_objects::{
        FlowDefinition, FlowNode, FlowEdge, FlowVariable, FlowMetadata,
        NodeType, NodePosition, VariableType, FlowId, TenantId, UserId, Version, SessionId,
    };

    fn create_simple_flow() -> FlowDefinition {
        FlowDefinition {
            nodes: vec![
                FlowNode {
                    id: "start".to_string(),
                    parent_id: None,
                    node_type: NodeType::Start,
                    title: "Start".to_string(),
                    data: json!({}),
                    position: NodePosition { x: 0.0, y: 0.0 },
                },
                FlowNode {
                    id: "end".to_string(),
                    parent_id: None,
                    node_type: NodeType::End,
                    title: "End".to_string(),
                    data: json!({}),
                    position: NodePosition { x: 200.0, y: 0.0 },
                },
            ],
            edges: vec![
                FlowEdge {
                    id: "e1".to_string(),
                    parent_id: None,
                    source: "start".to_string(),
                    target: "end".to_string(),
                    source_handle: None,
                    target_handle: None,
                },
            ],
            variables: vec![],
            metadata: FlowMetadata {
                description: Some("Simple flow".to_string()),
                tags: vec![],
                author: "Test".to_string(),
                version: "1.0.0".to_string(),
            },
        }
    }

    fn create_flow_with_variables() -> FlowDefinition {
        FlowDefinition {
            nodes: vec![
                FlowNode {
                    id: "start".to_string(),
                    parent_id: None,
                    node_type: NodeType::Start,
                    title: "Start".to_string(),
                    data: json!({}),
                    position: NodePosition { x: 0.0, y: 0.0 },
                },
                FlowNode {
                    id: "var1".to_string(),
                    parent_id: None,
                    node_type: NodeType::Variable,
                    title: "Set Variable".to_string(),
                    data: json!({
                        "assignments": [
                            {"name": "result", "value": "success"}
                        ]
                    }),
                    position: NodePosition { x: 100.0, y: 0.0 },
                },
                FlowNode {
                    id: "end".to_string(),
                    parent_id: None,
                    node_type: NodeType::End,
                    title: "End".to_string(),
                    data: json!({}),
                    position: NodePosition { x: 200.0, y: 0.0 },
                },
            ],
            edges: vec![
                FlowEdge {
                    id: "e1".to_string(),
                    source: "start".to_string(),
                    target: "var1".to_string(),
                    source_handle: None,
                    target_handle: None,
                },
                FlowEdge {
                    id: "e2".to_string(),
                    source: "var1".to_string(),
                    target: "end".to_string(),
                    source_handle: None,
                    target_handle: None,
                },
            ],
            variables: vec![
                FlowVariable {
                    name: "result".to_string(),
                    variable_type: VariableType::String,
                    default_value: Some(json!("pending")),
                    required: false,
                    description: Some("Result variable".to_string()),
                },
            ],
            metadata: FlowMetadata {
                description: Some("Flow with variables".to_string()),
                tags: vec![],
                author: "Test".to_string(),
                version: "1.0.0".to_string(),
            },
        }
    }

    fn create_flow_with_condition() -> FlowDefinition {
        FlowDefinition {
            nodes: vec![
                FlowNode {
                    id: "start".to_string(),
                    parent_id: None,
                    node_type: NodeType::Start,
                    title: "Start".to_string(),
                    data: json!({}),
                    position: NodePosition { x: 0.0, y: 0.0 },
                },
                FlowNode {
                    id: "condition".to_string(),
                    parent_id: None,
                    node_type: NodeType::Condition,
                    title: "Check Value".to_string(),
                    data: json!({
                        "condition": {
                            "variable": "input",
                            "operator": "==",
                            "value": "test"
                        }
                    }),
                    position: NodePosition { x: 100.0, y: 0.0 },
                },
                FlowNode {
                    id: "true_branch".to_string(),
                    parent_id: None,
                    node_type: NodeType::Variable,
                    title: "True Branch".to_string(),
                    data: json!({
                        "assignments": [
                            {"name": "result", "value": "condition_true"}
                        ]
                    }),
                    position: NodePosition { x: 200.0, y: -50.0 },
                },
                FlowNode {
                    id: "false_branch".to_string(),
                    parent_id: None,
                    node_type: NodeType::Variable,
                    title: "False Branch".to_string(),
                    data: json!({
                        "assignments": [
                            {"name": "result", "value": "condition_false"}
                        ]
                    }),
                    position: NodePosition { x: 200.0, y: 50.0 },
                },
                FlowNode {
                    id: "end".to_string(),
                    parent_id: None,
                    node_type: NodeType::End,
                    title: "End".to_string(),
                    data: json!({}),
                    position: NodePosition { x: 300.0, y: 0.0 },
                },
            ],
            edges: vec![
                FlowEdge {
                    id: "e1".to_string(),
                    source: "start".to_string(),
                    target: "condition".to_string(),
                    source_handle: None,
                    target_handle: None,
                },
                FlowEdge {
                    id: "e2".to_string(),
                    source: "condition".to_string(),
                    target: "true_branch".to_string(),
                    source_handle: Some("true".to_string()),
                    target_handle: None,
                },
                FlowEdge {
                    id: "e3".to_string(),
                    source: "condition".to_string(),
                    target: "false_branch".to_string(),
                    source_handle: Some("false".to_string()),
                    target_handle: None,
                },
                FlowEdge {
                    id: "e4".to_string(),
                    source: "true_branch".to_string(),
                    target: "end".to_string(),
                    source_handle: None,
                    target_handle: None,
                },
                FlowEdge {
                    id: "e5".to_string(),
                    source: "false_branch".to_string(),
                    target: "end".to_string(),
                    source_handle: None,
                    target_handle: None,
                },
            ],
            variables: vec![],
            metadata: FlowMetadata {
                description: Some("Flow with condition".to_string()),
                tags: vec![],
                author: "Test".to_string(),
                version: "1.0.0".to_string(),
            },
        }
    }

    fn create_flow_with_loop() -> FlowDefinition {
        FlowDefinition {
            nodes: vec![
                FlowNode {
                    id: "start".to_string(),
                    parent_id: None,
                    node_type: NodeType::Start,
                    title: "Start".to_string(),
                    data: json!({}),
                    position: NodePosition { x: 0.0, y: 0.0 },
                },
                FlowNode {
                    id: "loop".to_string(),
                    parent_id: None,
                    node_type: NodeType::Loop,
                    title: "Loop".to_string(),
                    data: json!({"max_iterations": 3}),
                    position: NodePosition { x: 100.0, y: 0.0 },
                },
                FlowNode {
                    id: "loop_body".to_string(),
                    parent_id: None,
                    node_type: NodeType::Variable,
                    title: "Loop Body".to_string(),
                    data: json!({
                        "assignments": [
                            {"name": "counter", "value": 1}
                        ]
                    }),
                    position: NodePosition { x: 200.0, y: 0.0 },
                },
                FlowNode {
                    id: "end".to_string(),
                    parent_id: None,
                    node_type: NodeType::End,
                    title: "End".to_string(),
                    data: json!({}),
                    position: NodePosition { x: 300.0, y: 0.0 },
                },
            ],
            edges: vec![
                FlowEdge {
                    id: "e1".to_string(),
                    source: "start".to_string(),
                    target: "loop".to_string(),
                    source_handle: None,
                    target_handle: None,
                },
                FlowEdge {
                    id: "e2".to_string(),
                    source: "loop".to_string(),
                    target: "loop_body".to_string(),
                    source_handle: Some("loop".to_string()),
                    target_handle: None,
                },
                FlowEdge {
                    id: "e3".to_string(),
                    source: "loop_body".to_string(),
                    target: "loop".to_string(),
                    source_handle: None,
                    target_handle: None,
                },
                FlowEdge {
                    id: "e4".to_string(),
                    source: "loop".to_string(),
                    target: "end".to_string(),
                    source_handle: Some("exit".to_string()),
                    target_handle: None,
                },
            ],
            variables: vec![],
            metadata: FlowMetadata {
                description: Some("Flow with loop".to_string()),
                tags: vec![],
                author: "Test".to_string(),
                version: "1.0.0".to_string(),
            },
        }
    }

    fn create_execution_engine() -> ExecutionEngineImpl {
        let executors: Vec<Arc<dyn crate::domain::services::execution_engine::NodeExecutor>> = vec![
            Arc::new(StartNodeExecutor::new()),
            Arc::new(EndNodeExecutor::new()),
            Arc::new(VariableNodeExecutor::new()),
            Arc::new(ConditionNodeExecutor::new()),
            Arc::new(LoopNodeExecutor::new()),
            Arc::new(CodeNodeExecutor::new()),
            Arc::new(HttpRequestNodeExecutor::new()),
            Arc::new(AnswerNodeExecutor::new()),
        ];
        ExecutionEngineImpl::new(executors)
    }

    fn create_test_execution() -> FlowExecution {
        FlowExecution::new(
            FlowId::new(),
            Version::new(),
            TenantId::new(),
            UserId::new(),
            None,
            Some(json!({})),
        )
    }

    #[tokio::test]
    async fn test_execute_simple_flow() {
        let engine = create_execution_engine();
        let definition = create_simple_flow();
        let mut execution = create_test_execution();
        let initial_variables = HashMap::new();

        let result = engine.execute(&mut execution, &definition, initial_variables).await;
        
        assert!(result.is_ok());
        let state = result.unwrap();
        assert_eq!(state.visited_nodes.len(), 2); // start and end
        assert!(execution.is_completed());
    }

    #[tokio::test]
    async fn test_execute_flow_with_variables() {
        let engine = create_execution_engine();
        let definition = create_flow_with_variables();
        let mut execution = create_test_execution();
        let initial_variables = HashMap::new();

        let result = engine.execute(&mut execution, &definition, initial_variables).await;
        
        assert!(result.is_ok());
        let state = result.unwrap();
        assert_eq!(state.visited_nodes.len(), 3); // start, var1, end
        assert_eq!(state.get_variable("result"), Some(&json!("success")));
        assert!(execution.is_completed());
    }

    #[tokio::test]
    async fn test_execute_flow_with_condition_true() {
        let engine = create_execution_engine();
        let definition = create_flow_with_condition();
        let mut execution = create_test_execution();
        
        let mut initial_variables = HashMap::new();
        initial_variables.insert("input".to_string(), json!("test"));

        let result = engine.execute(&mut execution, &definition, initial_variables).await;
        
        assert!(result.is_ok());
        let state = result.unwrap();
        assert!(state.visited_nodes.contains(&"true_branch".to_string()));
        assert!(!state.visited_nodes.contains(&"false_branch".to_string()));
        assert_eq!(state.get_variable("result"), Some(&json!("condition_true")));
        assert!(execution.is_completed());
    }

    #[tokio::test]
    async fn test_execute_flow_with_condition_false() {
        let engine = create_execution_engine();
        let definition = create_flow_with_condition();
        let mut execution = create_test_execution();
        
        let mut initial_variables = HashMap::new();
        initial_variables.insert("input".to_string(), json!("other"));

        let result = engine.execute(&mut execution, &definition, initial_variables).await;
        
        assert!(result.is_ok());
        let state = result.unwrap();
        assert!(!state.visited_nodes.contains(&"true_branch".to_string()));
        assert!(state.visited_nodes.contains(&"false_branch".to_string()));
        assert_eq!(state.get_variable("result"), Some(&json!("condition_false")));
        assert!(execution.is_completed());
    }

    #[tokio::test]
    async fn test_execute_flow_with_loop() {
        let engine = create_execution_engine();
        let definition = create_flow_with_loop();
        let mut execution = create_test_execution();
        let initial_variables = HashMap::new();

        let result = engine.execute(&mut execution, &definition, initial_variables).await;
        
        assert!(result.is_ok());
        let state = result.unwrap();
        
        // Loop should execute 3 times (max_iterations)
        let loop_executions = state.visited_nodes.iter()
            .filter(|n| *n == "loop")
            .count();
        assert_eq!(loop_executions, 3);
        
        assert!(execution.is_completed());
    }

    #[tokio::test]
    async fn test_condition_evaluation() {
        let engine = create_execution_engine();
        let state = ExecutionState::new(
            crate::domain::value_objects::FlowExecutionId::new(),
            {
                let mut vars = HashMap::new();
                vars.insert("num".to_string(), json!(42));
                vars.insert("text".to_string(), json!("hello world"));
                vars
            },
        );

        // Test equality
        let condition = json!({"variable": "num", "operator": "==", "value": 42});
        assert!(engine.evaluate_condition(&condition, &state).unwrap());

        // Test greater than
        let condition = json!({"variable": "num", "operator": ">", "value": 40});
        assert!(engine.evaluate_condition(&condition, &state).unwrap());

        // Test less than
        let condition = json!({"variable": "num", "operator": "<", "value": 50});
        assert!(engine.evaluate_condition(&condition, &state).unwrap());

        // Test contains
        let condition = json!({"variable": "text", "operator": "contains", "value": "world"});
        assert!(engine.evaluate_condition(&condition, &state).unwrap());

        // Test not equal
        let condition = json!({"variable": "num", "operator": "!=", "value": 100});
        assert!(engine.evaluate_condition(&condition, &state).unwrap());
    }

    #[tokio::test]
    async fn test_max_iterations_protection() {
        let engine = create_execution_engine().with_max_iterations(10);
        
        // Create a flow with an infinite loop (no exit condition)
        let definition = FlowDefinition {
            nodes: vec![
                FlowNode {
                    id: "start".to_string(),
                    parent_id: None,
                    node_type: NodeType::Start,
                    title: "Start".to_string(),
                    data: json!({}),
                    position: NodePosition { x: 0.0, y: 0.0 },
                },
                FlowNode {
                    id: "loop".to_string(),
                    parent_id: None,
                    node_type: NodeType::Variable,
                    title: "Loop".to_string(),
                    data: json!({}),
                    position: NodePosition { x: 100.0, y: 0.0 },
                },
            ],
            edges: vec![
                FlowEdge {
                    id: "e1".to_string(),
                    source: "start".to_string(),
                    target: "loop".to_string(),
                    source_handle: None,
                    target_handle: None,
                },
                FlowEdge {
                    id: "e2".to_string(),
                    source: "loop".to_string(),
                    target: "loop".to_string(),
                    source_handle: None,
                    target_handle: None,
                },
            ],
            variables: vec![],
            metadata: FlowMetadata {
                description: Some("Infinite loop".to_string()),
                tags: vec![],
                author: "Test".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let mut execution = create_test_execution();
        let initial_variables = HashMap::new();

        let result = engine.execute(&mut execution, &definition, initial_variables).await;
        
        assert!(result.is_err());
        assert!(execution.is_failed());
    }
}
