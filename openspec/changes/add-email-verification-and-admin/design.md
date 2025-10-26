# Email Verification and Admin Management - Design Document

## Context

The Cobalt Stack currently has a basic authentication system with JWT tokens, but lacks email verification and administrative capabilities. This document outlines the technical decisions for adding these features while maintaining security, scalability, and developer experience.

### Current State
- Users can register and login with username/password
- JWT-based authentication (access + refresh tokens)
- `email_verified` field exists but is unused
- No role-based access control
- No admin functionality

### Stakeholders
- **End Users**: Need email verification for account security
- **Administrators**: Need tools to manage users and view statistics
- **Developers**: Need maintainable, testable, documented code
- **Template Users**: Need simple, production-ready patterns to customize

### Constraints
- Must work with existing JWT auth system
- Must not break existing API contracts
- Database migrations must be reversible
- Email sending must work offline (mock mode) for development
- Admin features must be secure by default

## Goals / Non-Goals

### Goals
1. **Email Verification**: Implement complete verification flow with secure tokens
2. **Role-Based Access Control**: Add simple, extensible RBAC system
3. **Admin Dashboard**: Provide user management UI for administrators
4. **Developer Experience**: Mock email by default, easy SMTP configuration
5. **Security**: Prevent common attacks (token reuse, timing, privilege escalation)
6. **Auditability**: Track important admin actions
7. **Template-Ready**: Code should be extractable to cookiecutter template

### Non-Goals
1. Multi-factor authentication (future enhancement)
2. Complex permission system beyond roles (RBAC with permissions comes later)
3. Email template customization (use simple templates initially)
4. Background job queue (send emails synchronously for now)
5. Advanced admin features (bulk operations, complex analytics)

## Decisions

### Decision 1: Email Verification Token Storage

**Choice**: Hash tokens with SHA-256 before database storage, send unhashed to user.

**Rationale**:
- Tokens in database can't be used directly if database is compromised
- SHA-256 is fast enough for this use case (not a password hash)
- Similar pattern to refresh token storage (consistency)

**Alternatives Considered**:
1. Store plaintext tokens - **Rejected**: Security risk if database leaks
2. Use JWT for verification - **Rejected**: Adds complexity, can't revoke individual tokens easily
3. Use Argon2 like passwords - **Rejected**: Overkill for single-use tokens, performance cost

**Implementation**:
```rust
// Generate token
let token = generate_random_bytes(32);  // cryptographically secure
let token_hash = sha256(token);

// Store in DB
email_verifications { token_hash, user_id, expires_at }

// Send to user
email_link = format!("{}/verify-email?token={}", frontend_url, hex(token))
```

### Decision 2: Email Verification Expiry

**Choice**: 24-hour expiration with resend capability.

**Rationale**:
- Balance security (short window) and UX (reasonable time to check email)
- Industry standard (most services use 24-48 hours)
- Resend feature allows users to recover from missed emails

**Alternatives Considered**:
1. 1-hour expiry - **Rejected**: Too aggressive, bad UX for users who don't check email often
2. 7-day expiry - **Rejected**: Too permissive, higher risk window
3. No expiry - **Rejected**: Tokens would work forever, security risk

**Configuration**:
```env
EMAIL_VERIFICATION_EXPIRY=86400  # 24 hours in seconds
```

### Decision 3: Role Enum Design

**Choice**: Database enum with values 'user' and 'admin', default 'user'.

**Rationale**:
- Type safety at database level
- Clear, self-documenting
- Easy to add more roles later (moderator, support, etc.)
- Enforces valid role values

**Alternatives Considered**:
1. Boolean `is_admin` flag - **Rejected**: Not extensible to 3+ roles
2. String column with validation - **Rejected**: Less type-safe, allows invalid values
3. Separate permissions table - **Rejected**: Over-engineering for initial version

**Migration**:
```sql
CREATE TYPE user_role AS ENUM ('user', 'admin');
ALTER TABLE users ADD COLUMN role user_role NOT NULL DEFAULT 'user';
CREATE INDEX idx_users_role ON users(role);
```

### Decision 4: Admin Middleware Implementation

**Choice**: Separate admin middleware after auth middleware in Axum router.

**Rationale**:
- Separation of concerns (authentication vs authorization)
- Reusable for different admin route groups
- Clear error messages (401 for auth, 403 for authz)
- Easy to test independently

