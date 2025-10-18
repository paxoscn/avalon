use async_trait::async_trait;
use std::sync::Arc;
use serde_json::Value;
use crate::{
    domain::{
        entities::{Flow, FlowVersion, FlowExecution, User},
        repositories::{FlowRepository, FlowVersionRepository, FlowExecutionRepository},
        services::{FlowDomainService, ExecutionEngine},
        value_objects::{FlowId, TenantId, UserId, FlowName, FlowDefinition, Version, SessionId, FlowExecutionId},
    },
    error::{Result, PlatformError},
};

// Import ValidationResult explicitly to avoid ambiguity
use crate::domain::services::flow_service::ValidationResult;

/// Flow application service trait
#[async_trait]
pub trait FlowApplicationService: Send + Sync {
    /// Create a new flow
    async fn create_flow(
        &self,
        tenant_id: TenantId,
        name: String,
        description: Option<String>,
        user_id: UserId,
    ) -> Result<Flow>;

    /// Get flow by ID
    async fn get_flow(&self, flow_id: FlowId, tenant_id: TenantId) -> Result<Flow>;

    /// List flows for a tenant
    async fn list_flows(&self, tenant_id: TenantId, page: u64, limit: u64) -> Result<(Vec<Flow>, u64)>;

    /// Update flow
    async fn update_flow(
        &self,
        flow_id: FlowId,
        tenant_id: TenantId,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Flow>;

    /// Delete flow
    async fn delete_flow(&self, flow_id: FlowId, tenant_id: TenantId, user_id: UserId) -> Result<()>;

    /// Activate flow
    async fn activate_flow(&self, flow_id: FlowId, tenant_id: TenantId) -> Result<Flow>;

    /// Archive flow
    async fn archive_flow(&self, flow_id: FlowId, tenant_id: TenantId) -> Result<Flow>;

    /// Import flow from Dify DSL
    async fn import_from_dsl(
        &self,
        tenant_id: TenantId,
        name: String,
        dsl: String,
        user_id: UserId,
    ) -> Result<(Flow, ValidationResult)>;

    /// Validate flow definition
    async fn validate_flow_definition(&self, definition: FlowDefinition) -> Result<ValidationResult>;

    /// Execute flow
    async fn execute_flow(
        &self,
        flow_id: FlowId,
        tenant_id: TenantId,
        user_id: UserId,
        session_id: Option<SessionId>,
        input_data: Option<Value>,
    ) -> Result<FlowExecution>;

    /// Get flow execution status
    async fn get_execution_status(&self, execution_id: FlowExecutionId, tenant_id: TenantId) -> Result<FlowExecution>;

    /// List flow executions
    async fn list_executions(
        &self,
        tenant_id: TenantId,
        flow_id: Option<FlowId>,
        page: u64,
        limit: u64,
    ) -> Result<(Vec<FlowExecution>, u64)>;

    /// Create flow version
    async fn create_version(
        &self,
        flow_id: FlowId,
        tenant_id: TenantId,
        definition: FlowDefinition,
        change_log: Option<String>,
        user_id: UserId,
    ) -> Result<FlowVersion>;

    /// Get flow versions
    async fn get_versions(&self, flow_id: FlowId, tenant_id: TenantId) -> Result<Vec<FlowVersion>>;

    /// Rollback to specific version
    async fn rollback_to_version(
        &self,
        flow_id: FlowId,
        tenant_id: TenantId,
        target_version: i32,
        user_id: UserId,
    ) -> Result<Flow>;
}

/// Flow application service implementation
pub struct FlowApplicationServiceImpl {
    flow_repo: Arc<dyn FlowRepository>,
    version_repo: Arc<dyn FlowVersionRepository>,
    execution_repo: Arc<dyn FlowExecutionRepository>,
    flow_domain_service: Arc<dyn FlowDomainService>,
    execution_engine: Option<Arc<dyn ExecutionEngine>>,
}

impl FlowApplicationServiceImpl {
    pub fn new(
        flow_repo: Arc<dyn FlowRepository>,
        version_repo: Arc<dyn FlowVersionRepository>,
        execution_repo: Arc<dyn FlowExecutionRepository>,
        flow_domain_service: Arc<dyn FlowDomainService>,
        execution_engine: Option<Arc<dyn ExecutionEngine>>,
    ) -> Self {
        Self {
            flow_repo,
            version_repo,
            execution_repo,
            flow_domain_service,
            execution_engine,
        }
    }
}

#[async_trait]
impl FlowApplicationService for FlowApplicationServiceImpl {
    async fn create_flow(
        &self,
        tenant_id: TenantId,
        name: String,
        description: Option<String>,
        user_id: UserId,
    ) -> Result<Flow> {
        let flow_name = FlowName::new(name)
            .map_err(|e| PlatformError::ValidationError(e))?;

        // Check if name already exists
        if self.flow_repo.name_exists_in_tenant(&tenant_id, &flow_name.0).await? {
            return Err(PlatformError::ValidationError(
                "Flow name already exists in this tenant".to_string()
            ));
        }

        let flow = Flow::new(tenant_id, flow_name, description, user_id);
        
        // Validate flow
        self.flow_domain_service.validate_flow(&flow)?;
        
        self.flow_repo.save(&flow).await?;
        Ok(flow)
    }

