use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExecutionSteps::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExecutionSteps::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ExecutionSteps::ExecutionId)
                            .binary_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExecutionSteps::StepName)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExecutionSteps::StepType)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExecutionSteps::Status)
                            .enumeration(
                                Alias::new("step_status"),
                                [
                                    Alias::new("pending"),
                                    Alias::new("running"),
                                    Alias::new("completed"),
                                    Alias::new("failed"),
                                    Alias::new("skipped"),
                                ],
                            )
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(ExecutionSteps::InputData).json())
                    .col(ColumnDef::new(ExecutionSteps::OutputData).json())
                    .col(ColumnDef::new(ExecutionSteps::ErrorMessage).text())
                    .col(
                        ColumnDef::new(ExecutionSteps::StartedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(ExecutionSteps::CompletedAt).timestamp()
                            .timestamp_with_time_zone()
                            .default(Expr::value("1970-01-01 08:00:01"))
                            .not_null(),
                    )
                    .col(ColumnDef::new(ExecutionSteps::ExecutionTimeMs).integer())
                    .col(ColumnDef::new(ExecutionSteps::Metadata).json())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_execution_step_execution")
                            .from(ExecutionSteps::Table, ExecutionSteps::ExecutionId)
                            .to(FlowExecutions::Table, FlowExecutions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_execution_step_execution_id")
                            .col(ExecutionSteps::ExecutionId),
                    )
                    .index(
                        Index::create()
                            .name("idx_execution_step_status")
                            .col(ExecutionSteps::Status),
                    )
                    .index(
                        Index::create()
                            .name("idx_execution_step_started_at")
                            .col(ExecutionSteps::StartedAt),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExecutionSteps::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ExecutionSteps {
    Table,
    Id,
    ExecutionId,
    StepName,
    StepType,
    Status,
    InputData,
    OutputData,
    ErrorMessage,
    StartedAt,
    CompletedAt,
    ExecutionTimeMs,
    Metadata,
}

#[derive(DeriveIden)]
enum FlowExecutions {
    Table,
    Id,
}
