# Rust LLM Integration Guide

A comprehensive guide to integrating and communicating with Large Language Models (LLMs) in Rust.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Core Concepts](#core-concepts)
4. [Basic Usage](#basic-usage)
5. [Advanced Features](#advanced-features)
6. [Error Handling](#error-handling)
7. [Best Practices](#best-practices)
8. [Examples](#examples)

## Overview

This guide demonstrates how to build a robust LLM integration system in Rust that supports:
- Multiple LLM providers (OpenAI, Anthropic, etc.)
- Streaming responses
- Error handling and retries
- Circuit breakers and health monitoring
- Load balancing and failover
- Token counting and cost estimation

## Architecture

### Layer Structure

```
┌─────────────────────────────────────┐
│   Application Layer                 │
│   (Business Logic)                  │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│   Integration Service               │
│   (Orchestration & Config)          │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│   Domain Service                    │
│   (Provider Abstraction)            │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│   Infrastructure Layer              │
│   (Provider Implementations)        │
└─────────────────────────────────────┘
```


## Core Concepts

### 1. Provider Trait

Define a common interface for all LLM providers:

```rust
use async_trait::async_trait;
use futures::Stream;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Send a chat completion request
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, LLMError>;
    
    /// Generate embeddings for text
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, LLMError>;
    
    /// Stream chat completion responses
    async fn stream_chat_completion(
        &self,
        request: ChatRequest,
    ) -> Result<Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>, LLMError>;
    
    /// Get available models
    fn get_model_info(&self) -> Vec<ModelInfo>;
    
    /// Check if streaming is supported
    fn supports_streaming(&self) -> bool;
    
    /// Test connection to provider
    async fn test_connection(&self) -> Result<ConnectionTestResult, LLMError>;
}
```

### 2. Request/Response Types

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub stream: bool,
    pub tenant_id: Uuid,
    pub response_format: Option<ResponseFormat>,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub model_used: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: MessageContent,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone)]
pub enum MessageContent {
    Text(String),
    Multimodal(Vec<ContentPart>),
}

#[derive(Debug, Clone)]
pub enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}
```


## Basic Usage

### 1. Creating a Provider

```rust
use crate::infrastructure::llm::providers::openai::OpenAIProvider;

// Create an OpenAI provider
let provider = OpenAIProvider::new(
    "sk-your-api-key".to_string(),
    Some("https://api.openai.com/v1".to_string()),
)?;

// Test the connection
let test_result = provider.test_connection().await?;
if test_result.success {
    println!("Connected successfully in {}ms", test_result.response_time_ms);
}
```

### 2. Simple Chat Completion

```rust
use crate::domain::value_objects::{ChatMessage, MessageRole, MessageContent};
use chrono::Utc;

// Prepare messages
let messages = vec![
    ChatMessage {
        role: MessageRole::System,
        content: MessageContent::Text("You are a helpful assistant.".to_string()),
        metadata: None,
        timestamp: Utc::now(),
    },
    ChatMessage {
        role: MessageRole::User,
        content: MessageContent::Text("What is Rust?".to_string()),
        metadata: None,
        timestamp: Utc::now(),
    },
];

// Create request
let request = ChatRequest {
    messages,
    model: "gpt-3.5-turbo".to_string(),
    temperature: Some(0.7),
    max_tokens: Some(1000),
    top_p: Some(1.0),
    frequency_penalty: None,
    presence_penalty: None,
    stop_sequences: None,
    stream: false,
    tenant_id: Uuid::new_v4(),
    response_format: None,
};

// Send request
let response = provider.chat_completion(request).await?;
println!("Response: {}", response.content);
println!("Tokens used: {}", response.usage.total_tokens);
```

### 3. Streaming Responses

```rust
use futures::StreamExt;

// Create streaming request
let mut request = ChatRequest {
    messages: vec![/* your messages */],
    model: "gpt-3.5-turbo".to_string(),
    stream: true,
    // ... other fields
};

// Get stream
let mut stream = provider.stream_chat_completion(request).await?;

// Process chunks as they arrive
while let Some(chunk_result) = stream.next().await {
    match chunk_result {
        Ok(chunk) => {
            if let Some(content) = chunk.content {
                print!("{}", content);
                std::io::stdout().flush()?;
            }
        }
        Err(e) => eprintln!("Stream error: {}", e),
    }
}
println!(); // New line after streaming
```


### 4. Generating Embeddings

```rust
// Generate embeddings for text
let text = "Rust is a systems programming language";
let embedding = provider.generate_embedding(text).await?;

println!("Embedding dimension: {}", embedding.len());
// Typically 1536 for OpenAI's text-embedding-ada-002
```

### 5. Multimodal Messages (Vision)

```rust
use crate::domain::value_objects::chat_message::{ContentPart, ImageUrl};

let messages = vec![
    ChatMessage {
        role: MessageRole::User,
        content: MessageContent::Multimodal(vec![
            ContentPart::Text {
                text: "What's in this image?".to_string(),
            },
            ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: "https://example.com/image.jpg".to_string(),
                    detail: Some("high".to_string()),
                },
            },
        ]),
        metadata: None,
        timestamp: Utc::now(),
    },
];

let request = ChatRequest {
    messages,
    model: "gpt-4-vision-preview".to_string(),
    // ... other fields
};

let response = provider.chat_completion(request).await?;
```

## Advanced Features

### 1. Integrated Service with Load Balancing

```rust
use crate::application::services::integrated_llm_service::{
    IntegratedLLMService, IntegratedLLMConfig, LoadBalancingStrategy
};
use std::sync::Arc;

// Create provider registry
let provider_registry = Arc::new(LLMProviderRegistry::new());

// Configure integrated service
let config = IntegratedLLMConfig {
    load_balancing_strategy: LoadBalancingStrategy::HealthBased,
    health_check_interval: Duration::from_secs(60),
    max_retries: 3,
    circuit_breaker_threshold: 5,
    circuit_breaker_timeout: Duration::from_secs(30),
    enable_fallback: true,
    fallback_providers: vec!["openai".to_string(), "anthropic".to_string()],
    request_timeout: Duration::from_secs(30),
};

// Create integrated service
let service = IntegratedLLMService::new(provider_registry, config);

// Initialize health monitoring
service.initialize_health_monitoring().await;

// Use the service (automatically handles provider selection)
let response = service.chat_completion(
    &model_config,
    messages,
    tenant_id,
    None,
).await?;
```


### 2. Load Balancing Strategies

```rust
// Round Robin - Distribute requests evenly
let strategy = LoadBalancingStrategy::RoundRobin;

// Random - Random provider selection
let strategy = LoadBalancingStrategy::Random;

// Weighted Random - Prefer certain providers
let mut weights = HashMap::new();
weights.insert("openai".to_string(), 0.7);
weights.insert("anthropic".to_string(), 0.3);
let strategy = LoadBalancingStrategy::WeightedRandom(weights);

// Health Based - Use healthiest provider
let strategy = LoadBalancingStrategy::HealthBased;

// Response Time Based - Use fastest provider
let strategy = LoadBalancingStrategy::ResponseTimeBased;
```

### 3. Health Monitoring

```rust
// Get health status of all providers
let health_status = service.get_provider_health_status().await;

for (provider_name, health) in health_status {
    println!("Provider: {}", provider_name);
    println!("  Healthy: {}", health.is_healthy);
    println!("  Response Time: {}ms", health.response_time_ms);
    println!("  Success Rate: {}/{}", 
        health.success_count, 
        health.success_count + health.error_count
    );
}

// Run health checks manually
service.run_health_checks().await;

// Set provider health manually (for testing)
service.set_provider_health("openai", false).await;
```

### 4. High Availability Setup

```rust
use crate::application::services::llm_integration_service::{
    LLMIntegrationService, LLMIntegrationServiceBuilder
};

// Create high availability service
let service = LLMIntegrationService::new_high_availability(
    config_repository
).await?;

// Or use builder for custom configuration
let service = LLMIntegrationServiceBuilder::new()
    .with_config_repository(config_repository)
    .with_load_balancing_strategy(LoadBalancingStrategy::HealthBased)
    .with_health_monitoring(true, Duration::from_secs(30))
    .with_auto_failover(true)
    .with_request_timeout(Duration::from_secs(15))
    .with_caching(true, Duration::from_secs(600))
    .build()
    .await?;

// Start background health monitoring
service.start_health_monitoring();

// Use the service
let response = service.chat_completion_with_default_config(
    tenant_id,
    messages,
).await?;
```


## Error Handling

### 1. Error Types

```rust
#[derive(Debug, Clone)]
pub enum LLMError {
    // Configuration errors
    InvalidConfiguration(String),
    ModelNotFound(String),
    
    // Authentication errors
    AuthenticationFailed(String),
    
    // Rate limiting
    RateLimitExceeded(String),
    
    // Network errors
    NetworkError(String),
    
    // Provider errors
    ProviderError(String),
    InternalError(String),
    
    // Serialization errors
    SerializationError(String),
}

impl std::fmt::Display for LLMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            LLMError::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            LLMError::RateLimitExceeded(msg) => write!(f, "Rate limit exceeded: {}", msg),
            LLMError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            // ... other variants
        }
    }
}

