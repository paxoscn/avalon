# Image Content Support Implementation

## Summary

Successfully implemented image content support for ChatRequest and LLMChatNodeExecutor.

## Changes Made

### 1. Updated ChatMessage Structure (`src/domain/value_objects/chat_message.rs`)

- Changed `content` field from `String` to `MessageContent` enum
- Added `MessageContent` enum with two variants:
  - `Text(String)` - for plain text messages
  - `Multimodal(Vec<ContentPart>)` - for messages with text and images
- Added `ContentPart` enum with:
  - `Text { text: String }` - text content part
  - `ImageUrl { image_url: ImageUrl }` - image URL content part
- Added `ImageUrl` struct with `url` and optional `detail` fields
- Added `new_user_message_with_images()` constructor for creating multimodal messages
- Added `get_text_content()` method for backward compatibility
- Updated `validate()` method to handle both text and multimodal content

### 2. Updated LLM Providers

#### OpenAI Provider (`src/infrastructure/llm/providers/openai.rs`)
- Changed `OpenAIMessage.content` from `String` to `serde_json::Value`
- Updated `convert_request()` to handle multimodal content:
  - Text content is sent as a simple string
  - Multimodal content is sent as an array of content parts with proper OpenAI vision format

#### Claude Provider (`src/infrastructure/llm/providers/claude.rs`)
- Changed `ClaudeMessage.content` from `String` to `serde_json::Value`
- Updated `convert_request()` to handle multimodal content:
  - Text content is sent as a simple string
  - Multimodal content is sent as an array with Claude's image format (using "source" with "url" type)
  - System messages are extracted as text using `get_text_content()`

#### Local LLM Provider (`src/infrastructure/llm/providers/local_llm.rs`)
- Changed `LocalLLMMessage.content` from `String` to `serde_json::Value`
- Updated `convert_request()` to handle multimodal content with OpenAI-compatible format

### 3. Updated LLMChatNodeExecutor (`src/domain/services/node_executors.rs`)

Enhanced `extract_messages()` method to detect and handle image URLs:
- Checks if resolved content is a JSON array of strings
- Filters strings starting with "http://" or "https://" as image URLs
- Separates text and image URLs
- Creates multimodal messages when image URLs are detected
- Falls back to regular text messages when no images are found

### 4. Updated Supporting Code

- Fixed `estimate_token_count()` in `llm_service.rs` and `integrated_llm_service.rs` to use `get_text_content()`
- Fixed `context_management_service.rs` to use `get_text_content()` for token estimation
- Fixed `session_audit_handlers.rs` to create messages with `MessageContent::Text`
- Fixed `session_repository_impl.rs` to handle content conversion
- Fixed `providers/mod.rs` to use `get_text_content()` for standard message conversion

## Usage Example

### Creating a User Message with Images

```rust
use crate::domain::value_objects::ChatMessage;

// Simple text message (backward compatible)
let text_msg = ChatMessage::new_user_message("Hello".to_string());

// Message with images
let multimodal_msg = ChatMessage::new_user_message_with_images(
    "Please analyze these images".to_string(),
    vec![
        "https://example.com/image1.jpg".to_string(),
        "https://example.com/image2.png".to_string(),
    ]
);
```

### In Flow Execution

When a variable contains a JSON array with image URLs, the LLMChatNodeExecutor will automatically detect and create multimodal messages:

```json
{
  "prompt_template": [
    {
      "role": "user",
      "text": "{{#start.user_input#}}"
    }
  ]
}
```

If `{{#start.user_input#}}` resolves to:
```json
["Please describe this image", "https://example.com/image.jpg"]
```

The executor will create a multimodal message with the text and image URL.

## API Format

### OpenAI Format
```json
{
  "role": "user",
  "content": [
    {
      "type": "text",
      "text": "Please analyze this image"
    },
    {
      "type": "image_url",
      "image_url": {
        "url": "https://example.com/image.jpg"
      }
    }
  ]
}
```

### Claude Format
```json
{
  "role": "user",
  "content": [
    {
      "type": "text",
      "text": "Please analyze this image"
    },
    {
      "type": "image",
      "source": {
        "type": "url",
        "url": "https://example.com/image.jpg"
      }
    }
  ]
}
```

## Backward Compatibility

All existing code continues to work:
- `ChatMessage::new_user_message()` creates text-only messages
- `get_text_content()` extracts text from both simple and multimodal messages
- Existing message handling code automatically works with the new structure

## Testing

The implementation has been verified to compile successfully with all existing tests passing.
