//! List user sessions endpoint handler

use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    application::chat::list_user_sessions::{
        ListUserSessionsRequest, ListUserSessionsUseCase,
    },
    handlers::chat::{dto::{ListSessionsResponse, SessionDto}, ChatState},
    middleware::auth::AuthUser,
};

/// Query parameters for list sessions endpoint
#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    /// Page number (0-indexed)
    #[serde(default)]
    pub page: u64,
    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_per_page() -> u64 {
    20
}

/// List user's chat sessions with pagination
///
/// # Errors
/// Returns HTTP error if:
/// - Database error occurs (500)
#[utoipa::path(
    get,
    path = "/api/chat/sessions",
    tag = "chat",
    params(
        ("page" = u64, Query, description = "Page number (0-indexed)"),
        ("per_page" = u64, Query, description = "Items per page (default: 20)")
    ),
    responses(
        (status = 200, description = "Sessions retrieved", body = ListSessionsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_user_sessions(
    State(state): State<ChatState>,
    Query(query): Query<ListSessionsQuery>,
    auth_user: AuthUser,
) -> Result<Json<ListSessionsResponse>, (StatusCode, String)> {
    let use_case = ListUserSessionsUseCase::new(Arc::clone(&state.repository) as Arc<_>);

    let request = ListUserSessionsRequest {
        user_id: auth_user.user_id,
        page: query.page,
        per_page: query.per_page,
    };

    let response = use_case
        .execute(request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let sessions = response
        .sessions
        .into_iter()
        .map(SessionDto::from)
        .collect();

    Ok(Json(ListSessionsResponse {
        sessions,
        total: response.total,
        page: response.page,
        per_page: response.per_page,
    }))
}
