# Backend Testing

## Table of Contents
- [Overview](#overview)
- [Testing Philosophy](#testing-philosophy)
- [Unit Testing](#unit-testing)
- [Integration Testing](#integration-testing)
- [Test Database Setup](#test-database-setup)
- [Testing Patterns](#testing-patterns)
- [Test Coverage](#test-coverage)
- [Running Tests](#running-tests)

## Overview

The Cobalt Stack backend uses Rust's built-in testing framework with additional tools for database testing. Tests are organized by layer (handlers, services, models) and type (unit, integration).

**Testing Stack:**
- **Framework**: Rust's `#[test]` and `#[tokio::test]`
- **Mocking**: `mockall` crate for mock implementations
- **Assertions**: Standard `assert!`, `assert_eq!`, custom assertions
- **Database**: In-memory SQLite for fast tests

## Testing Philosophy

### The Testing Pyramid

```text
           ┌─────────────┐
          ╱  E2E Tests   ╱    ← Few, slow, brittle
         ╱    (Manual)  ╱
        ╱──────────────╱
       ╱  Integration ╱      ← Some, medium speed
      ╱     Tests    ╱
     ╱──────────────╱
    ╱  Unit Tests  ╱         ← Many, fast, reliable
   ╱──────────────╱
```

### Testing Principles

1. **Fast Feedback**: Tests should run quickly (< 5 minutes for full suite)
2. **Isolation**: Each test is independent and can run in any order
3. **Clarity**: Test names describe what they test and expected behavior
4. **Coverage**: Aim for >80% coverage on business logic
5. **Maintainability**: Tests should be as clean as production code

### What to Test

✅ **Always Test**
- Business logic in services
- Input validation
- Error handling paths
- Security-critical code (auth, passwords)
- Data transformations

⚠️ **Sometimes Test**
- Database queries (if complex)
- Handler orchestration
- Edge cases and boundary conditions

❌ **Rarely Test**
- Trivial getters/setters
- External library behavior
- Framework behavior

## Unit Testing

### Service Layer Tests

**Location**: Test modules within service files

```rust
// services/auth/password.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_success() {
        let password = "secure_password_123";
        let hash = hash_password(password).unwrap();

        // Verify hash format
        assert!(hash.starts_with("$argon2"));
        assert!(hash.len() > 50);
    }

    #[test]
    fn test_hash_password_too_short() {
        let password = "short";
        let result = hash_password(password);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Weak password"));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "correct_password";
        let hash = hash_password(password).unwrap();

        let is_valid = verify_password(password, &hash).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "correct_password";
        let hash = hash_password(password).unwrap();

        let is_valid = verify_password("wrong_password", &hash).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_hash_produces_unique_salts() {
        let password = "same_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Different salts produce different hashes
        assert_ne!(hash1, hash2);

        // But both verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }
}
```

### JWT Token Tests

```rust
// services/auth/jwt.rs

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test_secret_key".to_string(),
            access_token_expiry_minutes: 30,
            refresh_token_expiry_days: 7,
        }
    }

    #[test]
    fn test_create_access_token() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();

        let token = create_access_token(user_id, username.clone(), &config).unwrap();

        // JWT has 3 parts: header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_verify_access_token_valid() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();

        let token = create_access_token(user_id, username.clone(), &config).unwrap();
        let claims = verify_access_token(&token, &config).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);
        assert!(claims.exp > Utc::now().timestamp());
    }

    #[test]
    fn test_verify_access_token_wrong_secret() {
        let config = test_config();
        let user_id = Uuid::new_v4();

        let token = create_access_token(user_id, "test".to_string(), &config).unwrap();

        // Try to verify with different secret
        let wrong_config = JwtConfig {
            secret: "different_secret".to_string(),
            ..config
        };

        let result = verify_access_token(&token, &wrong_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_tokens_have_unique_jti() {
        let config = test_config();
        let user_id = Uuid::new_v4();

        let (_, jti1) = create_refresh_token(user_id, &config).unwrap();
        let (_, jti2) = create_refresh_token(user_id, &config).unwrap();

        assert_ne!(jti1, jti2);
    }
}
```

### Validation Tests

```rust
// handlers/auth.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_validation_valid() {
        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_register_request_validation_username_too_short() {
        let req = RegisterRequest {
            username: "ab".to_string(),
            email: "alice@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("between 3 and 50"));
    }

    #[test]
    fn test_register_request_validation_invalid_email() {
        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "not-an-email".to_string(),
            password: "SecurePass123!".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid email"));
    }

    #[test]
    fn test_register_request_validation_password_too_short() {
        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "short".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 8 characters"));
    }
}
```

### Middleware Tests

```rust
// middleware/auth.rs

#[cfg(test)]
mod tests {
    use super::*;

    fn test_jwt_config() -> JwtConfig {
        JwtConfig {
            secret: "test_secret_key".to_string(),
            access_token_expiry_minutes: 30,
            refresh_token_expiry_days: 7,
        }
    }

    #[test]
    fn test_extract_token_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer valid_token_here".parse().unwrap(),
        );

        let result = extract_token_from_header(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "valid_token_here");
    }

    #[test]
    fn test_extract_token_no_header() {
        let headers = HeaderMap::new();
        let result = extract_token_from_header(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_no_bearer_prefix() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "InvalidFormat".parse().unwrap());

        let result = extract_token_from_header(&headers);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_valid_token() {
        let config = test_jwt_config();
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();

        let token = create_access_token(user_id, username.clone(), &config).unwrap();
        let result = verify_access_token(&token, &config);

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);
    }
}
```

## Integration Testing

### Test Database Setup

Create test helpers for database operations:

```rust
// tests/helpers/database.rs

use sea_orm::{Database, DatabaseConnection, DbErr};

/// Create in-memory SQLite database for testing
pub async fn setup_test_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;

    // Run migrations
    migration::Migrator::up(&db, None).await?;

    Ok(db)
}

/// Seed test data
pub async fn seed_test_user(db: &DatabaseConnection) -> Result<users::Model, DbErr> {
    use sea_orm::{ActiveModelTrait, Set};

    let user = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set("testuser".to_string()),
        email: Set("test@example.com".to_string()),
        password_hash: Set(Some(hash_password("password123").unwrap())),
        email_verified: Set(true),
        role: Set(UserRole::User),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };

    user.insert(db).await
}
```

### Database Integration Tests

```rust
// tests/integration/token_rotation.rs

use crate::helpers::database::{setup_test_db, seed_test_user};

#[tokio::test]
async fn test_token_rotation_workflow() {
    let db = setup_test_db().await.unwrap();
    let user = seed_test_user(&db).await.unwrap();
    let config = test_jwt_config();

    // Create initial token
    let (token1, jti1) = create_refresh_token(user.id, &config).unwrap();
    store_refresh_token(&db, user.id, &token1, jti1, 7).await.unwrap();

    // Validate token works
    let validated_user_id = validate_refresh_token(&db, &token1, jti1).await.unwrap();
    assert_eq!(validated_user_id, user.id);

    // Rotate token
    let (token2, jti2) = create_refresh_token(user.id, &config).unwrap();
    rotate_refresh_token(&db, jti1, &token2, jti2, user.id, 7).await.unwrap();

    // Old token should be revoked
    let result = validate_refresh_token(&db, &token1, jti1).await;
    assert!(result.is_err());

    // New token should work
    let validated_user_id = validate_refresh_token(&db, &token2, jti2).await.unwrap();
    assert_eq!(validated_user_id, user.id);
}

#[tokio::test]
async fn test_revoke_all_user_tokens() {
    let db = setup_test_db().await.unwrap();
    let user = seed_test_user(&db).await.unwrap();
    let config = test_jwt_config();

    // Create multiple tokens
    let (token1, jti1) = create_refresh_token(user.id, &config).unwrap();
    let (token2, jti2) = create_refresh_token(user.id, &config).unwrap();

    store_refresh_token(&db, user.id, &token1, jti1, 7).await.unwrap();
    store_refresh_token(&db, user.id, &token2, jti2, 7).await.unwrap();

    // Revoke all tokens
    revoke_all_user_tokens(&db, user.id).await.unwrap();

    // Both tokens should be revoked
    assert!(validate_refresh_token(&db, &token1, jti1).await.is_err());
    assert!(validate_refresh_token(&db, &token2, jti2).await.is_err());
}
```

### Handler Integration Tests

```rust
// tests/integration/auth_handlers.rs

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_register_handler_success() {
    let db = setup_test_db().await.unwrap();
    let state = AppState {
        db: Arc::new(db),
        jwt_config: test_jwt_config(),
    };

    let app = create_test_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(r#"{
            "username": "newuser",
            "email": "newuser@example.com",
            "password": "SecurePass123!"
        }"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify response contains access token
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let auth_response: AuthResponse = serde_json::from_slice(&body).unwrap();
    assert!(!auth_response.access_token.is_empty());
    assert_eq!(auth_response.token_type, "Bearer");
}

#[tokio::test]
async fn test_register_handler_duplicate_username() {
    let db = setup_test_db().await.unwrap();
    seed_test_user(&db).await.unwrap(); // Username: testuser

    let state = AppState {
        db: Arc::new(db),
        jwt_config: test_jwt_config(),
    };

    let app = create_test_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(r#"{
            "username": "testuser",
            "email": "different@example.com",
            "password": "SecurePass123!"
        }"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}
```

## Testing Patterns

### Pattern 1: Arrange-Act-Assert (AAA)

```rust
#[test]
fn test_password_hashing() {
    // Arrange: Set up test data
    let password = "test_password_123";

    // Act: Perform the operation
    let hash = hash_password(password).unwrap();

    // Assert: Verify expectations
    assert!(hash.starts_with("$argon2"));
    assert!(verify_password(password, &hash).unwrap());
}
```

### Pattern 2: Table-Driven Tests

```rust
#[test]
fn test_password_validation() {
    let test_cases = vec![
        ("short", false, "too short"),
        ("validpassword", true, "valid"),
        ("a".repeat(129).as_str(), false, "too long"),
        ("12345678", true, "minimum length"),
    ];

    for (password, should_succeed, description) in test_cases {
        let result = validate_password(password);
        assert_eq!(
            result.is_ok(),
            should_succeed,
            "Failed for case: {}",
            description
        );
    }
}
```

### Pattern 3: Test Fixtures

```rust
struct TestFixture {
    db: DatabaseConnection,
    user: users::Model,
    config: JwtConfig,
}

impl TestFixture {
    async fn new() -> Self {
        let db = setup_test_db().await.unwrap();
        let user = seed_test_user(&db).await.unwrap();
        let config = test_jwt_config();

        Self { db, user, config }
    }
}

#[tokio::test]
async fn test_with_fixture() {
    let fixture = TestFixture::new().await;

    // Use fixture.db, fixture.user, fixture.config
    let token = create_access_token(
        fixture.user.id,
        fixture.user.username,
        &fixture.config
    ).unwrap();

    assert!(!token.is_empty());
}
```

### Pattern 4: Custom Assertions

```rust
// Custom assertion for better error messages
fn assert_auth_error(result: Result<(), AuthError>, expected_error: &str) {
    match result {
        Ok(_) => panic!("Expected error but got Ok"),
        Err(e) => assert!(
            e.to_string().contains(expected_error),
            "Expected error containing '{}', got '{}'",
            expected_error,
            e
        ),
    }
}

#[test]
fn test_invalid_credentials() {
    let result = authenticate("user", "wrong_password");
    assert_auth_error(result, "Invalid credentials");
}
```

### Pattern 5: Mocking External Services

```rust
use mockall::mock;

// Define mock trait
mock! {
    pub EmailSender {}

    impl EmailSender for EmailSender {
        fn send_verification_email(&self, to: &str, token: &str) -> Result<()>;
    }
}

#[tokio::test]
async fn test_registration_sends_email() {
    let mut mock_email = MockEmailSender::new();

    // Set expectation
    mock_email
        .expect_send_verification_email()
        .times(1)
        .returning(|_, _| Ok(()));

    // Test with mock
    let result = register_user("user", "email@test.com", mock_email).await;
    assert!(result.is_ok());
}
```

## Test Coverage

### Measuring Coverage

Install `cargo-tarpaulin`:

```bash
cargo install cargo-tarpaulin
```

Run coverage:

```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html
```

### Coverage Goals

```yaml
coverage_targets:
  services: 85%      # Business logic should be well-tested
  handlers: 70%      # Handler orchestration, some integration tests
  models: 60%        # Basic CRUD, complex queries tested
  middleware: 80%    # Security-critical, should be thorough
  overall: 75%       # Project-wide minimum
```

### Critical Paths (100% Coverage)

- Authentication logic
- Password hashing/verification
- Token generation/validation
- Input validation
- Security middleware

## Running Tests

### Run All Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_password_hashing
```

### Run Tests by Module

```bash
# Run service tests
cargo test --lib services

# Run integration tests
cargo test --test integration

# Run specific integration test file
cargo test --test token_rotation
```

### Run Tests in Parallel

```bash
# Run with specific thread count
cargo test -- --test-threads=4

# Run sequentially (for database tests)
cargo test -- --test-threads=1
```

### Watch Mode

Install `cargo-watch`:

```bash
cargo install cargo-watch

# Run tests on file change
cargo watch -x test

# Run specific tests on change
cargo watch -x "test services::auth"
```

### Continuous Integration

**.github/workflows/test.yml**:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: cobalt_stack_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost/cobalt_stack_test
        run: cargo test --all-features

      - name: Generate coverage
        run: cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v2
```

## Best Practices

1. **Test Naming**: Use descriptive names that explain what is being tested
   ```rust
   // ✅ Good
   #[test]
   fn test_register_fails_with_duplicate_username() { }

   // ❌ Bad
   #[test]
   fn test_register_2() { }
   ```

2. **One Assertion Per Test**: Focus each test on a single behavior
   ```rust
   // ✅ Good
   #[test]
   fn test_password_hash_starts_with_argon2() {
       let hash = hash_password("test").unwrap();
       assert!(hash.starts_with("$argon2"));
   }

   #[test]
   fn test_password_hash_is_long_enough() {
       let hash = hash_password("test").unwrap();
       assert!(hash.len() > 50);
   }
   ```

3. **Test Edge Cases**: Don't just test the happy path
   ```rust
   #[test]
   fn test_username_minimum_length() { }

   #[test]
   fn test_username_maximum_length() { }

   #[test]
   fn test_username_empty_string() { }

   #[test]
   fn test_username_special_characters() { }
   ```

4. **Use Test Helpers**: Extract common setup code
5. **Keep Tests Independent**: Each test should set up and tear down its own data
6. **Test Errors**: Ensure error cases are tested thoroughly
7. **Document Complex Tests**: Add comments explaining non-obvious test logic

## Related Documentation

- [Architecture](./architecture.md) - Understanding the code structure
- [Services](./services.md) - Service layer to be tested
- [Handlers](./api-handlers.md) - Handler layer testing
- [Database](./database.md) - Database testing setup
