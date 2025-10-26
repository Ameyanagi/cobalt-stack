//! OAuth account linkage entity for third-party authentication.
//!
//! This module defines the `OAuthAccount` entity which links user accounts
//! to OAuth providers (Google, GitHub, etc.) for social login.
//!
//! # Database Mapping
//!
//! - **Table**: `o_auth_accounts`
//! - **Primary Key**: `id` (UUID)
//! - **Foreign Key**: `user_id` → `users.id` (CASCADE on delete)
//!
//! # OAuth Flow
//!
//! 1. User initiates OAuth login with provider (Google, GitHub, etc.)
//! 2. Provider redirects back with authorization code
//! 3. Backend exchanges code for access/refresh tokens
//! 4. User is looked up by `provider_user_id` or created
//! 5. OAuth account is linked to user account
//!
//! # Token Storage
//!
//! - `access_token`: OAuth provider access token (stored encrypted)
//! - `refresh_token`: OAuth provider refresh token (stored encrypted)
//! - `expires_at`: When the OAuth access token expires
//!
//! # Security
//!
//! - OAuth tokens should be encrypted at rest (TODO: implement encryption)
//! - Tokens are never exposed in API responses
//! - One user can have multiple OAuth providers linked
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack::models::{o_auth_accounts, prelude::*};
//! use sea_orm::*;
//!
//! # async fn example(db: &DatabaseConnection) -> Result<(), DbErr> {
//! // Find Google account by provider user ID
//! let account = OAuthAccounts::find()
//!     .filter(o_auth_accounts::Column::Provider.eq("google"))
//!     .filter(o_auth_accounts::Column::ProviderUserId.eq("123456789"))
//!     .one(db)
//!     .await?;
//! # Ok(())
//! # }
//! ```

use sea_orm::entity::prelude::*;

/// OAuth account linkage entity.
///
/// Links user accounts to OAuth providers for social login.
/// Stores OAuth tokens for API access to provider services.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "o_auth_accounts")]
pub struct Model {
    /// Unique identifier for this OAuth linkage.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Foreign key to the linked user account.
    pub user_id: Uuid,

    /// OAuth provider name (e.g., "google", "github", "facebook").
    /// Lowercase string identifier for the provider.
    pub provider: String,

    /// User's unique identifier at the OAuth provider.
    /// Used to look up existing linkages on subsequent logins.
    pub provider_user_id: String,

    /// OAuth access token from the provider.
    /// Used to access provider APIs on behalf of the user.
    /// TODO: Should be encrypted at rest.
    #[sea_orm(column_type = "Text", nullable)]
    pub access_token: Option<String>,

    /// OAuth refresh token from the provider.
    /// Used to obtain new access tokens when they expire.
    /// TODO: Should be encrypted at rest.
    #[sea_orm(column_type = "Text", nullable)]
    pub refresh_token: Option<String>,

    /// When the OAuth access token expires.
    /// If set, token should be refreshed before this time.
    pub expires_at: Option<DateTimeWithTimeZone>,

    /// When this OAuth linkage was created.
    pub created_at: DateTimeWithTimeZone,
}

/// Entity relations for the OAuthAccount model.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// OAuthAccount belongs to a User.
    /// Cascades on delete: deleting user removes OAuth linkages.
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
