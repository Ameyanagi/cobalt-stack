# Database and Migrations

## Table of Contents
- [Overview](#overview)
- [Database Setup](#database-setup)
- [SeaORM Configuration](#seaorm-configuration)
- [Creating Migrations](#creating-migrations)
- [Running Migrations](#running-migrations)
- [Query Patterns](#query-patterns)
- [Schema Best Practices](#schema-best-practices)

## Overview

The Cobalt Stack backend uses **PostgreSQL** as its primary database with **SeaORM** as the ORM. Database schema changes are managed through **migrations** to ensure version control and reproducibility across environments.

**Technology Stack:**
- **Database**: PostgreSQL 14+
- **ORM**: SeaORM (async Rust ORM)
- **Migrations**: sea-orm-migration
- **Connection Pool**: Built into SeaORM

## Database Setup

### Prerequisites

Install PostgreSQL:

```bash
# Ubuntu/Debian
sudo apt install postgresql postgresql-contrib

# macOS
brew install postgresql@14
brew services start postgresql@14

# Arch Linux
sudo pacman -S postgresql
sudo systemctl start postgresql
```

### Create Database

```bash
# Connect to PostgreSQL
sudo -u postgres psql

# Create database
CREATE DATABASE cobalt_stack;

# Create user (optional, for non-superuser access)
CREATE USER cobalt_user WITH PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE cobalt_stack TO cobalt_user;

# Exit psql
\q
```

### Environment Configuration

Create `.env` file in project root:

```env
# Database connection
DATABASE_URL=postgres://username:password@localhost/cobalt_stack

# JWT configuration
JWT_SECRET=your_secret_key_change_in_production
JWT_ACCESS_EXPIRY_MINUTES=30
JWT_REFRESH_EXPIRY_DAYS=7

# Server configuration
PORT=3000
```

**Security Note**: Never commit `.env` with real credentials. Use `.env.example` for templates.

## SeaORM Configuration

### Connecting to Database

**Location**: `main.rs`

```rust
use sea_orm::Database;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Connect to database
    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Database connected");

    // Use db throughout application via Arc<DatabaseConnection>
    let db = Arc::new(db);
}
```

### Connection Pool Configuration

```rust
use sea_orm::{ConnectOptions, Database};

let mut opt = ConnectOptions::new(database_url);
opt.max_connections(100)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(8))
    .acquire_timeout(Duration::from_secs(8))
    .idle_timeout(Duration::from_secs(8))
    .max_lifetime(Duration::from_secs(8))
    .sqlx_logging(true);

let db = Database::connect(opt).await?;
```

## Creating Migrations

### Migration Structure

Migrations live in `backend/migration/src/`:

```text
migration/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Migration list
    ├── main.rs                   # Migration CLI
    ├── m20250125_000001_create_auth_tables.rs
    └── m20250126_000002_add_email_verification.rs
```

### Creating a New Migration

```bash
cd backend/migration
cargo run -- generate MIGRATION_NAME
```

This creates a new migration file: `src/mYYYYMMDD_HHMMSS_MIGRATION_NAME.rs`

### Migration Template

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Apply the migration (forward)
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create tables, add columns, etc.
        manager
            .create_table(
                Table::create()
                    .table(MyTable::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MyTable::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                    )
                    .to_owned(),
            )
            .await
    }

    /// Rollback the migration (backward)
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Undo changes
        manager
            .drop_table(Table::drop().table(MyTable::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MyTable {
    Table,
    Id,
}
```

### Real Example: Create Auth Tables

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()".to_owned()),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Email)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::PasswordHash)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Users::EmailVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()".to_owned()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()".to_owned()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on username
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_username")
                    .table(Users::Table)
                    .col(Users::Username)
                    .to_owned(),
            )
            .await?;

        // Create refresh_tokens table
        manager
            .create_table(
                Table::create()
                    .table(RefreshTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RefreshTokens::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                    )
                    .col(
                        ColumnDef::new(RefreshTokens::UserId)
                            .uuid()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(RefreshTokens::TokenHash)
                            .string_len(255)
                            .not_null()
                            .unique_key()
                    )
                    .col(
                        ColumnDef::new(RefreshTokens::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(RefreshTokens::RevokedAt)
                            .timestamp_with_time_zone()
                            .null()
                    )
                    .col(
                        ColumnDef::new(RefreshTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()".to_owned())
                    )
                    // Foreign key constraint
                    .foreign_key(
                        &mut ForeignKey::create()
                            .name("fk_refresh_tokens_user_id")
                            .from(RefreshTokens::Table, RefreshTokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for performance
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_refresh_tokens_user_id")
                    .table(RefreshTokens::Table)
                    .col(RefreshTokens::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_refresh_tokens_expires_at")
                    .table(RefreshTokens::Table)
                    .col(RefreshTokens::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order (foreign key constraints)
        manager
            .drop_table(Table::drop().table(RefreshTokens::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    Email,
    PasswordHash,
    EmailVerified,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RefreshTokens {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    RevokedAt,
    CreatedAt,
}
```

### Adding Columns to Existing Tables

```rust
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .alter_table(
            Table::alter()
                .table(Users::Table)
                .add_column(
                    ColumnDef::new(Users::Role)
                        .string_len(20)
                        .not_null()
                        .default("user")
                )
                .to_owned(),
        )
        .await
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .alter_table(
            Table::alter()
                .table(Users::Table)
                .drop_column(Users::Role)
                .to_owned(),
        )
        .await
}
```

### Creating Enums

```rust
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Create ENUM type
    manager
        .create_type(
            Type::create()
                .as_enum(UserRole::Enum)
                .values([UserRole::User, UserRole::Admin])
                .to_owned(),
        )
        .await?;

    // Add column using the enum
    manager
        .alter_table(
            Table::alter()
                .table(Users::Table)
                .add_column(
                    ColumnDef::new(Users::Role)
                        .enumeration(UserRole::Enum, [UserRole::User, UserRole::Admin])
                        .not_null()
                        .default("user")
                )
                .to_owned(),
        )
        .await
}

#[derive(DeriveIden)]
enum UserRole {
    #[sea_orm(iden = "user_role")]
    Enum,
    #[sea_orm(iden = "user")]
    User,
    #[sea_orm(iden = "admin")]
    Admin,
}
```

## Running Migrations

### Apply Migrations

```bash
# Run all pending migrations
cd backend/migration
cargo run -- up

# Or from project root
cd backend && cargo run --bin migration -- up
```

### Rollback Migrations

```bash
# Rollback last migration
cd backend/migration
cargo run -- down

# Rollback specific number of migrations
cargo run -- down -n 2
```

### Check Migration Status

```bash
# Show migration status
cargo run -- status
```

### Fresh Database

```bash
# Reset database (down all, then up all)
cargo run -- fresh

# Drop all tables and re-run all migrations
cargo run -- reset
```

### Register Migrations

**Location**: `migration/src/lib.rs`

```rust
use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250125_000001_create_auth_tables::Migration),
            Box::new(m20250126_000002_add_email_verification::Migration),
            // Add new migrations here
        ]
    }
}
```

## Query Patterns

### Basic CRUD Operations

#### Create (Insert)

```rust
use sea_orm::{ActiveModelTrait, Set};

let user = users::ActiveModel {
    id: Set(Uuid::new_v4()),
    username: Set("alice".to_string()),
    email: Set("alice@example.com".to_string()),
    created_at: Set(Utc::now().into()),
    updated_at: Set(Utc::now().into()),
    ..Default::default()
};

let user: users::Model = user.insert(db).await?;
```

#### Read (Select)

```rust
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

// Find by ID
let user = Users::find_by_id(user_id).one(db).await?;

// Find by column
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

#### Update

```rust
// Find and update
let user = Users::find_by_id(user_id).one(db).await?;
let mut user: users::ActiveModel = user.into();

user.email_verified = Set(true);
user.updated_at = Set(Utc::now().into());

let user = user.update(db).await?;
```

#### Delete

```rust
// Delete by ID
Users::delete_by_id(user_id).exec(db).await?;

// Delete with filter
Users::delete_many()
    .filter(users::Column::EmailVerified.eq(false))
    .filter(users::Column::CreatedAt.lt(thirty_days_ago))
    .exec(db)
    .await?;
```

### Advanced Queries

#### Joins and Relations

```rust
// Load related entities
let user_with_tokens = Users::find_by_id(user_id)
    .find_with_related(RefreshTokens)
    .all(db)
    .await?;

// Query with joins
let active_tokens = RefreshTokens::find()
    .inner_join(Users)
    .filter(users::Column::DisabledAt.is_null())
    .filter(refresh_tokens::Column::RevokedAt.is_null())
    .all(db)
    .await?;
```

#### Aggregations

```rust
use sea_orm::{QuerySelect, QueryOrder};

// Count
let user_count = Users::find().count(db).await?;

// Group by and count
let stats = Users::find()
    .select_only()
    .column(users::Column::Role)
    .column_as(users::Column::Id.count(), "count")
    .group_by(users::Column::Role)
    .into_model::<RoleStats>()
    .all(db)
    .await?;
```

#### Pagination

```rust
use sea_orm::{QuerySelect, PaginatorTrait};

let page = 1;
let items_per_page = 20;

let users = Users::find()
    .order_by_desc(users::Column::CreatedAt)
    .paginate(db, items_per_page);

let total_pages = users.num_pages().await?;
let users = users.fetch_page(page).await?;
```

### Transactions

```rust
use sea_orm::TransactionTrait;

let txn = db.begin().await?;

// Multiple operations in transaction
let user = users::ActiveModel { /* ... */ };
let user = user.insert(&txn).await?;

let token = refresh_tokens::ActiveModel {
    user_id: Set(user.id),
    // ...
};
token.insert(&txn).await?;

// Commit or rollback
txn.commit().await?;
// txn.rollback().await?;
```

### Raw SQL (When Necessary)

```rust
use sea_orm::{Statement, DatabaseBackend};

let sql = Statement::from_string(
    DatabaseBackend::Postgres,
    "SELECT * FROM users WHERE username LIKE $1".to_string(),
);

let result: Vec<users::Model> = users::Entity::find()
    .from_raw_sql(sql)
    .bind("alice%")
    .all(db)
    .await?;
```

## Schema Best Practices

### 1. Use Appropriate Column Types

```sql
-- ✅ Good: Use specific types
id              UUID PRIMARY KEY DEFAULT gen_random_uuid()
email           VARCHAR(255) UNIQUE NOT NULL
email_verified  BOOLEAN DEFAULT FALSE
created_at      TIMESTAMPTZ DEFAULT NOW()
role            user_role NOT NULL

-- ❌ Bad: Generic types
id              TEXT
email           TEXT
email_verified  INTEGER
created_at      TEXT
role            TEXT
```

### 2. Always Use Indexes

```rust
// Index frequently queried columns
manager.create_index(
    Index::create()
        .name("idx_users_email")
        .table(Users::Table)
        .col(Users::Email)
        .to_owned(),
).await?;

// Composite indexes for multi-column queries
manager.create_index(
    Index::create()
        .name("idx_tokens_user_expires")
        .table(RefreshTokens::Table)
        .col(RefreshTokens::UserId)
        .col(RefreshTokens::ExpiresAt)
        .to_owned(),
).await?;
```

### 3. Use Foreign Keys with Cascade

```rust
.foreign_key(
    &mut ForeignKey::create()
        .name("fk_refresh_tokens_user_id")
        .from(RefreshTokens::Table, RefreshTokens::UserId)
        .to(Users::Table, Users::Id)
        .on_delete(ForeignKeyAction::Cascade)  // Delete tokens when user is deleted
        .on_update(ForeignKeyAction::Cascade)  // Update tokens if user ID changes
)
```

### 4. Use Timestamps

```rust
.col(
    ColumnDef::new(Users::CreatedAt)
        .timestamp_with_time_zone()
        .not_null()
        .extra("DEFAULT NOW()".to_owned())
)
.col(
    ColumnDef::new(Users::UpdatedAt)
        .timestamp_with_time_zone()
        .not_null()
        .extra("DEFAULT NOW()".to_owned())
)
```

### 5. Use Constraints

```rust
// Unique constraints
.col(
    ColumnDef::new(Users::Email)
        .string_len(255)
        .not_null()
        .unique_key()
)

// Check constraints
.check(Expr::col(Users::Username).is_not_null())

// Not null constraints
.col(
    ColumnDef::new(Users::Email)
        .string_len(255)
        .not_null()
)
```

### 6. Plan for Growth

- Use UUIDs for distributed systems
- Index columns used in WHERE, JOIN, ORDER BY
- Consider partitioning for large tables
- Use appropriate data types (don't over-size VARCHAR)

### 7. Version Your Schema

- One migration per logical change
- Never edit existing migrations (create new ones)
- Write both `up` and `down` functions
- Test migrations on staging before production

## Generating Entities from Database

After creating migrations, generate SeaORM entities:

```bash
# Generate entities from database
sea-orm-cli generate entity \
    -u postgres://username:password@localhost/cobalt_stack \
    -o backend/src/models

# This creates:
# - src/models/users.rs
# - src/models/refresh_tokens.rs
# - src/models/prelude.rs
# - src/models/mod.rs
```

## Related Documentation

- [Architecture](./architecture.md) - Overall backend architecture
- [Models](./models.md) - Working with generated entities
- [Services](./services.md) - Using database in business logic
- [Testing](./testing.md) - Testing database operations
