# Database Guide

Complete guide to setting up and working with the database in Cobalt Stack.

## Table of Contents

- [Overview](#overview)
- [Setting Up Database](#setting-up-database)
- [Running Migrations](#running-migrations)
- [Database Schema](#database-schema)
- [Database Operations](#database-operations)
- [SeaORM Basics](#seaorm-basics)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

Cobalt Stack uses:

- **PostgreSQL**: Robust, production-ready relational database
- **SeaORM**: Async ORM for Rust with type-safe queries
- **SeaORM Migration**: Schema migration management
- **Connection Pooling**: Efficient database connection management

### Architecture

```
Application
    ↓
SeaORM (ORM Layer)
    ↓
PostgreSQL Connection Pool
    ↓
PostgreSQL Database
```

## Setting Up Database

### Using Docker Compose (Recommended)

The easiest way to set up PostgreSQL is with Docker Compose:

```bash
# Start PostgreSQL container
docker-compose up -d postgres

# Verify it's running
docker-compose ps
```

Configuration in `docker-compose.yml`:

```yaml
postgres:
  image: postgres:16-alpine
  environment:
    POSTGRES_DB: ${POSTGRES_DB:-cobalt_dev}
    POSTGRES_USER: ${POSTGRES_USER:-postgres}
    POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-postgres}
  ports:
    - "${POSTGRES_PORT:-2800}:5432"
  volumes:
    - postgres_data:/var/lib/postgresql/data
```

### Environment Configuration

Configure database connection in `backend/.env`:

```bash
# Database connection URL
DATABASE_URL=postgresql://postgres:postgres@localhost:2800/cobalt_dev

# Or use individual components
POSTGRES_HOST=localhost
POSTGRES_PORT=2800
POSTGRES_DB=cobalt_dev
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
```

### Manual PostgreSQL Installation

If not using Docker, install PostgreSQL manually:

**macOS** (Homebrew):
```bash
brew install postgresql@16
brew services start postgresql@16
```

**Ubuntu/Debian**:
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Create Database**:
```bash
createdb cobalt_dev
```

### Valkey/Redis for Caching

Cobalt Stack uses Valkey (Redis) for caching and sessions:

```bash
# Start with Docker Compose
docker-compose up -d valkey

# Or install manually
# macOS
brew install valkey

# Ubuntu
sudo apt install redis-server
```

Configure in `backend/.env`:
```bash
VALKEY_URL=redis://localhost:2900
```

## Running Migrations

### Install SeaORM CLI

```bash
cargo install sea-orm-cli
```

### Run Migrations

```bash
# From project root
make migrate

# Or from backend directory
cd backend
sea-orm-cli migrate up
```

### Check Migration Status

```bash
cd backend
sea-orm-cli migrate status
```

Output:
```
Checking migration status
Applying migration 'm20250101_000001_create_initial_schema'
Applying migration 'm20250125_000001_create_auth_tables'
Applying migration 'm20250126_000001_add_email_verification_and_roles'
All migrations applied successfully
```

### Rollback Migrations

```bash
# Rollback last migration
sea-orm-cli migrate down

# Rollback specific number of migrations
sea-orm-cli migrate down -n 2
```

### Fresh Migration (Reset Database)

```bash
# Drop all tables and re-run migrations
sea-orm-cli migrate fresh

# Or manually
sea-orm-cli migrate down -n 999
sea-orm-cli migrate up
```

### Creating New Migrations

```bash
cd backend
sea-orm-cli migrate generate add_new_feature
```

This creates a new migration file in `backend/migration/src/`.

Example migration:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Post::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Post::Title).string().not_null())
                    .col(ColumnDef::new(Post::Content).text().not_null())
                    .col(
                        ColumnDef::new(Post::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Post {
    Table,
    Id,
    Title,
    Content,
    CreatedAt,
}
```

## Database Schema

### Users Table

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'user',
    email_verified BOOLEAN DEFAULT FALSE,
    disabled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TYPE user_role AS ENUM ('user', 'admin');
```

### Refresh Tokens Table

```sql
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
```

### Email Verifications Table

```sql
CREATE TABLE email_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_verifications_user_id ON email_verifications(user_id);
CREATE INDEX idx_email_verifications_token_hash ON email_verifications(token_hash);
```

### Viewing Schema

```bash
# Connect to database
psql -h localhost -p 2800 -U postgres -d cobalt_dev

# List tables
\dt

# Describe table
\d users

# View table data
SELECT * FROM users LIMIT 10;
```

## Database Operations

### Connecting to Database

```rust
use sea_orm::{Database, DatabaseConnection};

// Connect using DATABASE_URL
async fn connect() -> Result<DatabaseConnection, DbErr> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let db = Database::connect(&database_url).await?;
    Ok(db)
}
```

### Creating Records

```rust
use crate::models::{users, prelude::*};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

// Create new user
async fn create_user(
    db: &DatabaseConnection,
    username: String,
    email: String,
    password_hash: String,
) -> Result<users::Model, DbErr> {
    let user = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(username),
        email: Set(email),
        password_hash: Set(password_hash),
        role: Set(UserRole::User),
        email_verified: Set(false),
        disabled_at: Set(None),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    let user = user.insert(db).await?;
    Ok(user)
}
```

### Reading Records

```rust
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

// Find user by ID
async fn find_user_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<users::Model>, DbErr> {
    Users::find_by_id(id).one(db).await
}

// Find user by username
async fn find_user_by_username(
    db: &DatabaseConnection,
    username: &str,
) -> Result<Option<users::Model>, DbErr> {
    Users::find()
        .filter(users::Column::Username.eq(username))
        .one(db)
        .await
}

// Find all active users
async fn find_active_users(
    db: &DatabaseConnection,
) -> Result<Vec<users::Model>, DbErr> {
    Users::find()
        .filter(users::Column::DisabledAt.is_null())
        .all(db)
        .await
}
```

### Updating Records

```rust
// Update user
async fn update_user_email(
    db: &DatabaseConnection,
    user_id: Uuid,
    new_email: String,
) -> Result<users::Model, DbErr> {
    let user = Users::find_by_id(user_id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

    let mut active_user: users::ActiveModel = user.into();
    active_user.email = Set(new_email);
    active_user.updated_at = Set(Utc::now().into());

    active_user.update(db).await
}
```

### Deleting Records

```rust
// Soft delete (disable user)
async fn disable_user(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<(), DbErr> {
    let user = Users::find_by_id(user_id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

    let mut active_user: users::ActiveModel = user.into();
    active_user.disabled_at = Set(Some(Utc::now().into()));
    active_user.update(db).await?;

    Ok(())
}

// Hard delete
async fn delete_user(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<(), DbErr> {
    Users::delete_by_id(user_id).exec(db).await?;
    Ok(())
}
```

## SeaORM Basics

### Entities

Entities represent database tables:

```rust
// Generated from database schema
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub email_verified: bool,
    pub disabled_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::refresh_tokens::Entity")]
    RefreshTokens,
}

impl Related<super::refresh_tokens::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RefreshTokens.def()
    }
}
```

### Queries

```rust
use sea_orm::{QueryFilter, QueryOrder, QuerySelect, PaginatorTrait};

// Filter and sort
let users = Users::find()
    .filter(users::Column::EmailVerified.eq(true))
    .order_by_asc(users::Column::CreatedAt)
    .all(db)
    .await?;

// Pagination
let paginator = Users::find()
    .paginate(db, 20); // 20 items per page

let users = paginator.fetch_page(0).await?; // First page
let total = paginator.num_pages().await?;

// Select specific columns
let usernames: Vec<String> = Users::find()
    .select_only()
    .column(users::Column::Username)
    .into_tuple()
    .all(db)
    .await?;

// Count
let count = Users::find()
    .filter(users::Column::Role.eq(UserRole::Admin))
    .count(db)
    .await?;
```

### Transactions

```rust
use sea_orm::TransactionTrait;

// Execute operations in a transaction
async fn transfer_operation(db: &DatabaseConnection) -> Result<(), DbErr> {
    let txn = db.begin().await?;

    // Perform multiple operations
    let user = create_user(&txn, "alice", "alice@example.com", "hash").await?;
    create_verification_token(&txn, user.id).await?;

    // Commit transaction
    txn.commit().await?;

    Ok(())
}

// Transaction with closure (auto-rollback on error)
async fn transfer_with_closure(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.transaction::<_, (), DbErr>(|txn| {
        Box::pin(async move {
            let user = create_user(txn, "bob", "bob@example.com", "hash").await?;
            create_verification_token(txn, user.id).await?;
            Ok(())
        })
    })
    .await
}
```

### Relationships

```rust
// Load related data
use sea_orm::ModelTrait;

let user = Users::find_by_id(user_id)
    .one(db)
    .await?
    .unwrap();

// Load user's refresh tokens
let tokens = user.find_related(RefreshTokens).all(db).await?;

// Eager loading with join
let users_with_tokens = Users::find()
    .find_with_related(RefreshTokens)
    .all(db)
    .await?;
```

## Best Practices

### 1. Use Transactions for Multiple Operations

```rust
// Good: All operations succeed or fail together
db.transaction(|txn| {
    Box::pin(async move {
        create_user(txn, data).await?;
        create_profile(txn, user_id).await?;
        Ok(())
    })
}).await?;

// Bad: Partial success can leave inconsistent state
create_user(db, data).await?;
create_profile(db, user_id).await?; // If this fails, user exists without profile
```

### 2. Use Connection Pooling

```rust
// Good: Reuse database connection pool
let db = Database::connect(&database_url).await?;
let app_state = Arc::new(AppState { db });

// Bad: Create new connection for each request (slow and wasteful)
```

### 3. Implement Soft Deletes

```rust
// Good: Soft delete preserves data and relationships
user.disabled_at = Some(Utc::now());

// Use when: User data should be preserved for auditing
// Avoid when: Need to free up username/email for re-registration
```

### 4. Use Prepared Statements

SeaORM automatically uses prepared statements, preventing SQL injection:

```rust
// Safe: Parameterized query
Users::find()
    .filter(users::Column::Username.eq(username))
    .one(db)
    .await?;

// Never do this (vulnerable to SQL injection):
// db.execute_raw(&format!("SELECT * FROM users WHERE username = '{}'", username))
```

### 5. Index Foreign Keys and Frequently Queried Columns

```sql
-- Good: Index on foreign key
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);

-- Good: Index on frequently searched column
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
```

### 6. Handle Unique Constraint Violations

```rust
match user.insert(db).await {
    Ok(user) => Ok(user),
    Err(DbErr::Exec(err)) if err.to_string().contains("duplicate key") => {
        Err(anyhow::anyhow!("Username or email already exists"))
    }
    Err(e) => Err(e.into()),
}
```

### 7. Use Timestamps

```rust
// Always track creation and modification times
pub created_at: DateTimeWithTimeZone,
pub updated_at: DateTimeWithTimeZone,

// Update timestamp on modification
active_model.updated_at = Set(Utc::now().into());
```

## Troubleshooting

### Connection Failed

**Problem**: "error connecting to server: Connection refused"

**Solutions**:
1. Check PostgreSQL is running: `docker-compose ps`
2. Verify port is correct in DATABASE_URL
3. Check firewall isn't blocking connection
4. Test connection: `psql -h localhost -p 2800 -U postgres`
5. Check PostgreSQL logs: `docker-compose logs postgres`

### Migration Failed

**Problem**: Migration fails with error

**Solutions**:
1. Check migration SQL syntax
2. Verify foreign key references exist
3. Check for existing tables (use IF NOT EXISTS)
4. Review migration order in `lib.rs`
5. Check database logs for detailed error

### "relation does not exist"

**Problem**: Query fails with "relation users does not exist"

**Solutions**:
1. Run migrations: `make migrate`
2. Check you're connected to correct database
3. Verify table name matches migration
4. Check schema: `\dt` in psql

### Deadlock Detected

**Problem**: "deadlock detected" error

**Solutions**:
1. Review transaction ordering
2. Keep transactions short
3. Lock rows in consistent order
4. Use `FOR UPDATE` appropriately
5. Consider optimistic locking

### Slow Queries

**Problem**: Database queries taking too long

**Solutions**:
1. Add indexes on filtered/joined columns
2. Use EXPLAIN ANALYZE to analyze queries
3. Optimize N+1 query problems with eager loading
4. Consider pagination for large result sets
5. Check connection pool size

### Connection Pool Exhausted

**Problem**: "timeout waiting for connection from pool"

**Solutions**:
1. Increase pool size in database URL: `?max_connections=20`
2. Review long-running queries
3. Ensure connections are properly closed
4. Use connection pooling correctly
5. Check for connection leaks

## Related Documentation

- [Authentication Guide](./authentication.md) - User authentication and JWT
- [Email Verification Guide](./email-verification.md) - Email verification flow
- [Admin Dashboard Guide](./admin-dashboard.md) - User management
- [Testing Guide](./testing.md) - Database testing strategies
- [Backend Architecture](../backend/README.md) - Backend structure and services
