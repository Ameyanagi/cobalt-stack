# Spec: Auto-Generated Session Titles

## ADDED Requirements

### System SHALL generate session title after first exchange

**ID**: `chat-title-001`

#### Scenario: User sends first message in new session

**Given** user creates new chat session with default "New Chat" title
**When** user sends first message
**And** assistant completes response
**Then** system automatically generates descriptive title
**And** updates session title in database
**And** frontend displays new title in sidebar

**Acceptance Criteria**:
- Title generation happens asynchronously (non-blocking)
- User sees message immediately, title updates shortly after
- Title reflects conversation topic (4-8 words)
- Graceful degradation if generation fails (keeps "New Chat")

### Backend SHALL provide endpoint for title generation

**ID**: `chat-title-002`

#### Scenario: Title generation API called

**Given** user owns session with ID `{id}`
**When** `PATCH /api/v1/chat/sessions/{id}/generate-title` is called
**Then** system loads first user message and assistant response
**And** constructs prompt for LLM
**And** calls LLM with fast/cheap model
**And** extracts title from LLM response
**And** updates session.title via `update_title()` method
**And** returns updated ChatSession object

**Acceptance Criteria**:
- Validates user ownership (403 if unauthorized)
- Returns 404 if session doesn't exist
- Truncates title to 255 characters (database limit)
- Uses fastest available model for generation
- Logs but doesn't fail on LLM errors

### System SHALL use structured prompt format for title generation

**ID**: `chat-title-003`

#### Scenario: LLM receives title generation prompt

**Given** session has first user and assistant messages
**When** generating title
**Then** prompt format is:
```
Generate a concise, descriptive title (4-8 words) for this conversation:

User: {first_user_message}
Assistant: {first_assistant_message}

Title:
```

**Acceptance Criteria**:
- Prompt is clear and constrained (4-8 words)
- Includes full context of first exchange
- No additional instructions that could cause confusion
- Consistent format across all requests

### Generated titles SHALL be validated and sanitized

**ID**: `chat-title-004`

#### Scenario: LLM returns title

**Given** LLM generates title string
**When** processing response
**Then** system removes leading/trailing whitespace
**And** truncates to 255 characters if needed
**And** removes any surrounding quotes
**And** falls back to "Chat about {topic}" if empty
**And** validates via `ChatSession::validate_title()`

**Acceptance Criteria**:
- No empty titles saved
- No titles exceeding database limits
- Clean formatting (no extra quotes, newlines)
- Fallback strategy for edge cases

### Frontend SHALL trigger title generation after first response

**ID**: `chat-title-005`

#### Scenario: First message stream completes

**Given** chat session has zero messages initially
**When** user sends first message
**And** assistant response stream completes
**Then** frontend calls `generateSessionTitle(sessionId)`
**And** updates local session state with response
**And** sidebar reflects new title immediately

**Acceptance Criteria**:
- Only triggers for first exchange (message count was 0)
- Non-blocking (doesn't prevent user from sending next message)
- Error handling logs but doesn't show user error
- State updates propagate to session sidebar

## MODIFIED Requirements

None - this is new functionality.

## REMOVED Requirements

None - no existing title generation existed.

## Related Capabilities

- **backend-model-filtering**: Uses model selection for title generation
- **chat-navigation**: Improved UX makes chat more discoverable
