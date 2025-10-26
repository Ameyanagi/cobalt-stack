# Rust Documentation Standards

**Change ID**: improve-documentation
**Purpose**: Define comprehensive standards for Rust doc strings following RFC 1574 and standard library conventions

## Overview

This document establishes the documentation standards for the Cobalt Stack backend codebase. All public APIs must have comprehensive documentation that enables `cargo doc` to generate complete, browsable API documentation.

## Module-Level Documentation

Use `//!` at the top of each `.rs` file for module-level documentation.

### Structure
```rust
//! Brief one-line description of the module.
//!
//! More detailed explanation of the module's purpose, responsibilities,
//! and how it fits into the larger architecture.
//!
//! # Examples
//!
//! ```
//! // Example usage of the module
//! use cobalt_stack::module_name;
//! ```
//!
//! # Architecture
//!
//! Explain the module's design patterns, layer responsibilities (handler/service/model),
//! and any important architectural decisions.
```

### Example
```rust
//! Authentication service layer.
//!
//! This module provides business logic for user authentication, including
//! JWT token generation, refresh token management, and email verification.
//! It follows Domain-Driven Design principles with clear separation between
//! the service layer (business logic) and handler layer (HTTP concerns).
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack::services::auth::AuthService;
//!
//! let service = AuthService::new(db_pool);
//! let result = service.authenticate(credentials).await?;
//! ```
//!
//! # Architecture
//!
//! The service layer:
//! - Validates business rules
//! - Manages domain logic
//! - Coordinates database operations
//! - Returns domain errors (not HTTP errors)
```

## Function Documentation

Use `///` for all public functions and methods.

### Structure
```rust
/// Brief one-line description of what the function does.
///
/// More detailed explanation if needed, including edge cases,
/// side effects, and important behaviors.
///
/// # Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
///
/// Description of what the function returns and when.
///
/// # Errors
///
/// This function will return an error if:
/// - Condition 1 happens
/// - Condition 2 occurs
///
/// # Examples
///
/// ```
/// let result = function_name(arg1, arg2);
/// assert_eq!(result, expected);
/// ```
///
/// # Panics
///
/// This function panics if... (only if applicable)
///
/// # Safety
///
/// This function is unsafe because... (only for unsafe functions)
```

### Example
```rust
/// Authenticates a user with email and password credentials.
///
/// Validates the credentials against the database, checks if the user's
/// email is verified, and generates JWT access and refresh tokens.
///
/// # Arguments
///
/// * `credentials` - User login credentials containing email and password
/// * `db` - Database connection pool for user lookup
///
/// # Returns
///
/// Returns `AuthResponse` containing JWT tokens and user information on success.
///
/// # Errors
///
/// This function will return an error if:
/// - User with the email doesn't exist (`AuthError::InvalidCredentials`)
/// - Password is incorrect (`AuthError::InvalidCredentials`)
/// - Email is not verified (`AuthError::EmailNotVerified`)
/// - Database query fails (`AuthError::DatabaseError`)
///
/// # Examples
///
/// ```no_run
/// use cobalt_stack::services::auth::AuthService;
/// use cobalt_stack::models::auth::LoginCredentials;
///
/// let credentials = LoginCredentials {
///     email: "user@example.com".to_string(),
///     password: "secure_password".to_string(),
/// };
///
/// let response = auth_service.authenticate(credentials, &db).await?;
/// println!("Access token: {}", response.access_token);
/// ```
pub async fn authenticate(
    &self,
    credentials: LoginCredentials,
    db: &DatabaseConnection,
) -> Result<AuthResponse, AuthError> {
    // Implementation
}
```

## Struct and Enum Documentation

### Struct Documentation
```rust
/// Brief description of the struct.
///
/// Detailed explanation of what this struct represents,
/// its role in the domain model, and usage guidelines.
///
/// # Examples
///
/// ```
/// use cobalt_stack::models::User;
///
/// let user = User {
///     id: 1,
///     email: "user@example.com".to_string(),
///     // ...
/// };
/// ```
pub struct StructName {
    /// Description of this field and its purpose.
    pub field1: Type1,

    /// Description of this field.
    /// Can be multiple lines if needed.
    pub field2: Type2,
}
```

### Enum Documentation
```rust
/// Brief description of what this enum represents.
///
/// Detailed explanation of the enum's purpose and when to use each variant.
///
/// # Examples
///
/// ```
/// use cobalt_stack::errors::AuthError;
///
/// match result {
///     Err(AuthError::InvalidCredentials) => println!("Bad login"),
///     Err(AuthError::EmailNotVerified) => println!("Check email"),
///     Ok(response) => println!("Success!"),
/// }
/// ```
pub enum EnumName {
    /// Description of when this variant is used.
    Variant1,

    /// Description of this variant.
    ///
    /// Additional details about the wrapped type if applicable.
    Variant2(Type),

    /// Description of this variant with named fields.
    Variant3 {
        /// Field documentation
        field1: Type1,
        /// Field documentation
        field2: Type2,
    },
}
```

### Example
```rust
/// Authentication error types.
///
/// Represents all possible errors that can occur during authentication,
/// authorization, and token management operations.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// User credentials are invalid (wrong email or password).
    #[error("Invalid email or password")]
    InvalidCredentials,

    /// User's email address has not been verified.
    ///
    /// The user must verify their email before accessing protected resources.
    #[error("Email not verified")]
    EmailNotVerified,

    /// JWT token is invalid or expired.
    #[error("Invalid or expired token")]
    InvalidToken,

    /// Database operation failed.
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
}
```

## Trait Documentation

```rust
/// Brief description of what this trait provides.
///
/// Detailed explanation of the trait's purpose, when to implement it,
/// and any important contracts or invariants.
///
/// # Examples
///
/// ```
/// use cobalt_stack::traits::Repository;
///
/// struct MyRepository;
///
/// impl Repository for MyRepository {
///     // implementation
/// }
/// ```
pub trait TraitName {
    /// Associated type documentation.
    type AssociatedType;

