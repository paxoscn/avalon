use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use crate::domain::value_objects::{FlowId, TenantId, UserId, FlowName, FlowDefinition, Version, SessionId, FlowExecutionId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlowStatus {
    Draft,
    Active,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flow {
    pub id: FlowId,
    pub tenant_id: TenantId,
    pub name: FlowName,
    pub description: Option<String>,
    pub current_version: Version,
    pub status: FlowStatus,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowVersion {
    pub id: FlowId,
    pub flow_id: FlowId,
    pub version: Version,
    pub definition: FlowDefinition,
    pub change_log: Option<String>,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
}

impl Flow {
    pub fn new(
        tenant_id: TenantId,
        name: FlowName,
        description: Option<String>,
        created_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: FlowId::new(),
            tenant_id,
            name,
            description,
            current_version: Version::new(),
            status: FlowStatus::Draft,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_name(&mut self, name: FlowName) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) -> Result<(), String> {
        match self.status {
            FlowStatus::Draft => {
                self.status = FlowStatus::Active;
                self.updated_at = Utc::now();
                Ok(())
            }
            FlowStatus::Active => Err("Flow is already active".to_string()),
            FlowStatus::Archived => Err("Cannot activate archived flow".to_string()),
        }
    }

    pub fn archive(&mut self) -> Result<(), String> {
        match self.status {
            FlowStatus::Active | FlowStatus::Draft => {
                self.status = FlowStatus::Archived;
                self.updated_at = Utc::now();
                Ok(())
            }
            FlowStatus::Archived => Err("Flow is already archived".to_string()),
        }
    }

    pub fn increment_version(&mut self) {
        self.current_version = self.current_version.next();
        self.updated_at = Utc::now();
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, FlowStatus::Active)
    }

    pub fn is_draft(&self) -> bool {
        matches!(self.status, FlowStatus::Draft)
    }

    pub fn is_archived(&self) -> bool {
        matches!(self.status, FlowStatus::Archived)
    }

    pub fn belongs_to_tenant(&self, tenant_id: &TenantId) -> bool {
        &self.tenant_id == tenant_id
    }

    pub fn can_be_executed(&self) -> bool {
        self.is_active()
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref desc) = self.description {
            if desc.len() > 1000 {
                return Err("Description cannot exceed 1000 characters".to_string());
            }
        }
        Ok(())
    }
}

impl FlowVersion {
    pub fn new(
        flow_id: FlowId,
        version: Version,
        definition: FlowDefinition,
        change_log: Option<String>,
        created_by: UserId,
    ) -> Result<Self, String> {
        // Validate the flow definition
        definition.validate()?;

        Ok(Self {
            id: FlowId::new(),
            flow_id,
            version,
            definition,
            change_log,
            created_by,
            created_at: Utc::now(),
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        self.definition.validate()?;
        
        if let Some(ref log) = self.change_log {
            if log.len() > 2000 {
                return Err("Change log cannot exceed 2000 characters".to_string());
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlowExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowExecution {
    pub id: FlowExecutionId,
    pub flow_id: FlowId,
    pub flow_version: Version,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub session_id: Option<SessionId>,
    pub status: FlowExecutionStatus,
    pub input_data: Option<Value>,
    pub output_data: Option<Value>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
}

impl FlowExecution {
    pub fn new(
        flow_id: FlowId,
        flow_version: Version,
        tenant_id: TenantId,
        user_id: UserId,
        session_id: Option<SessionId>,
        input_data: Option<Value>,
    ) -> Self {
        Self {
            id: FlowExecutionId::new(),
            flow_id,
            flow_version,
            tenant_id,
            user_id,
            session_id,
            status: FlowExecutionStatus::Pending,
            input_data,
            output_data: None,
            error_message: None,
            started_at: Utc::now(),
            completed_at: None,
            execution_time_ms: None,
        }
    }

    pub fn start(&mut self) {
        self.status = FlowExecutionStatus::Running;
        self.started_at = Utc::now();
    }

    pub fn complete(&mut self, output_data: Value) {
        self.status = FlowExecutionStatus::Completed;
        self.output_data = Some(output_data);
        self.completed_at = Some(Utc::now());
        self.calculate_execution_time();
    }

    pub fn fail(&mut self, error_message: String) {
        self.status = FlowExecutionStatus::Failed;
        self.error_message = Some(error_message);
        self.completed_at = Some(Utc::now());
        self.calculate_execution_time();
    }

    pub fn cancel(&mut self) {
        self.status = FlowExecutionStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.calculate_execution_time();
    }

    fn calculate_execution_time(&mut self) {
        if let Some(completed_at) = self.completed_at {
            let duration = completed_at.signed_duration_since(self.started_at);
            self.execution_time_ms = Some(duration.num_milliseconds() as i32);
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, FlowExecutionStatus::Running)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.status, FlowExecutionStatus::Completed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status, FlowExecutionStatus::Failed)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            FlowExecutionStatus::Completed | FlowExecutionStatus::Failed | FlowExecutionStatus::Cancelled
        )
    }

    pub fn belongs_to_tenant(&self, tenant_id: &TenantId) -> bool {
        &self.tenant_id == tenant_id
    }

    pub fn belongs_to_user(&self, user_id: &UserId) -> bool {
        &self.user_id == user_id
    }
}