impl std::error::Error for LLMError {}
```

### 2. Retry Logic

```rust
use crate::infrastructure::llm::error_handling::{RetryWrapper, RetryConfig};

// Configure retry behavior
let retry_config = RetryConfig {
    max_attempts: 3,
    base_delay: Duration::from_millis(1000),
    max_delay: Duration::from_secs(10),
    backoff_multiplier: 2.0,
    retryable_errors: vec![
        RetryableErrorType::RateLimit,
        RetryableErrorType::NetworkError,
        RetryableErrorType::InternalServerError,
    ],
};

let retry_wrapper = RetryWrapper::new(retry_config);

// Execute with automatic retries
let result = retry_wrapper.execute_with_retry(|| async {
    provider.chat_completion(request.clone()).await
}).await?;
```

### 3. Circuit Breaker

```rust
use crate::infrastructure::llm::error_handling::CircuitBreaker;

// Create circuit breaker
let circuit_breaker = CircuitBreaker::new(
    5,  // failure threshold
    Duration::from_secs(30),  // recovery timeout
);

// Execute with circuit breaker protection
let result = circuit_breaker.execute(|| async {
    provider.chat_completion(request.clone()).await
}).await?;

// Circuit breaker states:
// - Closed: Normal operation
// - Open: Too many failures, requests blocked
// - HalfOpen: Testing if service recovered
```


### 4. Error Mapping

```rust
use crate::infrastructure::llm::error_handling::ErrorMapper;

