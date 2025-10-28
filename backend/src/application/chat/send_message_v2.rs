//! Send message use case with provider abstraction
//!
//! Refactored version using LlmProvider trait and ProviderFactory

use std::sync::Arc;
use uuid::Uuid;
use futures::Stream;
use std::pin::Pin;

use crate::domain::chat::{
    entity::ChatMessage,
    repository::{ChatRepository, RepositoryError, RepositoryResult},
    value_objects::MessageRole,
};
use crate::infrastructure::llm::{
    ProviderFactory, ChatCompletionRequest, ChatMessage as ProviderMessage, LlmProviderError,
};

/// Request to send a message in a chat session
#[derive(Debug, Clone)]
pub struct SendMessageRequest {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    /// Optional model ID to use (defaults to registry default)
    pub model_id: Option<String>,
}

/// Streaming chunk from LLM response
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub is_final: bool,
}

/// Configuration for the use case
#[derive(Debug, Clone)]
pub struct UseCaseConfig {
    pub max_context_messages: u64,
    pub max_tokens: u16,
}

/// Use case for sending messages with streaming LLM responses
pub struct SendMessageUseCase {
    repository: Arc<dyn ChatRepository>,
    provider_factory: Arc<ProviderFactory>,
    config: UseCaseConfig,
}

impl SendMessageUseCase {
    /// Create a new use case instance
    ///
    /// # Errors
    /// Returns error if provider factory cannot be initialized
    pub fn new(
        repository: Arc<dyn ChatRepository>,
        config: UseCaseConfig,
    ) -> Result<Self, LlmProviderError> {
        let provider_factory = Arc::new(ProviderFactory::new()?);

        Ok(Self {
            repository,
            provider_factory,
            config,
        })
    }

    /// Execute the use case to send a message and stream LLM response
    ///
    /// # Errors
    /// Returns `RepositoryError` if:
    /// - Session not found
    /// - User not authorized
    /// - Message validation fails
    /// - Repository operations fail
    /// - Provider/model errors
    pub async fn execute(
        &self,
        request: SendMessageRequest,
    ) -> RepositoryResult<Pin<Box<dyn Stream<Item = Result<StreamChunk, String>> + Send>>> {
        // Verify session exists and belongs to user
        let session = self
            .repository
            .find_session_by_id(request.session_id)
            .await?
            .ok_or(RepositoryError::SessionNotFound(request.session_id))?;

        if session.user_id != request.user_id {
            return Err(RepositoryError::ValidationError(
                "User not authorized for this session".to_string(),
            ));
        }

        // Create and save user message
        let user_message = ChatMessage::new(
            request.session_id,
            MessageRole::User,
            request.content.clone(),
        )
        .map_err(|e| RepositoryError::ValidationError(e))?;

        self.repository.save_message(&user_message).await?;

        // Get recent context messages
        let context_messages = self
            .repository
            .find_recent_messages(request.session_id, self.config.max_context_messages)
            .await?;

        // Determine which model to use
        let model_id = request
            .model_id
            .as_deref()
            .unwrap_or_else(|| {
                self.provider_factory.model_registry().default_model().id.as_str()
            });

        tracing::info!(
            "Using model '{}' for session {}",
            model_id,
            request.session_id
        );

        // Get provider for the model
        let provider = self
            .provider_factory
            .get_provider_for_model(model_id)
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        tracing::info!("Selected provider: {}", provider.name());

        // Build provider request
        let provider_messages: Vec<ProviderMessage> = context_messages
            .iter()
            .map(|msg| msg.into())
            .collect();

        let llm_request = ChatCompletionRequest {
            model: model_id.to_string(),
            messages: provider_messages,
            max_tokens: self.config.max_tokens,
            stream: true,
        };

        // Create streaming response
        let stream = self
            .create_llm_stream(provider, llm_request, request.session_id)
            .await?;

        Ok(stream)
    }

