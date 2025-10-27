//! Create session endpoint handler

use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

use crate::{
    application::chat::create_session::{CreateSessionUseCase, CreateSessionRequest as UseCaseRequest},
    handlers::chat::{dto::{CreateSessionRequest, CreateSessionResponse}, ChatState},
    middleware::auth::AuthUser,
};

/// Create a new chat session
///
/// # Errors
/// Returns HTTP error if:
/// - Title validation fails (400)
/// - Database error occurs (500)
#[utoipa::path(
    post,
    path = "/api/chat/sessions",
    tag = "chat",
    request_body = CreateSessionRequest,
    responses(
        (status = 201, description = "Session created successfully", body = CreateSessionResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_session(
    State(state): State<ChatState>,
    auth_user: AuthUser,
    Json(request): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<CreateSessionResponse>), (StatusCode, String)> {
    let use_case = CreateSessionUseCase::new(Arc::clone(&state.repository) as Arc<_>);

    let use_case_request = UseCaseRequest {
        user_id: auth_user.user_id,
        title: request.title,
    };

    let response = use_case
        .execute(use_case_request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(CreateSessionResponse {
            session_id: response.session_id,
            title: response.title,
        }),
    ))
}
