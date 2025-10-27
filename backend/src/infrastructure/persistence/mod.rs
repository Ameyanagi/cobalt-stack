//! Persistence implementations
//!
//! Contains repository implementations using SeaORM.

pub mod chat_repository;

pub use chat_repository::SeaOrmChatRepository;
