//! Cobalt Stack Backend API Server.
//!
//! Full-featured REST API backend built with Axum, `SeaORM`, and `PostgreSQL`.
//! Provides JWT authentication, email verification, admin user management,
//! and comprehensive `OpenAPI` documentation.
//!
//! # Features
//!
//! - **JWT Authentication**: Secure token-based auth with refresh tokens
//! - **Email Verification**: Token-based email confirmation
//! - **Role-Based Access**: Admin and user role separation
//! - **Rate Limiting**: Login attempt protection via Valkey/Redis
//! - **Token Blacklist**: Immediate token revocation support
//! - **`OpenAPI` Documentation**: Interactive Swagger UI
//! - **Database Migrations**: `SeaORM` migration support
//! - **CORS Configuration**: Flexible cross-origin setup
//!
//! # Quick Start
//!
//! ```bash
//! # Set environment variables
//! cp .env.example .env
//! # Edit .env with your settings
//!
//! # Run migrations
//! sea-orm-cli migrate up
//!
//! # Seed admin user
//! cargo run --bin seed_admin
//!
//! # Start server
//! cargo run
//! ```
//!
//! # Environment Variables
//!
//! Required configuration via `.env` file:
//!
//! - `DATABASE_URL` - `PostgreSQL` connection string
//! - `JWT_SECRET` - Secret key for JWT signing
//! - `JWT_ACCESS_EXPIRY_MINUTES` - Access token lifetime (default: 30)
//! - `JWT_REFRESH_EXPIRY_DAYS` - Refresh token lifetime (default: 7)
//! - `PORT` - Server port (default: 3000)
//!
//! # API Endpoints
//!
//! ## Public Endpoints
//!
//! - `GET /health` - Health check
//! - `POST /api/v1/auth/register` - User registration
//! - `POST /api/v1/auth/login` - User login
//! - `POST /api/v1/auth/refresh` - Refresh access token
//! - `POST /api/v1/auth/verify-email` - Verify email address
//!
//! ## Protected Endpoints (Requires JWT)
//!
//! - `GET /api/v1/auth/me` - Get current user info
//! - `POST /api/v1/auth/logout` - Logout user
//! - `POST /api/v1/auth/send-verification` - Resend verification email
//!
//! ## Admin Endpoints (Requires Admin Role)
//!
//! - `GET /api/v1/admin/users` - List all users
//! - `GET /api/v1/admin/users/:id` - Get user details
//! - `PATCH /api/v1/admin/users/:id/disable` - Disable user account
//! - `PATCH /api/v1/admin/users/:id/enable` - Enable user account
//! - `GET /api/v1/admin/stats` - System statistics
//!
//! # Documentation
//!
//! Interactive API documentation available at:
//! - Swagger UI: <http://localhost:3000/swagger-ui>
//! - `OpenAPI` JSON: <http://localhost:3000/openapi.json>
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐
//! │   Handlers  │ ← HTTP layer (routes, extractors, responses)
//! ├─────────────┤
//! │ Middleware  │ ← Auth, CORS, logging
//! ├─────────────┤
//! │  Services   │ ← Business logic (auth, email, cache)
//! ├─────────────┤
//! │   Models    │ ← Database entities (SeaORM)
//! └─────────────┘
//! ```

mod config;
mod handlers;
mod middleware;
mod models;
mod openapi;
mod services;
mod utils;

use axum::{
    http::{header, HeaderValue, Method},
    middleware as axum_middleware,
    routing::{get, patch, post},
    Router,
};
use sea_orm::Database;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// API version prefix for all routes
const API_PREFIX: &str = "/api/v1";

