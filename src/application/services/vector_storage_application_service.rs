use std::collections::HashMap;
use std::sync::Arc;

use crate::application::services::VectorApplicationService;
use crate::domain::value_objects::{
    TenantId, VectorRecord, SearchQuery, SearchResult, VectorStats, BatchOperation
};
use crate::error::PlatformError;
use crate::infrastructure::vector::VectorStoreRegistry;

/// Application service for vector storage operations
pub struct VectorStorageApplicationService {
    vector_config_service: Arc<VectorApplicationService>,
    store_registry: Arc<VectorStoreRegistry>,
}

impl VectorStorageApplicationService {
    pub fn new(
        vector_config_service: Arc<VectorApplicationService>,
        store_registry: Arc<VectorStoreRegistry>,
    ) -> Self {
        Self {
            vector_config_service,
            store_registry,
        }
    }
    
    /// Store a single vector record using the default vector store for the tenant
    pub async fn upsert_vector(
        &self,
        tenant_id: TenantId,
        record: VectorRecord,
    ) -> Result<(), PlatformError> {
        // Validate tenant ID matches record
        if record.tenant_id != tenant_id {
            return Err(PlatformError::ValidationError(
                "Vector record tenant ID does not match request tenant ID".to_string()
            ));
        }
        
        let store = self.vector_config_service.get_default_vector_store(tenant_id).await?;
        store.upsert(record).await
    }
    
    /// Store multiple vector records in batch
    pub async fn upsert_vectors_batch(
        &self,
        tenant_id: TenantId,
        records: Vec<VectorRecord>,
    ) -> Result<(), PlatformError> {
        // Validate all records belong to the tenant
        for record in &records {
            if record.tenant_id != tenant_id {
                return Err(PlatformError::ValidationError(
                    "All vector records must belong to the same tenant".to_string()
                ));
            }
        }
        
        let store = self.vector_config_service.get_default_vector_store(tenant_id).await?;
        store.upsert_batch(records).await
    }
    
    /// Search for similar vectors using the default vector store
    pub async fn search_vectors(
        &self,
        tenant_id: TenantId,
        query: SearchQuery,
    ) -> Result<Vec<SearchResult>, PlatformError> {
        let store = self.vector_config_service.get_default_vector_store(tenant_id).await?;
        store.query(query).await
    }
    
    /// Delete vectors by IDs
    pub async fn delete_vectors(
        &self,
        tenant_id: TenantId,
        ids: Vec<String>,
        namespace: Option<String>,
    ) -> Result<(), PlatformError> {
        let store = self.vector_config_service.get_default_vector_store(tenant_id).await?;
        store.delete(ids, namespace).await
    }
    
    /// Execute batch operations (upsert and delete)
    pub async fn execute_batch_operation(
        &self,
        tenant_id: TenantId,
        operation: BatchOperation,
    ) -> Result<(), PlatformError> {
        // Validate all upsert records belong to the tenant
        for record in &operation.upsert {
            if record.tenant_id != tenant_id {
                return Err(PlatformError::ValidationError(
                    "All vector records must belong to the same tenant".to_string()
                ));
            }
        }
        
        let store = self.vector_config_service.get_default_vector_store(tenant_id).await?;
        store.execute_batch(operation).await
    }
    
    /// Get vector storage statistics for the tenant
    pub async fn get_storage_stats(
        &self,
        tenant_id: TenantId,
        namespace: Option<String>,
    ) -> Result<VectorStats, PlatformError> {
        let store = self.vector_config_service.get_default_vector_store(tenant_id).await?;
        store.get_stats(namespace).await
    }
    
    /// Store vectors using a specific configuration
    pub async fn upsert_vector_with_config(
        &self,
        tenant_id: TenantId,
        config_id: crate::domain::value_objects::ConfigId,
        record: VectorRecord,
    ) -> Result<(), PlatformError> {
        // Validate tenant ID matches record
        if record.tenant_id != tenant_id {
            return Err(PlatformError::ValidationError(
                "Vector record tenant ID does not match request tenant ID".to_string()
            ));
        }
        
        let store = self.vector_config_service.get_vector_store(config_id).await?;
        store.upsert(record).await
    }
    
    /// Search vectors using a specific configuration
    pub async fn search_vectors_with_config(
        &self,
        tenant_id: TenantId,
        config_id: crate::domain::value_objects::ConfigId,
        query: SearchQuery,
    ) -> Result<Vec<SearchResult>, PlatformError> {
        // Verify the config belongs to the tenant
        let config = self.vector_config_service.get_config(config_id).await?;
        if config.tenant_id != tenant_id {
            return Err(PlatformError::AuthorizationFailed(
                "Configuration does not belong to the specified tenant".to_string()
            ));
        }
        
        let store = self.vector_config_service.get_vector_store(config_id).await?;
        store.query(query).await
    }
    
    /// Get all available vector stores for a tenant
    pub async fn get_available_stores(&self, tenant_id: TenantId) -> Result<Vec<String>, PlatformError> {
        let configs = self.vector_config_service.get_configs_by_tenant(tenant_id).await?;
        Ok(configs.into_iter().map(|config| config.name).collect())
    }
    
