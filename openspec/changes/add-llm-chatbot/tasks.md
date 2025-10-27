# Implementation Tasks

## Phase 1: Foundation and Database (TDD + DDD)

### 1.1 Database Schema
- [ ] 1.1.1 Create migration: `m20250127_create_chat_tables.rs`
- [ ] 1.1.2 Define chat_sessions table (id, user_id, title, created_at, updated_at, deleted_at)
- [ ] 1.1.3 Define chat_messages table (id, session_id, role, content, token_count, created_at)
- [ ] 1.1.4 Add foreign keys and indexes
- [ ] 1.1.5 Run migration: `cd backend/migration && cargo run`
- [ ] 1.1.6 Verify schema in PostgreSQL

### 1.2 Domain Layer (TDD)
- [ ] 1.2.1 **Test**: Write tests for ChatSession entity creation
- [ ] 1.2.2 Create `backend/src/domain/chat/mod.rs`
- [ ] 1.2.3 **Test**: Write tests for ChatSession validation
- [ ] 1.2.4 Define ChatSession entity in `backend/src/domain/chat/entity.rs`
- [ ] 1.2.5 **Test**: Write tests for ChatMessage entity
- [ ] 1.2.6 Define ChatMessage entity
- [ ] 1.2.7 **Test**: Write tests for MessageRole value object
- [ ] 1.2.8 Define MessageRole enum in `backend/src/domain/chat/value_objects.rs` (User, Assistant, System)
- [ ] 1.2.9 Define ChatRepository trait in `backend/src/domain/chat/repository.rs`
- [ ] 1.2.10 Run all domain tests: `cargo test domain::chat`

### 1.3 Infrastructure Layer - Persistence (TDD)
- [ ] 1.3.1 **Test**: Write repository tests with test database
- [ ] 1.3.2 Create SeaORM models in `backend/src/infrastructure/persistence/models.rs`
- [ ] 1.3.3 **Test**: Write tests for session CRUD operations
- [ ] 1.3.4 Implement ChatRepository in `backend/src/infrastructure/persistence/chat_repository.rs`
- [ ] 1.3.5 **Test**: Write tests for message persistence
- [ ] 1.3.6 Implement message save/retrieve methods
- [ ] 1.3.7 **Test**: Write tests for soft delete
- [ ] 1.3.8 Implement soft delete logic
- [ ] 1.3.9 Run integration tests: `cargo test infrastructure::persistence::chat`

### 1.4 Dependencies
- [ ] 1.4.1 Add `async-openai = "0.20"` to backend/Cargo.toml
- [ ] 1.4.2 Verify react-markdown, remark-gfm, rehype-highlight in frontend/package.json (already present)
- [ ] 1.4.3 Run `cargo build` to verify backend compiles
- [ ] 1.4.4 Run `cd frontend && bun install` to verify frontend deps

## Phase 2: Application Layer and Use Cases (TDD)

### 2.1 Create Session Use Case (TDD)
- [ ] 2.1.1 **Test**: Write failing test for CreateSessionUseCase
- [ ] 2.1.2 Create `backend/src/application/chat/mod.rs`
- [ ] 2.1.3 **Test**: Write test for title generation
- [ ] 2.1.4 Implement CreateSessionUseCase in `create_session.rs`
- [ ] 2.1.5 **Test**: Write test for authorization check
- [ ] 2.1.6 Define CreateSessionDTO in `backend/src/application/dto/chat.rs`
- [ ] 2.1.7 Run tests: `cargo test application::chat::create_session`

### 2.2 List Sessions Use Case (TDD)
- [ ] 2.2.1 **Test**: Write tests for ListSessionsUseCase with pagination
- [ ] 2.2.2 Implement ListSessionsUseCase in `list_sessions.rs`
- [ ] 2.2.3 **Test**: Write test for filtering deleted sessions
- [ ] 2.2.4 Implement soft delete filtering
- [ ] 2.2.5 **Test**: Write test for ordering by created_at DESC
- [ ] 2.2.6 Define pagination DTOs
- [ ] 2.2.7 Run tests: `cargo test application::chat::list_sessions`

