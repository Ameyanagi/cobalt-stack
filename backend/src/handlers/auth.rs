use crate::services::auth::{AuthError, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================================================================
// DTOs (Data Transfer Objects)
// ============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "alice")]
    pub username: String,

    #[schema(example = "alice@example.com")]
    pub email: String,

    #[schema(example = "SecurePass123!")]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "alice")]
    pub username: String,

    #[schema(example = "SecurePass123!")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    #[schema(value_type = String, example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub role: crate::models::sea_orm_active_enums::UserRole,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

// ============================================================================
// Validation
// ============================================================================

impl RegisterRequest {
    pub fn validate(&self) -> Result<()> {
        // Username validation
        if self.username.is_empty() {
            return Err(AuthError::InvalidInput("Username cannot be empty".to_string()).into());
        }
        if self.username.len() < 3 || self.username.len() > 50 {
            return Err(AuthError::InvalidInput(
                "Username must be between 3 and 50 characters".to_string(),
            )
            .into());
        }

        // Email validation (basic)
        if self.email.is_empty() {
            return Err(AuthError::InvalidInput("Email cannot be empty".to_string()).into());
        }
        if !self.email.contains('@') {
            return Err(AuthError::InvalidInput("Invalid email format".to_string()).into());
        }

        // Password validation
        if self.password.len() < 8 {
            return Err(AuthError::InvalidInput(
                "Password must be at least 8 characters".to_string(),
            )
            .into());
        }
        if self.password.len() > 128 {
            return Err(
                AuthError::InvalidInput("Password must not exceed 128 characters".to_string())
                    .into(),
            );
        }

        Ok(())
    }
}

impl LoginRequest {
    pub fn validate(&self) -> Result<()> {
        if self.username.is_empty() {
            return Err(AuthError::InvalidInput("Username cannot be empty".to_string()).into());
        }
        if self.password.is_empty() {
            return Err(AuthError::InvalidInput("Password cannot be empty".to_string()).into());
        }
        Ok(())
    }
}

// ============================================================================
// Handlers
// ============================================================================

use crate::models::{prelude::*, users};
use crate::services::auth::{
    create_access_token, create_refresh_token, hash_password, verify_password, JwtConfig,
    store_refresh_token,
};
use axum::{extract::State, http::{header, StatusCode}, response::IntoResponse, Json};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub jwt_config: JwtConfig,
}

/// POST /api/auth/register - Register a new user
///
/// Creates a new user account with username/email/password.
/// Returns access token on success.
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 409, description = "User already exists", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> std::result::Result<impl IntoResponse, AuthError> {
    // Validate input
    req.validate().map_err(|e| {
        // The validate() function already returns AuthError wrapped in anyhow::Error
        // Extract the AuthError from the anyhow chain
        e.downcast::<AuthError>().unwrap_or(AuthError::InvalidInput("Validation failed".to_string()))
    })?;

    // Check if username already exists
    let existing_user = Users::find()
        .filter(users::Column::Username.eq(&req.username))
        .one(state.db.as_ref())
        .await?;

    if existing_user.is_some() {
        return Err(AuthError::UserAlreadyExists);
    }

    // Check if email already exists
    let existing_email = Users::find()
        .filter(users::Column::Email.eq(&req.email))
        .one(state.db.as_ref())
        .await?;

    if existing_email.is_some() {
        return Err(AuthError::UserAlreadyExists);
    }

    // Hash password
    let password_hash = hash_password(&req.password).map_err(|_| AuthError::PasswordHashError)?;

    // Create user
    let user = users::ActiveModel {
        username: Set(req.username.clone()),
        email: Set(req.email.clone()),
        password_hash: Set(Some(password_hash)),
        email_verified: Set(false),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };

    let user = user.insert(state.db.as_ref()).await?;

    // Send verification email
    {
        use crate::services::email::{create_verification_token, EmailSender, MockEmailSender};

        // Create verification token
        let token = create_verification_token(state.db.as_ref(), user.id)
            .await
            .map_err(|e| AuthError::DatabaseError(format!("Failed to create token: {}", e)))?;

        // Send verification email
        let email_sender = MockEmailSender;
        email_sender
            .send_verification_email(&user.email, &token)
            .map_err(|_| AuthError::InternalError)?;
    }

    // Generate tokens
    let access_token = create_access_token(user.id, user.username.clone(), &state.jwt_config)
        .map_err(|_| AuthError::JwtEncodingError)?;
    let (refresh_token, refresh_jti) = create_refresh_token(user.id, &state.jwt_config)
        .map_err(|_| AuthError::JwtEncodingError)?;

    // Store refresh token in database
    store_refresh_token(
        state.db.as_ref(),
        user.id,
        &refresh_token,
        refresh_jti,
        state.jwt_config.refresh_token_expiry_days,
    )
    .await
    .map_err(|_| AuthError::DatabaseError("Failed to store refresh token".to_string()))?;

    // Create HttpOnly cookie for refresh token
    let cookie = Cookie::build(("refresh_token", refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::days(state.jwt_config.refresh_token_expiry_days))
        .build();

    // Return response with cookie
    let response = AuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_config.access_token_expiry_minutes * 60,
    };

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(response),
    ))
}

