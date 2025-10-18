use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QuerySelect, PaginatorTrait, QueryOrder};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use crate::domain::entities::{Flow, FlowVersion, FlowExecution, FlowStatus, FlowExecutionStatus};
use crate::domain::repositories::{FlowRepository, FlowVersionRepository, FlowExecutionRepository};
use crate::domain::value_objects::{FlowId, TenantId, UserId, SessionId, FlowExecutionId, Version, FlowName, FlowDefinition};
use crate::domain::NodeType;
use serde_json::json;
use crate::infrastructure::database::entities;
use crate::error::{Result, PlatformError};

pub struct FlowRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl FlowRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_domain(entity: entities::flow::Model) -> Result<Flow> {
        let name = FlowName::new(entity.name)
            .map_err(|e| PlatformError::ValidationError(e))?;
        
        let status = match entity.status {
            entities::flow::FlowStatus::Draft => FlowStatus::Draft,
            entities::flow::FlowStatus::Active => FlowStatus::Active,
            entities::flow::FlowStatus::Archived => FlowStatus::Archived,
        };

        Ok(Flow {
            id: FlowId::from_uuid(entity.id),
            tenant_id: TenantId::from_uuid(entity.tenant_id),
            name,
            description: entity.description,
            current_version: Version(entity.current_version),
            status,
            created_by: UserId::from_uuid(entity.created_by),
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        })
    }

    fn domain_to_active_model(flow: &Flow) -> entities::flow::ActiveModel {
        use sea_orm::ActiveValue::Set;
        
        let status = match flow.status {
            FlowStatus::Draft => entities::flow::FlowStatus::Draft,
            FlowStatus::Active => entities::flow::FlowStatus::Active,
            FlowStatus::Archived => entities::flow::FlowStatus::Archived,
        };

        entities::flow::ActiveModel {
            id: Set(flow.id.0),
            tenant_id: Set(flow.tenant_id.0),
            name: Set(flow.name.0.clone()),
            description: Set(flow.description.clone()),
            current_version: Set(flow.current_version.0),
            status: Set(status),
            created_by: Set(flow.created_by.0),
            created_at: Set(flow.created_at),
            updated_at: Set(flow.updated_at),
        }
    }
}

