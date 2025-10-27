use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add employer_id column to agents table
        manager
            .alter_table(
                Table::alter()
                    .table(Agents::Table)
                    .add_column(ColumnDef::new(Agents::EmployerId).binary_len(16))
                    .to_owned(),
            )
            .await?;

        // Add fired_at column to agents table
        manager
            .alter_table(
                Table::alter()
                    .table(Agents::Table)
                    .add_column(ColumnDef::new(Agents::FiredAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint for employer_id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_agent_employer")
                    .from(Agents::Table, Agents::EmployerId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        // Create index on employer_id
        manager
            .create_index(
                Index::create()
                    .name("idx_agents_employer_id")
                    .table(Agents::Table)
                    .col(Agents::EmployerId)
                    .to_owned(),
            )
            .await?;

        // Create index on fired_at
        manager
            .create_index(
                Index::create()
                    .name("idx_agents_fired_at")
                    .table(Agents::Table)
                    .col(Agents::FiredAt)
                    .to_owned(),
            )
            .await?;

        // Drop agent_employments table if it exists
        manager
            .drop_table(
                Table::drop()
                    .table(AgentEmployments::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Recreate agent_employments table
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
            .await?;

        // Drop indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_agents_fired_at")
                    .table(Agents::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_agents_employer_id")
                    .table(Agents::Table)
                    .to_owned(),
            )
            .await?;

        // Drop foreign key constraint
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_agent_employer")
                    .table(Agents::Table)
                    .to_owned(),
            )
            .await?;

        // Drop columns
        manager
            .alter_table(
                Table::alter()
                    .table(Agents::Table)
                    .drop_column(Agents::FiredAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Agents::Table)
                    .drop_column(Agents::EmployerId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Agents {
    Table,
    Id,
    EmployerId,
    FiredAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum AgentEmployments {
    Table,
    AgentId,
    UserId,
    EmployedAt,
}