// Map HTTP errors to LLM errors
let llm_error = ErrorMapper::map_http_error(429, "Rate limit exceeded");
// Returns: LLMError::RateLimitExceeded

// Map network errors
let llm_error = ErrorMapper::map_network_error("connection timeout");
// Returns: LLMError::NetworkError

// Map serialization errors
let llm_error = ErrorMapper::map_serialization_error("invalid JSON");
// Returns: LLMError::SerializationError
```

### 5. Graceful Error Handling

```rust
async fn handle_llm_request(
    service: &IntegratedLLMService,
    messages: Vec<ChatMessage>,
) -> Result<String, String> {
    match service.chat_completion(&config, messages, tenant_id, None).await {
        Ok(response) => Ok(response.content),
        Err(LLMError::RateLimitExceeded(msg)) => {
            log::warn!("Rate limited: {}", msg);
            Err("Service temporarily unavailable. Please try again later.".to_string())
        }
        Err(LLMError::AuthenticationFailed(msg)) => {
            log::error!("Auth failed: {}", msg);
            Err("Authentication error. Please check your API key.".to_string())
        }
        Err(LLMError::NetworkError(msg)) => {
            log::error!("Network error: {}", msg);
            Err("Network error. Please check your connection.".to_string())
        }
        Err(e) => {
            log::error!("Unexpected error: {}", e);
            Err("An unexpected error occurred.".to_string())
        }
    }
}
```

## Best Practices

### 1. Configuration Management

```rust
use std::env;

// Load configuration from environment
pub struct LLMConfig {
    pub openai_api_key: String,
    pub anthropic_api_key: String,
    pub default_model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl LLMConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            openai_api_key: env::var("OPENAI_API_KEY")
                .map_err(|_| "OPENAI_API_KEY not set")?,
            anthropic_api_key: env::var("ANTHROPIC_API_KEY")
                .map_err(|_| "ANTHROPIC_API_KEY not set")?,
            default_model: env::var("DEFAULT_MODEL")
                .unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
            max_tokens: env::var("MAX_TOKENS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            temperature: env::var("TEMPERATURE")
                .unwrap_or_else(|_| "0.7".to_string())
                .parse()
                .unwrap_or(0.7),
        })
    }
}
```


### 2. Token Management

```rust
// Estimate token count before sending
let token_count = service.estimate_token_count(&messages, "gpt-3.5-turbo")?;
println!("Estimated tokens: {}", token_count);

// Check if within limits
const MAX_TOKENS: u32 = 4096;
if token_count > MAX_TOKENS {
    return Err("Message too long".into());
}

// Track usage after response
let response = provider.chat_completion(request).await?;
println!("Actual tokens used:");
println!("  Prompt: {}", response.usage.prompt_tokens);
println!("  Completion: {}", response.usage.completion_tokens);
println!("  Total: {}", response.usage.total_tokens);

// Calculate cost (example for GPT-3.5-turbo)
let cost = (response.usage.prompt_tokens as f64 * 0.0015 / 1000.0)
    + (response.usage.completion_tokens as f64 * 0.002 / 1000.0);
println!("Estimated cost: ${:.6}", cost);
```

### 3. Async Best Practices

```rust
use tokio::time::timeout;

