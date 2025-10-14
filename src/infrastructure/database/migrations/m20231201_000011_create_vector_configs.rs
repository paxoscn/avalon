use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(VectorConfigs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(VectorConfigs::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(VectorConfigs::TenantId).binary_len(16).not_null())
                    .col(ColumnDef::new(VectorConfigs::Name).string_len(255).not_null())
                    .col(ColumnDef::new(VectorConfigs::Provider).string_len(100).not_null())
                    .col(ColumnDef::new(VectorConfigs::Config).json().not_null())
                    .col(ColumnDef::new(VectorConfigs::IsDefault).boolean().default(false))
                    .col(
                        ColumnDef::new(VectorConfigs::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(VectorConfigs::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_vector_config_tenant")
                            .from(VectorConfigs::Table, VectorConfigs::TenantId)
                            .to(Tenants::Table, Tenants::Id),
                    )
                    .index(
                        Index::create()
                            .name("uk_vector_config_tenant_name")
                            .col(VectorConfigs::TenantId)
                            .col(VectorConfigs::Name)
                            .unique(),
                    )
                    .index(Index::create().name("idx_vector_config_tenant_id").col(VectorConfigs::TenantId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(VectorConfigs::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum VectorConfigs {
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