/// POST /api/auth/login - Login with username/password
///
/// Authenticates user and returns access token.
/// Rate limited to 5 attempts per 15 minutes per IP.
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> std::result::Result<impl IntoResponse, AuthError> {
    // Validate input
    req.validate().map_err(|e| {
        // The validate() function already returns AuthError wrapped in anyhow::Error
        // Extract the AuthError from the anyhow chain
        e.downcast::<AuthError>().unwrap_or(AuthError::InvalidInput("Validation failed".to_string()))
    })?;

    // Find user by username
    let user = Users::find()
        .filter(users::Column::Username.eq(&req.username))
        .one(state.db.as_ref())
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    // Verify password
    let password_hash = user.password_hash.ok_or(AuthError::InvalidCredentials)?;
    let is_valid = verify_password(&req.password, &password_hash)
        .map_err(|_| AuthError::InvalidCredentials)?;

    if !is_valid {
        return Err(AuthError::InvalidCredentials);
    }

    // Generate tokens
    let access_token = create_access_token(user.id, user.username.clone(), &state.jwt_config)
        .map_err(|_| AuthError::JwtEncodingError)?;
    let (refresh_token, refresh_jti) = create_refresh_token(user.id, &state.jwt_config)
        .map_err(|_| AuthError::JwtEncodingError)?;

    // Store refresh token in database
    store_refresh_token(
        state.db.as_ref(),
        user.id,
        &refresh_token,
        refresh_jti,
        state.jwt_config.refresh_token_expiry_days,
    )
    .await
    .map_err(|_| AuthError::DatabaseError("Failed to store refresh token".to_string()))?;

    // Create HttpOnly cookie for refresh token
    let cookie = Cookie::build(("refresh_token", refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::days(state.jwt_config.refresh_token_expiry_days))
        .build();

    // Return response with cookie
    let response = AuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_config.access_token_expiry_minutes * 60,
    };

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(response),
    ))
}

