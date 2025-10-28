//! Chat HTTP handlers
//!
//! REST API endpoints for chat session and message management.

mod create_session;
mod send_message;
mod send_message_v2; // New provider-based handler
mod get_history;
mod list_sessions;
mod delete_session;

pub mod dto;

pub use create_session::{create_session, __path_create_session};
pub use send_message::{send_message, __path_send_message};
pub use send_message_v2::{send_message_v2, __path_send_message_v2};
pub use get_history::{get_session_history, __path_get_session_history};
pub use list_sessions::{list_user_sessions, __path_list_user_sessions};
pub use delete_session::{delete_session, __path_delete_session};

use axum::{routing::{get, post, delete}, Router};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::infrastructure::persistence::SeaOrmChatRepository;
use crate::application::chat::send_message::LlmConfig;

/// Chat API state
#[derive(Clone)]
pub struct ChatState {
    pub repository: Arc<SeaOrmChatRepository>,
    pub llm_config: LlmConfig,
}

impl ChatState {
    /// Create new chat state
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>, llm_config: LlmConfig) -> Self {
        Self {
            repository: Arc::new(SeaOrmChatRepository::new(db)),
            llm_config,
        }
    }
}

/// Create chat routes
#[must_use]
pub fn routes(state: ChatState) -> Router {
    Router::new()
        .route("/sessions", post(create_session))
        .route("/sessions", get(list_user_sessions))
        .route("/sessions/:id/messages", post(send_message))
        .route("/sessions/:id/messages", get(get_session_history))
        .route("/sessions/:id", delete(delete_session))
        .with_state(state)
}

/// Create v2 chat routes with provider abstraction
#[must_use]
pub fn routes_v2(state: ChatState) -> Router {
    Router::new()
        .route("/sessions", post(create_session))
        .route("/sessions", get(list_user_sessions))
        .route("/sessions/:id/messages", post(send_message_v2)) // Use v2 handler with model selection
        .route("/sessions/:id/messages", get(get_session_history))
        .route("/sessions/:id", delete(delete_session))
        .with_state(state)
}