// Set timeout for requests
let response = timeout(
    Duration::from_secs(30),
    provider.chat_completion(request)
).await
    .map_err(|_| LLMError::NetworkError("Request timeout".to_string()))??;

// Process multiple requests concurrently
use futures::future::join_all;

let requests = vec![request1, request2, request3];
let futures: Vec<_> = requests
    .into_iter()
    .map(|req| provider.chat_completion(req))
    .collect();

let results = join_all(futures).await;

// Handle results
for (i, result) in results.into_iter().enumerate() {
    match result {
        Ok(response) => println!("Request {}: {}", i, response.content),
        Err(e) => eprintln!("Request {} failed: {}", i, e),
    }
}
```

### 4. Logging and Monitoring

```rust
use log::{info, warn, error, debug};

async fn chat_with_logging(
    provider: &dyn LLMProvider,
    request: ChatRequest,
) -> Result<ChatResponse, LLMError> {
    let start = std::time::Instant::now();
    
    info!("Starting LLM request: model={}, messages={}", 
        request.model, request.messages.len());
    debug!("Request details: {:?}", request);
    
    let result = provider.chat_completion(request).await;
    
    let elapsed = start.elapsed();
    
    match &result {
        Ok(response) => {
            info!("LLM request successful: duration={:?}, tokens={}", 
                elapsed, response.usage.total_tokens);
            debug!("Response: {:?}", response);
        }
        Err(e) => {
            error!("LLM request failed: duration={:?}, error={}", elapsed, e);
        }
    }
    
    result
}
```


### 5. Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    // Mock provider for testing
    mock! {
        pub LLMProvider {}
        
        #[async_trait]
        impl LLMProvider for LLMProvider {
            async fn chat_completion(&self, request: ChatRequest) 
                -> Result<ChatResponse, LLMError>;
            async fn generate_embedding(&self, text: &str) 
                -> Result<Vec<f32>, LLMError>;
            async fn stream_chat_completion(&self, request: ChatRequest)
                -> Result<Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>, LLMError>;
            fn get_model_info(&self) -> Vec<ModelInfo>;
            fn supports_streaming(&self) -> bool;
            async fn test_connection(&self) -> Result<ConnectionTestResult, LLMError>;
        }
    }

    #[tokio::test]
    async fn test_chat_completion() {
        let mut mock_provider = MockLLMProvider::new();
        
        mock_provider
            .expect_chat_completion()
            .times(1)
            .returning(|_| {
                Ok(ChatResponse {
                    content: "Hello!".to_string(),
                    model_used: "gpt-3.5-turbo".to_string(),
                    usage: TokenUsage {
                        prompt_tokens: 10,
                        completion_tokens: 5,
                        total_tokens: 15,
                    },
                    finish_reason: FinishReason::Stop,
                    metadata: None,
                })
            });

        let request = ChatRequest {
            messages: vec![/* test messages */],
            model: "gpt-3.5-turbo".to_string(),
            // ... other fields
        };

        let response = mock_provider.chat_completion(request).await.unwrap();
        assert_eq!(response.content, "Hello!");
        assert_eq!(response.usage.total_tokens, 15);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let mut mock_provider = MockLLMProvider::new();
        
        mock_provider
            .expect_chat_completion()
            .times(1)
            .returning(|_| {
                Err(LLMError::RateLimitExceeded("Rate limited".to_string()))
            });

        let request = ChatRequest {
            messages: vec![/* test messages */],
            model: "gpt-3.5-turbo".to_string(),
            // ... other fields
        };

        let result = mock_provider.chat_completion(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LLMError::RateLimitExceeded(_)));
    }
}
```


## Examples

### Example 1: Simple Chatbot

```rust
use tokio;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize provider
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let provider = OpenAIProvider::new(api_key, None)?;
    
    println!("Chatbot started! Type 'quit' to exit.");
    
    let mut conversation: Vec<ChatMessage> = vec![
        ChatMessage {
            role: MessageRole::System,
            content: MessageContent::Text(
                "You are a helpful assistant.".to_string()
            ),
            metadata: None,
            timestamp: chrono::Utc::now(),
        },
    ];
    
    loop {
        print!("You: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input == "quit" {
            break;
        }
        
        // Add user message
        conversation.push(ChatMessage {
            role: MessageRole::User,
            content: MessageContent::Text(input.to_string()),
            metadata: None,
            timestamp: chrono::Utc::now(),
        });
        
        // Get response
        let request = ChatRequest {
            messages: conversation.clone(),
            model: "gpt-3.5-turbo".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(500),
            stream: false,
            tenant_id: uuid::Uuid::new_v4(),
            // ... other fields with None
        };
        
        match provider.chat_completion(request).await {
            Ok(response) => {
                println!("Assistant: {}", response.content);
                
                // Add assistant response to conversation
                conversation.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: MessageContent::Text(response.content),
                    metadata: None,
                    timestamp: chrono::Utc::now(),
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
    
    Ok(())
}
```


