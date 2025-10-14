use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::value_objects::{
    TenantId, VectorRecord, SearchQuery, SearchResult, IndexConfig, 
    VectorStats, BatchOperation, SearchFilter
};
use crate::error::PlatformError;

/// Domain service for vector storage operations with business rules
#[async_trait]
pub trait VectorStoreDomainService: Send + Sync {
    /// Store a single vector record with tenant isolation
    async fn store_vector(&self, record: VectorRecord) -> Result<(), PlatformError>;
    
    /// Store multiple vector records in batch with tenant isolation
    async fn store_vectors_batch(&self, records: Vec<VectorRecord>) -> Result<(), PlatformError>;
    
    /// Search for similar vectors with tenant isolation
    async fn search_vectors(
        &self, 
        query: SearchQuery, 
        tenant_id: TenantId
    ) -> Result<Vec<SearchResult>, PlatformError>;
    
    /// Delete vectors by IDs with tenant isolation
    async fn delete_vectors(&self, ids: Vec<String>, tenant_id: TenantId) -> Result<(), PlatformError>;
    
    /// Execute batch operations with tenant isolation
    async fn execute_batch(
        &self, 
        operation: BatchOperation, 
        tenant_id: TenantId
    ) -> Result<(), PlatformError>;
    
    /// Create or update index configuration
    async fn manage_index(&self, config: IndexConfig, tenant_id: TenantId) -> Result<(), PlatformError>;
    
    /// Get vector storage statistics for tenant
    async fn get_stats(&self, tenant_id: TenantId) -> Result<VectorStats, PlatformError>;
    
    /// Validate vector record before storage
    fn validate_vector_record(&self, record: &VectorRecord) -> Result<(), PlatformError>;
    
    /// Validate search query
    fn validate_search_query(&self, query: &SearchQuery) -> Result<(), PlatformError>;
    
    /// Check if tenant has permission to access namespace
    fn validate_tenant_namespace_access(
        &self, 
        tenant_id: TenantId, 
        namespace: Option<&str>
    ) -> Result<(), PlatformError>;
    
    /// Generate tenant-specific namespace
    fn generate_tenant_namespace(&self, tenant_id: TenantId, namespace: Option<&str>) -> String;
}

/// Implementation of vector storage domain service
pub struct VectorStoreDomainServiceImpl {
    max_vector_dimension: usize,
    max_batch_size: usize,
    max_search_results: usize,
}

impl VectorStoreDomainServiceImpl {
    pub fn new() -> Self {
        Self {
            max_vector_dimension: 4096,
            max_batch_size: 1000,
            max_search_results: 10000,
        }
    }
    
    pub fn with_limits(
        max_vector_dimension: usize,
        max_batch_size: usize,
        max_search_results: usize,
    ) -> Self {
        Self {
            max_vector_dimension,
            max_batch_size,
            max_search_results,
        }
    }
}

#[async_trait]
impl VectorStoreDomainService for VectorStoreDomainServiceImpl {
    async fn store_vector(&self, mut record: VectorRecord) -> Result<(), PlatformError> {
        self.validate_vector_record(&record)?;
        
        // Apply tenant isolation by modifying namespace
        let tenant_namespace = self.generate_tenant_namespace(
            record.tenant_id, 
            record.namespace.as_deref()
        );
        record.namespace = Some(tenant_namespace);
        
        // Additional business rules can be added here
        // For example: rate limiting, quota checks, etc.
        
        Ok(())
    }
    
    async fn store_vectors_batch(&self, mut records: Vec<VectorRecord>) -> Result<(), PlatformError> {
        if records.len() > self.max_batch_size {
            return Err(PlatformError::ValidationError(
                format!("Batch size {} exceeds maximum allowed {}", records.len(), self.max_batch_size)
            ));
        }
        
        // Validate all records
        for record in &records {
            self.validate_vector_record(record)?;
        }
        
        // Apply tenant isolation to all records
        for record in &mut records {
            let tenant_namespace = self.generate_tenant_namespace(
                record.tenant_id, 
                record.namespace.as_deref()
            );
            record.namespace = Some(tenant_namespace);
        }
        
        Ok(())
    }
    
