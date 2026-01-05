# Proposal: Azure AI Foundry Support

**Status**: Draft
**Author**: Development Team
**Date**: 2025-10-29
**Target Release**: TBD

## Executive Summary

Add support for Azure AI Foundry as an alternative LLM provider alongside the existing SambaNova integration. This will provide deployment flexibility, enterprise-grade reliability, and access to Azure's model catalog including GPT-4, GPT-4o, Llama models, and other Azure-hosted models.

## Motivation

### Current State
- Single LLM provider (SambaNova) creates vendor lock-in
- No fallback if SambaNova experiences downtime
- Limited model selection (currently constrained to SambaNova's catalog)
- No support for Azure enterprise customers who prefer Azure-hosted solutions

### Benefits of Azure AI Foundry
1. **Multi-provider architecture**: Reduces vendor lock-in, enables A/B testing
2. **Enterprise integration**: Native Azure authentication, compliance, and governance
3. **Broader model selection**: Access to GPT-4, GPT-4o, Llama, Mistral, and more
4. **Regional deployment**: Data residency and latency optimization
5. **Cost optimization**: Leverage Azure credits and enterprise agreements
6. **Reliability**: Azure SLA-backed infrastructure with multi-region failover

## Goals

### Primary Goals
- [ ] Support Azure AI Foundry as a configurable LLM provider
- [ ] Maintain OpenAI-compatible API interface for consistency
- [ ] Enable runtime provider selection (SambaNova, Azure, or future providers)
- [ ] Preserve existing SambaNova functionality (no breaking changes)

### Secondary Goals
- [ ] Provider-specific configuration and validation
- [ ] Provider health monitoring and automatic fallback
- [ ] Usage tracking per provider for cost analysis
- [ ] Model capability detection and routing

### Non-Goals (Future Work)
- Multi-provider load balancing (round-robin, weighted)
- Automatic cost optimization across providers
- Provider-specific prompt optimization
- Advanced features like function calling, vision, or embeddings

## Design

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     Chat API Layer                       │
│                  (handlers/chat/*)                       │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│              Application Layer                           │
│            (application/chat/send_message.rs)            │
│                                                           │
│  ┌───────────────────────────────────────────────────┐  │
│  │         SendMessageUseCase                        │  │
│  │  - Message validation                             │  │
│  │  - Context retrieval                              │  │
│  │  - Provider selection ◄─────────┐                │  │
│  │  - Stream coordination          │                │  │
│  └─────────────────┬───────────────┘                │  │
│                    │                                 │  │
└────────────────────┼─────────────────────────────────┘  │
                     │                                     │
┌────────────────────▼─────────────────────────────────┐  │
│          LLM Provider Abstraction (NEW)               │  │
│        (infrastructure/llm/provider.rs)               │  │
│                                                        │  │
│  ┌──────────────────────────────────────────────┐   │  │
│  │  trait LlmProvider {                         │   │  │
│  │    async fn stream_chat(...)                 │   │  │
│  │    fn provider_name() -> &str                │   │  │
│  │    fn supports_model(model: &str) -> bool    │   │  │
│  │  }                                           │   │  │
│  └──────────────────────────────────────────────┘   │  │
└────────────────────┬─────────────────────────────────┘  │
                     │                                     │
        ┌────────────┴────────────┐                       │
        │                         │                       │
┌───────▼──────────┐    ┌────────▼────────────┐         │
│  SambaNovaProvider│    │  AzureAIProvider    │         │
│  (existing logic) │    │  (NEW)              │         │
│                   │    │                     │         │
│  - OpenAI compat  │    │  - Azure auth       │         │
│  - Direct API     │    │  - Model routing    │         │
└───────┬───────────┘    └────────┬────────────┘         │
        │                         │                       │
        │                         │                       │
┌───────▼─────────────────────────▼────────────┐         │
│      Provider Configuration Manager           │         │
│     (config/llm_providers.rs)                 │         │
│                                               │         │
│  - Environment-based config                   │         │
│  - Provider validation                        │◄────────┘
│  - Default provider selection                 │
│  - Fallback configuration                     │
└───────────────────────────────────────────────┘
```

### Component Design

#### 1. LLM Provider Trait
**Location**: `backend/src/infrastructure/llm/provider.rs`

```rust
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub is_final: bool,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider name for logging and identification
    fn provider_name(&self) -> &str;

    /// Check if provider supports a specific model
    fn supports_model(&self, model: &str) -> bool;

    /// Stream chat completion
    async fn stream_chat(
        &self,
        messages: Vec<ChatMessage>,
        model: &str,
        max_tokens: u16,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, String>> + Send>>, ProviderError>;

    /// Get provider health status
    async fn health_check(&self) -> Result<(), ProviderError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Model not supported: {0}")]
    UnsupportedModel(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}
```

#### 2. Azure AI Foundry Provider
**Location**: `backend/src/infrastructure/llm/azure_ai.rs`

```rust
use async_openai::{Client, config::AzureConfig};
use async_trait::async_trait;

pub struct AzureAIProvider {
    client: Client<AzureConfig>,
    deployment: String,
    api_version: String,
}

impl AzureAIProvider {
    pub fn new(
        endpoint: String,
        api_key: String,
        deployment: String,
        api_version: String,
    ) -> Result<Self, ProviderError> {
        let config = AzureConfig::new()
            .with_api_base(&endpoint)
            .with_api_key(&api_key)
            .with_api_version(&api_version)
            .with_deployment_id(&deployment);

        Ok(Self {
            client: Client::with_config(config),
            deployment,
            api_version,
        })
    }
}

#[async_trait]
impl LlmProvider for AzureAIProvider {
    fn provider_name(&self) -> &str {
        "azure-ai"
    }

    fn supports_model(&self, model: &str) -> bool {
        // Azure AI Foundry uses deployments, not model names
        // Always return true as model validation happens at deployment level
        true
    }

    async fn stream_chat(
        &self,
        messages: Vec<ChatMessage>,
        _model: &str, // Ignored, uses deployment
        max_tokens: u16,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, String>> + Send>>, ProviderError> {
        // Convert to OpenAI format
        let openai_messages = messages.into_iter()
            .map(|msg| match msg.role {
                MessageRole::System => ChatCompletionRequestMessage::System(/*...*/),
                MessageRole::User => ChatCompletionRequestMessage::User(/*...*/),
                MessageRole::Assistant => ChatCompletionRequestMessage::Assistant(/*...*/),
            })
            .collect();

        let request = CreateChatCompletionRequestArgs::default()
            .messages(openai_messages)
            .max_tokens(max_tokens)
            .stream(true)
            .build()
            .map_err(|e| ProviderError::Configuration(e.to_string()))?;

        let stream = self.client
            .chat()
            .create_stream(request)
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        // Transform to our StreamChunk format
        let output_stream = stream.map(|result| {
            result
                .map(|response| {
                    response.choices.into_iter()
                        .filter_map(|choice| {
                            choice.delta.content.map(|content| StreamChunk {
                                content,
                                is_final: choice.finish_reason.is_some(),
                            })
                        })
                        .next()
                        .unwrap_or(StreamChunk {
                            content: String::new(),
                            is_final: true,
                        })
                })
                .map_err(|e| e.to_string())
        });

        Ok(Box::pin(output_stream))
    }

    async fn health_check(&self) -> Result<(), ProviderError> {
        // Simple health check: try to create a minimal request
        // Implementation details...
        Ok(())
    }
}
```

#### 3. Provider Configuration
**Location**: `backend/src/config/llm_providers.rs`

```rust
use std::env;

