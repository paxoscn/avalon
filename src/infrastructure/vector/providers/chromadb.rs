use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::value_objects::{
    VectorRecord, SearchQuery, SearchResult, IndexConfig, VectorStats, BatchOperation,
    DistanceMetric, NamespaceStats
};
use crate::error::PlatformError;
use crate::infrastructure::vector::{VectorStore, VectorStoreConfig, VectorProviderInfo};
use super::{ProviderUtils, VectorHttpClient};

/// ChromaDB vector store implementation
pub struct ChromaDBStore {
    client: VectorHttpClient,
    base_url: String,
    collection_name: String,
    api_key: Option<String>,
}

impl ChromaDBStore {
    pub async fn new(config: VectorStoreConfig) -> Result<Self, PlatformError> {
        // Validate required parameters
        ProviderUtils::validate_required_params(&config, &["base_url", "collection_name"])?;
        
        let base_url = ProviderUtils::get_connection_param(&config, "base_url")?;
        let collection_name = ProviderUtils::get_connection_param(&config, "collection_name")?;
        let api_key = ProviderUtils::get_optional_connection_param(&config, "api_key");
        
        // Create HTTP client with ChromaDB-specific headers
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        if let Some(ref key) = api_key {
            headers.insert("Authorization".to_string(), format!("Bearer {}", key));
        }
        
        let client = ProviderUtils::create_http_client(&config, headers)?;
        
        let store = Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            collection_name,
            api_key,
        };
        
        // Test connection and ensure collection exists
        store.test_connection().await?;
        store.ensure_collection_exists().await?;
        
        Ok(store)
    }
    
    fn build_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        if let Some(ref api_key) = self.api_key {
            headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        }
        
        headers
    }
    
    async fn ensure_collection_exists(&self) -> Result<(), PlatformError> {
        // Check if collection exists
        let url = format!("{}/api/v1/collections/{}", self.base_url, self.collection_name);
        
        match self.client.get::<ChromaCollection>(&url, Some(self.build_headers())).await {
            Ok(_) => Ok(()), // Collection exists
            Err(_) => {
                // Collection doesn't exist, create it
                self.create_collection().await
            }
        }
    }
    
    async fn create_collection(&self) -> Result<(), PlatformError> {
        let request = ChromaCreateCollectionRequest {
            name: self.collection_name.clone(),
            metadata: Some(HashMap::new()),
            get_or_create: Some(true),
        };
        
        let url = format!("{}/api/v1/collections", self.base_url);
        let _response: ChromaCollection = self.client
            .post_json(&url, &request, Some(self.build_headers()))
            .await?;
        
        Ok(())
    }
    
    fn convert_distance_metric(metric: &DistanceMetric) -> String {
        match metric {
            DistanceMetric::Cosine => "cosine".to_string(),
            DistanceMetric::Euclidean => "l2".to_string(),
            DistanceMetric::DotProduct => "ip".to_string(),
        }
    }
    
    fn convert_search_results(response: ChromaQueryResponse) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let (Some(ids), Some(distances)) = (response.ids, response.distances) {
            for (i, id) in ids.into_iter().enumerate() {
                if let Some(distance) = distances.get(i) {
                    let score = 1.0 - distance; // Convert distance to similarity score
                    let mut result = SearchResult::new(id, score);
                    
                    // Add embeddings if available
                    if let Some(ref embeddings) = response.embeddings {
                        if let Some(embedding) = embeddings.get(i) {
                            result = result.with_vector(embedding.clone());
                        }
                    }
                    
                    // Add metadata if available
                    if let Some(ref metadatas) = response.metadatas {
                        if let Some(metadata) = metadatas.get(i) {
                            if let Some(metadata) = metadata {
                                result = result.with_metadata(metadata.clone());
                            }
                        }
                    }
                    
                    results.push(result);
                }
            }
        }
        
        results
    }
}

