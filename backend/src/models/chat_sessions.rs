//! Chat session entity for LLM conversations.
//!
//! This module defines the `ChatSession` entity which represents a conversation
//! thread between a user and the AI assistant.
//!
//! # Database Mapping
//!
//! - **Table**: `chat_sessions`
//! - **Primary Key**: `id` (UUID, not auto-increment)
//! - **Foreign Key**: `user_id` → `users.id` (CASCADE)
//!
//! # Relations
//!
//! - `belongs_to` `Users`: Session owner
//! - `has_many` `ChatMessages`: Messages in this session
//!
//! # Soft Delete
//!
//! Sessions use soft delete pattern with `deleted_at` timestamp.
//! Deleted sessions are filtered out in queries but remain in database.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Chat session entity.
///
/// Represents a conversation thread between a user and the AI assistant.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "chat_sessions")]
pub struct Model {
    /// Unique session identifier (UUID v4).
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// User who owns this session.
    /// Foreign key to users table.
    pub user_id: Uuid,

    /// Session title (user-provided or auto-generated).
    /// Maximum 255 characters.
    pub title: String,

    /// Timestamp when the session was created.
    pub created_at: DateTimeWithTimeZone,

    /// Timestamp when the session was last updated.
    pub updated_at: DateTimeWithTimeZone,

    /// Timestamp when the session was soft deleted.
    /// If set, session is considered deleted.
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

/// Entity relations for the ChatSession model.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Session belongs to a user.
    /// Cascades on delete: deleting user removes their sessions.
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_delete = "Cascade"
    )]
    Users,

    /// Session has many messages.
    /// Cascades on delete: deleting session removes its messages.
    #[sea_orm(has_many = "super::chat_messages::Entity")]
    ChatMessages,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::chat_messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChatMessages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
