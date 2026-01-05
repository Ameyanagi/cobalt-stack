# LLM Chatbot Feature Proposal

## Overview

Add an integrated AI chatbot feature powered by SambaNova's Llama-4-Maverick-17B-128E-Instruct model, providing users with conversational AI capabilities directly within the application.

## Library Selection Analysis

### Recommended: `async-openai` ✅

**Rationale:**
- **Most mature**: 1.4k+ stars, actively maintained, production-ready
- **Full streaming support**: Built-in Server-Sent Events (SSE) handling
- **Custom base URL**: Native support for SambaNova API via configuration
- **Type-safe**: Comprehensive type safety with excellent error handling
- **Async-first**: Built for Tokio, perfect integration with Axum
- **Better than `openai-api-rs`**: More features, better maintenance, superior streaming

**Configuration:**
```rust
use async_openai::{Client, config::OpenAIConfig};

let config = OpenAIConfig::new()
    .with_api_key(std::env::var("SAMBANOVA_API_KEY")?)
    .with_api_base("https://api.sambanova.ai/v1");

let client = Client::with_config(config);
```

## Architecture

### Database Schema

#### 1. `chat_sessions` Table
```sql
CREATE TABLE chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_chat_sessions_user_id ON chat_sessions(user_id);
CREATE INDEX idx_chat_sessions_created_at ON chat_sessions(created_at DESC);
```

#### 2. `chat_messages` Table
```sql
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    token_count INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_session FOREIGN KEY (session_id) REFERENCES chat_sessions(id)
);

CREATE INDEX idx_chat_messages_session_id ON chat_messages(session_id);
CREATE INDEX idx_chat_messages_created_at ON chat_messages(created_at);
```

### API Endpoints

#### Backend Routes (Axum)

1. **POST /api/v1/chat/sessions**
   - Create new chat session
   - Auth: Required (JWT)
   - Body: `{ "title"?: string }`
   - Returns: `{ "id": uuid, "title": string, "created_at": timestamp }`

2. **GET /api/v1/chat/sessions**
   - List user's chat sessions (paginated)
   - Auth: Required
   - Query: `?page=1&per_page=20`
   - Returns: `{ "sessions": [], "total": number, "page": number, "total_pages": number }`

3. **GET /api/v1/chat/sessions/:id**
   - Get session with message history
   - Auth: Required
   - Returns: `{ "session": {...}, "messages": [] }`

4. **POST /api/v1/chat/sessions/:id/messages**
   - Send message (streaming response)
   - Auth: Required
   - Body: `{ "content": string }`
   - Returns: SSE stream of assistant response chunks
   - Content-Type: `text/event-stream`

5. **DELETE /api/v1/chat/sessions/:id**
   - Soft delete session
   - Auth: Required
   - Returns: `{ "message": "Session deleted" }`

### Streaming Implementation

#### Backend (Axum SSE)
```rust
use axum::response::sse::{Event, Sse};
use futures::stream::Stream;

async fn stream_chat_response(
    session_id: Uuid,
    user_message: String,
    client: &Client<OpenAIConfig>,
    db: &DatabaseConnection,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // 1. Save user message
    save_message(session_id, "user", &user_message, db).await?;

    // 2. Build context from recent messages
    let context = build_context_messages(session_id, 20, db).await?;

    // 3. Create streaming request
    let request = CreateChatCompletionRequestArgs::default()
        .model("Llama-4-Maverick-17B-128E-Instruct")
        .messages(context)
        .stream(true)
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;

    // 4. Transform to SSE events
    let sse_stream = stream.map(|chunk| {
        match chunk {
            Ok(response) => {
                let content = response.choices[0].delta.content.clone();
                Event::default().data(json!({
                    "chunk": content,
                    "done": false
                }))
            }
            Err(e) => Event::default().data(json!({
                "error": e.to_string()
            }))
        }
    });

    // 5. Save complete assistant response when done
    // (accumulate chunks and save in background)

    Ok(Sse::new(sse_stream))
}
```

