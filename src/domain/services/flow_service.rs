use async_trait::async_trait;
use crate::domain::entities::{Flow, User};
use crate::domain::value_objects::{FlowId, TenantId, UserId, FlowDefinition};
use crate::error::Result;
use serde_json::Value;

/// Validation result for flow operations
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Execution context for flow execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub flow_id: FlowId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub variables: Value,
    pub metadata: Value,
}

impl ExecutionContext {
    pub fn new(flow_id: FlowId, tenant_id: TenantId, user_id: UserId) -> Self {
        Self {
            flow_id,
            tenant_id,
            user_id,
            variables: Value::Object(serde_json::Map::new()),
            metadata: Value::Object(serde_json::Map::new()),
        }
    }

    pub fn with_variables(mut self, variables: Value) -> Self {
        self.variables = variables;
        self
    }

    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Flow domain service interface
#[async_trait]
pub trait FlowDomainService: Send + Sync {
    /// Validate a flow definition
    fn validate_flow(&self, flow: &Flow) -> Result<ValidationResult>;

    /// Validate a flow definition structure
    fn validate_flow_definition(&self, definition: &FlowDefinition) -> Result<ValidationResult>;

    /// Check if a user can execute a flow
    fn can_execute(&self, flow: &Flow, user: &User) -> bool;

    /// Check if a user can modify a flow
    fn can_modify(&self, flow: &Flow, user: &User) -> bool;

    /// Create execution context for a flow
    fn create_execution_context(&self, flow: &Flow, user: &User, variables: Option<Value>) -> ExecutionContext;

    /// Validate execution input
    fn validate_execution_input(&self, flow: &Flow, input: &Value) -> Result<ValidationResult>;

    /// Check if flow can be deleted
    fn can_delete(&self, flow: &Flow, user: &User) -> bool;
}

/// Default implementation of FlowDomainService
pub struct FlowDomainServiceImpl;

impl FlowDomainServiceImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FlowDomainServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FlowDomainService for FlowDomainServiceImpl {
    fn validate_flow(&self, flow: &Flow) -> Result<ValidationResult> {
        let mut result = ValidationResult::valid();

        // Validate flow entity
        if let Err(e) = flow.validate() {
            result.add_error(e);
        }

        // Check flow name
        if flow.name.0.is_empty() {
            result.add_error("Flow name cannot be empty".to_string());
        }

        // Check if flow is in valid state
        if flow.is_archived() {
            result.add_warning("Flow is archived".to_string());
        }

        Ok(result)
    }

    fn validate_flow_definition(&self, definition: &FlowDefinition) -> Result<ValidationResult> {
        let mut result = ValidationResult::valid();

        // Validate definition structure
        if let Err(e) = definition.validate() {
            result.add_error(e);
        }

        // Check for start nodes
        let start_nodes = definition.get_start_nodes();
        if start_nodes.is_empty() {
            result.add_error("Flow must have at least one start node".to_string());
        } else if start_nodes.len() > 1 {
            result.add_warning("Flow has multiple start nodes".to_string());
        }

        // Check for end nodes
        let end_nodes = definition.get_end_nodes();
        if end_nodes.is_empty() {
            result.add_error("Flow must have at least one end node".to_string());
        }

        // Check for orphaned nodes (nodes with no incoming or outgoing edges)
        let connected_nodes: std::collections::HashSet<_> = definition.edges.iter()
            .flat_map(|e| vec![&e.source, &e.target])
            .collect();

        for node in &definition.nodes {
            if !connected_nodes.contains(&node.id) && 
               node.node_type != crate::domain::value_objects::NodeType::Start &&
               node.node_type != crate::domain::value_objects::NodeType::End {
                result.add_warning(format!("Node '{}' is not connected to any other nodes", node.id));
            }
        }

        Ok(result)
    }

    fn can_execute(&self, flow: &Flow, user: &User) -> bool {
        // Check if flow is active
        if !flow.can_be_executed() {
            return false;
        }

        // Check if user belongs to the same tenant
        if !flow.belongs_to_tenant(&user.tenant_id) {
            return false;
        }

        true
    }

