# Implementation Tasks

## 1. Database Schema
- [x] 1.1 Create migration for `role` enum type (user, admin)
- [x] 1.2 Add `role` column to users table with default 'user'
- [x] 1.3 Create `email_verifications` table with columns: id, user_id, token_hash, expires_at, verified_at, created_at
- [x] 1.4 Add indexes on role, email_verified, token_hash, expires_at
- [x] 1.5 Run migrations

## 2. Backend Models
- [x] 2.1 Add Role enum to user.rs (User, Admin)
- [x] 2.2 Add role field to User model
- [x] 2.3 Create EmailVerification model (backend/src/models/email_verification.rs)
- [x] 2.4 Add SeaORM entity for email_verifications table

## 3. Email Verification - Backend
- [x] 3.1 Create email service module (backend/src/services/email.rs)
- [x] 3.2 Implement mock email sender (logs to console)
- [x] 3.3 Add SMTP configuration (env vars, but use mock by default)
- [x] 3.4 Create verification token generation utility
- [x] 3.5 Add POST /api/auth/send-verification endpoint
- [x] 3.6 Add POST /api/auth/verify-email endpoint
- [x] 3.7 Modify registration to send verification email
- [x] 3.8 Add verification token validation logic
- [x] 3.9 Update email_verified field on successful verification
- [x] 3.10 Add OpenAPI annotations for verification endpoints

## 4. Email Verification - Frontend
- [x] 4.1 Create /verify-email page (frontend/src/app/verify-email/page.tsx)
- [x] 4.2 Add email verification UI component
- [x] 4.3 Add resend verification button to dashboard/profile
- [x] 4.4 Add unverified email warning banner component
- [x] 4.5 Update registration flow to show verification message
- [x] 4.6 Generate TypeScript types for verification endpoints

## 5. Admin Middleware & Authorization
- [x] 5.1 Create admin middleware (backend/src/middleware/admin.rs)
- [x] 5.2 Check user role in middleware (must be 'admin')
- [x] 5.3 Return 403 Forbidden for non-admin users
- [x] 5.4 Add middleware to Axum router for admin routes

## 6. Admin Endpoints - Backend
- [x] 6.1 Create admin handlers module (backend/src/handlers/admin.rs)
- [x] 6.2 Add GET /api/admin/users (list with pagination, filtering)
- [x] 6.3 Add GET /api/admin/users/:id (view user details)
- [x] 6.4 Add PATCH /api/admin/users/:id/disable (soft delete)
- [x] 6.5 Add PATCH /api/admin/users/:id/enable (restore)
- [x] 6.6 ADD GET /api/admin/stats (user count, verified count, admin count)
- [x] 6.7 Add OpenAPI annotations for admin endpoints
- [x] 6.8 Add admin route group to main.rs

## 7. Admin Dashboard - Frontend
- [ ] 7.1 Create /admin layout (frontend/src/app/admin/layout.tsx)
- [ ] 7.2 Create /admin page (frontend/src/app/admin/page.tsx) with stats
- [ ] 7.3 Create /admin/users page with user list table
- [ ] 7.4 Add data table component (shadcn/ui)
- [ ] 7.5 Add pagination controls
- [ ] 7.6 Add filtering controls (verified, role, search)
- [ ] 7.7 Add user detail modal/page
- [ ] 7.8 Add disable/enable user actions
- [ ] 7.9 Add role badge component
- [ ] 7.10 Add protected route check (only admins)
- [ ] 7.11 Generate TypeScript types for admin endpoints

## 8. Database Seeding
- [x] 8.1 Create seed script to create initial admin user
- [x] 8.2 Add Makefile/script command: make seed-admin
- [x] 8.3 Document admin user creation in README

## 9. Testing
- [ ] 9.1 Write unit tests for verification token generation
- [ ] 9.2 Write integration tests for send-verification endpoint
- [ ] 9.3 Write integration tests for verify-email endpoint
- [ ] 9.4 Write integration tests for admin middleware
- [ ] 9.5 Write integration tests for admin endpoints
- [ ] 9.6 Write integration tests for role-based authorization
- [ ] 9.7 Test email verification flow end-to-end with Playwright
- [ ] 9.8 Test admin dashboard with Playwright

## 10. Documentation
- [ ] 10.1 Document email verification flow in README
- [ ] 10.2 Document admin user creation process
- [ ] 10.3 Document SMTP configuration (optional)
- [ ] 10.4 Update API documentation (Swagger/OpenAPI)
- [ ] 10.5 Update authentication-implementation-summary.md

## 11. Configuration
- [ ] 11.1 Add EMAIL_VERIFICATION_EXPIRY env var (default: 86400 seconds)
- [ ] 11.2 Add SMTP_* env vars (host, port, user, password, from)
- [ ] 11.3 Add EMAIL_MOCK env var (default: true)
- [ ] 11.4 Update .env.example with new variables
- [ ] 11.5 Update docker-compose.yml with environment variables

## 12. UI Polish
- [ ] 12.1 Add loading states to verification page
- [ ] 12.2 Add success/error messages
- [ ] 12.3 Add admin dashboard navigation
- [ ] 12.4 Add breadcrumbs for admin pages
- [ ] 12.5 Add responsive design for admin tables
