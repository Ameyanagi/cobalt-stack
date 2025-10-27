# LLM Chat Capability

## ADDED Requirements

### Requirement: Chat Session Management
The system SHALL allow authenticated users to create, list, retrieve, and delete chat sessions for organizing conversations with the AI assistant.

#### Scenario: Create new chat session
- **WHEN** an authenticated user sends POST /api/v1/chat/sessions with optional title
- **THEN** the system creates a new session owned by that user
- **AND** returns session details (id, title, created_at, updated_at)
- **AND** generates default title if not provided

#### Scenario: List user's chat sessions
- **WHEN** an authenticated user sends GET /api/v1/chat/sessions with pagination parameters
- **THEN** the system returns all non-deleted sessions for that user
- **AND** includes pagination metadata (total, page, total_pages)
- **AND** orders sessions by most recent first (created_at DESC)

#### Scenario: Retrieve session with history
- **WHEN** an authenticated user sends GET /api/v1/chat/sessions/:id
- **THEN** the system returns session details and all messages in chronological order
- **AND** returns 404 if session doesn't exist or user doesn't own it

#### Scenario: Delete chat session
- **WHEN** an authenticated user sends DELETE /api/v1/chat/sessions/:id
- **THEN** the system soft deletes the session (sets deleted_at timestamp)
- **AND** CASCADE deletes all messages in that session
- **AND** returns 404 if session doesn't exist or user doesn't own it
- **AND** returns success message

### Requirement: Message Exchange with Streaming
The system SHALL enable users to send messages and receive AI-generated responses via real-time streaming using Server-Sent Events.

#### Scenario: Send message and receive streaming response
- **WHEN** an authenticated user sends POST /api/v1/chat/sessions/:id/messages with message content
- **THEN** the system validates session ownership and message content
- **AND** saves user message to database
- **AND** retrieves last N messages as context (configurable, default 20)
- **AND** streams response chunks from SambaNova API via SSE
- **AND** each SSE event contains chunk data and done flag
- **AND** saves complete assistant response to database when streaming completes

#### Scenario: Streaming error handling
- **WHEN** SambaNova API returns error during streaming
- **THEN** the system sends error event via SSE with error message
- **AND** closes the stream
- **AND** logs error details for monitoring

#### Scenario: Client disconnects during streaming
- **WHEN** client closes SSE connection before completion
- **THEN** the system cancels upstream SambaNova API request
- **AND** saves partial response if any content was received

### Requirement: Context Window Management
The system SHALL maintain conversation context by including recent message history in API requests while staying within token limits.

#### Scenario: Build context from recent messages
- **WHEN** preparing to send message to SambaNova API
- **THEN** the system retrieves last N messages (configurable, default 20) from session
- **AND** constructs message array with system message first
- **AND** appends conversation history in chronological order
- **AND** formats messages according to OpenAI API specification

#### Scenario: Handle empty conversation
- **WHEN** user sends first message in new session
- **THEN** the system includes only system message and user's message
- **AND** successfully generates response without history

### Requirement: Message Persistence
The system SHALL persist all chat messages (user and assistant) with metadata for history retrieval and analysis.

#### Scenario: Save user message
- **WHEN** user sends message to session
- **THEN** the system saves message with role='user', content, session_id, timestamp
- **AND** assigns unique message ID (UUID)

#### Scenario: Save assistant response
- **WHEN** assistant generates complete response
- **THEN** the system saves message with role='assistant', content, session_id, timestamp
- **AND** optionally saves token count if available
- **AND** associates with same session as user message

#### Scenario: Retrieve message history
- **WHEN** user requests session messages
- **THEN** the system returns all messages ordered by created_at ASC
- **AND** includes role, content, created_at for each message

### Requirement: Authorization and Ownership
The system SHALL enforce that users can only access and modify their own chat sessions and messages.

#### Scenario: Verify session ownership before access
- **WHEN** user attempts any operation on a session
- **THEN** the system verifies user_id in JWT matches session's user_id
- **AND** returns 404 if ownership check fails (not 403 to avoid leaking existence)

