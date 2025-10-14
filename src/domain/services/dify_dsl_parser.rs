use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::domain::value_objects::{FlowDefinition, FlowNode, FlowEdge, FlowVariable, FlowMetadata, NodeType, VariableType, NodePosition};
use crate::error::Result;

/// Dify DSL structure (simplified version based on Dify's workflow format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifyDSL {
    pub version: String,
    pub kind: String,
    pub nodes: Vec<DifyNode>,
    pub edges: Vec<DifyEdge>,
    #[serde(default)]
    pub variables: Vec<DifyVariable>,
    #[serde(default)]
    pub metadata: DifyMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifyNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub title: String,
    pub data: Value,
    #[serde(default)]
    pub position: DifyPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifyEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "sourceHandle")]
    pub source_handle: Option<String>,
    #[serde(rename = "targetHandle")]
    pub target_handle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifyVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub variable_type: String,
    #[serde(rename = "defaultValue")]
    pub default_value: Option<Value>,
    #[serde(default)]
    pub required: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DifyMetadata {
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DifyPosition {
    pub x: f64,
    pub y: f64,
}

/// Dify DSL Parser
pub struct DifyDSLParser;

impl DifyDSLParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse Dify DSL from JSON string
    pub fn parse(&self, dsl_json: &str) -> Result<FlowDefinition> {
        let dify_dsl: DifyDSL = serde_json::from_str(dsl_json)
            .map_err(|e| crate::error::PlatformError::ValidationError(
                format!("Failed to parse Dify DSL: {}", e)
            ))?;

        self.convert_to_flow_definition(dify_dsl)
    }

    /// Convert Dify DSL to internal FlowDefinition
    fn convert_to_flow_definition(&self, dsl: DifyDSL) -> Result<FlowDefinition> {
        // Validate DSL version
        self.validate_dsl_version(&dsl.version)?;

        // Convert nodes
        let nodes = dsl.nodes.into_iter()
            .map(|n| self.convert_node(n))
            .collect::<Result<Vec<_>>>()?;

        // Convert edges
        let edges = dsl.edges.into_iter()
            .map(|e| self.convert_edge(e))
            .collect();

        // Convert variables
        let variables = dsl.variables.into_iter()
            .map(|v| self.convert_variable(v))
            .collect::<Result<Vec<_>>>()?;

        // Convert metadata
        let metadata = self.convert_metadata(dsl.metadata);

        let definition = FlowDefinition {
            nodes,
            edges,
            variables,
            metadata,
        };

        // Validate the converted definition
        definition.validate()
            .map_err(|e| crate::error::PlatformError::ValidationError(e))?;

        Ok(definition)
    }

    fn validate_dsl_version(&self, version: &str) -> Result<()> {
        // Support versions 1.x and 2.x
        if !version.starts_with("1.") && !version.starts_with("2.") {
            return Err(crate::error::PlatformError::ValidationError(
                format!("Unsupported DSL version: {}. Supported versions: 1.x, 2.x", version)
            ));
        }
        Ok(())
    }

    fn convert_node(&self, node: DifyNode) -> Result<FlowNode> {
        let node_type = self.map_node_type(&node.node_type)?;
        
        Ok(FlowNode {
            id: node.id,
            node_type,
            title: node.title,
            data: node.data,
            position: NodePosition {
                x: node.position.x,
                y: node.position.y,
            },
        })
    }

    fn convert_edge(&self, edge: DifyEdge) -> FlowEdge {
        FlowEdge {
            id: edge.id,
            source: edge.source,
            target: edge.target,
            source_handle: edge.source_handle,
            target_handle: edge.target_handle,
        }
    }

    fn convert_variable(&self, var: DifyVariable) -> Result<FlowVariable> {
        let variable_type = self.map_variable_type(&var.variable_type)?;
        
        Ok(FlowVariable {
            name: var.name,
            variable_type,
            default_value: var.default_value,
            required: var.required,
            description: var.description,
        })
    }

    fn convert_metadata(&self, metadata: DifyMetadata) -> FlowMetadata {
        FlowMetadata {
            description: metadata.description,
            tags: metadata.tags,
            author: metadata.author.unwrap_or_else(|| "Unknown".to_string()),
            version: "1.0.0".to_string(),
        }
    }

    fn map_node_type(&self, dify_type: &str) -> Result<NodeType> {
        let node_type = match dify_type.to_lowercase().as_str() {
            "start" => NodeType::Start,
            "end" => NodeType::End,
            "llm" | "llm-chat" | "llm_chat" => NodeType::LlmChat,
            "knowledge-retrieval" | "knowledge_retrieval" | "vector-search" | "vector_search" => NodeType::VectorSearch,
            "tool" | "mcp-tool" | "mcp_tool" => NodeType::McpTool,
            "if-else" | "if_else" | "condition" => NodeType::Condition,
            "loop" | "iteration" => NodeType::Loop,
            "variable" | "variable-assigner" | "variable_assigner" => NodeType::Variable,
            "http-request" | "http_request" | "http" => NodeType::HttpRequest,
            "code" | "code-executor" | "code_executor" => NodeType::Code,
            _ => {
                return Err(crate::error::PlatformError::ValidationError(
                    format!("Unknown node type: {}", dify_type)
                ));
            }
        };
        Ok(node_type)
    }

    fn map_variable_type(&self, dify_type: &str) -> Result<VariableType> {
        let var_type = match dify_type.to_lowercase().as_str() {
            "string" | "text" => VariableType::String,
            "number" | "integer" | "float" => VariableType::Number,
            "boolean" | "bool" => VariableType::Boolean,
            "array" | "list" => VariableType::Array,
            "object" | "dict" | "map" => VariableType::Object,
            _ => {
                return Err(crate::error::PlatformError::ValidationError(
                    format!("Unknown variable type: {}", dify_type)
                ));
            }
        };
        Ok(var_type)
    }

    /// Validate Dify DSL without converting
    pub fn validate(&self, dsl_json: &str) -> Result<Vec<String>> {
        let dify_dsl: DifyDSL = serde_json::from_str(dsl_json)
            .map_err(|e| crate::error::PlatformError::ValidationError(
                format!("Failed to parse Dify DSL: {}", e)
            ))?;

        let mut warnings = Vec::new();

        // Check version
        if let Err(e) = self.validate_dsl_version(&dify_dsl.version) {
            return Err(e);
        }

        // Check for empty nodes
        if dify_dsl.nodes.is_empty() {
            warnings.push("DSL has no nodes".to_string());
        }

        // Check for start and end nodes
        let has_start = dify_dsl.nodes.iter().any(|n| n.node_type.to_lowercase() == "start");
        let has_end = dify_dsl.nodes.iter().any(|n| n.node_type.to_lowercase() == "end");

        if !has_start {
            warnings.push("DSL has no start node".to_string());
        }
        if !has_end {
            warnings.push("DSL has no end node".to_string());
        }

        // Check for duplicate node IDs
        let mut node_ids = std::collections::HashSet::new();
        for node in &dify_dsl.nodes {
            if !node_ids.insert(&node.id) {
                warnings.push(format!("Duplicate node ID: {}", node.id));
            }
        }

        // Check edges reference existing nodes
        for edge in &dify_dsl.edges {
            if !node_ids.contains(&edge.source) {
                warnings.push(format!("Edge references non-existent source node: {}", edge.source));
            }
            if !node_ids.contains(&edge.target) {
                warnings.push(format!("Edge references non-existent target node: {}", edge.target));
            }
        }

        Ok(warnings)
    }
}

