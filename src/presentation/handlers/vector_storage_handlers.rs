use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::application::services::VectorStorageApplicationService;
use crate::domain::value_objects::{
    VectorRecord, SearchResult, VectorStats, BatchOperation,
    SearchFilter, FilterCondition, FilterOperator, ComparisonOperator
};
use crate::error::PlatformError;
use crate::presentation::extractors::AuthenticatedUser;

/// Request to upsert a single vector
#[derive(Debug, Deserialize)]
pub struct UpsertVectorRequest {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub namespace: Option<String>,
}

/// Request to upsert multiple vectors
#[derive(Debug, Deserialize)]
pub struct UpsertVectorsBatchRequest {
    pub vectors: Vec<UpsertVectorRequest>,
}

/// Request for vector search
#[derive(Debug, Deserialize)]
pub struct SearchVectorsRequest {
    pub vector: Vec<f32>,
    pub top_k: usize,
    pub filter: Option<SearchFilterRequest>,
    pub namespace: Option<String>,
    pub include_metadata: Option<bool>,
    pub include_values: Option<bool>,
}

/// Search filter request
#[derive(Debug, Deserialize)]
pub struct SearchFilterRequest {
    pub conditions: Vec<FilterConditionRequest>,
    pub operator: String, // "and" or "or"
}

/// Filter condition request
#[derive(Debug, Deserialize)]
pub struct FilterConditionRequest {
    pub field: String,
    pub operator: String, // "eq", "ne", "gt", "gte", "lt", "lte", "in", "not_in", "contains"
    pub value: serde_json::Value,
}

/// Request to delete vectors
#[derive(Debug, Deserialize)]
pub struct DeleteVectorsRequest {
    pub ids: Vec<String>,
    pub namespace: Option<String>,
}

/// Request for batch operations
#[derive(Debug, Deserialize)]
pub struct BatchOperationRequest {
    pub upsert: Option<Vec<UpsertVectorRequest>>,
    pub delete: Option<Vec<String>>,
}

/// Response for vector operations
#[derive(Debug, Serialize)]
pub struct VectorOperationResponse {
    pub success: bool,
    pub message: String,
    pub processed_count: Option<usize>,
}

/// Response for vector search
#[derive(Debug, Serialize)]
pub struct SearchVectorsResponse {
    pub results: Vec<SearchResultResponse>,
    pub total_results: usize,
}

/// Search result response
#[derive(Debug, Serialize)]
pub struct SearchResultResponse {
    pub id: String,
    pub score: f32,
    pub vector: Option<Vec<f32>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Response for vector statistics
#[derive(Debug, Serialize)]
pub struct VectorStatsResponse {
    pub total_vectors: u64,
    pub dimension: usize,
    pub index_fullness: f32,
    pub namespaces: HashMap<String, NamespaceStatsResponse>,
}

/// Namespace statistics response
#[derive(Debug, Serialize)]
pub struct NamespaceStatsResponse {
    pub vector_count: u64,
}

/// Query parameters for search operations
#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub config_id: Option<String>,
}

/// Query parameters for multi-store search
#[derive(Debug, Deserialize)]
pub struct MultiStoreSearchQuery {
    pub max_results_per_store: Option<usize>,
    pub aggregate: Option<bool>,
}

impl From<SearchResult> for SearchResultResponse {
    fn from(result: SearchResult) -> Self {
        SearchResultResponse {
            id: result.id,
            score: result.score,
            vector: result.vector,
            metadata: result.metadata,
        }
    }
}

impl From<VectorStats> for VectorStatsResponse {
    fn from(stats: VectorStats) -> Self {
        let namespaces = stats.namespace_stats
            .into_iter()
            .map(|(name, stats)| (name, NamespaceStatsResponse {
                vector_count: stats.vector_count,
            }))
            .collect();
        
        VectorStatsResponse {
            total_vectors: stats.total_vectors,
            dimension: stats.dimension,
            index_fullness: stats.index_fullness,
            namespaces,
        }
    }
}

