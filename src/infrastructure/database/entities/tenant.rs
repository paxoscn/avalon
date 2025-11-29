use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tenants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user::Entity")]
    Users,
    #[sea_orm(has_many = "super::user_tenant_relation::Entity")]
    UserTenantRelations,
    #[sea_orm(has_many = "super::flow::Entity")]
    Flows,
    #[sea_orm(has_many = "super::flow_execution::Entity")]
    FlowExecutions,
    #[sea_orm(has_many = "super::chat_session::Entity")]
    ChatSessions,
    #[sea_orm(has_many = "super::mcp_tool::Entity")]
    McpTools,
    #[sea_orm(has_many = "super::llm_config::Entity")]
    LlmConfigs,
    #[sea_orm(has_many = "super::vector_config::Entity")]
    VectorConfigs,
    #[sea_orm(has_many = "super::audit_log::Entity")]
    AuditLogs,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::flow::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Flows.def()
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
        Relation::McpTools.def()
    }
}

impl Related<super::llm_config::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LlmConfigs.def()
    }
}

impl Related<super::vector_config::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VectorConfigs.def()
    }
}

impl Related<super::audit_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuditLogs.def()
    }
}

impl Related<super::user_tenant_relation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserTenantRelations.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}