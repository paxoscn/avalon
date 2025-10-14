use sea_orm::{Database as SeaDatabase, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;
use crate::error::Result;

pub mod migrations;
pub mod migrator;
pub mod entities;
pub mod query_optimizer;

pub use migrator::Migrator;
pub use query_optimizer::QueryOptimizer;

pub struct Database {
    pub connection: Arc<DatabaseConnection>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let connection = SeaDatabase::connect(database_url).await?;
        
        // Run migrations
        Migrator::up(&connection, None).await?;
        
        Ok(Database { connection: Arc::new(connection) })
    }
    
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }
    
    pub fn connection(&self) -> Arc<DatabaseConnection> {
        Arc::clone(&self.connection)
    }
}