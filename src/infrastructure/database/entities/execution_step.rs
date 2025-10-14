use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "step_status")]
pub enum StepStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "skipped")]
    Skipped,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "execution_steps")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub execution_id: Uuid,
    pub step_name: String,
    pub step_type: String,
    pub status: StepStatus,
    pub input_data: Option<Json>,
    pub output_data: Option<Json>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
    pub metadata: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::flow_execution::Entity",
        from = "Column::ExecutionId",
        to = "super::flow_execution::Column::Id"
    )]
    FlowExecution,
}

impl Related<super::flow_execution::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FlowExecution.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
