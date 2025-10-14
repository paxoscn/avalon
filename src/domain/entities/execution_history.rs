use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ExecutionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ExecutionStatus::Pending => "pending",
            ExecutionStatus::Running => "running",
            ExecutionStatus::Completed => "completed",
            ExecutionStatus::Failed => "failed",
            ExecutionStatus::Cancelled => "cancelled",
        }
    }
}

impl From<String> for ExecutionStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => ExecutionStatus::Pending,
            "running" => ExecutionStatus::Running,
            "completed" => ExecutionStatus::Completed,
            "failed" => ExecutionStatus::Failed,
            "cancelled" => ExecutionStatus::Cancelled,
            _ => ExecutionStatus::Pending,
        }
    }
}

/// Flow execution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowExecutionHistory {
    pub id: Uuid,
    pub flow_id: Uuid,
    pub flow_version: i32,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
    pub status: ExecutionStatus,
    pub input_data: Option<Value>,
    pub output_data: Option<Value>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
}

impl FlowExecutionHistory {
    pub fn new(
        flow_id: Uuid,
        flow_version: i32,
        tenant_id: Uuid,
        user_id: Uuid,
        session_id: Option<Uuid>,
        input_data: Option<Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            flow_id,
            flow_version,
            tenant_id,
            user_id,
            session_id,
            status: ExecutionStatus::Pending,
            input_data,
            output_data: None,
            error_message: None,
            started_at: Utc::now(),
            completed_at: None,
            execution_time_ms: None,
        }
    }

    pub fn start(&mut self) {
        self.status = ExecutionStatus::Running;
        self.started_at = Utc::now();
    }

    pub fn complete(&mut self, output_data: Option<Value>) {
        self.status = ExecutionStatus::Completed;
        self.output_data = output_data;
        self.completed_at = Some(Utc::now());
        self.execution_time_ms = Some(
            (self.completed_at.unwrap() - self.started_at)
                .num_milliseconds() as i32,
        );
    }

    pub fn fail(&mut self, error_message: String) {
        self.status = ExecutionStatus::Failed;
        self.error_message = Some(error_message);
        self.completed_at = Some(Utc::now());
        self.execution_time_ms = Some(
            (self.completed_at.unwrap() - self.started_at)
                .num_milliseconds() as i32,
        );
    }

    pub fn cancel(&mut self) {
        self.status = ExecutionStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.execution_time_ms = Some(
            (self.completed_at.unwrap() - self.started_at)
                .num_milliseconds() as i32,
        );
    }
}

/// Execution step status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl StepStatus {
    pub fn as_str(&self) -> &str {
        match self {
            StepStatus::Pending => "pending",
            StepStatus::Running => "running",
            StepStatus::Completed => "completed",
            StepStatus::Failed => "failed",
            StepStatus::Skipped => "skipped",
        }
    }
}

/// Execution step for detailed tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub step_name: String,
    pub step_type: String,
    pub status: StepStatus,
    pub input_data: Option<Value>,
    pub output_data: Option<Value>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
    pub metadata: Option<Value>,
}

impl ExecutionStep {
    pub fn new(execution_id: Uuid, step_name: String, step_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            execution_id,
            step_name,
            step_type,
            status: StepStatus::Pending,
            input_data: None,
            output_data: None,
            error_message: None,
            started_at: Utc::now(),
            completed_at: None,
            execution_time_ms: None,
            metadata: None,
        }
    }

    pub fn with_input(mut self, input_data: Value) -> Self {
        self.input_data = Some(input_data);
        self
    }

    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn start(&mut self) {
        self.status = StepStatus::Running;
        self.started_at = Utc::now();
    }

    pub fn complete(&mut self, output_data: Option<Value>) {
        self.status = StepStatus::Completed;
        self.output_data = output_data;
        self.completed_at = Some(Utc::now());
        self.execution_time_ms = Some(
            (self.completed_at.unwrap() - self.started_at)
                .num_milliseconds() as i32,
        );
    }

    pub fn fail(&mut self, error_message: String) {
        self.status = StepStatus::Failed;
        self.error_message = Some(error_message);
        self.completed_at = Some(Utc::now());
        self.execution_time_ms = Some(
            (self.completed_at.unwrap() - self.started_at)
                .num_milliseconds() as i32,
        );
    }

    pub fn skip(&mut self) {
        self.status = StepStatus::Skipped;
        self.completed_at = Some(Utc::now());
        self.execution_time_ms = Some(
            (self.completed_at.unwrap() - self.started_at)
                .num_milliseconds() as i32,
        );
    }
}

/// Performance metrics for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub execution_id: Uuid,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub failed_steps: u32,
    pub skipped_steps: u32,
    pub total_execution_time_ms: i32,
    pub average_step_time_ms: i32,
    pub slowest_step: Option<String>,
    pub slowest_step_time_ms: Option<i32>,
}

impl ExecutionMetrics {
    pub fn from_steps(execution_id: Uuid, steps: &[ExecutionStep]) -> Self {
        let total_steps = steps.len() as u32;
        let completed_steps = steps
            .iter()
            .filter(|s| s.status == StepStatus::Completed)
            .count() as u32;
        let failed_steps = steps
            .iter()
            .filter(|s| s.status == StepStatus::Failed)
            .count() as u32;
        let skipped_steps = steps
            .iter()
            .filter(|s| s.status == StepStatus::Skipped)
            .count() as u32;

        let total_execution_time_ms: i32 = steps
            .iter()
            .filter_map(|s| s.execution_time_ms)
            .sum();

        let average_step_time_ms = if total_steps > 0 {
            total_execution_time_ms / total_steps as i32
        } else {
            0
        };

        let slowest = steps
            .iter()
            .filter_map(|s| s.execution_time_ms.map(|t| (s.step_name.clone(), t)))
            .max_by_key(|(_, time)| *time);

        let (slowest_step, slowest_step_time_ms) = match slowest {
            Some((name, time)) => (Some(name), Some(time)),
            None => (None, None),
        };

        Self {
            execution_id,
            total_steps,
            completed_steps,
            failed_steps,
            skipped_steps,
            total_execution_time_ms,
            average_step_time_ms,
            slowest_step,
            slowest_step_time_ms,
        }
    }
}
