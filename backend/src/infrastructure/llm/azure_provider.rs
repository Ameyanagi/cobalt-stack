//! Azure AI LLM provider implementation
//!
//! Implements the LlmProvider trait using Azure AI's OpenAI-compatible API.

use super::provider::{
    ChatCompletionRequest, ChatMessage as ProviderMessage, ChatRole, LlmProvider,
    LlmProviderError, LlmResult, StreamChunk,
};
use async_openai::{
    config::AzureConfig,
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

/// Azure AI provider using OpenAI-compatible API
pub struct AzureAIProvider {
    endpoint: String,
    api_key: String,
    model_registry: ModelRegistry,
}

impl AzureAIProvider {
    /// Create a new Azure AI provider
    pub fn new(endpoint: String, api_key: String, model_registry: ModelRegistry) -> Self {
        Self {
            endpoint,
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

    /// Parse Azure endpoint URL to extract base URL and API version
    /// Expected format: https://host/path?api-version=xxx
    fn parse_azure_endpoint(&self) -> LlmResult<(String, String)> {
        // Split by '?' to separate path from query parameters
        let parts: Vec<&str> = self.endpoint.split('?').collect();
        if parts.len() != 2 {
            return Err(LlmProviderError::ConfigError(format!(
                "Invalid Azure endpoint format: {}. Expected format: https://host/path?api-version=xxx",
                self.endpoint
            )));
        }

        let url_path = parts[0];
        let query_string = parts[1];

        // Extract base URL (everything before /models or /chat)
        let api_base = if let Some(pos) = url_path.find("/models") {
            url_path[..pos].to_string()
        } else if let Some(pos) = url_path.find("/chat") {
            url_path[..pos].to_string()
        } else {
            url_path.to_string()
        };

        // Extract API version from query string
        let api_version = query_string
            .split('&')
            .find(|param| param.starts_with("api-version="))
            .and_then(|param| param.strip_prefix("api-version="))
            .ok_or_else(|| {
                LlmProviderError::ConfigError(format!(
                    "Missing api-version parameter in Azure endpoint: {}",
                    self.endpoint
                ))
            })?
            .to_string();

        tracing::debug!(
            "Parsed Azure endpoint: base={}, api_version={}",
            api_base,
            api_version
        );

        Ok((api_base, api_version))
    }
}

#[async_trait]
impl LlmProvider for AzureAIProvider {
    fn name(&self) -> &str {
        "Azure AI"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty() && !self.endpoint.is_empty()
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

        // Parse Azure endpoint URL to extract base URL and API version
        // Expected format: https://host/models/chat/completions?api-version=xxx
        let (api_base, api_version) = self.parse_azure_endpoint()?;

        // Configure Azure client
        let config = AzureConfig::new()
            .with_api_base(&api_base)
            .with_api_version(&api_version)
            .with_deployment_id(&model_config.model_id) // Use Azure deployment name
            .with_api_key(&self.api_key);

        let client = Client::with_config(config);

        // Create streaming request
        // Note: AzureConfig already handles deployment_id, so we don't need to set model here
        let openai_request = CreateChatCompletionRequestArgs::default()
            .messages(openai_messages)
            .max_tokens(request.max_tokens)
            .stream(true)
            .build()
            .map_err(|e| LlmProviderError::InvalidRequest(e.to_string()))?;

        tracing::info!(
            "Azure AI: Initiating stream request with deployment {} (api_version: {})",
            model_config.model_id,
            api_version
        );

        // Start streaming
        let mut api_stream = client
            .chat()
            .create_stream(openai_request)
            .await
            .map_err(|e| {
                tracing::error!("Azure AI: Failed to create stream: {}", e);
                LlmProviderError::ApiError(e.to_string())
            })?;

        tracing::info!("Azure AI: Stream created successfully");

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
                                tracing::debug!("Azure AI: Chunk #{}: {} bytes", chunk_count, content.len());

                                yield Ok(StreamChunk {
                                    content: content.clone(),
                                    is_final: false,
                                    finish_reason: None,
                                });
                            }

                            // Handle completion
                            if let Some(reason) = &choice.finish_reason {
                                tracing::info!(
                                    "Azure AI: Stream finished: reason={:?}, chunks={}",
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
                        tracing::error!("Azure AI: Stream error: {}", e);
                        yield Err(LlmProviderError::StreamError(e.to_string()));
                        return;
                    }
                }
            }

            tracing::warn!("Azure AI: Stream ended without finish_reason");
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

        let provider = AzureAIProvider::new(
            "https://test.azure.com/models/chat/completions".to_string(),
            "test-key".to_string(),
            registry,
        );

        assert_eq!(provider.name(), "Azure AI");
        assert!(provider.is_available());
    }

    #[test]
    fn test_provider_unavailable() {
        // Skip if models.toml not available
        let Ok(registry) = ModelRegistry::load() else {
            eprintln!("Skipping test: models.toml not found");
            return;
        };

        let provider = AzureAIProvider::new(String::new(), String::new(), registry);
        assert!(!provider.is_available());
    }
}
