use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatSessions::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChatSessions::TenantId).binary_len(16).not_null())
                    .col(ColumnDef::new(ChatSessions::UserId).binary_len(16).not_null())
                    .col(ColumnDef::new(ChatSessions::Title).string_len(255))
                    .col(ColumnDef::new(ChatSessions::Context).json())
                    .col(
                        ColumnDef::new(ChatSessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatSessions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_session_tenant")
                            .from(ChatSessions::Table, ChatSessions::TenantId)
                            .to(Tenants::Table, Tenants::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_session_user")
                            .from(ChatSessions::Table, ChatSessions::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .index(
                        Index::create()
                            .name("idx_chat_session_tenant_user")
                            .col(ChatSessions::TenantId)
                            .col(ChatSessions::UserId),
                    )
                    .index(Index::create().name("idx_chat_session_updated_at").col(ChatSessions::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChatSessions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ChatSessions {
    Table,
    Id,
    TenantId,
    UserId,
    Title,
    Context,
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