/// POST /api/auth/refresh - Refresh access token using refresh token
///
/// Rotates refresh token and returns new access token.
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    responses(
        (status = 200, description = "Token refreshed", body = AuthResponse),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    jar: axum_extra::extract::CookieJar,
) -> std::result::Result<impl IntoResponse, AuthError> {
    use crate::services::auth::{
        verify_refresh_token, rotate_refresh_token, validate_refresh_token,
        create_access_token, create_refresh_token,
    };

    // Extract refresh token from cookie
    let old_refresh_token = jar
        .get("refresh_token")
        .ok_or(AuthError::InvalidToken)?
        .value()
        .to_string();

    // Verify JWT signature and expiry
    let claims = verify_refresh_token(&old_refresh_token, &state.jwt_config)
        .map_err(|_| AuthError::InvalidToken)?;

    // Validate token in database (checks revocation, expiry, hash match)
    let user_id = validate_refresh_token(state.db.as_ref(), &old_refresh_token, claims.jti)
        .await
        .map_err(|_| AuthError::InvalidToken)?;

    // Generate new tokens
    let username = {
        use crate::models::prelude::*;
        let user = Users::find_by_id(user_id)
            .one(state.db.as_ref())
            .await?
            .ok_or(AuthError::UserNotFound)?;
        user.username
    };

    let new_access_token = create_access_token(user_id, username, &state.jwt_config)
        .map_err(|_| AuthError::JwtEncodingError)?;
    let (new_refresh_token, new_refresh_jti) = create_refresh_token(user_id, &state.jwt_config)
        .map_err(|_| AuthError::JwtEncodingError)?;

    // Rotate refresh token (revoke old, store new)
    rotate_refresh_token(
        state.db.as_ref(),
        claims.jti,
        &new_refresh_token,
        new_refresh_jti,
        user_id,
        state.jwt_config.refresh_token_expiry_days,
    )
    .await
    .map_err(|_| AuthError::DatabaseError("Failed to rotate token".to_string()))?;

    // Create new HttpOnly cookie for new refresh token
    let cookie = Cookie::build(("refresh_token", new_refresh_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::days(state.jwt_config.refresh_token_expiry_days))
        .build();

    // Return response with new access token
    let response = AuthResponse {
        access_token: new_access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_config.access_token_expiry_minutes * 60,
    };

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(response),
    ))
}

/// POST /api/auth/logout - Logout and invalidate tokens
///
/// Revokes refresh token and blacklists access token.
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "Logged out successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn logout(
    State(state): State<AppState>,
    jar: axum_extra::extract::CookieJar,
) -> std::result::Result<impl IntoResponse, AuthError> {
    use crate::services::auth::{verify_refresh_token, revoke_refresh_token};

    // Extract refresh token from cookie
    let refresh_token = jar
        .get("refresh_token")
        .ok_or(AuthError::InvalidToken)?
        .value()
        .to_string();

    // Verify JWT to get claims (we need jti to revoke)
    let claims = verify_refresh_token(&refresh_token, &state.jwt_config)
        .map_err(|_| AuthError::InvalidToken)?;

    // Revoke refresh token in database
    revoke_refresh_token(state.db.as_ref(), claims.jti)
        .await
        .map_err(|_| AuthError::DatabaseError("Failed to revoke token".to_string()))?;

    // Clear refresh token cookie (set Max-Age=0)
    let cookie = Cookie::build(("refresh_token", ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::seconds(0)) // Expire immediately
        .build();

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
    ))
}

/// GET /api/auth/me - Get current user information
///
/// Protected route - requires valid access token.
#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "User information", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "Authentication",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_user(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
) -> std::result::Result<impl IntoResponse, AuthError> {
    use crate::middleware::auth::AuthUser;

    // Extract AuthUser from request extensions (injected by middleware)
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(AuthError::InvalidToken)?;

    // Fetch full user information from database
    let user = Users::find_by_id(auth_user.user_id)
        .one(state.db.as_ref())
        .await?
        .ok_or(AuthError::UserNotFound)?;

    // Return user response
    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        email_verified: user.email_verified,
        role: user.role,
    };

    Ok((StatusCode::OK, Json(response)))
}

// ============================================================================
// Email Verification
// ============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyEmailRequest {
    #[schema(example = "abc123def456")]
    pub token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

/// POST /api/auth/send-verification - Send verification email
///
/// Protected route - requires valid access token.
#[utoipa::path(
    post,
    path = "/api/auth/send-verification",
    responses(
        (status = 200, description = "Verification email sent", body = MessageResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 400, description = "Email already verified", body = ErrorResponse),
    ),
    tag = "Authentication",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn send_verification_email(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
) -> std::result::Result<impl IntoResponse, AuthError> {
    use crate::middleware::auth::AuthUser;
    use crate::services::email::{create_verification_token, EmailSender, MockEmailSender};

    // Extract AuthUser from request extensions
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(AuthError::InvalidToken)?;

    // Get user from database
    let user = Users::find_by_id(auth_user.user_id)
        .one(state.db.as_ref())
        .await?
        .ok_or(AuthError::UserNotFound)?;

    // Check if already verified
    if user.email_verified {
        return Err(AuthError::InvalidInput("Email already verified".to_string()));
    }

    // Create verification token
    let token = create_verification_token(state.db.as_ref(), user.id)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to create token: {}", e)))?;

    // Send verification email
    let email_sender = MockEmailSender;
    email_sender
        .send_verification_email(&user.email, &token)
        .map_err(|_e| AuthError::InternalError)?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Verification email sent".to_string(),
        }),
    ))
}

