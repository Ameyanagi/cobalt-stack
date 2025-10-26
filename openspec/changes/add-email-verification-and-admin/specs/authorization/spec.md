# Authorization Specification

## ADDED Requirements

### Requirement: Role-Based Access Control
The system SHALL implement role-based authorization for admin functionality.

#### Scenario: User roles
- **WHEN** system initializes
- **THEN** support two roles: 'user' (default) and 'admin'
- **AND** store role as enum in database
- **AND** include role in JWT claims
- **AND** default new registrations to 'user' role

#### Scenario: Admin middleware protection
- **WHEN** request is made to admin-protected route
- **THEN** extract JWT from Authorization header
- **AND** verify JWT signature and expiration
- **AND** extract role from JWT claims
- **AND** check if role == 'admin'
- **AND** return 403 Forbidden if role != 'admin'
- **AND** allow request to proceed if role == 'admin'

#### Scenario: Non-admin access attempt
- **WHEN** user with role 'user' attempts to access admin route
- **THEN** return 403 Forbidden error with message "Insufficient permissions"

#### Scenario: Unauthenticated access attempt
- **WHEN** unauthenticated user attempts to access admin route
- **THEN** return 401 Unauthorized error with message "Missing authorization token"

### Requirement: List All Users
The system SHALL allow admins to view all user accounts with pagination.

#### Scenario: List users with pagination
- **WHEN** admin requests GET /api/admin/users?page=1&limit=20
- **THEN** return paginated list of users (max 100 per page)
- **AND** include total count in response
- **AND** include pagination metadata (page, limit, total_pages)
- **AND** return users ordered by created_at DESC
- **AND** include fields: id, username, email, email_verified, role, created_at

#### Scenario: Filter by email verification status
- **WHEN** admin requests GET /api/admin/users?verified=true
- **THEN** return only users where email_verified = true

#### Scenario: Filter by role
- **WHEN** admin requests GET /api/admin/users?role=admin
- **THEN** return only users with role = 'admin'

#### Scenario: Search by username or email
- **WHEN** admin requests GET /api/admin/users?search=john
- **THEN** return users where username OR email contains 'john' (case-insensitive)

#### Scenario: Invalid pagination parameters
- **WHEN** admin provides page < 1 or limit > 100
- **THEN** return 400 Bad Request with validation error

### Requirement: View User Details
The system SHALL allow admins to view detailed information about specific users.

#### Scenario: Get user by ID
- **WHEN** admin requests GET /api/admin/users/:id
- **THEN** verify user exists
- **AND** return full user object including: id, username, email, email_verified, role, created_at, updated_at, last_login_at
- **AND** return count of active refresh tokens
- **AND** return verification status details

#### Scenario: User not found
- **WHEN** admin requests non-existent user ID
- **THEN** return 404 Not Found error with message "User not found"

### Requirement: Disable User Account
The system SHALL allow admins to disable user accounts (soft delete).

#### Scenario: Disable user account
- **WHEN** admin requests PATCH /api/admin/users/:id/disable
- **THEN** verify target user exists
- **AND** prevent disabling own account (admin cannot disable themselves)
- **AND** set user.disabled_at = current timestamp
- **AND** revoke all active refresh tokens for user
- **AND** add all user's access tokens to Redis blacklist
- **AND** return updated user object

#### Scenario: Prevent self-disable
- **WHEN** admin attempts to disable their own account
- **THEN** return 400 Bad Request with message "Cannot disable own account"

#### Scenario: Already disabled user
- **WHEN** admin attempts to disable already-disabled user
- **THEN** return 400 Bad Request with message "User already disabled"

### Requirement: Enable User Account
The system SHALL allow admins to re-enable disabled accounts.

#### Scenario: Enable disabled account
- **WHEN** admin requests PATCH /api/admin/users/:id/enable
- **THEN** verify user exists and is disabled
- **AND** set user.disabled_at = NULL
- **AND** return updated user object

#### Scenario: Already enabled user
- **WHEN** admin attempts to enable already-enabled user
- **THEN** return 400 Bad Request with message "User already enabled"