**Alternatives Considered**:
1. Combined auth+admin middleware - **Rejected**: Less flexible, harder to test
2. Route-level checks - **Rejected**: Repetitive code, easy to forget
3. Attribute macros - **Rejected**: Adds proc-macro complexity

**Implementation**:
```rust
// Router structure
Router::new()
    .route("/api/admin/users", get(list_users))
    .layer(axum_middleware::from_fn(admin_middleware))  // Check role
    .layer(axum_middleware::from_fn_with_state(jwt_config, auth_middleware))  // Check JWT
```

### Decision 5: Email Sending Architecture

**Choice**: Synchronous email sending with mock/SMTP toggle via environment variable.

**Rationale**:
- Simplicity: No background job queue needed initially
- Development: Mock mode logs to console, no SMTP required
- Production-ready: SMTP configuration available when needed
- Acceptable latency: Email sending is fast enough for registration flow

**Alternatives Considered**:
1. Async background jobs - **Rejected**: Adds complexity (Redis queue, worker process), over-engineering for MVP
2. Webhook to external service - **Rejected**: Adds external dependency, costs money
3. Always mock, no real emails - **Rejected**: Not production-ready

**Configuration**:
```env
EMAIL_MOCK=true  # Default: log to console
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=user@example.com
SMTP_PASSWORD=secret
SMTP_FROM=noreply@example.com
```

### Decision 6: User Disable (Soft Delete) Strategy

**Choice**: Add `disabled_at` timestamp column, check in auth middleware.

**Rationale**:
- Reversible: Can re-enable accounts
- Audit trail: Know when user was disabled
- Token invalidation: Existing tokens stop working immediately
- Data retention: Maintain user data for analytics/legal

**Alternatives Considered**:
1. Hard delete users - **Rejected**: Irreversible, loses data, breaks foreign keys
2. `is_disabled` boolean - **Rejected**: Loses temporal information
3. Separate disabled_users table - **Rejected**: Adds JOIN complexity

**Implementation**:
```rust
// In auth middleware
if user.disabled_at.is_some() {
    return Err(AuthError::AccountDisabled);
}

// Disable user
UPDATE users SET disabled_at = NOW() WHERE id = $1;
```

### Decision 7: Admin Seed Script Approach

**Choice**: Cargo binary in backend/src/bin/seed_admin.rs

**Rationale**:
- Self-contained: No external dependencies
- Type-safe: Uses same models as main application
- Database access: Direct access to SeaORM
- Easy to run: `cargo run --bin seed_admin` or `make seed-admin`

**Alternatives Considered**:
1. SQL script - **Rejected**: Need to hash password outside of Rust
2. Migration with data - **Rejected**: Migrations shouldn't contain business logic
3. API endpoint - **Rejected**: Security risk, chicken-and-egg problem

**Implementation**:
```rust
// backend/src/bin/seed_admin.rs
#[tokio::main]
async fn main() -> Result<()> {
    let db = connect_database().await?;

    // Check if admin exists
    if Admin::exists(&db).await? {
        println!("Admin user already exists");
        return Ok(());
    }

    // Generate random password
    let password = generate_random_password();
    let hash = hash_password(&password)?;

    // Create admin user
    let admin = users::ActiveModel {
        username: Set("admin".to_string()),
        email: Set("admin@example.com".to_string()),
        password_hash: Set(Some(hash)),
        email_verified: Set(true),
        role: Set(UserRole::Admin),
        ..Default::default()
    };
    admin.insert(&db).await?;

    println!("Admin created!");
    println!("Username: admin");
    println!("Password: {}", password);
    println!("⚠️  Change password after first login!");

    Ok(())
}
```

### Decision 8: Admin Dashboard UI Framework

**Choice**: Use shadcn/ui data-table component with TanStack React Table.

**Rationale**:
- Consistent with existing UI components
- Feature-rich: sorting, filtering, pagination out of box
- Type-safe: TypeScript integration
- Customizable: Can add custom columns/actions

**Alternatives Considered**:
1. Build table from scratch - **Rejected**: Reinventing wheel, time-consuming
2. Use Material-UI or Ant Design - **Rejected**: Different design system, larger bundle
3. Server-side rendering only - **Rejected**: Less interactive UX

