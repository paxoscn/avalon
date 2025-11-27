use sea_orm_migration::prelude::*;

use crate::infrastructure::migrations;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(migrations::m20231201_000001_create_tenants::Migration),
            Box::new(migrations::m20231201_000002_create_users::Migration),
            Box::new(migrations::m20231201_000003_create_flows::Migration),
            Box::new(migrations::m20231201_000004_create_flow_versions::Migration),
            Box::new(migrations::m20231201_000005_create_flow_executions::Migration),
            Box::new(migrations::m20231201_000006_create_chat_sessions::Migration),
            Box::new(migrations::m20231201_000007_create_chat_messages::Migration),
            Box::new(migrations::m20231201_000008_create_mcp_tools::Migration),
            Box::new(migrations::m20231201_000009_create_mcp_tool_versions::Migration),
            Box::new(migrations::m20231201_000010_create_llm_configs::Migration),
            Box::new(migrations::m20231201_000011_create_vector_configs::Migration),
            Box::new(migrations::m20231201_000012_create_audit_logs::Migration),
            Box::new(migrations::m20231201_000013_create_execution_steps::Migration),
            Box::new(migrations::m20231201_000014_create_agents::Migration),
            Box::new(migrations::m20231201_000015_create_agent_employments::Migration),
            Box::new(migrations::m20241027_000001_add_greeting_to_agents::Migration),
            Box::new(migrations::m20241027_000002_create_agent_allocations::Migration),
            Box::new(migrations::m20241027_000003_refactor_agent::Migration),
            Box::new(migrations::m20241120_000001_create_api_keys::Migration),
            Box::new(migrations::m20241126_000001_create_agent_daily_stats::Migration),
            Box::new(migrations::m20241127_000001_add_published_to_agents::Migration),
        ]
    }
}