//! Chat message entity for LLM conversations.
//!
//! This module defines the `ChatMessage` entity which represents individual
//! messages within a chat session (user messages, assistant responses, system messages).
//!
//! # Database Mapping
//!
//! - **Table**: `chat_messages`
//! - **Primary Key**: `id` (UUID, not auto-increment)
//! - **Foreign Key**: `session_id` → `chat_sessions.id` (CASCADE)
//! - **CHECK Constraint**: `role` IN ('user', 'assistant', 'system')
//!
//! # Relations
//!
//! - `belongs_to` `ChatSessions`: Parent session
//!
//! # Message Roles
//!
//! - `user`: Message from the human user
//! - `assistant`: Response from the AI assistant
//! - `system`: System message for behavior control

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Chat message entity.
///
/// Represents a single message in a conversation thread.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "chat_messages")]
pub struct Model {
    /// Unique message identifier (UUID v4).
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Session this message belongs to.
    /// Foreign key to chat_sessions table.
    pub session_id: Uuid,

    /// Message role: 'user', 'assistant', or 'system'.
    /// Enforced by database CHECK constraint.
    pub role: String,

    /// Message content/text.
    /// Can be multiline and contain markdown.
    #[sea_orm(column_type = "Text")]
    pub content: String,

    /// Token count for this message (optional).
    /// Used for quota tracking and cost management.
    pub token_count: Option<i32>,

    /// Timestamp when the message was created.
    pub created_at: DateTimeWithTimeZone,
}

/// Entity relations for the ChatMessage model.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Message belongs to a session.
    /// Cascades on delete: deleting session removes its messages.
    #[sea_orm(
        belongs_to = "super::chat_sessions::Entity",
        from = "Column::SessionId",
        to = "super::chat_sessions::Column::Id",
        on_delete = "Cascade"
    )]
    ChatSessions,
}

impl Related<super::chat_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChatSessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
