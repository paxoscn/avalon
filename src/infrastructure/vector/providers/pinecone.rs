use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::value_objects::{
    VectorRecord, SearchQuery, SearchResult, IndexConfig, VectorStats, BatchOperation,
    DistanceMetric, NamespaceStats
};
use crate::error::PlatformError;
use crate::infrastructure::vector::{VectorStore, VectorStoreConfig, VectorProviderInfo};
use super::{ProviderUtils, VectorHttpClient, HttpClientConfig};

/// Pinecone vector store implementation
pub struct PineconeStore {
    client: VectorHttpClient,
    api_key: String,
    environment: String,
    index_name: String,
    base_url: String,
}

impl PineconeStore {
    pub async fn new(config: VectorStoreConfig) -> Result<Self, PlatformError> {
        // Validate required parameters
        ProviderUtils::validate_required_params(&config, &["api_key", "environment", "index_name"])?;
        
        let api_key = ProviderUtils::get_connection_param(&config, "api_key")?;
        let environment = ProviderUtils::get_connection_param(&config, "environment")?;
        let index_name = ProviderUtils::get_connection_param(&config, "index_name")?;
        
        let base_url = format!("https://{}-{}.svc.{}.pinecone.io", 
            index_name, 
            environment.split('-').next().unwrap_or(&environment),
            environment
        );
        
        // Create HTTP client with Pinecone-specific headers
        let mut headers = HashMap::new();
        headers.insert("Api-Key".to_string(), api_key.clone());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        let client = ProviderUtils::create_http_client(&config, headers)?;
        
        let store = Self {
            client,
            api_key,
            environment,
            index_name,
            base_url,
        };
        
        // Test connection
        store.test_connection().await?;
        
        Ok(store)
    }
    
    fn build_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Api-Key".to_string(), self.api_key.clone());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }
    
    fn convert_distance_metric(metric: &DistanceMetric) -> String {
        match metric {
            DistanceMetric::Cosine => "cosine".to_string(),
            DistanceMetric::Euclidean => "euclidean".to_string(),
            DistanceMetric::DotProduct => "dotproduct".to_string(),
        }
    }
    
    fn convert_search_results(response: PineconeQueryResponse) -> Vec<SearchResult> {
        response.matches.into_iter().map(|m| {
            let mut result = SearchResult::new(m.id, m.score);
            if let Some(values) = m.values {
                result = result.with_vector(values);
            }
            if let Some(metadata) = m.metadata {
                result = result.with_metadata(metadata);
            }
            result
        }).collect()
    }
}

#[async_trait]
impl VectorStore for PineconeStore {
    async fn upsert(&self, record: VectorRecord) -> Result<(), PlatformError> {
        let request = PineconeUpsertRequest {
            vectors: vec![PineconeVector {
                id: record.id,
                values: record.vector,
                metadata: Some(record.metadata),
            }],
            namespace: record.namespace,
        };
        
        let url = format!("{}/vectors/upsert", self.base_url);
        let _response: PineconeUpsertResponse = self.client
            .post_json(&url, &request, Some(self.build_headers()))
            .await?;
        
        Ok(())
    }
    
    async fn upsert_batch(&self, records: Vec<VectorRecord>) -> Result<(), PlatformError> {
        if records.is_empty() {
            return Ok(());
        }
        
        // Group records by namespace
        let mut namespace_groups: HashMap<Option<String>, Vec<VectorRecord>> = HashMap::new();
        for record in records {
            namespace_groups.entry(record.namespace.clone()).or_default().push(record);
        }
        
        // Process each namespace group separately
        for (namespace, group_records) in namespace_groups {
            let vectors: Vec<PineconeVector> = group_records.into_iter().map(|record| {
                PineconeVector {
                    id: record.id,
                    values: record.vector,
                    metadata: Some(record.metadata),
                }
            }).collect();
            
            let request = PineconeUpsertRequest {
                vectors,
                namespace,
            };
            
            let url = format!("{}/vectors/upsert", self.base_url);
            let _response: PineconeUpsertResponse = self.client
                .post_json(&url, &request, Some(self.build_headers()))
                .await?;
        }
        
        Ok(())
    }
    
    async fn query(&self, query: SearchQuery) -> Result<Vec<SearchResult>, PlatformError> {
        let request = PineconeQueryRequest {
            vector: Some(query.vector),
            top_k: query.top_k as u32,
            namespace: query.namespace,
            filter: query.filter.map(|f| self.convert_filter(f)),
            include_values: query.include_values,
            include_metadata: query.include_metadata,
        };
        
        let url = format!("{}/query", self.base_url);
        let response: PineconeQueryResponse = self.client
            .post_json(&url, &request, Some(self.build_headers()))
            .await?;
        
        Ok(Self::convert_search_results(response))
    }
    
    async fn delete(&self, ids: Vec<String>, namespace: Option<String>) -> Result<(), PlatformError> {
        let request = PineconeDeleteRequest {
            ids: Some(ids),
            delete_all: None,
            namespace,
            filter: None,
        };
        
        let url = format!("{}/vectors/delete", self.base_url);
        let _response: PineconeDeleteResponse = self.client
            .post_json(&url, &request, Some(self.build_headers()))
            .await?;
        
        Ok(())
    }
    
    async fn execute_batch(&self, operation: BatchOperation) -> Result<(), PlatformError> {
        // Execute upserts first
        if !operation.upsert.is_empty() {
            self.upsert_batch(operation.upsert).await?;
        }
        
        // Then execute deletes
        if !operation.delete.is_empty() {
            self.delete(operation.delete, None).await?;
        }
        
        Ok(())
    }
    