### 2.3 Send Message Use Case (TDD)
- [ ] 2.3.1 **Test**: Write tests for SendMessageUseCase
- [ ] 2.3.2 Implement SendMessageUseCase in `send_message.rs`
- [ ] 2.3.3 **Test**: Write tests for message validation
- [ ] 2.3.4 Implement validation logic (length, content)
- [ ] 2.3.5 **Test**: Write tests for context building
- [ ] 2.3.6 Implement context building (last N messages)
- [ ] 2.3.7 Run tests: `cargo test application::chat::send_message`

### 2.4 Delete Session Use Case (TDD)
- [ ] 2.4.1 **Test**: Write tests for DeleteSessionUseCase
- [ ] 2.4.2 Implement DeleteSessionUseCase in `delete_session.rs`
- [ ] 2.4.3 **Test**: Write test for ownership verification
- [ ] 2.4.4 Implement soft delete logic
- [ ] 2.4.5 Run tests: `cargo test application::chat::delete_session`

## Phase 3: SambaNova API Integration (TDD)

### 3.1 SambaNova Client Setup (TDD)
- [ ] 3.1.1 Create `backend/src/infrastructure/llm/mod.rs`
- [ ] 3.1.2 **Test**: Write tests for client configuration
- [ ] 3.1.3 Create SambaNova client wrapper in `sambanova_client.rs`
- [ ] 3.1.4 **Test**: Write tests for API key validation
- [ ] 3.1.5 Configure OpenAIConfig with custom base URL
- [ ] 3.1.6 Add environment variable loading for SAMBANOVA_API_KEY

### 3.2 Non-Streaming Chat (TDD - MVP First)
- [ ] 3.2.1 **Test**: Write mock tests for non-streaming chat completion
- [ ] 3.2.2 Implement non-streaming chat method
- [ ] 3.2.3 **Test**: Write tests for response parsing
- [ ] 3.2.4 Handle API errors gracefully
- [ ] 3.2.5 **Test**: Write tests for timeout handling
- [ ] 3.2.6 Run tests: `cargo test infrastructure::llm`

### 3.3 Streaming Chat (SSE)
- [ ] 3.3.1 **Test**: Write tests for streaming setup
- [ ] 3.3.2 Implement streaming chat method returning Stream
- [ ] 3.3.3 **Test**: Write tests for chunk transformation
- [ ] 3.3.4 Transform async-openai stream to SSE events
- [ ] 3.3.5 **Test**: Write tests for stream error handling
- [ ] 3.3.6 Implement error events in stream
- [ ] 3.3.7 **Test**: Write tests for stream cancellation
- [ ] 3.3.8 Handle client disconnection
- [ ] 3.3.9 Implement response accumulation for database save

## Phase 4: HTTP Handlers (Presentation Layer)

### 4.1 Session Management Handlers
- [ ] 4.1.1 Create `backend/src/presentation/handlers/chat.rs`
- [ ] 4.1.2 Implement POST /api/v1/chat/sessions (create_session_handler)
- [ ] 4.1.3 Implement GET /api/v1/chat/sessions (list_sessions_handler)
- [ ] 4.1.4 Implement GET /api/v1/chat/sessions/:id (get_session_handler)
- [ ] 4.1.5 Implement DELETE /api/v1/chat/sessions/:id (delete_session_handler)
- [ ] 4.1.6 Add utoipa annotations for OpenAPI generation
- [ ] 4.1.7 Write API tests in `backend/tests/api/chat_test.rs`

### 4.2 Message Streaming Handler
- [ ] 4.2.1 Implement POST /api/v1/chat/sessions/:id/messages (send_message_handler)
- [ ] 4.2.2 Return Sse<Stream<Item = Result<Event, Infallible>>>
- [ ] 4.2.3 Integrate with SendMessageUseCase and streaming client
- [ ] 4.2.4 Add authorization middleware check
- [ ] 4.2.5 Add utoipa annotations
- [ ] 4.2.6 Write streaming API tests

