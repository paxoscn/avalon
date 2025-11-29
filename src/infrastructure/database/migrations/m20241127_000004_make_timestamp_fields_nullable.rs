use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Modify flow_executions.completed_at to be nullable
        manager
            .alter_table(
                Table::alter()
                    .table(FlowExecutions::Table)
                    .modify_column(
                        ColumnDef::new(FlowExecutions::CompletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Modify execution_steps.completed_at to be nullable
        manager
            .alter_table(
                Table::alter()
                    .table(ExecutionSteps::Table)
                    .modify_column(
                        ColumnDef::new(ExecutionSteps::CompletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Modify agents.fired_at to be nullable
        manager
            .alter_table(
                Table::alter()
                    .table(Agents::Table)
                    .modify_column(
                        ColumnDef::new(Agents::FiredAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Modify api_keys.expires_at to be nullable
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .modify_column(
                        ColumnDef::new(ApiKeys::ExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Modify api_keys.last_used_at to be nullable
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .modify_column(
                        ColumnDef::new(ApiKeys::LastUsedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Modify interview_records.started_at to be nullable
        manager
            .alter_table(
                Table::alter()
                    .table(InterviewRecords::Table)
                    .modify_column(
                        ColumnDef::new(InterviewRecords::StartedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Modify interview_records.completed_at to be nullable
        manager
            .alter_table(
                Table::alter()
                    .table(InterviewRecords::Table)
                    .modify_column(
                        ColumnDef::new(InterviewRecords::CompletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert flow_executions.completed_at to not null with default
        manager
            .alter_table(
                Table::alter()
                    .table(FlowExecutions::Table)
                    .modify_column(
                        ColumnDef::new(FlowExecutions::CompletedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::value("1970-01-01 08:00:01"))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Revert execution_steps.completed_at to not null with default
        manager
            .alter_table(
                Table::alter()
                    .table(ExecutionSteps::Table)
                    .modify_column(
                        ColumnDef::new(ExecutionSteps::CompletedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::value("1970-01-01 08:00:01"))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Revert agents.fired_at to not null with default
        manager
            .alter_table(
                Table::alter()
                    .table(Agents::Table)
                    .modify_column(
                        ColumnDef::new(Agents::FiredAt)
                            .timestamp_with_time_zone()
                            .default(Expr::value("1970-01-01 08:00:01"))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Revert api_keys.expires_at to not null with default
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .modify_column(
                        ColumnDef::new(ApiKeys::ExpiresAt)
                            .timestamp_with_time_zone()
                            .default(Expr::value("1970-01-01 08:00:01"))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Revert api_keys.last_used_at to not null with default
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .modify_column(
                        ColumnDef::new(ApiKeys::LastUsedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::value("1970-01-01 08:00:01"))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Revert interview_records.started_at to not null with default
        manager
            .alter_table(
                Table::alter()
                    .table(InterviewRecords::Table)
                    .modify_column(
                        ColumnDef::new(InterviewRecords::StartedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::value("1970-01-01 08:00:01"))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Revert interview_records.completed_at to not null with default
        manager
            .alter_table(
                Table::alter()
                    .table(InterviewRecords::Table)
                    .modify_column(
                        ColumnDef::new(InterviewRecords::CompletedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::value("1970-01-01 08:00:01"))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum FlowExecutions {
    Table,
    CompletedAt,
}

#[derive(Iden)]
enum ExecutionSteps {
    Table,
    CompletedAt,
}

#[derive(Iden)]
enum Agents {
    Table,
    FiredAt,
}

#[derive(Iden)]
enum ApiKeys {
    Table,
    ExpiresAt,
    LastUsedAt,
}

#[derive(Iden)]
enum InterviewRecords {
    Table,
    StartedAt,
    CompletedAt,
}