impl TryFrom<SearchFilterRequest> for SearchFilter {
    type Error = PlatformError;
    
    fn try_from(filter: SearchFilterRequest) -> Result<Self, Self::Error> {
        let operator = match filter.operator.as_str() {
            "and" => FilterOperator::And,
            "or" => FilterOperator::Or,
            _ => return Err(PlatformError::ValidationError(
                "Invalid filter operator. Must be 'and' or 'or'".to_string()
            )),
        };
        
        let mut conditions = Vec::new();
        for condition in filter.conditions {
            conditions.push(condition.try_into()?);
        }
        
        Ok(SearchFilter {
            conditions,
            operator,
        })
    }
}

impl TryFrom<FilterConditionRequest> for FilterCondition {
    type Error = PlatformError;
    
    fn try_from(condition: FilterConditionRequest) -> Result<Self, Self::Error> {
        let operator = match condition.operator.as_str() {
            "eq" => ComparisonOperator::Equal,
            "ne" => ComparisonOperator::NotEqual,
            "gt" => ComparisonOperator::GreaterThan,
            "gte" => ComparisonOperator::GreaterThanOrEqual,
            "lt" => ComparisonOperator::LessThan,
            "lte" => ComparisonOperator::LessThanOrEqual,
            "in" => ComparisonOperator::In,
            "not_in" => ComparisonOperator::NotIn,
            "contains" => ComparisonOperator::Contains,
            _ => return Err(PlatformError::ValidationError(
                format!("Invalid comparison operator: {}", condition.operator)
            )),
        };
        
        Ok(FilterCondition {
            field: condition.field,
            operator,
            value: condition.value,
        })
    }
}

/// Upsert a single vector
pub async fn upsert_vector(
    State(service): State<Arc<VectorStorageApplicationService>>,
    user: AuthenticatedUser,
    Json(request): Json<UpsertVectorRequest>,
) -> Result<Json<VectorOperationResponse>, PlatformError> {
    let mut record = VectorRecord::new(request.id, request.vector, user.tenant_id)
        .map_err(|e| PlatformError::ValidationError(e))?;
    
    if let Some(metadata) = request.metadata {
        record = record.with_metadata(metadata);
    }
    
    if let Some(namespace) = request.namespace {
        record = record.with_namespace(namespace);
    }
    
    service.upsert_vector(user.tenant_id, record).await?;
    
    Ok(Json(VectorOperationResponse {
        success: true,
        message: "Vector upserted successfully".to_string(),
        processed_count: Some(1),
    }))
}

/// Upsert multiple vectors in batch
pub async fn upsert_vectors_batch(
    State(service): State<Arc<VectorStorageApplicationService>>,
    user: AuthenticatedUser,
    Json(request): Json<UpsertVectorsBatchRequest>,
) -> Result<Json<VectorOperationResponse>, PlatformError> {
    let mut records = Vec::new();
    
    for vector_req in request.vectors {
        let mut record = VectorRecord::new(vector_req.id, vector_req.vector, user.tenant_id)
            .map_err(|e| PlatformError::ValidationError(e))?;
        
        if let Some(metadata) = vector_req.metadata {
            record = record.with_metadata(metadata);
        }
        
        if let Some(namespace) = vector_req.namespace {
            record = record.with_namespace(namespace);
        }
        
        records.push(record);
    }
    
    let count = records.len();
    service.upsert_vectors_batch(user.tenant_id, records).await?;
    
    Ok(Json(VectorOperationResponse {
        success: true,
        message: format!("Successfully upserted {} vectors", count),
        processed_count: Some(count),
    }))
}