#### Frontend (EventSource)
```typescript
const sendMessage = async (sessionId: string, content: string) => {
  const eventSource = new EventSource(
    `/api/v1/chat/sessions/${sessionId}/messages?content=${encodeURIComponent(content)}`
  );

  let fullResponse = '';

  eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);

    if (data.error) {
      console.error('Stream error:', data.error);
      eventSource.close();
      return;
    }

    if (data.done) {
      eventSource.close();
      return;
    }

    fullResponse += data.chunk;
    setStreamingMessage(fullResponse);
  };

  eventSource.onerror = () => {
    eventSource.close();
  };
};
```

## Configuration

### Environment Variables

Add to `backend/.env`:
```bash
# SambaNova API Configuration
SAMBANOVA_API_KEY=your-api-key-here
SAMBANOVA_API_BASE=https://api.sambanova.ai/v1
SAMBANOVA_MODEL=Llama-4-Maverick-17B-128E-Instruct

# Chat Feature Settings
CHAT_MAX_CONTEXT_MESSAGES=20
CHAT_MAX_TOKENS=2048
CHAT_MAX_MESSAGE_LENGTH=4000
CHAT_DAILY_MESSAGE_QUOTA=100  # messages per user per day
CHAT_RATE_LIMIT_PER_MINUTE=20
```

## Security & Rate Limiting

### 1. Rate Limiting (Valkey/Redis)
```rust
// Reuse existing rate limiter
async fn check_chat_rate_limit(user_id: Uuid) -> Result<bool> {
    let config = RateLimitConfig {
        max_requests: 20,  // 20 messages per minute
        window_seconds: 60,
    };

    check_rate_limit(user_id, "chat_messages", config).await
}
```

### 2. User Quotas
```rust
// Track daily message count per user
struct UserChatQuota {
    user_id: Uuid,
    messages_today: i32,
    tokens_today: i32,
    last_reset: DateTime<Utc>,
    quota_limit: i32,
}

async fn check_daily_quota(user_id: Uuid) -> Result<bool> {
    let quota = get_or_create_quota(user_id).await?;

    // Reset if new day
    if quota.last_reset.date_naive() < Utc::now().date_naive() {
        reset_quota(user_id).await?;
        return Ok(true);
    }

    Ok(quota.messages_today < quota.quota_limit)
}
```

### 3. Input Validation
- Max message length: 4000 characters
- Sanitize user input to prevent injection
- Validate session ownership before operations

### 4. Authorization
- Middleware: Verify JWT token
- Session ownership: Users can only access their own sessions
- Admin role: Unlimited quotas

## Context Management

### Smart Context Windowing
```rust
async fn build_context_messages(
    session_id: Uuid,
    max_messages: usize,
    db: &DatabaseConnection,
) -> Result<Vec<ChatCompletionRequestMessage>> {
    // Get last N messages (chronological order)
    let messages = ChatMessages::find()
        .filter(chat_messages::Column::SessionId.eq(session_id))
        .order_by_desc(chat_messages::Column::CreatedAt)
        .limit(max_messages as u64)
        .all(db)
        .await?;

    let mut context = vec![
        // System message to set behavior
        ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage {
                content: "You are a helpful AI assistant.".to_string(),
                ..Default::default()
            }
        ),
    ];

    // Add conversation history (reverse to get chronological order)
    for msg in messages.iter().rev() {
        let message = match msg.role.as_str() {
            "user" => ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage::Text(msg.content.clone())
            ),
            "assistant" => ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessage {
                    content: Some(msg.content.clone()),
                    ..Default::default()
                }
            ),
            _ => continue,
        };
        context.push(message);
    }

    Ok(context)
}
```

**Benefits:**
- Maintains conversation coherency
- Stays within token limits
- Configurable window size
- Efficient database queries

## Frontend Components