    /// Create streaming LLM response with message persistence
    async fn create_llm_stream(
        &self,
        provider: Arc<dyn crate::infrastructure::llm::LlmProvider>,
        request: ChatCompletionRequest,
        session_id: Uuid,
    ) -> RepositoryResult<Pin<Box<dyn Stream<Item = Result<StreamChunk, String>> + Send>>> {
        // Start streaming from provider
        let mut provider_stream = provider
            .create_chat_completion_stream(request)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create provider stream: {}", e);
                RepositoryError::DatabaseError(e.to_string())
            })?;

        // Process stream and save assistant message
        let repository = Arc::clone(&self.repository);
        let mut accumulated_content = String::new();

        use futures::StreamExt;
        let output_stream = async_stream::stream! {
            tracing::info!("Starting provider stream processing");
            let mut chunk_count = 0;

            while let Some(result) = provider_stream.next().await {
                match result {
                    Ok(chunk) => {
                        if !chunk.content.is_empty() {
                            chunk_count += 1;
                            tracing::debug!("Chunk #{}: {} bytes", chunk_count, chunk.content.len());
                            accumulated_content.push_str(&chunk.content);

                            yield Ok(StreamChunk {
                                content: chunk.content,
                                is_final: false,
                            });
                        }

                        // Check if streaming is done
                        if chunk.is_final {
                            tracing::info!(
                                "Stream finished: finish_reason={:?}, chunks={}, content_length={}",
                                chunk.finish_reason,
                                chunk_count,
                                accumulated_content.len()
                            );

                            // Save complete assistant message
                            if !accumulated_content.is_empty() {
                                let assistant_message = match ChatMessage::new(
                                    session_id,
                                    MessageRole::Assistant,
                                    accumulated_content.clone(),
                                ) {
                                    Ok(msg) => msg,
                                    Err(e) => {
                                        tracing::error!("Failed to create message: {}", e);
                                        yield Err(format!("Failed to create message: {}", e));
                                        return;
                                    }
                                };

                                if let Err(e) = repository.save_message(&assistant_message).await {
                                    tracing::error!("Failed to save message: {}", e);
                                    yield Err(format!("Failed to save message: {}", e));
                                    return;
                                }

                                tracing::info!("Assistant message saved successfully");
                            }

                            yield Ok(StreamChunk {
                                content: String::new(),
                                is_final: true,
                            });
                            return;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Provider stream error: {}", e);
                        yield Err(format!("Stream error: {}", e));
                        return;
                    }
                }
            }

            tracing::warn!("Stream ended without final chunk (chunks: {})", chunk_count);
        };

        Ok(Box::pin(output_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::chat::{entity::ChatSession, repository::RepositoryError};
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockChatRepository {
        sessions: Mutex<Vec<ChatSession>>,
        messages: Mutex<Vec<ChatMessage>>,
    }

    #[async_trait]
    impl ChatRepository for MockChatRepository {
        async fn create_session(&self, _session: &ChatSession) -> RepositoryResult<()> {
            unimplemented!()
        }

        async fn find_session_by_id(&self, id: Uuid) -> RepositoryResult<Option<ChatSession>> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions.iter().find(|s| s.id == id).cloned())
        }

        async fn find_sessions_by_user(
            &self,
            _user_id: Uuid,
            _page: u64,
            _per_page: u64,
        ) -> RepositoryResult<(Vec<ChatSession>, u64)> {
            unimplemented!()
        }

        async fn update_session(&self, _session: &ChatSession) -> RepositoryResult<()> {
            unimplemented!()
        }

        async fn delete_session(&self, _id: Uuid) -> RepositoryResult<()> {
            unimplemented!()
        }

        async fn save_message(&self, message: &ChatMessage) -> RepositoryResult<()> {
            self.messages.lock().unwrap().push(message.clone());
            Ok(())
        }

        async fn find_messages_by_session(
            &self,
            _session_id: Uuid,
            _limit: Option<u64>,
        ) -> RepositoryResult<Vec<ChatMessage>> {
            unimplemented!()
        }

        async fn find_recent_messages(
            &self,
            session_id: Uuid,
            limit: u64,
        ) -> RepositoryResult<Vec<ChatMessage>> {
            let messages = self.messages.lock().unwrap();
            let recent: Vec<_> = messages
                .iter()
                .filter(|m| m.session_id == session_id)
                .rev()
                .take(limit as usize)
                .cloned()
                .collect();
            Ok(recent.into_iter().rev().collect())
        }
    }

    #[tokio::test]
    async fn test_send_message_validation() {
        let user_id = Uuid::new_v4();
        let session = ChatSession::new(user_id, "Test".to_string()).unwrap();
        let session_id = session.id;

        let mock_repo = Arc::new(MockChatRepository {
            sessions: Mutex::new(vec![session]),
            messages: Mutex::new(Vec::new()),
        });

        let config = UseCaseConfig {
            max_context_messages: 20,
            max_tokens: 2048,
        };

        // Skip test if models.toml not available
        let Ok(use_case) = SendMessageUseCase::new(mock_repo.clone(), config) else {
            eprintln!("Skipping test: ProviderFactory initialization failed");
            return;
        };

        // Test unauthorized user
        let request = SendMessageRequest {
            session_id,
            user_id: Uuid::new_v4(), // Different user
            content: "Hello".to_string(),
            model_id: None,
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, RepositoryError::ValidationError(_)));
        }
    }

    #[tokio::test]
    async fn test_send_message_session_not_found() {
        let mock_repo = Arc::new(MockChatRepository {
            sessions: Mutex::new(Vec::new()),
            messages: Mutex::new(Vec::new()),
        });

        let config = UseCaseConfig {
            max_context_messages: 20,
            max_tokens: 2048,
        };

        // Skip test if models.toml not available
        let Ok(use_case) = SendMessageUseCase::new(mock_repo, config) else {
            eprintln!("Skipping test: ProviderFactory initialization failed");
            return;
        };

        let request = SendMessageRequest {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            content: "Hello".to_string(),
            model_id: None,
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, RepositoryError::SessionNotFound(_)));
        }
    }
}
