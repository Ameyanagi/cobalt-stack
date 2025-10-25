# Add Username/Password Authentication

## Why

The Cobalt Stack currently has no authentication system. Users cannot register, login, or access protected resources. This blocks critical features like user-specific data, authorization, and account management.

## What Changes

- **JWT-based authentication** with access tokens (15-30 min) and refresh tokens (7-30 days)
- **Username/password registration and login** with secure password hashing (Argon2id)
- **Token refresh mechanism** with automatic rotation for enhanced security
- **Session management** with immediate logout via Valkey token blacklist
- **Rate limiting** for login attempts (5 attempts per 15 minutes per IP)
- **Database schema** for users, refresh tokens, and future OAuth providers
- **Auth middleware** for protecting routes and extracting user identity
- **Frontend integration** with React Query for seamless auth state management

**Future-Ready Design:**
- Schema supports OAuth providers (Google, EntraID, GitHub) without breaking changes
- Nullable password_hash field enables OAuth-only user accounts
- Extensible provider abstraction for adding new OAuth methods

## Impact

### Affected Components
- **New Capability**: `authentication` (user-auth with JWT)
- **Backend**: New auth service, middleware, handlers, database models
- **Frontend**: Auth context, login/register forms, protected routes
- **Database**: 3 new tables (users, refresh_tokens, oauth_accounts)
- **Infrastructure**: Valkey for blacklist and rate limiting

### Breaking Changes
None - this is a new capability with no existing auth to replace.

### Security Considerations
- Argon2id password hashing (PHC winner, recommended by OWASP)
- HttpOnly cookies for refresh tokens (XSS protection)
- CORS credentials enabled for cookie handling
- Rate limiting to prevent brute force attacks
- Token blacklist for immediate session invalidation
- Constant-time password comparison (timing attack prevention)

### Dependencies
New Rust crates:
- `jsonwebtoken` - JWT encoding/decoding
- `argon2` - Password hashing
- `thiserror` - Domain error types with HTTP mapping
- `anyhow` - Application error propagation with context
- Valkey connection pool (Redis-compatible)

### Performance
- Target: <100ms p99 for login, <50ms p99 for token refresh
- Valkey caching reduces DB queries by 80-90% for authenticated requests
- Connection pooling for PostgreSQL and Valkey
