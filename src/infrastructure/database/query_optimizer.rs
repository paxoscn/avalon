// Database query optimization utilities
// Requirement 2.3: Optimize database queries

use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use std::time::{Duration, Instant};
use tracing::{info, warn};

pub struct QueryOptimizer {
    db: DatabaseConnection,
    slow_query_threshold: Duration,
}

impl QueryOptimizer {
    pub fn new(db: DatabaseConnection, slow_query_threshold: Duration) -> Self {
        Self {
            db,
            slow_query_threshold,
        }
    }

    /// Execute a query and log if it's slow
    pub async fn execute_with_timing<F, T>(&self, query_name: &str, f: F) -> Result<T, sea_orm::DbErr>
    where
        F: std::future::Future<Output = Result<T, sea_orm::DbErr>>,
    {
        let start = Instant::now();
        let result = f.await;
        let duration = start.elapsed();

        if duration > self.slow_query_threshold {
            warn!(
                "Slow query detected: {} took {:?}",
                query_name, duration
            );
        } else {
            info!("Query {} completed in {:?}", query_name, duration);
        }

        result
    }

    /// Analyze query execution plan
    pub async fn explain_query(&self, sql: &str) -> Result<String, sea_orm::DbErr> {
        let explain_sql = format!("EXPLAIN {}", sql);
        let statement = Statement::from_string(DbBackend::MySql, explain_sql);
        
        let result = self.db.query_one(statement).await?;
        
        if let Some(row) = result {
            // Extract explain output
            Ok(format!("{:?}", row))
        } else {
            Ok("No explain output".to_string())
        }
    }
}

// Common query optimization patterns
pub mod patterns {
    use sea_orm::{EntityTrait, QueryFilter, QuerySelect, Select};
    use sea_orm::sea_query::Expr;

    /// Add pagination to query efficiently
    pub fn paginate<E>(query: Select<E>, page: u64, page_size: u64) -> Select<E>
    where
        E: EntityTrait,
    {
        let offset = (page - 1) * page_size;
        query.limit(page_size).offset(offset)
    }

    /// Add index hints for better query performance
    pub fn with_index_hint<E>(query: Select<E>, index_name: &str) -> Select<E>
    where
        E: EntityTrait,
    {
        // Note: SeaORM doesn't directly support index hints
        // This would need to be implemented with raw SQL if needed
        query
    }

    /// Optimize COUNT queries by using approximate counts for large tables
    pub fn approximate_count_hint() -> &'static str {
        // For MySQL, we can use information_schema for approximate counts
        "SELECT table_rows FROM information_schema.tables WHERE table_name = ?"
    }
}

// Query result caching decorator
pub struct CachedQuery<T> {
    cache_key: String,
    ttl: Duration,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> CachedQuery<T> {
    pub fn new(cache_key: String, ttl: Duration) -> Self {
        Self {
            cache_key,
            ttl,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn cache_key(&self) -> &str {
        &self.cache_key
    }

    pub fn ttl(&self) -> Duration {
        self.ttl
    }
}

// Batch operation utilities
pub struct BatchOperations;

impl BatchOperations {
    /// Optimal batch size for bulk inserts
    pub const OPTIMAL_BATCH_SIZE: usize = 1000;

    /// Split large operations into batches
    pub fn chunk_operations<T: Clone>(items: Vec<T>, batch_size: usize) -> Vec<Vec<T>> {
        items
            .chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_chunking() {
        let items: Vec<i32> = (0..2500).collect();
        let batches = BatchOperations::chunk_operations(items, BatchOperations::OPTIMAL_BATCH_SIZE);
        
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), 1000);
        assert_eq!(batches[1].len(), 1000);
        assert_eq!(batches[2].len(), 500);
    }

    #[test]
    fn test_cached_query_creation() {
        let query: CachedQuery<String> = CachedQuery::new(
            "test_key".to_string(),
            Duration::from_secs(300),
        );
        
        assert_eq!(query.cache_key(), "test_key");
        assert_eq!(query.ttl(), Duration::from_secs(300));
    }
}
