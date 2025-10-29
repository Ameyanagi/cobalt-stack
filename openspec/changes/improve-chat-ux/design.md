# Design: Improve Chat UX

## Architecture Overview

This change enhances the chat interface through three independent improvements that work together to provide a better user experience.

```
┌─────────────────────────────────────────────────────────────┐
│                     Landing Page / Header                    │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────┐             │
│  │  Docs  │ │API Docs│ │ Health │ │ 💬 Chat  │ (NEW)       │
│  └────────┘ └────────┘ └────────┘ └──────────┘             │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
        ┌───────────────────────────────────────┐
        │          Chat Interface               │
        │  ┌─────────────────┐ ┌──────────────┐│
        │  │  Session List   │ │ Chat Area    ││
        │  │  • Auto-titled  │ │              ││
        │  │    sessions     │ │  Model Select││
        │  │    (LLM gen.)   │ │  (from /models)
        │  └─────────────────┘ └──────────────┘│
        └───────────────────────────────────────┘
                            │
                            ▼
        ┌───────────────────────────────────────┐
        │       Backend API                     │
        │  GET /api/v1/chat/models (existing)   │
        │  PATCH /api/v1/chat/sessions/{id}/... │
        │         ...generate-title (NEW)       │
        └───────────────────────────────────────┘
```

## Component 1: Backend-Driven Model Filtering

### Current Flow
```
Frontend Load → Static models.ts → Display All Models
                     ↓
              (may not match backend)
```

### New Flow
```
Frontend Load → GET /api/v1/chat/models → Filter by .enabled
                      ↓
              Display Only Enabled Models
```

### Data Flow

1. **On Chat Mount**: Frontend calls `GET /api/v1/chat/models`
2. **Backend Response**: Returns only enabled models from `ModelRegistry`
3. **Frontend State**: Replaces static config with API response
4. **UI Update**: ModelSelector shows only available models
5. **Caching**: Store in component state for session duration

### API Contract (Existing)

**Endpoint**: `GET /api/v1/chat/models`

**Response**:
```json
{
  "models": [
    {
      "id": "grok-4-fast-non-reasoning",
      "name": "Grok 4 Fast",
      "provider": "Azure AI (xAI)",
      "context_window": 128000,
      "supports_streaming": true,
      ...
    }
  ],
  "groups": [
    {
      "name": "Fast Models",
      "models": ["grok-4-fast-non-reasoning", ...]
    }
  ],
  "default_model": "llama-3.3-70b"
}
```

### Frontend Changes

**Remove**: `frontend/src/config/models.ts` static exports
**Modify**: `chat-container.tsx` to fetch models on mount
**Add**: `use-models.ts` hook for model fetching and caching

## Component 2: Auto-Generated Titles

### Title Generation Flow

```
User sends message → Assistant responds → Frontend detects first response
                          ↓
              PATCH /api/v1/chat/sessions/{id}/generate-title
                          ↓
              Backend: Prompt LLM with first exchange
                          ↓
              LLM generates concise title (4-8 words)
                          ↓
              Update session.title in database
                          ↓
              Return updated session to frontend
                          ↓
              Frontend updates sidebar
```

### Title Generation Prompt

```
Given this conversation, generate a concise, descriptive title (4-8 words max):

User: {first_user_message}
Assistant: {first_assistant_response}

Title:
```

### Backend Implementation

**New Use Case**: `generate_session_title::GenerateSessionTitleUseCase`

**Location**: `backend/src/application/chat/generate_session_title.rs`

**Dependencies**:
- `ChatRepository`: Load first 2 messages
- `ProviderFactory`: Get LLM for title generation
- Session ownership validation

**Error Handling**:
- **Not Found**: Return 404 if session doesn't exist
- **Unauthorized**: Return 403 if user doesn't own session
- **LLM Failure**: Log error, keep existing title, return 200 with current session
- **Empty Response**: Use fallback "Chat about {topic}" format

**New Handler**: `handlers/chat/generate_title.rs`

**Route**: `PATCH /api/v1/chat/sessions/{id}/generate-title`

**OpenAPI Spec**:
```yaml
paths:
  /api/v1/chat/sessions/{id}/generate-title:
    patch:
      summary: Generate title for chat session
      description: Uses LLM to create descriptive title based on first message exchange
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Title generated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ChatSession'
        '404':
          description: Session not found
        '403':
          description: Unauthorized
```

### Frontend Integration

**Trigger**: After first assistant message completes streaming

**Location**: `chat-container.tsx` in `parseStream` callback

**Pseudo-code**:
```typescript
await parseStream(
  stream,
  (chunk) => { /* accumulate content */ },
  async () => {
    // Stream complete
    setMessages(/* add assistant message */);

    // Auto-generate title if this is first exchange
    if (messages.length === 0) { // Was empty before user message
      try {
        const updated = await generateSessionTitle(currentSession.id);
        setCurrentSession(updated);
        // Sidebar updates automatically via state
      } catch (err) {
        // Silently fail - keep "New Chat" title
        console.warn('Failed to generate title:', err);
      }
    }
  }
);
```

