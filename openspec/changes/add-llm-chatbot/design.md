# LLM Chatbot Feature Design

## Context

We are adding an AI chatbot feature to enable conversational AI interactions within the application. This is a cross-cutting feature spanning backend API, database schema, frontend UI, and external service integration.

**Background:**
- SambaNova provides OpenAI-compatible API for LLM access
- Users expect real-time streaming responses (not waiting for complete response)
- Need to manage costs through quotas and rate limiting
- Must integrate with existing authentication and authorization system

**Constraints:**
- Must use SambaNova API (OpenAI-compatible)
- API key stored in environment variables (.env)
- Chat sessions are per-user (privacy and isolation)
- Must not impact existing functionality
- Must be secure by default (rate limits, quotas, authorization)

**Stakeholders:**
- End users: Need intuitive chat interface with real-time responses
- Administrators: Need cost control and abuse prevention
- Developers: Need maintainable code following project patterns

## Goals / Non-Goals

**Goals:**
- ✅ Enable users to have AI-powered conversations with streaming responses
- ✅ Persist chat history with per-user sessions
- ✅ Prevent abuse through rate limiting and daily quotas
- ✅ Provide markdown rendering for rich content (code, tables, formatting)
- ✅ Follow DDD architecture with domain/application/infrastructure separation
- ✅ Maintain conversation context within token limits
- ✅ Integrate seamlessly with existing authentication system

**Non-Goals:**
- ❌ Multi-modal support (images, voice) - future enhancement
- ❌ Chat sharing or collaboration features - future enhancement
- ❌ Custom model selection - using single model initially
- ❌ Chat export functionality - future enhancement
- ❌ Search within conversations - future enhancement

## Decisions

### Decision 1: Use `async-openai` over `openai-api-rs`

**Rationale:**
- More mature library (1.4k+ stars vs ~200 stars)
- Better streaming support with proper error handling
- Active maintenance and community support
- Native custom base URL configuration for SambaNova API
- Production-ready with comprehensive type safety
- Built for Tokio async runtime (perfect fit with Axum)

**Alternatives considered:**
- `openai-api-rs`: User's initial suggestion, but less mature and fewer features
- Direct `reqwest` implementation: Too much manual work, reinventing the wheel

**Decision:** Use `async-openai = "0.20"`

### Decision 2: Server-Sent Events (SSE) for Streaming

**Rationale:**
- Native browser support via EventSource API (no additional libraries)
- Axum has built-in SSE support (`axum::response::sse`)
- Simpler than WebSockets for unidirectional streaming
- HTTP/2 compatible and works through proxies
- Graceful degradation possible (fallback to non-streaming)

**Alternatives considered:**
- WebSockets: Overkill for one-way streaming, more complex setup
- Long polling: Poor user experience, inefficient
- Chunked transfer encoding: Less browser-friendly

**Decision:** Use SSE with EventSource for real-time streaming

### Decision 3: Smart Context Windowing (Last 20 Messages)

**Rationale:**
- Token limits require limiting conversation history sent to API
- Recent context is more relevant than old messages
- Configurable via environment variable for flexibility
- Efficient database query with ORDER BY + LIMIT
- Maintains conversation coherency without excessive token usage

**Implementation:**
```rust
async fn build_context_messages(
    session_id: Uuid,
    max_messages: usize, // default: 20
    db: &DatabaseConnection,
) -> Result<Vec<ChatCompletionRequestMessage>>
```

**Alternatives considered:**
- Send entire conversation: Hits token limits quickly, expensive
- Sliding window with summary: Complex, requires additional API calls
- Smart chunking with embeddings: Overkill for initial implementation

**Decision:** Last N messages (default 20, configurable)

### Decision 4: Rate Limiting + Daily Quotas

**Rationale:**
- Prevent abuse and control costs
- Reuse existing Valkey/Redis infrastructure
- Two-tier protection: short-term (rate limit) + long-term (daily quota)
- Different limits for regular users vs admins

**Implementation:**
- Rate limit: 20 messages per minute (prevents spam)
- Daily quota: 100 messages per day (cost control)
- Admin users: Unlimited (configurable)
- Stored in Redis for performance

**Alternatives considered:**
- Token-based quotas: More complex to implement initially
- No quotas: Risk of cost overruns
- Hard limits only: Poor UX without graceful degradation

**Decision:** Combined rate limiting (minute) + daily quotas with Redis tracking

### Decision 5: Soft Delete for Chat Sessions

**Rationale:**
- Users may accidentally delete valuable conversations
- Allow recovery window before permanent deletion
- Compliance and audit trail requirements
- Storage can be reclaimed through archival process

