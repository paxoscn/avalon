use agent_platform::domain::services::{
    execution_engine::{ExecutionEngine, ExecutionState, NodeExecutionStatus},
    ExecutionEngineFactory,
};
use agent_platform::domain::value_objects::{
    FlowDefinition, FlowEdge, FlowGraph, FlowNode, FlowWorkflow, NodePosition, NodeType,
};
use agent_platform::domain::entities::FlowExecution;
use agent_platform::domain::value_objects::{FlowExecutionId, TenantId, UserId};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_iteration_node_basic() {
    // Create a simple flow with iteration
    // Start -> ParameterExtractor -> Iteration -> Answer
    
    let flow_definition = FlowDefinition {
        workflow: FlowWorkflow {
            graph: FlowGraph {
                nodes: vec![
                    // Start node
                    FlowNode {
                        id: "start_1".to_string(),
                        parent_id: None,
                        node_type: NodeType::Start,
                        data: json!({
                            "variables": [
                                {"variable": "user_input", "default": "Check items: apple, banana, orange"}
                            ]
                        }),
                        position: NodePosition { x: 0.0, y: 0.0 },
                    },
                    // Parameter Extractor node - extracts items as array
                    FlowNode {
                        id: "extractor_1".to_string(),
                        parent_id: None,
                        node_type: NodeType::ParameterExtractor,
                        data: json!({
                            "query": ["start_1", "user_input"],
                            "instruction": "Extract all items from the text as a JSON array of strings",
                            "parameters": [{"name": "items"}],
                            "model": {
                                "llm_config_id": "test-config-id"
                            }
                        }),
                        position: NodePosition { x: 100.0, y: 0.0 },
                    },
                    // Iteration node
                    FlowNode {
                        id: "iteration_1".to_string(),
                        parent_id: None,
                        node_type: NodeType::Iteration,
                        data: json!({
                            "iterator_selector": ["extractor_1", "items"],
                            "output_selector": ["llm_2", "text"],
                            "start_node_id": "llm_1"
                        }),
                        position: NodePosition { x: 200.0, y: 0.0 },
                    },
                    // LLM node inside iteration (processes each item)
                    FlowNode {
                        id: "llm_1".to_string(),
                        parent_id: None,
                        node_type: NodeType::Llm,
                        data: json!({
                            "prompt_template": [
                                {
                                    "role": "user",
                                    "text": "Process this item: {{#llm_1.item#}}"
                                }
                            ],
                            "model": {
                                "llm_config_id": "test-config-id"
                            }
                        }),
                        position: NodePosition { x: 250.0, y: 50.0 },
                    },
                    // LLM node that collects results
                    FlowNode {
                        id: "llm_2".to_string(),
                        parent_id: None,
                        node_type: NodeType::Llm,
                        data: json!({
                            "prompt_template": [
                                {
                                    "role": "user",
                                    "text": "Result for {{#llm_1.item#}}: {{#llm_1.text#}}"
                                }
                            ],
                            "model": {
                                "llm_config_id": "test-config-id"
                            }
                        }),
                        position: NodePosition { x: 300.0, y: 50.0 },
                    },
                    // Answer node
                    FlowNode {
                        id: "answer_1".to_string(),
                        parent_id: None,
                        node_type: NodeType::Answer,
                        data: json!({
                            "answer": "Processed items: {{#llm_2.text#}}"
                        }),
                        position: NodePosition { x: 400.0, y: 0.0 },
                    },
                ],
                edges: vec![
                    FlowEdge {
                        id: "e1".to_string(),
                        source: "start_1".to_string(),
                        target: "extractor_1".to_string(),
                        source_handle: None,
                        target_handle: None,
                    },
                    FlowEdge {
                        id: "e2".to_string(),
                        source: "extractor_1".to_string(),
                        target: "iteration_1".to_string(),
                        source_handle: None,
                        target_handle: None,
                    },
                    FlowEdge {
                        id: "e3".to_string(),
                        source: "llm_1".to_string(),
                        target: "llm_2".to_string(),
                        source_handle: None,
                        target_handle: None,
                    },
                    FlowEdge {
                        id: "e4".to_string(),
                        source: "iteration_1".to_string(),
                        target: "answer_1".to_string(),
                        source_handle: None,
                        target_handle: None,
                    },
                ],
            },
        },
    };

    // Validate the flow
    assert!(flow_definition.validate().is_ok());

    println!("✓ Iteration node flow definition created and validated");
}

#[test]
fn test_iteration_node_data_structure() {
    // Test that iteration node data structure is correctly defined
    let iteration_data = json!({
        "iterator_selector": ["node_1", "items"],
        "output_selector": ["node_2", "results"],
        "start_node_id": "sub_flow_start"
    });

    assert!(iteration_data.get("iterator_selector").is_some());
    assert!(iteration_data.get("output_selector").is_some());
    assert!(iteration_data.get("start_node_id").is_some());

    let iterator_selector = iteration_data["iterator_selector"].as_array().unwrap();
    assert_eq!(iterator_selector.len(), 2);
    assert_eq!(iterator_selector[0], "node_1");
    assert_eq!(iterator_selector[1], "items");

    println!("✓ Iteration node data structure validated");
}