/// Search for similar vectors
pub async fn search_vectors(
    State(service): State<Arc<VectorStorageApplicationService>>,
    user: AuthenticatedUser,
    Query(query_params): Query<SearchQueryParams>,
    Json(request): Json<SearchVectorsRequest>,
) -> Result<Json<SearchVectorsResponse>, PlatformError> {
    let mut search_query = crate::domain::value_objects::SearchQuery::new(request.vector, request.top_k)
        .map_err(|e| PlatformError::ValidationError(e))?;
    
    if let Some(filter_req) = request.filter {
        let filter = filter_req.try_into()?;
        search_query = search_query.with_filter(filter);
    }
    
    if let Some(namespace) = request.namespace {
        search_query = search_query.with_namespace(namespace);
    }
    
    if let Some(include_metadata) = request.include_metadata {
        search_query = search_query.include_metadata(include_metadata);
    }
    
    if let Some(include_values) = request.include_values {
        search_query = search_query.include_values(include_values);
    }
    
    let results = if let Some(config_id_str) = query_params.config_id {
        let config_id = crate::domain::value_objects::ConfigId::from_string(&config_id_str)
            .map_err(|_| PlatformError::ValidationError("Invalid config ID format".to_string()))?;
        service.search_vectors_with_config(user.tenant_id, config_id, search_query).await?
    } else {
        service.search_vectors(user.tenant_id, search_query).await?
    };
    
    let response_results: Vec<SearchResultResponse> = results
        .into_iter()
        .map(|result| result.into())
        .collect();
    
    let total_results = response_results.len();
    
    Ok(Json(SearchVectorsResponse {
        results: response_results,
        total_results,
    }))
}

/// Delete vectors by IDs
pub async fn delete_vectors(
    State(service): State<Arc<VectorStorageApplicationService>>,
    user: AuthenticatedUser,
    Json(request): Json<DeleteVectorsRequest>,
) -> Result<Json<VectorOperationResponse>, PlatformError> {
    let count = request.ids.len();
    service.delete_vectors(user.tenant_id, request.ids, request.namespace).await?;
    
    Ok(Json(VectorOperationResponse {
        success: true,
        message: format!("Successfully deleted {} vectors", count),
        processed_count: Some(count),
    }))
}

/// Execute batch operations
pub async fn execute_batch_operation(
    State(service): State<Arc<VectorStorageApplicationService>>,
    user: AuthenticatedUser,
    Json(request): Json<BatchOperationRequest>,
) -> Result<Json<VectorOperationResponse>, PlatformError> {
    let mut batch = BatchOperation::new();
    let mut total_operations = 0;
    
    if let Some(upsert_requests) = request.upsert {
        for vector_req in upsert_requests {
            let mut record = VectorRecord::new(vector_req.id, vector_req.vector, user.tenant_id)
                .map_err(|e| PlatformError::ValidationError(e))?;
            
            if let Some(metadata) = vector_req.metadata {
                record = record.with_metadata(metadata);
            }
            
            if let Some(namespace) = vector_req.namespace {
                record = record.with_namespace(namespace);
            }
            
            batch = batch.add_upsert(record);
            total_operations += 1;
        }
    }
    
    if let Some(delete_ids) = request.delete {
        for id in delete_ids {
            batch = batch.add_delete(id);
            total_operations += 1;
        }
    }
    
    service.execute_batch_operation(user.tenant_id, batch).await?;
    
    Ok(Json(VectorOperationResponse {
        success: true,
        message: format!("Successfully executed {} operations", total_operations),
        processed_count: Some(total_operations),
    }))
}

/// Get vector storage statistics
pub async fn get_storage_stats(
    State(service): State<Arc<VectorStorageApplicationService>>,
    user: AuthenticatedUser,
    Path(namespace): Path<Option<String>>,
) -> Result<Json<VectorStatsResponse>, PlatformError> {
    let stats = service.get_storage_stats(user.tenant_id, namespace).await?;
    Ok(Json(stats.into()))
}

