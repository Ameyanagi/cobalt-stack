# Authentication Specification - Email Verification Delta

## ADDED Requirements

### Requirement: Email Verification Token Generation
The system SHALL generate secure verification tokens for email confirmation.

#### Scenario: Token generation on registration
- **WHEN** a user successfully registers
- **THEN** generate cryptographically random 32-byte token
- **AND** hash token with SHA-256 before database storage
- **AND** store token hash with user_id and expiration (24 hours)
- **AND** return unhashed token for email link

#### Scenario: Token uniqueness
- **WHEN** generating verification token
- **THEN** ensure token is globally unique
- **AND** handle collision by regenerating
- **AND** verify uniqueness before storage

### Requirement: Send Verification Email
The system SHALL send verification emails to newly registered users.

#### Scenario: Send on registration
- **WHEN** user completes registration
- **THEN** generate verification token
- **AND** create verification link with token: `{FRONTEND_URL}/verify-email?token={token}`
- **AND** send email with verification link
- **AND** use mock email sender by default (log to console)
- **AND** support SMTP configuration via environment variables

#### Scenario: Email content
- **WHEN** sending verification email
- **THEN** include user's username in greeting
- **AND** include clickable verification link
- **AND** include link expiration time (24 hours)
- **AND** include resend instructions if link expires

### Requirement: Verify Email Address
The system SHALL validate verification tokens and mark emails as verified.

#### Scenario: Successful verification
- **WHEN** user clicks verification link with valid unexpired token
- **THEN** hash provided token with SHA-256
- **AND** find matching token_hash in email_verifications table
- **AND** verify token has not expired (< 24 hours old)
- **AND** verify token has not been used (verified_at IS NULL)
- **AND** set user.email_verified = true
- **AND** set verification.verified_at = current timestamp
- **AND** return success response
- **AND** redirect to login or dashboard

#### Scenario: Expired token
- **WHEN** user attempts verification with expired token
- **THEN** return 400 Bad Request with message "Verification link expired"
- **AND** provide option to resend verification email

#### Scenario: Already verified
- **WHEN** user attempts verification with already-used token
- **THEN** return 400 Bad Request with message "Email already verified"
- **AND** allow user to proceed to login

#### Scenario: Invalid token
- **WHEN** user provides malformed or non-existent token
- **THEN** return 400 Bad Request with message "Invalid verification link"

### Requirement: Resend Verification Email
The system SHALL allow users to request new verification emails.

#### Scenario: Resend for unverified user
- **WHEN** authenticated unverified user requests new verification email
- **THEN** invalidate any existing unexpired verification tokens
- **AND** generate new verification token
- **AND** send new verification email
- **AND** return success response

#### Scenario: Resend for already verified user
- **WHEN** verified user requests verification email
- **THEN** return 400 Bad Request with message "Email already verified"

#### Scenario: Rate limit resend requests
- **WHEN** user requests multiple verification emails
- **THEN** limit to 3 requests per hour per user
- **AND** return 429 Too Many Requests if limit exceeded

### Requirement: Email Verification Database Schema
The system SHALL persist email verification tokens in PostgreSQL.

#### Scenario: Email verifications table
- **WHEN** system initializes
- **THEN** create email_verifications table with columns: id (UUID, primary key), user_id (foreign key to users), token_hash (unique, indexed), expires_at (timestamp), verified_at (nullable timestamp), created_at (timestamp)
- **AND** create index on user_id for efficient lookup
- **AND** create index on token_hash for verification queries
- **AND** create index on expires_at for cleanup queries

#### Scenario: Token cleanup
- **WHEN** running scheduled cleanup job
- **THEN** delete verification records where expires_at < current_time - 7 days
- **AND** keep verified records for audit trail

### Requirement: Email Configuration
The system SHALL configure email sending via environment variables.

#### Scenario: Mock email mode
- **WHEN** EMAIL_MOCK=true (default)
- **THEN** log verification emails to console
- **AND** include full verification link in logs
- **AND** do not attempt SMTP connection

#### Scenario: SMTP configuration
- **WHEN** EMAIL_MOCK=false
- **THEN** read SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASSWORD, SMTP_FROM from environment
- **AND** validate all SMTP variables are present
- **AND** fail startup if any SMTP variable is missing when mock is disabled

## MODIFIED Requirements

### Requirement: User Registration
The system SHALL allow users to create accounts with username, email, and password, and send verification email.

#### Scenario: Successful registration
- **WHEN** a user provides valid username (3-50 chars), email (valid format), and password (min 8 chars)
- **THEN** create user account with hashed password (Argon2id)
- **AND** set email_verified = false
- **AND** generate and send email verification link
- **AND** return JWT access token (15-30 min expiry) and refresh token (7-30 days expiry)
- **AND** return user object (id, username, email, email_verified)

#### Scenario: Duplicate username
- **WHEN** a user attempts registration with existing username
- **THEN** return 409 Conflict error with message "Username already exists"

#### Scenario: Duplicate email
- **WHEN** a user attempts registration with existing email
- **THEN** return 409 Conflict error with message "Email already exists"

#### Scenario: Weak password
- **WHEN** a user provides password shorter than 8 characters
- **THEN** return 400 Bad Request error with message "Password must be at least 8 characters"

#### Scenario: Invalid email format
- **WHEN** a user provides invalid email format
- **THEN** return 400 Bad Request error with message "Invalid email format"

### Requirement: Get Current User
The system SHALL allow authenticated users to retrieve their profile information including verification status.

#### Scenario: Get user profile
- **WHEN** an authenticated user requests their profile
- **THEN** extract user ID from JWT claims
- **AND** retrieve user from database or Redis cache
- **AND** return user object (id, username, email, email_verified, role, created_at)

### Requirement: Database Schema
The system SHALL persist user and token data in PostgreSQL with email verification support.

#### Scenario: Users table
- **WHEN** system initializes
- **THEN** create users table with columns: id (UUID, primary key), username (unique), email (unique), password_hash (nullable for OAuth), email_verified (boolean, default false), role (enum: 'user'|'admin', default 'user'), created_at, updated_at
- **AND** create unique indexes on username and email
- **AND** create index on email_verified for filtering
- **AND** create index on role for admin queries

#### Scenario: Refresh tokens table
- **WHEN** system initializes
- **THEN** create refresh_tokens table with columns: id (UUID, primary key), user_id (foreign key), token_hash (unique), expires_at, revoked_at (nullable), created_at
- **AND** create indexes on user_id, token_hash, expires_at

#### Scenario: OAuth accounts table (future use)
- **WHEN** system initializes
- **THEN** create oauth_accounts table with columns: id (UUID, primary key), user_id (foreign key), provider (varchar), provider_user_id, access_token, refresh_token, expires_at, created_at
- **AND** create unique constraint on (provider, provider_user_id)
