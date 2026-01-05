# Domain Models and Entities

## Table of Contents
- [Overview](#overview)
- [SeaORM Entity Patterns](#seaorm-entity-patterns)
- [Core Entities](#core-entities)
- [Entity Relationships](#entity-relationships)
- [Active Enums](#active-enums)
- [Model Best Practices](#model-best-practices)

## Overview

The **Model Layer** defines database entities and domain models using SeaORM. These entities represent database tables and provide type-safe access to data through Rust structs.

**Key Concepts:**
- **Entity**: Immutable representation of database rows
- **ActiveModel**: Mutable builder for inserting/updating data
- **Relations**: Foreign key relationships between entities
- **ActiveEnums**: Type-safe database enum types

## SeaORM Entity Patterns

### Entity Structure

SeaORM entities consist of three main components:

```rust
// 1. Model struct - immutable data representation
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub username: String,
    pub email: String,
    // ... fields
}

// 2. Relation enum - defines foreign key relationships
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::refresh_tokens::Entity")]
    RefreshTokens,
}

// 3. ActiveModel behavior - lifecycle hooks
impl ActiveModelBehavior for ActiveModel {}
```

### Primary Keys

#### UUID Primary Keys

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    // UUID primary key, not auto-increment
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    // ...
}
```

**Why UUIDs?**
- Globally unique across distributed systems
- Can be generated client-side
- No sequential enumeration attacks
- Better for privacy (non-predictable IDs)

### Creating and Updating

#### Inserting New Records

```rust
use sea_orm::{ActiveModelTrait, Set};

// Create new user
let user = users::ActiveModel {
    id: Set(Uuid::new_v4()),
    username: Set("alice".to_string()),
    email: Set("alice@example.com".to_string()),
    password_hash: Set(Some(hashed_password)),
    email_verified: Set(false),
    created_at: Set(Utc::now().into()),
    updated_at: Set(Utc::now().into()),
    ..Default::default()
};

let user: users::Model = user.insert(db).await?;
```

#### Updating Records

```rust
// Find existing user
let user = Users::find_by_id(user_id)
    .one(db)
    .await?
    .ok_or(NotFoundError)?;

// Convert to ActiveModel for mutation
let mut user: users::ActiveModel = user.into();

// Update fields
user.email_verified = Set(true);
user.updated_at = Set(Utc::now().into());

// Save changes
let updated_user = user.update(db).await?;
```

### Querying

#### Basic Queries

```rust
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

// Find by ID
let user = Users::find_by_id(user_id)
    .one(db)
    .await?;

// Find by column value
let user = Users::find()
    .filter(users::Column::Username.eq("alice"))
    .one(db)
    .await?;

// Find all with conditions
let verified_users = Users::find()
    .filter(users::Column::EmailVerified.eq(true))
    .all(db)
    .await?;
```

#### Complex Queries

```rust
use sea_orm::{QuerySelect, QueryOrder};

// Query with multiple conditions and ordering
let active_admins = Users::find()
    .filter(users::Column::Role.eq(UserRole::Admin))
    .filter(users::Column::DisabledAt.is_null())
    .order_by_asc(users::Column::CreatedAt)
    .limit(10)
    .all(db)
    .await?;

// Count records
let user_count = Users::find()
    .filter(users::Column::EmailVerified.eq(true))
    .count(db)
    .await?;
```

## Core Entities

### User Entity

**Location**: `models/users.rs`

```rust
/// User account entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// Unique user identifier (UUID v4)
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Unique username for authentication
    #[sea_orm(unique)]
    pub username: String,

    /// Unique email address
    #[sea_orm(unique)]
    pub email: String,

    /// Argon2 password hash (optional for OAuth users)
    pub password_hash: Option<String>,

    /// Whether email has been verified
    pub email_verified: bool,

    /// Account creation timestamp
    pub created_at: DateTimeWithTimeZone,

    /// Last update timestamp
    pub updated_at: DateTimeWithTimeZone,

    /// User's authorization role
    pub role: UserRole,

    /// When account was disabled (if applicable)
    pub disabled_at: Option<DateTimeWithTimeZone>,

    /// Last successful login timestamp
    pub last_login_at: Option<DateTimeWithTimeZone>,
}
```

**Relationships:**
- `has_many` RefreshTokens (one user has many refresh tokens)
- `has_many` EmailVerifications (one user has many verification attempts)
- `has_many` OAuthAccounts (one user can link multiple OAuth providers)

**Usage Example:**

```rust
// Create new user
let user = users::ActiveModel {
    username: Set("alice".to_string()),
    email: Set("alice@example.com".to_string()),
    password_hash: Set(Some(hash_password("password")?)),
    email_verified: Set(false),
    role: Set(UserRole::User),
    created_at: Set(Utc::now().into()),
    updated_at: Set(Utc::now().into()),
    ..Default::default()
};

let user = user.insert(db).await?;

// Query by username
let user = Users::find()
    .filter(users::Column::Username.eq("alice"))
    .one(db)
    .await?
    .ok_or(NotFoundError)?;

// Check if user is active
let is_active = user.disabled_at.is_none() && user.email_verified;
```

### RefreshToken Entity

**Location**: `models/refresh_tokens.rs`

```rust
/// Refresh token entity for JWT token rotation
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "refresh_tokens")]
pub struct Model {
    /// Token ID matching JWT's jti claim
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Foreign key to user
    pub user_id: Uuid,

    /// SHA-256 hash of the refresh token
    #[sea_orm(unique)]
    pub token_hash: String,

    /// Token expiration timestamp
    pub expires_at: DateTimeWithTimeZone,

    /// Token revocation timestamp (if revoked)
    pub revoked_at: Option<DateTimeWithTimeZone>,

    /// Token creation timestamp
    pub created_at: DateTimeWithTimeZone,
}
```

**Relationships:**
- `belongs_to` Users (many refresh tokens belong to one user)

**Usage Example:**

```rust
// Store new refresh token
let token = refresh_tokens::ActiveModel {
    id: Set(jti),
    user_id: Set(user_id),
    token_hash: Set(hash_token(&refresh_token)),
    expires_at: Set((Utc::now() + Duration::days(7)).into()),
    revoked_at: Set(None),
    created_at: Set(Utc::now().into()),
    ..Default::default()
};

token.insert(db).await?;

// Find active tokens for user
let active_tokens = RefreshTokens::find()
    .filter(refresh_tokens::Column::UserId.eq(user_id))
    .filter(refresh_tokens::Column::RevokedAt.is_null())
    .filter(refresh_tokens::Column::ExpiresAt.gt(Utc::now()))
    .all(db)
    .await?;

// Revoke token
let mut token: refresh_tokens::ActiveModel = token.into();
token.revoked_at = Set(Some(Utc::now().into()));
token.update(db).await?;
```

### EmailVerification Entity

**Location**: `models/email_verifications.rs`

```rust
/// Email verification entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "email_verifications")]
pub struct Model {
    /// Verification record ID
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Foreign key to user
    pub user_id: Uuid,

    /// Verification token (hex-encoded random bytes)
    #[sea_orm(unique)]
    pub token: String,

    /// Token expiration timestamp
    pub expires_at: DateTimeWithTimeZone,

    /// When token was used (if verified)
    pub verified_at: Option<DateTimeWithTimeZone>,

    /// Token creation timestamp
    pub created_at: DateTimeWithTimeZone,
}
```

**Relationships:**
- `belongs_to` Users (many verifications belong to one user)

**Usage Example:**

```rust
// Create verification token
let verification = email_verifications::ActiveModel {
    user_id: Set(user_id),
    token: Set(generate_random_token()),
    expires_at: Set((Utc::now() + Duration::hours(24)).into()),
    verified_at: Set(None),
    created_at: Set(Utc::now().into()),
    ..Default::default()
};

verification.insert(db).await?;

// Verify email with token
let verification = EmailVerifications::find()
    .filter(email_verifications::Column::Token.eq(token))
    .filter(email_verifications::Column::VerifiedAt.is_null())
    .filter(email_verifications::Column::ExpiresAt.gt(Utc::now()))
    .one(db)
    .await?
    .ok_or(InvalidTokenError)?;
```

### OAuthAccount Entity

**Location**: `models/o_auth_accounts.rs`

```rust
/// OAuth account linkage entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "o_auth_accounts")]
pub struct Model {
    /// OAuth linkage ID
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Foreign key to user
    pub user_id: Uuid,

    /// OAuth provider (e.g., "google", "github")
    pub provider: String,

    /// Provider's user ID
    pub provider_user_id: String,

    /// Account linkage timestamp
    pub created_at: DateTimeWithTimeZone,
}
```

**Relationships:**
- `belongs_to` Users (many OAuth accounts can link to one user)

## Entity Relationships

### One-to-Many Relationships

```rust
// User has many RefreshTokens
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::refresh_tokens::Entity")]
    RefreshTokens,
}

// Query with related data
let user_with_tokens = Users::find_by_id(user_id)
    .find_with_related(RefreshTokens)
    .all(db)
    .await?;
```

### Belongs-To Relationships

```rust
// RefreshToken belongs to User
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_delete = "Cascade"
    )]
    Users,
}