#[async_trait]
impl VectorStore for ChromaDBStore {
    async fn upsert(&self, record: VectorRecord) -> Result<(), PlatformError> {
        let request = ChromaAddRequest {
            ids: vec![record.id],
            embeddings: vec![record.vector],
            metadatas: vec![Some(record.metadata)],
            documents: None,
        };
        
        let url = format!("{}/api/v1/collections/{}/add", self.base_url, self.collection_name);
        let _response: ChromaAddResponse = self.client
            .post_json(&url, &request, Some(self.build_headers()))
            .await?;
        
        Ok(())
    }
    
    async fn upsert_batch(&self, records: Vec<VectorRecord>) -> Result<(), PlatformError> {
        if records.is_empty() {
            return Ok(());
        }
        
        let ids: Vec<String> = records.iter().map(|r| r.id.clone()).collect();
        let embeddings: Vec<Vec<f32>> = records.iter().map(|r| r.vector.clone()).collect();
        let metadatas: Vec<Option<HashMap<String, serde_json::Value>>> = records
            .iter()
            .map(|r| Some(r.metadata.clone()))
            .collect();
        
        let request = ChromaAddRequest {
            ids,
            embeddings,
            metadatas,
            documents: None,
        };
        
        let url = format!("{}/api/v1/collections/{}/upsert", self.base_url, self.collection_name);
        let _response: ChromaAddResponse = self.client
            .post_json(&url, &request, Some(self.build_headers()))
            .await?;
        
        Ok(())
    }
    
    async fn query(&self, query: SearchQuery) -> Result<Vec<SearchResult>, PlatformError> {
        let request = ChromaQueryRequest {
            query_embeddings: vec![query.vector],
            n_results: Some(query.top_k as u32),
            where_clause: query.filter.map(|f| self.convert_filter(f)),
            where_document: None,
            include: Some(vec![
                "embeddings".to_string(),
                "metadatas".to_string(),
                "distances".to_string(),
            ]),
        };
        
        let url = format!("{}/api/v1/collections/{}/query", self.base_url, self.collection_name);
        let response: ChromaQueryResponse = self.client
            .post_json(&url, &request, Some(self.build_headers()))
            .await?;
        
        Ok(Self::convert_search_results(response))
    }
    
    async fn delete(&self, ids: Vec<String>, _namespace: Option<String>) -> Result<(), PlatformError> {
        let request = ChromaDeleteRequest {
            ids: Some(ids),
            where_clause: None,
        };
        
        let url = format!("{}/api/v1/collections/{}/delete", self.base_url, self.collection_name);
        let _response: ChromaDeleteResponse = self.client
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
        // ChromaDB doesn't have explicit index creation - collections serve as indexes
        let request = ChromaCreateCollectionRequest {
            name: config.name,
            metadata: Some({
                let mut metadata = HashMap::new();
                metadata.insert("dimension".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(config.dimension)
                ));
                metadata.insert("metric".to_string(), serde_json::Value::String(
                    Self::convert_distance_metric(&config.metric)
                ));
                metadata
            }),
            get_or_create: Some(false),
        };
        
        let url = format!("{}/api/v1/collections", self.base_url);
        let _response: ChromaCollection = self.client
            .post_json(&url, &request, Some(self.build_headers()))
            .await?;
        
