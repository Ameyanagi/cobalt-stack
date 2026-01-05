# LLM Chat Feature Documentation

## Overview

The LLM Chat feature provides authenticated users with real-time AI conversations through a server-sent events (SSE) streaming interface. It includes comprehensive rate limiting, session management, and quota tracking to prevent abuse while providing a smooth user experience.

## Architecture

### Backend (Rust/Axum)

**Domain-Driven Design (DDD) Layers:**

```
┌─────────────────────────────────────────┐
│         HTTP Handlers (Axum)            │  ← REST API endpoints
├─────────────────────────────────────────┤
│     Application Service Layer           │  ← Business orchestration
├─────────────────────────────────────────┤
│        Domain Layer (Entities)          │  ← Business logic & rules
├─────────────────────────────────────────┤
│   Infrastructure (Repository/LLM)       │  ← External integrations
└─────────────────────────────────────────┘
```

**Middleware Stack:**
```
Request → JWT Auth → Rate Limit → Handler
```

**Key Components:**

1. **Domain Layer** (`backend/src/domain/chat/`)
   - `ChatSession`: Session entity with title and timestamps
   - `ChatMessage`: Message entity with role (user/assistant) and content
   - `ChatRepository`: Repository trait for data persistence

2. **Application Layer** (`backend/src/application/chat/`)
   - `create_session`: Create new chat sessions
   - `send_message`: Send message and stream LLM response
   - `get_session_history`: Retrieve conversation history
   - `list_user_sessions`: List all user sessions
   - `delete_session`: Delete session and messages

3. **Infrastructure Layer** (`backend/src/infrastructure/persistence/`)
   - `SeaOrmChatRepository`: PostgreSQL implementation via SeaORM
   - Database tables: `chat_sessions`, `chat_messages`

4. **Handlers** (`backend/src/handlers/chat/`)
   - RESTful API endpoints with OpenAPI documentation
   - SSE streaming for real-time LLM responses

5. **Rate Limiting** (`backend/src/services/valkey/chat_rate_limit.rs`)
   - Two-tier Redis/Valkey-based rate limiting:
     - **Per-minute**: 20 messages/minute (configurable)
     - **Daily quota**: 100 messages/day (configurable)
   - Atomic operations with TTL auto-expiry
   - Transparent quota information via response headers

### Frontend (Next.js/React)

**Component Architecture:**

```
ChatContainer (Main)
├── SessionSidebar (Session management)
│   └── Session list with create/delete
├── RateLimitIndicator (Quota display)
├── MessageList (Message display)
│   └── Message (Individual message with markdown)
└── MessageInput (Text input)
```

**Key Features:**

1. **Real-time Streaming**: SSE parser for token-by-token LLM responses
2. **Markdown Rendering**: Full markdown support with syntax highlighting
3. **Rate Limit Transparency**: Visual quota indicators with warnings
4. **Session Management**: Create, switch, and delete chat sessions
5. **Error Handling**: Graceful handling of rate limit errors

## API Endpoints

### Base URL
```
http://localhost:3000/api/v1/chat
```

All endpoints require JWT authentication via `Authorization: Bearer <token>` header.

### 1. Create Session
```http
POST /sessions
Content-Type: application/json

{
  "title": "Optional session title"
}
```

**Response:**
```json
{
  "session_id": "uuid",
  "message": "Chat session created"
}
```

### 2. Send Message (SSE Stream)
```http
POST /sessions/{session_id}/messages
Content-Type: application/json

{
  "content": "Your message here"
}
```

**Response:** Server-Sent Events stream
```
data: {"content":"Hello"}
data: {"content":" there"}
data: {"content":"!"}
data: [DONE]
```

**Response Headers:**
```
X-RateLimit-Limit-Minute: 20
X-RateLimit-Remaining-Minute: 15
X-RateLimit-Reset-Minute: 1643234567
X-RateLimit-Limit-Daily: 100
X-RateLimit-Remaining-Daily: 85
X-RateLimit-Reset-Daily: 1643320967
```

**Rate Limit Exceeded (429):**
```json
{
  "error": "Rate limit exceeded",
  "limit_type": "per_minute",
  "limit": 20,
  "current": 20,
  "retry_after": 45,
  "message": "You have exceeded the per_minute rate limit. Please try again in 45 seconds."
}
```

### 3. Get Session History
```http
GET /sessions/{session_id}/history
```

