# Frontend Integration Example

## TypeScript/React Example

### API Client Function

```typescript
interface TestLLMConnectionRequest {
  system_prompt?: string;
  user_prompt: string;
}

interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

interface TestLLMConnectionResponse {
  success: boolean;
  response_time_ms: number;
  error_message?: string;
  model_info?: {
    model_used: string;
  };
  response?: string;
  usage?: TokenUsage;
}

async function testLLMConnection(
  configId: string,
  request: TestLLMConnectionRequest
): Promise<TestLLMConnectionResponse> {
  const response = await fetch(`/api/config/llm/${configId}/test`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${getAuthToken()}`,
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  return await response.json();
}
```

### React Component Example

```tsx
import React, { useState } from 'react';

interface LLMTestFormProps {
  configId: string;
}

export const LLMTestForm: React.FC<LLMTestFormProps> = ({ configId }) => {
  const [systemPrompt, setSystemPrompt] = useState('You are a helpful assistant.');
  const [userPrompt, setUserPrompt] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<TestLLMConnectionResponse | null>(null);

  const handleTest = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setResult(null);

    try {
      const response = await testLLMConnection(configId, {
        system_prompt: systemPrompt || undefined,
        user_prompt: userPrompt,
      });
      setResult(response);
    } catch (error) {
      console.error('Test failed:', error);
      setResult({
        success: false,
        response_time_ms: 0,
        error_message: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="llm-test-form">
      <form onSubmit={handleTest}>
        <div className="form-group">
          <label htmlFor="system-prompt">System Prompt (Optional)</label>
          <textarea
            id="system-prompt"
            value={systemPrompt}
            onChange={(e) => setSystemPrompt(e.target.value)}
            placeholder="Enter system prompt..."
            rows={3}
          />
        </div>

        <div className="form-group">
          <label htmlFor="user-prompt">User Prompt *</label>
          <textarea
            id="user-prompt"
            value={userPrompt}
            onChange={(e) => setUserPrompt(e.target.value)}
            placeholder="Enter your test message..."
            rows={4}
            required
          />
        </div>

        <button type="submit" disabled={loading || !userPrompt.trim()}>
          {loading ? 'Testing...' : 'Test LLM'}
        </button>
      </form>

      {result && (
        <div className={`result ${result.success ? 'success' : 'error'}`}>
          <h3>{result.success ? 'Success!' : 'Error'}</h3>
          
          {result.success && result.response && (
            <div className="response-content">
              <h4>Response:</h4>
              <p>{result.response}</p>
            </div>
          )}

          {result.error_message && (
            <div className="error-message">
              <h4>Error:</h4>
              <p>{result.error_message}</p>
            </div>
          )}

          <div className="metadata">
            <p>Response Time: {result.response_time_ms}ms</p>
            {result.model_info && (
              <p>Model: {result.model_info.model_used}</p>
            )}
            {result.usage && (
              <div className="token-usage">
                <h4>Token Usage:</h4>
                <ul>
                  <li>Prompt: {result.usage.prompt_tokens}</li>
                  <li>Completion: {result.usage.completion_tokens}</li>
                  <li>Total: {result.usage.total_tokens}</li>
                </ul>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
```

### Usage in Parent Component

```tsx
import { LLMTestForm } from './LLMTestForm';

function LLMConfigDetailPage() {
  const { configId } = useParams();

  return (
    <div>
      <h1>LLM Configuration Test</h1>
      <LLMTestForm configId={configId} />
    </div>
  );
}
```

## cURL Example

```bash
# Test with both system and user prompts
curl -X POST http://localhost:8080/api/config/llm/{config_id}/test \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "system_prompt": "You are a helpful assistant that answers concisely.",
    "user_prompt": "What is the capital of France?"
  }'

# Test with only user prompt
curl -X POST http://localhost:8080/api/config/llm/{config_id}/test \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "user_prompt": "Tell me a joke"
  }'
```

## Expected Response Examples

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

### Error Response
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