    async fn search_vectors(
        &self, 
        mut query: SearchQuery, 
        tenant_id: TenantId
    ) -> Result<Vec<SearchResult>, PlatformError> {
        self.validate_search_query(&query)?;
        self.validate_tenant_namespace_access(tenant_id, query.namespace.as_deref())?;
        
        // Apply tenant isolation
        let tenant_namespace = self.generate_tenant_namespace(tenant_id, query.namespace.as_deref());
        query.namespace = Some(tenant_namespace);
        
        // Add tenant filter to search query
        if let Some(ref mut filter) = query.filter {
            // Add tenant_id to existing filter conditions
            filter.conditions.push(crate::domain::value_objects::FilterCondition {
                field: "tenant_id".to_string(),
                operator: crate::domain::value_objects::ComparisonOperator::Equal,
                value: serde_json::Value::String(tenant_id.to_string()),
            });
        } else {
            // Create new filter with tenant_id
            query.filter = Some(SearchFilter {
                conditions: vec![crate::domain::value_objects::FilterCondition {
                    field: "tenant_id".to_string(),
                    operator: crate::domain::value_objects::ComparisonOperator::Equal,
                    value: serde_json::Value::String(tenant_id.to_string()),
                }],
                operator: crate::domain::value_objects::FilterOperator::And,
            });
        }
        
        Ok(Vec::new()) // Placeholder - actual implementation would call infrastructure layer
    }
    
    async fn delete_vectors(&self, ids: Vec<String>, tenant_id: TenantId) -> Result<(), PlatformError> {
        if ids.is_empty() {
            return Err(PlatformError::ValidationError("No vector IDs provided for deletion".to_string()));
        }
        
        if ids.len() > self.max_batch_size {
            return Err(PlatformError::ValidationError(
                format!("Delete batch size {} exceeds maximum allowed {}", ids.len(), self.max_batch_size)
            ));
        }
        
        // Validate IDs
        for id in &ids {
            if id.trim().is_empty() {
                return Err(PlatformError::ValidationError("Vector ID cannot be empty".to_string()));
            }
        }
        
        Ok(())
    }
    
    async fn execute_batch(
        &self, 
        mut operation: BatchOperation, 
        tenant_id: TenantId
    ) -> Result<(), PlatformError> {
        if operation.is_empty() {
            return Err(PlatformError::ValidationError("Batch operation is empty".to_string()));
        }
        
        let total_operations = operation.upsert.len() + operation.delete.len();
        if total_operations > self.max_batch_size {
            return Err(PlatformError::ValidationError(
                format!("Total batch operations {} exceeds maximum allowed {}", total_operations, self.max_batch_size)
            ));
        }
        
        // Validate and apply tenant isolation to upsert operations
        for record in &mut operation.upsert {
            self.validate_vector_record(record)?;
            let tenant_namespace = self.generate_tenant_namespace(
                record.tenant_id, 
                record.namespace.as_deref()
            );
            record.namespace = Some(tenant_namespace);
        }
        
        // Validate delete operations
        for id in &operation.delete {
            if id.trim().is_empty() {
                return Err(PlatformError::ValidationError("Vector ID cannot be empty".to_string()));
            }
        }
        
        Ok(())
    }
    
    async fn manage_index(&self, config: IndexConfig, tenant_id: TenantId) -> Result<(), PlatformError> {
        if config.dimension > self.max_vector_dimension {
            return Err(PlatformError::ValidationError(
                format!("Vector dimension {} exceeds maximum allowed {}", config.dimension, self.max_vector_dimension)
            ));
        }
        
        // Generate tenant-specific index name
        let tenant_index_name = format!("{}_{}", tenant_id, config.name);
        
        Ok(())
    }
    
    async fn get_stats(&self, tenant_id: TenantId) -> Result<VectorStats, PlatformError> {
        // Placeholder implementation
        Ok(VectorStats {
            total_vectors: 0,
            dimension: 0,
            index_fullness: 0.0,
            namespace_stats: HashMap::new(),
        })
    }
    
    fn validate_vector_record(&self, record: &VectorRecord) -> Result<(), PlatformError> {
        if record.id.trim().is_empty() {
            return Err(PlatformError::ValidationError("Vector record ID cannot be empty".to_string()));
        }
        
        if record.vector.is_empty() {
            return Err(PlatformError::ValidationError("Vector cannot be empty".to_string()));
        }
        
        if record.vector.len() > self.max_vector_dimension {
            return Err(PlatformError::ValidationError(
                format!("Vector dimension {} exceeds maximum allowed {}", record.vector.len(), self.max_vector_dimension)
            ));
        }
        
        // Validate vector values (no NaN or infinite values)
        for (i, &value) in record.vector.iter().enumerate() {
            if !value.is_finite() {
                return Err(PlatformError::ValidationError(
                    format!("Vector contains invalid value at index {}: {}", i, value)
                ));
            }
        }
        
        Ok(())
    }
    
