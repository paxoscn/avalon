pub mod providers;
pub mod error_handling;

use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::value_objects::{
    VectorRecord, SearchQuery, SearchResult, IndexConfig, VectorStats, BatchOperation
};
use crate::error::PlatformError;

/// Vector store trait for different vector database providers
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Store a single vector record
    async fn upsert(&self, record: VectorRecord) -> Result<(), PlatformError>;
    
    /// Store multiple vector records in batch
    async fn upsert_batch(&self, records: Vec<VectorRecord>) -> Result<(), PlatformError>;
    
    /// Search for similar vectors
    async fn query(&self, query: SearchQuery) -> Result<Vec<SearchResult>, PlatformError>;
    
    /// Delete vectors by IDs
    async fn delete(&self, ids: Vec<String>, namespace: Option<String>) -> Result<(), PlatformError>;
    
    /// Execute batch operations (upsert and delete)
    async fn execute_batch(&self, operation: BatchOperation) -> Result<(), PlatformError>;
    
    /// Create or update index configuration
    async fn create_index(&self, config: IndexConfig) -> Result<(), PlatformError>;
    
    /// Delete an index
    async fn delete_index(&self, index_name: String) -> Result<(), PlatformError>;
    
    /// List all indexes
    async fn list_indexes(&self) -> Result<Vec<String>, PlatformError>;
    
    /// Get vector storage statistics
    async fn get_stats(&self, namespace: Option<String>) -> Result<VectorStats, PlatformError>;
    
    /// Test connection to vector store
    async fn test_connection(&self) -> Result<(), PlatformError>;
    
    /// Get provider information
    fn provider_info(&self) -> VectorProviderInfo;
}

/// Information about a vector store provider
#[derive(Debug, Clone)]
pub struct VectorProviderInfo {
    pub name: String,
    pub version: String,
    pub supports_namespaces: bool,
    pub supports_metadata_filtering: bool,
    pub supports_hybrid_search: bool,
    pub max_vector_dimension: usize,
    pub max_batch_size: usize,
}

/// Vector store configuration
#[derive(Debug, Clone)]
pub struct VectorStoreConfig {
    pub provider: VectorProvider,
    pub connection_params: HashMap<String, String>,
    pub default_namespace: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

/// Supported vector store providers
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum VectorProvider {
    Pinecone,
    ChromaDB,
    Weaviate,
    Qdrant,
    Milvus,
}

impl VectorProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            VectorProvider::Pinecone => "pinecone",
            VectorProvider::ChromaDB => "chromadb",
            VectorProvider::Weaviate => "weaviate",
            VectorProvider::Qdrant => "qdrant",
            VectorProvider::Milvus => "milvus",
        }
    }
    
    pub fn from_str(s: &str) -> Result<Self, PlatformError> {
        match s.to_lowercase().as_str() {
            "pinecone" => Ok(VectorProvider::Pinecone),
            "chromadb" | "chroma" => Ok(VectorProvider::ChromaDB),
            "weaviate" => Ok(VectorProvider::Weaviate),
            "qdrant" => Ok(VectorProvider::Qdrant),
            "milvus" => Ok(VectorProvider::Milvus),
            _ => Err(PlatformError::ValidationError(
                format!("Unsupported vector provider: {}", s)
            )),
        }
    }
}

/// Vector store factory for creating provider instances
pub struct VectorStoreFactory;

impl VectorStoreFactory {
    /// Create a vector store instance based on configuration
    pub async fn create_store(config: VectorStoreConfig) -> Result<Box<dyn VectorStore>, PlatformError> {
        match config.provider {
            VectorProvider::Pinecone => {
                let provider = providers::pinecone::PineconeStore::new(config).await?;
                Ok(Box::new(provider))
            },
            VectorProvider::ChromaDB => {
                let provider = providers::chromadb::ChromaDBStore::new(config).await?;
                Ok(Box::new(provider))
            },
            VectorProvider::Weaviate => {
                let provider = providers::weaviate::WeaviateStore::new(config).await?;
                Ok(Box::new(provider))
            },
            VectorProvider::Qdrant => {
                let provider = providers::qdrant::QdrantStore::new(config).await?;
                Ok(Box::new(provider))
            },
            VectorProvider::Milvus => {
                let provider = providers::milvus::MilvusStore::new(config).await?;
                Ok(Box::new(provider))
            },
        }
    }
    
    /// Create a vector store registry with multiple providers
    pub async fn create_registry(configs: Vec<VectorStoreConfig>) -> Result<VectorStoreRegistry, PlatformError> {
        let mut registry = VectorStoreRegistry::new();
        
        for config in configs {
            let provider_name = config.provider.as_str().to_string();
            let store = Self::create_store(config).await?;
            registry.register_store(provider_name, store);
        }
        
        Ok(registry)
    }
}

/// Registry for managing multiple vector store providers
pub struct VectorStoreRegistry {
    stores: HashMap<String, Box<dyn VectorStore>>,
    default_store: Option<String>,
}

impl VectorStoreRegistry {
    pub fn new() -> Self {
        Self {
            stores: HashMap::new(),
            default_store: None,
        }
    }
    
    pub fn register_store(&mut self, name: String, store: Box<dyn VectorStore>) {
        self.stores.insert(name, store);
    }
    
    pub fn set_default_store(&mut self, name: String) -> Result<(), PlatformError> {
        if self.stores.contains_key(&name) {
            self.default_store = Some(name);
            Ok(())
        } else {
            Err(PlatformError::NotFound(format!("Vector store '{}' not found", name)))
        }
    }
    
    pub fn get_store(&self, name: &str) -> Result<&dyn VectorStore, PlatformError> {
        self.stores.get(name)
            .map(|store| store.as_ref())
            .ok_or_else(|| PlatformError::NotFound(format!("Vector store '{}' not found", name)))
    }
    
    pub fn get_default_store(&self) -> Result<&dyn VectorStore, PlatformError> {
        match &self.default_store {
            Some(name) => self.get_store(name),
            None => Err(PlatformError::ValidationError("No default vector store configured".to_string())),
        }
    }
    
    pub fn list_stores(&self) -> Vec<String> {
        self.stores.keys().cloned().collect()
    }
    
    pub async fn test_all_connections(&self) -> HashMap<String, Result<(), PlatformError>> {
        let mut results = HashMap::new();
        
        for (name, store) in &self.stores {
            let result = store.test_connection().await;
            results.insert(name.clone(), result);
        }
        
        results
    }
}

impl Default for VectorStoreRegistry {
    fn default() -> Self {
        Self::new()
    }
}