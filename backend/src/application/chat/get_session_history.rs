//! Get chat session history use case

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::chat::{
    entity::ChatMessage,
    repository::{ChatRepository, RepositoryResult},
};

/// Request to get session message history
#[derive(Debug, Clone)]
pub struct GetSessionHistoryRequest {
    pub session_id: Uuid,
    pub limit: Option<u64>,
}

/// Response containing message history
#[derive(Debug, Clone)]
pub struct GetSessionHistoryResponse {
    pub messages: Vec<ChatMessage>,
}

/// Use case for retrieving chat session history
pub struct GetSessionHistoryUseCase {
    repository: Arc<dyn ChatRepository>,
}

impl GetSessionHistoryUseCase {
    /// Create a new use case instance
    #[must_use]
    pub fn new(repository: Arc<dyn ChatRepository>) -> Self {
        Self { repository }
    }

    /// Execute the use case to get session history
    ///
    /// # Errors
    /// Returns `RepositoryError` if retrieval fails
    pub async fn execute(&self, request: GetSessionHistoryRequest) -> RepositoryResult<GetSessionHistoryResponse> {
        let messages = self
            .repository
            .find_messages_by_session(request.session_id, request.limit)
            .await?;

        Ok(GetSessionHistoryResponse { messages })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::chat::{entity::ChatSession, value_objects::MessageRole, repository::RepositoryError};
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Mutex;

    struct MockChatRepository {
        messages: Mutex<Vec<ChatMessage>>,
    }

    #[async_trait]
    impl ChatRepository for MockChatRepository {
        async fn create_session(&self, _session: &ChatSession) -> RepositoryResult<()> {
            unimplemented!()
        }

        async fn find_session_by_id(&self, _id: Uuid) -> RepositoryResult<Option<ChatSession>> {
            unimplemented!()
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

        async fn save_message(&self, _message: &ChatMessage) -> RepositoryResult<()> {
            unimplemented!()
        }

        async fn find_messages_by_session(
            &self,
            _session_id: Uuid,
            limit: Option<u64>,
        ) -> RepositoryResult<Vec<ChatMessage>> {
            let messages = self.messages.lock().unwrap();
            let result = if let Some(limit_value) = limit {
                messages.iter().take(limit_value as usize).cloned().collect()
            } else {
                messages.clone()
            };
            Ok(result)
        }

        async fn find_recent_messages(
            &self,
            _session_id: Uuid,
            _limit: u64,
        ) -> RepositoryResult<Vec<ChatMessage>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_get_session_history_all() {
        let session_id = Uuid::new_v4();
        let messages = vec![
            ChatMessage::new(session_id, MessageRole::User, "Hello".to_string()).unwrap(),
            ChatMessage::new(session_id, MessageRole::Assistant, "Hi!".to_string()).unwrap(),
        ];

        let mock_repo = Arc::new(MockChatRepository {
            messages: Mutex::new(messages.clone()),
        });
        let use_case = GetSessionHistoryUseCase::new(mock_repo);

        let request = GetSessionHistoryRequest {
            session_id,
            limit: None,
        };

        let response = use_case.execute(request).await.unwrap();

        assert_eq!(response.messages.len(), 2);
        assert_eq!(response.messages[0].content, "Hello");
        assert_eq!(response.messages[1].content, "Hi!");
    }

    #[tokio::test]
    async fn test_get_session_history_with_limit() {
        let session_id = Uuid::new_v4();
        let messages = vec![
            ChatMessage::new(session_id, MessageRole::User, "Message 1".to_string()).unwrap(),
            ChatMessage::new(session_id, MessageRole::Assistant, "Response 1".to_string()).unwrap(),
            ChatMessage::new(session_id, MessageRole::User, "Message 2".to_string()).unwrap(),
        ];

        let mock_repo = Arc::new(MockChatRepository {
            messages: Mutex::new(messages),
        });
        let use_case = GetSessionHistoryUseCase::new(mock_repo);

        let request = GetSessionHistoryRequest {
            session_id,
            limit: Some(2),
        };

        let response = use_case.execute(request).await.unwrap();

        assert_eq!(response.messages.len(), 2);
    }
}
