# Testing Guide

Complete guide to testing in Cobalt Stack with TDD workflow and coverage requirements.

## Table of Contents

- [Overview](#overview)
- [Running Tests](#running-tests)
- [Writing Tests](#writing-tests)
- [TDD Workflow](#tdd-workflow)
- [Unit Testing](#unit-testing)
- [Integration Testing](#integration-testing)
- [E2E Testing](#e2e-testing)
- [Coverage Requirements](#coverage-requirements)
- [Troubleshooting](#troubleshooting)

## Overview

Cobalt Stack uses comprehensive testing strategies:

- **Backend**: Rust with `cargo test` and `cargo-tarpaulin` for coverage
- **Frontend**: Jest and React Testing Library (planned)
- **E2E**: Playwright for browser automation (planned)
- **TDD Approach**: Red-Green-Refactor cycle

### Testing Philosophy

1. **Test-Driven Development**: Write tests before implementation
2. **Fast Feedback**: Run tests frequently during development
3. **High Coverage**: Aim for >80% code coverage
4. **Meaningful Tests**: Focus on behavior, not implementation
5. **Maintainable**: Tests should be easy to read and update

## Running Tests

### Backend Tests

```bash
# Run all tests
make test

# Or directly with cargo
cd backend
cargo test

# Run tests in watch mode (auto-rerun on changes)
make test-watch

# Or with cargo-watch
cargo watch -x test

# Run specific test
cargo test test_create_access_token

# Run tests in a specific file
cargo test --test auth_tests

# Run with output (show println!)
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test -- --test-threads=4

# Run tests sequentially
cargo test -- --test-threads=1
```

### Test Coverage

```bash
# Generate coverage report
make test-coverage

# Or with cargo-tarpaulin
cd backend
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/index.html
```

Coverage targets:
- **Minimum**: 70% overall coverage
- **Goal**: 80% overall coverage
- **Critical paths**: 90%+ coverage (auth, payment, security)

### Continuous Integration

Tests run automatically on:
- Every commit (local pre-commit hook)
- Pull requests (GitHub Actions)
- Before deployment (CI/CD pipeline)

```bash
# Run all CI checks
make ci

# This runs: fmt-check, lint, test
```

## Writing Tests

### Test Organization

```
backend/src/
├── services/
│   ├── auth/
│   │   ├── jwt.rs
│   │   └── jwt_tests.rs  # Unit tests alongside code
│   └── email/
│       ├── mod.rs
│       └── tests.rs      # Module-level tests
└── tests/
    ├── integration_tests.rs  # Integration tests
    └── common/
        └── mod.rs           # Test utilities
```

### Basic Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Simple test
    #[test]
    fn test_addition() {
        assert_eq!(2 + 2, 4);
    }

    // Async test
    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }

    // Test with expected panic
    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_divide_by_zero() {
        let _ = 1 / 0;
    }

    // Ignored test (not run by default)
    #[test]
    #[ignore]
    fn expensive_test() {
        // Long-running test
    }
}
```

### Test Naming Conventions

```rust
// Pattern: test_[function]_[scenario]_[expected_result]

#[test]
fn test_create_token_valid_input_returns_token() { }

#[test]
fn test_verify_token_expired_returns_error() { }

#[test]
fn test_hash_password_empty_string_panics() { }
```

## TDD Workflow

### Red-Green-Refactor Cycle

```
1. RED:    Write failing test
    ↓
2. GREEN:  Write minimal code to pass
    ↓
3. REFACTOR: Improve code quality
    ↓
    Repeat
```

### Example: TDD for JWT Creation

**Step 1: RED (Write Failing Test)**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_access_token_returns_valid_jwt() {
        let config = JwtConfig {
            secret: "test_secret".to_string(),
            access_token_expiry_minutes: 30,
            refresh_token_expiry_days: 7,
        };

        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();

        // This will fail because function doesn't exist yet
        let token = create_access_token(user_id, username, &config).unwrap();

        // JWT should have 3 parts
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }
}
```

Run test: `cargo test test_create_access_token` → **FAILS** ❌

**Step 2: GREEN (Minimal Implementation)**

```rust
pub fn create_access_token(
    user_id: Uuid,
    username: String,
    config: &JwtConfig,
) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::minutes(config.access_token_expiry_minutes);

    let claims = AccessTokenClaims {
        sub: user_id,
        username,
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|e| anyhow::Error::new(AuthError::JwtEncodingError))
}
```

Run test: `cargo test test_create_access_token` → **PASSES** ✅

**Step 3: REFACTOR (Improve Code)**

Add more tests and error handling:

```rust
#[test]
fn test_create_access_token_includes_user_id() {
    let config = test_config();
    let user_id = Uuid::new_v4();
    let token = create_access_token(user_id, "test".to_string(), &config).unwrap();

    let claims = verify_access_token(&token, &config).unwrap();
    assert_eq!(claims.sub, user_id);
}

#[test]
fn test_create_access_token_sets_expiration() {
    let config = test_config();
    let token = create_access_token(Uuid::new_v4(), "test".to_string(), &config).unwrap();

    let claims = verify_access_token(&token, &config).unwrap();
    assert!(claims.exp > Utc::now().timestamp());
}
```

## Unit Testing

### Testing Pure Functions

```rust
// Pure function to test
pub fn calculate_discount(price: f64, discount_percent: f64) -> f64 {
    price * (1.0 - discount_percent / 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_discount_10_percent() {
        assert_eq!(calculate_discount(100.0, 10.0), 90.0);
    }

    #[test]
    fn test_calculate_discount_no_discount() {
        assert_eq!(calculate_discount(100.0, 0.0), 100.0);
    }

    #[test]
    fn test_calculate_discount_full_discount() {
        assert_eq!(calculate_discount(100.0, 100.0), 0.0);
    }
}
```

### Testing with Mock Data

```rust
fn test_user() -> User {
    User {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        role: UserRole::User,
        email_verified: true,
        disabled_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn test_user_is_active() {
    let user = test_user();
    assert!(user.disabled_at.is_none());
}
```

### Testing Error Cases

```rust
#[test]
fn test_verify_token_invalid_signature() {
    let config = test_config();
    let token = "invalid.token.here";

    let result = verify_access_token(token, &config);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid token"));
}

#[test]
fn test_hash_password_empty_input() {
    let result = hash_password("");
    assert!(result.is_err());
}
```

### Parameterized Tests

```rust
#[test]
fn test_validate_username() {
    let valid_usernames = vec!["alice", "bob123", "user_name", "user-name"];

    for username in valid_usernames {
        assert!(validate_username(username).is_ok(), "Failed for: {}", username);
    }

    let invalid_usernames = vec!["ab", "user@name", "user name", "very_long_username_that_exceeds_limit"];

    for username in invalid_usernames {
        assert!(validate_username(username).is_err(), "Should fail for: {}", username);
    }
}
```

## Integration Testing

### Database Integration Tests

```rust
// tests/integration_tests.rs
use cobalt_stack::*;
use sea_orm::{Database, DatabaseConnection};
use uuid::Uuid;

async fn setup_test_db() -> DatabaseConnection {
    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/test_db".to_string());

    Database::connect(&db_url).await.expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_create_user_integration() {
    let db = setup_test_db().await;

    // Create user
    let user = create_user(
        &db,
        "testuser".to_string(),
        "test@example.com".to_string(),
        "password_hash".to_string(),
    )
    .await
    .expect("Failed to create user");

    // Verify user was created
    assert!(!user.id.is_nil());
    assert_eq!(user.username, "testuser");

    // Clean up
    Users::delete_by_id(user.id).exec(&db).await.expect("Failed to delete test user");
}
```

### API Endpoint Tests

```rust
use axum::http::StatusCode;
use axum_test_helper::TestClient;

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_app().await;
    let client = TestClient::new(app);

    let response = client.get("/health").send().await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: HealthResponse = response.json().await;
    assert_eq!(body.status, "healthy");
}

#[tokio::test]
async fn test_login_endpoint_success() {
    let app = create_test_app().await;
    let client = TestClient::new(app);

    let response = client
        .post("/api/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "password123"
        }))
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    let body: LoginResponse = response.json().await;
    assert!(!body.access_token.is_empty());
}
```

### Testing Middleware

```rust
#[tokio::test]
async fn test_auth_middleware_valid_token() {
    let app = create_test_app().await;
    let client = TestClient::new(app);

    // Get valid token
    let token = create_test_token();

    let response = client
        .get("/api/auth/me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_middleware_missing_token() {
    let app = create_test_app().await;
    let client = TestClient::new(app);

    let response = client.get("/api/auth/me").send().await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

## E2E Testing

### Playwright Tests (Planned)

```typescript
// tests/e2e/auth.spec.ts
import { test, expect } from '@playwright/test'

test.describe('Authentication', () => {
  test('user can register and login', async ({ page }) => {
    // Navigate to registration page
    await page.goto('http://localhost:2727/register')

    // Fill registration form
    await page.fill('[name="username"]', 'testuser')
    await page.fill('[name="email"]', 'test@example.com')
    await page.fill('[name="password"]', 'password123')

    // Submit form
    await page.click('button[type="submit"]')

    // Should redirect to login
    await expect(page).toHaveURL(/.*login/)

    // Login with new credentials
    await page.fill('[name="username"]', 'testuser')
    await page.fill('[name="password"]', 'password123')
    await page.click('button[type="submit"]')

    // Should be logged in
    await expect(page).toHaveURL(/.*dashboard/)
    await expect(page.locator('text=testuser')).toBeVisible()
  })

  test('invalid credentials show error', async ({ page }) => {
    await page.goto('http://localhost:2727/login')

    await page.fill('[name="username"]', 'invalid')
    await page.fill('[name="password"]', 'wrong')
    await page.click('button[type="submit"]')

    await expect(page.locator('.error-message')).toContainText('Invalid credentials')
  })
})
```

## Coverage Requirements

### Measuring Coverage

```bash
# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# Generate multiple formats
cargo tarpaulin --out Html --out Lcov --output-dir coverage

# Exclude test code from coverage
cargo tarpaulin --exclude-files '*tests.rs' '*test_*.rs'

# Set minimum coverage threshold (fails if below)
cargo tarpaulin --fail-under 80
```

### Coverage Goals

| Category | Minimum | Target | Notes |
|----------|---------|--------|-------|
| Overall | 70% | 80% | All production code |
| Services | 80% | 90% | Business logic layer |
| Authentication | 90% | 95% | Security-critical |
| Models | 60% | 70% | Generated code |
| Handlers | 75% | 85% | API endpoints |
| Middleware | 85% | 90% | Request processing |
| Utils | 70% | 80% | Helper functions |

### What to Test

**High Priority** (Must Test):
- Authentication and authorization
- Data validation
- Error handling
- Security features
- Payment processing
- User data operations

**Medium Priority** (Should Test):
- Business logic
- API endpoints
- Middleware
- Database operations
- Email functionality

**Low Priority** (Optional):
- Simple getters/setters
- Generated code
- Third-party integrations (mock instead)
- Configuration loading

### Improving Coverage

```bash
# Find uncovered lines
cargo tarpaulin --out Stdout | grep "Uncovered Lines"

# View detailed coverage
cargo tarpaulin --out Html
open coverage/index.html

# Focus on specific module
cargo tarpaulin --packages backend --lib --tests --exclude-files 'tests/*'
```

## Troubleshooting

### Tests Failing Intermittently

**Problem**: Tests pass sometimes, fail other times

**Solutions**:
1. Check for race conditions in parallel tests
2. Use `--test-threads=1` to run sequentially
3. Ensure proper test isolation
4. Check for shared mutable state
5. Use proper async/await patterns

### Database Tests Failing

**Problem**: Database-related tests failing

**Solutions**:
1. Ensure test database is running
2. Run migrations on test database
3. Clean up data between tests
4. Use transactions for test isolation
5. Check TEST_DATABASE_URL is set

### Slow Tests

**Problem**: Test suite takes too long

**Solutions**:
1. Run tests in parallel (default)
2. Use `cargo test --release` for faster compilation
3. Split long-running tests with `#[ignore]`
4. Mock external services
5. Use in-memory databases for tests

### Coverage Not Generating

**Problem**: `cargo tarpaulin` fails or hangs

**Solutions**:
1. Install/update tarpaulin: `cargo install cargo-tarpaulin`
2. Clean build: `cargo clean && cargo tarpaulin`
3. Check Rust version compatibility
4. Try with `--ignore-tests` flag
5. Check for hanging tests (use timeouts)

### Mock Data Issues

**Problem**: Test data setup is complex and brittle

**Solutions**:
1. Create factory functions for test data
2. Use builders pattern for complex objects
3. Store fixtures in separate files
4. Use shared test utilities
5. Consider property-based testing

## Best Practices

1. **Write tests first**: Follow TDD workflow
2. **Test behavior, not implementation**: Focus on what, not how
3. **Keep tests fast**: Use mocks and in-memory databases
4. **One assertion per test**: Makes failures clear
5. **Descriptive names**: Test name should explain what and why
6. **Arrange-Act-Assert**: Clear test structure
7. **Don't repeat yourself**: Use helper functions
8. **Test edge cases**: Empty strings, nulls, boundaries
9. **Clean up**: Remove test data after tests
10. **Review coverage**: Regularly check and improve

### Test Structure Template

```rust
#[test]
fn test_feature_scenario_outcome() {
    // ARRANGE: Set up test data and preconditions
    let config = test_config();
    let user_id = Uuid::new_v4();

    // ACT: Execute the function being tested
    let result = create_token(user_id, &config);

    // ASSERT: Verify the outcome
    assert!(result.is_ok());
    let token = result.unwrap();
    assert!(!token.is_empty());
}
```

## Related Documentation

- [Authentication Guide](./authentication.md) - Testing auth flows
- [Database Guide](./database.md) - Database testing strategies
- [API Client Guide](./api-client.md) - Testing API calls
- [Backend Architecture](../backend/README.md) - Backend testing patterns