    /// Method documentation following function documentation rules.
    fn method_name(&self) -> Result<(), Error>;
}
```

## Documentation Best Practices

### 1. Write for the Reader
- Assume the reader knows Rust but not your codebase
- Explain the "why" not just the "what"
- Include context about architectural decisions
- Reference related functions/modules with `[function_name]`

### 2. Use Examples Liberally
- Include runnable examples whenever possible
- Use `no_run` for examples that need external resources
- Use `ignore` for pseudo-code examples
- Examples should compile and demonstrate real usage

### 3. Link Related Items
```rust
/// Authenticates a user.
///
/// See also:
/// - [`register_user`] for creating new accounts
/// - [`verify_email`] for email verification
/// - [`AuthResponse`] for the return type structure
```

### 4. Document Error Conditions
Always document:
- When functions return errors
- What error variants can occur
- How to handle or recover from errors

### 5. Document Panics
If a function can panic, document when and why:
```rust
/// # Panics
///
/// Panics if the user ID is 0 or negative.
```

### 6. Safety Documentation
For `unsafe` code, explain:
- Why the code is unsafe
- What invariants must be upheld
- What the caller must guarantee

### 7. Code Examples Syntax
```rust
// Runnable example
/// # Examples
/// ```
/// let result = function();
/// ```

// Example that needs external resources (database, network)
/// # Examples
/// ```no_run
/// let db = setup_database().await;
/// ```

// Pseudo-code or incomplete example
/// # Examples
/// ```ignore
/// // Conceptual example
/// ```

// Example with compilation errors (generally avoid)
/// # Examples
/// ```compile_fail
/// let x: u32 = "string"; // This won't compile
/// ```
```

## Domain-Specific Guidelines

### Handler Layer
Document:
- HTTP-specific concerns (status codes, headers)
- Request/response transformations
- Middleware interactions
- Error mapping to HTTP responses

```rust
/// HTTP handler for user login.
///
/// Accepts JSON credentials, validates them, and returns JWT tokens.
///
/// # Request
/// ```json
/// {
///   "email": "user@example.com",
///   "password": "secure_password"
/// }
/// ```
///
/// # Response
/// - `200 OK`: Authentication successful with JWT tokens
/// - `401 Unauthorized`: Invalid credentials
/// - `403 Forbidden`: Email not verified
/// - `500 Internal Server Error`: Server error
```

### Service Layer
Document:
- Business logic and rules
- Domain operations
- Transaction boundaries
- Domain error conditions

```rust
/// Validates and creates a new user account.
///
/// Business rules:
/// - Email must be unique
/// - Password must meet complexity requirements
/// - Email verification is sent automatically
/// - User is created as inactive until verified
```

### Model Layer
Document:
- Domain entity relationships
- Database mapping details
- Validation rules
- Lifecycle considerations

```rust
/// User entity representing an authenticated user.
///
/// # Database Mapping
/// - Table: `users`
/// - Primary Key: `id` (auto-increment)
/// - Indexes: Unique on `email`
///
/// # Lifecycle
/// 1. Created with `is_verified = false`
/// 2. Email verification sets `is_verified = true`
/// 3. Can be disabled by admin (soft delete)
```

## Validation Checklist

Before marking documentation as complete:

- [ ] All public modules have `//!` documentation
- [ ] All public functions have `///` documentation
- [ ] All public structs and fields are documented
- [ ] All public enums and variants are documented
- [ ] All public traits and methods are documented
- [ ] Complex functions include examples
- [ ] Error conditions are documented
- [ ] Panics are documented (if applicable)
- [ ] Links to related items are included
- [ ] `cargo doc --no-deps` runs without warnings
- [ ] Generated documentation is browsable and readable

## Cargo Doc Generation

### Generate Documentation
```bash
# Generate documentation for your crate only (no dependencies)
cargo doc --no-deps

# Generate and open in browser
cargo doc --no-deps --open

# Generate with private items (for internal development)
cargo doc --no-deps --document-private-items
```

### Documentation Features
```rust
// Hide items from documentation
#[doc(hidden)]
pub fn internal_function() {}

// Add search keywords
#[doc(alias = "authenticate")]
#[doc(alias = "signin")]
pub fn login() {}

// Include external documentation
#![doc = include_str!("../README.md")]
```

## References

- [Rust RFC 1574: API Documentation Conventions](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html)
- [The Rust Book: Making Useful Documentation Comments](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments)
- [Rust By Example: Documentation](https://doc.rust-lang.org/rust-by-example/meta/doc.html)
- [Rustdoc Book](https://doc.rust-lang.org/rustdoc/)

## Enforcement

Documentation quality will be enforced through:

1. **CI/CD Pipeline**: `cargo doc --no-deps` must pass without warnings
2. **Code Review**: PR checklist includes documentation review
3. **Coverage Metrics**: Track percentage of documented public items
4. **Quality Standards**: Examples required for complex functions
