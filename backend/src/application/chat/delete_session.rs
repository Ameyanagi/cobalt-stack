//! Delete chat session use case

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::chat::repository::{ChatRepository, RepositoryResult};

/// Request to delete a chat session
#[derive(Debug, Clone)]
pub struct DeleteSessionRequest {
    pub session_id: Uuid,
    pub user_id: Uuid, // For authorization verification
}

/// Response indicating successful deletion
#[derive(Debug, Clone)]
pub struct DeleteSessionResponse {
    pub session_id: Uuid,
}

/// Use case for deleting a chat session (soft delete)
pub struct DeleteSessionUseCase {
    repository: Arc<dyn ChatRepository>,
}

impl DeleteSessionUseCase {
    /// Create a new use case instance
    #[must_use]
    pub fn new(repository: Arc<dyn ChatRepository>) -> Self {
        Self { repository }
    }

    /// Execute the use case to delete a session
    ///
    /// # Errors
    /// Returns `RepositoryError` if:
    /// - Session not found
    /// - User not authorized (session belongs to different user)
    /// - Deletion fails
    pub async fn execute(&self, request: DeleteSessionRequest) -> RepositoryResult<DeleteSessionResponse> {
        // Verify session exists and belongs to user
        let session = self
            .repository
            .find_session_by_id(request.session_id)
            .await?
            .ok_or(crate::domain::chat::repository::RepositoryError::SessionNotFound(
                request.session_id,
            ))?;

        // Authorization check
        if session.user_id != request.user_id {
            return Err(crate::domain::chat::repository::RepositoryError::ValidationError(
                "User not authorized to delete this session".to_string(),
            ));
        }

        // Perform soft delete
        self.repository.delete_session(request.session_id).await?;

        Ok(DeleteSessionResponse {
            session_id: request.session_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::chat::{entity::{ChatSession, ChatMessage}, repository::RepositoryError};
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Mutex;

    struct MockChatRepository {
        sessions: Mutex<Vec<ChatSession>>,
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

        async fn delete_session(&self, id: Uuid) -> RepositoryResult<()> {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(session) = sessions.iter_mut().find(|s| s.id == id) {
                session.mark_deleted();
                Ok(())
            } else {
                Err(RepositoryError::SessionNotFound(id))
            }
        }

        async fn save_message(&self, _message: &ChatMessage) -> RepositoryResult<()> {
            unimplemented!()
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
            _session_id: Uuid,
            _limit: u64,
        ) -> RepositoryResult<Vec<ChatMessage>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_delete_session_success() {
        let user_id = Uuid::new_v4();
        let session = ChatSession::new(user_id, "Test Session".to_string()).unwrap();
        let session_id = session.id;

        let mock_repo = Arc::new(MockChatRepository {
            sessions: Mutex::new(vec![session]),
        });
        let use_case = DeleteSessionUseCase::new(mock_repo.clone());

        let request = DeleteSessionRequest { session_id, user_id };

        let response = use_case.execute(request).await.unwrap();

        assert_eq!(response.session_id, session_id);

        // Verify soft delete was performed
        let deleted_session = mock_repo.sessions.lock().unwrap()[0].clone();
        assert!(deleted_session.deleted_at.is_some());
    }

    #[tokio::test]
    async fn test_delete_session_not_found() {
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        let mock_repo = Arc::new(MockChatRepository {
            sessions: Mutex::new(Vec::new()),
        });
        let use_case = DeleteSessionUseCase::new(mock_repo);

        let request = DeleteSessionRequest { session_id, user_id };

        let result = use_case.execute(request).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RepositoryError::SessionNotFound(_)));
    }

    #[tokio::test]
    async fn test_delete_session_unauthorized() {
        let owner_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let session = ChatSession::new(owner_id, "Test Session".to_string()).unwrap();
        let session_id = session.id;

        let mock_repo = Arc::new(MockChatRepository {
            sessions: Mutex::new(vec![session]),
        });
        let use_case = DeleteSessionUseCase::new(mock_repo);

        let request = DeleteSessionRequest {
            session_id,
            user_id: other_user_id,
        };

        let result = use_case.execute(request).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RepositoryError::ValidationError(_)));
    }
}
