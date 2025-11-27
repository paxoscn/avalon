use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "agents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub greeting: Option<String>,
    pub knowledge_base_ids: Json,
    pub mcp_tool_ids: Json,
    pub flow_ids: Json,
    pub system_prompt: String,
    pub additional_settings: Option<String>,
    pub preset_questions: Json,
    pub source_agent_id: Option<Uuid>,
    pub creator_id: Uuid,
    pub employer_id: Option<Uuid>,
    pub fired_at: Option<DateTime<Utc>>,
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub price: Option<Decimal>,
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
        from = "Column::CreatorId",
        to = "super::user::Column::Id"
    )]
    Creator,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::EmployerId",
        to = "super::user::Column::Id"
    )]
    Employer,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::SourceAgentId",
        to = "Column::Id"
    )]
    SourceAgent,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Creator.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
