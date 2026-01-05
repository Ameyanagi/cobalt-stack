//! Admin user seeding utility.
//!
//! This binary creates an initial admin user in the database for bootstrapping
//! the application. It's designed to be run once during initial setup or when
//! resetting the admin account.
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin seed_admin
//! ```
//!
//! # Credentials
//!
//! Creates an admin user with:
//! - **Email**: admin@example.com
//! - **Username**: admin
//! - **Password**: admin123
//! - **Role**: Admin
//! - **Email Verified**: true (auto-verified)
//!
//! # Security Warning
//!
//! **IMPORTANT**: Change the default password immediately after first login!
//! The default credentials are intentionally simple for initial setup but
//! should never be used in production.
//!
//! # Idempotency
//!
//! Safe to run multiple times - checks if admin exists before creating.
//! If admin@example.com already exists, the script exits without changes.
//!
//! # Environment Variables
//!
//! Requires `DATABASE_URL` to be set:
//! ```bash
//! DATABASE_URL=postgres://user:pass@localhost/cobalt_stack
//! ```
//!
//! # Examples
//!
//! ```bash
//! # First time setup
//! cargo run --bin seed_admin
//! # Output: ✅ Admin user created successfully!
//!
//! # Running again (idempotent)
//! cargo run --bin seed_admin
//! # Output: ⚠️  Admin user already exists
//! ```

use cobalt_stack_backend::models::{prelude::*, sea_orm_active_enums::UserRole, users};
use cobalt_stack_backend::services::auth::hash_password;
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")?;
    let db = Database::connect(&database_url).await?;

    println!("🌱 Seeding admin user...");

    // Check if admin already exists
    let existing_admin = Users::find()
        .filter(users::Column::Email.eq("admin@example.com"))
        .one(&db)
        .await?;

    if existing_admin.is_some() {
        println!("⚠️  Admin user already exists with email: admin@example.com");
        return Ok(());
    }

    // Create admin user
    let password_hash =
        hash_password("admin123").map_err(|e| format!("Failed to hash password: {e}"))?;

    let admin = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set("admin".to_string()),
        email: Set("admin@example.com".to_string()),
        password_hash: Set(Some(password_hash)),
        role: Set(UserRole::Admin),
        email_verified: Set(true), // Auto-verify admin email
        disabled_at: Set(None),
        last_login_at: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };

    let inserted_admin = admin.insert(&db).await?;

    println!("✅ Admin user created successfully!");
    println!("📧 Email: admin@example.com");
    println!("🔑 Password: admin123");
    println!("🆔 User ID: {}", inserted_admin.id);
    println!("\n⚠️  IMPORTANT: Change the default password after first login!");

    Ok(())
}
