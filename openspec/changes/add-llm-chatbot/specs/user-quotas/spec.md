# User Quotas Capability

## ADDED Requirements

### Requirement: Rate Limiting
The system SHALL prevent abuse and control costs by limiting the number of chat messages users can send within a time window.

#### Scenario: Enforce message rate limit
- **WHEN** user attempts to send chat message
- **THEN** the system checks messages sent by user in last minute
- **AND** allows message if count < configured limit (default: 20/minute)
- **AND** returns 429 Too Many Requests if limit exceeded
- **AND** includes Retry-After header with seconds until reset

#### Scenario: Rate limit per user isolation
- **WHEN** user A exhausts their rate limit
- **THEN** user B can still send messages normally
- **AND** each user has independent rate limit counter

#### Scenario: Rate limit reset
- **WHEN** rate limit time window expires (60 seconds)
- **THEN** the system resets counter for that user
- **AND** allows new messages up to limit again

### Requirement: Daily Message Quotas
The system SHALL limit the total number of messages each user can send per day to control API costs.

#### Scenario: Track daily message count
- **WHEN** user sends message successfully
- **THEN** the system increments user's daily message counter
- **AND** stores counter with current date in Redis
- **AND** sets TTL to reset at midnight UTC

#### Scenario: Enforce daily quota limit
- **WHEN** user attempts to send message
- **THEN** the system checks user's message count for current day
- **AND** allows message if count < daily quota (default: 100)
- **AND** returns 429 Too Many Requests with error "Daily quota exceeded"
- **AND** includes quota information in response (used, limit, resets_at)

#### Scenario: Daily quota reset
- **WHEN** new day begins (midnight UTC)
- **THEN** the system resets user's daily message counter to 0
- **AND** user can send up to quota limit again

#### Scenario: Admin users unlimited quota
- **WHEN** user with admin role sends message
- **THEN** the system bypasses daily quota check
- **AND** allows unlimited messages
- **AND** still enforces rate limiting for stability

### Requirement: Quota Information Display
The system SHALL inform users about their quota usage and limits to set clear expectations.

#### Scenario: Include quota info in API responses
- **WHEN** user successfully sends message
- **THEN** response headers include X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset
- **AND** response headers include X-Daily-Quota-Limit, X-Daily-Quota-Remaining, X-Daily-Quota-Reset

#### Scenario: Get quota status endpoint
- **WHEN** user sends GET /api/v1/chat/quota
- **THEN** the system returns current quota usage
- **AND** includes rate_limit (limit, remaining, resets_in_seconds)
- **AND** includes daily_quota (limit, used, remaining, resets_at)

#### Scenario: Display quota in UI
- **WHEN** user is in chat interface
- **THEN** frontend displays remaining daily messages
- **AND** shows warning when approaching limit (80%+)
- **AND** shows error message when quota exhausted
- **AND** displays time until quota reset

### Requirement: Quota Configuration
The system SHALL allow administrators to configure quota limits via environment variables.

#### Scenario: Configure rate limit
- **GIVEN** backend starts up
- **THEN** it reads CHAT_RATE_LIMIT_PER_MINUTE (default: 20)
- **AND** applies this limit to all non-admin users

#### Scenario: Configure daily quota
- **GIVEN** backend starts up
- **THEN** it reads CHAT_DAILY_MESSAGE_QUOTA (default: 100)
- **AND** applies this quota to all non-admin users

#### Scenario: Custom quota per user (future)
- **GIVEN** system supports user-specific quotas (future enhancement)
- **THEN** individual users can have custom limits
- **AND** custom limits override default configuration
- **AND** stored in database user_chat_quotas table

### Requirement: Quota Storage in Redis
The system SHALL use Redis (Valkey) for efficient quota tracking with automatic expiration.

#### Scenario: Store rate limit counter in Redis
- **WHEN** user sends message
- **THEN** the system increments Redis key "chat:ratelimit:{user_id}"
- **AND** sets TTL to 60 seconds if key is new
- **AND** uses INCR for atomic increment

#### Scenario: Store daily quota counter in Redis
- **WHEN** user sends message
- **THEN** the system increments Redis key "chat:quota:daily:{user_id}:{date}"
- **AND** sets TTL to expire at next midnight UTC
- **AND** uses INCR for atomic increment

#### Scenario: Handle Redis unavailable
- **WHEN** Redis connection fails
- **THEN** the system logs error and allows message (fail open for availability)
- **OR** returns 503 Service Unavailable if configured for strict quota enforcement
- **AND** sends alert to monitoring system

### Requirement: Graceful Quota Enforcement
The system SHALL enforce quotas without disrupting user experience unnecessarily.

#### Scenario: Viewing history when quota exceeded
- **WHEN** user exceeds daily quota
- **THEN** user can still view existing chat sessions and messages
- **AND** cannot send new messages until quota resets
- **AND** UI shows quota status clearly

#### Scenario: Progressive warnings
- **WHEN** user reaches 80% of daily quota
- **THEN** frontend displays warning message
- **AND** shows exact number of messages remaining

#### Scenario: Quota exceeded error message
- **WHEN** user attempts message after quota exceeded
- **THEN** frontend displays clear error message
- **AND** shows time until quota reset
- **AND** suggests upgrading account if applicable (future)
