// Admin handlers for user management

use crate::models::{prelude::*, sea_orm_active_enums::UserRole, users};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Application state for admin handlers
#[derive(Clone)]
pub struct AdminState {
    pub db: Arc<DatabaseConnection>,
}

// ============================================================================
// DTOs (Data Transfer Objects)
// ============================================================================

/// Query parameters for listing users
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListUsersQuery {
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: u64,

    /// Number of items per page
    #[serde(default = "default_per_page")]
    pub per_page: u64,

    /// Filter by role
    pub role: Option<String>,

    /// Filter by email verification status
    pub email_verified: Option<bool>,

    /// Search by username or email
    pub search: Option<String>,
}

fn default_page() -> u64 {
    1
}
fn default_per_page() -> u64 {
    20
}

/// User response for admin view (includes all fields)
#[derive(Debug, Serialize, ToSchema)]
pub struct AdminUserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub email_verified: bool,
    pub disabled_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub last_login_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

/// Paginated list response
#[derive(Debug, Serialize, ToSchema)]
pub struct UserListResponse {
    pub users: Vec<AdminUserResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// Admin statistics
#[derive(Debug, Serialize, ToSchema)]
pub struct AdminStatsResponse {
    pub total_users: u64,
    pub verified_users: u64,
    pub admin_users: u64,
    pub disabled_users: u64,
}

/// Generic message response
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// List all users with pagination and filtering
#[utoipa::path(
    get,
    path = "/api/admin/users",
    params(ListUsersQuery),
    responses(
        (status = 200, description = "List of users", body = UserListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin"
)]
pub async fn list_users(
    State(state): State<AdminState>,
    Query(query): Query<ListUsersQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let page = query.page.max(1);
    let per_page = query.per_page.clamp(1, 100);

    // Build query with filters
    let mut select = Users::find();

    // Filter by role
    if let Some(role_str) = query.role {
        let role = match role_str.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            "user" => UserRole::User,
            _ => return Err(StatusCode::BAD_REQUEST),
        };
        select = select.filter(users::Column::Role.eq(role));
    }

    // Filter by email verification
    if let Some(verified) = query.email_verified {
        select = select.filter(users::Column::EmailVerified.eq(verified));
    }

    // Search by username or email
    if let Some(search) = query.search {
        let search_pattern = format!("%{}%", search);
        select = select.filter(
            users::Column::Username
                .like(&search_pattern)
                .or(users::Column::Email.like(&search_pattern)),
        );
    }

    // Order by created_at descending
    select = select.order_by_desc(users::Column::CreatedAt);

    // Get total count
    let total = select
        .clone()
        .count(state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Paginate
    let paginator = select.paginate(state.db.as_ref(), per_page);
    let users = paginator
        .fetch_page(page - 1)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to response
    let users: Vec<AdminUserResponse> = users
        .into_iter()
        .map(|u| AdminUserResponse {
            id: u.id,
            username: u.username,
            email: u.email,
            role: u.role,
            email_verified: u.email_verified,
            disabled_at: u.disabled_at,
            last_login_at: u.last_login_at,
            created_at: u.created_at,
            updated_at: u.updated_at,
        })
        .collect();

    let total_pages = (total + per_page - 1) / per_page;

    Ok(Json(UserListResponse {
        users,
        total,
        page,
        per_page,
        total_pages,
    }))
}

/// Get user details by ID
#[utoipa::path(
    get,
    path = "/api/admin/users/{id}",
    params(
        ("id" = String, Path, description = "User ID (UUID format)")
    ),
    responses(
        (status = 200, description = "User details", body = AdminUserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "User not found"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin"
)]
pub async fn get_user(
    State(state): State<AdminState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = Users::find_by_id(user_id)
        .one(state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(AdminUserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        email_verified: user.email_verified,
        disabled_at: user.disabled_at,
        last_login_at: user.last_login_at,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }))
}

/// Disable a user account (soft delete)
#[utoipa::path(
    patch,
    path = "/api/admin/users/{id}/disable",
    params(
        ("id" = String, Path, description = "User ID (UUID format)")
    ),
    responses(
        (status = 200, description = "User disabled", body = MessageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "User not found"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin"
)]
pub async fn disable_user(
    State(state): State<AdminState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = Users::find_by_id(user_id)
        .one(state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if already disabled
    if user.disabled_at.is_some() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Update user
    let mut active_user: users::ActiveModel = user.into();
    active_user.disabled_at = Set(Some(chrono::Utc::now().into()));
    active_user
        .update(state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MessageResponse {
        message: "User disabled successfully".to_string(),
    }))
}