### Requirement: View System Statistics
The system SHALL allow admins to view aggregate statistics.

#### Scenario: Get system stats
- **WHEN** admin requests GET /api/admin/stats
- **THEN** return JSON with:
  - total_users (count)
  - verified_users (count where email_verified = true)
  - unverified_users (count where email_verified = false)
  - admin_users (count where role = 'admin')
  - disabled_users (count where disabled_at IS NOT NULL)
  - users_registered_today (count where created_at >= today)
  - users_registered_this_week (count where created_at >= 7 days ago)
  - users_registered_this_month (count where created_at >= 30 days ago)

### Requirement: Admin User Creation
The system SHALL support creating initial admin user via seed script.

#### Scenario: Seed admin user
- **WHEN** running make seed-admin or equivalent command
- **THEN** check if any admin users exist
- **AND** if no admins exist, create default admin with:
  - username: "admin"
  - email: "admin@example.com"
  - password: randomly generated and printed to console
  - role: "admin"
  - email_verified: true
- **AND** if admin already exists, print message and exit
- **AND** print admin credentials to console for initial login

#### Scenario: Promote existing user to admin
- **WHEN** running SQL or migration to promote user
- **THEN** allow UPDATE users SET role = 'admin' WHERE id = '{user_id}'
- **AND** document this process in README

### Requirement: Admin Database Schema
The system SHALL extend database schema for admin functionality.

#### Scenario: Users table with role and disabled_at
- **WHEN** system initializes
- **THEN** users table includes:
  - role ENUM('user', 'admin') NOT NULL DEFAULT 'user'
  - disabled_at TIMESTAMP NULL
  - last_login_at TIMESTAMP NULL
- **AND** create index on role column
- **AND** create index on disabled_at column


### Requirement: Disabled User Login Prevention
The system SHALL prevent disabled users from authenticating.

#### Scenario: Disabled user login attempt
- **WHEN** disabled user (disabled_at IS NOT NULL) attempts login
- **THEN** return 403 Forbidden error with message "Account has been disabled"
- **AND** do not issue JWT tokens

#### Scenario: Disabled user with valid token
- **WHEN** disabled user attempts to access protected route with previously-issued valid token
- **THEN** check if user is disabled during auth middleware
- **AND** return 403 Forbidden if user.disabled_at IS NOT NULL
- **AND** revoke token by adding to blacklist

### Requirement: Admin Security Constraints
The system SHALL enforce security restrictions on admin operations.

#### Scenario: Admin cannot promote users to admin
- **WHEN** system is deployed
- **THEN** admin role assignment MUST be manual (SQL or seed script)
- **AND** no API endpoint SHALL allow changing user role
- **AND** document role promotion process in README

#### Scenario: Minimum one admin requirement
- **WHEN** admin attempts to demote themselves or disable last admin
- **THEN** prevent action to ensure at least one admin exists
- **AND** return 400 Bad Request with message "Cannot remove last admin"

### Requirement: Frontend Admin Dashboard
The system SHALL provide admin UI for user management.

#### Scenario: Admin dashboard route protection
- **WHEN** non-admin user attempts to access /admin routes
- **THEN** check user role from auth context
- **AND** redirect to 403 error page if not admin
- **AND** show "Access Denied" message

#### Scenario: Admin user list display
- **WHEN** admin views /admin/users page
- **THEN** display data table with columns: username, email, verified status, role, created date, actions
- **AND** show pagination controls
- **AND** show filter controls (verified, role, search)
- **AND** show disable/enable button per user
- **AND** highlight own account (cannot disable)

#### Scenario: Admin stats dashboard
- **WHEN** admin views /admin page
- **THEN** display stat cards with:
  - Total users
  - Verified vs unverified users
  - Admin users count
  - Disabled users count
  - Recent registrations (today, this week, this month)
- **AND** show charts/visualizations of user growth

#### Scenario: User detail view
- **WHEN** admin clicks on user in list
- **THEN** show modal or page with full user details
- **AND** show verification status and verification date
- **AND** show active sessions count
- **AND** show last login timestamp
- **AND** show disable/enable action button