### 1. ChatList Component
```tsx
// Sidebar showing all chat sessions
interface ChatListProps {
  sessions: ChatSession[];
  currentSessionId?: string;
  onSelectSession: (id: string) => void;
  onNewChat: () => void;
  onDeleteSession: (id: string) => void;
}

const ChatList: React.FC<ChatListProps> = ({...}) => {
  return (
    <div className="chat-list">
      <button onClick={onNewChat}>+ New Chat</button>
      <div className="session-list">
        {sessions.map(session => (
          <ChatSessionItem
            key={session.id}
            session={session}
            isActive={session.id === currentSessionId}
            onSelect={onSelectSession}
            onDelete={onDeleteSession}
          />
        ))}
      </div>
    </div>
  );
};
```

### 2. ChatWindow Component
```tsx
// Main chat interface
const ChatWindow: React.FC<{ sessionId: string }> = ({ sessionId }) => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);

  const sendMessage = async () => {
    if (!input.trim()) return;

    // Optimistic UI update
    const userMessage = { role: 'user', content: input };
    setMessages(prev => [...prev, userMessage]);
    setInput('');
    setIsStreaming(true);

    // Stream response
    await streamChatResponse(sessionId, input, (chunk) => {
      setMessages(prev => {
        const last = prev[prev.length - 1];
        if (last?.role === 'assistant') {
          return [...prev.slice(0, -1), { ...last, content: last.content + chunk }];
        }
        return [...prev, { role: 'assistant', content: chunk }];
      });
    });

    setIsStreaming(false);
  };

  return (
    <div className="chat-window">
      <MessageList messages={messages} />
      {isStreaming && <TypingIndicator />}
      <MessageInput
        value={input}
        onChange={setInput}
        onSend={sendMessage}
        disabled={isStreaming}
      />
    </div>
  );
};
```

### 3. MessageBubble Component
```tsx
// Individual message display with markdown rendering
const MessageBubble: React.FC<{ message: Message }> = ({ message }) => {
  return (
    <div className={`message ${message.role}`}>
      <div className="message-content">
        <ReactMarkdown
          remarkPlugins={[remarkGfm]}
          rehypePlugins={[rehypeHighlight]}
          components={{
            code: CodeBlock,  // Syntax highlighting
          }}
        >
          {message.content}
        </ReactMarkdown>
      </div>
      <div className="message-timestamp">
        {formatTimestamp(message.created_at)}
      </div>
    </div>
  );
};
```

## Error Handling

### Backend Error Scenarios

1. **SambaNova API Failures**
   - Network timeout → Retry with exponential backoff (3 attempts)
   - Invalid API key → Log error, return 503 Service Unavailable
   - Rate limit exceeded → Return 429 Too Many Requests
   - Model unavailable → Return user-friendly error message

2. **Database Errors**
   - Connection failure → Return 500 with retry suggestion
   - Constraint violations → Return 400 with validation details

3. **Streaming Interruptions**
   - Client disconnect → Cancel upstream request to SambaNova
   - Save partial response for recovery

### Frontend Error Handling

1. **User-friendly error messages**
2. **Retry button** for failed requests
3. **Draft message persistence** in localStorage
4. **Connection status indicator**
5. **Graceful degradation**: Fall back to non-streaming if SSE fails

## Performance Optimization

### Database Optimizations
- **Indexes**: user_id, session_id, created_at
- **Pagination**: Limit results to 20 per page
- **Query optimization**: SELECT only needed columns
- **Connection pooling**: Reuse database connections

### Caching Strategy
- **Valkey/Redis caching**:
  - Session titles: TTL 5 minutes
  - User's recent sessions: TTL 5 minutes
  - Rate limit counters: TTL 60 seconds

### API Optimizations
- **HTTP client reuse**: Single client instance
- **Connection pooling**: Persistent connections to SambaNova
- **Request timeout**: 30 seconds
- **Circuit breaker**: Stop requests after 5 consecutive failures

### Frontend Optimizations
- **Virtual scrolling**: For long chat histories
- **Lazy loading**: Load older messages on scroll
- **Debouncing**: Typing indicators
- **React.memo**: Minimize re-renders
- **Code splitting**: Lazy load chat components

## Cost Management

### User Quota System
- **Default quota**: 100 messages per day per user
- **Admin users**: Unlimited
- **Quota tracking**: Daily reset at midnight UTC
- **Quota display**: Show remaining messages in UI
- **Graceful degradation**: Allow viewing history when quota exceeded

