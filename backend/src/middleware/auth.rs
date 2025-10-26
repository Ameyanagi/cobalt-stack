//! JWT authentication middleware for Axum.
//!
//! This module provides authentication middleware that validates JWT tokens
//! from the Authorization header and injects authenticated user information
//! into request extensions for downstream handlers.
//!
//! # Security
//!
//! - Validates JWT signature and expiration
//! - Requires "Bearer \<token\>" format in Authorization header
//! - Returns 401 Unauthorized for missing/invalid tokens
//! - Injects [`AuthUser`] into request extensions for handler access
//!
//! # Usage
//!
//! ```no_run
//! use axum::{Router, routing::get, middleware};
//! use cobalt_stack_backend::middleware::auth::auth_middleware;
//! use cobalt_stack_backend::services::auth::JwtConfig;
//!
//! # async fn example() {
//! let jwt_config = JwtConfig::from_env();
//!
//! let app = Router::new()
//!     .route("/protected", get(protected_handler))
//!     .layer(middleware::from_fn_with_state(
//!         jwt_config,
//!         auth_middleware
//!     ));
//! # }
//!
//! # async fn protected_handler() -> &'static str { "OK" }
//! ```
//!
//! # Handler Access
//!
//! Protected handlers can access authenticated user via request extensions:
//!
//! ```no_run
//! use axum::extract::Request;
//! use cobalt_stack_backend::middleware::auth::AuthUser;
//!
//! async fn handler(req: Request) -> String {
//!     let auth_user = req.extensions().get::<AuthUser>().unwrap();
//!     format!("Hello, {}!", auth_user.username)
//! }
//! ```

use crate::services::auth::{verify_access_token, AuthError, JwtConfig};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Authenticated user information extracted from JWT token.
///
/// This struct is injected into request extensions by [`auth_middleware`]
/// and can be accessed by downstream handlers to identify the authenticated user.
///
/// # Fields
///
/// - `user_id`: Unique identifier of the authenticated user
/// - `username`: Username of the authenticated user
///
/// # Examples
///
/// ```no_run
/// use axum::extract::Request;
/// use cobalt_stack_backend::middleware::auth::AuthUser;
///
/// async fn handler(req: Request) -> String {
///     let auth_user = req.extensions().get::<AuthUser>().unwrap();
///     format!("User ID: {}, Username: {}", auth_user.user_id, auth_user.username)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// Unique identifier of the authenticated user.
    pub user_id: Uuid,
    /// Username of the authenticated user.
    pub username: String,
}

/// Extract JWT token from Authorization header.
///
/// Parses the Authorization header and extracts the JWT token.
/// Expected header format: `Authorization: Bearer <token>`
///
/// # Arguments
///
/// * `headers` - HTTP request headers
///
/// # Returns
///
/// - `Ok(String)` - Extracted JWT token
/// - `Err(AuthError::InvalidToken)` - Missing header, invalid format, or empty token
///
/// # Examples
///
/// ```
/// use axum::http::HeaderMap;
/// # use cobalt_stack_backend::middleware::auth::extract_token_from_header;
///
/// # fn example() {
/// let mut headers = HeaderMap::new();
/// headers.insert("authorization", "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...".parse().unwrap());
///
/// // let token = extract_token_from_header(&headers).unwrap();
/// // assert!(token.starts_with("eyJ"));
/// # }
/// ```
///
/// # Errors
///
/// Returns [`AuthError::InvalidToken`] if:
/// - Authorization header is missing
/// - Header value is not valid UTF-8
/// - Header doesn't start with "Bearer "
/// - Token portion is empty after "Bearer " prefix
fn extract_token_from_header(headers: &HeaderMap) -> Result<String, AuthError> {
    let auth_header = headers
        .get("authorization")
        .ok_or(AuthError::InvalidToken)?
        .to_str()
        .map_err(|_| AuthError::InvalidToken)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidToken);
    }

    let token = auth_header.trim_start_matches("Bearer ").to_string();
    
    if token.is_empty() {
        return Err(AuthError::InvalidToken);
    }

    Ok(token)
}

