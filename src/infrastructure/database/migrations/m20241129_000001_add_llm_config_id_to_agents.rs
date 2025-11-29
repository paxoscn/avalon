use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add llm_config_id column
        manager
            .alter_table(
                Table::alter()
                    .table(Agents::Table)
                    .add_column(
                        ColumnDef::new(Agents::LlmConfigId)
                            .uuid()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint to llm_configs table
        manager
            .alter_table(
                Table::alter()
                    .table(Agents::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_agents_llm_config_id")
                            .from_tbl(Agents::Table)
                            .from_col(Agents::LlmConfigId)
                            .to_tbl(LlmConfigs::Table)
                            .to_col(LlmConfigs::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop foreign key constraint
        manager
            .alter_table(
                Table::alter()
                    .table(Agents::Table)
                    .drop_foreign_key(Alias::new("fk_agents_llm_config_id"))
                    .to_owned(),
            )
            .await?;

        // Drop llm_config_id column
        manager
            .alter_table(
                Table::alter()
                    .table(Agents::Table)
                    .drop_column(Agents::LlmConfigId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Agents {
    Table,
    LlmConfigId,
}

#[derive(Iden)]
enum LlmConfigs {
    Table,
    Id,
}