    async fn get_flow(&self, flow_id: FlowId, tenant_id: TenantId) -> Result<Flow> {
        let flow = self.flow_repo.find_by_id(&flow_id).await?
            .ok_or_else(|| PlatformError::NotFound("Flow not found".to_string()))?;

        if !flow.belongs_to_tenant(&tenant_id) {
            return Err(PlatformError::AuthorizationFailed("Access denied".to_string()));
        }

        Ok(flow)
    }

    async fn list_flows(&self, tenant_id: TenantId, page: u64, limit: u64) -> Result<(Vec<Flow>, u64)> {
        let offset = page * limit;
        let flows = self.flow_repo.find_by_tenant_paginated(&tenant_id, offset, limit).await?;
        let total = self.flow_repo.count_by_tenant(&tenant_id).await?;
        Ok((flows, total))
    }

    async fn update_flow(
        &self,
        flow_id: FlowId,
        tenant_id: TenantId,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Flow> {
        let mut flow = self.get_flow(flow_id, tenant_id).await?;

        if let Some(name_str) = name {
            let flow_name = FlowName::new(name_str)
                .map_err(|e| PlatformError::ValidationError(e))?;
            flow.update_name(flow_name);
        }

        if description.is_some() {
            flow.update_description(description);
        }

        self.flow_domain_service.validate_flow(&flow)?;
        self.flow_repo.save(&flow).await?;
        Ok(flow)
    }

    async fn delete_flow(&self, flow_id: FlowId, tenant_id: TenantId, user_id: UserId) -> Result<()> {
        let flow = self.get_flow(flow_id, tenant_id).await?;

        // Create a minimal user for permission check
        let user = User {
            id: user_id,
            tenant_id,
            username: crate::domain::value_objects::Username::new("temp".to_string()).unwrap(),
            nickname: None,
            password_hash: String::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        if !self.flow_domain_service.can_delete(&flow, &user) {
            return Err(PlatformError::AuthorizationFailed("Cannot delete this flow".to_string()));
        }

        // Delete versions first
        self.version_repo.delete_by_flow(&flow_id).await?;
        
        // Delete flow
        self.flow_repo.delete(&flow_id).await?;
        Ok(())
    }

    async fn activate_flow(&self, flow_id: FlowId, tenant_id: TenantId) -> Result<Flow> {
        let mut flow = self.get_flow(flow_id, tenant_id).await?;
        flow.activate()
            .map_err(|e| PlatformError::ValidationError(e))?;
        self.flow_repo.save(&flow).await?;
        Ok(flow)
    }

    async fn archive_flow(&self, flow_id: FlowId, tenant_id: TenantId) -> Result<Flow> {
        let mut flow = self.get_flow(flow_id, tenant_id).await?;
        flow.archive()
            .map_err(|e| PlatformError::ValidationError(e))?;
        self.flow_repo.save(&flow).await?;
        Ok(flow)
    }

    async fn import_from_dsl(
        &self,
        tenant_id: TenantId,
        name: String,
        dsl: String,
        user_id: UserId,
    ) -> Result<(Flow, ValidationResult)> {
        // Parse DSL to FlowDefinition
        let definition = FlowDefinition::from_dsl(&dsl)
            .map_err(|e| PlatformError::DSLParsingFailed(e))?;

        // Validate definition
        let validation = self.flow_domain_service.validate_flow_definition(&definition)?;

        if !validation.is_valid {
            return Err(PlatformError::ValidationError(
                format!("Invalid flow definition: {:?}", validation.errors)
            ));
        }

        // Create flow
        let flow = self.create_flow(tenant_id, name, Some("Imported from Dify DSL".to_string()), user_id).await?;

        // Create initial version with the definition
        let version = FlowVersion::new(
            flow.id,
            Version::initial(),
            definition,
            Some("Initial version from DSL import".to_string()),
            user_id,
        ).map_err(|e| PlatformError::ValidationError(e))?;

        self.version_repo.save(&version, &tenant_id).await?;

        Ok((flow, validation))
    }

    async fn validate_flow_definition(&self, definition: FlowDefinition) -> Result<ValidationResult> {
        self.flow_domain_service.validate_flow_definition(&definition)
    }

    async fn execute_flow(
        &self,
        flow_id: FlowId,
        tenant_id: TenantId,
        user_id: UserId,
        session_id: Option<SessionId>,
        input_data: Option<Value>,
    ) -> Result<FlowExecution> {
        let flow = self.get_flow(flow_id, tenant_id).await?;

        // Create minimal user for permission check
        let user = User {
            id: user_id,
            tenant_id,
            username: crate::domain::value_objects::Username::new("temp".to_string()).unwrap(),
            nickname: None,
            password_hash: String::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        if !self.flow_domain_service.can_execute(&flow, &user) {
            return Err(PlatformError::AuthorizationFailed("Cannot execute this flow".to_string()));
        }

        // Validate input
        if let Some(ref input) = input_data {
            let validation = self.flow_domain_service.validate_execution_input(&flow, input)?;
            if !validation.is_valid {
                return Err(PlatformError::ValidationError(
                    format!("Invalid execution input: {:?}", validation.errors)
                ));
            }
        }

        // Create execution record
        let mut execution = FlowExecution::new(
            flow_id,
            flow.current_version,
            tenant_id,
            user_id,
            session_id,
            input_data.clone(),
        );

        execution.start();
        self.execution_repo.save(&execution).await?;

        // Execute flow if engine is available
        if let Some(ref engine) = self.execution_engine {
            // Get the latest version definition
            let version = self.version_repo.find_latest_by_flow(&flow_id).await?
                .ok_or_else(|| PlatformError::NotFound("Flow version not found".to_string()))?;

            // Prepare initial variables
            let mut initial_variables = std::collections::HashMap::new();
            if let Some(Value::Object(map)) = input_data {
                for (key, value) in map {
                    initial_variables.insert(key, value);
                }
            }

            match engine.execute(&mut execution, &version.definition, initial_variables).await {
                Ok(_state) => {
                    // For now, just mark as completed with a simple status
                    let output = serde_json::json!({"status": "completed"});
                    execution.complete(output);
                }
                Err(e) => {
                    execution.fail(e.to_string());
                }
            }
            
            self.execution_repo.save(&execution).await?;
        }

        Ok(execution)
    }

    async fn get_execution_status(&self, execution_id: FlowExecutionId, tenant_id: TenantId) -> Result<FlowExecution> {
        let execution = self.execution_repo.find_by_id(&execution_id).await?
            .ok_or_else(|| PlatformError::NotFound("Execution not found".to_string()))?;

        if !execution.belongs_to_tenant(&tenant_id) {
            return Err(PlatformError::AuthorizationFailed("Access denied".to_string()));
        }

        Ok(execution)
    }

    async fn list_executions(
        &self,
        tenant_id: TenantId,
        flow_id: Option<FlowId>,
        page: u64,
        limit: u64,
    ) -> Result<(Vec<FlowExecution>, u64)> {
        let offset = page * limit;
        
        let (executions, total) = if let Some(fid) = flow_id {
            let execs = self.execution_repo.find_by_flow(&fid).await?
                .into_iter()
                .filter(|e| e.belongs_to_tenant(&tenant_id))
                .skip(offset as usize)
                .take(limit as usize)
                .collect::<Vec<_>>();
            let count = self.execution_repo.count_by_flow(&fid).await?;
            (execs, count)
        } else {
            let execs = self.execution_repo.find_by_tenant_paginated(&tenant_id, offset, limit).await?;
            let count = self.execution_repo.count_by_tenant(&tenant_id).await?;
            (execs, count)
        };

        Ok((executions, total))
    }

    async fn create_version(
        &self,
        flow_id: FlowId,
        tenant_id: TenantId,
        definition: FlowDefinition,
        change_log: Option<String>,
        user_id: UserId,
    ) -> Result<FlowVersion> {
        let mut flow = self.get_flow(flow_id, tenant_id).await?;

        // Validate definition
        let validation = self.flow_domain_service.validate_flow_definition(&definition)?;
        if !validation.is_valid {
            return Err(PlatformError::ValidationError(
                format!("Invalid flow definition: {:?}", validation.errors)
            ));
        }

        // Increment flow version
        flow.increment_version();
        
        let version = FlowVersion::new(
            flow_id,
            flow.current_version,
            definition,
            change_log,
            user_id,
        ).map_err(|e| PlatformError::ValidationError(e))?;

        self.version_repo.save(&version, &tenant_id).await?;
        self.flow_repo.save(&flow).await?;

        Ok(version)
    }

    async fn get_versions(&self, flow_id: FlowId, tenant_id: TenantId) -> Result<Vec<FlowVersion>> {
        // Verify access
        let _ = self.get_flow(flow_id, tenant_id).await?;
        
        let versions = self.version_repo.find_by_flow(&flow_id).await?;
        Ok(versions)
    }

    async fn rollback_to_version(
        &self,
        flow_id: FlowId,
        tenant_id: TenantId,
        target_version: i32,
        user_id: UserId,
    ) -> Result<Flow> {
        let mut flow = self.get_flow(flow_id, tenant_id).await?;

        // Get target version
        let target_ver = Version(target_version);
        let version = self.version_repo.find_by_flow_and_version(&flow_id, &target_ver).await?
            .ok_or_else(|| PlatformError::NotFound("Version not found".to_string()))?;

        // Create new version with the old definition
        flow.increment_version();
        let new_version = FlowVersion::new(
            flow_id,
            flow.current_version,
            version.definition,
            Some(format!("Rollback to version {}", target_version)),
            user_id,
        ).map_err(|e| PlatformError::ValidationError(e))?;

        self.version_repo.save(&new_version, &tenant_id).await?;
        self.flow_repo.save(&flow).await?;

        Ok(flow)
    }
}
