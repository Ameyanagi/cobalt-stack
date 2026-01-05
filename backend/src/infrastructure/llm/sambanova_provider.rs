//! SambaNova LLM provider implementation
//!
//! Implements the LlmProvider trait using SambaNova's OpenAI-compatible API.

use super::provider::{
    ChatCompletionRequest, ChatMessage as ProviderMessage, ChatRole, LlmProvider,
    LlmProviderError, LlmResult, StreamChunk,
};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::pin::Pin;

use crate::infrastructure::llm::{ModelConfig, ModelRegistry};

/// SambaNova provider using OpenAI-compatible API
pub struct SambaNovaProvider {
    api_base: String,
    api_key: String,
    model_registry: ModelRegistry,
}

impl SambaNovaProvider {
    /// Create a new SambaNova provider
    pub fn new(api_base: String, api_key: String, model_registry: ModelRegistry) -> Self {
        Self {
            api_base,
            api_key,
            model_registry,
        }
    }

    /// Convert provider messages to OpenAI API format
    fn convert_messages(
        &self,
        messages: Vec<ProviderMessage>,
    ) -> LlmResult<Vec<ChatCompletionRequestMessage>> {
        messages
            .into_iter()
            .map(|msg| match msg.role {
                ChatRole::System => ChatCompletionRequestSystemMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .map(ChatCompletionRequestMessage::System)
                    .map_err(|e| LlmProviderError::InvalidRequest(e.to_string())),
                ChatRole::User => ChatCompletionRequestUserMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .map(ChatCompletionRequestMessage::User)
                    .map_err(|e| LlmProviderError::InvalidRequest(e.to_string())),
                ChatRole::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .map(ChatCompletionRequestMessage::Assistant)
                    .map_err(|e| LlmProviderError::InvalidRequest(e.to_string())),
            })
            .collect()
    }

    /// Get model configuration from registry
    fn get_model_config(&self, model_id: &str) -> LlmResult<&ModelConfig> {
        self.model_registry
            .get_model(model_id)
            .map_err(|e| LlmProviderError::ConfigError(e.to_string()))
    }
}

#[async_trait]
impl LlmProvider for SambaNovaProvider {
    fn name(&self) -> &str {
        "SambaNova"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty() && !self.api_base.is_empty()
    }

    async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> LlmResult<Pin<Box<dyn Stream<Item = Result<StreamChunk, LlmProviderError>> + Send>>> {
        // Get model config to retrieve provider-specific model_id
        let model_config = self.get_model_config(&request.model)?;

        // Verify streaming is supported
        if !model_config.supports_streaming {
            return Err(LlmProviderError::InvalidRequest(format!(
                "Model {} does not support streaming",
                request.model
            )));
        }

        // Convert messages to OpenAI format
        let openai_messages = self.convert_messages(request.messages)?;

        // Configure OpenAI client for SambaNova API
        let config = OpenAIConfig::new()
            .with_api_base(&self.api_base)
            .with_api_key(&self.api_key);

        let client = Client::with_config(config);

        // Create streaming request using provider-specific model_id
        let openai_request = CreateChatCompletionRequestArgs::default()
            .model(&model_config.model_id) // Use provider-specific model_id
            .messages(openai_messages)
            .max_tokens(request.max_tokens)
            .stream(true)
            .build()
            .map_err(|e| LlmProviderError::InvalidRequest(e.to_string()))?;

        tracing::info!(
            "SambaNova: Initiating stream request to {} with model {}",
            self.api_base,
            model_config.model_id
        );

        // Start streaming
        let mut api_stream = client
            .chat()
            .create_stream(openai_request)
            .await
            .map_err(|e| {
                tracing::error!("SambaNova: Failed to create stream: {}", e);
                LlmProviderError::ApiError(e.to_string())
            })?;

        tracing::info!("SambaNova: Stream created successfully");

        // Transform API stream to provider stream
        let output_stream = async_stream::stream! {
            let mut chunk_count = 0;

            while let Some(result) = api_stream.next().await {
                match result {
                    Ok(response) => {
                        for choice in response.choices {
                            // Handle content chunk
                            if let Some(content) = &choice.delta.content {
                                chunk_count += 1;
                                tracing::debug!("SambaNova: Chunk #{}: {} bytes", chunk_count, content.len());

                                yield Ok(StreamChunk {
                                    content: content.clone(),
                                    is_final: false,
                                    finish_reason: None,
                                });
                            }

                            // Handle completion
                            if let Some(reason) = &choice.finish_reason {
                                tracing::info!(
                                    "SambaNova: Stream finished: reason={:?}, chunks={}",
                                    reason,
                                    chunk_count
                                );

                                yield Ok(StreamChunk {
                                    content: String::new(),
                                    is_final: true,
                                    finish_reason: Some(format!("{:?}", reason)),
                                });
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("SambaNova: Stream error: {}", e);
                        yield Err(LlmProviderError::StreamError(e.to_string()));
                        return;
                    }
                }
            }

            tracing::warn!("SambaNova: Stream ended without finish_reason");
        };

        Ok(Box::pin(output_stream))
    }

    fn max_context_tokens(&self, model: &str) -> Option<u32> {
        self.model_registry
            .get_model(model)
            .ok()
            .map(|m| m.context_window)
    }

    fn max_output_tokens(&self, model: &str) -> Option<u32> {
        self.model_registry
            .get_model(model)
            .ok()
            .map(|m| m.max_output_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        // Skip if models.toml not available
        let Ok(registry) = ModelRegistry::load() else {
            eprintln!("Skipping test: models.toml not found");
            return;
        };

        let provider = SambaNovaProvider::new(
            "https://api.sambanova.ai/v1".to_string(),
            "test-key".to_string(),
            registry,
        );

        assert_eq!(provider.name(), "SambaNova");
        assert!(provider.is_available());
    }

    #[test]
    fn test_provider_unavailable() {
        // Skip if models.toml not available
        let Ok(registry) = ModelRegistry::load() else {
            eprintln!("Skipping test: models.toml not found");
            return;
        };

        let provider = SambaNovaProvider::new(String::new(), String::new(), registry);
        assert!(!provider.is_available());
    }

    #[test]
    fn test_model_metadata() {
        // Skip if models.toml not available
        let Ok(registry) = ModelRegistry::load() else {
            eprintln!("Skipping test: models.toml not found");
            return;
        };

        let provider = SambaNovaProvider::new(
            "https://api.sambanova.ai/v1".to_string(),
            "test-key".to_string(),
            registry,
        );

        // Test with default model
        let context = provider.max_context_tokens("llama-3.3-70b");
        assert!(context.is_some());
        assert!(context.unwrap() > 0);

        let output = provider.max_output_tokens("llama-3.3-70b");
        assert!(output.is_some());
        assert!(output.unwrap() > 0);
    }
}