#[async_trait]
impl FlowRepository for FlowRepositoryImpl {
    async fn find_by_id(&self, id: &FlowId) -> Result<Option<Flow>> {
        let flow = entities::Flow::find_by_id(id.0)
            .one(self.db.as_ref())
            .await?;

        match flow {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<Flow>> {
        let flows = entities::Flow::find()
            .filter(entities::flow::Column::TenantId.eq(tenant_id.0))
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in flows {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_tenant_and_status(&self, tenant_id: &TenantId, status: &FlowStatus) -> Result<Vec<Flow>> {
        let db_status = match status {
            FlowStatus::Draft => entities::flow::FlowStatus::Draft,
            FlowStatus::Active => entities::flow::FlowStatus::Active,
            FlowStatus::Archived => entities::flow::FlowStatus::Archived,
        };

        let flows = entities::Flow::find()
            .filter(entities::flow::Column::TenantId.eq(tenant_id.0))
            .filter(entities::flow::Column::Status.eq(db_status))
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in flows {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_creator(&self, created_by: &UserId) -> Result<Vec<Flow>> {
        let flows = entities::Flow::find()
            .filter(entities::flow::Column::CreatedBy.eq(created_by.0))
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in flows {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn save(&self, flow: &Flow) -> Result<()> {
        let active_model = Self::domain_to_active_model(flow);
        
        // Check if flow exists
        let existing = entities::Flow::find_by_id(flow.id.0)
            .one(self.db.as_ref())
            .await?;

        if existing.is_some() {
            // Update existing flow
            entities::Flow::update(active_model)
                .exec(self.db.as_ref())
                .await?;
        } else {
            // Insert new flow
            entities::Flow::insert(active_model)
                .exec(self.db.as_ref())
                .await?;
        }

        Ok(())
    }

    async fn delete(&self, id: &FlowId) -> Result<()> {
        entities::Flow::delete_by_id(id.0)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn count_by_tenant(&self, tenant_id: &TenantId) -> Result<u64> {
        let count = entities::Flow::find()
            .filter(entities::flow::Column::TenantId.eq(tenant_id.0))
            .count(self.db.as_ref())
            .await?;

        Ok(count)
    }

    async fn find_by_tenant_paginated(
        &self, 
        tenant_id: &TenantId, 
        offset: u64, 
        limit: u64
    ) -> Result<Vec<Flow>> {
        let flows = entities::Flow::find()
            .filter(entities::flow::Column::TenantId.eq(tenant_id.0))
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in flows {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn name_exists_in_tenant(&self, tenant_id: &TenantId, name: &str) -> Result<bool> {
        let count = entities::Flow::find()
            .filter(entities::flow::Column::TenantId.eq(tenant_id.0))
            .filter(entities::flow::Column::Name.eq(name))
            .count(self.db.as_ref())
            .await?;

        Ok(count > 0)
    }
}

pub struct FlowVersionRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl FlowVersionRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_domain(entity: entities::flow_version::Model) -> Result<FlowVersion> {
        let definition: FlowDefinition = serde_json::from_value(entity.definition.clone())
            .map_err(|e| PlatformError::ValidationError(format!("Invalid flow definition: {}", e)))?;

        Ok(FlowVersion {
            id: FlowId::from_uuid(entity.id),
            flow_id: FlowId::from_uuid(entity.flow_id),
            version: Version(entity.version),
            definition,
            change_log: entity.change_log,
            created_by: UserId::from_uuid(entity.created_by),
            created_at: entity.created_at,
        })
    }

    fn domain_to_active_model(version: &FlowVersion) -> Result<entities::flow_version::ActiveModel> {
        use sea_orm::ActiveValue::Set;
        
        let definition_json = serde_json::to_value(&version.definition)
            .map_err(|e| PlatformError::ValidationError(format!("Failed to serialize flow definition: {}", e)))?;

        Ok(entities::flow_version::ActiveModel {
            id: Set(version.id.0),
            flow_id: Set(version.flow_id.0),
            version: Set(version.version.0),
            definition: Set(definition_json),
            change_log: Set(version.change_log.clone()),
            created_by: Set(version.created_by.0),
            created_at: Set(version.created_at),
        })
    }
}

#[async_trait]
impl FlowVersionRepository for FlowVersionRepositoryImpl {
    async fn find_by_id(&self, id: &FlowId) -> Result<Option<FlowVersion>> {
        let version = entities::FlowVersion::find_by_id(id.0)
            .one(self.db.as_ref())
            .await?;

        match version {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_flow_and_version(&self, flow_id: &FlowId, version: &Version) -> Result<Option<FlowVersion>> {
        let flow_version = entities::FlowVersion::find()
            .filter(entities::flow_version::Column::FlowId.eq(flow_id.0))
            .filter(entities::flow_version::Column::Version.eq(version.0))
            .one(self.db.as_ref())
            .await?;

        match flow_version {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_flow(&self, flow_id: &FlowId) -> Result<Vec<FlowVersion>> {
        let versions = entities::FlowVersion::find()
            .filter(entities::flow_version::Column::FlowId.eq(flow_id.0))
            .order_by_desc(entities::flow_version::Column::Version)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in versions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_latest_by_flow(&self, flow_id: &FlowId) -> Result<Option<FlowVersion>> {
        let version = entities::FlowVersion::find()
            .filter(entities::flow_version::Column::FlowId.eq(flow_id.0))
            .order_by_desc(entities::flow_version::Column::Version)
            .one(self.db.as_ref())
            .await?;

        match version {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, version: &FlowVersion, tenant_id: &TenantId) -> Result<()> {
        let mut version = version.clone();
        
        for node in &mut version.definition.workflow.graph.nodes {
            if node.node_type == NodeType::Llm {
                let config_data: &mut serde_json::Value = node.data.get_mut("model").ok_or_else(|| {
                    crate::error::PlatformError::ValidationError(
                        "LLM node missing 'model' field".to_string(),
                    )
                })?;

                let model_name = config_data
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("qwen-plus-latest");

                let model_provider = config_data
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or("openai")
                    .replace("langgenius/tongyi/tongyi", "openai");

                let llm_configs = entities::llm_config::Entity::find()
                    .filter(entities::llm_config::Column::TenantId.eq(tenant_id.0))
                    .filter(entities::llm_config::Column::Provider.eq(model_provider.clone()))
                    .order_by_asc(entities::llm_config::Column::Name)
                    .all(self.db.as_ref())
                    .await
                    .map_err(PlatformError::DatabaseError)?;

                // Find the first config that matches model_name
                let matching_llm_config = llm_configs.iter().find(|llm_config| {
                    llm_config.config.get("model_name").and_then(|v| v.as_str()).unwrap_or("openai") == model_name
                });

                if let Some(matching_llm_config) = matching_llm_config {
                    config_data["provider"] = json!(model_provider);
                    config_data["llm_config_id"] = json!(matching_llm_config.id);
                } else {
                    return Err(
                        crate::error::PlatformError::ValidationError(
                            format!(
                                "No matching model config matching provider '{}' and name '{}' found in database and failed to parse",
                                model_provider.clone(), model_name,
                            )
                        )
                    )
                }
            }
        }

        let active_model = Self::domain_to_active_model(&version)?;
        
        // Check if version exists
        let existing = entities::FlowVersion::find_by_id(version.id.0)
            .one(self.db.as_ref())
            .await?;

        if existing.is_some() {
            // Update existing version
            entities::FlowVersion::update(active_model)
                .exec(self.db.as_ref())
                .await?;
        } else {
            // Insert new version
            entities::FlowVersion::insert(active_model)
                .exec(self.db.as_ref())
                .await?;
        }

        Ok(())
    }

    async fn delete(&self, id: &FlowId) -> Result<()> {
        entities::FlowVersion::delete_by_id(id.0)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn delete_by_flow(&self, flow_id: &FlowId) -> Result<()> {
        entities::FlowVersion::delete_many()
            .filter(entities::flow_version::Column::FlowId.eq(flow_id.0))
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn count_by_flow(&self, flow_id: &FlowId) -> Result<u64> {
        let count = entities::FlowVersion::find()
            .filter(entities::flow_version::Column::FlowId.eq(flow_id.0))
            .count(self.db.as_ref())
            .await?;

        Ok(count)
    }

    async fn find_by_flow_paginated(
        &self, 
        flow_id: &FlowId, 
        offset: u64, 
        limit: u64
    ) -> Result<Vec<FlowVersion>> {
        let versions = entities::FlowVersion::find()
            .filter(entities::flow_version::Column::FlowId.eq(flow_id.0))
            .order_by_desc(entities::flow_version::Column::Version)
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in versions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }
}
pub struct
 FlowExecutionRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

impl FlowExecutionRepositoryImpl {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn entity_to_domain(entity: entities::flow_execution::Model) -> Result<FlowExecution> {
        let status = match entity.status {
            entities::flow_execution::ExecutionStatus::Pending => FlowExecutionStatus::Pending,
            entities::flow_execution::ExecutionStatus::Running => FlowExecutionStatus::Running,
            entities::flow_execution::ExecutionStatus::Completed => FlowExecutionStatus::Completed,
            entities::flow_execution::ExecutionStatus::Failed => FlowExecutionStatus::Failed,
            entities::flow_execution::ExecutionStatus::Cancelled => FlowExecutionStatus::Cancelled,
        };

        Ok(FlowExecution {
            id: FlowExecutionId::from_uuid(entity.id),
            flow_id: FlowId::from_uuid(entity.flow_id),
            flow_version: Version(entity.flow_version),
            tenant_id: TenantId::from_uuid(entity.tenant_id),
            user_id: UserId::from_uuid(entity.user_id),
            session_id: entity.session_id.map(SessionId::from_uuid),
            status,
            input_data: entity.input_data,
            output_data: entity.output_data,
            error_message: entity.error_message,
            started_at: entity.started_at,
            completed_at: entity.completed_at,
            execution_time_ms: entity.execution_time_ms,
        })
    }

    fn domain_to_active_model(execution: &FlowExecution) -> entities::flow_execution::ActiveModel {
        use sea_orm::ActiveValue::Set;
        
        let status = match execution.status {
            FlowExecutionStatus::Pending => entities::flow_execution::ExecutionStatus::Pending,
            FlowExecutionStatus::Running => entities::flow_execution::ExecutionStatus::Running,
            FlowExecutionStatus::Completed => entities::flow_execution::ExecutionStatus::Completed,
            FlowExecutionStatus::Failed => entities::flow_execution::ExecutionStatus::Failed,
            FlowExecutionStatus::Cancelled => entities::flow_execution::ExecutionStatus::Cancelled,
        };

        entities::flow_execution::ActiveModel {
            id: Set(execution.id.0),
            flow_id: Set(execution.flow_id.0),
            flow_version: Set(execution.flow_version.0),
            tenant_id: Set(execution.tenant_id.0),
            user_id: Set(execution.user_id.0),
            session_id: Set(execution.session_id.map(|s| s.0)),
            status: Set(status),
            input_data: Set(execution.input_data.clone()),
            output_data: Set(execution.output_data.clone()),
            error_message: Set(execution.error_message.clone()),
            started_at: Set(execution.started_at),
            completed_at: Set(execution.completed_at),
            execution_time_ms: Set(execution.execution_time_ms),
        }
    }
}

#[async_trait]
impl FlowExecutionRepository for FlowExecutionRepositoryImpl {
    async fn find_by_id(&self, id: &FlowExecutionId) -> Result<Option<FlowExecution>> {
        let execution = entities::FlowExecution::find_by_id(id.0)
            .one(self.db.as_ref())
            .await?;

        match execution {
            Some(entity) => Ok(Some(Self::entity_to_domain(entity)?)),
            None => Ok(None),
        }
    }

    async fn find_by_flow(&self, flow_id: &FlowId) -> Result<Vec<FlowExecution>> {
        let executions = entities::FlowExecution::find()
            .filter(entities::flow_execution::Column::FlowId.eq(flow_id.0))
            .order_by_desc(entities::flow_execution::Column::StartedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in executions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<FlowExecution>> {
        let executions = entities::FlowExecution::find()
            .filter(entities::flow_execution::Column::TenantId.eq(tenant_id.0))
            .order_by_desc(entities::flow_execution::Column::StartedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in executions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<FlowExecution>> {
        let executions = entities::FlowExecution::find()
            .filter(entities::flow_execution::Column::UserId.eq(user_id.0))
            .order_by_desc(entities::flow_execution::Column::StartedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in executions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_session(&self, session_id: &SessionId) -> Result<Vec<FlowExecution>> {
        let executions = entities::FlowExecution::find()
            .filter(entities::flow_execution::Column::SessionId.eq(session_id.0))
            .order_by_desc(entities::flow_execution::Column::StartedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in executions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_status(&self, tenant_id: &TenantId, status: &FlowExecutionStatus) -> Result<Vec<FlowExecution>> {
        let db_status = match status {
            FlowExecutionStatus::Pending => entities::flow_execution::ExecutionStatus::Pending,
            FlowExecutionStatus::Running => entities::flow_execution::ExecutionStatus::Running,
            FlowExecutionStatus::Completed => entities::flow_execution::ExecutionStatus::Completed,
            FlowExecutionStatus::Failed => entities::flow_execution::ExecutionStatus::Failed,
            FlowExecutionStatus::Cancelled => entities::flow_execution::ExecutionStatus::Cancelled,
        };

        let executions = entities::FlowExecution::find()
            .filter(entities::flow_execution::Column::TenantId.eq(tenant_id.0))
            .filter(entities::flow_execution::Column::Status.eq(db_status))
            .order_by_desc(entities::flow_execution::Column::StartedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in executions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_by_time_range(
        &self,
        tenant_id: &TenantId,
        start: DateTime<Utc>,
        end: DateTime<Utc>
    ) -> Result<Vec<FlowExecution>> {
        let executions = entities::FlowExecution::find()
            .filter(entities::flow_execution::Column::TenantId.eq(tenant_id.0))
            .filter(entities::flow_execution::Column::StartedAt.gte(start))
            .filter(entities::flow_execution::Column::StartedAt.lte(end))
            .order_by_desc(entities::flow_execution::Column::StartedAt)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in executions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn save(&self, execution: &FlowExecution) -> Result<()> {
        let active_model = Self::domain_to_active_model(execution);
        
        // Check if execution exists
        let existing = entities::FlowExecution::find_by_id(execution.id.0)
            .one(self.db.as_ref())
            .await?;

        if existing.is_some() {
            // Update existing execution
            entities::FlowExecution::update(active_model)
                .exec(self.db.as_ref())
                .await?;
        } else {
            // Insert new execution
            entities::FlowExecution::insert(active_model)
                .exec(self.db.as_ref())
                .await?;
        }

        Ok(())
    }

    async fn delete(&self, id: &FlowExecutionId) -> Result<()> {
        entities::FlowExecution::delete_by_id(id.0)
            .exec(self.db.as_ref())
            .await?;
        Ok(())
    }

    async fn count_by_flow(&self, flow_id: &FlowId) -> Result<u64> {
        let count = entities::FlowExecution::find()
            .filter(entities::flow_execution::Column::FlowId.eq(flow_id.0))
            .count(self.db.as_ref())
            .await?;

        Ok(count)
    }

    async fn count_by_tenant(&self, tenant_id: &TenantId) -> Result<u64> {
        let count = entities::FlowExecution::find()
            .filter(entities::flow_execution::Column::TenantId.eq(tenant_id.0))
            .count(self.db.as_ref())
            .await?;

        Ok(count)
    }

    async fn find_by_tenant_paginated(
        &self,
        tenant_id: &TenantId,
        offset: u64,
        limit: u64
    ) -> Result<Vec<FlowExecution>> {
        let executions = entities::FlowExecution::find()
            .filter(entities::flow_execution::Column::TenantId.eq(tenant_id.0))
            .order_by_desc(entities::flow_execution::Column::StartedAt)
            .offset(offset)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in executions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_recent_by_flow(
        &self,
        flow_id: &FlowId,
        limit: u64
    ) -> Result<Vec<FlowExecution>> {
        let executions = entities::FlowExecution::find()
            .filter(entities::flow_execution::Column::FlowId.eq(flow_id.0))
            .order_by_desc(entities::flow_execution::Column::StartedAt)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in executions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }

    async fn find_failed_executions(
        &self,
        tenant_id: &TenantId,
        limit: u64
    ) -> Result<Vec<FlowExecution>> {
        let executions = entities::FlowExecution::find()
            .filter(entities::flow_execution::Column::TenantId.eq(tenant_id.0))
            .filter(entities::flow_execution::Column::Status.eq(entities::flow_execution::ExecutionStatus::Failed))
            .order_by_desc(entities::flow_execution::Column::StartedAt)
            .limit(limit)
            .all(self.db.as_ref())
            .await?;

        let mut result = Vec::new();
        for entity in executions {
            result.push(Self::entity_to_domain(entity)?);
        }
        Ok(result)
    }
}
