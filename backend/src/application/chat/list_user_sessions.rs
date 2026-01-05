//! List user chat sessions use case

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::chat::{
    entity::ChatSession,
    repository::{ChatRepository, RepositoryResult},
};

/// Request to list user's chat sessions
#[derive(Debug, Clone)]
pub struct ListUserSessionsRequest {
    pub user_id: Uuid,
    pub page: u64,
    pub per_page: u64,
}

/// Response containing paginated sessions
#[derive(Debug, Clone)]
pub struct ListUserSessionsResponse {
    pub sessions: Vec<ChatSession>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

/// Use case for listing user's chat sessions
pub struct ListUserSessionsUseCase {
    repository: Arc<dyn ChatRepository>,
}

impl ListUserSessionsUseCase {
    /// Create a new use case instance
    #[must_use]
    pub fn new(repository: Arc<dyn ChatRepository>) -> Self {
        Self { repository }
    }

    /// Execute the use case to list user sessions
    ///
    /// # Errors
    /// Returns `RepositoryError` if retrieval fails
    pub async fn execute(&self, request: ListUserSessionsRequest) -> RepositoryResult<ListUserSessionsResponse> {
        let (sessions, total) = self
            .repository
            .find_sessions_by_user(request.user_id, request.page, request.per_page)
            .await?;

        Ok(ListUserSessionsResponse {
            sessions,
            total,
            page: request.page,
            per_page: request.per_page,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::chat::{entity::ChatMessage, repository::RepositoryError};
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockChatRepository {
        sessions: Mutex<Vec<ChatSession>>,
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
            user_id: Uuid,
            page: u64,
            per_page: u64,
        ) -> RepositoryResult<(Vec<ChatSession>, u64)> {
            let sessions = self.sessions.lock().unwrap();
            let user_sessions: Vec<_> = sessions
                .iter()
                .filter(|s| s.user_id == user_id && s.deleted_at.is_none())
                .cloned()
                .collect();

            let total = user_sessions.len() as u64;
            let start = (page * per_page) as usize;
            let end = std::cmp::min(start + per_page as usize, user_sessions.len());
            let page_sessions = user_sessions[start..end].to_vec();

            Ok((page_sessions, total))
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
    async fn test_list_user_sessions() {
        let user_id = Uuid::new_v4();
        let sessions = vec![
            ChatSession::new(user_id, "Session 1".to_string()).unwrap(),
            ChatSession::new(user_id, "Session 2".to_string()).unwrap(),
            ChatSession::new(user_id, "Session 3".to_string()).unwrap(),
        ];

        let mock_repo = Arc::new(MockChatRepository {
            sessions: Mutex::new(sessions),
        });
        let use_case = ListUserSessionsUseCase::new(mock_repo);

        let request = ListUserSessionsRequest {
            user_id,
            page: 0,
            per_page: 10,
        };

        let response = use_case.execute(request).await.unwrap();

        assert_eq!(response.sessions.len(), 3);
        assert_eq!(response.total, 3);
        assert_eq!(response.page, 0);
        assert_eq!(response.per_page, 10);
    }

    #[tokio::test]
    async fn test_list_user_sessions_pagination() {
        let user_id = Uuid::new_v4();
        let sessions = vec![
            ChatSession::new(user_id, "Session 1".to_string()).unwrap(),
            ChatSession::new(user_id, "Session 2".to_string()).unwrap(),
            ChatSession::new(user_id, "Session 3".to_string()).unwrap(),
        ];

        let mock_repo = Arc::new(MockChatRepository {
            sessions: Mutex::new(sessions),
        });
        let use_case = ListUserSessionsUseCase::new(mock_repo);

        let request = ListUserSessionsRequest {
            user_id,
            page: 0,
            per_page: 2,
        };

        let response = use_case.execute(request).await.unwrap();

        assert_eq!(response.sessions.len(), 2);
        assert_eq!(response.total, 3);
    }
}
