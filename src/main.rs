mod config;
mod error;
mod domain;
mod application;
mod infrastructure;
mod presentation;

use tklog::LOG;
use std::sync::Arc;
use crate::config::AppConfig;
use crate::infrastructure::database::Database;
use crate::infrastructure::cache::RedisCache;
use crate::presentation::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    LOG.uselog();
    // tracing_subscriber::fmt::init();

    // Load configuration
    let config = AppConfig::load()?;
    
    // Initialize database
    let database = Database::new(&config.database_url).await?;
    
    // Initialize Redis cache
    let cache = RedisCache::new(&config.redis_url).await?;
    
    // Start server
    let server = Server::new(config, Arc::new(database), Arc::new(cache));
    server.start().await?;

    Ok(())
}