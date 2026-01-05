//! Email verification entity for email address confirmation.
//!
//! This module defines the `EmailVerification` entity which stores email
//! verification tokens sent to users to confirm their email addresses.
//!
//! # Database Mapping
//!
//! - **Table**: `email_verifications`
//! - **Primary Key**: `id` (UUID)
//! - **Unique Constraints**: `token_hash`
//! - **Foreign Key**: `user_id` → `users.id` (CASCADE on delete)
//!
//! # Verification Flow
//!
//! 1. User registers or requests email verification
//! 2. Random token is generated and stored as hash
//! 3. Email is sent with verification link containing token
//! 4. User clicks link, token is validated
//! 5. `verified_at` is set, `users.email_verified` is set to true
//!
//! # Security
//!
//! - Tokens are stored as SHA-256 hashes, never plaintext
//! - Tokens expire after 24 hours
//! - One-time use: `verified_at` prevents reuse
//! - Expired tokens are cleaned up periodically
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack::models::{email_verifications, prelude::*};
//! use sea_orm::*;
//! use chrono::Utc;
//!
//! # async fn example(db: &DatabaseConnection, user_id: uuid::Uuid) -> Result<(), DbErr> {
//! // Find pending verification tokens for a user
//! let pending = EmailVerifications::find()
//!     .filter(email_verifications::Column::UserId.eq(user_id))
//!     .filter(email_verifications::Column::VerifiedAt.is_null())
//!     .filter(email_verifications::Column::ExpiresAt.gt(Utc::now()))
//!     .all(db)
//!     .await?;
//! # Ok(())
//! # }
//! ```

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Email verification token entity.
///
/// Stores email verification tokens for confirming user email addresses.
/// Tokens are one-time use and expire after 24 hours.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "email_verifications")]
pub struct Model {
    /// Unique identifier for this verification record.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Foreign key to the user requesting verification.
    pub user_id: Uuid,

    /// SHA-256 hash of the verification token.
    /// Token is sent via email, never stored in plaintext.
    #[sea_orm(unique)]
    pub token_hash: String,

    /// When the token expires (typically 24 hours from creation).
    /// Expired tokens cannot be used for verification.
    pub expires_at: DateTimeWithTimeZone,

    /// When the token was successfully used for verification.
    /// If set, token cannot be reused (one-time use).
    pub verified_at: Option<DateTimeWithTimeZone>,

    /// When the verification token was created.
    pub created_at: DateTimeWithTimeZone,
}

/// Entity relations for the `EmailVerification` model.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// `EmailVerification` belongs to a User.
    /// Cascades on delete: deleting user removes verification records.
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
