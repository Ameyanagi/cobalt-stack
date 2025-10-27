# Chat Feature Setup & Testing Guide

## Prerequisites Checklist

Before starting the chat feature, ensure the following services are running:

### 1. PostgreSQL Database
```bash
# Check if PostgreSQL is running
pg_isready -h localhost -p 5432

# If not running, start PostgreSQL:
# On Ubuntu/Debian
sudo systemctl start postgresql

# On macOS (Homebrew)
brew services start postgresql

# On Docker
docker run -d \
  --name postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=cobalt_dev \
  -p 5432:5432 \
  postgres:16
```

### 2. Redis/Valkey (Required for Rate Limiting)
```bash
# Check if Redis is running
redis-cli ping

# If not running, start Redis:
# On Ubuntu/Debian
sudo systemctl start redis

# On macOS (Homebrew)
brew services start redis

# On Docker
docker run -d \
  --name redis \
  -p 6379:6379 \
  redis:7-alpine
```

### 3. SambaNova API Key
✅ **Already configured**: `33938d57-5ac4-47f2-b663-c41d5db55d8a`

## Quick Start

### Step 1: Environment Configuration

Your `.env` file is already configured with:
```bash
FEATURE_CHAT_ENABLED=true
SAMBANOVA_API_KEY=33938d57-5ac4-47f2-b663-c41d5db55d8a
VALKEY_URL=redis://localhost:6379
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/cobalt_dev
```

### Step 2: Database Setup

```bash
cd backend

# Run migrations to create chat tables
sea-orm-cli migrate up

# Expected output:
# Applying migration 'm20250127_000001_create_chat_tables'
# Migration applied successfully
```

**Tables Created:**
- `chat_sessions` - User chat sessions with titles
- `chat_messages` - Individual messages (user/assistant)

### Step 3: Start Backend

```bash
cd backend

# Build and run (development mode)
cargo run

# Expected output:
# Database connected
# Valkey connected for chat rate limiting
# Chat feature enabled - mounting chat routes with rate limiting
# Starting server on 0.0.0.0:3000
```

### Step 4: Start Frontend

```bash
cd frontend

# Install dependencies (if not already done)
bun install

# Start development server
bun run dev

# Expected output:
# ready - started server on 0.0.0.0:3001
```

### Step 5: Access Chat

Open your browser and navigate to:
```
http://localhost:3001/chat
```

## Testing the Chat Feature

### 1. Authentication Test

First, you need to be logged in. If not already authenticated:

```bash
# Register a new user
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "SecurePass123!",
    "name": "Test User"
  }'

# Login to get JWT token
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "SecurePass123!"
  }'

# Copy the "access_token" from response
```

### 2. Create Chat Session

```bash
# Replace <TOKEN> with your access token
curl -X POST http://localhost:3000/api/v1/chat/sessions \
  -H "Authorization: Bearer <TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"title": "My First Chat"}'

# Expected response:
{
  "session_id": "uuid-here",
  "message": "Chat session created"
}
```

### 3. Send Message (SSE Stream)

```bash
# Replace <SESSION_ID> and <TOKEN>
curl -X POST http://localhost:3000/api/v1/chat/sessions/<SESSION_ID>/messages \
  -H "Authorization: Bearer <TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello! How are you?"}' \
  --no-buffer

# Expected response (streaming):
data: {"content":"Hello"}
data: {"content":"!"}
data: {"content":" I'm"}
data: {"content":" doing"}
data: {"content":" well"}
data: [DONE]
```

**Check Rate Limit Headers:**
```bash
curl -X POST http://localhost:3000/api/v1/chat/sessions/<SESSION_ID>/messages \
  -H "Authorization: Bearer <TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"content": "Test message"}' \
  -i | grep -i x-ratelimit

# Expected headers:
X-RateLimit-Limit-Minute: 20
X-RateLimit-Remaining-Minute: 19
X-RateLimit-Reset-Minute: 1706356267
X-RateLimit-Limit-Daily: 100
X-RateLimit-Remaining-Daily: 99
X-RateLimit-Reset-Daily: 1706442667
```

### 4. Test Rate Limiting

