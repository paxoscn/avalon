// Redis cache implementation for performance optimization
use redis::{AsyncCommands, Client, RedisError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, RedisError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value: Option<String> = conn.get(key).await?;
        
        match value {
            Some(v) => {
                let deserialized = serde_json::from_str(&v)
                    .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed", e.to_string())))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), RedisError>
    where
        T: Serialize,
    {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let serialized = serde_json::to_string(value)
            .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string())))?;
        
        if let Some(ttl) = ttl {
            conn.set_ex(key, serialized, ttl.as_secs()).await?;
        } else {
            conn.set(key, serialized).await?;
        }
        
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        conn.del(key).await?;
        Ok(())
    }

    pub async fn exists(&self, key: &str) -> Result<bool, RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<(), RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let keys: Vec<String> = conn.keys(pattern).await?;
        
        if !keys.is_empty() {
            conn.del(keys).await?;
        }
        
        Ok(())
    }

    // Cache key builders
    pub fn flow_key(tenant_id: &uuid::Uuid, flow_id: &uuid::Uuid) -> String {
        format!("flow:{}:{}", tenant_id, flow_id)
    }

    pub fn flow_list_key(tenant_id: &uuid::Uuid) -> String {
        format!("flows:{}", tenant_id)
    }

    pub fn llm_config_key(tenant_id: &uuid::Uuid, config_id: &uuid::Uuid) -> String {
        format!("llm_config:{}:{}", tenant_id, config_id)
    }

    pub fn vector_config_key(tenant_id: &uuid::Uuid, config_id: &uuid::Uuid) -> String {
        format!("vector_config:{}:{}", tenant_id, config_id)
    }

    pub fn session_key(session_id: &uuid::Uuid) -> String {
        format!("session:{}", session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis connection
    async fn test_cache_operations() {
        let cache = RedisCache::new("redis://localhost:6379").unwrap();
        
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestData {
            value: String,
        }
        
        let test_data = TestData {
            value: "test".to_string(),
        };
        
        // Set
        cache.set("test_key", &test_data, Some(Duration::from_secs(60))).await.unwrap();
        
        // Get
        let retrieved: Option<TestData> = cache.get("test_key").await.unwrap();
        assert_eq!(retrieved, Some(test_data));
        
        // Delete
        cache.delete("test_key").await.unwrap();
        
        // Verify deleted
        let retrieved: Option<TestData> = cache.get("test_key").await.unwrap();
        assert_eq!(retrieved, None);
    }
}