**Response:**
```json
{
  "session": {
    "id": "uuid",
    "user_id": "uuid",
    "title": "My Chat",
    "created_at": "2025-01-27T10:00:00Z",
    "updated_at": "2025-01-27T11:00:00Z"
  },
  "messages": [
    {
      "id": "uuid",
      "session_id": "uuid",
      "role": "user",
      "content": "Hello",
      "created_at": "2025-01-27T10:01:00Z"
    },
    {
      "id": "uuid",
      "session_id": "uuid",
      "role": "assistant",
      "content": "Hi! How can I help?",
      "created_at": "2025-01-27T10:01:02Z"
    }
  ]
}
```

### 4. List User Sessions
```http
GET /sessions
```

**Response:**
```json
{
  "sessions": [
    {
      "id": "uuid",
      "user_id": "uuid",
      "title": "My Chat",
      "created_at": "2025-01-27T10:00:00Z",
      "updated_at": "2025-01-27T11:00:00Z"
    }
  ],
  "total": 1
}
```

### 5. Delete Session
```http
DELETE /sessions/{session_id}
```

**Response:**
```json
{
  "message": "Chat session deleted successfully"
}
```

## Configuration

### Backend Environment Variables

```bash
# Enable chat feature
FEATURE_CHAT_ENABLED=true

# SambaNova API configuration
SAMBANOVA_API_KEY=your-api-key-here
SAMBANOVA_API_BASE=https://api.sambanova.ai/v1
SAMBANOVA_MODEL=Llama-4-Maverick-17B-128E-Instruct

# Chat settings
CHAT_MAX_CONTEXT_MESSAGES=20      # Max messages in conversation context
CHAT_MAX_TOKENS=2048               # Max tokens per LLM response
CHAT_MAX_MESSAGE_LENGTH=4000       # Max characters per user message

# Rate limiting
CHAT_RATE_LIMIT_PER_MINUTE=20     # Messages per minute per user
CHAT_DAILY_MESSAGE_QUOTA=100      # Messages per day per user

# Valkey/Redis (required for rate limiting)
VALKEY_URL=redis://localhost:6379
```

### Frontend Environment Variables

```bash
# Backend API URL
NEXT_PUBLIC_API_URL=http://localhost:3000
```

## Database Schema

### chat_sessions
```sql
CREATE TABLE chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chat_sessions_user_id ON chat_sessions(user_id);
```

### chat_messages
```sql
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('user', 'assistant')),
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chat_messages_session_id ON chat_messages(session_id);
CREATE INDEX idx_chat_messages_created_at ON chat_messages(created_at);
```

## Rate Limiting

### Two-Tier System

**1. Per-Minute Rate Limit** (Burst Protection)
- Default: 20 messages/minute
- Redis key: `ratelimit:chat:user:{user_id}:minute`
- TTL: 60 seconds
- Purpose: Prevent rapid-fire spam

**2. Daily Quota** (Cost Control)
- Default: 100 messages/day
- Redis key: `quota:chat:user:{user_id}:daily`
- TTL: 86400 seconds (24 hours)
- Purpose: Prevent long-term abuse

### How It Works

1. **Check Phase**: Both limits checked before processing
2. **Fast Fail**: Per-minute checked first (cheaper operation)
3. **Increment**: Counters incremented only if both checks pass
4. **Transparency**: Current usage returned in response headers
5. **Auto-Reset**: TTL ensures automatic cleanup

### Rate Limit Response Headers

```
X-RateLimit-Limit-Minute: 20           # Max per minute
X-RateLimit-Remaining-Minute: 15       # Remaining this minute
X-RateLimit-Reset-Minute: 1643234567   # Unix timestamp for reset

X-RateLimit-Limit-Daily: 100           # Max per day
X-RateLimit-Remaining-Daily: 85        # Remaining today
X-RateLimit-Reset-Daily: 1643320967    # Unix timestamp for reset
```

## Testing

### Backend Tests

Run all tests:
```bash
cd backend
cargo test --lib
```

**Test Coverage:**
- Domain layer: Entity creation and validation
- Application layer: Use case logic and error handling
- Infrastructure layer: Repository CRUD operations
- Rate limiting: Two-tier limit enforcement

**Test Results:**
```
116 tests passing
- Domain tests: 8
- Application tests: 20
- Infrastructure tests: 15
- Rate limit tests: 4
- Other backend tests: 69
```

### Frontend Development

Start development server:
```bash
cd frontend
bun install
bun run dev
```

TypeScript type checking:
```bash
bun run typecheck
```

## Deployment Checklist

### Prerequisites
1. ✅ PostgreSQL database running
2. ✅ Redis/Valkey instance running
3. ✅ SambaNova API key obtained
4. ✅ Backend migrations applied

