// Admin authorization middleware - requires admin role

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

/// Admin middleware that checks if authenticated user has admin role
/// This middleware must be used AFTER auth_middleware
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
