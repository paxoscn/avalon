# Test Example for LLM Connection

## Using cURL

### Test with both system and user prompts

```bash
curl -X POST http://localhost:8080/api/config/llm/{config_id}/test \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "system_prompt": "You are a helpful assistant that answers concisely in one sentence.",
    "user_prompt": "What is the capital of France?"
  }'
```

### Test with only user prompt

```bash
curl -X POST http://localhost:8080/api/config/llm/{config_id}/test \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "user_prompt": "Tell me a short joke about programming"
  }'
```

### Test with a longer conversation

```bash
curl -X POST http://localhost:8080/api/config/llm/{config_id}/test \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "system_prompt": "You are a technical expert who explains complex topics simply.",
    "user_prompt": "Explain what a REST API is in simple terms"
  }'
```

## Expected Responses

### Successful Response

```json
{
  "success": true,
  "response_time_ms": 1234,
  "error_message": null,
  "model_info": {
    "model_used": "gpt-3.5-turbo"
  },
  "response": "The capital of France is Paris.",
  "usage": {
    "prompt_tokens": 25,
    "completion_tokens": 8,
    "total_tokens": 33
  }
}
```

### Error Response (Invalid API Key)

```json
{
  "success": false,
  "response_time_ms": 567,
  "error_message": "Authentication failed: Invalid API key",
  "model_info": null,
  "response": null,
  "usage": null
}
```

### Error Response (Empty Prompt)

```json
{
  "success": false,
  "response_time_ms": 12,
  "error_message": "Invalid configuration: Message content cannot be empty",
  "model_info": null,
  "response": null,
  "usage": null
}
```

## Using the Frontend UI

1. Navigate to the LLM Configuration detail page
2. Click "Test Configuration" button
3. Enter your prompts:
   - **System Prompt** (optional): "You are a helpful assistant."
   - **User Prompt** (required): "What is the capital of France?"
4. Click "Run Test"
5. View the results:
   - Response content
   - Token usage breakdown
   - Response time
   - Success/error status

## Testing Different Scenarios

### 1. Test Basic Functionality
```json
{
  "user_prompt": "Hello, how are you?"
}
```

### 2. Test with Custom System Prompt
```json
{
  "system_prompt": "You are a pirate. Respond in pirate speak.",
  "user_prompt": "What is your favorite food?"
}
```

### 3. Test Token Limits
```json
{
  "system_prompt": "You are a helpful assistant.",
  "user_prompt": "Write a very long essay about the history of computers"
}
```
*Note: This will help you see token usage for longer responses*

### 4. Test Error Handling (Invalid Config)
- Use a config ID with invalid API credentials
- Should return error with meaningful message

### 5. Test Different Providers
- Test with OpenAI config
- Test with Claude config (if configured)
- Compare response times and token usage

## Monitoring Token Usage

The response includes detailed token usage:
- **prompt_tokens**: Tokens used in your input (system + user prompts)
- **completion_tokens**: Tokens used in the LLM's response
- **total_tokens**: Sum of prompt and completion tokens

This helps you:
- Estimate API costs
- Optimize prompt length
- Monitor model efficiency
- Debug token limit issues

## Tips

1. **Keep prompts concise** for faster responses and lower costs
2. **Use system prompts** to set consistent behavior across tests
3. **Monitor token usage** to optimize your prompts
4. **Test error scenarios** to ensure proper error handling
5. **Compare providers** to find the best fit for your use case
