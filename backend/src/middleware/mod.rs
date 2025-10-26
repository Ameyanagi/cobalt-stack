//! Axum middleware modules for authentication and authorization.
//!
//! This module provides middleware layers for securing API endpoints
//! with JWT-based authentication and role-based authorization.
//!
//! # Modules
//!
//! - **auth**: JWT authentication middleware that validates tokens
//! - **admin**: Role-based authorization middleware for admin-only endpoints
//!
//! # Middleware Chain
//!
//! Middleware is applied in layers with specific ordering requirements:
//!
//! 1. **auth_middleware** - First layer: validates JWT token, injects AuthUser
//! 2. **admin_middleware** - Second layer: checks admin role (requires auth_middleware)
//!
//! # Usage Example
//!
//! ```no_run
//! use axum::{Router, routing::get, middleware};
//! use cobalt_stack_backend::middleware::{auth::auth_middleware, admin::admin_middleware};
//! use cobalt_stack_backend::services::auth::JwtConfig;
//! use sea_orm::DatabaseConnection;
//! use std::sync::Arc;
//!
//! # async fn example(db: Arc<DatabaseConnection>) {
//! let jwt_config = JwtConfig::from_env();
//!
//! // Protected routes (authenticated users only)
//! let protected_routes = Router::new()
//!     .route("/profile", get(get_profile))
//!     .layer(middleware::from_fn_with_state(
//!         jwt_config.clone(),
//!         auth_middleware
//!     ));
//!
//! // Admin routes (admin users only)
//! let admin_routes = Router::new()
//!     .route("/admin/users", get(list_users))
//!     .layer(middleware::from_fn_with_state(db, admin_middleware))
//!     .layer(middleware::from_fn_with_state(jwt_config, auth_middleware));
//! # }
//! # async fn get_profile() -> &'static str { "Profile" }
//! # async fn list_users() -> &'static str { "Users" }
//! ```

pub mod admin;
pub mod auth;