### Example 2: Streaming Chatbot

```rust
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let provider = OpenAIProvider::new(api_key, None)?;
    
    let messages = vec![
        ChatMessage {
            role: MessageRole::User,
            content: MessageContent::Text(
                "Write a short story about a robot.".to_string()
            ),
            metadata: None,
            timestamp: chrono::Utc::now(),
        },
    ];
    
    let request = ChatRequest {
        messages,
        model: "gpt-3.5-turbo".to_string(),
        temperature: Some(0.8),
        max_tokens: Some(1000),
        stream: true,
        tenant_id: uuid::Uuid::new_v4(),
        // ... other fields
    };
    
    println!("Assistant: ");
    
    let mut stream = provider.stream_chat_completion(request).await?;
    let mut full_response = String::new();
    
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if let Some(content) = chunk.content {
                    print!("{}", content);
                    io::stdout().flush()?;
                    full_response.push_str(&content);
                }
            }
            Err(e) => {
                eprintln!("\nStream error: {}", e);
                break;
            }
        }
    }
    
    println!("\n\nFull response length: {} characters", full_response.len());
    
    Ok(())
}
```

### Example 3: Multi-Provider with Fallback

```rust
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create provider registry
    let registry = Arc::new(LLMProviderRegistry::new());
    
    // Configure integrated service with fallback
    let config = IntegratedLLMConfig {
        load_balancing_strategy: LoadBalancingStrategy::HealthBased,
        enable_fallback: true,
        fallback_providers: vec![
            "openai".to_string(),
            "anthropic".to_string(),
        ],
        max_retries: 3,
        ..Default::default()
    };
    
    let service = IntegratedLLMService::new(registry, config);
    service.initialize_health_monitoring().await;
    
    // Prepare request
    let messages = vec![
        ChatMessage {
            role: MessageRole::User,
            content: MessageContent::Text(
                "Explain quantum computing in simple terms.".to_string()
            ),
            metadata: None,
            timestamp: chrono::Utc::now(),
        },
    ];
    
    let model_config = ModelConfig {
        provider: Provider::OpenAI,
        model_name: "gpt-3.5-turbo".to_string(),
        // ... other config
    };
    
    // Service automatically handles provider selection and fallback
    match service.chat_completion(
        &model_config,
        messages,
        uuid::Uuid::new_v4(),
        None,
    ).await {
        Ok(response) => {
            println!("Response from {}: {}", 
                response.model_used, 
                response.content
            );
        }
        Err(e) => {
            eprintln!("All providers failed: {}", e);
        }
    }
    
    Ok(())
}
```


### Example 4: Batch Processing with Concurrency

```rust
use futures::future::join_all;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let provider = Arc::new(OpenAIProvider::new(api_key, None)?);
    
    // Limit concurrent requests
    let semaphore = Arc::new(Semaphore::new(5));
    
    let texts = vec![
        "Summarize the benefits of Rust",
        "Explain async/await in Rust",
        "What are Rust's ownership rules?",
        "Describe Rust's type system",
        "How does Rust handle memory safety?",
    ];
    
    let futures: Vec<_> = texts
        .into_iter()
        .map(|text| {
            let provider = provider.clone();
            let semaphore = semaphore.clone();
            
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                let messages = vec![
                    ChatMessage {
                        role: MessageRole::User,
                        content: MessageContent::Text(text.to_string()),
                        metadata: None,
                        timestamp: chrono::Utc::now(),
                    },
                ];
                
                let request = ChatRequest {
                    messages,
                    model: "gpt-3.5-turbo".to_string(),
                    temperature: Some(0.7),
                    max_tokens: Some(200),
                    stream: false,
                    tenant_id: uuid::Uuid::new_v4(),
                    // ... other fields
                };
                
                (text, provider.chat_completion(request).await)
            }
        })
        .collect();
    
    let results = join_all(futures).await;
    
    for (text, result) in results {
        match result {
            Ok(response) => {
                println!("Question: {}", text);
                println!("Answer: {}", response.content);
                println!("Tokens: {}\n", response.usage.total_tokens);
            }
            Err(e) => {
                eprintln!("Failed for '{}': {}\n", text, e);
            }
        }
    }
    
    Ok(())
}
```

### Example 5: Structured Output with JSON Schema