```bash
# Send 21 messages rapidly to trigger per-minute limit
for i in {1..21}; do
  curl -X POST http://localhost:3000/api/v1/chat/sessions/<SESSION_ID>/messages \
    -H "Authorization: Bearer <TOKEN>" \
    -H "Content-Type: application/json" \
    -d "{\"content\": \"Message $i\"}" \
    --no-buffer
  echo ""
done

# After 20 messages, you should get:
{
  "error": "Rate limit exceeded",
  "limit_type": "per_minute",
  "limit": 20,
  "current": 20,
  "retry_after": 45,
  "message": "You have exceeded the per_minute rate limit. Please try again in 45 seconds."
}
```

### 5. List Sessions

```bash
curl -X GET http://localhost:3000/api/v1/chat/sessions \
  -H "Authorization: Bearer <TOKEN>"

# Expected response:
{
  "sessions": [
    {
      "id": "uuid",
      "user_id": "uuid",
      "title": "My First Chat",
      "created_at": "2025-01-27T10:00:00Z",
      "updated_at": "2025-01-27T10:05:00Z"
    }
  ],
  "total": 1
}
```

### 6. Get Session History

```bash
curl -X GET http://localhost:3000/api/v1/chat/sessions/<SESSION_ID>/history \
  -H "Authorization: Bearer <TOKEN>"

# Expected response:
{
  "session": { ... },
  "messages": [
    {
      "id": "uuid",
      "session_id": "uuid",
      "role": "user",
      "content": "Hello! How are you?",
      "created_at": "2025-01-27T10:01:00Z"
    },
    {
      "id": "uuid",
      "session_id": "uuid",
      "role": "assistant",
      "content": "Hello! I'm doing well, thank you for asking!",
      "created_at": "2025-01-27T10:01:02Z"
    }
  ]
}
```

### 7. Delete Session

```bash
curl -X DELETE http://localhost:3000/api/v1/chat/sessions/<SESSION_ID> \
  -H "Authorization: Bearer <TOKEN>"

# Expected response:
{
  "message": "Chat session deleted successfully"
}
```

## Frontend Testing

### Manual UI Testing Checklist

1. **Authentication**
   - [ ] Login redirects to /chat
   - [ ] Logout clears chat state
   - [ ] Unauthorized access redirects to login

2. **Session Management**
   - [ ] "New Chat" button creates session
   - [ ] Sessions list displays all user sessions
   - [ ] Clicking session loads messages
   - [ ] Delete button shows confirmation dialog
   - [ ] Deleting session removes from list

3. **Messaging**
   - [ ] Text input accepts multi-line (Shift+Enter)
   - [ ] Send button sends message (Enter key)
   - [ ] User message appears immediately
   - [ ] Assistant response streams token-by-token
   - [ ] Markdown renders correctly
   - [ ] Code blocks have syntax highlighting
   - [ ] Auto-scroll to new messages

4. **Rate Limiting**
   - [ ] Rate limit indicator shows quotas
   - [ ] Progress bars update after each message
   - [ ] Warning color at 80% usage
   - [ ] 429 error shows user-friendly message
   - [ ] Retry-after countdown displayed

5. **Error Handling**
   - [ ] Network errors show error message
   - [ ] Rate limit errors show retry time
   - [ ] Invalid sessions show error
   - [ ] SSE connection errors handled

## Troubleshooting

### Backend Won't Start

**Problem**: Database connection error
```
Error: Connection refused (os error 111)
```
**Solution**: Start PostgreSQL and verify DATABASE_URL

**Problem**: Valkey connection error
```
Error: Connection refused (redis)
```
**Solution**: Start Redis/Valkey and verify VALKEY_URL

**Problem**: Chat feature not enabled
```
Chat feature disabled
```
**Solution**: Set `FEATURE_CHAT_ENABLED=true` in .env

### Rate Limiting Issues

**Problem**: Always shows 0 remaining
```bash
# Debug Redis keys
redis-cli KEYS "ratelimit:chat:*"
redis-cli GET "ratelimit:chat:user:<UUID>:minute"

# Check TTL
redis-cli TTL "ratelimit:chat:user:<UUID>:minute"
```

