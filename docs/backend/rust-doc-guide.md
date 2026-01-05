# Rust Documentation Guide

Complete guide to generating and using Rust documentation in Cobalt Stack.

## Table of Contents

- [Quick Start](#quick-start)
- [Generating Documentation](#generating-documentation)
- [Documentation Commands](#documentation-commands)
- [Writing Documentation](#writing-documentation)
- [Best Practices](#best-practices)
- [Examples](#examples)

## Quick Start

Generate and open documentation:

```bash
cargo doc --no-deps --open
```

This command:
1. Generates documentation for your project (excluding dependencies)
2. Opens the documentation in your default browser

## Generating Documentation

### Basic Commands

```bash
# Generate documentation
cargo doc

# Generate without dependencies (faster)
cargo doc --no-deps

# Generate and open in browser
cargo doc --no-deps --open

# Include private items
cargo doc --no-deps --document-private-items

# Generate for specific package
cargo doc -p cobalt-backend
```

### Development Workflow

```bash
# Watch for changes and regenerate
cargo watch -x "doc --no-deps"

# Generate with all features
cargo doc --no-deps --all-features

# Generate for specific target
cargo doc --no-deps --target x86_64-unknown-linux-gnu
```

## Documentation Commands

### Common Options

| Command | Description |
|---------|-------------|
| `--no-deps` | Don't document dependencies |
| `--open` | Open documentation in browser |
| `--document-private-items` | Include private items |
| `--all-features` | Document all features |
| `-p <package>` | Document specific package |

### Environment Variables

```bash
# Custom output directory
CARGO_TARGET_DIR=./my-docs cargo doc

# Verbose output
CARGO_LOG=cargo::ops::cargo_rustdoc=debug cargo doc
```

## Writing Documentation

### Documentation Comments

```rust
/// Single-line documentation comment
///
/// # Examples
///
/// ```
/// let result = my_function();
/// assert_eq!(result, 42);
/// ```
pub fn my_function() -> i32 {
    42
}

/** Multi-line documentation comment
 *
 * Useful for longer descriptions.
 */
pub struct MyStruct {
    /// Field-level documentation
    pub field: String,
}
```

### Documentation Sections

```rust
/// Brief description of the function
///
/// # Arguments
///
/// * `param1` - Description of param1
/// * `param2` - Description of param2
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// This function will return an error if:
/// - Condition 1
/// - Condition 2
///
/// # Examples
///
/// ```
/// use crate::my_function;
///
/// let result = my_function(1, 2);
/// assert_eq!(result, 3);
/// ```
///
/// # Panics
///
/// This function panics if the input is invalid
///
/// # Safety
///
/// (For unsafe functions) Explain safety requirements
pub fn my_function(param1: i32, param2: i32) -> Result<i32, Error> {
    // Implementation
}
```

### Module Documentation

```rust
//! Module-level documentation
//!
//! This module provides functionality for...
//!
//! # Examples
//!
//! ```
//! use crate::my_module::MyStruct;
//!
//! let instance = MyStruct::new();
//! ```

pub struct MyStruct;
```

## Best Practices

### 1. Document Public APIs

Always document public items:
- Functions and methods
- Structs and enums
- Traits and implementations
- Constants and statics

### 2. Include Examples

Examples are the most valuable documentation:

```rust
/// Adds two numbers together
///
/// # Examples
///
/// ```
/// use cobalt_backend::math::add;
///
/// assert_eq!(add(1, 2), 3);
/// assert_eq!(add(-1, 1), 0);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### 3. Document Error Conditions

```rust
/// Reads configuration from file
///
/// # Errors
///
/// Returns `ConfigError::NotFound` if the file doesn't exist
/// Returns `ConfigError::ParseError` if the file is malformed
pub fn read_config(path: &Path) -> Result<Config, ConfigError> {
    // Implementation
}
```

### 4. Link to Related Items

```rust
/// See also [`related_function`] and [`RelatedStruct`]
///
/// [`related_function`]: crate::module::related_function
/// [`RelatedStruct`]: crate::module::RelatedStruct
pub fn my_function() {
    // Implementation
}
```

### 5. Use Code Blocks

```rust
/// # Example with Rust code
///
/// ```rust
/// let x = 5;
/// println!("{}", x);
/// ```
///
/// # Example with ignored code
///
/// ```rust,ignore
/// // This won't be tested
/// ```
///
/// # Example that should fail
///
/// ```rust,should_panic
/// panic!("This is expected");
/// ```
pub fn example() {}
```

## Examples

### Documenting a Service

```rust
/// User service for managing user accounts
///
/// This service provides operations for:
/// - Creating new users
/// - Updating user information
/// - Deleting users
/// - Retrieving user data
///
/// # Examples
///
/// ```
/// use cobalt_backend::services::UserService;
///
/// let service = UserService::new(db_pool);
/// let user = service.create_user("john@example.com").await?;
/// ```
pub struct UserService {
    pool: PgPool,
}

impl UserService {
    /// Creates a new UserService instance
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    ///
    /// # Examples
    ///
    /// ```
    /// let service = UserService::new(pool);
    /// ```
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new user
    ///
    /// # Arguments
    ///
    /// * `email` - User's email address
    ///
    /// # Returns
    ///
    /// Returns the created user on success
    ///
    /// # Errors
    ///
    /// Returns `UserError::EmailExists` if email is already registered
    /// Returns `UserError::DatabaseError` on database failures
    ///
    /// # Examples
    ///
    /// ```
    /// let user = service.create_user("john@example.com").await?;
    /// assert_eq!(user.email, "john@example.com");
    /// ```
    pub async fn create_user(&self, email: &str) -> Result<User, UserError> {
        // Implementation
    }
}
```

### Documenting Errors

```rust
/// Errors that can occur in user operations
///
/// # Examples
///
/// ```
/// use cobalt_backend::errors::UserError;
///
/// fn example() -> Result<(), UserError> {
///     Err(UserError::NotFound("user123".to_string()))
/// }
/// ```
#[derive(Debug, Error)]
pub enum UserError {
    /// User was not found
    #[error("User not found: {0}")]
    NotFound(String),

    /// Email already exists
    #[error("Email already registered: {0}")]
    EmailExists(String),

    /// Database operation failed
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
```

## Related Resources

- [Backend Documentation](./README.md)
- [API Reference](../api/README.md)
- [Contributing Guidelines](../contributing/README.md)
- [Rust Documentation Book](https://doc.rust-lang.org/rustdoc/)
