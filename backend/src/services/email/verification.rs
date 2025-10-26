// Email verification token management

use crate::models::{email_verifications, users};
use crate::utils::token::{generate_verification_token, hash_token};
use anyhow::Result;
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

/// Create a verification token for a user
pub async fn create_verification_token(db: &DatabaseConnection, user_id: Uuid) -> Result<String> {
    // Generate token and hash it
    let token = generate_verification_token();
    let token_hash = hash_token(&token);

    // Set expiration to 24 hours from now
    let expires_at = Utc::now() + Duration::hours(24);

    // Create verification record
    let verification = email_verifications::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        token_hash: Set(token_hash),
        expires_at: Set(expires_at.into()),
        verified_at: Set(None),
        created_at: Set(Utc::now().into()),
    };

    verification.insert(db).await?;

    Ok(token)
}

/// Verify an email token and mark user as verified
pub async fn verify_email_token(db: &DatabaseConnection, token: &str) -> Result<Uuid> {
    let token_hash = hash_token(token);

    // Find the verification record
    let verification = email_verifications::Entity::find()
        .filter(email_verifications::Column::TokenHash.eq(&token_hash))
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Invalid verification token"))?;

    // Check if already verified
    if verification.verified_at.is_some() {
        return Err(anyhow::anyhow!("Email already verified"));
    }

    // Check if expired
    let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
    if verification.expires_at < now {
        return Err(anyhow::anyhow!("Verification token expired"));
    }

    // Mark verification as complete
    let mut active_verification: email_verifications::ActiveModel = verification.clone().into();
    active_verification.verified_at = Set(Some(Utc::now().into()));
    active_verification.update(db).await?;

    // Update user's email_verified flag
    let user = users::Entity::find_by_id(verification.user_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    let mut active_user: users::ActiveModel = user.into();
    active_user.email_verified = Set(true);
    active_user.update(db).await?;

    Ok(verification.user_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require a test database setup
    // For now, we define the test structure but won't run them without DB

    #[test]
    #[ignore] // Ignore until we have test database setup
    fn test_create_verification_token_structure() {
        // Test would verify:
        // 1. Token is generated correctly
        // 2. Token hash is stored in database
        // 3. Expiration is set to 24 hours from now
        // 4. Verification record is created with correct user_id
    }

    #[test]
    #[ignore]
    fn test_verify_email_token_success() {
        // Test would verify:
        // 1. Valid token verifies successfully
        // 2. User's email_verified flag is set to true
        // 3. Verification record is marked with verified_at timestamp
    }

    #[test]
    #[ignore]
    fn test_verify_email_token_already_verified() {
        // Test would verify:
        // 1. Already verified token returns error
    }

    #[test]
    #[ignore]
    fn test_verify_email_token_expired() {
        // Test would verify:
        // 1. Expired token returns error
    }

    #[test]
    #[ignore]
    fn test_verify_email_token_invalid() {
        // Test would verify:
        // 1. Invalid token returns error
    }
}