**Implementation**:
```tsx
// frontend/src/app/admin/users/page.tsx
import { DataTable } from "@/components/ui/data-table"

const columns = [
  { accessorKey: "username", header: "Username" },
  { accessorKey: "email", header: "Email" },
  { accessorKey: "email_verified", header: "Verified", cell: StatusBadge },
  { accessorKey: "role", header: "Role", cell: RoleBadge },
  { accessorKey: "created_at", header: "Created", cell: DateCell },
  { id: "actions", cell: ActionsMenu }
]

export default function AdminUsersPage() {
  const { data, isLoading } = useQuery(['admin', 'users'], fetchUsers)
  return <DataTable columns={columns} data={data?.users ?? []} />
}
```

### Decision 9: Email Verification Rate Limiting

**Choice**: 3 resend requests per hour per user, tracked in Redis.

**Rationale**:
- Prevent abuse: Can't spam verification emails
- Reasonable limit: Legitimate users won't hit this
- Redis storage: Fast, TTL-based expiration

**Alternatives Considered**:
1. No rate limit - **Rejected**: Vulnerable to abuse
2. Database tracking - **Rejected**: Slower, requires cleanup job
3. 1 per hour - **Rejected**: Too restrictive for legitimate retries

**Implementation**:
```rust
let key = format!("verify_email_limit:{}",user_id);
let count: i32 = redis.get(&key).unwrap_or(0);

if count >= 3 {
    return Err(Error::TooManyRequests);
}

redis.incr(&key);
redis.expire(&key, 3600);  // 1 hour TTL
```

### Decision 10: JWT Claims for Role

**Choice**: Include `role` in JWT access token claims.

**Rationale**:
- Performance: Avoid database lookup on every request
- Stateless: JWT contains all needed information
- Standard: Similar to including `email`, `username`

**Tradeoff**: Role changes require token refresh (acceptable for rare event).

**Implementation**:
```rust
#[derive(Serialize, Deserialize)]
struct Claims {
    sub: Uuid,           // user_id
    username: String,
    email: String,
    role: UserRole,      // ADDED
    iat: i64,
    exp: i64,
}
```

## Risks / Trade-offs

### Risk 1: Email Deliverability
**Risk**: Emails may go to spam or not deliver.
**Mitigation**:
- Use reputable SMTP provider (SendGrid, AWS SES)
- Implement SPF/DKIM/DMARC (deployment concern)
- Provide resend functionality
- Log all email attempts for debugging

### Risk 2: Last Admin Protection
**Risk**: Admin could accidentally disable/delete last admin account.
**Mitigation**:
- Check admin count before disable/delete
- Prevent self-disable
- Document manual SQL recovery process
- Consider audit logging for admin actions

### Risk 3: Token Enumeration Attack
**Risk**: Attacker could guess verification tokens.
**Mitigation**:
- Use cryptographically random 32-byte tokens (2^256 space)
- Rate limit verification attempts per IP
- Hash tokens before storage
- Tokens expire after 24 hours

### Risk 4: Role Change Requires Logout
**Trade-off**: If user is promoted to admin via SQL, they must logout/login to get new role in JWT.
**Acceptance**: Acceptable for rare admin promotion event. Document in README.
**Alternative**: Could add `GET /api/auth/refresh-claims` endpoint to re-issue token with current role.

### Risk 5: Synchronous Email Blocking
**Trade-off**: Email sending blocks HTTP response during registration.
**Impact**: ~100-500ms added latency to registration endpoint.
**Acceptance**: Acceptable for MVP, can add background jobs later if needed.
**Mitigation**: Set SMTP timeout to 5 seconds max.

### Risk 6: Database Migration Rollback
**Risk**: Rolling back migrations with data could lose information.
**Mitigation**:
- Separate schema changes from data changes
- Make `role` and `disabled_at` nullable first, then add defaults
- Test migrations in staging before production
- Document rollback procedures

## Migration Plan

