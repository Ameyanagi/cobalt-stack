# Authentication System Design

## Context

The Cobalt Stack requires user authentication to support protected resources, user-specific data, and account management. This design implements a production-ready JWT-based authentication system with username/password login as the initial authentication method, while providing an extensible architecture for future OAuth providers (Google, EntraID, GitHub).

**Stakeholders:**
- Users: Need secure, seamless login experience
- Developers: Need maintainable, testable auth implementation
- Security: Need industry-standard security practices

**Constraints:**
- Must use existing PostgreSQL and Redis infrastructure
- Must integrate with Axum backend and Next.js 16 frontend
- Must support future OAuth without breaking changes
- Must follow OWASP security guidelines

## Goals / Non-Goals

### Goals
- Implement secure username/password authentication with JWT
- Support access tokens (short-lived) and refresh tokens (long-lived) with rotation
- Enable immediate logout via token blacklist
- Prevent brute force attacks via rate limiting
- Provide extensible architecture for future OAuth providers
- Achieve >90% test coverage for auth service code
- Meet performance targets (<100ms p99 for login)

### Non-Goals
- OAuth implementation (deferred to Phase 2)
- Email verification (deferred to future enhancement)
- Password reset flow (deferred to future enhancement)
- Multi-factor authentication (deferred to future enhancement)
- Session management UI (admin view of active sessions)
- Account lockout after repeated failures (use rate limiting instead)

## Decisions

### Decision 1: JWT with Access + Refresh Token Pattern

**Choice:** Implement dual-token system with short-lived access tokens and long-lived refresh tokens.

**Rationale:**
- Access tokens (15-30 min): Stateless, fast validation, no DB query per request
- Refresh tokens (7-30 days): Provides persistence without long-term risk of access token compromise
- Token rotation: Mitigates refresh token theft by invalidating old token on use

**Alternatives Considered:**
1. **Session-based auth (cookies + server-side session store)**
   - Rejected: Requires DB/Redis query on every request (slower), harder to scale across microservices

2. **Single long-lived JWT**
   - Rejected: No way to revoke compromised tokens without complex blacklisting infrastructure

3. **Short-lived JWT only (no refresh)**
   - Rejected: Poor UX requiring frequent logins

### Decision 2: Argon2id for Password Hashing

**Choice:** Use Argon2id algorithm with parameters: memory=19MB, iterations=2, parallelism=1.

**Rationale:**
- Winner of Password Hashing Competition (PHC) in 2015
- Resistant to GPU cracking attacks
- OWASP recommended over bcrypt for new applications
- Configurable memory/CPU trade-off for future-proofing

**Alternatives Considered:**
1. **bcrypt**
   - Rejected: Vulnerable to GPU attacks, limited to 72-byte passwords

2. **PBKDF2**
   - Rejected: Lower memory usage makes GPU attacks easier

### Decision 3: Redis for Blacklist + Rate Limiting

**Choice:** Use Redis for token blacklist and login rate limiting, not for session storage.

**Rationale:**
- Blacklist: Enables immediate logout by invalidating access tokens
- Rate limiting: Fast, distributed-friendly counter with TTL
- Already in stack: No new infrastructure needed
- Ephemeral data: Perfect fit for Redis vs PostgreSQL

**Alternatives Considered:**
1. **Database-only (no Redis)**
   - Rejected: Slower rate limiting, adds DB load for blacklist checks

2. **In-memory application cache**
   - Rejected: Doesn't work across multiple backend instances

### Decision 4: HttpOnly Cookies for Refresh Tokens

**Choice:** Store refresh tokens in HttpOnly, Secure, SameSite cookies; access tokens in memory only.