        Ok(())
    }
    
    async fn delete_index(&self, index_name: String) -> Result<(), PlatformError> {
        let url = format!("{}/api/v1/collections/{}", self.base_url, index_name);
        self.client.delete(&url, Some(self.build_headers())).await?;
        Ok(())
    }
    
    async fn list_indexes(&self) -> Result<Vec<String>, PlatformError> {
        let url = format!("{}/api/v1/collections", self.base_url);
        let collections: Vec<ChromaCollection> = self.client
            .get(&url, Some(self.build_headers()))
            .await?;
        
        Ok(collections.into_iter().map(|c| c.name).collect())
    }
    
    async fn get_stats(&self, _namespace: Option<String>) -> Result<VectorStats, PlatformError> {
        let url = format!("{}/api/v1/collections/{}", self.base_url, self.collection_name);
        let collection: ChromaCollection = self.client
            .get(&url, Some(self.build_headers()))
            .await?;
        
        // ChromaDB doesn't provide detailed stats like Pinecone
        // We'll return basic information
        Ok(VectorStats {
            total_vectors: 0, // ChromaDB doesn't expose this easily
            dimension: 0,     // Would need to infer from data
            index_fullness: 0.0,
            namespace_stats: HashMap::new(),
        })
    }
    
    async fn test_connection(&self) -> Result<(), PlatformError> {
        let url = format!("{}/api/v1/heartbeat", self.base_url);
        
        match self.client.get::<serde_json::Value>(&url, Some(self.build_headers())).await {
            Ok(_) => Ok(()),
            Err(e) => Err(PlatformError::VectorStoreError(
                format!("ChromaDB connection test failed: {}", e)
            )),
        }
    }
    
    fn provider_info(&self) -> VectorProviderInfo {
        VectorProviderInfo {
            name: "ChromaDB".to_string(),
            version: "0.4".to_string(),
            supports_namespaces: false, // ChromaDB uses collections instead
            supports_metadata_filtering: true,
            supports_hybrid_search: true,
            max_vector_dimension: 2048, // Typical limit, may vary
            max_batch_size: 5000,
        }
    }
}

impl ChromaDBStore {
    fn convert_filter(&self, filter: crate::domain::value_objects::SearchFilter) -> serde_json::Value {
        // Convert our generic filter format to ChromaDB's where clause format
        let mut chroma_filter = serde_json::Map::new();
        
        for condition in filter.conditions {
            let value = match condition.operator {
                crate::domain::value_objects::ComparisonOperator::Equal => {
                    serde_json::json!({ "$eq": condition.value })
                },
                crate::domain::value_objects::ComparisonOperator::NotEqual => {
                    serde_json::json!({ "$ne": condition.value })
                },
                crate::domain::value_objects::ComparisonOperator::GreaterThan => {
                    serde_json::json!({ "$gt": condition.value })
                },
                crate::domain::value_objects::ComparisonOperator::GreaterThanOrEqual => {
                    serde_json::json!({ "$gte": condition.value })
                },
                crate::domain::value_objects::ComparisonOperator::LessThan => {
                    serde_json::json!({ "$lt": condition.value })
                },
                crate::domain::value_objects::ComparisonOperator::LessThanOrEqual => {
                    serde_json::json!({ "$lte": condition.value })
                },
                crate::domain::value_objects::ComparisonOperator::In => {
                    serde_json::json!({ "$in": condition.value })
                },
                crate::domain::value_objects::ComparisonOperator::NotIn => {
                    serde_json::json!({ "$nin": condition.value })
                },
                crate::domain::value_objects::ComparisonOperator::Contains => {
                    serde_json::json!({ "$contains": condition.value })
                },
            };
            
            chroma_filter.insert(condition.field, value);
        }
        
        serde_json::Value::Object(chroma_filter)
    }
}

// ChromaDB API request/response structures

#[derive(Debug, Serialize)]
struct ChromaCreateCollectionRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    get_or_create: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ChromaCollection {
    name: String,
    id: String,
    metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize)]
struct ChromaAddRequest {
    ids: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    metadatas: Vec<Option<HashMap<String, serde_json::Value>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    documents: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct ChromaAddResponse {
    // ChromaDB add response is typically empty on success
}

#[derive(Debug, Serialize)]
struct ChromaQueryRequest {
    query_embeddings: Vec<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n_results: Option<u32>,
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    where_clause: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    where_document: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct ChromaQueryResponse {
    ids: Option<Vec<String>>,
    distances: Option<Vec<f32>>,
    metadatas: Option<Vec<Option<HashMap<String, serde_json::Value>>>>,
    embeddings: Option<Vec<Vec<f32>>>,
    documents: Option<Vec<Option<String>>>,
}

#[derive(Debug, Serialize)]
struct ChromaDeleteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    ids: Option<Vec<String>>,
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    where_clause: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ChromaDeleteResponse {
    // ChromaDB delete response is typically empty on success
}