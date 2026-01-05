//! Send message endpoint handler with provider abstraction and model selection

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
    application::chat::{SendMessageUseCaseV2, send_message_v2::{
        SendMessageRequest as UseCaseRequest, UseCaseConfig,
    }},
    domain::chat::repository::RepositoryError,
    handlers::chat::{dto::SendMessageRequest, ChatState},
    middleware::auth::AuthUser,
};

/// Send a message in a chat session with model selection and stream LLM response
///
/// Returns Server-Sent Events (SSE) stream with message chunks
///
/// # Errors
/// Returns HTTP error if:
/// - Session not found (404)
/// - User not authorized (403)
/// - Message validation fails (400)
/// - Model not found (400)
/// - Provider error (500)
/// - Database error (500)
#[utoipa::path(
    post,
    path = "/api/v1/chat/sessions/{id}/messages",
    tag = "chat",
    request_body = SendMessageRequest,
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "SSE stream of message chunks", content_type = "text/event-stream"),
        (status = 400, description = "Invalid message content or model"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - user does not own this session"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn send_message_v2(
    State(state): State<ChatState>,
    Path(session_id): Path<Uuid>,
    auth_user: AuthUser,
    Json(request): Json<SendMessageRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Create use case with shared provider factory
    let config = UseCaseConfig {
        max_context_messages: state.llm_config.max_context_messages,
        max_tokens: state.llm_config.max_tokens,
    };

    let use_case = SendMessageUseCaseV2::new(
        Arc::clone(&state.repository) as Arc<_>,
        Arc::clone(&state.provider_factory),
        config,
    );

    let use_case_request = UseCaseRequest {
        session_id,
        user_id: auth_user.user_id,
        content: request.content,
        model_id: request.model_id, // Pass model selection
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
        RepositoryError::DatabaseError(msg) if msg.contains("Model") || msg.contains("Provider") => {
            (StatusCode::BAD_REQUEST, msg)
        }
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    })?;

    // Convert to SSE stream
    let sse_stream = convert_to_sse_stream(stream);

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}

/// Convert application stream to SSE event stream
fn convert_to_sse_stream(
    stream: std::pin::Pin<
        Box<
            dyn Stream<Item = Result<crate::application::chat::send_message_v2::StreamChunk, String>>
                + Send,
        >,
    >,
) -> impl Stream<Item = Result<Event, Infallible>> {
    use futures::StreamExt;

    stream.map(|result| match result {
        Ok(chunk) => {
            if chunk.is_final {
                // Send final event to indicate completion
                Ok(Event::default().data("[DONE]"))
            } else {
                // Send chunk content as JSON
                let json_data = format!(
                    r#"{{"content":"{}"}}"#,
                    chunk.content.replace('"', r#"\""#).replace('\n', r#"\n"#)
                );
                Ok(Event::default().data(json_data))
            }
        }
        Err(e) => {
            // Send error event as JSON
            let error_json = format!(r#"{{"error":"{}"}}"#, e.replace('"', r#"\""#));
            Ok(Event::default().event("error").data(error_json))
        }
    })
}
