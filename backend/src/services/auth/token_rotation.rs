use super::{AuthError, Result};
use crate::models::{prelude::*, refresh_tokens};
use chrono::{DateTime, Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Store a refresh token in the database
///
/// The token is hashed before storage for security
pub async fn store_refresh_token(
    db: &DatabaseConnection,
    user_id: Uuid,
    token: &str,
    jti: Uuid,
    expires_in_days: i64,
) -> Result<()> {
    let token_hash = hash_token(token);
    let expires_at = Utc::now() + Duration::days(expires_in_days);

    let refresh_token = refresh_tokens::ActiveModel {
        id: Set(jti),
        user_id: Set(user_id),
        token_hash: Set(token_hash),
        expires_at: Set(expires_at.into()),
        revoked_at: Set(None),
        created_at: Set(Utc::now().into()),
    };

    refresh_token.insert(db).await?;
    Ok(())
}

/// Validate a refresh token
///
/// Checks that:
/// - Token exists in database
/// - Token hash matches
/// - Token is not expired
/// - Token is not revoked
pub async fn validate_refresh_token(
    db: &DatabaseConnection,
    token: &str,
    jti: Uuid,
) -> Result<Uuid> {
    let token_hash = hash_token(token);

    let stored_token = RefreshTokens::find_by_id(jti)
        .one(db)
        .await?
        .ok_or(AuthError::InvalidToken)?;

    // Check if token hash matches
    if stored_token.token_hash != token_hash {
        return Err(AuthError::InvalidToken.into());
    }

    // Check if token is revoked
    if stored_token.revoked_at.is_some() {
        return Err(AuthError::TokenBlacklisted.into());
    }

    // Check if token is expired
    let now: DateTime<Utc> = Utc::now();
    let expires_at: DateTime<Utc> = stored_token.expires_at.with_timezone(&Utc);
    if expires_at < now {
        return Err(AuthError::TokenExpired.into());
    }

    Ok(stored_token.user_id)
}

/// Revoke a refresh token
///
/// Marks the token as revoked in the database
pub async fn revoke_refresh_token(db: &DatabaseConnection, jti: Uuid) -> Result<()> {
    let stored_token = RefreshTokens::find_by_id(jti)
        .one(db)
        .await?
        .ok_or(AuthError::InvalidToken)?;

    let mut active_token: refresh_tokens::ActiveModel = stored_token.into();
    active_token.revoked_at = Set(Some(Utc::now().into()));
    active_token.update(db).await?;

    Ok(())
}

/// Rotate refresh token (revoke old, issue new)
///
/// This implements token rotation pattern for enhanced security
pub async fn rotate_refresh_token(
    db: &DatabaseConnection,
    old_jti: Uuid,
    new_token: &str,
    new_jti: Uuid,
    user_id: Uuid,
    expires_in_days: i64,
) -> Result<()> {
    // Revoke old token
    revoke_refresh_token(db, old_jti).await?;

    // Store new token
    store_refresh_token(db, user_id, new_token, new_jti, expires_in_days).await?;

    Ok(())
}

/// Revoke all refresh tokens for a user (logout from all devices)
pub async fn revoke_all_user_tokens(db: &DatabaseConnection, user_id: Uuid) -> Result<()> {
    let tokens = RefreshTokens::find()
        .filter(refresh_tokens::Column::UserId.eq(user_id))
        .filter(refresh_tokens::Column::RevokedAt.is_null())
        .all(db)
        .await?;

    for token in tokens {
        let mut active_token: refresh_tokens::ActiveModel = token.into();
        active_token.revoked_at = Set(Some(Utc::now().into()));
        active_token.update(db).await?;
    }

    Ok(())
}

/// Clean up expired tokens (for maintenance tasks)
///
/// Deletes tokens that have been expired for more than `retention_days`
pub async fn cleanup_expired_tokens(db: &DatabaseConnection, retention_days: i64) -> Result<u64> {
    let cutoff = Utc::now() - Duration::days(retention_days);

    let result = RefreshTokens::delete_many()
        .filter(refresh_tokens::Column::ExpiresAt.lt(cutoff))
        .exec(db)
        .await?;

    Ok(result.rows_affected)
}

/// Hash a token for storage
///
/// Uses SHA-256 for token hashing
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase};

    fn mock_refresh_token(
        id: Uuid,
        user_id: Uuid,
        token_hash: String,
        expired: bool,
        revoked: bool,
    ) -> refresh_tokens::Model {
        let now = Utc::now();
        refresh_tokens::Model {
            id,
            user_id,
            token_hash,
            expires_at: if expired {
                (now - Duration::hours(1)).into()
            } else {
                (now + Duration::days(7)).into()
            },
            revoked_at: if revoked { Some(now.into()) } else { None },
            created_at: now.into(),
        }
    }

    #[test]
    fn test_hash_token() {
        let token = "test_token_123";
        let hash = hash_token(token);

        // SHA-256 produces 64 character hex string
        assert_eq!(hash.len(), 64);

        // Same token produces same hash
        assert_eq!(hash, hash_token(token));

        // Different token produces different hash
        assert_ne!(hash, hash_token("different_token"));
    }

    #[test]
    fn test_hash_token_consistency() {
        // Test that hash is deterministic
        let token = "my_secret_token_456";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);
        assert_eq!(hash1, hash2);

        // Test uniqueness
        let different_hash = hash_token("my_secret_token_457");
        assert_ne!(hash1, different_hash);
    }

    #[tokio::test]
    async fn test_validate_refresh_token_valid() {
        let user_id = Uuid::new_v4();
        let jti = Uuid::new_v4();
        let token = "test_token";
        let token_hash = hash_token(token);

        let mock_token = mock_refresh_token(jti, user_id, token_hash, false, false);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[mock_token]])
            .into_connection();

        let result = validate_refresh_token(&db, token, jti).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user_id);
    }

    #[tokio::test]
    async fn test_validate_refresh_token_not_found() {
        let empty_results: Vec<Vec<refresh_tokens::Model>> = vec![vec![]];
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(empty_results)
            .into_connection();

        let jti = Uuid::new_v4();
        let result = validate_refresh_token(&db, "any_token", jti).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid token"));
    }

    #[tokio::test]
    async fn test_validate_refresh_token_wrong_hash() {
        let user_id = Uuid::new_v4();
        let jti = Uuid::new_v4();
        let token_hash = hash_token("correct_token");

        let mock_token = mock_refresh_token(jti, user_id, token_hash, false, false);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[mock_token]])
            .into_connection();

        let result = validate_refresh_token(&db, "wrong_token", jti).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid token"));
    }

    #[tokio::test]
    async fn test_validate_refresh_token_revoked() {
        let user_id = Uuid::new_v4();
        let jti = Uuid::new_v4();
        let token = "test_token";
        let token_hash = hash_token(token);

        let mock_token = mock_refresh_token(jti, user_id, token_hash, false, true);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[mock_token]])
            .into_connection();

        let result = validate_refresh_token(&db, token, jti).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Token blacklisted"));
    }

    #[tokio::test]
    async fn test_validate_refresh_token_expired() {
        let user_id = Uuid::new_v4();
        let jti = Uuid::new_v4();
        let token = "test_token";
        let token_hash = hash_token(token);

        let mock_token = mock_refresh_token(jti, user_id, token_hash, true, false);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[mock_token]])
            .into_connection();

        let result = validate_refresh_token(&db, token, jti).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Token expired"));
    }

    // Note: Write operation tests (store, revoke, rotate, cleanup) will be covered
    // by integration tests in Phase 4 with actual database connections.
    // Unit testing these with mock database is complex and provides limited value
    // compared to integration tests that verify actual database behavior.
}
