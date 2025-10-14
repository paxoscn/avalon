use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::domain::value_objects::TenantId;

/// Vector record containing the vector data and metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorRecord {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub tenant_id: TenantId,
    pub namespace: Option<String>,
}

impl VectorRecord {
    pub fn new(
        id: String,
        vector: Vec<f32>,
        tenant_id: TenantId,
    ) -> Result<Self, String> {
        if id.trim().is_empty() {
            return Err("Vector record ID cannot be empty".to_string());
        }
        if vector.is_empty() {
            return Err("Vector cannot be empty".to_string());
        }
        
        Ok(VectorRecord {
            id: id.trim().to_string(),
            vector,
            metadata: HashMap::new(),
            tenant_id,
            namespace: None,
        })
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.namespace = Some(namespace);
        self
    }
    
    pub fn dimension(&self) -> usize {
        self.vector.len()
    }
}

/// Search query for vector similarity search
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    pub vector: Vec<f32>,
    pub top_k: usize,
    pub filter: Option<SearchFilter>,
    pub namespace: Option<String>,
    pub include_metadata: bool,
    pub include_values: bool,
}

impl SearchQuery {
    pub fn new(vector: Vec<f32>, top_k: usize) -> Result<Self, String> {
        if vector.is_empty() {
            return Err("Search vector cannot be empty".to_string());
        }
        if top_k == 0 {
            return Err("top_k must be greater than 0".to_string());
        }
        if top_k > 10000 {
            return Err("top_k cannot exceed 10000".to_string());
        }
        
        Ok(SearchQuery {
            vector,
            top_k,
            filter: None,
            namespace: None,
            include_metadata: true,
            include_values: false,
        })
    }
    
    pub fn with_filter(mut self, filter: SearchFilter) -> Self {
        self.filter = Some(filter);
        self
    }
    
    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.namespace = Some(namespace);
        self
    }
    
    pub fn include_values(mut self, include: bool) -> Self {
        self.include_values = include;
        self
    }
    
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }
}

/// Filter for vector search
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchFilter {
    pub conditions: Vec<FilterCondition>,
    pub operator: FilterOperator,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterCondition {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    In,
    NotIn,
    Contains,
}

/// Search result from vector similarity search
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub vector: Option<Vec<f32>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl SearchResult {
    pub fn new(id: String, score: f32) -> Self {
        SearchResult {
            id,
            score,
            vector: None,
            metadata: None,
        }
    }
    
    pub fn with_vector(mut self, vector: Vec<f32>) -> Self {
        self.vector = Some(vector);
        self
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Index configuration for vector storage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexConfig {
    pub name: String,
    pub dimension: usize,
    pub metric: DistanceMetric,
    pub pod_type: Option<String>,
    pub replicas: Option<i32>,
    pub shards: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

impl IndexConfig {
    pub fn new(name: String, dimension: usize, metric: DistanceMetric) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Index name cannot be empty".to_string());
        }
        if dimension == 0 {
            return Err("Dimension must be greater than 0".to_string());
        }
        
        Ok(IndexConfig {
            name: name.trim().to_string(),
            dimension,
            metric,
            pod_type: None,
            replicas: None,
            shards: None,
        })
    }
    
    pub fn with_pod_type(mut self, pod_type: String) -> Self {
        self.pod_type = Some(pod_type);
        self
    }
    
    pub fn with_replicas(mut self, replicas: i32) -> Self {
        self.replicas = Some(replicas);
        self
    }
    
    pub fn with_shards(mut self, shards: i32) -> Self {
        self.shards = Some(shards);
        self
    }
}

/// Vector storage statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorStats {
    pub total_vectors: u64,
    pub dimension: usize,
    pub index_fullness: f32,
    pub namespace_stats: HashMap<String, NamespaceStats>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamespaceStats {
    pub vector_count: u64,
}

/// Batch operation for vector storage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchOperation {
    pub upsert: Vec<VectorRecord>,
    pub delete: Vec<String>,
}

impl BatchOperation {
    pub fn new() -> Self {
        BatchOperation {
            upsert: Vec::new(),
            delete: Vec::new(),
        }
    }
    
    pub fn add_upsert(mut self, record: VectorRecord) -> Self {
        self.upsert.push(record);
        self
    }
    
    pub fn add_delete(mut self, id: String) -> Self {
        self.delete.push(id);
        self
    }
    
    pub fn is_empty(&self) -> bool {
        self.upsert.is_empty() && self.delete.is_empty()
    }
}

impl Default for BatchOperation {
    fn default() -> Self {
        Self::new()
    }
}