### Backend Setup

1. **Run migrations:**
```bash
cd backend
sea-orm-cli migrate up
```

2. **Configure environment:**
```bash
cp .env.example .env
# Edit .env with your settings
```

3. **Enable chat feature:**
```bash
FEATURE_CHAT_ENABLED=true
SAMBANOVA_API_KEY=your-key
VALKEY_URL=redis://localhost:6379
```

4. **Start backend:**
```bash
cargo run
```

### Frontend Setup

1. **Configure API URL:**
```bash
cd frontend
cp .env.local.example .env.local
# Set NEXT_PUBLIC_API_URL=http://localhost:3000
```

2. **Install dependencies:**
```bash
bun install
```

3. **Start frontend:**
```bash
bun run dev
```

### Access

- **Frontend**: http://localhost:3001/chat
- **Backend API**: http://localhost:3000/api/v1/chat
- **OpenAPI Docs**: http://localhost:3000/swagger-ui

## Troubleshooting

### Rate Limiting Issues

**Problem**: Rate limit always returns 0 remaining
```bash
# Check Valkey connection
redis-cli -u redis://localhost:6379 PING

# Inspect rate limit keys
redis-cli -u redis://localhost:6379 KEYS "ratelimit:chat:*"
redis-cli -u redis://localhost:6379 GET "ratelimit:chat:user:{user_id}:minute"
```

**Problem**: Rate limit not resetting
```bash
# Check TTL on keys
redis-cli -u redis://localhost:6379 TTL "ratelimit:chat:user:{user_id}:minute"
```

### SSE Streaming Issues

**Problem**: Stream not working in browser
- Check browser DevTools Network tab for EventStream type
- Verify `Content-Type: text/event-stream` header
- Ensure no proxy buffering responses

**Problem**: Incomplete messages
- Check CHAT_MAX_TOKENS setting
- Verify SambaNova API connectivity
- Review backend logs for stream errors

### Database Issues

**Problem**: Session not found
```sql
-- Check session exists and belongs to user
SELECT * FROM chat_sessions WHERE id = 'session-uuid' AND user_id = 'user-uuid';
```

**Problem**: Messages not saving
```sql
-- Check foreign key constraints
SELECT * FROM chat_messages WHERE session_id = 'session-uuid';
```

## Performance Considerations

### Backend

- **Connection Pooling**: Consider r2d2 or bb8 for production Valkey pooling
- **Database Indexes**: Ensure indexes on `user_id` and `session_id`
- **Context Window**: Limit CHAT_MAX_CONTEXT_MESSAGES to reduce memory
- **Streaming**: SSE more efficient than polling for real-time responses

### Frontend

- **Lazy Loading**: Sessions list virtualized for 100+ sessions
- **Message Caching**: React Query caches session history
- **Optimistic Updates**: User messages appear immediately
- **Auto-scroll**: Smooth scroll to new messages without janky jumps

## Security Notes

1. **JWT Authentication**: All endpoints require valid JWT token
2. **User Isolation**: Queries filtered by authenticated user_id
3. **SQL Injection**: SeaORM provides query parameterization
4. **Rate Limiting**: Prevents abuse and DoS attacks
5. **Content Filtering**: Consider adding content moderation layer
6. **API Keys**: Store SAMBANOVA_API_KEY securely, never in frontend

## Future Enhancements

### Potential Improvements

1. **Streaming History**: Paginate message history for long conversations
2. **Message Reactions**: Allow users to rate LLM responses
3. **Export Conversations**: Download chat history as markdown/JSON
4. **Conversation Branching**: Fork conversations at any message
5. **Multi-model Support**: Switch between different LLM providers
6. **Voice Input**: Speech-to-text integration
7. **Code Execution**: Sandboxed code execution for assistant
8. **File Attachments**: Upload documents for context
9. **Collaborative Chats**: Share sessions with other users
10. **Analytics Dashboard**: Usage metrics and insights

### Scaling Considerations

- **Horizontal Scaling**: Stateless backend supports load balancing
- **Database Sharding**: Partition by user_id for large user bases
- **CDN**: Cache static assets and frontend build
- **Redis Cluster**: Distributed rate limiting for multi-region
- **WebSockets**: Consider for bidirectional communication
- **Message Queue**: Async processing for heavy workloads

## Support

For issues or questions:
1. Check backend logs: `RUST_LOG=debug cargo run`
2. Review OpenAPI documentation: http://localhost:3000/swagger-ui
3. Consult source code documentation in-line comments
4. Test rate limits with: `curl -H "Authorization: Bearer <token>" ...`