#[derive(Debug, Clone)]
pub enum LlmProviderType {
    SambaNova,
    AzureAI,
}

#[derive(Debug, Clone)]
pub struct LlmProviderConfig {
    pub provider_type: LlmProviderType,
    pub fallback_provider: Option<LlmProviderType>,

    // SambaNova config
    pub sambanova_api_base: Option<String>,
    pub sambanova_api_key: Option<String>,
    pub sambanova_model: Option<String>,

    // Azure AI config
    pub azure_endpoint: Option<String>,
    pub azure_api_key: Option<String>,
    pub azure_deployment: Option<String>,
    pub azure_api_version: Option<String>,

    // Common config
    pub max_tokens: u16,
    pub max_context_messages: u64,
}

impl LlmProviderConfig {
    pub fn from_env() -> Self {
        let provider_type = env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "sambanova".to_string())
            .parse()
            .unwrap_or(LlmProviderType::SambaNova);

        let fallback_provider = env::var("LLM_FALLBACK_PROVIDER")
            .ok()
            .and_then(|s| s.parse().ok());

        Self {
            provider_type,
            fallback_provider,

            // SambaNova
            sambanova_api_base: env::var("SAMBANOVA_API_BASE").ok(),
            sambanova_api_key: env::var("SAMBANOVA_API_KEY").ok(),
            sambanova_model: env::var("SAMBANOVA_MODEL").ok(),

            // Azure AI
            azure_endpoint: env::var("AZURE_AI_ENDPOINT").ok(),
            azure_api_key: env::var("AZURE_AI_API_KEY").ok(),
            azure_deployment: env::var("AZURE_AI_DEPLOYMENT").ok(),
            azure_api_version: env::var("AZURE_AI_API_VERSION")
                .unwrap_or_else(|_| "2024-02-15-preview".to_string())
                .into(),

            // Common
            max_tokens: env::var("CHAT_MAX_TOKENS")
                .unwrap_or_else(|_| "2048".to_string())
                .parse()
                .unwrap_or(2048),
            max_context_messages: env::var("CHAT_MAX_CONTEXT_MESSAGES")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        match self.provider_type {
            LlmProviderType::SambaNova => {
                self.sambanova_api_key.as_ref()
                    .ok_or("SAMBANOVA_API_KEY required")?;
                self.sambanova_model.as_ref()
                    .ok_or("SAMBANOVA_MODEL required")?;
            }
            LlmProviderType::AzureAI => {
                self.azure_endpoint.as_ref()
                    .ok_or("AZURE_AI_ENDPOINT required")?;
                self.azure_api_key.as_ref()
                    .ok_or("AZURE_AI_API_KEY required")?;
                self.azure_deployment.as_ref()
                    .ok_or("AZURE_AI_DEPLOYMENT required")?;
            }
        }
        Ok(())
    }
}
```

#### 4. Provider Factory
**Location**: `backend/src/infrastructure/llm/factory.rs`

```rust
use super::{
    provider::{LlmProvider, ProviderError},
    sambanova::SambaNovaProvider,
    azure_ai::AzureAIProvider,
};
use crate::config::llm_providers::{LlmProviderConfig, LlmProviderType};
use std::sync::Arc;

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create_provider(
        config: &LlmProviderConfig
    ) -> Result<Arc<dyn LlmProvider>, ProviderError> {
        match config.provider_type {
            LlmProviderType::SambaNova => {
                let api_base = config.sambanova_api_base.as_ref()
                    .ok_or_else(|| ProviderError::Configuration(
                        "SAMBANOVA_API_BASE required".to_string()
                    ))?;
                let api_key = config.sambanova_api_key.as_ref()
                    .ok_or_else(|| ProviderError::Configuration(
                        "SAMBANOVA_API_KEY required".to_string()
                    ))?;
                let model = config.sambanova_model.as_ref()
                    .ok_or_else(|| ProviderError::Configuration(
                        "SAMBANOVA_MODEL required".to_string()
                    ))?;

                Ok(Arc::new(SambaNovaProvider::new(
                    api_base.clone(),
                    api_key.clone(),
                    model.clone(),
                )?))
            }

            LlmProviderType::AzureAI => {
                let endpoint = config.azure_endpoint.as_ref()
                    .ok_or_else(|| ProviderError::Configuration(
                        "AZURE_AI_ENDPOINT required".to_string()
                    ))?;
                let api_key = config.azure_api_key.as_ref()
                    .ok_or_else(|| ProviderError::Configuration(
                        "AZURE_AI_API_KEY required".to_string()
                    ))?;
                let deployment = config.azure_deployment.as_ref()
                    .ok_or_else(|| ProviderError::Configuration(
                        "AZURE_AI_DEPLOYMENT required".to_string()
                    ))?;
                let api_version = config.azure_api_version.as_ref()
                    .ok_or_else(|| ProviderError::Configuration(
                        "AZURE_AI_API_VERSION required".to_string()
                    ))?;

                Ok(Arc::new(AzureAIProvider::new(
                    endpoint.clone(),
                    api_key.clone(),
                    deployment.clone(),
                    api_version.clone(),
                )?))
            }
        }
    }

    pub fn create_with_fallback(
        config: &LlmProviderConfig
    ) -> (Arc<dyn LlmProvider>, Option<Arc<dyn LlmProvider>>) {
        let primary = Self::create_provider(config)
            .expect("Primary provider configuration failed");

        let fallback = config.fallback_provider.as_ref()
            .and_then(|fallback_type| {
                let mut fallback_config = config.clone();
                fallback_config.provider_type = *fallback_type;
                Self::create_provider(&fallback_config).ok()
            });

        (primary, fallback)
    }
}
```

### Environment Variables

```bash
# LLM Provider Selection
LLM_PROVIDER=sambanova                    # or "azure" or "azureai"
LLM_FALLBACK_PROVIDER=                    # Optional fallback

