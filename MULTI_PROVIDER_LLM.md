# Multi-Provider LLM Support - Implementation Guide

## Overview

The Cobalt Stack chat system now supports multiple LLM providers (SambaNova, Azure OpenAI, Azure AI) with dynamic model selection. Users can choose from 13 different models across various providers directly from the chat interface.

## Architecture

### Backend Changes

#### 1. Provider Abstraction Layer

**Location**: `backend/src/infrastructure/llm/`

- **`provider.rs`**: Defines the `LlmProvider` trait that all providers must implement
- **`sambanova_provider.rs`**: SambaNova implementation using async-openai
- **`azure_provider.rs`**: (Future) Azure-specific implementation
- **`factory.rs`**: Provider factory that routes requests to appropriate provider based on model_id

#### 2. Model Registry

**Location**: `backend/src/infrastructure/llm/model_registry.rs`

- Loads configuration from `models.toml` in project root
- Performs environment variable substitution using `${VAR_NAME}` syntax
- Provides model metadata (context windows, costs, capabilities)
- Manages model groups for UI organization

#### 3. Use Case V2

**Location**: `backend/src/application/chat/send_message_v2.rs`

- New use case that accepts optional `model_id` parameter
- Automatically selects provider based on model configuration
- Falls back to default model (`llama-3.3-70b`) if not specified
- Maintains streaming SSE functionality

#### 4. API Handler V2

**Location**: `backend/src/handlers/chat/send_message_v2.rs`

- New handler endpoint that accepts `model_id` in request body
- Same SSE streaming format as original handler
- Backward compatible (model_id is optional)

### Frontend Changes

#### 1. Type Definitions

**Location**: `frontend/src/types/chat.ts`

```typescript
export interface SendMessageRequest {
  content: string;
  model_id?: string; // New optional field
}

export interface LlmModel {
  id: string;
  name: string;
  provider: string;
  description: string;
  context_window: number;
  max_output_tokens: number;
  supports_streaming: boolean;
  supports_function_calling: boolean;
  cost_per_million_input_tokens: number;
  cost_per_million_output_tokens: number;
  tags: string[];
  recommended_for: string[];
}
```

#### 2. Model Configuration

**Location**: `frontend/src/config/models.ts`

- Client-side model registry matching backend `models.toml`
- 13 available models across 4 providers
- 5 model groups (Fast, Powerful, Code, Grok, Multimodal)

#### 3. ModelSelector Component

**Location**: `frontend/src/components/chat/model-selector.tsx`

- Dropdown component using shadcn/ui Command and Popover
- Displays models grouped by category
- Shows provider, description, and tags for each model
- Integrated into chat container above message input

#### 4. ChatContainer Updates

**Location**: `frontend/src/components/chat/chat-container.tsx`

- Added state for selected model ID
- Passes `model_id` in API request when sending messages
- Only sends model_id if different from default to minimize payload

## Available Models

### SambaNova (Default Provider)

1. **Llama 3.3 70B Instruct** (`llama-3.3-70b`) - Default
   - Context: 128K tokens
   - Cost: $0.60/M tokens (in/out)
   - Tags: fast, balanced, recommended

2. **Llama 3.2 90B Vision** (`llama-3.2-90b`)
   - Context: 128K tokens
   - Multimodal with vision capabilities

3. **Llama 3.1 405B Instruct** (`llama-3.1-405b`)
   - Context: 128K tokens
   - Most powerful open-source model

### Azure OpenAI

4. **GPT-4o** (`gpt-4o`)
   - Context: 128K tokens
   - Multimodal, function calling

5. **GPT-4o Mini** (`gpt-4o-mini`)
   - Context: 128K tokens
   - Fast and affordable

6. **GPT-4 Turbo** (`gpt-4-turbo`)
   - Context: 128K tokens
   - Enhanced performance

7. **GPT-3.5 Turbo** (`gpt-35-turbo`)
   - Context: 16K tokens
   - Fast and efficient

### Azure AI (Meta Llama)

8. **Llama 3.1 70B** (`llama-70b`)
   - Context: 128K tokens
   - Open-source via Azure