### 4.3 Router Integration
- [ ] 4.3.1 Update `backend/src/presentation/routes.rs`
- [ ] 4.3.2 Add chat routes under /api/v1/chat
- [ ] 4.3.3 Apply JWT authentication middleware to all chat routes
- [ ] 4.3.4 Update OpenAPI schema generation in `backend/src/presentation/openapi/mod.rs`
- [ ] 4.3.5 Rebuild backend and verify Swagger UI: `cargo build && cargo run`
- [ ] 4.3.6 Test endpoints at http://localhost:8080/swagger-ui

## Phase 5: Rate Limiting and Quotas

### 5.1 Rate Limiting (TDD)
- [ ] 5.1.1 **Test**: Write tests for rate limit checking
- [ ] 5.1.2 Create `backend/src/application/quota/rate_limiter.rs`
- [ ] 5.1.3 **Test**: Write tests for Redis counter increment
- [ ] 5.1.4 Implement check_chat_rate_limit function using existing Redis infrastructure
- [ ] 5.1.5 **Test**: Write tests for TTL setting
- [ ] 5.1.6 Use INCR + EXPIRE pattern with 60-second window
- [ ] 5.1.7 **Test**: Write tests for rate limit headers
- [ ] 5.1.8 Add middleware to inject rate limit into handlers
- [ ] 5.1.9 Run tests: `cargo test application::quota::rate_limiter`

### 5.2 Daily Quotas (TDD)
- [ ] 5.2.1 **Test**: Write tests for daily quota tracking
- [ ] 5.2.2 Create `backend/src/application/quota/daily_quota.rs`
- [ ] 5.2.3 **Test**: Write tests for quota counter increment
- [ ] 5.2.4 Implement check_daily_quota function
- [ ] 5.2.5 **Test**: Write tests for midnight UTC reset
- [ ] 5.2.6 Use Redis key pattern: chat:quota:daily:{user_id}:{date}
- [ ] 5.2.7 **Test**: Write tests for admin bypass
- [ ] 5.2.8 Set TTL to next midnight UTC
- [ ] 5.2.9 Run tests: `cargo test application::quota::daily_quota`

### 5.3 Quota Info Endpoint
- [ ] 5.3.1 Implement GET /api/v1/chat/quota (get_quota_handler)
- [ ] 5.3.2 Return rate limit and daily quota status
- [ ] 5.3.3 Add response headers: X-RateLimit-*, X-Daily-Quota-*
- [ ] 5.3.4 Write API test for quota endpoint
- [ ] 5.3.5 Add utoipa annotations

### 5.4 Quota Integration
- [ ] 5.4.1 Add quota checks to send_message_handler
- [ ] 5.4.2 Return 429 Too Many Requests when limits exceeded
- [ ] 5.4.3 Include quota info in error responses
- [ ] 5.4.4 Test quota enforcement end-to-end

## Phase 6: Frontend Components

### 6.1 API Client
- [ ] 6.1.1 Generate TypeScript types: `cd frontend && bun run generate-types`
- [ ] 6.1.2 Create `frontend/src/lib/api/chat.ts` with typed API functions
- [ ] 6.1.3 Implement createSession(), listSessions(), getSession(), deleteSession()
- [ ] 6.1.4 Implement sendMessage() with EventSource for SSE
- [ ] 6.1.5 Implement getQuota() for quota status

### 6.2 Chat Session List Component
- [ ] 6.2.1 Create `frontend/src/components/chat/ChatList.tsx`
- [ ] 6.2.2 Implement session list with React Query
- [ ] 6.2.3 Add "New Chat" button
- [ ] 6.2.4 Add delete button for each session
- [ ] 6.2.5 Show active session highlight
- [ ] 6.2.6 Style with shadcn/ui components

### 6.3 Chat Window Component
- [ ] 6.3.1 Create `frontend/src/components/chat/ChatWindow.tsx`
- [ ] 6.3.2 Implement message list display
- [ ] 6.3.3 Add streaming message state management
- [ ] 6.3.4 Implement EventSource connection for SSE
- [ ] 6.3.5 Add typing indicator component
- [ ] 6.3.6 Implement scroll to bottom on new messages

