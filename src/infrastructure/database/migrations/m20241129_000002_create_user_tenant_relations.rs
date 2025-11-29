use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserTenantRelations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserTenantRelations::UserId)
                            .binary_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserTenantRelations::TenantId)
                            .binary_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserTenantRelations::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(UserTenantRelations::UserId)
                            .col(UserTenantRelations::TenantId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_tenant_relation_user")
                            .from(UserTenantRelations::Table, UserTenantRelations::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_tenant_relation_tenant")
                            .from(UserTenantRelations::Table, UserTenantRelations::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_user_tenant_relation_user_id")
                            .col(UserTenantRelations::UserId),
                    )
                    .index(
                        Index::create()
                            .name("idx_user_tenant_relation_tenant_id")
                            .col(UserTenantRelations::TenantId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserTenantRelations::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum UserTenantRelations {
    Table,
    UserId,
    TenantId,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum Tenants {
    Table,
    Id,
}
