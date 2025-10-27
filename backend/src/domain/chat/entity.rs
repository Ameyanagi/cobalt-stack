//! Chat domain entities
//!
//! Core business entities representing chat sessions and messages.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::value_objects::MessageRole;

/// Chat session entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSession {
    /// Unique session identifier
    pub id: Uuid,
    /// Owner user identifier
    pub user_id: Uuid,
    /// Session title
    pub title: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp
    pub deleted_at: Option<DateTime<Utc>>,
}

impl ChatSession {
    /// Create a new chat session
    ///
    /// # Errors
    ///
    /// Returns error if title is empty or too long
    pub fn new(user_id: Uuid, title: String) -> Result<Self, String> {
        Self::validate_title(&title)?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            title,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    /// Validate title
    fn validate_title(title: &str) -> Result<(), String> {
        if title.is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if title.len() > 255 {
            return Err("Title cannot exceed 255 characters".to_string());
        }
        Ok(())
    }

    /// Check if session is deleted (soft delete)
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Mark session as deleted (soft delete)
    pub fn mark_deleted(&mut self) {
        self.deleted_at = Some(Utc::now());
    }

    /// Update session title
    ///
    /// # Errors
    ///
    /// Returns error if title is invalid
    pub fn update_title(&mut self, title: String) -> Result<(), String> {
        Self::validate_title(&title)?;
        self.title = title;
        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Chat message entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Unique message identifier
    pub id: Uuid,
    /// Session identifier this message belongs to
    pub session_id: Uuid,
    /// Message role (user, assistant, system)
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Token count (optional, for tracking usage)
    pub token_count: Option<i32>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl ChatMessage {
    /// Create a new chat message
    ///
    /// # Errors
    ///
    /// Returns error if content is empty or too long
    pub fn new(
        session_id: Uuid,
        role: MessageRole,
        content: String,
    ) -> Result<Self, String> {
        Self::validate_content(&content)?;

        Ok(Self {
            id: Uuid::new_v4(),
            session_id,
            role,
            content,
            token_count: None,
            created_at: Utc::now(),
        })
    }

    /// Create a new message with token count
    ///
    /// # Errors
    ///
    /// Returns error if content is invalid
    pub fn new_with_tokens(
        session_id: Uuid,
        role: MessageRole,
        content: String,
        token_count: i32,
    ) -> Result<Self, String> {
        let mut message = Self::new(session_id, role, content)?;
        message.token_count = Some(token_count);
        Ok(message)
    }

    /// Validate message content
    fn validate_content(content: &str) -> Result<(), String> {
        if content.is_empty() {
            return Err("Message content cannot be empty".to_string());
        }
        if content.len() > 10_000 {
            return Err("Message content cannot exceed 10,000 characters".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_session_new() {
        let user_id = Uuid::new_v4();
        let session = ChatSession::new(user_id, "Test Session".to_string()).unwrap();

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.title, "Test Session");
        assert!(!session.is_deleted());
        assert!(session.deleted_at.is_none());
    }

    #[test]
    fn test_chat_session_empty_title() {
        let user_id = Uuid::new_v4();
        let result = ChatSession::new(user_id, String::new());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Title cannot be empty");
    }

    #[test]
    fn test_chat_session_title_too_long() {
        let user_id = Uuid::new_v4();
        let long_title = "a".repeat(256);
        let result = ChatSession::new(user_id, long_title);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Title cannot exceed 255 characters"
        );
    }

    #[test]
    fn test_chat_session_mark_deleted() {
        let user_id = Uuid::new_v4();
        let mut session = ChatSession::new(user_id, "Test".to_string()).unwrap();

        assert!(!session.is_deleted());

        session.mark_deleted();

        assert!(session.is_deleted());
        assert!(session.deleted_at.is_some());
    }

    #[test]
    fn test_chat_session_update_title() {
        let user_id = Uuid::new_v4();
        let mut session = ChatSession::new(user_id, "Original".to_string()).unwrap();
        let original_updated_at = session.updated_at;

        // Small delay to ensure updated_at changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        session.update_title("Updated".to_string()).unwrap();

        assert_eq!(session.title, "Updated");
        assert!(session.updated_at > original_updated_at);
    }

    #[test]
    fn test_chat_message_new() {
        let session_id = Uuid::new_v4();
        let message =
            ChatMessage::new(session_id, MessageRole::User, "Hello".to_string()).unwrap();

        assert_eq!(message.session_id, session_id);
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, "Hello");
        assert!(message.token_count.is_none());
    }

    #[test]
    fn test_chat_message_with_tokens() {
        let session_id = Uuid::new_v4();
        let message = ChatMessage::new_with_tokens(
            session_id,
            MessageRole::Assistant,
            "Response".to_string(),
            42,
        )
        .unwrap();

        assert_eq!(message.token_count, Some(42));
    }

    #[test]
    fn test_chat_message_empty_content() {
        let session_id = Uuid::new_v4();
        let result = ChatMessage::new(session_id, MessageRole::User, String::new());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Message content cannot be empty");
    }

    #[test]
    fn test_chat_message_content_too_long() {
        let session_id = Uuid::new_v4();
        let long_content = "a".repeat(10_001);
        let result = ChatMessage::new(session_id, MessageRole::User, long_content);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Message content cannot exceed 10,000 characters"
        );
    }

    #[test]
    fn test_chat_message_all_roles() {
        let session_id = Uuid::new_v4();

        let user_msg =
            ChatMessage::new(session_id, MessageRole::User, "User message".to_string()).unwrap();
        assert_eq!(user_msg.role, MessageRole::User);

        let assistant_msg = ChatMessage::new(
            session_id,
            MessageRole::Assistant,
            "Assistant message".to_string(),
        )
        .unwrap();
        assert_eq!(assistant_msg.role, MessageRole::Assistant);

        let system_msg = ChatMessage::new(
            session_id,
            MessageRole::System,
            "System message".to_string(),
        )
        .unwrap();
        assert_eq!(system_msg.role, MessageRole::System);
    }
}
