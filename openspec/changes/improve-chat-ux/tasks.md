# Tasks: Improve Chat UX

## Phase 1: Backend-Driven Model Filtering

### Task 1.1: Create useModels hook
**Effort**: 30min
**Dependencies**: None
**Validation**: Hook returns models from API

- [ ] Create `frontend/src/hooks/use-models.ts`
- [ ] Implement `useModels()` hook with fetch logic
- [ ] Handle loading, error, and success states
- [ ] Cache models in React state for session
- [ ] Export types: `UseModelsReturn`

**Acceptance**:
```typescript
const { models, groups, defaultModel, isLoading, error } = useModels();
// models populated from /api/v1/chat/models
```

### Task 1.2: Update chat-container to use API models
**Effort**: 45min
**Dependencies**: 1.1
**Validation**: Model selector shows API models only

- [ ] Import `useModels()` in `chat-container.tsx`
- [ ] Replace static `AVAILABLE_MODELS` import
- [ ] Pass API models to `<ModelSelector>`
- [ ] Add loading state while fetching models
- [ ] Show error message if models fail to load

**Acceptance**:
```bash
# Start chat, open model selector
# Verify only enabled models shown (e.g., only Grok models if others disabled)
```

### Task 1.3: Remove static models configuration
**Effort**: 15min
**Dependencies**: 1.2
**Validation**: No hardcoded models remain

- [ ] Delete or deprecate `frontend/src/config/models.ts`
- [ ] Update any other components importing from `models.ts`
- [ ] Remove exports: `AVAILABLE_MODELS`, `MODEL_GROUPS`
- [ ] Keep type definitions in `types/chat.ts`

**Acceptance**:
```bash
grep -r "from '@/config/models'" frontend/src
# Should find no active imports
```

### Task 1.4: Test model filtering
**Effort**: 30min
**Dependencies**: 1.3
**Validation**: E2E test passes

- [ ] Manual test: Disable all but one model in `models.toml`
- [ ] Rebuild backend, restart
- [ ] Verify frontend shows only enabled model
- [ ] Re-enable models, verify full list returns

**Acceptance**: Frontend always matches backend's enabled models

---

## Phase 2: Auto-Generated Titles

### Task 2.1: Create generate_session_title use case
**Effort**: 1.5 hours
**Dependencies**: None
**Validation**: Unit tests pass

- [ ] Create `backend/src/application/chat/generate_session_title.rs`
- [ ] Define `GenerateSessionTitleUseCase` struct
- [ ] Implement `execute(session_id, user_id)` method
- [ ] Load first 2 messages from repository
- [ ] Construct title generation prompt
- [ ] Call LLM via provider factory (use fastest model)
- [ ] Extract and validate title from response
- [ ] Update session title via `ChatRepository`
- [ ] Write unit tests with mock repository and LLM

**Acceptance**:
```rust
#[tokio::test]
async fn test_generates_title_from_first_exchange() {
    let use_case = GenerateSessionTitleUseCase::new(mock_repo, mock_llm);
    let result = use_case.execute(session_id, user_id).await.unwrap();
    assert_ne!(result.title, "New Chat");
    assert!(result.title.len() > 0 && result.title.len() <= 255);
}
```

### Task 2.2: Create generate_title handler
**Effort**: 45min
**Dependencies**: 2.1
**Validation**: Handler compiles, routes correctly

- [ ] Create `backend/src/handlers/chat/generate_title.rs`
- [ ] Define `generate_title` handler function
- [ ] Extract session ID from path params
- [ ] Get user ID from JWT claims (auth middleware)
- [ ] Call `GenerateSessionTitleUseCase`
- [ ] Return updated session as JSON
- [ ] Add error handling (404, 403, 500)
- [ ] Annotate with `#[utoipa::path]` for OpenAPI

**Acceptance**:
```bash
curl -X PATCH http://localhost:2750/api/v1/chat/sessions/{id}/generate-title \
  -H "Authorization: Bearer $TOKEN"
# Returns updated ChatSession with new title
```

### Task 2.3: Register generate_title route
**Effort**: 15min
**Dependencies**: 2.2
**Validation**: Route accessible via API

- [ ] Add route to `backend/src/handlers/chat/mod.rs`
- [ ] Register in router: `PATCH /sessions/:id/generate-title`
- [ ] Apply JWT auth middleware
- [ ] Add to OpenAPI schema generation
- [ ] Export handler function

**Acceptance**: Route appears in `/swagger-ui` and responds to requests

### Task 2.4: Add generateSessionTitle to frontend API client
**Effort**: 30min
**Dependencies**: 2.3
**Validation**: TypeScript types generated

- [ ] Rebuild backend to regenerate OpenAPI schema
- [ ] Run `npm run generate-types` in frontend
- [ ] Add `generateSessionTitle(sessionId)` to `use-chat-api.ts`
- [ ] Implement HTTP PATCH with auth header
- [ ] Return updated `ChatSession` object
- [ ] Add error handling

**Acceptance**:
```typescript
const { generateSessionTitle } = useChatApi();
const updated = await generateSessionTitle(sessionId);
console.log(updated.title); // "Python recursion help"
```

### Task 2.5: Integrate title generation in chat-container
**Effort**: 45min
**Dependencies**: 2.4
**Validation**: Titles auto-generate after first message

