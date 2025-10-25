use crate::services::auth::{verify_access_token, AuthError, JwtConfig};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// User claims extracted from JWT token
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub username: String,
}

/// Extract JWT token from Authorization header
/// Expected format: "Bearer <token>"
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

/// Auth middleware that validates JWT tokens and injects user claims
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