/// Application entry point.
///
/// Initializes logging, database connection, and starts the Axum HTTP server.
/// Loads configuration from environment variables and `.env` file.
///
/// # Errors
///
/// Returns error if:
/// - `DATABASE_URL` environment variable is not set
/// - Database connection fails
/// - Server fails to bind to port
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cobalt_stack_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Generate OpenAPI schema for frontend
    if let Err(e) = openapi::write_openapi_schema() {
        tracing::warn!("Failed to write OpenAPI schema: {}", e);
    } else {
        tracing::info!("OpenAPI schema generated at openapi/schema.json");
    }

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")?;
    let db = Database::connect(&database_url).await?;
    tracing::info!("Database connected");

    // Initialize JWT config
    let jwt_config = services::auth::JwtConfig::from_env();

    // Create application state
    let state = handlers::auth::AppState {
        db: Arc::new(db),
        jwt_config: jwt_config.clone(),
    };

    // Build application router with state
    let app = create_app(state, jwt_config);

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting server on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Create the Axum router with all routes, middleware, and state.
///
/// Configures the complete application including:
/// - Public routes (register, login, refresh)
/// - Protected routes (profile, logout)
/// - Admin routes (user management)
/// - CORS middleware
/// - Swagger UI documentation
///
/// # Arguments
///
/// * `state` - Application state with database connection and JWT config
/// * `jwt_config` - JWT configuration for authentication middleware
///
/// # Returns
///
/// Fully configured Axum [`Router`] ready to serve HTTP requests.
///
/// # CORS Configuration
///
/// Allows requests from origins ending with `:2727` (frontend port) for development.
/// In production, configure specific allowed origins via environment variables.
#[allow(clippy::too_many_lines)]
fn create_app(state: handlers::auth::AppState, jwt_config: services::auth::JwtConfig) -> Router {
    // Configure CORS with credentials support

    // Get allowed origins from environment variable
    let allowed_origins = std::env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:2727,http://localhost:3001".to_string());

    let origins: Vec<HeaderValue> = allowed_origins
        .split(',')
        .filter_map(|origin| origin.trim().parse().ok())
        .collect();

    tracing::info!("CORS allowed origins: {:?}", origins);

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec![
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::COOKIE,
        ])
        .allow_credentials(true);

    // Auth routes (public)
    let auth_public_routes = Router::new()
        .route(
            &format!("{API_PREFIX}/auth/register"),
            post(handlers::auth::register),
        )
        .route(
            &format!("{API_PREFIX}/auth/login"),
            post(handlers::auth::login),
        )
        .route(
            &format!("{API_PREFIX}/auth/refresh"),
            post(handlers::auth::refresh_token),
        )
        .route(
            &format!("{API_PREFIX}/auth/verify-email"),
            post(handlers::auth::verify_email),
        )
        .with_state(state.clone());

    // Auth routes (protected)
    let auth_protected_routes = Router::new()
        .route(
            &format!("{API_PREFIX}/auth/me"),
            get(handlers::auth::get_current_user),
        )
        .route(
            &format!("{API_PREFIX}/auth/logout"),
            post(handlers::auth::logout),
        )
        .route(
            &format!("{API_PREFIX}/auth/send-verification"),
            post(handlers::auth::send_verification_email),
        )
        .layer(axum_middleware::from_fn_with_state(
            jwt_config.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(state.clone());

    // Admin routes (protected - requires admin role)
    let admin_state = handlers::admin::AdminState {
        db: state.db.clone(),
    };

    let admin_routes = Router::new()
        .route(
            &format!("{API_PREFIX}/admin/users"),
            get(handlers::admin::list_users),
        )
        .route(
            &format!("{API_PREFIX}/admin/users/:id"),
            get(handlers::admin::get_user),
        )
        .route(
            &format!("{API_PREFIX}/admin/users/:id/disable"),
            patch(handlers::admin::disable_user),
        )
        .route(
            &format!("{API_PREFIX}/admin/users/:id/enable"),
            patch(handlers::admin::enable_user),
        )
        .route(
            &format!("{API_PREFIX}/admin/stats"),
            get(handlers::admin::get_stats),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.db,
            middleware::admin::admin_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            jwt_config,
            middleware::auth::auth_middleware,
        ))
        .with_state(admin_state);

    // Build main router
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .merge(auth_public_routes)
        .merge(auth_protected_routes)
        .merge(admin_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", openapi::ApiDoc::openapi()))
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

// TODO: Add integration tests later
