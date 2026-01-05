//! Delete session endpoint handler

use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::chat::delete_session::{DeleteSessionRequest, DeleteSessionUseCase},
    domain::chat::repository::RepositoryError,
    handlers::chat::{dto::DeleteSessionResponse, ChatState},
    middleware::auth::AuthUser,
};

/// Delete a chat session (soft delete)
///
/// # Errors
/// Returns HTTP error if:
/// - Session not found (404)
/// - User not authorized (403)
/// - Database error (500)
#[utoipa::path(
    delete,
    path = "/api/chat/sessions/{id}",
    tag = "chat",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Session deleted", body = DeleteSessionResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - user does not own this session"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_session(
    State(state): State<ChatState>,
    Path(session_id): Path<Uuid>,
    auth_user: AuthUser,
) -> Result<Json<DeleteSessionResponse>, (StatusCode, String)> {
    let use_case = DeleteSessionUseCase::new(Arc::clone(&state.repository) as Arc<_>);

    let request = DeleteSessionRequest {
        session_id,
        user_id: auth_user.user_id,
    };

    let response = use_case.execute(request).await.map_err(|e| match e {
        RepositoryError::SessionNotFound(_) => (StatusCode::NOT_FOUND, "Session not found".to_string()),
        RepositoryError::ValidationError(msg) if msg.contains("not authorized") => {
            (StatusCode::FORBIDDEN, msg)
        }
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    })?;

    Ok(Json(DeleteSessionResponse {
        session_id: response.session_id,
        message: "Session deleted successfully".to_string(),
    }))
}
