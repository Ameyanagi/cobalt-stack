# Authentication Specification

## ADDED Requirements

### Requirement: User Registration
The system SHALL allow users to create accounts with username, email, and password.

#### Scenario: Successful registration
- **WHEN** a user provides valid username (3-50 chars), email (valid format), and password (min 8 chars)
- **THEN** create user account with hashed password (Argon2id)
- **AND** return JWT access token (15-30 min expiry) and refresh token (7-30 days expiry)
- **AND** return user object (id, username, email)

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

### Requirement: User Login
The system SHALL authenticate users with username/email and password.

#### Scenario: Successful login with username
- **WHEN** a user provides valid username and correct password
- **THEN** verify password hash using constant-time comparison
- **AND** return JWT access token and refresh token
- **AND** return user object (id, username, email)

#### Scenario: Successful login with email
- **WHEN** a user provides valid email and correct password
- **THEN** verify password hash using constant-time comparison
- **AND** return JWT access token and refresh token
- **AND** return user object (id, username, email)

#### Scenario: Invalid credentials
- **WHEN** a user provides wrong password or non-existent username
- **THEN** return 401 Unauthorized error with message "Invalid credentials"
- **AND** increment rate limit counter for IP address

#### Scenario: Rate limit exceeded
- **WHEN** a user exceeds 5 failed login attempts within 15 minutes
- **THEN** return 429 Too Many Requests error with message "Too many login attempts"
- **AND** block further attempts until rate limit window expires

### Requirement: Token Refresh
The system SHALL allow users to obtain new access tokens using refresh tokens.

#### Scenario: Successful token refresh
- **WHEN** a user provides valid non-expired refresh token
- **THEN** verify refresh token hash in database
- **AND** check token is not revoked
- **AND** issue new access token and refresh token pair
- **AND** revoke old refresh token (token rotation)

#### Scenario: Expired refresh token
- **WHEN** a user provides expired refresh token
- **THEN** return 401 Unauthorized error with message "Refresh token expired"
- **AND** require user to login again

#### Scenario: Revoked refresh token
- **WHEN** a user provides revoked refresh token
- **THEN** return 401 Unauthorized error with message "Invalid refresh token"

#### Scenario: Invalid refresh token
- **WHEN** a user provides malformed or tampered refresh token
- **THEN** return 401 Unauthorized error with message "Invalid refresh token"

### Requirement: User Logout
The system SHALL allow users to invalidate their sessions.

#### Scenario: Successful logout
- **WHEN** an authenticated user requests logout
- **THEN** revoke refresh token in database (set revoked_at timestamp)
- **AND** add access token to Redis blacklist (TTL: remaining token lifetime)
- **AND** return success response

#### Scenario: Logout with invalid token
- **WHEN** a user attempts logout with invalid access token
- **THEN** return 401 Unauthorized error

### Requirement: Protected Route Access
The system SHALL protect routes requiring authentication via middleware.

#### Scenario: Access with valid token
- **WHEN** a user accesses protected route with valid access token in Authorization header
- **THEN** extract and verify JWT signature
- **AND** check token expiration
- **AND** verify token not in Redis blacklist
- **AND** inject user claims into request context
- **AND** allow request to proceed

#### Scenario: Access with expired token
- **WHEN** a user accesses protected route with expired access token
- **THEN** return 401 Unauthorized error with message "Token expired"

#### Scenario: Access with blacklisted token
- **WHEN** a user accesses protected route with blacklisted token (after logout)
- **THEN** return 401 Unauthorized error with message "Invalid token"

#### Scenario: Access without token
- **WHEN** a user accesses protected route without Authorization header
- **THEN** return 401 Unauthorized error with message "Missing authorization token"

### Requirement: Get Current User
The system SHALL allow authenticated users to retrieve their profile information.

#### Scenario: Get user profile
- **WHEN** an authenticated user requests their profile
- **THEN** extract user ID from JWT claims
- **AND** retrieve user from database or Redis cache
- **AND** return user object (id, username, email, created_at)

### Requirement: Password Security
The system SHALL hash passwords using Argon2id with secure parameters.

#### Scenario: Password hashing
- **WHEN** a user registers or changes password
- **THEN** hash password with Argon2id algorithm
- **AND** use parameters: memory=19456 KiB, iterations=2, parallelism=1
- **AND** store only password hash, never plaintext

