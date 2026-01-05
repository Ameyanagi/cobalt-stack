//! Refresh token entity for JWT token rotation.
//!
//! This module defines the `RefreshToken` entity which stores refresh tokens
//! for secure token rotation. Each refresh token has a unique ID (jti) and
//! can be revoked independently.
//!
//! # Database Mapping
//!
//! - **Table**: `refresh_tokens`
//! - **Primary Key**: `id` (UUID, matches JWT jti claim)
//! - **Unique Constraints**: `token_hash`
//! - **Foreign Key**: `user_id` → `users.id` (CASCADE on delete)
//!
//! # Security
//!
//! - Tokens are stored as SHA-256 hashes, never plaintext
//! - Each token has unique jti for rotation tracking
//! - Tokens can be revoked individually via `revoked_at`
//! - Expired tokens are cleaned up periodically
//!
//! # Token Rotation
//!
//! 1. User requests token refresh with old token
//! 2. Old token is validated and revoked
//! 3. New token pair is generated and stored
//! 4. Old token becomes unusable immediately
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack::models::{refresh_tokens, prelude::*};
//! use sea_orm::*;
//!
//! # async fn example(db: &DatabaseConnection, user_id: uuid::Uuid) -> Result<(), DbErr> {
//! // Find all active tokens for a user
//! let tokens = RefreshTokens::find()
//!     .filter(refresh_tokens::Column::UserId.eq(user_id))
//!     .filter(refresh_tokens::Column::RevokedAt.is_null())
//!     .all(db)
//!     .await?;
//! # Ok(())
//! # }
//! ```

use sea_orm::entity::prelude::*;

/// Refresh token entity for JWT token rotation.
///
/// Stores hashed refresh tokens with revocation tracking.
/// Each token corresponds to a JWT refresh token issued to a user.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "refresh_tokens")]
pub struct Model {
    /// Token ID matching the JWT's jti claim.
    /// Used for token rotation and revocation tracking.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Foreign key to the user who owns this token.
    pub user_id: Uuid,

    /// SHA-256 hash of the refresh token.
    /// Tokens are never stored in plaintext for security.
    #[sea_orm(unique)]
    pub token_hash: String,

    /// When the token expires and becomes invalid.
    /// Typically 7 days from creation.
    pub expires_at: DateTimeWithTimeZone,

    /// When the token was revoked (logout, rotation, compromise).
    /// If set, token is no longer valid regardless of expiry.
    pub revoked_at: Option<DateTimeWithTimeZone>,

    /// When the token was created.
    pub created_at: DateTimeWithTimeZone,
}

/// Entity relations for the `RefreshToken` model.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// `RefreshToken` belongs to a User.
    /// Cascades on delete: deleting user removes all their tokens.
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
