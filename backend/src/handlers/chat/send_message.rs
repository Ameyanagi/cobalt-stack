//! Send message endpoint handler with SSE streaming

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures::Stream;
use std::{convert::Infallible, sync::Arc};
use uuid::Uuid;

use crate::{
    application::chat::send_message::{SendMessageRequest as UseCaseRequest, SendMessageUseCase},
    domain::chat::repository::RepositoryError,
    handlers::chat::{dto::SendMessageRequest, ChatState},
    middleware::auth::AuthUser,
};

/// Send a message in a chat session and stream LLM response
///
/// Returns Server-Sent Events (SSE) stream with message chunks
///
/// # Errors
/// Returns HTTP error if:
/// - Session not found (404)
/// - User not authorized (403)
/// - Message validation fails (400)
/// - Database error (500)
#[utoipa::path(
    post,
    path = "/api/chat/sessions/{id}/messages",
    tag = "chat",
    request_body = SendMessageRequest,
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "SSE stream of message chunks", content_type = "text/event-stream"),
        (status = 400, description = "Invalid message content"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - user does not own this session"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn send_message(
    State(state): State<ChatState>,
    Path(session_id): Path<Uuid>,
    auth_user: AuthUser,
    Json(request): Json<SendMessageRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let use_case = SendMessageUseCase::new(
        Arc::clone(&state.repository) as Arc<_>,
        state.llm_config.clone(),
    );

    let use_case_request = UseCaseRequest {
        session_id,
        user_id: auth_user.user_id,
        content: request.content,
    };

    // Execute use case to get streaming response
    let stream = use_case.execute(use_case_request).await.map_err(|e| match e {
        RepositoryError::SessionNotFound(_) => {
            (StatusCode::NOT_FOUND, "Session not found".to_string())
        }
        RepositoryError::ValidationError(msg) if msg.contains("not authorized") => {
            (StatusCode::FORBIDDEN, msg)
        }
        RepositoryError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    })?;

    // Convert to SSE stream
    let sse_stream = convert_to_sse_stream(stream);

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}

/// Convert application stream to SSE event stream
fn convert_to_sse_stream(
    stream: std::pin::Pin<
        Box<dyn Stream<Item = Result<crate::application::chat::send_message::StreamChunk, String>> + Send>,
    >,
) -> impl Stream<Item = Result<Event, Infallible>> {
    use futures::StreamExt;

    stream.map(|result| {
        match result {
            Ok(chunk) => {
                if chunk.is_final {
                    // Send final event to indicate completion
                    Ok(Event::default().event("done").data(""))
                } else {
                    // Send chunk content
                    Ok(Event::default().data(chunk.content))
                }
            }
            Err(e) => {
                // Send error event
                Ok(Event::default().event("error").data(e))
            }
        }
    })
}
