use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Flows::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Flows::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Flows::TenantId).binary_len(16).not_null())
                    .col(ColumnDef::new(Flows::Name).string_len(255).not_null())
                    .col(ColumnDef::new(Flows::Description).text())
                    .col(ColumnDef::new(Flows::CurrentVersion).integer().default(1))
                    .col(
                        ColumnDef::new(Flows::Status)
                            .enumeration(
                                FlowStatus::Table,
                                [FlowStatus::Draft, FlowStatus::Active, FlowStatus::Archived],
                            )
                            .default("draft"),
                    )
                    .col(ColumnDef::new(Flows::CreatedBy).binary_len(16).not_null())
                    .col(
                        ColumnDef::new(Flows::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Flows::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_flow_tenant")
                            .from(Flows::Table, Flows::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_flow_created_by")
                            .from(Flows::Table, Flows::CreatedBy)
                            .to(Users::Table, Users::Id),
                    )
                    .index(Index::create().name("idx_flow_tenant_id").col(Flows::TenantId))
                    .index(Index::create().name("idx_flow_status").col(Flows::Status))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Flows::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Flows {
    Table,
    Id,
    TenantId,
    Name,
    Description,
    CurrentVersion,
    Status,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum FlowStatus {
    Table,
    Draft,
    Active,
    Archived,
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