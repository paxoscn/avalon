use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
    pub password_hash: String,
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
    #[sea_orm(has_many = "super::flow::Entity")]
    CreatedFlows,
    #[sea_orm(has_many = "super::flow_version::Entity")]
    CreatedFlowVersions,
    #[sea_orm(has_many = "super::flow_execution::Entity")]
    FlowExecutions,
    #[sea_orm(has_many = "super::chat_session::Entity")]
    ChatSessions,
    #[sea_orm(has_many = "super::mcp_tool::Entity")]
    CreatedMcpTools,
    #[sea_orm(has_many = "super::mcp_tool_version::Entity")]
    CreatedMcpToolVersions,
    #[sea_orm(has_many = "super::audit_log::Entity")]
    AuditLogs,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl Related<super::flow::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedFlows.def()
    }
}

impl Related<super::flow_version::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedFlowVersions.def()
    }
}

impl Related<super::flow_execution::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FlowExecutions.def()
    }
}

impl Related<super::chat_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChatSessions.def()
    }
}

impl Related<super::mcp_tool::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedMcpTools.def()
    }
}

impl Related<super::mcp_tool_version::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedMcpToolVersions.def()
    }
}

impl Related<super::audit_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuditLogs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}