#### Scenario: Password verification
- **WHEN** a user attempts login
- **THEN** verify password using constant-time comparison
- **AND** prevent timing attacks

### Requirement: JWT Token Structure
The system SHALL issue JWTs with standardized claims and secure algorithms.

#### Scenario: Access token creation
- **WHEN** issuing access token
- **THEN** include claims: user_id (sub), username, email, iat, exp
- **AND** sign with HS256 (dev) or RS256 (production)
- **AND** set expiration to 15-30 minutes

#### Scenario: Refresh token creation
- **WHEN** issuing refresh token
- **THEN** generate cryptographically random token
- **AND** hash token with SHA-256 before database storage
- **AND** set expiration to 7-30 days
- **AND** store hash in refresh_tokens table with user_id reference

### Requirement: Database Schema
The system SHALL persist user and token data in PostgreSQL.

#### Scenario: Users table
- **WHEN** system initializes
- **THEN** create users table with columns: id (UUID, primary key), username (unique), email (unique), password_hash (nullable for OAuth), email_verified (boolean), created_at, updated_at
- **AND** create unique indexes on username and email

#### Scenario: Refresh tokens table
- **WHEN** system initializes
- **THEN** create refresh_tokens table with columns: id (UUID, primary key), user_id (foreign key), token_hash (unique), expires_at, revoked_at (nullable), created_at
- **AND** create indexes on user_id, token_hash, expires_at

#### Scenario: OAuth accounts table (future use)
- **WHEN** system initializes
- **THEN** create oauth_accounts table with columns: id (UUID, primary key), user_id (foreign key), provider (varchar), provider_user_id, access_token, refresh_token, expires_at, created_at
- **AND** create unique constraint on (provider, provider_user_id)

### Requirement: Rate Limiting
The system SHALL prevent brute force attacks via rate limiting.

#### Scenario: Login rate limit
- **WHEN** tracking login attempts
- **THEN** use Redis to store attempt count per IP address
- **AND** limit to 5 attempts per 15-minute window
- **AND** return 429 Too Many Requests after limit exceeded

### Requirement: Session Blacklist
The system SHALL maintain immediate logout capability via token blacklist.

#### Scenario: Blacklist on logout
- **WHEN** user logs out
- **THEN** add access token JTI to Redis blacklist
- **AND** set TTL to remaining token lifetime
- **AND** check blacklist on every protected route access

### Requirement: CORS Configuration
The system SHALL enable cross-origin requests with credentials for frontend integration.

#### Scenario: CORS headers
- **WHEN** frontend makes authenticated request
- **THEN** allow credentials (cookies)
- **AND** set Access-Control-Allow-Credentials: true
- **AND** set Access-Control-Allow-Origin to configured frontend URL
- **AND** allow Authorization header in requests

### Requirement: Error Handling
The system SHALL return consistent error responses with appropriate HTTP status codes.

#### Scenario: Authentication errors
- **WHEN** authentication fails
- **THEN** return 401 Unauthorized for invalid credentials or tokens
- **AND** return 409 Conflict for duplicate username/email
- **AND** return 400 Bad Request for validation errors
- **AND** return 429 Too Many Requests for rate limit violations
- **AND** never leak sensitive information in error messages

### Requirement: Environment Configuration
The system SHALL configure auth parameters via environment variables.

#### Scenario: JWT configuration
- **WHEN** application starts
- **THEN** read JWT_SECRET from environment (min 256 bits)
- **AND** read JWT_ACCESS_EXPIRY (default: 1800 seconds)
- **AND** read JWT_REFRESH_EXPIRY (default: 2592000 seconds)
- **AND** fail startup if JWT_SECRET is missing

#### Scenario: Rate limit configuration
- **WHEN** application starts
- **THEN** read RATE_LIMIT_LOGIN_MAX (default: 5)
- **AND** read RATE_LIMIT_LOGIN_WINDOW (default: 900 seconds)

### Requirement: Future OAuth Extensibility
The system SHALL support adding OAuth providers without breaking changes.

#### Scenario: OAuth-only users
- **WHEN** user authenticates via OAuth (future feature)
- **THEN** create user with NULL password_hash
- **AND** link OAuth account in oauth_accounts table
- **AND** issue same JWT tokens as username/password users

#### Scenario: Multiple OAuth providers per user
- **WHEN** user links multiple OAuth providers
- **THEN** allow multiple oauth_accounts entries per user_id
- **AND** use same user_id for all linked accounts
