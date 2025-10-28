//! LLM Provider trait abstraction
//!
//! Defines the interface that all LLM providers must implement for streaming chat completions.

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

/// Request for creating a chat completion
#[derive(Debug, Clone)]
pub struct ChatCompletionRequest {
    /// The model ID to use (provider-specific)
    pub model: String,
    /// Chat messages in the conversation
    pub messages: Vec<ChatMessage>,
    /// Maximum tokens to generate
    pub max_tokens: u16,
    /// Whether to stream the response
    pub stream: bool,
}

/// A message in a chat conversation
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

/// Role of a message in the conversation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

/// A chunk of streamed response
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Content of this chunk
    pub content: String,
    /// Whether this is the final chunk
    pub is_final: bool,
    /// Optional finish reason
    pub finish_reason: Option<String>,
}

/// Error types for LLM provider operations
#[derive(Debug, thiserror::Error)]
pub enum LlmProviderError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

pub type LlmResult<T> = Result<T, LlmProviderError>;

/// Trait that all LLM providers must implement
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Check if the provider is available/configured
    fn is_available(&self) -> bool;

    /// Create a streaming chat completion
    ///
    /// Returns a stream of chunks that can be consumed asynchronously.
    async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> LlmResult<Pin<Box<dyn Stream<Item = Result<StreamChunk, LlmProviderError>> + Send>>>;

    /// Get the maximum context window for a model
    fn max_context_tokens(&self, model: &str) -> Option<u32>;

    /// Get the maximum output tokens for a model
    fn max_output_tokens(&self, model: &str) -> Option<u32>;
}

/// Helper to convert domain role to provider role
impl From<crate::domain::chat::value_objects::MessageRole> for ChatRole {
    fn from(role: crate::domain::chat::value_objects::MessageRole) -> Self {
        match role {
            crate::domain::chat::value_objects::MessageRole::System => ChatRole::System,
            crate::domain::chat::value_objects::MessageRole::User => ChatRole::User,
            crate::domain::chat::value_objects::MessageRole::Assistant => ChatRole::Assistant,
        }
    }
}

/// Helper to convert domain message to provider message
impl From<&crate::domain::chat::entity::ChatMessage> for ChatMessage {
    fn from(msg: &crate::domain::chat::entity::ChatMessage) -> Self {
        ChatMessage {
            role: msg.role.into(),
            content: msg.content.clone(),
        }
    }
}
