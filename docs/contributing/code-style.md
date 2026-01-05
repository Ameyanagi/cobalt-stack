# Code Style Guide

Comprehensive code style guidelines for Cobalt Stack contributors.

## Table of Contents

- [Rust Style Guide](#rust-style-guide)
- [TypeScript Style Guide](#typescript-style-guide)
- [General Principles](#general-principles)
- [Naming Conventions](#naming-conventions)
- [Code Organization](#code-organization)
- [Documentation Standards](#documentation-standards)

## Rust Style Guide

### Formatting

Use `rustfmt` with the default configuration:

```bash
cargo fmt
```

### Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Crates | snake_case | `cobalt_stack` |
| Modules | snake_case | `auth_service` |
| Types | PascalCase | `UserResponse` |
| Traits | PascalCase | `Authenticable` |
| Enum Variants | PascalCase | `UserRole::Admin` |
| Functions | snake_case | `create_user` |
| Variables | snake_case | `access_token` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_RETRY_COUNT` |
| Lifetimes | lowercase | `'a`, `'db` |

### Code Structure

#### Module Organization

```rust
// External imports first
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Internal crate imports
use crate::models::users;
use crate::services::auth::{AuthError, JwtConfig};

// Module-level items
const MAX_PAGE_SIZE: u64 = 100;

// Type definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct UserRequest {
    pub username: String,
}

// Functions
pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<UserRequest>,
) -> Result<Json<UserResponse>, AuthError> {
    // Implementation
}
```

#### Function Organization

```rust
pub async fn complex_operation() -> Result<()> {
    // 1. Validation
    validate_input()?;

    // 2. Data retrieval
    let data = fetch_data().await?;

    // 3. Business logic
    let result = process_data(data)?;

    // 4. Side effects (database, external APIs)
    save_result(result).await?;

    // 5. Return
    Ok(())
}
```

### Error Handling

Use custom error types with `thiserror`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    DatabaseError(String),
}

// Use Result type alias for cleaner signatures
pub type Result<T> = std::result::Result<T, AuthError>;

pub fn authenticate(username: &str) -> Result<User> {
    let user = find_user(username)?;
    Ok(user)
}
```

### Documentation

Use Rust doc comments with examples:

```rust
/// Authenticates a user with username and password.
///
/// # Arguments
///
/// * `username` - The user's unique username
/// * `password` - The user's plain text password
///
/// # Returns
///
/// Returns `Ok(User)` if authentication succeeds, or an error if:
/// - User does not exist
/// - Password is incorrect
/// - Database connection fails
///
/// # Examples
///
/// ```no_run
/// use cobalt_stack::services::auth::authenticate;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let user = authenticate("alice", "password123").await?;
/// println!("Authenticated: {}", user.username);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`AuthError::InvalidCredentials`] if authentication fails.
pub async fn authenticate(username: &str, password: &str) -> Result<User> {
    // Implementation
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("alice", "alice@example.com");
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");
    }

    #[tokio::test]
    async fn test_user_authentication() {
        let result = authenticate("alice", "password").await;
        assert!(result.is_ok());
    }

    #[test]
    #[should_panic(expected = "invalid username")]
    fn test_invalid_username() {
        User::new("", "email@example.com");
    }
}
```

### Best Practices

#### Use Type Safety

```rust
// Good: Use newtype pattern for domain-specific types
pub struct UserId(Uuid);
pub struct Email(String);

// Bad: Use generic types everywhere
pub fn get_user(id: String) -> User { /* ... */ }
```

#### Prefer Iterators Over Loops

```rust
// Good
let usernames: Vec<String> = users
    .iter()
    .filter(|u| u.active)
    .map(|u| u.username.clone())
    .collect();

// Avoid
let mut usernames = Vec::new();
for user in &users {
    if user.active {
        usernames.push(user.username.clone());
    }
}
```

#### Use `?` for Error Propagation

```rust
// Good
pub fn process_user(id: Uuid) -> Result<User> {
    let user = find_user(id)?;
    validate_user(&user)?;
    Ok(user)
}

// Avoid
pub fn process_user(id: Uuid) -> Result<User> {
    match find_user(id) {
        Ok(user) => match validate_user(&user) {
            Ok(_) => Ok(user),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}
```

#### Avoid Unnecessary Clones

```rust
// Good: Use references
pub fn display_user(user: &User) {
    println!("{}", user.username);
}

// Bad: Unnecessary clone
pub fn display_user(user: User) {
    println!("{}", user.username);
}
```

---

## TypeScript Style Guide

### Formatting

Use Prettier with ESLint:

```bash
npm run lint
npm run format
```

### Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Variables | camelCase | `accessToken` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_RETRY_COUNT` |
| Functions | camelCase | `createUser` |
| Classes | PascalCase | `UserService` |
| Interfaces | PascalCase | `UserResponse` |
| Type Aliases | PascalCase | `UserId` |
| Enums | PascalCase | `UserRole` |
| Enum Members | PascalCase | `UserRole.Admin` |
| Files | kebab-case | `user-service.ts` |
| Components | PascalCase | `UserProfile.tsx` |

### TypeScript Features

#### Use Strict Type Checking

```typescript
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true
  }
}
```

#### Define Clear Interfaces

```typescript
// Good: Clear interface definitions
interface UserResponse {
  id: string;
  username: string;
  email: string;
  emailVerified: boolean;
  role: UserRole;
}

interface LoginRequest {
  username: string;
  password: string;
}

// Bad: Using any
function processUser(user: any) {
  console.log(user.username);
}
```

#### Use Type Guards

```typescript
// Type guard function
function isUserResponse(obj: unknown): obj is UserResponse {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    'id' in obj &&
    'username' in obj
  );
}

// Usage
if (isUserResponse(response)) {
  console.log(response.username); // TypeScript knows the type
}
```

### Code Organization

#### File Structure

```typescript
// 1. Imports (external first, then internal)
import React, { useState, useEffect } from 'react';
import { useRouter } from 'next/router';

import { api } from '@/lib/api';
import { UserResponse } from '@/types';

// 2. Type definitions
interface Props {
  userId: string;
}

// 3. Constants
const MAX_RETRIES = 3;

// 4. Component/Function
export function UserProfile({ userId }: Props) {
  // Implementation
}
```

#### Function Organization

```typescript
async function complexOperation(): Promise<Result> {
  // 1. Type validation
  if (!isValidInput(input)) {
    throw new Error('Invalid input');
  }

  // 2. Data fetching
  const data = await fetchData();

  // 3. Business logic
  const result = processData(data);

  // 4. Side effects
  await saveResult(result);

  // 5. Return
  return result;
}
```

### React Components

#### Functional Components with TypeScript

```typescript
import React from 'react';

interface UserProfileProps {
  user: UserResponse;
  onUpdate?: (user: UserResponse) => void;
}

export function UserProfile({ user, onUpdate }: UserProfileProps) {
  const [editing, setEditing] = React.useState(false);

  const handleSubmit = async (data: UpdateUserRequest) => {
    const updated = await api.updateUser(user.id, data);
    onUpdate?.(updated);
  };

  return (
    <div>
      <h1>{user.username}</h1>
      {editing ? (
        <UserEditForm user={user} onSubmit={handleSubmit} />
      ) : (
        <UserDetails user={user} />
      )}
    </div>
  );
}
```

#### Custom Hooks

```typescript
import { useState, useEffect } from 'react';

interface UseUserResult {
  user: UserResponse | null;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

export function useUser(userId: string): UseUserResult {
  const [user, setUser] = useState<UserResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchUser = async () => {
    try {
      setLoading(true);
      const data = await api.getUser(userId);
      setUser(data);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchUser();
  }, [userId]);

  return { user, loading, error, refetch: fetchUser };
}
```

### Error Handling

```typescript
// Define custom error classes
export class ApiError extends Error {
  constructor(
    message: string,
    public statusCode: number,
    public response?: unknown
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

// Use try-catch with typed errors
async function fetchUser(id: string): Promise<UserResponse> {
  try {
    const response = await fetch(`/api/users/${id}`);

    if (!response.ok) {
      throw new ApiError(
        'Failed to fetch user',
        response.status,
        await response.json()
      );
    }

    return await response.json();
  } catch (error) {
    if (error instanceof ApiError) {
      console.error(`API Error ${error.statusCode}:`, error.message);
    } else {
      console.error('Unexpected error:', error);
    }
    throw error;
  }
}
```

### Documentation

```typescript
/**
 * Authenticates a user with username and password.
 *
 * @param username - The user's unique username
 * @param password - The user's password
 * @returns A promise that resolves to the authentication response
 * @throws {ApiError} When authentication fails
 *
 * @example
 * ```typescript
 * const auth = await login('alice', 'password123');
 * console.log('Access token:', auth.access_token);
 * ```
 */
export async function login(
  username: string,
  password: string
): Promise<AuthResponse> {
  // Implementation
}
```

---

## General Principles

### SOLID Principles

#### Single Responsibility Principle

```rust
// Good: Each function has one responsibility
fn validate_email(email: &str) -> Result<()> { /* ... */ }
fn validate_password(password: &str) -> Result<()> { /* ... */ }
fn validate_username(username: &str) -> Result<()> { /* ... */ }

// Bad: Function does too much
fn validate_user(user: &User) -> Result<()> {
    // Validates email, password, username, checks database, sends email...
}
```

#### Open/Closed Principle

```typescript
// Good: Open for extension, closed for modification
interface AuthProvider {
  authenticate(credentials: unknown): Promise<User>;
}

class PasswordAuthProvider implements AuthProvider {
  async authenticate(credentials: LoginRequest): Promise<User> {
    // Password authentication
  }
}

class OAuthAuthProvider implements AuthProvider {
  async authenticate(credentials: OAuthRequest): Promise<User> {
    // OAuth authentication
  }
}
```

### DRY (Don't Repeat Yourself)

```rust
// Good: Reusable function
fn hash_token(token: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

// Use it multiple times
let access_hash = hash_token(&access_token);
let refresh_hash = hash_token(&refresh_token);
```

### KISS (Keep It Simple and Straightforward)

```typescript
// Good: Simple and clear
function isAuthenticated(user: User | null): boolean {
  return user !== null && user.emailVerified;
}

// Bad: Overly complex
function isAuthenticated(user: User | null): boolean {
  return Boolean(user && user.emailVerified === true);
}
```

---

## Naming Conventions

### General Rules

- Use descriptive names that reveal intent
- Avoid abbreviations unless widely known
- Use consistent terminology across the codebase
- Avoid magic numbers; use named constants

### Examples

```rust
// Good
const MAX_LOGIN_ATTEMPTS: u32 = 5;
const TOKEN_EXPIRY_SECONDS: i64 = 900;

// Bad
const MAX: u32 = 5;
const EXP: i64 = 900;
```

```typescript
// Good
const MAX_PAGE_SIZE = 100;
const API_TIMEOUT_MS = 5000;

// Bad
const max = 100;
const timeout = 5000;
```

---

## Code Organization

### Project Structure

```
backend/
├── src/
│   ├── handlers/         # HTTP request handlers
│   ├── services/         # Business logic
│   ├── models/           # Database models
│   ├── middleware/       # Axum middleware
│   ├── utils/            # Utility functions
│   └── main.rs           # Application entry point

frontend/
├── src/
│   ├── components/       # React components
│   ├── pages/            # Next.js pages
│   ├── lib/              # Utility libraries
│   ├── hooks/            # Custom React hooks
│   ├── types/            # TypeScript type definitions
│   └── styles/           # CSS/styling
```

### File Naming

- Use descriptive names that match content
- Group related files together
- Use consistent file extensions

---

## Documentation Standards

### When to Document

- Public APIs and interfaces
- Complex algorithms or business logic
- Non-obvious code decisions
- Configuration options
- Error conditions and handling

### What to Avoid

- Obvious comments that restate code
- Outdated documentation
- Implementation details in public docs
- Personal notes or TODOs in production code

### Documentation Tools

**Rust**:
```bash
cargo doc --no-deps --open
```

**TypeScript**:
```bash
npm run docs
```

---

## Tools and Automation

### Rust

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run tests
cargo test

# Check code without building
cargo check
```

### TypeScript

```bash
# Lint code
npm run lint

# Format code
npm run format

# Type check
npm run type-check

# Run tests
npm test
```

### Pre-commit Hooks

Set up pre-commit hooks to enforce style:

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install
```

---

## Code Review Checklist

When reviewing code, check for:

- [ ] Follows naming conventions
- [ ] Properly formatted (rustfmt/prettier)
- [ ] No linting errors (clippy/eslint)
- [ ] Adequate test coverage
- [ ] Clear documentation
- [ ] No code duplication
- [ ] Proper error handling
- [ ] Type safety (TypeScript)
- [ ] Security considerations

---

## Related Documentation

- [CONTRIBUTING.md](../../CONTRIBUTING.md)
- [Testing Requirements](./testing-requirements.md)
- [Pull Request Guidelines](./pull-requests.md)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Airbnb JavaScript Style Guide](https://github.com/airbnb/javascript)
