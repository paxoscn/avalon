use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: MessageContent,
    pub metadata: Option<MessageMetadata>,
    pub timestamp: DateTime<Utc>,
}

/// Message content that can be either text or multimodal (text + images)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Multimodal(Vec<ContentPart>),
}

/// Individual content part (text or image)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

/// Image URL with optional detail level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub model_used: Option<String>,
    pub tokens_used: Option<u32>,
    pub response_time_ms: Option<u64>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub custom_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_id: String,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionContext {
    pub variables: HashMap<String, serde_json::Value>,
    pub conversation_summary: Option<String>,
    pub last_activity: DateTime<Utc>,
    pub message_count: u32,
    pub custom_context: HashMap<String, serde_json::Value>,
}

impl ChatMessage {
    pub fn new_user_message(content: String) -> Self {
        ChatMessage {
            role: MessageRole::User,
            content: MessageContent::Text(content),
            metadata: None,
            timestamp: Utc::now(),
        }
    }

    pub fn new_assistant_message(content: String) -> Self {
        ChatMessage {
            role: MessageRole::Assistant,
            content: MessageContent::Text(content),
            metadata: None,
            timestamp: Utc::now(),
        }
    }

    pub fn new_system_message(content: String) -> Self {
        ChatMessage {
            role: MessageRole::System,
            content: MessageContent::Text(content),
            metadata: None,
            timestamp: Utc::now(),
        }
    }

    pub fn new_user_message_with_images(text: String, image_urls: Vec<String>) -> Self {
        let mut parts = vec![ContentPart::Text { text }];
        for url in image_urls {
            parts.push(ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url,
                    detail: None,
                },
            });
        }
        ChatMessage {
            role: MessageRole::User,
            content: MessageContent::Multimodal(parts),
            metadata: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, metadata: MessageMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        match &self.content {
            MessageContent::Text(text) => {
                if text.trim().is_empty() {
                    return Err("Message content cannot be empty".to_string());
                }
                if text.len() > 100_000 {
                    return Err("Message content exceeds maximum length of 100,000 characters".to_string());
                }
            }
            MessageContent::Multimodal(parts) => {
                if parts.is_empty() {
                    return Err("Multimodal content cannot be empty".to_string());
                }
                for part in parts {
                    match part {
                        ContentPart::Text { text } => {
                            if text.len() > 100_000 {
                                return Err("Text part exceeds maximum length of 100,000 characters".to_string());
                            }
                        }
                        ContentPart::ImageUrl { image_url } => {
                            if image_url.url.trim().is_empty() {
                                return Err("Image URL cannot be empty".to_string());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Get the text content from the message (for backward compatibility)
    pub fn get_text_content(&self) -> String {
        match &self.content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Multimodal(parts) => {
                parts
                    .iter()
                    .filter_map(|part| match part {
                        ContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }
    }

    pub fn is_from_user(&self) -> bool {
        self.role == MessageRole::User
    }

    pub fn is_from_assistant(&self) -> bool {
        self.role == MessageRole::Assistant
    }

    pub fn has_tool_calls(&self) -> bool {
        self.metadata
            .as_ref()
            .and_then(|m| m.tool_calls.as_ref())
            .map(|calls| !calls.is_empty())
            .unwrap_or(false)
    }
}

impl SessionContext {
    pub fn new() -> Self {
        SessionContext {
            variables: HashMap::new(),
            conversation_summary: None,
            last_activity: Utc::now(),
            message_count: 0,
            custom_context: HashMap::new(),
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    pub fn increment_message_count(&mut self) {
        self.message_count += 1;
        self.update_activity();
    }

    pub fn set_variable(&mut self, key: String, value: serde_json::Value) {
        self.variables.insert(key, value);
        self.update_activity();
    }

    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }

    pub fn is_expired(&self, timeout_minutes: u64) -> bool {
        let timeout_duration = chrono::Duration::minutes(timeout_minutes as i64);
        Utc::now() - self.last_activity > timeout_duration
    }
}

impl Default for SessionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MessageMetadata {
    fn default() -> Self {
        MessageMetadata {
            model_used: None,
            tokens_used: None,
            response_time_ms: None,
            tool_calls: None,
            custom_data: HashMap::new(),
        }
    }
}