/// List all namespaces for the tenant
pub async fn list_namespaces(
    State(service): State<Arc<VectorStorageApplicationService>>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<String>>, PlatformError> {
    let namespaces = service.list_namespaces(user.tenant_id).await?;
    Ok(Json(namespaces))
}

/// Get statistics for a specific namespace
pub async fn get_namespace_stats(
    State(service): State<Arc<VectorStorageApplicationService>>,
    user: AuthenticatedUser,
    Path(namespace): Path<String>,
) -> Result<Json<NamespaceStatsResponse>, PlatformError> {
    let stats = service.get_namespace_stats(user.tenant_id, namespace).await?;
    Ok(Json(NamespaceStatsResponse {
        vector_count: stats.vector_count,
    }))
}

/// Search across multiple vector stores
pub async fn multi_store_search(
    State(service): State<Arc<VectorStorageApplicationService>>,
    user: AuthenticatedUser,
    Query(query_params): Query<MultiStoreSearchQuery>,
    Json(request): Json<SearchVectorsRequest>,
) -> Result<Json<serde_json::Value>, PlatformError> {
    let mut search_query = crate::domain::value_objects::SearchQuery::new(request.vector, request.top_k)
        .map_err(|e| PlatformError::ValidationError(e))?;
    
    if let Some(filter_req) = request.filter {
        let filter = filter_req.try_into()?;
        search_query = search_query.with_filter(filter);
    }
    
    if let Some(namespace) = request.namespace {
        search_query = search_query.with_namespace(namespace);
    }
    
    if let Some(include_metadata) = request.include_metadata {
        search_query = search_query.include_metadata(include_metadata);
    }
    
    if let Some(include_values) = request.include_values {
        search_query = search_query.include_values(include_values);
    }
    
    let max_results_per_store = query_params.max_results_per_store.unwrap_or(10);
    let aggregate = query_params.aggregate.unwrap_or(false);
    
    if aggregate {
        let results = service.aggregated_search(user.tenant_id, search_query, max_results_per_store).await?;
        let response_results: Vec<SearchResultResponse> = results
            .into_iter()
            .map(|result| result.into())
            .collect();
        
        Ok(Json(serde_json::json!({
            "aggregated": true,
            "results": response_results,
            "total_results": response_results.len()
        })))
    } else {
        let results = service.multi_store_search(user.tenant_id, search_query, max_results_per_store).await?;
        let mut response_results = HashMap::new();
        
        for (store_name, store_results) in results {
            let converted_results: Vec<SearchResultResponse> = store_results
                .into_iter()
                .map(|result| result.into())
                .collect();
            response_results.insert(store_name, converted_results);
        }
        
        Ok(Json(serde_json::json!({
            "aggregated": false,
            "results_by_store": response_results
        })))
    }
}

/// Get available vector stores for the tenant
pub async fn get_available_stores(
    State(service): State<Arc<VectorStorageApplicationService>>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<String>>, PlatformError> {
    let stores = service.get_available_stores(user.tenant_id).await?;
    Ok(Json(stores))
}

/// Test connections to all configured vector stores
pub async fn test_all_connections(
    State(service): State<Arc<VectorStorageApplicationService>>,
    user: AuthenticatedUser,
) -> Result<Json<HashMap<String, bool>>, PlatformError> {
    let results = service.test_all_connections(user.tenant_id).await?;
    Ok(Json(results))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::TenantId;

    #[test]
    fn test_upsert_vector_request_validation() {
        let request = UpsertVectorRequest {
            id: "test_vector".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: None,
            namespace: None,
        };
        
        assert_eq!(request.id, "test_vector");
        assert_eq!(request.vector.len(), 3);
    }
    
    #[test]
    fn test_search_filter_conversion() {
        let filter_req = SearchFilterRequest {
            conditions: vec![
                FilterConditionRequest {
                    field: "category".to_string(),
                    operator: "eq".to_string(),
                    value: serde_json::Value::String("test".to_string()),
                }
            ],
            operator: "and".to_string(),
        };
        
        let filter: Result<SearchFilter, PlatformError> = filter_req.try_into();
        assert!(filter.is_ok());
        
        let filter = filter.unwrap();
        assert_eq!(filter.conditions.len(), 1);
        assert_eq!(filter.operator, FilterOperator::And);
    }
    
    #[test]
    fn test_invalid_filter_operator() {
        let filter_req = SearchFilterRequest {
            conditions: vec![],
            operator: "invalid".to_string(),
        };
        
        let filter: Result<SearchFilter, PlatformError> = filter_req.try_into();
        assert!(filter.is_err());
    }
}