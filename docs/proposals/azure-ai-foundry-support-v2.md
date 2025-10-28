# Proposal: Multi-Provider LLM Support with Model Selection (v2)

**Status**: Draft
**Author**: Development Team
**Date**: 2025-10-29
**Target Release**: TBD

## Executive Summary

Add support for multiple LLM providers (Azure AI Foundry alongside SambaNova) with user-selectable models defined in a centralized TOML configuration file. Users will be able to choose which model to use per chat session or per message, with models organized by provider, capabilities, and cost.

## Key Changes from v1

1. **User Model Selection**: Users can select models from UI
2. **TOML Configuration**: Centralized `models.toml` for all model definitions
3. **Provider Abstraction**: Models are abstracted from providers using `provider` keyword
4. **Model Metadata**: Rich metadata including cost, capabilities, context windows
5. **Model Groups**: Organize models by use case (fast, powerful, open-source, etc.)

## Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│                    Frontend UI                            │
│  ┌────────────────────────────────────────────────────┐  │
│  │  Model Selector Dropdown                           │  │
│  │  - Group by: Provider / Capability / Cost          │  │
│  │  - Show: Name, Description, Cost, Context Window   │  │
│  └────────────────────────────────────────────────────┘  │
└────────────────────────┬─────────────────────────────────┘
                         │ POST /messages { model_id }
┌────────────────────────▼─────────────────────────────────┐
│                    Backend API                            │
│  ┌────────────────────────────────────────────────────┐  │
│  │  Model Registry (models.toml)                      │  │
│  │  - Load model definitions                          │  │
│  │  - Validate model_id                               │  │
│  │  - Get provider for model                          │  │
│  └────────────────┬───────────────────────────────────┘  │
│                   │                                       │
│  ┌────────────────▼───────────────────────────────────┐  │
│  │  Provider Factory                                  │  │
│  │  - Route to correct provider                       │  │
│  │  - Apply model-specific config                     │  │
│  └────────────────┬───────────────────────────────────┘  │
└────────────────────┼─────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
┌───────▼──────────┐    ┌────────▼────────────┐
│  SambaNova       │    │  Azure AI           │
│  Provider        │    │  Provider           │
└──────────────────┘    └─────────────────────┘
```

## Models Configuration (models.toml)

### Structure
```toml
# Top-level configuration
default_provider = "sambanova"
default_model = "llama-3.3-70b"

# Provider definitions
[providers.sambanova]
name = "SambaNova"
api_base = "${SAMBANOVA_API_BASE}"
api_key = "${SAMBANOVA_API_KEY}"

[providers.azure]
name = "Azure AI Foundry"
endpoint = "${AZURE_AI_ENDPOINT}"
api_key = "${AZURE_AI_API_KEY}"
api_version = "2024-02-15-preview"

# Model definitions
[[models]]
id = "llama-3.3-70b"              # Unique identifier for API/UI
name = "Llama 3.3 70B Instruct"   # Display name
provider = "sambanova"             # Which provider to use
model_id = "Meta-Llama-3.3-70B-Instruct"  # Provider-specific ID
context_window = 8192
max_output_tokens = 4096
supports_streaming = true
cost_per_million_input_tokens = 0.0
tags = ["general", "fast"]
recommended_for = ["chat", "general-purpose"]

[[models]]
id = "gpt-4o"
name = "GPT-4o"
provider = "azure"
model_id = "${AZURE_GPT4O_DEPLOYMENT}"  # Deployment name
context_window = 128000
max_output_tokens = 4096
supports_streaming = true
supports_function_calling = true
cost_per_million_input_tokens = 2.50
cost_per_million_output_tokens = 10.00
tags = ["multimodal", "advanced"]

# Model groups for UI organization
[model_groups.fast]
name = "Fast & Cheap"
models = ["llama-3.1-8b", "gpt-4o-mini"]

[model_groups.powerful]
name = "Most Capable"
models = ["llama-3.1-405b", "gpt-4o"]
```

### Environment Variable Interpolation
```bash
# Provider credentials in .env
SAMBANOVA_API_KEY=sk-xxx
AZURE_AI_ENDPOINT=https://my-resource.openai.azure.com
AZURE_AI_API_KEY=xxx

