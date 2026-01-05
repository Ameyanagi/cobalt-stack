//! Data Transfer Objects for chat API

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

use crate::domain::chat::entity::{ChatMessage, ChatSession};

/// Request to create a new chat session
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSessionRequest {
    /// Session title
    #[schema(example = "My Chat Session")]
    pub title: String,
}

/// Response containing created session details
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSessionResponse {
    /// Session ID
    pub session_id: Uuid,
    /// Session title
    pub title: String,
}

/// Request to send a message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SendMessageRequest {
    /// Message content
    #[schema(example = "Hello, how are you?")]
    pub content: String,
    /// Optional model ID to use (defaults to configured default)
    #[serde(default)]
    #[schema(example = "llama-3.3-70b")]
    pub model_id: Option<String>,
}

/// Session details
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionDto {
    /// Session ID
    pub id: Uuid,
    /// Session title
    pub title: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl From<ChatSession> for SessionDto {
    fn from(session: ChatSession) -> Self {
        Self {
            id: session.id,
            title: session.title,
            created_at: session.created_at,
            updated_at: session.updated_at,
        }
    }
}

/// Message details
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageDto {
    /// Message ID
    pub id: Uuid,
    /// Message role
    pub role: String,
    /// Message content
    pub content: String,
    /// Token count (if available)
    pub token_count: Option<i32>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl From<ChatMessage> for MessageDto {
    fn from(message: ChatMessage) -> Self {
        Self {
            id: message.id,
            role: message.role.as_str().to_string(),
            content: message.content,
            token_count: message.token_count,
            created_at: message.created_at,
        }
    }
}

/// Response containing message history
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetHistoryResponse {
    /// Session details
    pub session: SessionDto,
    /// Messages in chronological order
    pub messages: Vec<MessageDto>,
}

/// Response containing paginated sessions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListSessionsResponse {
    /// Sessions for the current page
    pub sessions: Vec<SessionDto>,
    /// Total number of sessions
    pub total: u64,
    /// Current page number (0-indexed)
    pub page: u64,
    /// Items per page
    pub per_page: u64,
}

/// Response confirming deletion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeleteSessionResponse {
    /// Deleted session ID
    pub session_id: Uuid,
    /// Confirmation message
    pub message: String,
}