### Storage Management
- **Soft delete**: Mark sessions as deleted, archive after 90 days
- **Hard delete**: Permanently remove after 1 year
- **Compression**: Compress old message content
- **Storage quotas**: Limit per-user storage

## Testing Strategy

### Backend Tests

1. **Unit Tests**
   - Message formatting functions
   - Context building logic
   - Token counting
   - Error handling utilities

2. **Integration Tests**
   - Session CRUD operations
   - Message persistence
   - Authorization checks
   - Rate limiting behavior

3. **Mock SambaNova API**
   ```rust
   #[cfg(test)]
   mod tests {
       use mockito::Server;

       #[tokio::test]
       async fn test_stream_chat_response() {
           let mut server = Server::new();
           let mock = server.mock("POST", "/v1/chat/completions")
               .with_status(200)
               .with_header("content-type", "text/event-stream")
               .with_body("data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n")
               .create();

           // Test streaming logic
           // ...
       }
   }
   ```

### Frontend Tests

1. **Component Tests**
   - MessageBubble rendering
   - ChatWindow interactions
   - ChatList display and actions

2. **Integration Tests**
   - Message sending flow
   - Streaming reception
   - Error handling
   - Session switching

3. **E2E Tests** (with Playwright)
   - Complete user journey
   - Create session → Send message → Receive response
   - Test across different browsers

## Implementation Phases

### Phase 1: MVP (Week 1) ⭐
**Goal**: Basic working chat without streaming

**Tasks:**
- [ ] Create database migrations for chat_sessions and chat_messages
- [ ] Add async-openai dependency to Cargo.toml
- [ ] Implement basic backend endpoints (non-streaming)
  - POST /chat/sessions (create session)
  - POST /chat/sessions/:id/messages (send message, non-streaming)
  - GET /chat/sessions/:id (get session with messages)
- [ ] Create basic frontend components
  - ChatWindow with message list
  - MessageInput component
  - Basic styling
- [ ] Configure SambaNova API client
- [ ] Test end-to-end with non-streaming responses

**Deliverable**: Users can create sessions and send/receive messages (no streaming)

### Phase 2: Streaming (Week 2) 🚀
**Goal**: Real-time streaming responses

**Tasks:**
- [ ] Implement SSE streaming in backend
  - Convert endpoint to return `Sse<Stream>`
  - Handle streaming from async-openai
  - Accumulate and save complete assistant response
- [ ] Frontend EventSource integration
  - Connect to SSE endpoint
  - Display chunks as they arrive
  - Handle stream completion
- [ ] Add typing indicator component
- [ ] Error handling for stream interruptions
- [ ] Test streaming reliability

**Deliverable**: Real-time streaming chat experience

### Phase 3: Polish (Week 3) ✨
**Goal**: Production-ready features

**Tasks:**
- [ ] Session management
  - GET /chat/sessions (list all sessions, paginated)
  - DELETE /chat/sessions/:id (soft delete)
  - Auto-generate session titles from first message
- [ ] UI improvements
  - Markdown rendering with react-markdown
  - Code syntax highlighting with rehype-highlight
  - Copy button for code blocks
  - Improved styling with shadcn/ui components
- [ ] Message history
  - Lazy loading of older messages
  - Scroll to bottom on new messages
  - Virtual scrolling for long histories
- [ ] Error handling improvements
  - User-friendly error messages
  - Retry failed requests
  - Connection status indicator

**Deliverable**: Polished, user-friendly chat interface

### Phase 4: Production Features (Week 4) 🔒
**Goal**: Security, performance, and monitoring

**Tasks:**
- [ ] Rate limiting
  - Implement per-user rate limits (20 msg/min)
  - Add rate limit headers to responses
- [ ] User quotas
  - Create user_chat_quotas table
  - Track daily message counts
  - Display quota in UI
  - Handle quota exceeded gracefully
- [ ] Performance optimizations
  - Add database indexes
  - Implement caching strategy
  - Connection pooling
  - Circuit breaker for API failures