# Azure deployment names (referenced in models.toml)
AZURE_GPT4O_DEPLOYMENT=gpt-4o-prod
AZURE_GPT4O_MINI_DEPLOYMENT=gpt-4o-mini-prod
```

## Backend Implementation

### 1. Model Registry
**Location**: `backend/src/infrastructure/llm/model_registry.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model_id: String,  // Provider-specific identifier
    pub description: Option<String>,
    pub context_window: u32,
    pub max_output_tokens: u32,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub cost_per_million_input_tokens: f64,
    pub cost_per_million_output_tokens: f64,
    pub tags: Vec<String>,
    pub recommended_for: Vec<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_base: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: String,
    pub api_version: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelGroup {
    pub name: String,
    pub description: Option<String>,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelsToml {
    pub default_provider: String,
    pub default_model: String,
    pub providers: HashMap<String, ProviderConfig>,
    pub models: Vec<ModelConfig>,
    pub model_groups: Option<HashMap<String, ModelGroup>>,
}

pub struct ModelRegistry {
    config: ModelsToml,
    models_by_id: HashMap<String, ModelConfig>,
}

impl ModelRegistry {
    /// Load models.toml from file with environment variable substitution
    pub fn load() -> Result<Self, ModelRegistryError> {
        let content = std::fs::read_to_string("models.toml")
            .map_err(|e| ModelRegistryError::FileError(e.to_string()))?;

        // Substitute environment variables
        let content = Self::substitute_env_vars(&content)?;

        let config: ModelsToml = toml::from_str(&content)
            .map_err(|e| ModelRegistryError::ParseError(e.to_string()))?;

        // Build lookup map
        let models_by_id = config.models.iter()
            .map(|m| (m.id.clone(), m.clone()))
            .collect();

        Ok(Self {
            config,
            models_by_id,
        })
    }

    /// Substitute ${VAR_NAME} with environment variable values
    fn substitute_env_vars(content: &str) -> Result<String, ModelRegistryError> {
        let re = regex::Regex::new(r"\$\{([A-Z_]+)\}").unwrap();
        let mut result = content.to_string();

        for cap in re.captures_iter(content) {
            let var_name = &cap[1];
            let value = std::env::var(var_name)
                .map_err(|_| ModelRegistryError::MissingEnvVar(var_name.to_string()))?;
            result = result.replace(&format!("${{{}}}", var_name), &value);
        }

        Ok(result)
    }

    /// Get model by ID
    pub fn get_model(&self, id: &str) -> Option<&ModelConfig> {
        self.models_by_id.get(id)
    }

    /// Get default model
    pub fn default_model(&self) -> &ModelConfig {
        self.get_model(&self.config.default_model)
            .expect("Default model must exist")
    }

    /// Get all enabled models
    pub fn enabled_models(&self) -> Vec<&ModelConfig> {
        self.config.models.iter()
            .filter(|m| m.enabled)
            .collect()
    }

    /// Get models by provider
    pub fn models_by_provider(&self, provider: &str) -> Vec<&ModelConfig> {
        self.config.models.iter()
            .filter(|m| m.provider == provider && m.enabled)
            .collect()
    }

    /// Get provider config
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.config.providers.get(name)
    }

    /// Get models in a group
    pub fn get_model_group(&self, group: &str) -> Option<Vec<&ModelConfig>> {
        let groups = self.config.model_groups.as_ref()?;
        let group = groups.get(group)?;

        Some(group.models.iter()
            .filter_map(|id| self.get_model(id))
            .collect())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModelRegistryError {
    #[error("Failed to read models.toml: {0}")]
    FileError(String),

    #[error("Failed to parse models.toml: {0}")]
    ParseError(String),

    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Provider not configured: {0}")]
    ProviderNotConfigured(String),
}
```

### 2. Updated Send Message Request
**Location**: `backend/src/application/chat/send_message.rs`

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageRequest {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub content: String,

    /// Optional: User-selected model ID from models.toml
    /// If not provided, uses default_model from models.toml
    pub model_id: Option<String>,

    /// Optional: Override max_tokens for this request
    /// Must not exceed model's max_output_tokens
    pub max_tokens: Option<u16>,
}

impl SendMessageUseCase {
    pub async fn execute(
        &self,
        request: SendMessageRequest,
    ) -> RepositoryResult<Pin<Box<dyn Stream<Item = Result<StreamChunk, String>> + Send>>> {
        // Load model registry
        let registry = ModelRegistry::load()
            .map_err(|e| RepositoryError::ValidationError(e.to_string()))?;

        // Get model config
        let model_config = if let Some(model_id) = &request.model_id {
            registry.get_model(model_id)
                .ok_or_else(|| RepositoryError::ValidationError(
                    format!("Model not found: {}", model_id)
                ))?
        } else {
            registry.default_model()
        };

        // Validate model is enabled
        if !model_config.enabled {
            return Err(RepositoryError::ValidationError(
                format!("Model is disabled: {}", model_config.id)
            ));
        }

        // Get provider config
        let provider_config = registry.get_provider(&model_config.provider)
            .ok_or_else(|| RepositoryError::ValidationError(
                format!("Provider not configured: {}", model_config.provider)
            ))?;

        if !provider_config.enabled {
            return Err(RepositoryError::ValidationError(
                format!("Provider is disabled: {}", model_config.provider)
            ));
        }

        // Validate max_tokens
        let max_tokens = if let Some(requested) = request.max_tokens {
            if requested > model_config.max_output_tokens as u16 {
                return Err(RepositoryError::ValidationError(
                    format!("Requested max_tokens ({}) exceeds model limit ({})",
                        requested, model_config.max_output_tokens)
                ));
            }
            requested
        } else {
            model_config.max_output_tokens.min(2048) as u16  // Default to 2048 or model max
        };

        // Create provider
        let provider = ProviderFactory::create(
            &model_config.provider,
            provider_config,
        )?;

        // Verify session and save user message (existing logic)
        // ...

        // Get context messages
        let context_messages = self.repository
            .find_recent_messages(request.session_id, model_config.context_window as u64)
            .await?;

        // Stream with selected model
        let stream = provider.stream_chat(
            context_messages,
            &model_config.model_id,  // Use provider-specific model ID
            max_tokens,
        ).await?;

        Ok(stream)
    }
}
```

### 3. API Endpoint Changes

**Request** (with model selection):
```http
POST /api/v1/chat/sessions/{session_id}/messages
Content-Type: application/json

{
  "content": "Write a story",
  "model_id": "gpt-4o",           // Optional: select specific model
  "max_tokens": 4000              // Optional: override default
}
```

**Response** (unchanged):
```
data: {"content":"Once","done":false}
data: {"content":" upon","done":false}
...
data: [DONE]
```

### 4. Get Available Models Endpoint

**New endpoint**:
```http
GET /api/v1/chat/models
```

**Response**:
```json
{
  "default_model": "llama-3.3-70b",
  "models": [
    {
      "id": "llama-3.3-70b",
      "name": "Llama 3.3 70B Instruct",
      "provider": "sambanova",
      "description": "Fast and capable instruction-tuned model",
      "context_window": 8192,
      "max_output_tokens": 4096,
      "supports_streaming": true,
      "supports_function_calling": false,
      "cost_per_million_input_tokens": 0.0,
      "cost_per_million_output_tokens": 0.0,
      "tags": ["general", "fast", "instruct"],
      "recommended_for": ["chat", "general-purpose"]
    },
    {
      "id": "gpt-4o",
      "name": "GPT-4o",
      "provider": "azure",
      "description": "OpenAI's most advanced multimodal model",
      "context_window": 128000,
      "max_output_tokens": 4096,
      "supports_streaming": true,
      "supports_function_calling": true,
      "cost_per_million_input_tokens": 2.50,
      "cost_per_million_output_tokens": 10.00,
      "tags": ["multimodal", "advanced", "gpt"],
      "recommended_for": ["complex-tasks", "multimodal"]
    }
  ],
  "model_groups": {
    "fast": {
      "name": "Fast & Cheap",
      "description": "Quick responses with lower cost",
      "models": ["llama-3.1-8b", "gpt-4o-mini"]
    },
    "powerful": {
      "name": "Most Capable",
      "description": "Best performance for complex tasks",
      "models": ["llama-3.1-405b", "gpt-4o"]
    }
  }
}
```

## Frontend Implementation

### 1. Model Selector Component

**Location**: `frontend/src/components/chat/model-selector.tsx`

```typescript
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import { Info } from 'lucide-react';

interface Model {
  id: string;
  name: string;
  provider: string;
  description: string;
  context_window: number;
  max_output_tokens: number;
  cost_per_million_input_tokens: number;
  cost_per_million_output_tokens: number;
  tags: string[];
}

interface ModelSelectorProps {
  models: Model[];
  selectedModelId: string;
  onModelChange: (modelId: string) => void;
  showCost?: boolean;
}

export function ModelSelector({
  models,
  selectedModelId,
  onModelChange,
  showCost = false
}: ModelSelectorProps) {
  const selectedModel = models.find(m => m.id === selectedModelId);

  return (
    <div className="flex flex-col gap-2">
      <Select value={selectedModelId} onValueChange={onModelChange}>
        <SelectTrigger className="w-full">
          <SelectValue placeholder="Select model">
            {selectedModel && (
              <div className="flex items-center gap-2">
                <span>{selectedModel.name}</span>
                <Badge variant="outline" className="text-xs">
                  {selectedModel.provider}
                </Badge>
              </div>
            )}
          </SelectValue>
        </SelectTrigger>

        <SelectContent>
          {models.map((model) => (
            <SelectItem key={model.id} value={model.id}>
              <div className="flex flex-col gap-1 py-1">
                <div className="flex items-center gap-2">
                  <span className="font-medium">{model.name}</span>
                  <Badge variant="outline" className="text-xs">
                    {model.provider}
                  </Badge>
                </div>

                <div className="text-xs text-muted-foreground">
                  {model.description}
                </div>

                <div className="flex gap-2 text-xs text-muted-foreground">
                  <span>{model.context_window.toLocaleString()} tokens</span>
                  {showCost && model.cost_per_million_input_tokens > 0 && (
                    <span>
                      ${model.cost_per_million_input_tokens.toFixed(2)}/1M in
                    </span>
                  )}
                </div>

                <div className="flex gap-1">
                  {model.tags.map(tag => (
                    <Badge key={tag} variant="secondary" className="text-xs">
                      {tag}
                    </Badge>
                  ))}
                </div>
              </div>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      {selectedModel && (
        <div className="flex items-start gap-2 text-xs text-muted-foreground">
          <Info className="h-3 w-3 mt-0.5" />
          <span>
            Context: {selectedModel.context_window.toLocaleString()} tokens |
            Max output: {selectedModel.max_output_tokens.toLocaleString()} tokens
          </span>
        </div>
      )}
    </div>
  );
}
```

### 2. Update Chat Container

**Location**: `frontend/src/components/chat/chat-container.tsx`

```typescript
import { ModelSelector } from './model-selector';

export function ChatContainer() {
  const [models, setModels] = useState<Model[]>([]);
  const [selectedModelId, setSelectedModelId] = useState<string>('');

  useEffect(() => {
    // Fetch available models
    fetch('/api/v1/chat/models')
      .then(res => res.json())
      .then(data => {
        setModels(data.models);
        setSelectedModelId(data.default_model);
      });
  }, []);

  const handleSendMessage = async (content: string) => {
    await fetch(`/api/v1/chat/sessions/${sessionId}/messages`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        content,
        model_id: selectedModelId  // Include selected model
      })
    });
  };

  return (
    <div className="flex flex-col h-full">
      {/* Model selector at the top */}
      <div className="p-4 border-b">
        <ModelSelector
          models={models}
          selectedModelId={selectedModelId}
          onModelChange={setSelectedModelId}
          showCost={true}
        />
      </div>

      {/* Messages area */}
      <div className="flex-1 overflow-y-auto">
        {messages.map(msg => <Message key={msg.id} {...msg} />)}
      </div>

      {/* Input area */}
      <MessageInput onSend={handleSendMessage} />
    </div>
  );
}
```

## Database Schema Changes

### Store model information with messages

```sql
-- Add model_id column to chat_messages table
ALTER TABLE chat_messages
ADD COLUMN model_id VARCHAR(100),
ADD COLUMN model_name VARCHAR(255),
ADD COLUMN provider VARCHAR(50);

-- Add index for analytics
CREATE INDEX idx_chat_messages_model_id ON chat_messages(model_id);
CREATE INDEX idx_chat_messages_provider ON chat_messages(provider);
```

This enables:
- Usage analytics per model
- Cost tracking per model
- Model performance comparison
- User preference analysis

## Configuration Management

### Development (.env)
```bash
# Provider selection (not needed with models.toml)
# LLM_PROVIDER=sambanova  # Deprecated

# SambaNova
SAMBANOVA_API_BASE=https://api.sambanova.ai/v1
SAMBANOVA_API_KEY=sk-xxx
# SAMBANOVA_MODEL=Meta-Llama-3.3-70B-Instruct  # Now in models.toml

# Azure AI Foundry
AZURE_AI_ENDPOINT=https://my-resource.openai.azure.com
AZURE_AI_API_KEY=xxx
AZURE_GPT4O_DEPLOYMENT=gpt-4o-prod
AZURE_GPT4O_MINI_DEPLOYMENT=gpt-4o-mini-prod
AZURE_GPT35_TURBO_DEPLOYMENT=gpt-35-turbo-prod

# Common settings
CHAT_MAX_CONTEXT_MESSAGES=20
CHAT_MAX_MESSAGE_LENGTH=4000
CHAT_DAILY_MESSAGE_QUOTA=10000
CHAT_RATE_LIMIT_PER_MINUTE=100
```

### models.toml Management
- ✅ Version controlled (check into git)
- ✅ Environment-specific overrides via env vars
- ✅ Easy to add new models without code changes
- ✅ Centralized model metadata
- ✅ Can be hot-reloaded (future feature)

## Benefits of This Approach

### 1. Flexibility
- ✅ Add new models by editing TOML file
- ✅ Add new providers with minimal code
- ✅ Enable/disable models without deployment
- ✅ A/B test different models easily

### 2. User Experience
- ✅ Users see all available models
- ✅ Clear cost information upfront
- ✅ Model recommendations based on use case
- ✅ Per-session model selection

### 3. Operational
- ✅ Cost tracking per model
- ✅ Usage analytics per provider
- ✅ Easy model deprecation
- ✅ Centralized model configuration

### 4. Developer Experience
- ✅ Single source of truth for models
- ✅ Type-safe model configuration
- ✅ Easy testing with different models
- ✅ Clear provider abstraction

## Migration Path

### Phase 1: Model Registry (Week 1)
- [ ] Create `models.toml` with existing SambaNova models
- [ ] Implement `ModelRegistry` loader
- [ ] Update `SendMessageRequest` to accept `model_id`
- [ ] Maintain backward compatibility (no model_id = default)

### Phase 2: API & Database (Week 1-2)
- [ ] Add `GET /api/v1/chat/models` endpoint
- [ ] Add `model_id`, `model_name`, `provider` columns to DB
- [ ] Update save logic to store model metadata

### Phase 3: Frontend (Week 2)
- [ ] Implement `ModelSelector` component
- [ ] Integrate into chat UI
- [ ] Fetch and display available models
- [ ] Send model_id with messages

### Phase 4: Azure Support (Week 2-3)
- [ ] Add Azure models to `models.toml`
- [ ] Implement `AzureAIProvider`
- [ ] Test with Azure deployments
- [ ] Update documentation

### Phase 5: Advanced Features (Week 3-4)
- [ ] Model groups and recommendations
- [ ] Cost tracking and display
- [ ] Usage analytics dashboard
- [ ] Model performance metrics

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_model_registry_load() {
    let registry = ModelRegistry::load().unwrap();
    assert!(registry.get_model("llama-3.3-70b").is_some());
}

#[test]
fn test_env_var_substitution() {
    std::env::set_var("TEST_VAR", "test_value");
    let content = "key = \"${TEST_VAR}\"";
    let result = ModelRegistry::substitute_env_vars(content).unwrap();
    assert_eq!(result, "key = \"test_value\"");
}

#[test]
fn test_model_validation() {
    let registry = ModelRegistry::load().unwrap();
    let model = registry.get_model("gpt-4o").unwrap();
    assert_eq!(model.provider, "azure");
    assert!(model.supports_streaming);
}
```

### Integration Tests
- [ ] Send message with default model
- [ ] Send message with specific model_id
- [ ] Validate model_id not found error
- [ ] Validate disabled model error
- [ ] Verify model metadata stored in DB

## Open Questions

1. **Model Selection Persistence**: Should model selection persist per session or be per-message?
   - **Proposal**: Per-session default, with per-message override

2. **Cost Limits**: Should we enforce per-user cost limits?
   - **Proposal**: Future feature, start with display only

3. **Model Recommendations**: Should we auto-suggest models based on message content?
   - **Proposal**: Future feature, manual selection for now

4. **Fallback Strategy**: If user-selected model fails, fallback to default or error?
   - **Proposal**: Error for transparency, user can retry with different model

5. **Model Deprecation**: How to handle when a model is removed from models.toml?
   - **Proposal**: Mark as `enabled = false`, keep in TOML for historical data

## Documentation Updates

- [ ] Update API documentation with model_id parameter
- [ ] Document models.toml format and structure
- [ ] Create guide for adding new models
- [ ] Create guide for configuring Azure deployments
- [ ] Update environment variable documentation

## Next Steps

1. ✅ Create `models.toml` with SambaNova models
2. ⏭️ Review and approve proposal
3. ⏭️ Implement ModelRegistry
4. ⏭️ Add model selection to API
5. ⏭️ Implement frontend model selector
6. ⏭️ Add Azure AI support

---

**Ready for implementation? This approach provides maximum flexibility with minimal breaking changes.**
