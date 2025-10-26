# Email Verification and Admin Management

## Why

User accounts currently have an `email_verified` field that is unused, and there's no way to verify user email addresses or manage users administratively. Email verification is essential for:
- Confirming user ownership of email addresses
- Enabling password reset functionality
- Preventing spam accounts
- Meeting compliance requirements

Admin functionality is needed for:
- Managing user accounts (view, disable, delete)
- Viewing system statistics
- Monitoring authentication activity
- Supporting users with account issues

## What Changes

### Email Verification
- Email verification token generation and storage
- Send verification email on registration (mock implementation, SMTP-ready)
- Email verification endpoint
- Resend verification email endpoint
- Mark `email_verified` field as `true` upon successful verification
- Block certain actions for unverified users (configurable)
- Frontend verification page and flow

### Admin Management
- Add `role` field to User model (enum: `user`, `admin`)
- Admin middleware for protecting admin-only routes
- Admin dashboard endpoints:
  - List all users with pagination and filtering
  - View user details
  - Disable/enable user accounts
  - View authentication statistics
- Frontend admin dashboard UI
- Seed script to create initial admin user

## Impact

### Affected Specs
- **authentication** - MODIFIED to add email verification requirements
- **authorization** - ADDED for role-based access control (RBAC)

### Affected Code
- **Backend:**
  - `backend/src/models/user.rs` - Add role enum
  - `backend/src/models/email_verification.rs` - New model
  - `backend/migration/` - New migrations for role and email_verification table
  - `backend/src/handlers/auth.rs` - Add verification endpoints
  - `backend/src/handlers/admin.rs` - New admin endpoints
  - `backend/src/middleware/admin.rs` - New admin authorization middleware
  - `backend/src/services/email.rs` - New email service (mock + SMTP)
  - `backend/src/utils/token.rs` - Add verification token generation

- **Frontend:**
  - `frontend/src/app/verify-email/page.tsx` - Email verification page
  - `frontend/src/app/admin/page.tsx` - Admin dashboard
  - `frontend/src/app/admin/users/page.tsx` - User management page
  - `frontend/src/components/admin/` - Admin UI components
  - `frontend/src/components/ui/` - New UI components (data table, badge)

- **Database:**
  - Migration to add `role` enum and column to users table
  - Migration to create `email_verifications` table

### Breaking Changes
None - all changes are additive.

### Migration Path
1. Run database migration to add `role` column (defaults to 'user')
2. Run database migration to create `email_verifications` table
3. Manually promote first user to admin via SQL or seed script
4. Deploy backend with new endpoints
5. Deploy frontend with admin UI

## Dependencies
- Email sending: Will use mock implementation initially, with SMTP configuration ready
- No new external dependencies required

## Security Considerations
- Admin routes MUST be protected by role-based middleware
- Email verification tokens MUST be cryptographically secure
- Email verification tokens MUST expire after 24 hours
- Email verification tokens MUST be single-use
- Admin actions MUST be logged for audit trail
- User disable/delete MUST be reversible (soft delete)

## Performance Considerations
- Email sending should be async/background job (future enhancement)
- Admin user list pagination MUST be implemented
- Database indexes on role and email_verified fields

## Testing Requirements
- Unit tests for email verification token generation
- Integration tests for verification flow
- Integration tests for admin endpoints
- Authorization tests for admin middleware
- Frontend E2E tests for verification and admin flows