### Phase 1: Database Migrations (Zero Downtime)
```sql
-- Migration 1: Add role enum and column (nullable initially)
CREATE TYPE user_role AS ENUM ('user', 'admin');
ALTER TABLE users ADD COLUMN role user_role;
UPDATE users SET role = 'user' WHERE role IS NULL;
ALTER TABLE users ALTER COLUMN role SET DEFAULT 'user';
ALTER TABLE users ALTER COLUMN role SET NOT NULL;
CREATE INDEX idx_users_role ON users(role);

-- Migration 2: Add disabled_at column
ALTER TABLE users ADD COLUMN disabled_at TIMESTAMP NULL;
CREATE INDEX idx_users_disabled ON users(disabled_at);

-- Migration 3: Add last_login_at column
ALTER TABLE users ADD COLUMN last_login_at TIMESTAMP NULL;

-- Migration 4: Create email_verifications table
CREATE TABLE email_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    verified_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_email_verifications_user_id ON email_verifications(user_id);
CREATE INDEX idx_email_verifications_token_hash ON email_verifications(token_hash);
CREATE INDEX idx_email_verifications_expires_at ON email_verifications(expires_at);
```

### Phase 2: Backend Deployment
1. Deploy backend with new code (backward compatible)
2. New endpoints are available but not yet used
3. Old auth flow continues to work
4. Run seed script to create admin user

### Phase 3: Frontend Deployment
1. Deploy frontend with verification and admin UI
2. New users get verification emails
3. Admin users can access /admin routes
4. Existing users continue normal operation

### Phase 4: Data Cleanup
1. Send verification emails to existing unverified users (optional)
2. Clean up expired verification tokens after 7 days

### Rollback Procedure
1. Revert frontend deployment
2. Revert backend deployment
3. Keep database migrations (adding columns is safe)
4. If needed, drop columns:
   ```sql
   ALTER TABLE users DROP COLUMN disabled_at;
   ALTER TABLE users DROP COLUMN role;
   DROP TABLE email_verifications;
   DROP TYPE user_role;
   ```

## Open Questions

1. **Email Template Design**: Should we support HTML emails or plain text only?
   - **Recommendation**: Start with plain text, add HTML templates later

2. **Admin Audit Logging**: Should we log all admin actions now or later?
   - **Recommendation**: Defer to future enhancement, add audit_logs table structure but don't implement yet

3. **Email Verification Requirement**: Should we block unverified users from any actions?
   - **Recommendation**: Allow unverified users to use the app, show warning banner only

4. **Multiple Email Verification**: Can users change email and re-verify?
   - **Recommendation**: Defer email change feature to future enhancement

5. **Admin Role Assignment UI**: Should admins be able to promote users via UI?
   - **Recommendation**: No, require manual SQL for security. Document process.

6. **Bulk Operations**: Should admins be able to disable multiple users at once?
   - **Recommendation**: Defer to future enhancement, implement one-at-a-time first

## Success Criteria

- [ ] Email verification tokens are cryptographically secure (32 bytes)
- [ ] Tokens expire after 24 hours
- [ ] Tokens are single-use (verified_at timestamp prevents reuse)
- [ ] Admin middleware prevents non-admin access (returns 403)
- [ ] Disabled users cannot login or use existing tokens
- [ ] Admin seed script creates functional admin account
- [ ] Email mock mode works offline (no SMTP required)
- [ ] SMTP mode sends real emails when configured
- [ ] All endpoints have OpenAPI documentation
- [ ] All features have integration tests
- [ ] Frontend admin dashboard is responsive and accessible
- [ ] Zero breaking changes to existing API contracts

## Future Enhancements

1. **Background Email Queue**: Move to async email sending with Redis queue
2. **Advanced Admin Features**: Bulk operations, export users, complex filters
3. **Audit Logging**: Complete audit trail of all admin actions
4. **Email Change Flow**: Allow users to change and re-verify email
5. **Email Templates**: HTML email templates with branding
6. **Admin Roles**: Super admin, moderator, support roles with different permissions
7. **Two-Factor Authentication**: TOTP-based 2FA for admin accounts
8. **IP Whitelisting**: Restrict admin access to specific IP ranges
9. **Session Management**: View and revoke active sessions per user
10. **Notification System**: Email users when account is disabled/enabled

## References

- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [JWT Best Practices](https://datatracker.ietf.org/doc/html/rfc8725)
- [Email Verification Best Practices](https://postmarkapp.com/blog/email-verification-best-practices)
- SeaORM Documentation: Migrations, Enums
- Axum Middleware Documentation
- shadcn/ui Data Table Example