### 6.4 Message Components
- [ ] 6.4.1 Create `frontend/src/components/chat/MessageBubble.tsx`
- [ ] 6.4.2 Integrate react-markdown with remark-gfm
- [ ] 6.4.3 Add rehype-highlight for code syntax highlighting
- [ ] 6.4.4 Implement copy button for code blocks
- [ ] 6.4.5 Style user vs assistant messages differently
- [ ] 6.4.6 Add timestamp display

### 6.5 Message Input Component
- [ ] 6.5.1 Create `frontend/src/components/chat/MessageInput.tsx`
- [ ] 6.5.2 Implement textarea with auto-resize
- [ ] 6.5.3 Add send button with loading state
- [ ] 6.5.4 Implement Enter to send (Shift+Enter for newline)
- [ ] 6.5.5 Add character count display
- [ ] 6.5.6 Disable input when streaming or quota exceeded

### 6.6 Quota Display Component
- [ ] 6.6.1 Create `frontend/src/components/chat/QuotaDisplay.tsx`
- [ ] 6.6.2 Fetch and display quota status
- [ ] 6.6.3 Show warning at 80% usage
- [ ] 6.6.4 Show error when quota exceeded
- [ ] 6.6.5 Display time until reset
- [ ] 6.6.6 Auto-refresh quota on quota reset time

### 6.7 Main Chat Page
- [ ] 6.7.1 Create `frontend/src/app/chat/page.tsx`
- [ ] 6.7.2 Integrate ChatList and ChatWindow
- [ ] 6.7.3 Implement session selection logic
- [ ] 6.7.4 Add error boundary
- [ ] 6.7.5 Add loading states
- [ ] 6.7.6 Add responsive layout (mobile + desktop)

## Phase 7: Error Handling and Polish

### 7.1 Backend Error Handling
- [ ] 7.1.1 Add custom error types in `backend/src/shared/errors.rs`
- [ ] 7.1.2 Implement ChatError enum (SessionNotFound, QuotaExceeded, ApiUnavailable, etc.)
- [ ] 7.1.3 Map domain errors to HTTP status codes
- [ ] 7.1.4 Add structured error responses with user-friendly messages
- [ ] 7.1.5 Implement circuit breaker for SambaNova API (5 consecutive failures)

### 7.2 Frontend Error Handling
- [ ] 7.2.1 Create error display components
- [ ] 7.2.2 Implement retry logic for failed requests
- [ ] 7.2.3 Add connection status indicator
- [ ] 7.2.4 Implement fallback to non-streaming on SSE failure
- [ ] 7.2.5 Add toast notifications for errors (using sonner)
- [ ] 7.2.6 Persist draft messages in localStorage

### 7.3 Logging and Monitoring
- [ ] 7.3.1 Add tracing spans for chat operations
- [ ] 7.3.2 Log API response times
- [ ] 7.3.3 Log quota violations
- [ ] 7.3.4 Log SambaNova API errors
- [ ] 7.3.5 Add structured logging with context (user_id, session_id)

## Phase 8: Testing

### 8.1 Backend Tests
- [ ] 8.1.1 Run all unit tests: `cargo test`
- [ ] 8.1.2 Verify domain layer has 100% coverage
- [ ] 8.1.3 Verify application layer has >80% coverage
- [ ] 8.1.4 Run integration tests with test database
- [ ] 8.1.5 Run API tests: `cargo test api::chat`
- [ ] 8.1.6 Mock SambaNova API responses in tests
- [ ] 8.1.7 Test quota enforcement scenarios

### 8.2 Frontend Tests
- [ ] 8.2.1 Write component tests for ChatWindow
- [ ] 8.2.2 Write component tests for MessageBubble
- [ ] 8.2.3 Write component tests for MessageInput
- [ ] 8.2.4 Test streaming behavior with MSW
- [ ] 8.2.5 Test error handling scenarios
- [ ] 8.2.6 Run tests: `cd frontend && bun test`

### 8.3 End-to-End Tests
- [ ] 8.3.1 Write E2E test: Create session → Send message → Receive response
- [ ] 8.3.2 Write E2E test: List sessions → Delete session
- [ ] 8.3.3 Write E2E test: Quota exceeded scenario
- [ ] 8.3.4 Write E2E test: Markdown rendering
- [ ] 8.3.5 Run E2E tests: `cd frontend && bun run test:e2e`

