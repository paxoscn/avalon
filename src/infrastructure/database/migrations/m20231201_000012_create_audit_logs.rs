use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditLogs::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuditLogs::TenantId).binary_len(16).not_null())
                    .col(ColumnDef::new(AuditLogs::UserId).binary_len(16))
                    .col(ColumnDef::new(AuditLogs::Action).string_len(100).not_null())
                    .col(ColumnDef::new(AuditLogs::ResourceType).string_len(100).not_null())
                    .col(ColumnDef::new(AuditLogs::ResourceId).binary_len(16))
                    .col(ColumnDef::new(AuditLogs::Details).json())
                    .col(ColumnDef::new(AuditLogs::IpAddress).string_len(45))
                    .col(ColumnDef::new(AuditLogs::UserAgent).text())
                    .col(
                        ColumnDef::new(AuditLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_log_tenant")
                            .from(AuditLogs::Table, AuditLogs::TenantId)
                            .to(Tenants::Table, Tenants::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_log_user")
                            .from(AuditLogs::Table, AuditLogs::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .index(Index::create().name("idx_audit_log_tenant_id").col(AuditLogs::TenantId))
                    .index(Index::create().name("idx_audit_log_user_id").col(AuditLogs::UserId))
                    .index(Index::create().name("idx_audit_log_action").col(AuditLogs::Action))
                    .index(
                        Index::create()
                            .name("idx_audit_log_resource")
                            .col(AuditLogs::ResourceType)
                            .col(AuditLogs::ResourceId),
                    )
                    .index(Index::create().name("idx_audit_log_created_at").col(AuditLogs::CreatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AuditLogs {
    Table,
    Id,
    TenantId,
    UserId,
    Action,
    ResourceType,
    ResourceId,
    Details,
    IpAddress,
    UserAgent,
    CreatedAt,
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