**Problem**: Rate limits not resetting
```bash
# Manually reset for testing
redis-cli DEL "ratelimit:chat:user:<UUID>:minute"
redis-cli DEL "quota:chat:user:<UUID>:daily"
```

### SSE Streaming Issues

**Problem**: No streaming response
- Check browser Network tab for EventStream
- Verify Content-Type: text/event-stream
- Check SAMBANOVA_API_KEY is valid

**Problem**: Incomplete messages
- Increase CHAT_MAX_TOKENS in .env
- Check backend logs for stream errors
- Verify SambaNova API status

### Frontend Issues

**Problem**: API calls failing with CORS
- Verify CORS_ORIGINS in backend .env
- Check NEXT_PUBLIC_API_URL in frontend .env.local
- Inspect browser console for CORS errors

**Problem**: TypeScript errors
```bash
cd frontend
bun run typecheck
```

**Problem**: Chat page not accessible
- Verify route exists: frontend/src/app/chat/page.tsx
- Check authentication redirects
- Review browser console for errors

## Verification Commands

### Backend Health Check
```bash
# Health endpoint
curl http://localhost:3000/health

# OpenAPI docs
curl http://localhost:3000/openapi.json | jq '.paths | keys | .[] | select(contains("chat"))'

# Expected output:
"/api/v1/chat/sessions"
"/api/v1/chat/sessions/{session_id}"
"/api/v1/chat/sessions/{session_id}/history"
"/api/v1/chat/sessions/{session_id}/messages"
```

### Database Verification
```bash
# Check tables exist
psql -h localhost -U postgres -d cobalt_dev -c "\dt chat*"

# Expected output:
# chat_messages
# chat_sessions

# Check session count
psql -h localhost -U postgres -d cobalt_dev -c "SELECT COUNT(*) FROM chat_sessions;"
```

### Redis Verification
```bash
# Check rate limit keys
redis-cli KEYS "ratelimit:chat:*"

# Check daily quota keys
redis-cli KEYS "quota:chat:*"

# Monitor real-time commands
redis-cli MONITOR
```

## Performance Testing

### Load Testing with curl

```bash
# Test 100 concurrent sessions
for i in {1..100}; do
  (
    TOKEN="your-token-here"
    SESSION_ID=$(curl -s -X POST http://localhost:3000/api/v1/chat/sessions \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d '{"title":"Load Test"}' | jq -r '.session_id')

    curl -X POST http://localhost:3000/api/v1/chat/sessions/$SESSION_ID/messages \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d '{"content":"Hello"}' \
      --no-buffer &
  )
done
wait
```

### Monitoring

```bash
# Watch backend logs
tail -f backend/logs/app.log

# Monitor Redis memory
redis-cli INFO memory | grep used_memory_human

# Monitor PostgreSQL connections
psql -h localhost -U postgres -d cobalt_dev -c "SELECT count(*) FROM pg_stat_activity;"
```

## Next Steps

Once everything is working:

1. **User Testing**: Test with real users for feedback
2. **Performance Optimization**: Monitor and optimize slow queries
3. **Error Monitoring**: Set up Sentry or similar for error tracking
4. **Analytics**: Track usage metrics (messages per user, popular times)
5. **Content Moderation**: Add filtering for inappropriate content
6. **Backup Strategy**: Regular database backups of chat data
7. **Scaling**: Consider connection pooling, caching strategies

## Support Resources

- **Documentation**: `/docs/chat-feature.md`
- **API Reference**: `http://localhost:3000/swagger-ui`
- **Backend Logs**: Check RUST_LOG=debug output
- **Frontend DevTools**: Browser console for errors
- **Database Logs**: PostgreSQL logs for query issues

---

**Setup completed!** 🎉

Your chat feature is now configured with:
- ✅ SambaNova API key: `33938...5d8a`
- ✅ Feature enabled: `FEATURE_CHAT_ENABLED=true`
- ✅ Rate limiting configured: 20/min, 100/day
- ✅ Database schema ready
- ✅ Frontend UI ready

**Access the chat**: http://localhost:3001/chat