**Implementation:**
- Add `deleted_at` timestamp column (NULL = not deleted)
- Filter out deleted sessions in normal queries
- Background job for permanent deletion after 90 days (future)

**Alternatives considered:**
- Hard delete: No recovery, potential data loss
- Trash folder: Additional complexity in UI
- Never delete: Storage costs grow unbounded

**Decision:** Soft delete with `deleted_at` timestamp

### Decision 6: Follow DDD Architecture

**Rationale:**
- Project mandates DDD for all backend features
- Separates business logic from infrastructure concerns
- Makes code more testable and maintainable
- Aligns with existing project structure

**Structure:**
```
backend/src/
├── domain/
│   └── chat/
│       ├── entity.rs           # ChatSession, ChatMessage entities
│       ├── value_objects.rs    # MessageRole, MessageContent
│       ├── repository.rs       # ChatRepository trait
│       └── service.rs          # Domain services (if needed)
├── application/
│   └── chat/
│       ├── create_session.rs   # CreateSessionUseCase
│       ├── send_message.rs     # SendMessageUseCase
│       └── list_sessions.rs    # ListSessionsUseCase
├── infrastructure/
│   ├── persistence/
│   │   └── chat_repository.rs  # ChatRepository implementation
│   └── llm/
│       └── sambanova_client.rs # SambaNova API adapter
└── presentation/
    └── handlers/
        └── chat.rs             # HTTP handlers (thin adapters)
```

**Alternatives considered:**
- Simpler layered architecture: Doesn't match project standards
- Feature-based organization: Less clear separation of concerns

**Decision:** Strict DDD with domain/application/infrastructure/presentation layers

## Risks / Trade-offs

### Risk 1: SambaNova API Downtime

**Impact:** High - Users cannot use chat feature
**Probability:** Low - SambaNova is a reliable service

**Mitigation:**
- Circuit breaker pattern (stop requests after 5 consecutive failures)
- Clear error messages to users
- Graceful degradation (show error state, don't break app)
- Optional: Queue failed requests for retry (future)

### Risk 2: Streaming Reliability

**Impact:** Medium - Poor UX if streams frequently fail
**Probability:** Medium - Network issues, browser compatibility

**Mitigation:**
- Fallback to non-streaming on SSE failure
- Comprehensive error handling in frontend
- Save partial responses for recovery
- Retry logic with exponential backoff
- Extensive testing across browsers

### Risk 3: Cost Overruns

**Impact:** High - Unexpected API costs
**Probability:** Medium - User behavior unpredictable

**Mitigation:**
- Strict rate limiting (20 msg/min per user)
- Daily quotas (100 msg/day per user)
- Monitoring and alerting on token usage
- Admin dashboard for cost tracking (future)
- Feature flag for quick disable if needed

### Risk 4: Database Performance

**Impact:** High - Slow queries affect entire application
**Probability:** Low - With proper indexing

**Mitigation:**
- Proper indexes on user_id, session_id, created_at
- Pagination for all list endpoints
- Connection pooling (existing infrastructure)
- Query optimization (SELECT only needed columns)
- Load testing before production deployment

### Trade-off: Context Window Size

**Choice:** Fixed window of last N messages vs intelligent summarization

**Pros of fixed window:**
- Simple to implement and understand
- Fast database queries
- Predictable token usage

**Cons of fixed window:**
- May lose important early context
- Less sophisticated than summarization

**Decision:** Start with fixed window (last 20 messages), iterate based on user feedback

### Trade-off: Streaming vs Non-Streaming

**Choice:** Implement both vs streaming-only

**Pros of both:**
- Better compatibility (fallback for SSE issues)
- Simpler testing

**Cons of both:**
- More code to maintain
- Two code paths to test

**Decision:** Implement streaming with graceful degradation to non-streaming on failure

## Migration Plan

This is a new feature, no migration needed. Implementation is additive only.

**Rollout Strategy:**
1. Phase 1: Deploy to staging, internal testing (week 1)
2. Phase 2: Enable for admin users only via feature flag (week 2)
3. Phase 3: Gradual rollout to 10% of users (week 3)
4. Phase 4: Monitor metrics, increase to 100% (week 4)

**Feature Flag:**
```bash
FEATURE_CHAT_ENABLED=true  # Enable/disable entire feature
```

**Rollback Plan:**
- Set `FEATURE_CHAT_ENABLED=false` to disable feature instantly
- No database rollback needed (tables can remain)
- Remove routes from router if needed
- No impact on existing functionality

## Open Questions

None at this time. All design decisions have been made based on:
- Requirements analysis
- Project constraints and conventions
- Technical feasibility assessment
- Cost-benefit analysis

Ready for implementation.