- [ ] Monitoring and logging
  - Log API response times
  - Track token usage per user
  - Error rate monitoring
  - Alert on API failures
- [ ] Comprehensive testing
  - Unit tests (>80% coverage)
  - Integration tests
  - E2E tests
- [ ] Documentation
  - API documentation (OpenAPI/Swagger)
  - User guide
  - Admin guide

**Deliverable**: Production-ready, secure, monitored feature

## Deployment Strategy

### Feature Flags
```rust
// Use feature flags for gradual rollout
const CHAT_FEATURE_ENABLED: bool = std::env::var("FEATURE_CHAT_ENABLED")
    .unwrap_or_else(|_| "false".to_string())
    .parse()
    .unwrap_or(false);

// In route handler
if !CHAT_FEATURE_ENABLED {
    return Err(StatusCode::NOT_FOUND);
}
```

### Rollout Plan
1. **Week 1**: Deploy to staging, internal testing
2. **Week 2**: Enable for admin users only (beta test)
3. **Week 3**: Enable for 10% of users (A/B test)
4. **Week 4**: Monitor metrics, gradually increase to 100%

### Monitoring Metrics
- API response time (p50, p95, p99)
- Error rate (target: <1%)
- Streaming success rate (target: >98%)
- Token usage per user
- Daily active users of chat feature
- Session duration and message count

## Success Criteria

### Functional Requirements ✅
- [ ] Users can create multiple chat sessions
- [ ] Users can send messages and receive streaming responses
- [ ] Messages are persisted and can be retrieved
- [ ] Sessions can be listed and deleted
- [ ] Markdown and code rendering works correctly

### Non-Functional Requirements ✅
- [ ] Response time: <2 seconds for first token
- [ ] Streaming latency: <100ms between chunks
- [ ] Uptime: >99.5%
- [ ] Rate limiting: Effective prevention of abuse
- [ ] Error rate: <1%
- [ ] Test coverage: >80%

### User Experience ✅
- [ ] Intuitive UI matching existing design system
- [ ] Smooth streaming experience
- [ ] Clear error messages
- [ ] Quota visibility
- [ ] Mobile responsive

## Risk Mitigation

### Technical Risks

1. **SambaNova API Downtime**
   - **Mitigation**: Circuit breaker, graceful error messages, queue failed requests
   - **Impact**: Medium
   - **Probability**: Low

2. **Streaming Reliability Issues**
   - **Mitigation**: Fallback to non-streaming, retry logic, comprehensive testing
   - **Impact**: Medium
   - **Probability**: Medium

3. **Database Performance**
   - **Mitigation**: Proper indexing, pagination, caching, load testing
   - **Impact**: High
   - **Probability**: Low

### Business Risks

1. **API Cost Overruns**
   - **Mitigation**: User quotas, rate limiting, monitoring, alerts
   - **Impact**: High
   - **Probability**: Medium

2. **User Abuse**
   - **Mitigation**: Strict rate limiting, quotas, content moderation
   - **Impact**: Medium
   - **Probability**: Medium

### Security Risks

1. **Prompt Injection Attacks**
   - **Mitigation**: Input sanitization, system message protection
   - **Impact**: Medium
   - **Probability**: Medium

2. **Data Exposure**
   - **Mitigation**: Proper authorization checks, encrypted connections
   - **Impact**: High
   - **Probability**: Low

## Dependencies

### Backend
```toml
[dependencies]
async-openai = "0.20"  # SambaNova API client
```

### Frontend
```json
{
  "dependencies": {
    "react-markdown": "^10.1.0",
    "remark-gfm": "^4.0.1",
    "rehype-highlight": "^7.0.2"
  }
}
```

## Future Enhancements (Post-MVP)

1. **Multi-modal Support**
   - Image uploads in chat
   - File attachments
   - Voice input/output

2. **Advanced Features**
   - Chat sharing (public/private links)
   - Export chat history (PDF, TXT)
   - Search within conversations
   - Chat templates/prompts

3. **Collaboration**
   - Shared team chats
   - Commenting on messages
   - Chat permissions

