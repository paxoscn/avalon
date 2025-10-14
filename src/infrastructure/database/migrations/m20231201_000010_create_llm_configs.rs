use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LlmConfigs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LlmConfigs::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LlmConfigs::TenantId).binary_len(16).not_null())
                    .col(ColumnDef::new(LlmConfigs::Name).string_len(255).not_null())
                    .col(ColumnDef::new(LlmConfigs::Provider).string_len(100).not_null())
                    .col(ColumnDef::new(LlmConfigs::Config).json().not_null())
                    .col(ColumnDef::new(LlmConfigs::IsDefault).boolean().default(false))
                    .col(
                        ColumnDef::new(LlmConfigs::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LlmConfigs::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_llm_config_tenant")
                            .from(LlmConfigs::Table, LlmConfigs::TenantId)
                            .to(Tenants::Table, Tenants::Id),
                    )
                    .index(
                        Index::create()
                            .name("uk_llm_config_tenant_name")
                            .col(LlmConfigs::TenantId)
                            .col(LlmConfigs::Name)
                            .unique(),
                    )
                    .index(Index::create().name("idx_llm_config_tenant_id").col(LlmConfigs::TenantId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LlmConfigs::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum LlmConfigs {
    Table,
    Id,
    TenantId,
    Name,
    Provider,
    Config,
    IsDefault,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Tenants {
    Table,
    Id,
}