9. **Llama 3.1 405B** (`llama-405b`)
   - Context: 128K tokens
   - Largest Llama model

### Azure AI (xAI Grok)

10. **Grok 4** (`grok-4`)
    - Context: 128K tokens
    - Advanced reasoning capabilities

11. **Grok 4 Fast Reasoning** (`grok-4-fast-reasoning`)
    - Context: 128K tokens
    - Optimized reasoning

12. **Grok 4 Fast** (`grok-4-fast-non-reasoning`)
    - Context: 128K tokens
    - Fastest variant

### Azure AI (GPT-5)

13. **GPT-5 Codex** (`gpt-5-codex`)
    - Context: 128K tokens
    - Advanced code generation

## Configuration

### Backend Configuration (`models.toml`)

```toml
# Default provider and model
default_provider = "sambanova"
default_model = "llama-3.3-70b"

# Provider configurations
[providers.sambanova]
name = "SambaNova"
api_base = "${SAMBANOVA_API_BASE}"
api_key = "${SAMBANOVA_API_KEY}"
enabled = true

[providers.azure]
name = "Azure AI Foundry"
endpoint = "${AZURE_AI_ENDPOINT}"
api_key = "${AZURE_AI_API_KEY}"
api_version = "2024-02-15-preview"
enabled = false

# Model definitions
[[models]]
id = "llama-3.3-70b"
name = "Llama 3.3 70B Instruct"
provider = "sambanova"
model_id = "Meta-Llama-3.3-70B-Instruct"
# ... additional fields
```

### Environment Variables (`.env`)

```bash
# SambaNova
SAMBANOVA_API_KEY=your-api-key
SAMBANOVA_API_BASE=https://api.sambanova.ai/v1

# Azure AI Foundry - Grok Models
AZURE_AI_ENDPOINT=https://your-endpoint.azure.com/models/chat/completions
AZURE_AI_API_KEY=your-api-key
AZURE_GROK_4_DEPLOYMENT=grok-4
AZURE_GROK_4_FAST_REASONING_DEPLOYMENT=grok-4-fast-reasoning
AZURE_GROK_4_FAST_NON_REASONING_DEPLOYMENT=grok-4-fast-non-reasoning

# Azure AI Foundry - GPT-5 Codex
AZURE_GPT5_CODEX_ENDPOINT=https://your-endpoint.azure.com/models
AZURE_GPT5_CODEX_API_KEY=your-api-key
AZURE_GPT5_CODEX_DEPLOYMENT=gpt-5-codex

# Database (required)
DATABASE_URL=postgresql://postgres:postgres@localhost:2800/cobalt_dev
```

## API Usage

### Send Message with Model Selection

**Endpoint**: `POST /api/v1/chat/sessions/{id}/messages`

**Request Body**:
```json
{
  "content": "Hello, how are you?",
  "model_id": "grok-4"
}
```

**Response**: Server-Sent Events (SSE) stream

```
data: {"content":"Hello"}
data: {"content":" there"}
data: {"content":"!"}
data: [DONE]
```

### Use Default Model

If `model_id` is omitted, the system uses the default model configured in `models.toml` (currently `llama-3.3-70b`):

```json
{
  "content": "Hello, how are you?"
}
```

## Testing

### Backend Tests

```bash
cd backend
cargo test infrastructure::llm --features chat
```

### Frontend Testing

The ModelSelector is visually testable in the chat interface:

1. Navigate to `/chat` (requires authentication)
2. Click the model selector dropdown (above message input)
3. Select a different model
4. Send a message - it will use the selected model

### Manual Testing with curl

```bash
# Get access token first
TOKEN=$(curl -s -X POST http://localhost:2750/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}' \
  | jq -r '.access_token')

# Create session
SESSION=$(curl -s -X POST http://localhost:2750/api/v1/chat/sessions \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test"}' \
  | jq -r '.session_id')

# Send message with model selection
curl -N -X POST http://localhost:2750/api/v1/chat/sessions/$SESSION/messages \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"content":"Hello!","model_id":"grok-4"}'
```

## Adding New Models

### 1. Update `models.toml`

