//! Chat repository trait
//!
//! Domain interface for chat data persistence.
//! Infrastructure layer will implement these traits.

use async_trait::async_trait;
use uuid::Uuid;

use super::entity::{ChatMessage, ChatSession};

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Repository errors
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),

    /// Message not found
    #[error("Message not found: {0}")]
    MessageNotFound(Uuid),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Chat repository trait for session and message persistence
#[async_trait]
pub trait ChatRepository: Send + Sync {
    /// Create a new chat session
    async fn create_session(&self, session: &ChatSession) -> RepositoryResult<()>;

    /// Find session by ID
    async fn find_session_by_id(&self, id: Uuid) -> RepositoryResult<Option<ChatSession>>;

    /// Find all sessions for a user (excluding deleted)
    async fn find_sessions_by_user(
        &self,
        user_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> RepositoryResult<(Vec<ChatSession>, u64)>;

    /// Update session
    async fn update_session(&self, session: &ChatSession) -> RepositoryResult<()>;

    /// Soft delete session
    async fn delete_session(&self, id: Uuid) -> RepositoryResult<()>;

    /// Save a message
    async fn save_message(&self, message: &ChatMessage) -> RepositoryResult<()>;

    /// Find messages for a session
    async fn find_messages_by_session(
        &self,
        session_id: Uuid,
        limit: Option<u64>,
    ) -> RepositoryResult<Vec<ChatMessage>>;

    /// Find recent messages for context building
    async fn find_recent_messages(
        &self,
        session_id: Uuid,
        limit: u64,
    ) -> RepositoryResult<Vec<ChatMessage>>;
}
