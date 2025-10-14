use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FlowVersions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FlowVersions::Id)
                            .binary_len(16)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FlowVersions::FlowId).binary_len(16).not_null())
                    .col(ColumnDef::new(FlowVersions::Version).integer().not_null())
                    .col(ColumnDef::new(FlowVersions::Definition).json().not_null())
                    .col(ColumnDef::new(FlowVersions::ChangeLog).text())
                    .col(ColumnDef::new(FlowVersions::CreatedBy).binary_len(16).not_null())
                    .col(
                        ColumnDef::new(FlowVersions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_flow_version_flow")
                            .from(FlowVersions::Table, FlowVersions::FlowId)
                            .to(Flows::Table, Flows::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_flow_version_created_by")
                            .from(FlowVersions::Table, FlowVersions::CreatedBy)
                            .to(Users::Table, Users::Id),
                    )
                    .index(
                        Index::create()
                            .name("uk_flow_version")
                            .col(FlowVersions::FlowId)
                            .col(FlowVersions::Version)
                            .unique(),
                    )
                    .index(Index::create().name("idx_flow_version_flow_id").col(FlowVersions::FlowId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FlowVersions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum FlowVersions {
    Table,
    Id,
    FlowId,
    Version,
    Definition,
    ChangeLog,
    CreatedBy,
    CreatedAt,
}

#[derive(Iden)]
enum Flows {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}