/// Enable a user account (restore from soft delete)
#[utoipa::path(
    patch,
    path = "/api/admin/users/{id}/enable",
    params(
        ("id" = String, Path, description = "User ID (UUID format)")
    ),
    responses(
        (status = 200, description = "User enabled", body = MessageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "User not found"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin"
)]
pub async fn enable_user(
    State(state): State<AdminState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = Users::find_by_id(user_id)
        .one(state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if already enabled
    if user.disabled_at.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Update user
    let mut active_user: users::ActiveModel = user.into();
    active_user.disabled_at = Set(None);
    active_user
        .update(state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MessageResponse {
        message: "User enabled successfully".to_string(),
    }))
}

/// Get admin statistics
#[utoipa::path(
    get,
    path = "/api/admin/stats",
    responses(
        (status = 200, description = "Admin statistics", body = AdminStatsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin"
)]
pub async fn get_stats(State(state): State<AdminState>) -> Result<impl IntoResponse, StatusCode> {
    // Total users
    let total_users = Users::find()
        .count(state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verified users
    let verified_users = Users::find()
        .filter(users::Column::EmailVerified.eq(true))
        .count(state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Admin users
    let admin_users = Users::find()
        .filter(users::Column::Role.eq(UserRole::Admin))
        .count(state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Disabled users
    let disabled_users = Users::find()
        .filter(users::Column::DisabledAt.is_not_null())
        .count(state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AdminStatsResponse {
        total_users,
        verified_users,
        admin_users,
        disabled_users,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pagination_values() {
        assert_eq!(default_page(), 1);
        assert_eq!(default_per_page(), 20);
    }

    #[test]
    fn test_per_page_clamping() {
        // Test that per_page will be clamped between 1 and 100
        let query = ListUsersQuery {
            page: 1,
            per_page: 0,
            role: None,
            email_verified: None,
            search: None,
        };

        let clamped = query.per_page.clamp(1, 100);
        assert_eq!(clamped, 1);

        let query = ListUsersQuery {
            page: 1,
            per_page: 200,
            role: None,
            email_verified: None,
            search: None,
        };

        let clamped = query.per_page.clamp(1, 100);
        assert_eq!(clamped, 100);
    }

    #[test]
    fn test_page_minimum_value() {
        let query = ListUsersQuery {
            page: 0,
            per_page: 20,
            role: None,
            email_verified: None,
            search: None,
        };

        let page = query.page.max(1);
        assert_eq!(page, 1);
    }

    #[test]
    fn test_total_pages_calculation() {
        // 0 users, 20 per page = 0 pages
        let total: u64 = 0;
        let per_page: u64 = 20;
        let total_pages = (total + per_page - 1) / per_page;
        assert_eq!(total_pages, 0);

        // 20 users, 20 per page = 1 page
        let total: u64 = 20;
        let total_pages = (total + per_page - 1) / per_page;
        assert_eq!(total_pages, 1);

        // 21 users, 20 per page = 2 pages
        let total: u64 = 21;
        let total_pages = (total + per_page - 1) / per_page;
        assert_eq!(total_pages, 2);

        // 40 users, 20 per page = 2 pages
        let total: u64 = 40;
        let total_pages = (total + per_page - 1) / per_page;
        assert_eq!(total_pages, 2);
    }

    // Integration tests (require database)
    #[test]
    #[ignore]
    fn test_list_users_pagination() {
        // Test would verify:
        // 1. Correct number of users returned per page
        // 2. Pagination metadata is accurate
        // 3. Results are ordered by created_at DESC
    }

    #[test]
    #[ignore]
    fn test_list_users_role_filter() {
        // Test would verify:
        // 1. Filter by role='admin' returns only admin users
        // 2. Filter by role='user' returns only regular users
    }

    #[test]
    #[ignore]
    fn test_list_users_search() {
        // Test would verify:
        // 1. Search matches username
        // 2. Search matches email
        // 3. Search is case-insensitive (SQL LIKE)
    }

    #[test]
    #[ignore]
    fn test_disable_user_sets_timestamp() {
        // Test would verify:
        // 1. disabled_at is set to current timestamp
        // 2. User cannot login after being disabled
    }

    #[test]
    #[ignore]
    fn test_enable_user_clears_timestamp() {
        // Test would verify:
        // 1. disabled_at is set to NULL
        // 2. User can login again after being enabled
    }

    #[test]
    #[ignore]
    fn test_get_stats_counts() {
        // Test would verify:
        // 1. All counts are accurate
        // 2. Stats update when users are created/modified
    }
}