    fn can_modify(&self, flow: &Flow, user: &User) -> bool {
        // Check if user belongs to the same tenant
        if !flow.belongs_to_tenant(&user.tenant_id) {
            return false;
        }

        // Check if flow is not archived
        if flow.is_archived() {
            return false;
        }

        true
    }

    fn create_execution_context(&self, flow: &Flow, user: &User, variables: Option<Value>) -> ExecutionContext {
        let mut context = ExecutionContext::new(flow.id, flow.tenant_id, user.id);

        if let Some(vars) = variables {
            context = context.with_variables(vars);
        }

        // Add metadata
        let metadata = serde_json::json!({
            "flow_name": flow.name.0,
            "flow_version": flow.current_version.0,
            "user_id": user.id.to_string(),
            "tenant_id": flow.tenant_id.to_string(),
        });
        context = context.with_metadata(metadata);

        context
    }

    fn validate_execution_input(&self, flow: &Flow, input: &Value) -> Result<ValidationResult> {
        let mut result = ValidationResult::valid();

        // Check if flow can be executed
        if !flow.can_be_executed() {
            result.add_error(format!("Flow is not in executable state: {:?}", flow.status));
        }

        // Validate input is an object
        if !input.is_object() && !input.is_null() {
            result.add_error("Execution input must be a JSON object".to_string());
        }

        Ok(result)
    }

    fn can_delete(&self, flow: &Flow, user: &User) -> bool {
        // Check if user belongs to the same tenant
        if !flow.belongs_to_tenant(&user.tenant_id) {
            return false;
        }

        // Only creator or admin can delete
        // For now, we'll allow deletion if user is in same tenant
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Flow;
    use crate::domain::value_objects::{FlowName, Username};

    fn create_test_user() -> User {
        User {
            id: UserId::new(),
            tenant_id: TenantId::new(),
            username: Username::new("testuser".to_string()).unwrap(),
            nickname: Some("Test User".to_string()),
            password_hash: "hash".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn create_test_flow(tenant_id: TenantId, user_id: UserId) -> Flow {
        Flow::new(
            tenant_id,
            FlowName::new("Test Flow".to_string()).unwrap(),
            Some("Test description".to_string()),
            user_id,
        )
    }

    #[test]
    fn test_validate_flow() {
        let service = FlowDomainServiceImpl::new();
        let user = create_test_user();
        let flow = create_test_flow(user.tenant_id, user.id);

        let result = service.validate_flow(&flow).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_can_execute_active_flow() {
        let service = FlowDomainServiceImpl::new();
        let user = create_test_user();
        let mut flow = create_test_flow(user.tenant_id, user.id);
        flow.activate().unwrap();

        assert!(service.can_execute(&flow, &user));
    }

    #[test]
    fn test_cannot_execute_draft_flow() {
        let service = FlowDomainServiceImpl::new();
        let user = create_test_user();
        let flow = create_test_flow(user.tenant_id, user.id);

        assert!(!service.can_execute(&flow, &user));
    }

    #[test]
    fn test_cannot_execute_different_tenant() {
        let service = FlowDomainServiceImpl::new();
        let user = create_test_user();
        let other_tenant = TenantId::new();
        let mut flow = create_test_flow(other_tenant, user.id);
        flow.activate().unwrap();

        assert!(!service.can_execute(&flow, &user));
    }

    #[test]
    fn test_create_execution_context() {
        let service = FlowDomainServiceImpl::new();
        let user = create_test_user();
        let flow = create_test_flow(user.tenant_id, user.id);

        let variables = serde_json::json!({"key": "value"});
        let context = service.create_execution_context(&flow, &user, Some(variables.clone()));

        assert_eq!(context.flow_id, flow.id);
        assert_eq!(context.tenant_id, flow.tenant_id);
        assert_eq!(context.user_id, user.id);
        assert_eq!(context.variables, variables);
    }
}
