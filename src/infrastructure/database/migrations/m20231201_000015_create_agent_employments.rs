use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AgentEmployments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentEmployments::AgentId)
                            .binary_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentEmployments::UserId)
                            .binary_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentEmployments::EmployedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_agent_employments")
                            .col(AgentEmployments::AgentId)
                            .col(AgentEmployments::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_employment_agent")
                            .from(AgentEmployments::Table, AgentEmployments::AgentId)
                            .to(Agents::Table, Agents::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_employment_user")
                            .from(AgentEmployments::Table, AgentEmployments::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_agent_employments_user_id")
                            .col(AgentEmployments::UserId),
                    )
                    .index(
                        Index::create()
                            .name("idx_agent_employments_agent_id")
                            .col(AgentEmployments::AgentId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AgentEmployments::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AgentEmployments {
    Table,
    AgentId,
    UserId,
    EmployedAt,
}

#[derive(Iden)]
enum Agents {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
