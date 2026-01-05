//! Convenient re-exports of all entity types.
//!
//! This module provides a prelude with commonly used entity types
//! for easy importing throughout the application.
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack::models::prelude::*;
//! use sea_orm::*;
//!
//! # async fn example(db: &DatabaseConnection) -> Result<(), DbErr> {
//! // All entities are available without full paths
//! let user = Users::find().one(db).await?;
//! let tokens = RefreshTokens::find().all(db).await?;
//! # Ok(())
//! # }
//! ```

pub use super::chat_messages::Entity as ChatMessages;
pub use super::chat_sessions::Entity as ChatSessions;
pub use super::refresh_tokens::Entity as RefreshTokens;
pub use super::users::Entity as Users;