```rust
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let provider = OpenAIProvider::new(api_key, None)?;
    
    // Define JSON schema for structured output
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "number" },
            "occupation": { "type": "string" },
            "hobbies": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["name", "age", "occupation"],
        "additionalProperties": false
    });
    
    let messages = vec![
        ChatMessage {
            role: MessageRole::User,
            content: MessageContent::Text(
                "Generate a profile for a fictional software engineer.".to_string()
            ),
            metadata: None,
            timestamp: chrono::Utc::now(),
        },
    ];
    
    let request = ChatRequest {
        messages,
        model: "gpt-4-turbo-preview".to_string(),
        temperature: Some(0.7),
        max_tokens: Some(500),
        stream: false,
        tenant_id: uuid::Uuid::new_v4(),
        response_format: Some(ResponseFormat {
            format_type: "json_schema".to_string(),
            json_schema: Some(JsonSchema {
                name: "person_profile".to_string(),
                strict: true,
                schema,
            }),
        }),
        // ... other fields
    };
    
    let response = provider.chat_completion(request).await?;
    
    // Parse structured response
    let profile: serde_json::Value = serde_json::from_str(&response.content)?;
    println!("Generated profile:");
    println!("{}", serde_json::to_string_pretty(&profile)?);
    
    Ok(())
}
```


### Example 6: Embeddings and Semantic Search

```rust
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let provider = OpenAIProvider::new(api_key, None)?;
    
    // Documents to search
    let documents = vec![
        "Rust is a systems programming language focused on safety and performance.",
        "Python is a high-level programming language known for its simplicity.",
        "JavaScript is the language of the web, running in browsers.",
        "Go is a statically typed language designed for building scalable systems.",
    ];
    
    // Generate embeddings for all documents
    println!("Generating embeddings...");
    let mut doc_embeddings: HashMap<String, Vec<f32>> = HashMap::new();
    
    for doc in &documents {
        let embedding = provider.generate_embedding(doc).await?;
        doc_embeddings.insert(doc.to_string(), embedding);
    }
    
    // Query
    let query = "What language is good for web development?";
    let query_embedding = provider.generate_embedding(query).await?;
    
    // Calculate cosine similarity
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot_product / (magnitude_a * magnitude_b)
    }
    
    // Find most similar document
    let mut similarities: Vec<(String, f32)> = doc_embeddings
        .iter()
        .map(|(doc, emb)| {
            (doc.clone(), cosine_similarity(&query_embedding, emb))
        })
        .collect();
    
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    println!("\nQuery: {}", query);
    println!("\nMost similar documents:");
    for (doc, similarity) in similarities.iter().take(3) {
        println!("  Similarity: {:.4} - {}", similarity, doc);
    }
    
    Ok(())
}
```

## Performance Optimization

### 1. Connection Pooling

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProviderPool {
    providers: Arc<RwLock<Vec<Arc<dyn LLMProvider>>>>,
    max_size: usize,
}

impl ProviderPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            providers: Arc::new(RwLock::new(Vec::new())),
            max_size,
        }
    }
    
    pub async fn get_provider(&self) -> Option<Arc<dyn LLMProvider>> {
        let mut providers = self.providers.write().await;
        providers.pop()
    }
    
    pub async fn return_provider(&self, provider: Arc<dyn LLMProvider>) {
        let mut providers = self.providers.write().await;
        if providers.len() < self.max_size {
            providers.push(provider);
        }
    }
}
```


### 2. Response Caching

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct ResponseCache {
    cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    ttl: Duration,
}

struct CachedResponse {
    response: ChatResponse,
    timestamp: Instant,
}

impl ResponseCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }
    
    pub async fn get(&self, key: &str) -> Option<ChatResponse> {
        let cache = self.cache.read().await;
        
        if let Some(cached) = cache.get(key) {
            if cached.timestamp.elapsed() < self.ttl {
                return Some(cached.response.clone());
            }
        }
        
        None
    }
    
    pub async fn set(&self, key: String, response: ChatResponse) {
        let mut cache = self.cache.write().await;
        cache.insert(key, CachedResponse {
            response,
            timestamp: Instant::now(),
        });
    }
    
    pub async fn clear_expired(&self) {
        let mut cache = self.cache.write().await;
        cache.retain(|_, v| v.timestamp.elapsed() < self.ttl);
    }
    
    // Generate cache key from request
    pub fn generate_key(request: &ChatRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        request.model.hash(&mut hasher);
        
        for msg in &request.messages {
            msg.get_text_content().hash(&mut hasher);
        }
        
        format!("{:x}", hasher.finish())
    }
}

// Usage
async fn cached_chat_completion(
    provider: &dyn LLMProvider,
    cache: &ResponseCache,
    request: ChatRequest,
) -> Result<ChatResponse, LLMError> {
    let cache_key = ResponseCache::generate_key(&request);
    
    // Check cache first
    if let Some(cached_response) = cache.get(&cache_key).await {
        log::info!("Cache hit for request");
        return Ok(cached_response);
    }
    
    // Cache miss - make actual request
    let response = provider.chat_completion(request).await?;
    
    // Store in cache
    cache.set(cache_key, response.clone()).await;
    
    Ok(response)
}
```


