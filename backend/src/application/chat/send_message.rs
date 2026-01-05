//! Send message use case with LLM streaming

use std::sync::Arc;
use uuid::Uuid;
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionRequestAssistantMessageArgs,
        CreateChatCompletionRequestArgs, Role,
    },
};
use futures::Stream;
use std::pin::Pin;

use crate::domain::chat::{
    entity::{ChatMessage, ChatSession},
    repository::{ChatRepository, RepositoryError, RepositoryResult},
    value_objects::MessageRole,
};

/// Request to send a message in a chat session
#[derive(Debug, Clone)]
pub struct SendMessageRequest {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
}

/// Streaming chunk from LLM response
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub is_final: bool,
}

/// Configuration for LLM client
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub api_base: String,
    pub api_key: String,
    pub model: String,
    pub max_context_messages: u64,
    pub max_tokens: u16,
}

/// Use case for sending messages with streaming LLM responses
pub struct SendMessageUseCase {
    repository: Arc<dyn ChatRepository>,
    llm_config: LlmConfig,
}

impl SendMessageUseCase {
    /// Create a new use case instance
    #[must_use]
    pub fn new(repository: Arc<dyn ChatRepository>, llm_config: LlmConfig) -> Self {
        Self {
            repository,
            llm_config,
        }
    }

    /// Execute the use case to send a message and stream LLM response
    ///
    /// # Errors
    /// Returns `RepositoryError` if:
    /// - Session not found
    /// - User not authorized
    /// - Message validation fails
    /// - Repository operations fail
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
            .find_recent_messages(request.session_id, self.llm_config.max_context_messages)
            .await?;

        // Build LLM request
        let llm_messages = self.build_llm_messages(&context_messages)?;

        // Create streaming response
        let stream = self.create_llm_stream(llm_messages, request.session_id).await?;

        Ok(stream)
    }

    /// Build LLM API messages from domain messages
    fn build_llm_messages(
        &self,
        messages: &[ChatMessage],
    ) -> RepositoryResult<Vec<ChatCompletionRequestMessage>> {
        let mut llm_messages = Vec::new();

        for msg in messages {
            let message = match msg.role {
                MessageRole::System => ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(msg.content.clone())
                        .build()
                        .map_err(|e| RepositoryError::ValidationError(e.to_string()))?,
                ),
                MessageRole::User => ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(msg.content.clone())
                        .build()
                        .map_err(|e| RepositoryError::ValidationError(e.to_string()))?,
                ),
                MessageRole::Assistant => ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .content(msg.content.clone())
                        .build()
                        .map_err(|e| RepositoryError::ValidationError(e.to_string()))?,
                ),
            };
            llm_messages.push(message);
        }

        Ok(llm_messages)
    }

    /// Create streaming LLM response
    async fn create_llm_stream(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        session_id: Uuid,
    ) -> RepositoryResult<Pin<Box<dyn Stream<Item = Result<StreamChunk, String>> + Send>>> {
        // Configure OpenAI client for SambaNova API
        let config = OpenAIConfig::new()
            .with_api_base(&self.llm_config.api_base)
            .with_api_key(&self.llm_config.api_key);

        let client = Client::with_config(config);

        // Create streaming request
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.llm_config.model)
            .messages(messages)
            .max_tokens(self.llm_config.max_tokens)
            .stream(true)
            .build()
            .map_err(|e| RepositoryError::ValidationError(e.to_string()))?;

        tracing::info!("Initiating LLM stream request to: {}", self.llm_config.api_base);

        // Start streaming
        let mut stream = client
            .chat()
            .create_stream(request)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create LLM stream: {}", e);
                RepositoryError::DatabaseError(e.to_string())
            })?;

        tracing::info!("LLM stream created successfully");

        // Process stream and save assistant message
        let repository = Arc::clone(&self.repository);
        let mut accumulated_content = String::new();

        use futures::StreamExt;
        let output_stream = async_stream::stream! {
            tracing::info!("Starting LLM stream processing");
            let mut chunk_count = 0;

            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        tracing::debug!("Received stream response with {} choices", response.choices.len());

                        for choice in response.choices {
                            if let Some(content) = &choice.delta.content {
                                chunk_count += 1;
                                tracing::debug!("Chunk #{}: {} bytes", chunk_count, content.len());
                                accumulated_content.push_str(content);
                                yield Ok(StreamChunk {
                                    content: content.clone(),
                                    is_final: false,
                                });
                            }

                            // Check if streaming is done
                            if let Some(reason) = &choice.finish_reason {
                                tracing::info!("Stream finished: reason={:?}, total_chunks={}, content_length={}",
                                    reason, chunk_count, accumulated_content.len());

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
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("LLM stream error: {}", e);
                        yield Err(format!("Stream error: {}", e));
                        return;
                    }
                }
            }

            tracing::warn!("Stream ended without finish_reason (chunks received: {})", chunk_count);
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

        let config = LlmConfig {
            api_base: "http://localhost".to_string(),
            api_key: "test".to_string(),
            model: "test-model".to_string(),
            max_context_messages: 20,
            max_tokens: 2048,
        };

        let use_case = SendMessageUseCase::new(mock_repo.clone(), config);

        // Test unauthorized user
        let request = SendMessageRequest {
            session_id,
            user_id: Uuid::new_v4(), // Different user
            content: "Hello".to_string(),
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

        let config = LlmConfig {
            api_base: "http://localhost".to_string(),
            api_key: "test".to_string(),
            model: "test-model".to_string(),
            max_context_messages: 20,
            max_tokens: 2048,
        };

        let use_case = SendMessageUseCase::new(mock_repo, config);

        let request = SendMessageRequest {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            content: "Hello".to_string(),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, RepositoryError::SessionNotFound(_)));
        }
    }
}