## Phase 9: Documentation and Configuration

### 9.1 Configuration
- [ ] 9.1.1 Add environment variables to `backend/.env.example`
- [ ] 9.1.2 Add SAMBANOVA_API_KEY, SAMBANOVA_API_BASE, SAMBANOVA_MODEL
- [ ] 9.1.3 Add CHAT_MAX_CONTEXT_MESSAGES, CHAT_MAX_TOKENS, CHAT_MAX_MESSAGE_LENGTH
- [ ] 9.1.4 Add CHAT_DAILY_MESSAGE_QUOTA, CHAT_RATE_LIMIT_PER_MINUTE
- [ ] 9.1.5 Add FEATURE_CHAT_ENABLED (default: false)
- [ ] 9.1.6 Update configuration loading in `backend/src/config/mod.rs`

### 9.2 API Documentation
- [ ] 9.2.1 Verify OpenAPI schema is complete: `cargo run` → visit /swagger-ui
- [ ] 9.2.2 Add request/response examples to utoipa annotations
- [ ] 9.2.3 Document all error codes and messages
- [ ] 9.2.4 Add authentication requirements to OpenAPI

### 9.3 User Documentation
- [ ] 9.3.1 Create `docs/features/llm-chatbot.md`
- [ ] 9.3.2 Document how to use chat feature
- [ ] 9.3.3 Explain quota limits and rate limiting
- [ ] 9.3.4 Add troubleshooting section
- [ ] 9.3.5 Add screenshots of chat interface

### 9.4 Developer Documentation
- [ ] 9.4.1 Update README.md with chat feature setup
- [ ] 9.4.2 Document SambaNova API key setup
- [ ] 9.4.3 Add migration instructions
- [ ] 9.4.4 Document environment variables

## Phase 10: Deployment and Rollout

### 10.1 Database Migration
- [ ] 10.1.1 Test migration on development database
- [ ] 10.1.2 Test migration on staging database
- [ ] 10.1.3 Verify rollback (down migration) works
- [ ] 10.1.4 Plan production migration window

### 10.2 Feature Flag Setup
- [ ] 10.2.1 Deploy with FEATURE_CHAT_ENABLED=false
- [ ] 10.2.2 Run smoke tests in staging
- [ ] 10.2.3 Enable for admin users only (beta test)
- [ ] 10.2.4 Collect feedback and fix issues

### 10.3 Production Rollout
- [ ] 10.3.1 Enable for 10% of users (gradual rollout)
- [ ] 10.3.2 Monitor error rates and API performance
- [ ] 10.3.3 Monitor quota usage and costs
- [ ] 10.3.4 Increase to 50% of users
- [ ] 10.3.5 Monitor for 48 hours
- [ ] 10.3.6 Enable for 100% of users

### 10.4 Post-Deployment
- [ ] 10.4.1 Set up alerts for high error rates
- [ ] 10.4.2 Set up alerts for API cost spikes
- [ ] 10.4.3 Monitor user engagement metrics
- [ ] 10.4.4 Collect user feedback
- [ ] 10.4.5 Plan iteration based on feedback

## Dependencies

**Tasks must be completed sequentially within each phase.**
**Phases can have some parallelization:**
- Phase 5 (Quotas) can start after Phase 2 (Application Layer) is complete
- Phase 6 (Frontend) can start after Phase 4 (Handlers) completes API implementation
- Phase 7 (Error Handling) can be integrated throughout

**Critical path:** 1 → 2 → 3 → 4 → 6 → 8 → 9 → 10
**Parallel work:** Phase 5 alongside Phase 3-4

## Validation Checkpoints

After each phase:
1. All tests must pass (`cargo test` and `bun test`)
2. `make check` must pass (typecheck + lint)
3. OpenAPI schema must be valid
4. Manual testing of affected functionality

**Final validation before deployment:**
- [ ] All phases completed
- [ ] Test coverage >80% overall, 100% domain layer
- [ ] All E2E tests passing
- [ ] OpenAPI documentation complete
- [ ] Environment variables documented
- [ ] Database migrations tested with rollback
- [ ] Performance testing completed
- [ ] Security review completed