# SambaNova Configuration (existing)
SAMBANOVA_API_BASE=https://api.sambanova.ai/v1
SAMBANOVA_API_KEY=your-key-here
SAMBANOVA_MODEL=Meta-Llama-3.3-70B-Instruct

# Azure AI Foundry Configuration (new)
AZURE_AI_ENDPOINT=https://your-resource.openai.azure.com
AZURE_AI_API_KEY=your-azure-key
AZURE_AI_DEPLOYMENT=gpt-4o                # Deployment name in Azure
AZURE_AI_API_VERSION=2024-02-15-preview   # Optional, has default

# Common Configuration (existing)
CHAT_MAX_TOKENS=2048
CHAT_MAX_CONTEXT_MESSAGES=20
CHAT_MAX_MESSAGE_LENGTH=4000
CHAT_DAILY_MESSAGE_QUOTA=10000
CHAT_RATE_LIMIT_PER_MINUTE=100
```

### Migration Path

#### Phase 1: Refactor Existing Code (No Breaking Changes)
1. Create provider abstraction layer
2. Migrate SambaNova logic to `SambaNovaProvider` implementation
3. Update `SendMessageUseCase` to use provider trait
4. All tests pass with SambaNova as default

#### Phase 2: Add Azure AI Support
1. Implement `AzureAIProvider`
2. Add Azure configuration loading
3. Implement provider factory and selection
4. Add provider-specific tests

#### Phase 3: Optional Enhancements
1. Provider health monitoring
2. Automatic fallback on provider failure
3. Usage tracking per provider
4. Admin UI for provider management

## Implementation Plan

### File Changes

#### New Files
```
backend/src/infrastructure/llm/
├── mod.rs                    # Module exports
├── provider.rs               # LlmProvider trait + types
├── sambanova.rs              # SambaNovaProvider implementation
├── azure_ai.rs               # AzureAIProvider implementation
└── factory.rs                # Provider factory