// Query with parent
let token_with_user = RefreshTokens::find_by_id(token_id)
    .find_also_related(Users)
    .one(db)
    .await?;
```

### Cascade Behavior

```rust
// Deleting user cascades to refresh tokens
#[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::UserId",
    to = "super::users::Column::Id",
    on_delete = "Cascade"  // Delete tokens when user is deleted
)]
Users,

// In practice:
Users::delete_by_id(user_id).exec(db).await?;
// This automatically deletes all related refresh_tokens, email_verifications, etc.
```

## Active Enums

**Location**: `models/sea_orm_active_enums.rs`

### UserRole Enum

```rust
/// User role enum for RBAC
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Standard user with basic permissions
    #[sea_orm(string_value = "user")]
    User,

    /// Administrator with elevated permissions
    #[sea_orm(string_value = "admin")]
    Admin,
}
```

**Usage:**

```rust
// Create admin user
let admin = users::ActiveModel {
    role: Set(UserRole::Admin),
    // ...
};

// Query by role
let admins = Users::find()
    .filter(users::Column::Role.eq(UserRole::Admin))
    .all(db)
    .await?;

// Check role
if user.role == UserRole::Admin {
    // Grant admin access
}

// Serialize to JSON
let json = serde_json::to_string(&user.role)?;
// Output: "admin"
```

## Model Best Practices

### 1. Use Appropriate Types

```rust
// ✅ Good: Use specific types
pub struct Model {
    pub id: Uuid,                          // Not String
    pub created_at: DateTimeWithTimeZone,  // Not String
    pub role: UserRole,                    // Not String
}

