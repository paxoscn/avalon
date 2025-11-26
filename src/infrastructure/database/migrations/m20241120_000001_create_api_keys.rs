use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApiKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ApiKeys::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ApiKeys::TenantId).binary_len(16).not_null())
                    .col(ColumnDef::new(ApiKeys::UserId).binary_len(16).not_null())
                    .col(ColumnDef::new(ApiKeys::Name).string_len(255).not_null())
                    .col(ColumnDef::new(ApiKeys::KeyHash).string_len(64).not_null().unique_key())
                    .col(
                        ColumnDef::new(ApiKeys::PermissionScope)
                            .json_binary()
                            .not_null(),
                            // .default(Expr::value(r#"{"agent_ids":[],"flow_ids":[],"mcp_tool_ids":[],"vector_store_ids":[]}"#)),
                    )
                    .col(
                        ColumnDef::new(ApiKeys::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(ApiKeys::ExpiresAt)
                            .timestamp_with_time_zone()
                            .default(Expr::value("1970-01-01 08:00:01"))
                            .not_null(),
                    )
                    .col(ColumnDef::new(ApiKeys::LastUsedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::value("1970-01-01 08:00:01"))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ApiKeys::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ApiKeys::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_api_key_tenant")
                            .from(ApiKeys::Table, ApiKeys::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_api_key_user")
                            .from(ApiKeys::Table, ApiKeys::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_api_keys_key_hash")
                            .col(ApiKeys::KeyHash),
                    )
                    .index(
                        Index::create()
                            .name("idx_api_keys_tenant_id")
                            .col(ApiKeys::TenantId),
                    )
                    .index(
                        Index::create()
                            .name("idx_api_keys_user_id")
                            .col(ApiKeys::UserId),
                    )
                    .index(
                        Index::create()
                            .name("idx_api_keys_enabled")
                            .col(ApiKeys::Enabled)
                            .if_not_exists(),
                    )
                    .index(
                        Index::create()
                            .name("idx_api_keys_expires_at")
                            .col(ApiKeys::ExpiresAt)
                            .if_not_exists(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiKeys::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ApiKeys {
    Table,
    Id,
    TenantId,
    UserId,
    Name,
    KeyHash,
    PermissionScope,
    Enabled,
    ExpiresAt,
    LastUsedAt,
    CreatedAt,
    UpdatedAt,
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