    async fn create_index(&self, config: IndexConfig) -> Result<(), PlatformError> {
        // Note: Pinecone index creation is typically done through their control plane API
        // This would require a different endpoint and potentially different authentication
        Err(PlatformError::VectorStoreError(
            "Index creation not supported through data plane API. Use Pinecone console or control plane API.".to_string()
        ))
    }
    
    async fn delete_index(&self, _index_name: String) -> Result<(), PlatformError> {
        // Note: Pinecone index deletion is typically done through their control plane API
        Err(PlatformError::VectorStoreError(
            "Index deletion not supported through data plane API. Use Pinecone console or control plane API.".to_string()
        ))
    }
    
    async fn list_indexes(&self) -> Result<Vec<String>, PlatformError> {
        // Note: Pinecone index listing is typically done through their control plane API
        Err(PlatformError::VectorStoreError(
            "Index listing not supported through data plane API. Use Pinecone console or control plane API.".to_string()
        ))
    }
    
    async fn get_stats(&self, namespace: Option<String>) -> Result<VectorStats, PlatformError> {
        let request = PineconeStatsRequest { namespace };
        
        let url = format!("{}/describe_index_stats", self.base_url);
        let response: PineconeStatsResponse = self.client
            .post_json(&url, &request, Some(self.build_headers()))
            .await?;
        
        let mut namespace_stats = HashMap::new();
        if let Some(namespaces) = response.namespaces {
            for (ns_name, ns_data) in namespaces {
                namespace_stats.insert(ns_name, NamespaceStats {
                    vector_count: ns_data.vector_count,
                });
            }
        }
        
        Ok(VectorStats {
            total_vectors: response.total_vector_count,
            dimension: response.dimension,
            index_fullness: response.index_fullness,
            namespace_stats,
        })
    }
    
    async fn test_connection(&self) -> Result<(), PlatformError> {
        let request = PineconeStatsRequest { namespace: None };
        let url = format!("{}/describe_index_stats", self.base_url);
        
        let _response: PineconeStatsResponse = self.client
            .post_json(&url, &request, Some(self.build_headers()))
            .await
            .map_err(|e| PlatformError::VectorStoreError(
                format!("Pinecone connection test failed: {}", e)
            ))?;
        
        Ok(())
    }
    
    fn provider_info(&self) -> VectorProviderInfo {
        VectorProviderInfo {
            name: "Pinecone".to_string(),
            version: "1.0".to_string(),
            supports_namespaces: true,
            supports_metadata_filtering: true,
            supports_hybrid_search: false,
            max_vector_dimension: 20000,
            max_batch_size: 1000,
        }
    }
}

impl PineconeStore {
    fn convert_filter(&self, filter: crate::domain::value_objects::SearchFilter) -> serde_json::Value {
        // Convert our generic filter format to Pinecone's filter format
        // This is a simplified implementation - Pinecone has complex filter syntax
        let mut pinecone_filter = serde_json::Map::new();
        
        for condition in filter.conditions {
            let value = match condition.operator {
                crate::domain::value_objects::ComparisonOperator::Equal => {
                    serde_json::json!({ "$eq": condition.value })
                },
                crate::domain::value_objects::ComparisonOperator::NotEqual => {
                    serde_json::json!({ "$ne": condition.value })
                },
                crate::domain::value_objects::ComparisonOperator::In => {
                    serde_json::json!({ "$in": condition.value })
                },
                crate::domain::value_objects::ComparisonOperator::NotIn => {
                    serde_json::json!({ "$nin": condition.value })
                },
                _ => condition.value, // For other operators, use value directly
            };
            
            pinecone_filter.insert(condition.field, value);
        }
        
        serde_json::Value::Object(pinecone_filter)
    }
}

// Pinecone API request/response structures

#[derive(Debug, Serialize)]
struct PineconeUpsertRequest {
    vectors: Vec<PineconeVector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
}

#[derive(Debug, Serialize)]
struct PineconeVector {
    id: String,
    values: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct PineconeUpsertResponse {
    upserted_count: u32,
}

#[derive(Debug, Serialize)]
struct PineconeQueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    vector: Option<Vec<f32>>,
    #[serde(rename = "topK")]
    top_k: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<serde_json::Value>,
    #[serde(rename = "includeValues")]
    include_values: bool,
    #[serde(rename = "includeMetadata")]
    include_metadata: bool,
}

#[derive(Debug, Deserialize)]
struct PineconeQueryResponse {
    matches: Vec<PineconeMatch>,
}

#[derive(Debug, Deserialize)]
struct PineconeMatch {
    id: String,
    score: f32,
    values: Option<Vec<f32>>,
    metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize)]
struct PineconeDeleteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    ids: Option<Vec<String>>,
    #[serde(rename = "deleteAll", skip_serializing_if = "Option::is_none")]
    delete_all: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct PineconeDeleteResponse {
    // Pinecone delete response is typically empty on success
}

#[derive(Debug, Serialize)]
struct PineconeStatsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PineconeStatsResponse {
    #[serde(rename = "totalVectorCount")]
    total_vector_count: u64,
    dimension: usize,
    #[serde(rename = "indexFullness")]
    index_fullness: f32,
    namespaces: Option<HashMap<String, PineconeNamespaceStats>>,
}

#[derive(Debug, Deserialize)]
struct PineconeNamespaceStats {
    #[serde(rename = "vectorCount")]
    vector_count: u64,
}