// ❌ Bad: Everything as String
pub struct Model {
    pub id: String,
    pub created_at: String,
    pub role: String,
}
```

### 2. Make Optional Fields Explicit

```rust
// ✅ Good: Optional fields use Option<T>
pub struct Model {
    pub password_hash: Option<String>,     // OAuth users may not have password
    pub disabled_at: Option<DateTimeWithTimeZone>,  // Most users are not disabled
    pub last_login_at: Option<DateTimeWithTimeZone>, // May not have logged in yet
}
```

### 3. Use Unique Constraints

```rust
// ✅ Good: Unique constraints at database level
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(unique)]
    pub username: String,

    #[sea_orm(unique)]
    pub email: String,
}
```

### 4. Timestamp Management

```rust
// ✅ Good: Always set timestamps
let user = users::ActiveModel {
    created_at: Set(Utc::now().into()),
    updated_at: Set(Utc::now().into()),
    // ...
};

// When updating, refresh updated_at
let mut user: users::ActiveModel = user.into();
user.updated_at = Set(Utc::now().into());
user.update(db).await?;
```

### 5. Use Transactions for Related Changes

```rust
// ✅ Good: Transaction for atomic operations
let txn = db.begin().await?;

let user = users::ActiveModel { /* ... */ };
let user = user.insert(&txn).await?;

let token = refresh_tokens::ActiveModel {
    user_id: Set(user.id),
    // ...
};
token.insert(&txn).await?;

txn.commit().await?;
```

### 6. Validate Before Saving

```rust
// ✅ Good: Validate data before database operation
fn validate_username(username: &str) -> Result<()> {
    if username.len() < 3 || username.len() > 50 {
        return Err(ValidationError::InvalidLength);
    }
    Ok(())
}

validate_username(&req.username)?;

let user = users::ActiveModel {
    username: Set(req.username),
    // ...
};
```

### 7. Use Prelude for Imports

```rust
// models/prelude.rs
pub use super::users::Entity as Users;
pub use super::refresh_tokens::Entity as RefreshTokens;
pub use super::email_verifications::Entity as EmailVerifications;
pub use super::o_auth_accounts::Entity as OAuthAccounts;

// In other files
use crate::models::prelude::*;

let user = Users::find_by_id(id).one(db).await?;
```

### 8. Document Complex Fields

```rust
/// User account entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// Unique user identifier (UUID v4)
    /// Generated on creation, immutable
    pub id: Uuid,

    /// Argon2 password hash
    /// Optional because OAuth users may not have passwords
    /// Never expose this field in API responses
    pub password_hash: Option<String>,

    /// Timestamp when account was disabled by admin
    /// If set, user cannot authenticate regardless of credentials
    pub disabled_at: Option<DateTimeWithTimeZone>,
}
```

## Common Patterns

### Soft Delete Pattern

```rust
// Check if entity is "deleted" (disabled)
let active_users = Users::find()
    .filter(users::Column::DisabledAt.is_null())
    .all(db)
    .await?;

// "Delete" (disable) user
let mut user: users::ActiveModel = user.into();
user.disabled_at = Set(Some(Utc::now().into()));
user.update(db).await?;
```

### Audit Trail Pattern

```rust
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub created_by: Option<Uuid>,  // Who created this
    pub updated_by: Option<Uuid>,  // Who last updated this
}
```

### Versioning Pattern

```rust
pub struct Model {
    pub version: i32,  // Optimistic locking
}

// Update with version check
let result = Users::update_many()
    .filter(users::Column::Id.eq(user_id))
    .filter(users::Column::Version.eq(current_version))
    .col_expr(users::Column::Version, Expr::value(current_version + 1))
    .exec(db)
    .await?;

if result.rows_affected == 0 {
    return Err(ConcurrentModificationError);
}
```

## Related Documentation

- [Architecture](./architecture.md) - Overall architecture overview
- [Database](./database.md) - SeaORM setup and migrations
- [Services](./services.md) - Business logic using models
- [Testing](./testing.md) - Testing database operations
