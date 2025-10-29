# Spec: Backend-Driven Model Filtering

## ADDED Requirements

### Frontend SHALL fetch models from backend API

**ID**: `chat-models-001`

#### Scenario: Chat component mounts and fetches models

**Given** user navigates to `/chat` page
**When** the chat component mounts
**Then** frontend calls `GET /api/v1/chat/models`
**And** stores response in component state
**And** renders ModelSelector with only enabled models

**Acceptance Criteria**:
- No hardcoded model list in `models.ts`
- Models fetched once per session
- Loading state displayed while fetching
- Error handling if API fails (fallback to empty state)

### Model selector SHALL display only enabled models

**ID**: `chat-models-002`

#### Scenario: User opens model dropdown

**Given** chat session is active
**When** user clicks model selector dropdown
**Then** only models from API response are shown
**And** disabled models are not displayed
**And** default model is pre-selected

**Acceptance Criteria**:
- Models grouped by provider/category as defined by backend
- Default model selected on mount
- Model metadata (context window, streaming support) shown

### Model list SHALL be cached per session

**ID**: `chat-models-003`

#### Scenario: User navigates between sessions

**Given** models have been fetched once
**When** user switches between chat sessions
**Then** models are not refetched
**And** cached list is reused

**Acceptance Criteria**:
- Single API call per page load
- Cache invalidated on page refresh
- No stale data persisted across sessions

## REMOVED Requirements

### Requirement: Frontend maintains static model configuration

**ID**: `chat-models-old-001`

~~Frontend maintained hardcoded model list in `config/models.ts`.~~

**Rationale**: Replaced by backend-driven approach to ensure frontend-backend consistency.

## Related Capabilities

- **auto-title-generation**: Uses model for title generation
- **chat-navigation**: Chat link requires working model endpoint
