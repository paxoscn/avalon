use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InterviewRecords::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InterviewRecords::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(InterviewRecords::AgentId).binary_len(16).not_null())
                    .col(ColumnDef::new(InterviewRecords::TenantId).binary_len(16).not_null())
                    .col(ColumnDef::new(InterviewRecords::UserId).binary_len(16))
                    .col(ColumnDef::new(InterviewRecords::SessionId).binary_len(16))
                    .col(
                        ColumnDef::new(InterviewRecords::Status)
                            .string_len(20)
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(InterviewRecords::Score).integer())
                    .col(ColumnDef::new(InterviewRecords::Feedback).text())
                    .col(ColumnDef::new(InterviewRecords::Questions).json())
                    .col(ColumnDef::new(InterviewRecords::Answers).json())
                    .col(ColumnDef::new(InterviewRecords::StartedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(InterviewRecords::CompletedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(InterviewRecords::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InterviewRecords::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_interview_records_agent")
                            .from(InterviewRecords::Table, InterviewRecords::AgentId)
                            .to(Agents::Table, Agents::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_interview_records_tenant")
                            .from(InterviewRecords::Table, InterviewRecords::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_interview_records_user")
                            .from(InterviewRecords::Table, InterviewRecords::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .index(
                        Index::create()
                            .name("idx_interview_records_agent")
                            .col(InterviewRecords::AgentId),
                    )
                    .index(
                        Index::create()
                            .name("idx_interview_records_tenant")
                            .col(InterviewRecords::TenantId),
                    )
                    .index(
                        Index::create()
                            .name("idx_interview_records_user")
                            .col(InterviewRecords::UserId),
                    )
                    .index(
                        Index::create()
                            .name("idx_interview_records_session")
                            .col(InterviewRecords::SessionId),
                    )
                    .index(
                        Index::create()
                            .name("idx_interview_records_status")
                            .col(InterviewRecords::Status),
                    )
                    .index(
                        Index::create()
                            .name("idx_interview_records_created_at")
                            .col(InterviewRecords::CreatedAt),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InterviewRecords::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum InterviewRecords {
    Table,
    Id,
    AgentId,
    TenantId,
    UserId,
    SessionId,
    Status,
    Score,
    Feedback,
    Questions,
    Answers,
    StartedAt,
    CompletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Agents {
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