backend/src/config/
└── llm_providers.rs          # Provider configuration

backend/tests/integration/
└── llm_providers_test.rs     # Integration tests
```

#### Modified Files
```
backend/src/application/chat/send_message.rs  # Use provider trait
backend/src/config/chat.rs                     # Update configuration
backend/src/config/mod.rs                      # Export new config
backend/Cargo.toml                             # Add dependencies
.env.example                                   # Document new vars
```

### Dependencies

Add to `backend/Cargo.toml`:
```toml
[dependencies]
async-openai = { version = "0.23", features = ["azure"] }
```

### Testing Strategy

#### Unit Tests
- Provider trait implementations
- Configuration loading and validation
- Provider factory creation logic
- Error handling for each provider

#### Integration Tests
```rust
#[tokio::test]
async fn test_sambanova_provider_streaming() {
    // Test SambaNova provider with real/mock API
}

#[tokio::test]
async fn test_azure_ai_provider_streaming() {
    // Test Azure AI provider with real/mock API
}

#[tokio::test]
async fn test_provider_fallback() {
    // Test fallback from primary to secondary provider
}
```

#### Manual Testing
- [ ] Send message with SambaNova provider (existing)
- [ ] Send message with Azure AI provider (new)
- [ ] Switch providers via environment variable
- [ ] Test fallback behavior on provider failure
- [ ] Verify streaming works for both providers
- [ ] Test rate limiting per provider

## Azure AI Foundry Setup Guide

### Prerequisites
1. Azure subscription with AI Foundry access
2. Azure AI Foundry project created
3. Model deployment configured in Azure

### Setup Steps

#### 1. Create Azure AI Foundry Resources
```bash
# Using Azure CLI
az login

# Create resource group
az group create --name cobalt-ai-rg --location eastus

