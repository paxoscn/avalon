use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatMessages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatMessages::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChatMessages::SessionId).binary_len(16).not_null())
                    .col(
                        ColumnDef::new(ChatMessages::Role)
                            .enumeration(
                                MessageRoles::Table,
                                [MessageRoles::User, MessageRoles::Assistant, MessageRoles::System],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(ChatMessages::Content).text().not_null())
                    .col(ColumnDef::new(ChatMessages::Metadata).json())
                    .col(
                        ColumnDef::new(ChatMessages::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_message_session")
                            .from(ChatMessages::Table, ChatMessages::SessionId)
                            .to(ChatSessions::Table, ChatSessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(Index::create().name("idx_chat_message_session_id").col(ChatMessages::SessionId))
                    .index(Index::create().name("idx_chat_message_created_at").col(ChatMessages::CreatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChatMessages::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ChatMessages {
    Table,
    Id,
    SessionId,
    Role,
    Content,
    Metadata,
    CreatedAt,
}

#[derive(Iden)]
enum MessageRoles {
    Table,
    User,
    Assistant,
    System,
}

#[derive(Iden)]
enum ChatSessions {
    Table,
    Id,
}