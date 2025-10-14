use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{DomainEvent, EventMetadata};
use crate::domain::entities::{AuditAction, ResourceType};

/// Event emitted when an audit log is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogCreated {
    pub metadata: EventMetadata,
    pub audit_log_id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub resource_type: ResourceType,
    pub resource_id: Option<Uuid>,
    pub details: Option<Value>,
}

impl AuditLogCreated {
    pub fn new(
        audit_log_id: Uuid,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        action: AuditAction,
        resource_type: ResourceType,
        resource_id: Option<Uuid>,
        details: Option<Value>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(1),
            audit_log_id,
            tenant_id,
            user_id,
            action,
            resource_type,
            resource_id,
            details,
        }
    }
}

impl DomainEvent for AuditLogCreated {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "audit_log.created"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_id(&self) -> Uuid {
        self.audit_log_id
    }

    fn version(&self) -> i64 {
        self.metadata.version
    }
}

/// Event emitted when a flow execution starts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowExecutionStarted {
    pub metadata: EventMetadata,
    pub execution_id: Uuid,
    pub flow_id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub input_data: Option<Value>,
}

impl FlowExecutionStarted {
    pub fn new(
        execution_id: Uuid,
        flow_id: Uuid,
        tenant_id: Uuid,
        user_id: Uuid,
        input_data: Option<Value>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(1),
            execution_id,
            flow_id,
            tenant_id,
            user_id,
            input_data,
        }
    }
}

impl DomainEvent for FlowExecutionStarted {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "flow_execution.started"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_id(&self) -> Uuid {
        self.execution_id
    }

    fn version(&self) -> i64 {
        self.metadata.version
    }
}

/// Event emitted when a flow execution completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowExecutionCompleted {
    pub metadata: EventMetadata,
    pub execution_id: Uuid,
    pub flow_id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub output_data: Option<Value>,
    pub execution_time_ms: i32,
}

impl FlowExecutionCompleted {
    pub fn new(
        execution_id: Uuid,
        flow_id: Uuid,
        tenant_id: Uuid,
        user_id: Uuid,
        output_data: Option<Value>,
        execution_time_ms: i32,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(1),
            execution_id,
            flow_id,
            tenant_id,
            user_id,
            output_data,
            execution_time_ms,
        }
    }
}

impl DomainEvent for FlowExecutionCompleted {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "flow_execution.completed"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_id(&self) -> Uuid {
        self.execution_id
    }

    fn version(&self) -> i64 {
        self.metadata.version
    }
}

/// Event emitted when a flow execution fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowExecutionFailed {
    pub metadata: EventMetadata,
    pub execution_id: Uuid,
    pub flow_id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub error_message: String,
    pub execution_time_ms: i32,
}

impl FlowExecutionFailed {
    pub fn new(
        execution_id: Uuid,
        flow_id: Uuid,
        tenant_id: Uuid,
        user_id: Uuid,
        error_message: String,
        execution_time_ms: i32,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(1),
            execution_id,
            flow_id,
            tenant_id,
            user_id,
            error_message,
            execution_time_ms,
        }
    }
}

impl DomainEvent for FlowExecutionFailed {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "flow_execution.failed"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_id(&self) -> Uuid {
        self.execution_id
    }

    fn version(&self) -> i64 {
        self.metadata.version
    }
}
