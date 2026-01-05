# Testing Requirements

Comprehensive testing guidelines for Cobalt Stack contributors.

## Table of Contents

- [Overview](#overview)
- [Testing Philosophy](#testing-philosophy)
- [Backend Testing (Rust)](#backend-testing-rust)
- [Frontend Testing (TypeScript)](#frontend-testing-typescript)
- [Test Coverage Requirements](#test-coverage-requirements)
- [CI/CD Integration](#cicd-integration)
- [Best Practices](#best-practices)

## Overview

Testing is a critical part of the development process in Cobalt Stack. All code contributions must include appropriate tests to ensure quality, reliability, and maintainability.

### Testing Pyramid

We follow the testing pyramid approach:

```
        /\
       /  \  E2E Tests (Few)
      /    \
     /------\
    / Integ  \ Integration Tests (Some)
   /   Tests  \
  /------------\
 /   Unit Tests \ Unit Tests (Many)
/________________\
```

- **Unit Tests**: Test individual functions and modules in isolation
- **Integration Tests**: Test multiple components working together
- **E2E Tests**: Test complete user workflows

## Testing Philosophy

### Core Principles

1. **Test Behavior, Not Implementation**
   - Focus on what the code does, not how it does it
   - Tests should survive refactoring

2. **Write Tests First (TDD)**
   - Write failing test
   - Implement minimum code to pass
   - Refactor

3. **Keep Tests Simple**
   - Each test should verify one behavior
   - Tests should be easy to understand
   - Avoid complex logic in tests

4. **Make Tests Independent**
   - Tests should not depend on each other
   - Tests should be able to run in any order
   - Clean up after each test

5. **Make Tests Fast**
   - Unit tests should run in milliseconds
   - Integration tests should run in seconds
   - Use mocks for external dependencies

## Backend Testing (Rust)

### Test Organization

```
backend/
├── src/
│   ├── handlers/
│   │   ├── auth.rs
│   │   └── mod.rs
│   └── services/
│       ├── auth/
│       │   ├── mod.rs
│       │   └── jwt.rs
│       └── mod.rs
└── tests/
    ├── integration/
    │   ├── auth_tests.rs
    │   └── mod.rs
    └── common/
        └── mod.rs
```

### Unit Tests

Unit tests live in the same file as the code they test:

```rust
// src/services/auth/jwt.rs

pub fn validate_token(token: &str) -> Result<Claims> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_token_success() {
        let token = "valid_token";
        let result = validate_token(token);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_token_expired() {
        let expired_token = "expired_token";
        let result = validate_token(expired_token);
        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[test]
    fn test_validate_token_invalid_format() {
        let invalid_token = "invalid";
        let result = validate_token(invalid_token);
        assert!(result.is_err());
    }
}
```

### Integration Tests

Integration tests go in the `tests/` directory:

```rust
// tests/integration/auth_tests.rs

use axum::http::StatusCode;
use serde_json::json;

mod common;

#[tokio::test]
async fn test_login_success() {
    let app = common::setup_test_app().await;

    let response = app
        .request()
        .method("POST")
        .uri("/api/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: AuthResponse = response.json().await;
    assert!(!body.access_token.is_empty());
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let app = common::setup_test_app().await;

    let response = app
        .request()
        .method("POST")
        .uri("/api/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "wrongpassword"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

### Test Database Setup

```rust
// tests/common/mod.rs

use sea_orm::{Database, DatabaseConnection};

pub async fn setup_test_db() -> DatabaseConnection {
    // Use test database
    let db_url = "postgres://postgres:postgres@localhost/cobalt_test";
    let db = Database::connect(db_url).await.unwrap();

    // Run migrations
    migration::run(&db).await.unwrap();

    db
}

pub async fn teardown_test_db(db: &DatabaseConnection) {
    // Clean up test data
    migration::rollback(&db).await.unwrap();
}
```

### Mocking

Use `mockall` for mocking:

```rust
use mockall::{automock, predicate::*};

#[automock]
pub trait EmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_verification_email() {
        let mut mock_email = MockEmailService::new();

        mock_email
            .expect_send_email()
            .with(eq("user@example.com"), eq("Verify Email"), always())
            .times(1)
            .returning(|_, _, _| Ok(()));

        let result = send_verification(&mock_email, "user@example.com");
        assert!(result.is_ok());
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_login_success

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration

# Run with coverage
cargo tarpaulin --out Html
```

### Test Coverage

Generate coverage reports:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage
```

## Frontend Testing (TypeScript)

### Test Organization

```
frontend/
├── src/
│   ├── components/
│   │   ├── UserProfile.tsx
│   │   └── UserProfile.test.tsx
│   ├── lib/
│   │   ├── api.ts
│   │   └── api.test.ts
│   └── hooks/
│       ├── useAuth.ts
│       └── useAuth.test.ts
└── e2e/
    └── auth.spec.ts
```

### Unit Tests

Test React components with React Testing Library:

```typescript
// src/components/UserProfile.test.tsx

import { render, screen } from '@testing-library/react';
import { UserProfile } from './UserProfile';

describe('UserProfile', () => {
  it('renders user information', () => {
    const user = {
      id: '123',
      username: 'alice',
      email: 'alice@example.com',
      emailVerified: true,
      role: 'User' as const,
    };

    render(<UserProfile user={user} />);

    expect(screen.getByText('alice')).toBeInTheDocument();
    expect(screen.getByText('alice@example.com')).toBeInTheDocument();
  });

  it('shows verification badge for verified users', () => {
    const user = {
      id: '123',
      username: 'alice',
      email: 'alice@example.com',
      emailVerified: true,
      role: 'User' as const,
    };

    render(<UserProfile user={user} />);

    expect(screen.getByText('Verified ✓')).toBeInTheDocument();
  });

  it('shows verification prompt for unverified users', () => {
    const user = {
      id: '123',
      username: 'alice',
      email: 'alice@example.com',
      emailVerified: false,
      role: 'User' as const,
    };

    render(<UserProfile user={user} />);

    expect(screen.getByText('Verify Email')).toBeInTheDocument();
  });
});
```

### Testing Hooks

```typescript
// src/hooks/useAuth.test.ts

import { renderHook, waitFor } from '@testing-library/react';
import { useAuth } from './useAuth';

describe('useAuth', () => {
  it('loads user on mount', async () => {
    const { result } = renderHook(() => useAuth());

    expect(result.current.loading).toBe(true);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.user).toBeDefined();
  });

  it('handles authentication errors', async () => {
    // Mock API to return error
    jest.spyOn(global, 'fetch').mockRejectedValueOnce(
      new Error('Authentication failed')
    );

    const { result } = renderHook(() => useAuth());

    await waitFor(() => {
      expect(result.current.error).toBeDefined();
    });
  });
});
```

### Mocking API Calls

Use `msw` (Mock Service Worker):

```typescript
// src/lib/test-utils.ts

import { rest } from 'msw';
import { setupServer } from 'msw/node';

export const handlers = [
  rest.post('/api/auth/login', (req, res, ctx) => {
    return res(
      ctx.json({
        access_token: 'mock_token',
        token_type: 'Bearer',
        expires_in: 900,
      })
    );
  }),

  rest.get('/api/auth/me', (req, res, ctx) => {
    return res(
      ctx.json({
        id: '123',
        username: 'testuser',
        email: 'test@example.com',
        emailVerified: true,
        role: 'User',
      })
    );
  }),
];

export const server = setupServer(...handlers);

// Setup
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

### E2E Tests

Use Playwright for E2E testing:

```typescript
// e2e/auth.spec.ts

import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('user can login', async ({ page }) => {
    await page.goto('http://localhost:3000/login');

    await page.fill('input[name="username"]', 'testuser');
    await page.fill('input[name="password"]', 'password123');
    await page.click('button[type="submit"]');

    await expect(page).toHaveURL('http://localhost:3000/dashboard');
    await expect(page.locator('text=Welcome, testuser')).toBeVisible();
  });

  test('shows error for invalid credentials', async ({ page }) => {
    await page.goto('http://localhost:3000/login');

    await page.fill('input[name="username"]', 'testuser');
    await page.fill('input[name="password"]', 'wrongpassword');
    await page.click('button[type="submit"]');

    await expect(page.locator('text=Invalid credentials')).toBeVisible();
  });
});
```

### Running Tests

```bash
# Run all tests
npm test

# Run in watch mode
npm test -- --watch

# Run with coverage
npm run test:coverage

# Run E2E tests
npm run test:e2e
```

## Test Coverage Requirements

### Minimum Coverage

| Type | Minimum Coverage |
|------|------------------|
| Overall | 70% |
| New Code | 80% |
| Critical Paths | 90% |
| Business Logic | 85% |

### Coverage Reports

**Backend (Rust)**:
```bash
cargo tarpaulin --out Html
open tarpaulin-report.html
```

**Frontend (TypeScript)**:
```bash
npm run test:coverage
open coverage/index.html
```

### What to Test

#### Must Test

- All public APIs
- Business logic
- Error handling
- Edge cases
- Security-critical code

#### Should Test

- Helper functions
- Utilities
- Validation logic
- Data transformations

#### Can Skip

- Trivial getters/setters
- Simple pass-through functions
- Generated code
- Third-party integrations (mock instead)

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/test.yml

name: Tests

on: [push, pull_request]

jobs:
  backend:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        run: |
          cd backend
          cargo test

      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3

  frontend:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: 18

      - name: Install dependencies
        run: |
          cd frontend
          npm ci

      - name: Run tests
        run: |
          cd frontend
          npm test -- --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## Best Practices

### Do's

- Write tests before code (TDD)
- Keep tests simple and focused
- Use descriptive test names
- Test edge cases and error paths
- Clean up after tests
- Use fixtures for test data
- Mock external dependencies

### Don'ts

- Don't test implementation details
- Don't write flaky tests
- Don't share state between tests
- Don't skip tests (fix or remove them)
- Don't write overly complex tests
- Don't test third-party libraries

### Test Naming

Use descriptive test names that explain the scenario:

```rust
// Good
#[test]
fn test_login_returns_token_for_valid_credentials() { }

#[test]
fn test_login_returns_error_for_invalid_password() { }

// Bad
#[test]
fn test_login() { }

#[test]
fn test_login2() { }
```

### Test Structure

Follow the AAA pattern:

```rust
#[test]
fn test_user_creation() {
    // Arrange
    let username = "alice";
    let email = "alice@example.com";

    // Act
    let user = User::new(username, email);

    // Assert
    assert_eq!(user.username, username);
    assert_eq!(user.email, email);
}
```

### Assertions

Use specific assertions:

```rust
// Good
assert_eq!(result, expected);
assert!(result.is_ok());
assert!(list.contains(&item));

// Bad
assert!(result == expected);
assert!(result.is_ok() == true);
```

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)
- [Playwright Documentation](https://playwright.dev/)
- [Jest Documentation](https://jestjs.io/)
- [CONTRIBUTING.md](../../CONTRIBUTING.md)
- [Code Style Guide](./code-style.md)
- [Pull Request Guidelines](./pull-requests.md)

## Questions?

If you have questions about testing:

1. Check this documentation
2. Look at existing tests in the codebase
3. Ask in GitHub Discussions
4. Reach out to maintainers