### 3. Request Batching

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub struct BatchProcessor {
    batch_size: usize,
    batch_timeout: Duration,
}

impl BatchProcessor {
    pub fn new(batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            batch_size,
            batch_timeout,
        }
    }
    
    pub async fn process_batch(
        &self,
        provider: Arc<dyn LLMProvider>,
        mut rx: mpsc::Receiver<(ChatRequest, tokio::sync::oneshot::Sender<Result<ChatResponse, LLMError>>)>,
    ) {
        let mut batch = Vec::new();
        
        loop {
            tokio::select! {
                Some((request, response_tx)) = rx.recv() => {
                    batch.push((request, response_tx));
                    
                    if batch.len() >= self.batch_size {
                        self.execute_batch(&provider, &mut batch).await;
                    }
                }
                _ = sleep(self.batch_timeout), if !batch.is_empty() => {
                    self.execute_batch(&provider, &mut batch).await;
                }
            }
        }
    }
    
    async fn execute_batch(
        &self,
        provider: &Arc<dyn LLMProvider>,
        batch: &mut Vec<(ChatRequest, tokio::sync::oneshot::Sender<Result<ChatResponse, LLMError>>)>,
    ) {
        let futures: Vec<_> = batch
            .drain(..)
            .map(|(request, response_tx)| {
                let provider = provider.clone();
                async move {
                    let result = provider.chat_completion(request).await;
                    let _ = response_tx.send(result);
                }
            })
            .collect();
        
        join_all(futures).await;
    }
}
```

## Security Considerations

### 1. API Key Management

```rust
use secrecy::{Secret, ExposeSecret};

pub struct SecureConfig {
    api_key: Secret<String>,
}

impl SecureConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key: Secret::new(api_key),
        }
    }
    
    pub fn get_api_key(&self) -> &str {
        self.api_key.expose_secret()
    }
}

// Never log API keys
impl std::fmt::Debug for SecureConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecureConfig")
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}
```

### 2. Input Validation

```rust
pub fn validate_user_input(input: &str) -> Result<(), String> {
    // Check length
    if input.len() > 10000 {
        return Err("Input too long".to_string());
    }
    
    // Check for suspicious patterns
    if input.contains("IGNORE PREVIOUS INSTRUCTIONS") {
        return Err("Suspicious input detected".to_string());
    }
    
    // Check for excessive repetition
    let unique_chars: std::collections::HashSet<char> = input.chars().collect();
    if unique_chars.len() < 5 && input.len() > 100 {
        return Err("Input appears to be spam".to_string());
    }
    
    Ok(())
}
```


### 3. Rate Limiting

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration, Instant};

pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    requests_per_minute: u32,
    last_reset: Arc<tokio::sync::Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(requests_per_minute as usize)),
            requests_per_minute,
            last_reset: Arc::new(tokio::sync::Mutex::new(Instant::now())),
        }
    }
    
    pub async fn acquire(&self) -> Result<(), String> {
        // Reset counter every minute
        let mut last_reset = self.last_reset.lock().await;
        if last_reset.elapsed() >= Duration::from_secs(60) {
            *last_reset = Instant::now();
            // Add permits back
            self.semaphore.add_permits(self.requests_per_minute as usize);
        }
        drop(last_reset);
        
        // Acquire permit
        self.semaphore
            .acquire()
            .await
            .map_err(|_| "Rate limit semaphore closed".to_string())?
            .forget();
        
        Ok(())
    }
}

// Usage
async fn rate_limited_request(
    provider: &dyn LLMProvider,
    rate_limiter: &RateLimiter,
    request: ChatRequest,
) -> Result<ChatResponse, LLMError> {
    rate_limiter.acquire().await
        .map_err(|e| LLMError::RateLimitExceeded(e))?;
    
    provider.chat_completion(request).await
}
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Connection Timeouts

```rust
// Increase timeout duration
let http_config = HttpClientConfig {
    timeout: Duration::from_secs(60),
    connect_timeout: Duration::from_secs(10),
    ..Default::default()
};

let provider = OpenAIProvider::new(api_key, None)?
    .with_custom_config(http_config)?;
