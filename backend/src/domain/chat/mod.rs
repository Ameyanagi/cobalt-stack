//! Chat domain module
//!
//! Contains entities, value objects, and repository traits for chat functionality.
//! Pure business logic with no infrastructure dependencies.

pub mod entity;
pub mod repository;
pub mod value_objects;

pub use entity::{ChatMessage, ChatSession};
pub use repository::{ChatRepository, RepositoryError, RepositoryResult};
pub use value_objects::MessageRole;