#### Scenario: Prevent cross-user session access
- **WHEN** user A attempts to access user B's session
- **THEN** the system returns 404 Not Found
- **AND** logs security event for monitoring

### Requirement: Error Handling and Resilience
The system SHALL handle API failures, network issues, and invalid inputs gracefully with user-friendly error messages.

#### Scenario: SambaNova API unavailable
- **WHEN** SambaNova API returns 5xx error or times out
- **THEN** the system returns 503 Service Unavailable
- **AND** includes user-friendly message "AI service temporarily unavailable"
- **AND** logs detailed error for debugging

#### Scenario: Invalid message content
- **WHEN** user submits empty message or message exceeding length limit
- **THEN** the system returns 400 Bad Request
- **AND** includes validation error details
- **AND** does not make API call to SambaNova

#### Scenario: Session not found
- **WHEN** user references non-existent session ID
- **THEN** the system returns 404 Not Found
- **AND** includes message "Session not found"

### Requirement: Database Schema
The system SHALL maintain database tables for chat sessions and messages with proper relationships and indexes.

#### Scenario: Chat sessions table structure
- **GIVEN** chat_sessions table exists
- **THEN** it includes columns: id (UUID PK), user_id (UUID FK), title (VARCHAR 255), created_at, updated_at, deleted_at
- **AND** has foreign key to users(id) with CASCADE delete
- **AND** has index on user_id for efficient querying
- **AND** has index on created_at for ordering

#### Scenario: Chat messages table structure
- **GIVEN** chat_messages table exists
- **THEN** it includes columns: id (UUID PK), session_id (UUID FK), role (VARCHAR 20), content (TEXT), token_count (INT nullable), created_at
- **AND** has foreign key to chat_sessions(id) with CASCADE delete
- **AND** has CHECK constraint on role IN ('user', 'assistant', 'system')
- **AND** has index on session_id for efficient history retrieval
- **AND** has index on created_at for ordering

### Requirement: Configuration
The system SHALL be configurable via environment variables for API credentials, model selection, and feature behavior.

#### Scenario: SambaNova API configuration
- **GIVEN** backend starts up
- **THEN** it reads SAMBANOVA_API_KEY from environment
- **AND** reads SAMBANOVA_API_BASE (default: https://api.sambanova.ai/v1)
- **AND** reads SAMBANOVA_MODEL (default: Llama-4-Maverick-17B-128E-Instruct)
- **AND** fails to start if SAMBANOVA_API_KEY is missing

#### Scenario: Chat feature configuration
- **GIVEN** backend starts up
- **THEN** it reads CHAT_MAX_CONTEXT_MESSAGES (default: 20)
- **AND** reads CHAT_MAX_TOKENS (default: 2048)
- **AND** reads CHAT_MAX_MESSAGE_LENGTH (default: 4000)
- **AND** reads FEATURE_CHAT_ENABLED (default: false)

### Requirement: Frontend Chat Interface
The system SHALL provide intuitive UI components for chat interaction with real-time streaming display.

#### Scenario: Chat session list display
- **WHEN** user accesses chat interface
- **THEN** frontend displays list of user's sessions in sidebar
- **AND** shows session title and last updated time
- **AND** highlights currently active session
- **AND** provides "New Chat" button
- **AND** provides delete button for each session

#### Scenario: Real-time message streaming
- **WHEN** user sends message
- **THEN** frontend immediately displays user message (optimistic UI)
- **AND** shows typing indicator for assistant response
- **AND** connects to SSE endpoint for streaming
- **AND** displays assistant response chunks as they arrive
- **AND** updates message in real-time character by character

#### Scenario: Markdown rendering
- **WHEN** assistant response contains markdown
- **THEN** frontend renders formatted text, lists, tables
- **AND** applies syntax highlighting to code blocks
- **AND** provides copy button for code snippets
- **AND** handles inline code, bold, italic, links

#### Scenario: Error display
- **WHEN** chat operation fails
- **THEN** frontend displays user-friendly error message
- **AND** provides retry button if applicable
- **AND** maintains user's input for retry
- **AND** shows connection status indicator
