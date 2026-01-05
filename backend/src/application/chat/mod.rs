//! Chat application layer
//!
//! Use cases for chat session and message management.

pub mod create_session;
pub mod send_message;
pub mod send_message_v2; // New provider-based implementation
pub mod get_session_history;
pub mod list_user_sessions;
pub mod delete_session;

pub use create_session::CreateSessionUseCase;
pub use send_message::SendMessageUseCase;
pub use send_message_v2::SendMessageUseCase as SendMessageUseCaseV2;
pub use get_session_history::GetSessionHistoryUseCase;
pub use list_user_sessions::ListUserSessionsUseCase;
pub use delete_session::DeleteSessionUseCase;
