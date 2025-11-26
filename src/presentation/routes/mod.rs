pub mod agent_routes;
pub mod auth_routes;
pub mod config_routes;
pub mod flow_routes;
pub mod mcp_routes;
pub mod mcp_server_routes;
pub mod session_audit_routes;
pub mod vector_config_routes;
pub mod vector_storage_routes;
pub mod file_routes;
pub mod api_key_routes;

pub use auth_routes::*;

// Re-export route creation functions
pub use agent_routes::agent_routes;
pub use config_routes::{llm_config_routes, vector_config_routes};
pub use flow_routes::flow_routes;
pub use mcp_routes::create_mcp_api_routes;
pub use mcp_server_routes::create_mcp_server_api_routes;
pub use session_audit_routes::{audit_routes, execution_history_routes, session_routes};
pub use vector_config_routes::create_vector_config_routes;
pub use vector_storage_routes::create_vector_storage_routes;
pub use file_routes::file_routes;
pub use api_key_routes::api_key_routes;