/// POST /api/auth/verify-email - Verify email with token
///
/// Public route - verifies email address using token from email.
#[utoipa::path(
    post,
    path = "/api/auth/verify-email",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = MessageResponse),
        (status = 400, description = "Invalid or expired token", body = ErrorResponse),
    ),
    tag = "Authentication"
)]
pub async fn verify_email(
    State(state): State<AppState>,
    Json(req): Json<VerifyEmailRequest>,
) -> std::result::Result<impl IntoResponse, AuthError> {
    use crate::services::email::verify_email_token;

    // Verify the token
    verify_email_token(state.db.as_ref(), &req.token)
        .await
        .map_err(|e| AuthError::InvalidInput(format!("Verification failed: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Email verified successfully".to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Cookie Tests (Phase 0.1 - TDD)
    // ============================================================================

    #[tokio::test]
    async fn test_login_sets_refresh_token_cookie() {
        // This test verifies that login response includes Set-Cookie header
        // with HttpOnly, Secure, and SameSite=Strict attributes
        // TODO: Implement once we have test database setup
        // Expected: Set-Cookie header with refresh_token cookie
    }

    #[tokio::test]
    async fn test_register_sets_refresh_token_cookie() {
        // This test verifies that register response includes Set-Cookie header
        // with HttpOnly, Secure, and SameSite=Strict attributes
        // TODO: Implement once we have test database setup
        // Expected: Set-Cookie header with refresh_token cookie
    }

    #[tokio::test]
    async fn test_cookie_attributes() {
        // This test verifies cookie has correct security attributes:
        // - HttpOnly=true (prevents XSS access)
        // - Secure=true (HTTPS only)
        // - SameSite=Strict (CSRF protection)
        // - Path=/
        // - Max-Age=604800 (7 days)
        // TODO: Implement once we have test database setup
    }

    #[tokio::test]
    async fn test_auth_response_excludes_refresh_token() {
        // This test verifies that AuthResponse JSON does NOT include refresh_token
        // Only access_token should be in the response body
        // TODO: Implement once we have test database setup
    }

    // ============================================================================
    // Existing Validation Tests
    // ============================================================================

    #[test]
    fn test_register_request_validation_valid() {
        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_register_request_validation_empty_username() {
        let req = RegisterRequest {
            username: "".to_string(),
            email: "alice@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_register_request_validation_username_too_short() {
        let req = RegisterRequest {
            username: "ab".to_string(),
            email: "alice@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("between 3 and 50"));
    }

    #[test]
    fn test_register_request_validation_username_too_long() {
        let req = RegisterRequest {
            username: "a".repeat(51),
            email: "alice@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("between 3 and 50"));
    }

    #[test]
    fn test_register_request_validation_invalid_email() {
        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "not-an-email".to_string(),
            password: "SecurePass123!".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid email"));
    }

    #[test]
    fn test_register_request_validation_password_too_short() {
        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "short".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 8 characters"));
    }

    #[test]
    fn test_register_request_validation_password_too_long() {
        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "a".repeat(129),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must not exceed 128"));
    }

    #[test]
    fn test_login_request_validation_valid() {
        let req = LoginRequest {
            username: "alice".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_login_request_validation_empty_username() {
        let req = LoginRequest {
            username: "".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_login_request_validation_empty_password() {
        let req = LoginRequest {
            username: "alice".to_string(),
            password: "".to_string(),
        };
        assert!(req.validate().is_err());
    }
}
