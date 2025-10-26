//! Business logic and service layer modules.
//!
//! This module organizes domain services that implement core business logic
//! separate from HTTP handlers. Services handle authentication, email delivery,
//! caching, and other infrastructure concerns.
//!
//! # Architecture
//!
//! Services follow Domain-Driven Design principles:
//! - **Domain Logic**: Core business rules and workflows
//! - **Infrastructure**: External integrations (database, cache, email)
//! - **Error Handling**: Domain-specific error types
//! - **Testability**: Pure functions and mockable dependencies
//!
//! # Modules
//!
//! - **auth**: Authentication services (JWT, passwords, token rotation)
//! - **email**: Email delivery services (verification emails)
//! - **valkey**: Valkey/Redis caching services (blacklist, rate limiting)
//!
//! # Service Layer Benefits
//!
//! - **Reusability**: Services can be called from multiple handlers
//! - **Testability**: Business logic testable without HTTP layer
//! - **Maintainability**: Changes to business rules isolated from HTTP concerns
//! - **Domain Clarity**: Service names express business intent

pub mod auth;
pub mod email;
pub mod valkey;