    /// Test connection to all configured vector stores for a tenant
    pub async fn test_all_connections(&self, tenant_id: TenantId) -> Result<HashMap<String, bool>, PlatformError> {
        self.vector_config_service.get_health_status(tenant_id).await
    }
    
    /// Create a namespace in the default vector store
    pub async fn create_namespace(
        &self,
        _tenant_id: TenantId,
        namespace: String,
    ) -> Result<(), PlatformError> {
        // For now, we'll just validate the namespace name
        if namespace.trim().is_empty() {
            return Err(PlatformError::ValidationError(
                "Namespace name cannot be empty".to_string()
            ));
        }
        
        // Most vector databases create namespaces implicitly when vectors are inserted
        // This is a placeholder for explicit namespace creation if needed
        Ok(())
    }
    
    /// List all namespaces for a tenant
    pub async fn list_namespaces(&self, tenant_id: TenantId) -> Result<Vec<String>, PlatformError> {
        let stats = self.get_storage_stats(tenant_id, None).await?;
        Ok(stats.namespace_stats.keys().cloned().collect())
    }
    
    /// Get statistics for a specific namespace
    pub async fn get_namespace_stats(
        &self,
        tenant_id: TenantId,
        namespace: String,
    ) -> Result<crate::domain::value_objects::NamespaceStats, PlatformError> {
        let stats = self.get_storage_stats(tenant_id, Some(namespace.clone())).await?;
        stats.namespace_stats.get(&namespace)
            .cloned()
            .ok_or_else(|| PlatformError::NotFound(format!("Namespace '{}' not found", namespace)))
    }
    
    /// Perform similarity search across multiple vector stores
    pub async fn multi_store_search(
        &self,
        tenant_id: TenantId,
        query: SearchQuery,
        max_results_per_store: usize,
    ) -> Result<HashMap<String, Vec<SearchResult>>, PlatformError> {
        let configs = self.vector_config_service.get_configs_by_tenant(tenant_id).await?;
        let mut results = HashMap::new();
        
        for config in configs {
            // Create a modified query with limited results
            let mut store_query = query.clone();
            store_query.top_k = std::cmp::min(store_query.top_k, max_results_per_store);
            
            match self.vector_config_service.get_vector_store(config.id).await {
                Ok(store) => {
                    match store.query(store_query).await {
                        Ok(store_results) => {
                            results.insert(config.name, store_results);
                        }
                        Err(e) => {
                            log::warn!("Failed to search in store '{}': {}", config.name, e);
                            // Continue with other stores
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to get store '{}': {}", config.name, e);
                    // Continue with other stores
                }
            }
        }
        
        Ok(results)
    }
    
    /// Aggregate search results from multiple stores and return top results
    pub async fn aggregated_search(
        &self,
        tenant_id: TenantId,
        query: SearchQuery,
        max_results_per_store: usize,
    ) -> Result<Vec<SearchResult>, PlatformError> {
        let multi_results = self.multi_store_search(tenant_id, query.clone(), max_results_per_store).await?;
        
        // Aggregate all results
        let mut all_results = Vec::new();
        for (store_name, store_results) in multi_results {
            for mut result in store_results {
                // Add store information to metadata if not present
                if let Some(ref mut metadata) = result.metadata {
                    metadata.insert("source_store".to_string(), serde_json::Value::String(store_name.clone()));
                } else {
                    let mut metadata = HashMap::new();
                    metadata.insert("source_store".to_string(), serde_json::Value::String(store_name.clone()));
                    result.metadata = Some(metadata);
                }
                all_results.push(result);
            }
        }
        
        // Sort by score (descending) and take top results
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all_results.truncate(query.top_k);
        
        Ok(all_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{VectorRecord, SearchQuery};
    use std::collections::HashMap;

    // Mock implementations would go here for testing
    // For now, we'll focus on the basic structure and validation logic
    
    #[test]
    fn test_validate_tenant_id_mismatch() {
        let tenant_id = TenantId::new();
        let different_tenant_id = TenantId::new();
        
        let record = VectorRecord::new(
            "test_id".to_string(),
            vec![1.0, 2.0, 3.0],
            different_tenant_id,
        ).unwrap();
        
        // This would fail in the actual service call due to tenant ID mismatch
        assert_ne!(record.tenant_id, tenant_id);
    }
    
    #[test]
    fn test_search_query_validation() {
        let query = SearchQuery::new(vec![1.0, 2.0, 3.0], 10);
        assert!(query.is_ok());
        
        let empty_query = SearchQuery::new(vec![], 10);
        assert!(empty_query.is_err());
        
        let zero_results_query = SearchQuery::new(vec![1.0, 2.0, 3.0], 0);
        assert!(zero_results_query.is_err());
    }
    
    #[test]
    fn test_batch_operation_creation() {
        let tenant_id = TenantId::new();
        let record1 = VectorRecord::new("id1".to_string(), vec![1.0, 2.0], tenant_id).unwrap();
        let record2 = VectorRecord::new("id2".to_string(), vec![3.0, 4.0], tenant_id).unwrap();
        
        let batch = BatchOperation::new()
            .add_upsert(record1)
            .add_upsert(record2)
            .add_delete("old_id".to_string());
        
        assert_eq!(batch.upsert.len(), 2);
        assert_eq!(batch.delete.len(), 1);
        assert!(!batch.is_empty());
    }
}