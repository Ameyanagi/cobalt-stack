//! Get session history endpoint handler

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::chat::get_session_history::{
        GetSessionHistoryRequest, GetSessionHistoryUseCase,
    },
    domain::chat::repository::ChatRepository,
    handlers::chat::{dto::{GetHistoryResponse, MessageDto}, ChatState},
    middleware::auth::AuthUser,
};

/// Query parameters for history endpoint
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    /// Maximum number of messages to return
    pub limit: Option<u64>,
}

/// Get chat session message history
///
/// # Errors
/// Returns HTTP error if:
/// - Session not found (404)
/// - User not authorized (403)
/// - Database error (500)
#[utoipa::path(
    get,
    path = "/api/chat/sessions/{id}/messages",
    tag = "chat",
    params(
        ("id" = Uuid, Path, description = "Session ID"),
        ("limit" = Option<u64>, Query, description = "Maximum number of messages to return")
    ),
    responses(
        (status = 200, description = "Message history retrieved", body = GetHistoryResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_session_history(
    State(state): State<ChatState>,
    Path(session_id): Path<Uuid>,
    Query(query): Query<HistoryQuery>,
    auth_user: AuthUser,
) -> Result<Json<GetHistoryResponse>, (StatusCode, String)> {
    // First, verify the session exists and belongs to the user
    let session = state
        .repository
        .find_session_by_id(session_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    // Verify ownership
    if session.user_id != auth_user.user_id {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let use_case = GetSessionHistoryUseCase::new(Arc::clone(&state.repository) as Arc<_>);

    let request = GetSessionHistoryRequest {
        session_id,
        limit: query.limit,
    };

    let response = use_case
        .execute(request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let messages = response
        .messages
        .into_iter()
        .map(MessageDto::from)
        .collect();

    Ok(Json(GetHistoryResponse {
        session: session.into(),
        messages,
    }))
}
