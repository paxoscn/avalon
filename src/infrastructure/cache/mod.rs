use redis::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::error::Result;
use uuid::Uuid;

pub struct RedisCache {
}

impl RedisCache {
    pub async fn new(redis_url: &str) -> Result<Self> {
        Ok(RedisCache {})
    }
}

// pub struct RedisCache {
//     client: Client,
// }

// impl RedisCache {
//     pub async fn new(redis_url: &str) -> Result<Self> {
//         let client = Client::open(redis_url)?;
        
//         // Test connection
//         let mut conn = client.get_async_connection().await?;
//         redis::cmd("PING").query_async::<_, String>(&mut conn).await?;
        
//         Ok(RedisCache { client })
//     }
    
//     pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
//     where
//         T: for<'de> Deserialize<'de>,
//     {
//         let mut conn = self.client.get_async_connection().await?;
//         let value: Option<String> = redis::cmd("GET")
//             .arg(key)
//             .query_async(&mut conn)
//             .await?;
            
//         match value {
//             Some(json_str) => {
//                 let deserialized = serde_json::from_str(&json_str)?;
//                 Ok(Some(deserialized))
//             }
//             None => Ok(None),
//         }
//     }
    
//     pub async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
//     where
//         T: Serialize,
//     {
//         let mut conn = self.client.get_async_connection().await?;
//         let json_str = serde_json::to_string(value)?;
        
//         match ttl {
//             Some(duration) => {
//                 redis::cmd("SETEX")
//                     .arg(key)
//                     .arg(duration.as_secs())
//                     .arg(json_str)
//                     .query_async::<_, ()>(&mut conn)
//                     .await?;
//             }
//             None => {
//                 redis::cmd("SET")
//                     .arg(key)
//                     .arg(json_str)
//                     .query_async::<_, ()>(&mut conn)
//                     .await?;
//             }
//         }
        
//         Ok(())
//     }
    
//     pub async fn delete(&self, key: &str) -> Result<()> {
//         let mut conn = self.client.get_async_connection().await?;
//         redis::cmd("DEL")
//             .arg(key)
//             .query_async::<_, ()>(&mut conn)
//             .await?;
//         Ok(())
//     }

//     // Requirement 2.3, 6.3, 7.3: Cache invalidation patterns
//     pub async fn invalidate_pattern(&self, pattern: &str) -> Result<()> {
//         let mut conn = self.client.get_async_connection().await?;
//         let keys: Vec<String> = redis::cmd("KEYS")
//             .arg(pattern)
//             .query_async(&mut conn)
//             .await?;
        
//         if !keys.is_empty() {
//             redis::cmd("DEL")
//                 .arg(&keys)
//                 .query_async::<_, ()>(&mut conn)
//                 .await?;
//         }
        
//         Ok(())
//     }

//     pub async fn exists(&self, key: &str) -> Result<bool> {
//         let mut conn = self.client.get_async_connection().await?;
//         let exists: bool = redis::cmd("EXISTS")
//             .arg(key)
//             .query_async(&mut conn)
//             .await?;
//         Ok(exists)
//     }

//     // Cache key builders for consistent naming
//     pub fn flow_key(tenant_id: &Uuid, flow_id: &Uuid) -> String {
//         format!("flow:{}:{}", tenant_id, flow_id)
//     }

//     pub fn flow_list_key(tenant_id: &Uuid, page: u64, limit: u64) -> String {
//         format!("flows:{}:{}:{}", tenant_id, page, limit)
//     }

//     pub fn llm_config_key(tenant_id: &Uuid, config_id: &Uuid) -> String {
//         format!("llm_config:{}:{}", tenant_id, config_id)
//     }

//     pub fn vector_config_key(tenant_id: &Uuid, config_id: &Uuid) -> String {
//         format!("vector_config:{}:{}", tenant_id, config_id)
//     }

//     pub fn session_key(session_id: &Uuid) -> String {
//         format!("session:{}", session_id)
//     }

//     pub fn mcp_tool_key(tenant_id: &Uuid, tool_id: &Uuid) -> String {
//         format!("mcp_tool:{}:{}", tenant_id, tool_id)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis connection
    async fn test_cache_operations() {
        let cache = RedisCache::new("redis://localhost:6379").await.unwrap();
        
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestData {
            value: String,
        }
        
        let test_data = TestData {
            value: "test".to_string(),
        };
        
        // Set with TTL
        cache.set("test_key", &test_data, Some(Duration::from_secs(60))).await.unwrap();
        
        // Get
        let retrieved: Option<TestData> = cache.get("test_key").await.unwrap();
        assert_eq!(retrieved, Some(test_data));
        
        // Exists
        let exists = cache.exists("test_key").await.unwrap();
        assert!(exists);
        
        // Delete
        cache.delete("test_key").await.unwrap();
        
        // Verify deleted
        let retrieved: Option<TestData> = cache.get("test_key").await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_pattern_invalidation() {
        let cache = RedisCache::new("redis://localhost:6379").await.unwrap();
        
        // Set multiple keys
        cache.set("test:1", &"value1", None).await.unwrap();
        cache.set("test:2", &"value2", None).await.unwrap();
        cache.set("other:1", &"value3", None).await.unwrap();
        
        // Invalidate pattern
        cache.invalidate_pattern("test:*").await.unwrap();
        
        // Verify test keys are deleted
        let exists1 = cache.exists("test:1").await.unwrap();
        let exists2 = cache.exists("test:2").await.unwrap();
        let exists3 = cache.exists("other:1").await.unwrap();
        
        assert!(!exists1);
        assert!(!exists2);
        assert!(exists3);
        
        // Cleanup
        cache.delete("other:1").await.unwrap();
    }
}