    fn validate_search_query(&self, query: &SearchQuery) -> Result<(), PlatformError> {
        if query.vector.is_empty() {
            return Err(PlatformError::ValidationError("Search vector cannot be empty".to_string()));
        }
        
        if query.vector.len() > self.max_vector_dimension {
            return Err(PlatformError::ValidationError(
                format!("Search vector dimension {} exceeds maximum allowed {}", query.vector.len(), self.max_vector_dimension)
            ));
        }
        
        if query.top_k == 0 {
            return Err(PlatformError::ValidationError("top_k must be greater than 0".to_string()));
        }
        
        if query.top_k > self.max_search_results {
            return Err(PlatformError::ValidationError(
                format!("top_k {} exceeds maximum allowed {}", query.top_k, self.max_search_results)
            ));
        }
        
        // Validate search vector values
        for (i, &value) in query.vector.iter().enumerate() {
            if !value.is_finite() {
                return Err(PlatformError::ValidationError(
                    format!("Search vector contains invalid value at index {}: {}", i, value)
                ));
            }
        }
        
        Ok(())
    }
    
    fn validate_tenant_namespace_access(
        &self, 
        tenant_id: TenantId, 
        namespace: Option<&str>
    ) -> Result<(), PlatformError> {
        // Business rule: tenants can only access their own namespaces
        if let Some(ns) = namespace {
            if ns.starts_with(&format!("{}_", tenant_id)) || ns == tenant_id.to_string() {
                Ok(())
            } else {
                Err(PlatformError::AuthorizationFailed(
                    format!("Tenant {} does not have access to namespace {}", tenant_id, ns)
                ))
            }
        } else {
            Ok(())
        }
    }
    
    fn generate_tenant_namespace(&self, tenant_id: TenantId, namespace: Option<&str>) -> String {
        match namespace {
            Some(ns) if !ns.is_empty() => format!("{}_{}", tenant_id, ns),
            _ => tenant_id.to_string(),
        }
    }
}

impl Default for VectorStoreDomainServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::TenantId;
    use uuid::Uuid;

    fn create_test_tenant_id() -> TenantId {
        TenantId::new()
    }

    fn create_test_vector_record(tenant_id: TenantId) -> VectorRecord {
        VectorRecord::new(
            "test_vector_1".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
            tenant_id,
        ).unwrap()
    }

    #[tokio::test]
    async fn test_validate_vector_record_success() {
        let service = VectorStoreDomainServiceImpl::new();
        let tenant_id = create_test_tenant_id();
        let record = create_test_vector_record(tenant_id);
        
        let result = service.validate_vector_record(&record);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_vector_record_empty_id() {
        let service = VectorStoreDomainServiceImpl::new();
        let tenant_id = create_test_tenant_id();
        let mut record = create_test_vector_record(tenant_id);
        record.id = "".to_string();
        
        let result = service.validate_vector_record(&record);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ID cannot be empty"));
    }

    #[tokio::test]
    async fn test_validate_vector_record_empty_vector() {
        let service = VectorStoreDomainServiceImpl::new();
        let tenant_id = create_test_tenant_id();
        let mut record = create_test_vector_record(tenant_id);
        record.vector = vec![];
        
        let result = service.validate_vector_record(&record);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Vector cannot be empty"));
    }

    #[tokio::test]
    async fn test_validate_search_query_success() {
        let service = VectorStoreDomainServiceImpl::new();
        let query = SearchQuery::new(vec![0.1, 0.2, 0.3], 10).unwrap();
        
        let result = service.validate_search_query(&query);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_tenant_namespace() {
        let service = VectorStoreDomainServiceImpl::new();
        let tenant_id = create_test_tenant_id();
        
        // Test with custom namespace
        let namespace = service.generate_tenant_namespace(tenant_id, Some("custom"));
        assert_eq!(namespace, format!("{}_{}", tenant_id, "custom"));
        
        // Test without namespace
        let namespace = service.generate_tenant_namespace(tenant_id, None);
        assert_eq!(namespace, tenant_id.to_string());
    }

    #[tokio::test]
    async fn test_validate_tenant_namespace_access() {
        let service = VectorStoreDomainServiceImpl::new();
        let tenant_id = create_test_tenant_id();
        
        // Test valid access
        let valid_namespace = format!("{}_{}", tenant_id, "test");
        let result = service.validate_tenant_namespace_access(tenant_id, Some(&valid_namespace));
        assert!(result.is_ok());
        
        // Test invalid access
        let invalid_namespace = "other_tenant_namespace";
        let result = service.validate_tenant_namespace_access(tenant_id, Some(invalid_namespace));
        assert!(result.is_err());
    }
}