4. **Analytics**
   - User engagement metrics
   - Popular topics
   - Usage patterns
   - Cost analysis dashboard

5. **Model Selection**
   - Allow users to choose different models
   - A/B testing different models
   - Custom fine-tuned models

## Appendix

### A. Example API Requests

#### Create Session
```bash
curl -X POST https://api.example.com/api/v1/chat/sessions \
  -H "Authorization: Bearer <jwt-token>" \
  -H "Content-Type: application/json" \
  -d '{"title": "My First Chat"}'
```

#### Send Message (Streaming)
```bash
curl -X POST https://api.example.com/api/v1/chat/sessions/<session-id>/messages \
  -H "Authorization: Bearer <jwt-token>" \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{"content": "What is Rust?"}'
```

### B. Database Migration Example

```rust
// migration/src/m20250127_000001_create_chat_tables.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create chat_sessions table
        manager.create_table(
            Table::create()
                .table(ChatSessions::Table)
                .if_not_exists()
                .col(ColumnDef::new(ChatSessions::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(ChatSessions::UserId).uuid().not_null())
                .col(ColumnDef::new(ChatSessions::Title).string().not_null())
                .col(ColumnDef::new(ChatSessions::CreatedAt).timestamp_with_time_zone().not_null())
                .col(ColumnDef::new(ChatSessions::UpdatedAt).timestamp_with_time_zone().not_null())
                .col(ColumnDef::new(ChatSessions::DeletedAt).timestamp_with_time_zone().null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_chat_sessions_user_id")
                        .from(ChatSessions::Table, ChatSessions::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

        // Create chat_messages table
        manager.create_table(
            Table::create()
                .table(ChatMessages::Table)
                .if_not_exists()
                .col(ColumnDef::new(ChatMessages::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(ChatMessages::SessionId).uuid().not_null())
                .col(ColumnDef::new(ChatMessages::Role).string().not_null())
                .col(ColumnDef::new(ChatMessages::Content).text().not_null())
                .col(ColumnDef::new(ChatMessages::TokenCount).integer().null())
                .col(ColumnDef::new(ChatMessages::CreatedAt).timestamp_with_time_zone().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_chat_messages_session_id")
                        .from(ChatMessages::Table, ChatMessages::SessionId)
                        .to(ChatSessions::Table, ChatSessions::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

        // Create indexes
        manager.create_index(
            Index::create()
                .name("idx_chat_sessions_user_id")
                .table(ChatSessions::Table)
                .col(ChatSessions::UserId)
                .to_owned(),
        )
        .await?;

        manager.create_index(
            Index::create()
                .name("idx_chat_messages_session_id")
                .table(ChatMessages::Table)
                .col(ChatMessages::SessionId)
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(ChatMessages::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(ChatSessions::Table).to_owned()).await?;
        Ok(())
    }
}
```

### C. Configuration Checklist

- [ ] Add SAMBANOVA_API_KEY to .env
- [ ] Add SAMBANOVA_API_BASE to .env
- [ ] Add SAMBANOVA_MODEL to .env
- [ ] Configure CHAT_MAX_CONTEXT_MESSAGES
- [ ] Configure CHAT_DAILY_MESSAGE_QUOTA
- [ ] Configure CHAT_RATE_LIMIT_PER_MINUTE
- [ ] Add async-openai to Cargo.toml
- [ ] Run database migrations
- [ ] Update .env.example with new variables
- [ ] Configure monitoring/alerting for chat endpoints

## Conclusion

This proposal outlines a comprehensive plan for adding an LLM chatbot feature powered by SambaNova's API. The phased implementation approach ensures incremental delivery of value while managing technical risks. The recommended use of `async-openai` provides a robust, production-ready foundation for streaming chat responses.

**Estimated Timeline**: 4 weeks for full implementation
**Estimated Effort**: 1-2 developers
**Risk Level**: Medium (mitigated through phased rollout and comprehensive testing)

**Next Steps:**
1. Review and approve proposal
2. Set up SambaNova API access
3. Begin Phase 1 implementation (MVP)
4. Schedule weekly progress reviews