- [ ] Update `handleSendMessage` in `chat-container.tsx`
- [ ] Add logic in `parseStream` completion callback
- [ ] Check if this is first exchange (messages.length === 0 before user message)
- [ ] Call `generateSessionTitle(currentSession.id)` async
- [ ] Update `currentSession` state with response
- [ ] Catch and log errors silently (don't show to user)
- [ ] Trigger session list refresh to show new title

**Acceptance**:
```bash
# Create new session
# Send first message: "How does Python recursion work?"
# Wait for assistant response
# Verify session title changes from "New Chat" to descriptive title
```

### Task 2.6: Test title generation
**Effort**: 1 hour
**Dependencies**: 2.5
**Validation**: Integration tests pass

- [ ] Write integration test for use case
- [ ] Test with various first messages (short, long, code, etc.)
- [ ] Test error cases (LLM timeout, empty response, etc.)
- [ ] Test ownership validation (user can't gen title for others' sessions)
- [ ] Test E2E: new session → message → verify title
- [ ] Test fallback behavior when LLM fails

**Acceptance**: All test scenarios pass without blocking user experience

---

## Phase 3: Chat Navigation

### Task 3.1: Add chat link to header
**Effort**: 15min
**Dependencies**: None
**Validation**: Link visible and functional

- [ ] Edit `frontend/src/app/page.tsx`
- [ ] Add chat link in header after "API Docs"
- [ ] Import and use `MessageSquare` icon
- [ ] Style as `Button variant="ghost" size="sm"`
- [ ] Wrap in `<Link href="/chat">`

**Acceptance**: Click chat link from any page → navigate to `/chat`

### Task 3.2: Add chat button to landing hero
**Effort**: 20min
**Dependencies**: None
**Validation**: Button shown for authenticated users

- [ ] Edit hero section in `page.tsx`
- [ ] Add conditional "Try Chat" button for `isAuthenticated` users
- [ ] Position next to "Dashboard" button
- [ ] Style as `Button size="lg" variant="outline"`
- [ ] Test both authenticated and unauthenticated states

**Acceptance**:
```bash
# Logged in: See "Go to Dashboard" and "Try Chat"
# Logged out: See only "Get Started Free"
```

### Task 3.3: Add chat feature card to features grid
**Effort**: 30min
**Dependencies**: None
**Validation**: Card displays correctly

- [ ] Add new `<Card>` in features section
- [ ] Title: "AI Chat Assistant"
- [ ] Icon: `<MessageSquare className="h-10 w-10 text-primary mb-2" />`
- [ ] Description: Multi-model chat with session management
- [ ] List 3 features with checkmarks:
  - Multiple AI models (Llama, GPT, Grok)
  - Real-time streaming responses
  - Organized session management
- [ ] Match styling of other feature cards

**Acceptance**: Feature card visually consistent, no layout issues

### Task 3.4: Test navigation integration
**Effort**: 30min
**Dependencies**: 3.1, 3.2, 3.3
**Validation**: All nav paths work

- [ ] Test header link from various pages
- [ ] Test hero button (authenticated)
- [ ] Verify feature card displays correctly
- [ ] Test responsive design (mobile, tablet, desktop)
- [ ] Verify icons render correctly
- [ ] Test with `FEATURE_CHAT_ENABLED=false` (links should hide)

**Acceptance**: All navigation paths lead to functional chat interface

---

## Final Integration & Testing

### Task 4.1: End-to-end smoke test
**Effort**: 30min
**Dependencies**: All phases complete
**Validation**: Full user flow works

- [ ] Full flow: Landing page → Click chat link → Create session → Send message → Verify auto-title → Switch sessions → Verify models match backend
- [ ] Test model changes: Disable/enable models in backend → Restart → Verify frontend updates
- [ ] Test error scenarios: LLM failure, network issues, auth errors
- [ ] Performance check: Measure title generation latency, model fetch time

**Acceptance**: No blocking issues, graceful degradation on errors

### Task 4.2: Documentation updates
**Effort**: 30min
**Dependencies**: 4.1
**Validation**: Docs accurate

- [ ] Update `README.md` with chat feature description
- [ ] Document new endpoint in API docs (auto-generated)
- [ ] Add chat feature to features list
- [ ] Update screenshots if needed
- [ ] Document title generation behavior

**Acceptance**: Users understand chat capabilities and new features

### Task 4.3: Monitoring and alerts
**Effort**: 30min
**Dependencies**: 4.1
**Validation**: Metrics tracked

- [ ] Add logging for title generation (success/failure rates)
- [ ] Monitor `/api/v1/chat/models` endpoint usage
- [ ] Track model selection distribution
- [ ] Alert on high failure rates (>10% for title generation)

**Acceptance**: Operations team can monitor chat health

---

## Task Summary

**Total Estimated Effort**: ~9 hours

**Parallelizable Tasks**:
- Phase 1 (model filtering) and Phase 3 (navigation) can run in parallel
- Phase 2 (titles) depends on Phase 1 indirectly (model selection)

**Critical Path**:
1. Phase 1: Backend model filtering (2 hours)
2. Phase 2: Auto-title generation (4.5 hours)
3. Phase 3: Chat navigation (1.5 hours)
4. Final integration (1 hour)

**Risk Mitigation**:
- Each phase independently deployable
- Fallback strategies for all LLM operations
- Graceful degradation on errors

## Validation Checklist

- [ ] Frontend shows only backend-enabled models
- [ ] New sessions get auto-generated titles
- [ ] Chat links visible in header and landing page
- [ ] No hardcoded model configuration remains
- [ ] Title generation fails gracefully
- [ ] Navigation works from all pages
- [ ] Responsive design maintained
- [ ] OpenAPI schema regenerated
- [ ] Tests pass (unit, integration, E2E)
- [ ] Documentation updated
- [ ] Monitoring configured