```

#### 2. Rate Limit Errors

```rust
// Implement exponential backoff
async fn retry_with_backoff<F, T>(
    mut operation: F,
    max_retries: u32,
) -> Result<T, LLMError>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, LLMError>> + Send>>,
{
    let mut delay = Duration::from_secs(1);
    
    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(LLMError::RateLimitExceeded(_)) if attempt < max_retries - 1 => {
                log::warn!("Rate limited, waiting {:?} before retry", delay);
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(LLMError::RateLimitExceeded("Max retries exceeded".to_string()))
}
```


#### 3. Memory Issues with Streaming

```rust
// Process stream in chunks to avoid memory buildup
async fn process_stream_efficiently(
    mut stream: Box<dyn Stream<Item = Result<ChatStreamChunk, LLMError>> + Send + Unpin>,
) -> Result<String, LLMError> {
    let mut buffer = String::with_capacity(1024);
    let mut total_response = String::new();
    
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        
        if let Some(content) = chunk.content {
            buffer.push_str(&content);
            
            // Process buffer when it reaches a certain size
            if buffer.len() > 1024 {
                total_response.push_str(&buffer);
                buffer.clear();
            }
        }
    }
    
    // Don't forget remaining buffer
    total_response.push_str(&buffer);
    
    Ok(total_response)
}
```

#### 4. Debugging API Responses

```rust
use log::debug;

async fn debug_chat_completion(
    provider: &dyn LLMProvider,
    request: ChatRequest,
) -> Result<ChatResponse, LLMError> {
    debug!("Request: {:#?}", request);
    
    let start = Instant::now();
    let result = provider.chat_completion(request).await;
    let elapsed = start.elapsed();
    
    match &result {
        Ok(response) => {
            debug!("Response received in {:?}", elapsed);
            debug!("Model: {}", response.model_used);
            debug!("Tokens: {:?}", response.usage);
            debug!("Content length: {} chars", response.content.len());
        }
        Err(e) => {
            debug!("Error after {:?}: {}", elapsed, e);
        }
    }
    
    result
}
```

## Additional Resources

### Dependencies

Add these to your `Cargo.toml`:

```toml
[dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# HTTP client
reqwest = { version = "0.11", features = ["json", "stream"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Async traits
async-trait = "0.1"

# Streaming
futures = "0.3"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
log = "0.4"
env_logger = "0.11"

# UUID
uuid = { version = "1.6", features = ["v4", "serde"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# Security
secrecy = "0.8"

# Testing
mockall = "0.12"

# Random (for load balancing)
rand = "0.8"
```


### Environment Variables

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."
export OPENAI_BASE_URL="https://api.openai.com/v1"

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."
export ANTHROPIC_BASE_URL="https://api.anthropic.com"

# Configuration
export DEFAULT_MODEL="gpt-3.5-turbo"
export MAX_TOKENS="1000"
export TEMPERATURE="0.7"
export REQUEST_TIMEOUT="30"

# Logging
export RUST_LOG="info,my_app=debug"
```

### Useful Links

- [OpenAI API Documentation](https://platform.openai.com/docs/api-reference)
- [Anthropic API Documentation](https://docs.anthropic.com/claude/reference)
- [Tokio Documentation](https://tokio.rs/)
- [Async Rust Book](https://rust-lang.github.io/async-book/)
- [Reqwest Documentation](https://docs.rs/reqwest/)

## Summary

This guide covered:

1. **Architecture**: Multi-layer design with provider abstraction
2. **Basic Usage**: Simple chat completions, streaming, and embeddings
3. **Advanced Features**: Load balancing, health monitoring, and failover
4. **Error Handling**: Retries, circuit breakers, and error mapping
5. **Best Practices**: Configuration, token management, logging, and testing
6. **Examples**: Real-world implementations from chatbots to semantic search
7. **Performance**: Caching, batching, and connection pooling
8. **Security**: API key management, input validation, and rate limiting
9. **Troubleshooting**: Common issues and solutions

### Key Takeaways

- **Use traits** for provider abstraction to support multiple LLM providers
- **Implement proper error handling** with retries and circuit breakers
- **Monitor health** of providers and implement automatic failover
- **Stream responses** for better user experience with long completions
- **Cache responses** when appropriate to reduce costs and latency
- **Rate limit** requests to avoid hitting API limits
- **Validate inputs** to prevent prompt injection and abuse
- **Test thoroughly** using mocks and integration tests
- **Log appropriately** but never log sensitive data like API keys

### Next Steps

1. Implement additional providers (Anthropic, Cohere, etc.)
2. Add metrics collection and monitoring
3. Implement cost tracking and budgeting
4. Add support for function calling/tools
5. Build a web API layer on top
6. Add database persistence for conversations
7. Implement user authentication and authorization
8. Add support for fine-tuned models

---

**Happy coding with Rust and LLMs!** 🦀🤖
