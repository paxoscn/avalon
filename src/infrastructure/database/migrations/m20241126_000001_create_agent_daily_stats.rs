use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AgentDailyStats::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentDailyStats::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AgentDailyStats::AgentId).binary_len(16).not_null())
                    .col(ColumnDef::new(AgentDailyStats::TenantId).binary_len(16).not_null())
                    .col(ColumnDef::new(AgentDailyStats::StatDate).date().not_null())
                    .col(
                        ColumnDef::new(AgentDailyStats::InterviewCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AgentDailyStats::InterviewPassedCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AgentDailyStats::EmploymentCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AgentDailyStats::SessionCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AgentDailyStats::MessageCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AgentDailyStats::TokenCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AgentDailyStats::Revenue)
                            .decimal_len(20, 6)
                            .not_null()
                            .default(0.0),
                    )
                    .col(
                        ColumnDef::new(AgentDailyStats::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AgentDailyStats::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_daily_stats_agent")
                            .from(AgentDailyStats::Table, AgentDailyStats::AgentId)
                            .to(Agents::Table, Agents::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_daily_stats_tenant")
                            .from(AgentDailyStats::Table, AgentDailyStats::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_agent_daily_stats_agent_date")
                            .col(AgentDailyStats::AgentId)
                            .col(AgentDailyStats::StatDate)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("idx_agent_daily_stats_tenant_date")
                            .col(AgentDailyStats::TenantId)
                            .col(AgentDailyStats::StatDate),
                    )
                    .index(
                        Index::create()
                            .name("idx_agent_daily_stats_stat_date")
                            .col(AgentDailyStats::StatDate),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AgentDailyStats::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AgentDailyStats {
    Table,
    Id,
    AgentId,
    TenantId,
    StatDate,
    InterviewCount,
    InterviewPassedCount,
    EmploymentCount,
    SessionCount,
    MessageCount,
    TokenCount,
    Revenue,
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
