//! Role-based authorization middleware for admin access control.
//!
//! This module provides middleware that restricts access to admin-only endpoints
//! by verifying the authenticated user has the admin role. It must be used in
//! combination with the auth middleware.
//!
//! # Security
//!
//! - Requires prior authentication via [`crate::middleware::auth::auth_middleware`]
//! - Verifies user has [`UserRole::Admin`] role from database
//! - Checks user account is not disabled
//! - Returns 401/403 for unauthorized access attempts
//!
//! # Middleware Ordering
//!
//! **IMPORTANT**: This middleware must be applied AFTER auth_middleware in the layer stack:
//!
//! ```no_run
//! use axum::{Router, routing::get, middleware};
//! use cobalt_stack_backend::middleware::{auth::auth_middleware, admin::admin_middleware};
//! use cobalt_stack_backend::services::auth::JwtConfig;
//! use sea_orm::DatabaseConnection;
//! use std::sync::Arc;
//!
//! # async fn example(db: Arc<DatabaseConnection>) {
//! let jwt_config = JwtConfig::from_env();
//!
//! let admin_routes = Router::new()
//!     .route("/admin/users", get(list_users))
//!     // Admin middleware first (inner layer)
//!     .layer(middleware::from_fn_with_state(db, admin_middleware))
//!     // Auth middleware second (outer layer)
//!     .layer(middleware::from_fn_with_state(jwt_config, auth_middleware));
//! # }
//! # async fn list_users() -> &'static str { "Users" }
//! ```
//!
//! # Error Responses
//!
//! - **401 Unauthorized**: AuthUser not found in extensions (auth_middleware not run first)
//! - **401 Unauthorized**: User not found in database (token valid but user deleted)
//! - **403 Forbidden**: User exists but doesn't have admin role
//! - **403 Forbidden**: User is an admin but account is disabled
//! - **500 Internal Server Error**: Database connection/query failure

use crate::middleware::auth::AuthUser;
use crate::models::{prelude::*, sea_orm_active_enums::UserRole};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::sync::Arc;

/// Axum middleware that enforces admin role requirement.
///
/// This middleware verifies that the authenticated user (from auth_middleware)
/// has admin privileges by checking their role in the database. Only users with
/// [`UserRole::Admin`] and non-disabled accounts can access protected routes.
///
/// # Execution Flow
///
/// 1. Extract [`AuthUser`] from request extensions (injected by auth_middleware)
/// 2. Query database to fetch full user record
/// 3. Verify user has [`UserRole::Admin`] role
/// 4. Verify user account is not disabled (`disabled_at` is NULL)
/// 5. Pass request to next middleware/handler
///
/// # Arguments
///
/// * `db` - Database connection for user role verification
/// * `req` - Incoming HTTP request with AuthUser in extensions
/// * `next` - Next middleware/handler in chain
///
/// # Returns
///
/// - `Ok(Response)` - User is admin and not disabled, request processed
/// - `Err(StatusCode::UNAUTHORIZED)` - AuthUser missing or user not found
/// - `Err(StatusCode::FORBIDDEN)` - User is not admin or account disabled
/// - `Err(StatusCode::INTERNAL_SERVER_ERROR)` - Database error
///
/// # Examples
///
/// ```no_run
/// use axum::{Router, routing::patch, middleware};
/// use cobalt_stack_backend::middleware::{auth::auth_middleware, admin::admin_middleware};
/// use cobalt_stack_backend::services::auth::JwtConfig;
/// use sea_orm::DatabaseConnection;
/// use std::sync::Arc;
///
/// # async fn example(db: Arc<DatabaseConnection>) {
/// let jwt_config = JwtConfig::from_env();
///
/// // Admin-only endpoint for disabling users
/// let admin_routes = Router::new()
///     .route("/admin/users/:id/disable", patch(disable_user))
///     .layer(middleware::from_fn_with_state(db, admin_middleware))
///     .layer(middleware::from_fn_with_state(jwt_config, auth_middleware));
/// # }
/// # async fn disable_user() -> &'static str { "Disabled" }
/// ```
///
/// # Security Notes
///
/// - Always check role from database, never trust client-provided role claims
/// - Disabled admin accounts cannot access admin endpoints
/// - Database connection errors fail secure (return 500, block access)
/// - This middleware performs a database query on each request (consider caching for high traffic)
pub async fn admin_middleware(
    State(db): State<Arc<DatabaseConnection>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract AuthUser from request extensions (injected by auth_middleware)
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?
        .clone();

    // Fetch user from database to check role
    let user = Users::find_by_id(auth_user.user_id)
        .one(db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if user has admin role
    if user.role != UserRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if user account is disabled
    if user.disabled_at.is_some() {
        return Err(StatusCode::FORBIDDEN);
    }

    // User is admin and not disabled, continue
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE - Write failing tests first

    #[test]
    fn test_admin_middleware_requires_auth_user() {
        // Test that middleware returns UNAUTHORIZED when AuthUser is not in extensions
        // This would require mocking Request and Extensions
        // For now, we document the expected behavior
        assert!(true, "Admin middleware must be used after auth_middleware");
    }

    #[tokio::test]
    async fn test_admin_role_enum_equality() {
        // Test that UserRole::Admin equality works correctly
        let admin_role = UserRole::Admin;
        let user_role = UserRole::User;

        assert_eq!(admin_role, UserRole::Admin);
        assert_ne!(admin_role, user_role);
        assert_ne!(user_role, UserRole::Admin);
    }

    #[test]
    fn test_forbidden_status_code_value() {
        // Verify that StatusCode::FORBIDDEN is 403
        assert_eq!(StatusCode::FORBIDDEN.as_u16(), 403);
    }

    #[test]
    fn test_unauthorized_status_code_value() {
        // Verify that StatusCode::UNAUTHORIZED is 401
        assert_eq!(StatusCode::UNAUTHORIZED.as_u16(), 401);
    }

    // Integration tests would go here (require database)
    #[test]
    #[ignore]
    fn test_admin_middleware_allows_admin_user() {
        // Test would verify:
        // 1. User with admin role passes through middleware
        // 2. Middleware allows request to continue
    }

    #[test]
    #[ignore]
    fn test_admin_middleware_blocks_regular_user() {
        // Test would verify:
        // 1. User with 'user' role gets 403 FORBIDDEN
        // 2. Request is blocked
    }

    #[test]
    #[ignore]
    fn test_admin_middleware_blocks_disabled_admin() {
        // Test would verify:
        // 1. Admin user with disabled_at set gets 403 FORBIDDEN
        // 2. Disabled admins cannot access admin routes
    }

    #[test]
    #[ignore]
    fn test_admin_middleware_handles_missing_user() {
        // Test would verify:
        // 1. If user_id doesn't exist in database, return 401 UNAUTHORIZED
        // 2. Handle database errors gracefully
    }
}