**Rationale:**
- HttpOnly: Protects against XSS attacks (JavaScript can't access)
- Secure: HTTPS-only transmission
- SameSite: CSRF protection
- Memory-only access tokens: Cleared on browser close, minimal XSS risk

**Alternatives Considered:**
1. **localStorage for both tokens**
   - Rejected: Vulnerable to XSS attacks

2. **Both in HttpOnly cookies**
   - Rejected: Complicates CORS, every request requires cookie

### Decision 5: CORS with Credentials

**Choice:** Enable CORS credentials to allow cross-origin cookie transmission.

**Configuration:**
```rust
CorsLayer::new()
    .allow_origin(Origin::exact("http://localhost:2727".parse()?))
    .allow_credentials(true)
    .allow_headers([AUTHORIZATION, CONTENT_TYPE])
    .allow_methods([GET, POST, PUT, DELETE, OPTIONS])
```

**Rationale:**
- Required for HttpOnly cookie access from frontend
- Restricts origins to configured frontend URLs only
- Prevents CSRF via SameSite cookie attribute

### Decision 6: Database Schema for OAuth Extensibility

**Choice:** Design schema with nullable password_hash and separate oauth_accounts table.

**Schema:**
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NULL,  -- NULL for OAuth-only users
    email_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE oauth_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,  -- 'google', 'github', 'microsoft'
    provider_user_id VARCHAR(255) NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(provider, provider_user_id)
);
```

**Rationale:**
- Nullable password_hash allows OAuth-only users (future)
- Separate oauth_accounts table allows multiple providers per user
- No breaking changes when adding OAuth
- Foreign key cascades cleanup user data on account deletion

### Decision 7: Rate Limiting Strategy

**Choice:** Implement IP-based rate limiting: 5 attempts per 15 minutes per IP.

**Implementation:**
```rust
// Redis key: "ratelimit:login:{ip}"
// Increment counter, set TTL on first attempt
// Block if counter exceeds 5
```

**Rationale:**
- Prevents brute force password attacks
- Doesn't require user account to exist (IP-based)
- Automatic reset via TTL (no cleanup job needed)
- False positives acceptable (shared IPs can request unblock)

**Alternatives Considered:**
1. **Account-based rate limiting**
   - Rejected: Allows enumeration attack (testing many accounts from one IP)

2. **No rate limiting**
   - Rejected: Security requirement for password-based auth

### Decision 8: Constant-Time Password Comparison

**Choice:** Use Argon2's built-in constant-time verification.

**Rationale:**
- Prevents timing attacks to discover valid usernames
- Built into argon2 crate
- Performance impact negligible

### Decision 9: JWT Algorithm Selection

**Choice:**
- Development: HS256 (symmetric key from JWT_SECRET)
- Production: RS256 (asymmetric keys) - documented, not implemented in Phase 1

**Rationale:**
- HS256: Simple, single secret, sufficient for monolith
- RS256: Better for distributed systems (public key validation)
- Start simple, upgrade path documented

**Phase 1 Implementation:** HS256 only
**Phase 2:** Add RS256 support with public/private key pair

## Risks / Trade-offs

### Risk 1: Token Theft

**Risk:** Attacker obtains access or refresh token.

**Mitigation:**
- Access token: Short 15-30 min lifetime limits damage window
- Refresh token: HttpOnly cookies prevent JavaScript access
- Blacklist: Allows immediate revocation on logout
- Token rotation: Invalidates old refresh tokens
- HTTPS: Prevents man-in-the-middle interception (production)

### Risk 2: Brute Force Attacks

**Risk:** Attacker attempts to guess passwords.

**Mitigation:**
- Rate limiting: 5 attempts per 15 minutes per IP
- Argon2id: Slow hash function (intentionally CPU/memory expensive)
- Password complexity: Minimum 8 characters enforced
- Monitoring: Log failed login attempts for security analysis

### Risk 3: Password Database Leak

**Risk:** Database compromise exposes password hashes.

**Mitigation:**
- Argon2id: GPU-resistant hashing algorithm
- Strong parameters: 19MB memory, 2 iterations
- No plaintext: Passwords never stored unhashed
- Salting: Automatic per-password salt in Argon2id

### Risk 4: XSS Attacks

**Risk:** Malicious JavaScript steals tokens.

**Mitigation:**
- Refresh tokens: HttpOnly cookies inaccessible to JavaScript
- Access tokens: Memory-only storage (React state), cleared on page refresh
- CSP headers: Document for production deployment
- Input sanitization: Validate/escape user input

### Risk 5: CSRF Attacks

**Risk:** Malicious site triggers authenticated requests.

**Mitigation:**
- SameSite cookies: Prevents cross-site cookie transmission
- CORS origin restrictions: Only configured frontend URLs allowed
- Credentials mode: Explicitly required for cross-origin requests

### Risk 6: Session Fixation

**Risk:** Attacker sets user's session ID.

**Mitigation:**
- Token rotation: New tokens issued on each refresh
- Cryptographically random: Unpredictable token generation
- Server-side validation: All tokens validated against database

## Performance Targets

### Latency Targets (p99)
- Login endpoint: <100ms
- Token refresh: <50ms
- Protected route auth check: <10ms

### Caching Strategy
- Cache user data in Redis after login (TTL: access token lifetime)
- Cache hit rate target: >80% for user lookups
- Expected query reduction: 80-90% vs no caching

### Database Optimization
- Indexes on username, email (unique constraint creates index)
- Index on refresh_tokens.user_id for user token lookup
- Index on refresh_tokens.token_hash for verification
- Index on refresh_tokens.expires_at for cleanup queries
- Composite index on (user_id, expires_at, revoked_at) for active token queries

### Connection Pooling
- PostgreSQL: SeaORM default (max: 10 connections)
- Redis: Connection pool (r2d2 or deadpool, max: 20 connections)
- Reuse connections across requests

## Migration Plan

### Phase 1: Username/Password Auth (Current)
1. Deploy database migrations (users, refresh_tokens, oauth_accounts)
2. Deploy backend with auth service, middleware, handlers
3. Deploy frontend with login/register forms, auth context
4. Update environment configuration
5. Run integration tests
6. Deploy to staging for security audit
7. Deploy to production

### Phase 2: OAuth Providers (Future)
1. Implement OAuth trait abstraction
2. Add Google OAuth provider
3. Add GitHub OAuth provider
4. Add EntraID OAuth provider
5. Update frontend with OAuth buttons
6. No database schema changes needed (oauth_accounts already exists)

### Rollback Plan
1. Database migrations are reversible (down migrations)
2. Deploy code rollback: redeploy previous version
3. Clear Redis blacklist: `FLUSHDB` (or let TTL expire)
4. Remove auth routes from router if needed

### Zero-Downtime Deployment
1. Database migrations run first (additive only)
2. Backend deployment (new auth routes added)
3. Frontend deployment (auth pages added)
4. No breaking changes to existing endpoints

## Open Questions

### Q1: Should we implement email verification in Phase 1?
**Status:** Deferred to future enhancement
**Rationale:** Not critical for MVP, can be added without breaking changes

### Q2: Should we use RS256 instead of HS256 in production?
**Status:** HS256 for Phase 1, RS256 documented for future
**Rationale:** HS256 simpler for monolith, RS256 better for microservices (future)

### Q3: What password complexity requirements beyond minimum length?
**Status:** Minimum 8 characters for Phase 1
**Rationale:** Balance security and UX, can add complexity rules later

### Q4: Should we implement password reset flow in Phase 1?
**Status:** Deferred to future enhancement
**Rationale:** Requires email infrastructure (not in scope for auth MVP)

### Q5: Should we track device/IP for refresh tokens?
**Status:** Optional, can add user_agent column later
**Rationale:** Useful for security monitoring, not critical for Phase 1

## Security Checklist

- [x] Passwords hashed with Argon2id (PHC winner)
- [x] Constant-time password comparison (timing attack prevention)
- [x] Rate limiting on login endpoint (brute force prevention)
- [x] HTTPS required in production (documented)
- [x] HttpOnly, Secure, SameSite cookies (XSS/CSRF protection)
- [x] CORS credentials restricted to configured origins
- [x] JWT expiration enforced (access: 15-30 min, refresh: 7-30 days)
- [x] Token rotation on refresh (replay attack mitigation)
- [x] Token blacklist for logout (immediate revocation)
- [x] Input validation (username, email, password)
- [x] Error messages don't leak information (username enumeration prevention)
- [x] Secrets in environment variables (not hardcoded)
- [x] SQL injection protection (SeaORM parameterized queries)
- [x] No plaintext passwords in logs or errors
