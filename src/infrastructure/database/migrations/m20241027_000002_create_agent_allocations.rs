use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AgentAllocations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentAllocations::AgentId)
                            .binary_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentAllocations::UserId)
                            .binary_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentAllocations::AllocatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_agent_allocations")
                            .col(AgentAllocations::AgentId)
                            .col(AgentAllocations::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_allocation_agent")
                            .from(AgentAllocations::Table, AgentAllocations::AgentId)
                            .to(Agents::Table, Agents::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_allocation_user")
                            .from(AgentAllocations::Table, AgentAllocations::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_agent_allocations_user_id")
                            .col(AgentAllocations::UserId),
                    )
                    .index(
                        Index::create()
                            .name("idx_agent_allocations_agent_id")
                            .col(AgentAllocations::AgentId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AgentAllocations::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AgentAllocations {
    Table,
    AgentId,
    UserId,
    AllocatedAt,
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
