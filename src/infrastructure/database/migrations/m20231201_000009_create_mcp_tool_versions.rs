use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(McpToolVersions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(McpToolVersions::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(McpToolVersions::ToolId).binary_len(16).not_null())
                    .col(ColumnDef::new(McpToolVersions::Version).integer().not_null())
                    .col(ColumnDef::new(McpToolVersions::Config).json().not_null())
                    .col(ColumnDef::new(McpToolVersions::ChangeLog).text())
                    .col(ColumnDef::new(McpToolVersions::CreatedBy).binary_len(16).not_null())
                    .col(
                        ColumnDef::new(McpToolVersions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mcp_tool_version_tool")
                            .from(McpToolVersions::Table, McpToolVersions::ToolId)
                            .to(McpTools::Table, McpTools::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mcp_tool_version_created_by")
                            .from(McpToolVersions::Table, McpToolVersions::CreatedBy)
                            .to(Users::Table, Users::Id),
                    )
                    .index(
                        Index::create()
                            .name("uk_mcp_tool_version")
                            .col(McpToolVersions::ToolId)
                            .col(McpToolVersions::Version)
                            .unique(),
                    )
                    .index(Index::create().name("idx_mcp_tool_version_tool_id").col(McpToolVersions::ToolId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(McpToolVersions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum McpToolVersions {
    Table,
    Id,
    ToolId,
    Version,
    Config,
    ChangeLog,
    CreatedBy,
    CreatedAt,
}

#[derive(Iden)]
enum McpTools {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}