use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Agents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Agents::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Agents::TenantId).binary_len(16).not_null())
                    .col(ColumnDef::new(Agents::Name).string_len(255).not_null())
                    .col(ColumnDef::new(Agents::Avatar).text())
                    .col(
                        ColumnDef::new(Agents::KnowledgeBaseIds)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Agents::McpToolIds)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Agents::FlowIds)
                            .json()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Agents::SystemPrompt).text().not_null())
                    .col(ColumnDef::new(Agents::AdditionalSettings).text())
                    .col(
                        ColumnDef::new(Agents::PresetQuestions)
                            .json()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Agents::SourceAgentId).binary_len(16))
                    .col(ColumnDef::new(Agents::CreatorId).binary_len(16).not_null())
                    .col(
                        ColumnDef::new(Agents::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Agents::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_tenant")
                            .from(Agents::Table, Agents::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_creator")
                            .from(Agents::Table, Agents::CreatorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_source")
                            .from(Agents::Table, Agents::SourceAgentId)
                            .to(Agents::Table, Agents::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .index(Index::create().name("idx_agents_tenant_id").col(Agents::TenantId))
                    .index(Index::create().name("idx_agents_creator_id").col(Agents::CreatorId))
                    .index(Index::create().name("idx_agents_source_agent_id").col(Agents::SourceAgentId))
                    .index(Index::create().name("idx_agents_created_at").col(Agents::CreatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Agents::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Agents {
    Table,
    Id,
    TenantId,
    Name,
    Avatar,
    KnowledgeBaseIds,
    McpToolIds,
    FlowIds,
    SystemPrompt,
    AdditionalSettings,
    PresetQuestions,
    SourceAgentId,
    CreatorId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Tenants {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
