//! Create chat session use case

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::chat::{
    entity::ChatSession,
    repository::{ChatRepository, RepositoryResult},
};

/// Request to create a new chat session
#[derive(Debug, Clone)]
pub struct CreateSessionRequest {
    pub user_id: Uuid,
    pub title: String,
}

/// Response containing created session details
#[derive(Debug, Clone)]
pub struct CreateSessionResponse {
    pub session_id: Uuid,
    pub title: String,
}

/// Use case for creating a new chat session
pub struct CreateSessionUseCase {
    repository: Arc<dyn ChatRepository>,
}

impl CreateSessionUseCase {
    /// Create a new use case instance
    #[must_use]
    pub fn new(repository: Arc<dyn ChatRepository>) -> Self {
        Self { repository }
    }

    /// Execute the use case to create a new session
    ///
    /// # Errors
    /// Returns `RepositoryError` if session creation fails
    pub async fn execute(&self, request: CreateSessionRequest) -> RepositoryResult<CreateSessionResponse> {
        // Create domain entity with validation
        let session = ChatSession::new(request.user_id, request.title)
            .map_err(|e| crate::domain::chat::repository::RepositoryError::ValidationError(e))?;

        // Persist to repository
        self.repository.create_session(&session).await?;

        Ok(CreateSessionResponse {
            session_id: session.id,
            title: session.title,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::chat::{entity::ChatSession, repository::RepositoryError};
    use async_trait::async_trait;
    use std::sync::Mutex;

    // Mock repository for testing
    struct MockChatRepository {
        sessions: Mutex<Vec<ChatSession>>,
    }

    #[async_trait]
    impl ChatRepository for MockChatRepository {
        async fn create_session(&self, session: &ChatSession) -> RepositoryResult<()> {
            self.sessions.lock().unwrap().push(session.clone());
            Ok(())
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

        async fn save_message(&self, _message: &crate::domain::chat::entity::ChatMessage) -> RepositoryResult<()> {
            unimplemented!()
        }

        async fn find_messages_by_session(
            &self,
            _session_id: Uuid,
            _limit: Option<u64>,
        ) -> RepositoryResult<Vec<crate::domain::chat::entity::ChatMessage>> {
            unimplemented!()
        }

        async fn find_recent_messages(
            &self,
            _session_id: Uuid,
            _limit: u64,
        ) -> RepositoryResult<Vec<crate::domain::chat::entity::ChatMessage>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_create_session_success() {
        let mock_repo = Arc::new(MockChatRepository {
            sessions: Mutex::new(Vec::new()),
        });
        let use_case = CreateSessionUseCase::new(mock_repo.clone());

        let request = CreateSessionRequest {
            user_id: Uuid::new_v4(),
            title: "Test Session".to_string(),
        };

        let response = use_case.execute(request.clone()).await.unwrap();

        assert_eq!(response.title, "Test Session");
        assert_eq!(mock_repo.sessions.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_create_session_empty_title() {
        let mock_repo = Arc::new(MockChatRepository {
            sessions: Mutex::new(Vec::new()),
        });
        let use_case = CreateSessionUseCase::new(mock_repo.clone());

        let request = CreateSessionRequest {
            user_id: Uuid::new_v4(),
            title: "".to_string(),
        };

        let result = use_case.execute(request).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RepositoryError::ValidationError(_)));
    }

    #[tokio::test]
    async fn test_create_session_title_too_long() {
        let mock_repo = Arc::new(MockChatRepository {
            sessions: Mutex::new(Vec::new()),
        });
        let use_case = CreateSessionUseCase::new(mock_repo.clone());

        let request = CreateSessionRequest {
            user_id: Uuid::new_v4(),
            title: "a".repeat(256),
        };

        let result = use_case.execute(request).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RepositoryError::ValidationError(_)));
    }
}
