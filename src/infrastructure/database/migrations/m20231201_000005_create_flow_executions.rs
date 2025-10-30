use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FlowExecutions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FlowExecutions::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FlowExecutions::FlowId).binary_len(16).not_null())
                    .col(ColumnDef::new(FlowExecutions::FlowVersion).integer().not_null())
                    .col(ColumnDef::new(FlowExecutions::TenantId).binary_len(16).not_null())
                    .col(ColumnDef::new(FlowExecutions::UserId).binary_len(16).not_null())
                    .col(ColumnDef::new(FlowExecutions::SessionId).binary_len(16))
                    .col(
                        ColumnDef::new(FlowExecutions::Status)
                            .enumeration(
                                ExecutionStatus::Table,
                                [
                                    ExecutionStatus::Pending,
                                    ExecutionStatus::Running,
                                    ExecutionStatus::Completed,
                                    ExecutionStatus::Failed,
                                    ExecutionStatus::Cancelled,
                                ],
                            )
                            .default("pending"),
                    )
                    .col(ColumnDef::new(FlowExecutions::InputData).json())
                    .col(ColumnDef::new(FlowExecutions::OutputData).json())
                    .col(ColumnDef::new(FlowExecutions::ErrorMessage).text())
                    .col(
                        ColumnDef::new(FlowExecutions::StartedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(ColumnDef::new(FlowExecutions::CompletedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::value("1970-01-01 08:00:01"))
                            .not_null(),
                    )
                    .col(ColumnDef::new(FlowExecutions::ExecutionTimeMs).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_flow_execution_flow")
                            .from(FlowExecutions::Table, FlowExecutions::FlowId)
                            .to(Flows::Table, Flows::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_flow_execution_tenant")
                            .from(FlowExecutions::Table, FlowExecutions::TenantId)
                            .to(Tenants::Table, Tenants::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_flow_execution_user")
                            .from(FlowExecutions::Table, FlowExecutions::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .index(Index::create().name("idx_flow_execution_tenant_id").col(FlowExecutions::TenantId))
                    .index(Index::create().name("idx_flow_execution_user_id").col(FlowExecutions::UserId))
                    .index(Index::create().name("idx_flow_execution_session_id").col(FlowExecutions::SessionId))
                    .index(Index::create().name("idx_flow_execution_status").col(FlowExecutions::Status))
                    .index(Index::create().name("idx_flow_execution_started_at").col(FlowExecutions::StartedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FlowExecutions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum FlowExecutions {
    Table,
    Id,
    FlowId,
    FlowVersion,
    TenantId,
    UserId,
    SessionId,
    Status,
    InputData,
    OutputData,
    ErrorMessage,
    StartedAt,
    CompletedAt,
    ExecutionTimeMs,
}

#[derive(Iden)]
enum ExecutionStatus {
    Table,
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Iden)]
enum Flows {
    Table,
    Id,
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