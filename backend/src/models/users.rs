//! User entity representing authenticated users.
//!
//! This module defines the `User` entity which represents a user account
//! in the system. Users can authenticate via password or OAuth providers.
//!
//! # Database Mapping
//!
//! - **Table**: `users`
//! - **Primary Key**: `id` (UUID, not auto-increment)
//! - **Unique Constraints**: `username`, `email`
//!
//! # Lifecycle
//!
//! 1. User registers with username/email/password
//! 2. Email verification token is sent
//! 3. User verifies email (`email_verified = true`)
//! 4. User can authenticate and access protected resources
//! 5. Admin can disable account (`disabled_at` is set)
//!
//! # Relations
//!
//! - `has_many` RefreshTokens: User's active refresh tokens
//! - `has_many` EmailVerifications: Email verification history
//! - `has_many` OAuthAccounts: Linked OAuth provider accounts
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack::models::{users, prelude::*};
//! use sea_orm::*;
//!
//! # async fn example(db: &DatabaseConnection) -> Result<(), DbErr> {
//! // Find user by username
//! let user = Users::find()
//!     .filter(users::Column::Username.eq("alice"))
//!     .one(db)
//!     .await?;
//!
//! // Check if user is active
//! if let Some(user) = user {
//!     let is_active = user.disabled_at.is_none() && user.email_verified;
//! }
//! # Ok(())
//! # }
//! ```

use super::sea_orm_active_enums::UserRole;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// User account entity.
///
/// Represents an authenticated user in the system with support for
/// both password-based and OAuth authentication.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// Unique user identifier (UUID v4).
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Unique username for authentication.
    /// Must be 3-50 characters, validated at handler layer.
    #[sea_orm(unique)]
    pub username: String,

    /// Unique email address for authentication and notifications.
    /// Must contain '@', validated at handler layer.
    #[sea_orm(unique)]
    pub email: String,

    /// Argon2 password hash.
    /// Optional because OAuth users may not have passwords.
    pub password_hash: Option<String>,

    /// Whether the user's email has been verified.
    /// Set to true after successful email verification.
    pub email_verified: bool,

    /// Timestamp when the account was created.
    pub created_at: DateTimeWithTimeZone,

    /// Timestamp when the account was last updated.
    /// Updated automatically on any field change.
    pub updated_at: DateTimeWithTimeZone,

    /// User's role for authorization (User or Admin).
    /// See [`UserRole`].
    pub role: UserRole,

    /// Timestamp when the account was disabled by an admin.
    /// If set, user cannot authenticate.
    pub disabled_at: Option<DateTimeWithTimeZone>,

    /// Timestamp of the user's last successful login.
    /// Updated on each successful authentication.
    pub last_login_at: Option<DateTimeWithTimeZone>,
}

/// Entity relations for the User model.
///
/// Defines one-to-many relationships with related entities.
/// All related entities cascade delete when user is deleted.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// User has many email verification records.
    /// Cascades on delete: deleting user removes verification records.
    #[sea_orm(has_many = "super::email_verifications::Entity")]
    EmailVerifications,

    /// User has many OAuth account linkages.
    /// Cascades on delete: deleting user removes OAuth linkages.
    #[sea_orm(has_many = "super::o_auth_accounts::Entity")]
    OAuthAccounts,

    /// User has many refresh tokens.
    /// Cascades on delete: deleting user removes all tokens.
    #[sea_orm(has_many = "super::refresh_tokens::Entity")]
    RefreshTokens,
}

impl Related<super::email_verifications::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmailVerifications.def()
    }
}

impl Related<super::o_auth_accounts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OAuthAccounts.def()
    }
}

impl Related<super::refresh_tokens::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RefreshTokens.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