/// Axum middleware that validates JWT tokens and injects authenticated user.
///
/// This middleware extracts and validates the JWT token from the Authorization header,
/// then injects the authenticated user's information into request extensions for
/// downstream handlers to access.
///
/// # Flow
///
/// 1. Extract token from `Authorization: Bearer <token>` header
/// 2. Verify token signature and validate expiration
/// 3. Extract user claims (user_id, username) from token
/// 4. Create [`AuthUser`] and inject into request extensions
/// 5. Pass request to next middleware/handler
///
/// # Arguments
///
/// * `jwt_config` - JWT configuration with secret and expiry settings
/// * `req` - Incoming HTTP request
/// * `next` - Next middleware/handler in chain
///
/// # Returns
///
/// - `Ok(Response)` - Request processed successfully by downstream handler
/// - `Err(StatusCode::UNAUTHORIZED)` - Token missing, invalid, or expired
///
/// # Examples
///
/// ```no_run
/// use axum::{Router, routing::get, middleware};
/// use cobalt_stack_backend::middleware::auth::auth_middleware;
/// use cobalt_stack_backend::services::auth::JwtConfig;
///
/// # async fn example() {
/// let jwt_config = JwtConfig::from_env();
///
/// let protected_routes = Router::new()
///     .route("/profile", get(get_profile))
///     .layer(middleware::from_fn_with_state(
///         jwt_config,
///         auth_middleware
///     ));
/// # }
/// # async fn get_profile() -> &'static str { "Profile" }
/// ```
///
/// # Security Notes
///
/// - Always use HTTPS in production to protect tokens in transit
/// - Token validation includes signature check and expiration check
/// - Invalid tokens return 401 Unauthorized without detailed error messages
/// - This middleware should be applied to all protected routes
pub async fn auth_middleware(
    State(jwt_config): State<JwtConfig>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from header
    let token = extract_token_from_header(req.headers())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Verify token
    let claims = verify_access_token(&token, &jwt_config)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Create AuthUser from claims
    let auth_user = AuthUser {
        user_id: claims.sub,
        username: claims.username,
    };

    // Inject user into request extensions
    req.extensions_mut().insert(auth_user);

    // Continue to next middleware/handler
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::auth::create_access_token;

    fn test_jwt_config() -> JwtConfig {
        JwtConfig {
            secret: "test_secret_key_for_middleware".to_string(),
            access_token_expiry_minutes: 30,
            refresh_token_expiry_days: 7,
        }
    }

    #[test]
    fn test_extract_token_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer valid_token_here".parse().unwrap(),
        );

        let result = extract_token_from_header(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "valid_token_here");
    }

    #[test]
    fn test_extract_token_no_header() {
        let headers = HeaderMap::new();
        let result = extract_token_from_header(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_no_bearer_prefix() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "InvalidFormat".parse().unwrap());

        let result = extract_token_from_header(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_empty_token() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer ".parse().unwrap());

        let result = extract_token_from_header(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_case_sensitive() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "bearer token".parse().unwrap());

        let result = extract_token_from_header(&headers);
        // Should fail because "bearer" is lowercase, we expect "Bearer"
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_valid_token() {
        let config = test_jwt_config();
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();

        // Create a valid token
        let token = create_access_token(user_id, username.clone(), &config).unwrap();

        // Verify it
        let result = verify_access_token(&token, &config);
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);
    }

    #[tokio::test]
    async fn test_verify_invalid_token() {
        let config = test_jwt_config();
        let result = verify_access_token("invalid.token.format", &config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_token_wrong_secret() {
        let config = test_jwt_config();
        let user_id = Uuid::new_v4();

        // Create token with one secret
        let token = create_access_token(user_id, "testuser".to_string(), &config).unwrap();

        // Try to verify with different secret
        let wrong_config = JwtConfig {
            secret: "different_secret".to_string(),
            ..config
        };

        let result = verify_access_token(&token, &wrong_config);
        assert!(result.is_err());
    }
}
