//! Cobalt Stack Backend API Library
//!
//! This library provides the core functionality for the Cobalt Stack authentication
//! and authorization system. It follows a layered architecture with clear separation
//! of concerns between HTTP handlers, business logic services, and data models.
//!
//! # Architecture
//!
//! The codebase is organized into the following layers:
//!
//! - **Handlers**: HTTP request/response handling and routing
//! - **Services**: Business logic and domain operations
//! - **Models**: Database entities and domain models (SeaORM)
//! - **Middleware**: Cross-cutting concerns (authentication, authorization)
//! - **Config**: Application configuration management
//! - **Utils**: Shared utilities and helpers
//!
//! # Key Features
//!
//! - JWT-based authentication with access and refresh tokens
//! - Secure token rotation to prevent token theft
//! - Email verification workflow
//! - Role-based access control (RBAC)
//! - Rate limiting and token blacklisting
//! - PostgreSQL database with SeaORM
//! - Valkey/Redis for caching and session management
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack::{handlers, services, models};
//! use axum::Router;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create application with configured routes
//! let app = Router::new()
//!     .merge(handlers::auth::routes());
//! # Ok(())
//! # }
//! ```
//!
//! # Security
//!
//! This library implements security best practices:
//! - HttpOnly cookies for refresh tokens
//! - Secure password hashing with Argon2
//! - JWT signature verification
//! - Token rotation and revocation
//! - Rate limiting on authentication endpoints

pub mod config;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;
pub mod utils;
