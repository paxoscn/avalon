# Text to Vector Embeddings with PostgreSQL in Rust

This guide demonstrates how to convert text to vector embeddings using embedding models and store/search them in PostgreSQL with the pgvector extension.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Database Setup](#database-setup)
4. [Rust Dependencies](#rust-dependencies)
5. [Implementation](#implementation)
6. [Usage Examples](#usage-examples)
7. [Performance Optimization](#performance-optimization)

## Overview

Vector embeddings are numerical representations of text that capture semantic meaning. This enables:
- Semantic search (finding similar content by meaning, not just keywords)
- Recommendation systems
- Clustering and classification
- Question answering systems

## Prerequisites

- PostgreSQL 12+ with pgvector extension
- Rust 1.70+
- Access to an embedding model API (OpenAI, Cohere, etc.)

## Database Setup

### Install pgvector Extension

```sql
-- Connect to your database
CREATE EXTENSION IF NOT EXISTS vector;

-- Create a table for storing embeddings
CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    embedding vector(1024),  -- Qwen uses 1024 and OpenAI ada-002 uses 1536 dimensions
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Create an index for fast similarity search
-- Using HNSW (Hierarchical Navigable Small World) for better performance
CREATE INDEX ON documents USING hnsw (embedding vector_cosine_ops);

-- Alternative: IVFFlat index (faster build, slower query)
-- CREATE INDEX ON documents USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
```

### Migration Example

```rust
// In your migration file
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Enable pgvector extension
        manager
            .get_connection()
            .execute_unprepared("CREATE EXTENSION IF NOT EXISTS vector")
            .await?;

        // Create documents table
        manager
            .create_table(
                Table::create()
                    .table(Documents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Documents::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Documents::Content).text().not_null())
                    .col(ColumnDef::new(Documents::Metadata).json_binary())
                    .col(
                        ColumnDef::new(Documents::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Add vector column (pgvector type)
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE documents ADD COLUMN embedding vector(1536)"
            )
            .await?;

        // Create HNSW index
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX documents_embedding_idx ON documents USING hnsw (embedding vector_cosine_ops)"
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Documents::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Documents {
    Table,
    Id,
    Content,
    Metadata,
    CreatedAt,
}
```

## Rust Dependencies

Add these to your `Cargo.toml`:

```toml
[dependencies]
# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "json"] }
# Or use SeaORM
sea-orm = { version = "0.12", features = ["sqlx-postgres", "runtime-tokio-rustls"] }

# HTTP client for API calls
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"
thiserror = "1.0"
```

## Implementation

### 1. Embedding Service

```rust
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

pub struct EmbeddingService {
    client: Client,
    api_key: String,
    model: String,
}

impl EmbeddingService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: "text-embedding-ada-002".to_string(),
        }
    }

    /// Generate embedding vector for text
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            input: text.to_string(),
            model: self.model.clone(),
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let embedding_response: EmbeddingResponse = response.json().await?;
        
        Ok(embedding_response
            .data
            .first()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))?
            .embedding
            .clone())
    }

    /// Batch embed multiple texts
    pub async fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        
        // Process in batches to avoid rate limits
        for chunk in texts.chunks(20) {
            for text in chunk {
                let embedding = self.embed_text(text).await?;
                embeddings.push(embedding);
            }
            // Add delay to respect rate limits
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        Ok(embeddings)
    }
}
```

### 2. Vector Storage Repository

```rust
use sqlx::{PgPool, Row};
use serde_json::Value as JsonValue;

pub struct VectorRepository {
    pool: PgPool,
}

impl VectorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert document with embedding
    pub async fn insert_document(
        &self,
        content: &str,
        embedding: &[f32],
        metadata: Option<JsonValue>,
    ) -> Result<i32> {
        let embedding_str = format!("[{}]", 
            embedding.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let row = sqlx::query(
            r#"
            INSERT INTO documents (content, embedding, metadata)
            VALUES ($1, $2::vector, $3)
            RETURNING id
            "#,
        )
        .bind(content)
        .bind(embedding_str)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    /// Search for similar documents using cosine similarity
    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: i64,
        threshold: Option<f32>,
    ) -> Result<Vec<SearchResult>> {
        let embedding_str = format!("[{}]", 
            query_embedding.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let threshold_clause = threshold
            .map(|t| format!("AND (1 - (embedding <=> $1::vector)) >= {}", t))
            .unwrap_or_default();

        let query_str = format!(
            r#"
            SELECT 
                id,
                content,
                metadata,
                1 - (embedding <=> $1::vector) AS similarity
            FROM documents
            WHERE embedding IS NOT NULL
            {}
            ORDER BY embedding <=> $1::vector
            LIMIT $2
            "#,
            threshold_clause
        );

        let rows = sqlx::query(&query_str)
            .bind(embedding_str)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let results = rows
            .into_iter()
            .map(|row| SearchResult {
                id: row.get("id"),
                content: row.get("content"),
                metadata: row.get("metadata"),
                similarity: row.get("similarity"),
            })
            .collect();

        Ok(results)
    }

    /// Search with metadata filters
    pub async fn search_with_filters(
        &self,
        query_embedding: &[f32],
        filters: &JsonValue,
        limit: i64,
    ) -> Result<Vec<SearchResult>> {
        let embedding_str = format!("[{}]", 
            query_embedding.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                content,
                metadata,
                1 - (embedding <=> $1::vector) AS similarity
            FROM documents
            WHERE embedding IS NOT NULL
            AND metadata @> $2
            ORDER BY embedding <=> $1::vector
            LIMIT $3
            "#,
        )
        .bind(embedding_str)
        .bind(filters)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| SearchResult {
                id: row.get("id"),
                content: row.get("content"),
                metadata: row.get("metadata"),
                similarity: row.get("similarity"),
            })
            .collect();

        Ok(results)
    }

    /// Update document embedding
    pub async fn update_embedding(
        &self,
        id: i32,
        embedding: &[f32],
    ) -> Result<()> {
        let embedding_str = format!("[{}]", 
            embedding.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        sqlx::query(
            r#"
            UPDATE documents
            SET embedding = $1::vector
            WHERE id = $2
            "#,
        )
        .bind(embedding_str)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete document
    pub async fn delete_document(&self, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: i32,
    pub content: String,
    pub metadata: Option<JsonValue>,
    pub similarity: f32,
}
```

### 3. Vector Storage Service (Application Layer)

```rust
use anyhow::Result;

pub struct VectorStorageService {
    embedding_service: EmbeddingService,
    repository: VectorRepository,
}

impl VectorStorageService {
    pub fn new(
        embedding_service: EmbeddingService,
        repository: VectorRepository,
    ) -> Self {
        Self {
            embedding_service,
            repository,
        }
    }

    /// Store text with automatic embedding generation
    pub async fn store_text(
        &self,
        content: &str,
        metadata: Option<JsonValue>,
    ) -> Result<i32> {
        // Generate embedding
        let embedding = self.embedding_service.embed_text(content).await?;
        
        // Store in database
        let id = self.repository
            .insert_document(content, &embedding, metadata)
            .await?;
        
        Ok(id)
    }

    /// Semantic search by text query
    pub async fn semantic_search(
        &self,
        query: &str,
        limit: i64,
        threshold: Option<f32>,
    ) -> Result<Vec<SearchResult>> {
        // Generate query embedding
        let query_embedding = self.embedding_service.embed_text(query).await?;
        
        // Search similar documents
        let results = self.repository
            .search_similar(&query_embedding, limit, threshold)
            .await?;
        
        Ok(results)
    }

    /// Semantic search with metadata filters
    pub async fn semantic_search_filtered(
        &self,
        query: &str,
        filters: &JsonValue,
        limit: i64,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedding_service.embed_text(query).await?;
        
        let results = self.repository
            .search_with_filters(&query_embedding, filters, limit)
            .await?;
        
        Ok(results)
    }

    /// Batch store multiple texts
    pub async fn store_batch(
        &self,
        items: Vec<(String, Option<JsonValue>)>,
    ) -> Result<Vec<i32>> {
        let texts: Vec<&str> = items.iter().map(|(t, _)| t.as_str()).collect();
        let embeddings = self.embedding_service.embed_batch(texts).await?;
        
        let mut ids = Vec::new();
        for ((content, metadata), embedding) in items.iter().zip(embeddings.iter()) {
            let id = self.repository
                .insert_document(content, embedding, metadata.clone())
                .await?;
            ids.push(id);
        }
        
        Ok(ids)
    }
}
```

## Usage Examples

### Basic Usage

```rust
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup database connection
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://user:password@localhost/dbname")
        .await?;

    // Initialize services
    let embedding_service = EmbeddingService::new(
        std::env::var("OPENAI_API_KEY")?
    );
    let repository = VectorRepository::new(pool);
    let vector_service = VectorStorageService::new(
        embedding_service,
        repository,
    );

    // Store a document
    let doc_id = vector_service
        .store_text(
            "Rust is a systems programming language focused on safety and performance.",
            Some(serde_json::json!({
                "category": "programming",
                "language": "rust"
            })),
        )
        .await?;
    
    println!("Stored document with ID: {}", doc_id);

    // Semantic search
    let results = vector_service
        .semantic_search(
            "What is Rust programming language?",
            5,
            Some(0.7), // 70% similarity threshold
        )
        .await?;

    for result in results {
        println!(
            "ID: {}, Similarity: {:.2}%, Content: {}",
            result.id,
            result.similarity * 100.0,
            result.content
        );
    }

    Ok(())
}
```

### Batch Processing

```rust
async fn index_documents(vector_service: &VectorStorageService) -> Result<()> {
    let documents = vec![
        (
            "PostgreSQL is a powerful open-source relational database.".to_string(),
            Some(serde_json::json!({"category": "database"})),
        ),
        (
            "Vector embeddings enable semantic search capabilities.".to_string(),
            Some(serde_json::json!({"category": "ai"})),
        ),
        (
            "Tokio is an async runtime for Rust applications.".to_string(),
            Some(serde_json::json!({"category": "programming"})),
        ),
    ];

    let ids = vector_service.store_batch(documents).await?;
    println!("Indexed {} documents", ids.len());

    Ok(())
}
```

### Filtered Search

```rust
async fn search_by_category(
    vector_service: &VectorStorageService,
    query: &str,
    category: &str,
) -> Result<Vec<SearchResult>> {
    let filters = serde_json::json!({
        "category": category
    });

    let results = vector_service
        .semantic_search_filtered(query, &filters, 10)
        .await?;

    Ok(results)
}
```

## Performance Optimization

### 1. Index Selection

```sql
-- HNSW: Better query performance, slower build
CREATE INDEX ON documents USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);

-- IVFFlat: Faster build, good for smaller datasets
CREATE INDEX ON documents USING ivfflat (embedding vector_cosine_ops)
WITH (lists = 100);
```

### 2. Connection Pooling

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(30))
    .connect(&database_url)
    .await?;
```

### 3. Batch Operations

Process embeddings in batches to reduce API calls and database round-trips.

### 4. Caching

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CachedEmbeddingService {
    inner: EmbeddingService,
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
}

impl CachedEmbeddingService {
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(embedding) = cache.get(text) {
                return Ok(embedding.clone());
            }
        }

        // Generate and cache
        let embedding = self.inner.embed_text(text).await?;
        
        {
            let mut cache = self.cache.write().await;
            cache.insert(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }
}
```

### 5. Distance Metrics

```sql
-- Cosine distance (most common for embeddings)
SELECT * FROM documents ORDER BY embedding <=> '[...]' LIMIT 10;

-- L2 distance (Euclidean)
SELECT * FROM documents ORDER BY embedding <-> '[...]' LIMIT 10;

-- Inner product
SELECT * FROM documents ORDER BY embedding <#> '[...]' LIMIT 10;
```

## Best Practices

1. **Normalize embeddings**: Some models return normalized vectors; ensure consistency
2. **Choose appropriate dimensions**: Match your embedding model (e.g., 1536 for ada-002)
3. **Set similarity thresholds**: Filter out low-quality matches
4. **Use metadata filters**: Combine vector search with traditional filters
5. **Monitor costs**: Embedding API calls can be expensive at scale
6. **Implement retry logic**: Handle API rate limits and transient failures
7. **Version your embeddings**: Track which model version generated each embedding

## Troubleshooting

### Common Issues

**Issue**: Slow queries
- Solution: Ensure indexes are created and analyze query plans with `EXPLAIN ANALYZE`

**Issue**: Out of memory
- Solution: Reduce batch sizes or increase available memory

**Issue**: Low similarity scores
- Solution: Check embedding model consistency and consider re-embedding with better model

**Issue**: Rate limiting
- Solution: Implement exponential backoff and request batching

## Additional Resources

- [pgvector Documentation](https://github.com/pgvector/pgvector)
- [OpenAI Embeddings Guide](https://platform.openai.com/docs/guides/embeddings)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Vector Search Best Practices](https://www.pinecone.io/learn/vector-search/)
