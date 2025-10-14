use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "flow_status")]
pub enum FlowStatus {
    #[sea_orm(string_value = "draft")]
    Draft,
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "archived")]
    Archived,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "flows")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub current_version: i32,
    pub status: FlowStatus,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id"
    )]
    Tenant,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    CreatedBy,
    #[sea_orm(has_many = "super::flow_version::Entity")]
    FlowVersions,
    #[sea_orm(has_many = "super::flow_execution::Entity")]
    FlowExecutions,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedBy.def()
    }
}

impl Related<super::flow_version::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FlowVersions.def()
    }
}

impl Related<super::flow_execution::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FlowExecutions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}