# Create Azure AI Foundry hub (if not exists)
az ml workspace create \
  --name cobalt-ai-hub \
  --resource-group cobalt-ai-rg \
  --kind hub

# Deploy a model (example: GPT-4o)
az ml online-deployment create \
  --name gpt-4o-deployment \
  --model gpt-4o \
  --workspace-name cobalt-ai-hub \
  --resource-group cobalt-ai-rg
```

#### 2. Get Credentials
```bash
# Get endpoint
az ml workspace show \
  --name cobalt-ai-hub \
  --resource-group cobalt-ai-rg \
  --query "endpoint" -o tsv

# Get API key
az ml workspace get-keys \
  --name cobalt-ai-hub \
  --resource-group cobalt-ai-rg
```

#### 3. Configure Environment
```bash
# Add to .env
AZURE_AI_ENDPOINT=https://cobalt-ai-hub.openai.azure.com
AZURE_AI_API_KEY=your-api-key-here
AZURE_AI_DEPLOYMENT=gpt-4o-deployment
AZURE_AI_API_VERSION=2024-02-15-preview

# Set as active provider
LLM_PROVIDER=azure
```

## API Compatibility

### Request Format (Unchanged)
```json
POST /api/v1/chat/sessions/{session_id}/messages
{
  "content": "Hello, world!"
}
```

### Response Format (Unchanged)
```
data: {"content":"Hello","done":false}
data: {"content":"!","done":false}
data: {"content":" How","done":false}
data: [DONE]
```

### Provider Transparency
- Frontend remains unchanged
- API contracts remain identical
- Provider selection is transparent to clients
- SSE streaming format consistent across providers

## Monitoring & Observability

### Metrics to Track
```rust
// Provider-specific metrics
- llm_requests_total{provider="sambanova", status="success"}
- llm_requests_total{provider="azure", status="error"}
- llm_request_duration_seconds{provider="sambanova"}
- llm_tokens_used_total{provider="azure"}
- llm_cost_usd{provider="azure"}

// Fallback metrics
- llm_fallback_triggered_total{from="sambanova", to="azure"}
```

### Logging
```rust
tracing::info!(
    provider = provider.provider_name(),
    model = %model,
    "Initiating LLM stream request"
);