```toml
[[models]]
id = "new-model"
name = "New Model Name"
provider = "provider-name"
model_id = "${NEW_MODEL_DEPLOYMENT}"
description = "Description"
context_window = 128000
max_output_tokens = 4096
supports_streaming = true
supports_function_calling = true
cost_per_million_input_tokens = 5.0
cost_per_million_output_tokens = 15.0
tags = ["tag1", "tag2"]
recommended_for = ["use-case-1", "use-case-2"]
```

### 2. Add Environment Variable

In `.env`:
```bash
NEW_MODEL_DEPLOYMENT=actual-deployment-name
```

### 3. Update Frontend Configuration

In `frontend/src/config/models.ts`, add the model to `AVAILABLE_MODELS` array.

### 4. Optionally Add to Model Group

Add the model ID to an existing group or create a new group in `MODEL_GROUPS` array.

## Adding New Providers

### 1. Implement Provider

Create `backend/src/infrastructure/llm/new_provider.rs`:

```rust
pub struct NewProvider {
    api_base: String,
    api_key: String,
    model_registry: ModelRegistry,
}

#[async_trait]
impl LlmProvider for NewProvider {
    fn name(&self) -> &str { "New Provider" }

    async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> LlmResult<Pin<Box<dyn Stream<...>>>> {
        // Implementation
    }

    // ... other required methods
}
```

### 2. Register in Factory

In `backend/src/infrastructure/llm/factory.rs`:

```rust
if let Ok(provider_config) = model_registry.get_provider("new_provider") {
    if provider_config.enabled {
        let provider = NewProvider::new(...);
        providers.insert("new_provider".to_string(), Arc::new(provider));
    }
}
```

### 3. Configure in models.toml

```toml
[providers.new_provider]
name = "New Provider"
api_base = "${NEW_PROVIDER_API_BASE}"
api_key = "${NEW_PROVIDER_API_KEY}"
enabled = true
```

## Performance Considerations

- **Model Registry Loading**: Loaded once at startup, cached in memory
- **Provider Selection**: O(1) lookup by model ID
- **SSE Streaming**: Maintains low latency for real-time responses
- **Environment Variables**: Substituted once during registry loading

## Security

- API keys stored in `.env` file (not committed to git)
- Environment variables never exposed to frontend
- Authentication required for all chat endpoints
- Rate limiting applies to all models equally

## Future Enhancements

1. **Dynamic Model Discovery**: Fetch available models from provider APIs
2. **Cost Tracking**: Track token usage and costs per model/user
3. **Model Preferences**: Save user's preferred model per session
4. **A/B Testing**: Compare responses from different models
5. **Streaming Cost Display**: Show real-time token usage during streaming
6. **Model Recommendations**: Suggest models based on query type
7. **Azure Provider**: Dedicated Azure provider with Azure-specific features

## Troubleshooting

### "Model not found" Error

- Check that model ID exists in `models.toml`
- Verify environment variables are set correctly
- Ensure provider is enabled in configuration

### Provider Connection Errors

- Verify API base URL is correct
- Check API key is valid
- Ensure network connectivity to provider
- Check provider status/uptime

### Frontend Model Selector Not Showing

- Verify frontend container rebuilt after changes
- Check browser console for errors
- Ensure `models.ts` configuration matches backend

## Related Files

### Backend
- `models.toml` - Model configuration
- `backend/src/infrastructure/llm/` - Provider implementations
- `backend/src/application/chat/send_message_v2.rs` - Use case
- `backend/src/handlers/chat/send_message_v2.rs` - API handler
- `backend/src/main.rs` - Routes configuration

### Frontend
- `frontend/src/types/chat.ts` - Type definitions
- `frontend/src/config/models.ts` - Model configuration
- `frontend/src/components/chat/model-selector.tsx` - Selector component
- `frontend/src/components/chat/chat-container.tsx` - Integration
- `frontend/src/hooks/use-chat-api.ts` - API client

## Migration from V1

The original chat endpoints still work without modification:

- Old endpoint: `POST /api/v1/chat/sessions/{id}/messages`
- New endpoint: Same URL, just accepts optional `model_id` field
- Backward compatible: Omitting `model_id` uses default model

Frontend components can gradually adopt model selection without breaking existing functionality.