impl Default for DifyDSLParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_dsl() {
        let dsl_json = r#"{
            "version": "1.0",
            "kind": "workflow",
            "nodes": [
                {
                    "id": "start",
                    "type": "start",
                    "title": "Start",
                    "data": {},
                    "position": {"x": 0, "y": 0}
                },
                {
                    "id": "end",
                    "type": "end",
                    "title": "End",
                    "data": {},
                    "position": {"x": 200, "y": 0}
                }
            ],
            "edges": [
                {
                    "id": "e1",
                    "source": "start",
                    "target": "end"
                }
            ],
            "variables": [],
            "metadata": {
                "description": "Simple flow",
                "tags": ["test"],
                "author": "Test Author"
            }
        }"#;

        let parser = DifyDSLParser::new();
        let result = parser.parse(dsl_json);
        
        assert!(result.is_ok());
        let definition = result.unwrap();
        assert_eq!(definition.nodes.len(), 2);
        assert_eq!(definition.edges.len(), 1);
    }

    #[test]
    fn test_parse_with_llm_node() {
        let dsl_json = r#"{
            "version": "2.0",
            "kind": "workflow",
            "nodes": [
                {
                    "id": "start",
                    "type": "start",
                    "title": "Start",
                    "data": {},
                    "position": {"x": 0, "y": 0}
                },
                {
                    "id": "llm1",
                    "type": "llm-chat",
                    "title": "LLM Chat",
                    "data": {
                        "model": "gpt-4",
                        "prompt": "Hello"
                    },
                    "position": {"x": 100, "y": 0}
                },
                {
                    "id": "end",
                    "type": "end",
                    "title": "End",
                    "data": {},
                    "position": {"x": 200, "y": 0}
                }
            ],
            "edges": [
                {
                    "id": "e1",
                    "source": "start",
                    "target": "llm1"
                },
                {
                    "id": "e2",
                    "source": "llm1",
                    "target": "end"
                }
            ]
        }"#;

        let parser = DifyDSLParser::new();
        let result = parser.parse(dsl_json);
        
        assert!(result.is_ok());
        let definition = result.unwrap();
        assert_eq!(definition.nodes.len(), 3);
        assert!(matches!(definition.nodes[1].node_type, NodeType::LlmChat));
    }

    #[test]
    fn test_validate_invalid_version() {
        let dsl_json = r#"{
            "version": "3.0",
            "kind": "workflow",
            "nodes": [],
            "edges": []
        }"#;

        let parser = DifyDSLParser::new();
        let result = parser.parse(dsl_json);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_missing_start_node() {
        let dsl_json = r#"{
            "version": "1.0",
            "kind": "workflow",
            "nodes": [
                {
                    "id": "end",
                    "type": "end",
                    "title": "End",
                    "data": {},
                    "position": {"x": 0, "y": 0}
                }
            ],
            "edges": []
        }"#;

        let parser = DifyDSLParser::new();
        let warnings = parser.validate(dsl_json).unwrap();
        
        assert!(warnings.iter().any(|w| w.contains("no start node")));
    }

    #[test]
    fn test_parse_with_variables() {
        let dsl_json = r#"{
            "version": "1.0",
            "kind": "workflow",
            "nodes": [
                {
                    "id": "start",
                    "type": "start",
                    "title": "Start",
                    "data": {},
                    "position": {"x": 0, "y": 0}
                },
                {
                    "id": "end",
                    "type": "end",
                    "title": "End",
                    "data": {},
                    "position": {"x": 200, "y": 0}
                }
            ],
            "edges": [
                {
                    "id": "e1",
                    "source": "start",
                    "target": "end"
                }
            ],
            "variables": [
                {
                    "name": "input_text",
                    "type": "string",
                    "defaultValue": "Hello",
                    "required": true,
                    "description": "Input text"
                }
            ]
        }"#;

        let parser = DifyDSLParser::new();
        let result = parser.parse(dsl_json);
        
        assert!(result.is_ok());
        let definition = result.unwrap();
        assert_eq!(definition.variables.len(), 1);
        assert_eq!(definition.variables[0].name, "input_text");
    }
}