**Non-Blocking**: Title generation happens async, doesn't block chat

## Component 3: Chat Navigation

### Navigation Locations

1. **Header** (All Pages)
   - Location: After "API Docs", before theme selectors
   - Icon: MessageSquare (💬)
   - Label: "Chat"
   - Style: Button variant="ghost"

2. **Landing Page** (Hero Section)
   - Location: Next to "Dashboard" / "Get Started" button
   - Label: "Try Chat"
   - Style: Button variant="outline"
   - Conditional: Only show if `FEATURE_CHAT_ENABLED`

3. **Landing Page** (Features Grid)
   - Add new feature card for Chat
   - Title: "AI Chat Assistant"
   - Description: "Multi-model LLM chat with conversation management"
   - Features:
     - Multiple AI models (Llama, GPT, Grok)
     - Streaming responses
     - Session management

### Implementation

**File**: `frontend/src/app/page.tsx`

**Header Addition**:
```tsx
<Link href="/chat">
  <Button variant="ghost" size="sm" className="gap-2">
    <MessageSquare className="h-4 w-4" />
    Chat
  </Button>
</Link>
```

**Hero Button** (for authenticated users):
```tsx
{isAuthenticated && (
  <Link href="/chat">
    <Button size="lg" variant="outline" className="gap-2">
      <MessageSquare className="h-4 w-4" />
      Try Chat
    </Button>
  </Link>
)}
```

## Data Models

### Existing (No Changes)

```rust
// ChatSession entity already has title field
pub struct ChatSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,  // ← We update this
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
```

### No Migration Required

Title field exists, just need to update via `update_title()` method.

## Security Considerations

### Title Generation

1. **Ownership Validation**: Verify user owns session before title generation
2. **Content Sanitization**: Truncate LLM output to 255 chars (database limit)
3. **Rate Limiting**: Use existing chat rate limiter
4. **Input Validation**: Only use first 2 messages, ignore injections

### Model Endpoint

- Already secured via JWT middleware
- Returns only enabled models (no sensitive config data)

## Performance Considerations

### Model Endpoint Caching

- **Frontend**: Cache for component lifecycle (~session duration)
- **Backend**: ModelRegistry already loaded once at startup
- **Network**: Single request on chat mount (~1KB response)

### Title Generation

- **Async**: Runs after message displayed, non-blocking
- **Token Usage**: ~50-100 tokens per title (minimal cost)
- **Latency**: ~1-2 seconds, user doesn't wait
- **Failure Mode**: Graceful degradation to "New Chat"

### Navigation Links

- **Static**: No runtime cost, just JSX
- **Conditional**: Check env var once at build time

## Testing Strategy

### Unit Tests

- `generate_session_title.rs`: Mock LLM responses, test title extraction
- `generate_title.rs` handler: Test auth, ownership, error cases

### Integration Tests

- Test full title generation flow with test LLM
- Verify title updates in database
- Test unauthorized access scenarios

### E2E Tests

- Create session → send message → verify auto-title appears
- Verify models endpoint returns only enabled models
- Click chat link from landing page → verify navigation

## Rollout Plan

### Phase 1: Backend Model Filtering
- Deploy backend changes (none needed)
- Update frontend to use `/api/v1/chat/models`
- Remove `models.ts` static config
- **Validation**: Models match backend's enabled list

### Phase 2: Auto Title Generation
- Add backend endpoint and use case
- Integrate frontend after first message
- **Validation**: New sessions get descriptive titles

### Phase 3: Navigation Links
- Add header and landing page links
- **Validation**: Chat accessible from all pages

### Rollback Plan

Each phase independent:
- **Phase 1**: Revert to static `models.ts`
- **Phase 2**: Remove title generation call, keep "New Chat"
- **Phase 3**: Remove navigation links

## Monitoring

### Metrics to Track

- **Model Endpoint**: Request count, latency, errors
- **Title Generation**: Success rate, LLM latency, fallback usage
- **Navigation**: Click-through rate from landing/header

### Alerts

- **Model Endpoint 5xx**: Alerts if endpoint fails (breaks chat UX)
- **Title Generation > 90% Fail**: Investigate LLM issues
- **Zero Models Enabled**: Critical alert (chat unusable)

## Open Questions

1. **Which model for title generation?**
   - **Recommendation**: Use fastest/cheapest model (grok-4-fast-non-reasoning or llama-3.3-70b)
   - **Rationale**: Simple task, no reasoning needed

2. **Regenerate title on demand?**
   - **Decision**: No, out of scope
   - **Future**: Add "Regenerate Title" button if requested

3. **Title generation for multi-turn convos?**
   - **Decision**: Only first exchange
   - **Rationale**: First message usually establishes topic

4. **Localization for titles?**
   - **Decision**: No, out of scope
   - **Future**: Pass user locale to LLM if needed