tracing::error!(
    provider = provider.provider_name(),
    error = %e,
    "Provider health check failed"
);
```

## Security Considerations

### API Key Management
- ✅ Environment variable storage (current)
- ✅ Separate keys per provider
- 🔄 Future: Azure Key Vault integration
- 🔄 Future: Secrets rotation support

### Data Privacy
- All messages encrypted in transit (TLS)
- No logging of message content (existing)
- Provider-specific data residency (Azure regions)
- Compliance with provider ToS

### Authentication
- User authentication unchanged
- Provider authentication via API keys
- Azure: Supports Managed Identity (future enhancement)

## Cost Analysis

### SambaNova (Current)
- Free tier: Unknown
- Pay-as-you-go: Variable pricing
- Enterprise: Contact for pricing

### Azure AI Foundry
- Pay-per-token pricing
- Example: GPT-4o
  - Input: $2.50 / 1M tokens
  - Output: $10.00 / 1M tokens
- Reserved capacity discounts available
- Azure credits applicable

### Cost Optimization Strategies
1. Model selection per use case
2. Token limit enforcement
3. Caching responses (future)
4. Intelligent provider routing by cost
5. Usage monitoring and alerting

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Azure API changes | High | Pin API versions, monitor deprecations |
| Provider downtime | High | Implement fallback provider |
| Cost overruns | Medium | Rate limiting, quotas, monitoring |
| Configuration errors | Medium | Validation, comprehensive tests |
| Latency increase | Low | Regional deployment, monitoring |
| Vendor lock-in | Low | Provider abstraction layer |

## Success Metrics

### Technical Success
- [ ] Zero breaking changes to existing API
- [ ] <100ms overhead for provider abstraction
- [ ] 99.9% streaming reliability with Azure
- [ ] <5% error rate on provider switching

### Business Success
- [ ] Support Azure enterprise customers
- [ ] Reduce vendor dependency risk
- [ ] Enable A/B testing of models
- [ ] Maintain or improve user experience

## Timeline

### Phase 1: Foundation (Week 1-2)
- [ ] Design review and approval
- [ ] Create provider abstraction
- [ ] Refactor SambaNova to provider trait
- [ ] Write comprehensive tests

### Phase 2: Azure Integration (Week 2-3)
- [ ] Implement AzureAIProvider
- [ ] Add configuration management
- [ ] Provider factory and selection
- [ ] Integration testing

### Phase 3: Testing & Refinement (Week 3-4)
- [ ] End-to-end testing
- [ ] Performance benchmarking
- [ ] Documentation
- [ ] Deployment preparation

### Phase 4: Rollout (Week 4+)
- [ ] Deploy to staging
- [ ] User acceptance testing
- [ ] Production deployment
- [ ] Monitoring and optimization

## Alternatives Considered

### 1. LangChain Integration
**Pros**: Rich ecosystem, many integrations
**Cons**: Heavy dependency, Python-focused, overkill for our needs
**Decision**: Build custom abstraction for better control

### 2. Direct Azure SDK
**Pros**: Official support, comprehensive
**Cons**: Different from OpenAI format, more refactoring
**Decision**: Use OpenAI-compatible Azure API for consistency

### 3. OpenAI Native Support
**Pros**: Most popular, well-documented
**Cons**: Higher costs, no Azure benefits
**Decision**: Azure AI Foundry provides OpenAI models with Azure benefits

## Open Questions

1. **Model Selection**: Should users select models per message or per session?
2. **Fallback Strategy**: Automatic failover or manual intervention?
3. **Cost Tracking**: Do we need per-user cost attribution?
4. **Admin UI**: Should we build a provider management dashboard?
5. **Rate Limits**: Per-provider or combined rate limiting?

## References

- [Azure AI Foundry Documentation](https://learn.microsoft.com/en-us/azure/ai-services/openai/)
- [Azure OpenAI Service Pricing](https://azure.microsoft.com/en-us/pricing/details/cognitive-services/openai-service/)
- [async-openai Azure Support](https://docs.rs/async-openai/latest/async_openai/config/struct.AzureConfig.html)
- [OpenAI API Compatibility](https://platform.openai.com/docs/api-reference)

## Appendix

### A. Example Azure Deployment Script
```bash
#!/bin/bash
# deploy-azure-ai.sh

set -e

RESOURCE_GROUP="cobalt-ai-rg"
LOCATION="eastus"
WORKSPACE="cobalt-ai-hub"
DEPLOYMENT="gpt-4o-deployment"

# Create resources
az group create --name "$RESOURCE_GROUP" --location "$LOCATION"
az ml workspace create --kind hub --name "$WORKSPACE" --resource-group "$RESOURCE_GROUP"

# Deploy model
az ml online-deployment create \
  --name "$DEPLOYMENT" \
  --model gpt-4o \
  --workspace-name "$WORKSPACE" \
  --resource-group "$RESOURCE_GROUP"

# Get credentials
echo "AZURE_AI_ENDPOINT=$(az ml workspace show --name "$WORKSPACE" --resource-group "$RESOURCE_GROUP" --query 'endpoint' -o tsv)"
echo "AZURE_AI_API_KEY=$(az ml workspace get-keys --name "$WORKSPACE" --resource-group "$RESOURCE_GROUP" --query 'primaryKey' -o tsv)"
echo "AZURE_AI_DEPLOYMENT=$DEPLOYMENT"
```

### B. Configuration Examples

**Development (.env.development)**
```bash
LLM_PROVIDER=sambanova
SAMBANOVA_API_KEY=dev-key
SAMBANOVA_MODEL=Meta-Llama-3.3-70B-Instruct
```

**Production with Fallback (.env.production)**
```bash
LLM_PROVIDER=azure
LLM_FALLBACK_PROVIDER=sambanova

AZURE_AI_ENDPOINT=https://prod.openai.azure.com
AZURE_AI_API_KEY={{secrets.AZURE_AI_KEY}}
AZURE_AI_DEPLOYMENT=gpt-4o-prod

SAMBANOVA_API_KEY={{secrets.SAMBANOVA_KEY}}
SAMBANOVA_MODEL=Meta-Llama-3.3-70B-Instruct
```

---

**Next Steps**:
1. Review and approve this proposal
2. Create implementation tasks
3. Set up Azure AI Foundry test environment
4. Begin Phase 1 implementation
