# Authentication System - Implementation Guide

## Overview

This guide provides a systematic approach to implementing the complete authentication system with TDD. The implementation is broken into phases that can be completed incrementally.

## Current Status

✅ **Completed:**
- Dependencies added (jsonwebtoken, argon2, thiserror, anyhow)
- OpenSpec proposal validated and committed
- Branch created: `feature/auth-system`

📋 **Todo:** Implementation of 23 major components

## Recommended Implementation Order (TDD)

### Phase 1: Database Foundation (Day 1)
**Approach**: Migrations → Entities → Tests

1. **Create migrations** (`backend/migration/src/`)
   ```bash
   cd backend
   sea-orm-cli migrate generate create_users_table
   sea-orm-cli migrate generate create_refresh_tokens_table
   sea-orm-cli migrate generate create_oauth_accounts_table
   ```

2. **Write migration SQL** following the schema in `design.md`

3. **Generate entities**
   ```bash
   sea-orm-cli generate entity -o src/models
   ```

4. **Test migrations**
   ```bash
   sea-orm-cli migrate up
   sea-orm-cli migrate down
   ```

### Phase 2: Core Auth Service with TDD (Day 2-3)
**Approach**: RED → GREEN → REFACTOR

#### 2.1 Error Handling First
```rust
// backend/src/services/auth/error.rs
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    // ... other variants
}
```

#### 2.2 Password Service (TDD)
**RED**: Write failing test
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_hash_password() {
        let password = "test123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }
}
```

**GREEN**: Implement minimum code
```rust
pub fn hash_password(password: &str) -> Result<String> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}
```

**REFACTOR**: Extract constants, improve error handling

#### 2.3 JWT Service (TDD)
Follow same RED-GREEN-REFACTOR pattern for:
- `create_access_token()`
- `create_refresh_token()`
- `verify_access_token()`
- `decode_token()`

#### 2.4 Token Rotation (TDD)
Test token refresh flow with rotation

### Phase 3: Redis Integration (Day 3-4)

#### 3.1 Token Blacklist (TDD)
```rust
#[tokio::test]
async fn test_blacklist_token() {
    let redis = setup_test_redis().await;
    blacklist_token(&redis, "token123", 300).await.unwrap();
    assert!(is_token_blacklisted(&redis, "token123").await.unwrap());
}
```

#### 3.2 Rate Limiting (TDD)
```rust
#[tokio::test]
async fn test_rate_limiting() {
    let redis = setup_test_redis().await;
    let ip = "127.0.0.1";

    // Should allow first 5 attempts
    for _ in 0..5 {
        assert!(!is_rate_limited(&redis, ip).await.unwrap());
        increment_rate_limit(&redis, ip).await.unwrap();
    }

    // Should block 6th attempt
    assert!(is_rate_limited(&redis, ip).await.unwrap());
}
```

### Phase 4: API Handlers (Day 4-5)
**Approach**: Integration tests first, then implementation

#### 4.1 Register Endpoint (TDD)
**RED**: Write integration test
```rust
#[tokio::test]
async fn test_register_success() {
    let app = setup_test_app().await;
    let response = app
        .post("/api/auth/register")
        .json(&json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123"
        }))
        .await;

    assert_eq!(response.status(), 201);
    let body: RegisterResponse = response.json().await;
    assert_eq!(body.user.username, "testuser");
    assert!(body.access_token.len() > 0);
}
```

**GREEN**: Implement handler
**REFACTOR**: Extract validation, improve error responses

#### 4.2 Login Endpoint (TDD)
Same pattern: Test → Implement → Refactor

#### 4.3 Refresh Endpoint (TDD)
Same pattern with token rotation testing

#### 4.4 Logout Endpoint (TDD)
Test blacklist integration

#### 4.5 Me Endpoint (TDD)
Test middleware integration

### Phase 5: Middleware (Day 5-6)

#### 5.1 Auth Middleware (TDD)
```rust
#[tokio::test]
async fn test_auth_middleware_valid_token() {
    let app = setup_test_app().await;
    let token = create_test_token();

    let response = app
        .get("/api/auth/me")
        .header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_auth_middleware_invalid_token() {
    let app = setup_test_app().await;

    let response = app
        .get("/api/auth/me")
        .header("Authorization", "Bearer invalid")
        .await;

    assert_eq!(response.status(), 401);
}
```

### Phase 6: Frontend Implementation (Day 7-8)

#### 6.1 Generate Types
```bash
make generate-openapi
make generate-types
```

#### 6.2 API Client
Implement methods matching backend endpoints

#### 6.3 Auth Context
React Query integration for auth state

#### 6.4 Forms
Login and register with validation

#### 6.5 Protected Routes
Next.js middleware for route protection

### Phase 7: E2E Testing (Day 8-9)

Use Playwright to test:
- Complete registration flow
- Login → protected route → logout
- Token refresh
- Rate limiting

### Phase 8: Security & Performance (Day 9-10)

- Security audit
- Performance benchmarks
- Documentation updates

## TDD Best Practices

### 1. Test Structure
```rust
// Arrange
let user = create_test_user();
let password = "test123";

// Act
let result = authenticate(&user, password);

// Assert
assert!(result.is_ok());
```

### 2. Test Helpers
Create helper functions for:
- Database setup/teardown
- Test user creation
- Token generation
- Redis cleanup

### 3. Coverage Target
- Auth service: >90% (critical security code)
- Handlers: >80%
- Integration tests: All endpoints

### 4. Test Naming
```rust
#[test]
fn test_{function}_{scenario}_{expected_outcome}() {
    // Example: test_login_with_invalid_credentials_returns_unauthorized
}
```

## Quick Start Commands

```bash
# Run tests
cd backend
cargo test

# Run specific test
cargo test test_hash_password

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration

# Check test coverage
cargo tarpaulin --out Html
# Open backend/coverage/index.html
```

## Common Pitfalls to Avoid

1. **Don't skip tests** - Write tests first, even if it feels slow
2. **Don't test implementation details** - Test behavior, not internals
3. **Don't share mutable state** - Each test should be isolated
4. **Don't forget cleanup** - Clear Redis/DB between tests
5. **Don't hardcode secrets** - Use test environment variables

## Token Limit Considerations

Given Claude's token constraints, the full implementation requires breaking it into smaller sessions:

**Session 1**: Database + Auth Service Core
**Session 2**: Redis + Handlers (Register, Login)
**Session 3**: Handlers (Refresh, Logout, Me) + Middleware
**Session 4**: Frontend API Client + Context
**Session 5**: Frontend Forms + Routes
**Session 6**: E2E Tests + Polish

Each session should follow TDD and include test updates in tasks.md.

## Next Steps

1. Start with Phase 1 (Database migrations)
2. Follow TDD cycle: RED → GREEN → REFACTOR
3. Update tasks.md as you complete each item
4. Commit frequently with descriptive messages
5. Run full test suite before moving to next phase

## Resources

- OpenSpec proposal: `openspec/changes/add-username-password-auth/`
- Design decisions: `design.md`
- Task checklist: `tasks.md`
- Specification: `specs/authentication/spec.md`

## Getting Help

If you encounter issues:
1. Check the design document for architectural decisions
2. Review the spec for requirement details
3. Look at task descriptions for implementation hints
4. Ask Claude for specific component help (e.g., "implement JWT service with TDD")
