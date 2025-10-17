use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowDefinition {
    pub workflow: FlowWorkflow,
    // pub variables: Vec<FlowVariable>,
    // pub metadata: FlowMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowWorkflow {
    pub graph: FlowGraph,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowGraph {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub node_type: NodeType,
    // pub title: String,
    pub data: Value,
    pub position: NodePosition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "sourceHandle")]
    pub source_handle: Option<String>,
    #[serde(rename = "targetHandle")]
    pub target_handle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowVariable {
    pub name: String,
    pub variable_type: VariableType,
    pub default_value: Option<Value>,
    pub required: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub author: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Start,
    End,
    Llm,
    VectorSearch,
    McpTool,
    Condition,
    Loop,
    Variable,
    HttpRequest,
    Code,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

impl FlowDefinition {
    pub fn new() -> Self {
        FlowDefinition {
            workflow: FlowWorkflow {
                graph: FlowGraph {
                    nodes: Vec::new(),
                    edges: Vec::new(),
                },
            },
            // variables: Vec::new(),
            // metadata: FlowMetadata {
            //     description: None,
            //     tags: Vec::new(),
            //     author: String::new(),
            //     version: "1.0.0".to_string(),
            // },
        }
    }

    /// Parse FlowDefinition from Dify DSL string
    pub fn from_dsl(dsl: &str) -> Result<Self, String> {
        // Parse the DSL JSON string
        serde_json::from_str(dsl)
            .map_err(|e| format!("Failed to parse DSL: {}", e))
    }

    /// Parse FlowDefinition from JSON value
    pub fn from_json(json: &Value) -> Result<Self, String> {
        serde_json::from_value(json.clone())
            .map_err(|e| format!("Failed to parse flow definition: {}", e))
    }

    /// Convert FlowDefinition to JSON value
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    pub fn validate(&self) -> Result<(), String> {
        // Check if there's at least one start node
        let start_nodes: Vec<_> = self.workflow.graph.nodes.iter()
            .filter(|n| n.node_type == NodeType::Start)
            .collect();
        
        if start_nodes.is_empty() {
            return Err("Flow must have at least one start node".to_string());
        }

        // Check if there's at least one end node
        let end_nodes: Vec<_> = self.workflow.graph.nodes.iter()
            .filter(|n| n.node_type == NodeType::End)
            .collect();
        
        if end_nodes.is_empty() {
            return Err("Flow must have at least one end node".to_string());
        }

        // Validate node IDs are unique
        let mut node_ids = std::collections::HashSet::new();
        for node in &self.workflow.graph.nodes {
            if !node_ids.insert(&node.id) {
                return Err(format!("Duplicate node ID: {}", node.id));
            }
        }

        // Validate edges reference existing nodes
        for edge in &self.workflow.graph.edges {
            if !node_ids.contains(&edge.source) {
                return Err(format!("Edge references non-existent source node: {}", edge.source));
            }
            if !node_ids.contains(&edge.target) {
                return Err(format!("Edge references non-existent target node: {}", edge.target));
            }
        }

        Ok(())
    }

    pub fn get_start_nodes(&self) -> Vec<&FlowNode> {
        self.workflow.graph.nodes.iter()
            .filter(|n| n.node_type == NodeType::Start)
            .collect()
    }

    pub fn get_end_nodes(&self) -> Vec<&FlowNode> {
        self.workflow.graph.nodes.iter()
            .filter(|n| n.node_type == NodeType::End)
            .collect()
    }
}

impl Default for FlowDefinition {
    fn default() -> Self {
        Self::new()
    }
}