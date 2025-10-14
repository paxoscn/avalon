use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(McpTools::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(McpTools::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(McpTools::TenantId).binary_len(16).not_null())
                    .col(ColumnDef::new(McpTools::Name).string_len(255).not_null())
                    .col(ColumnDef::new(McpTools::Description).text())
                    .col(ColumnDef::new(McpTools::CurrentVersion).integer().default(1))
                    .col(
                        ColumnDef::new(McpTools::Status)
                            .enumeration(
                                ToolStatus::Table,
                                [ToolStatus::Active, ToolStatus::Inactive],
                            )
                            .default("active"),
                    )
                    .col(ColumnDef::new(McpTools::CreatedBy).binary_len(16).not_null())
                    .col(
                        ColumnDef::new(McpTools::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(McpTools::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mcp_tool_tenant")
                            .from(McpTools::Table, McpTools::TenantId)
                            .to(Tenants::Table, Tenants::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mcp_tool_created_by")
                            .from(McpTools::Table, McpTools::CreatedBy)
                            .to(Users::Table, Users::Id),
                    )
                    .index(
                        Index::create()
                            .name("uk_mcp_tool_tenant_name")
                            .col(McpTools::TenantId)
                            .col(McpTools::Name)
                            .unique(),
                    )
                    .index(Index::create().name("idx_mcp_tool_tenant_id").col(McpTools::TenantId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(McpTools::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum McpTools {
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
enum ToolStatus {
    Table,
    Active,
    Inactive,
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