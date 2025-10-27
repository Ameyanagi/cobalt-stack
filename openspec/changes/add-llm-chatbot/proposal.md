# Add LLM Chatbot Feature

## Why

Users need conversational AI capabilities to interact with an intelligent assistant within the application. This enables natural language interactions for help, information retrieval, and task assistance, powered by SambaNova's Llama-4-Maverick-17B-128E-Instruct model through their OpenAI-compatible API.

## What Changes

- Add new `llm-chat` capability for AI-powered conversation sessions with streaming responses
- Add new `user-quotas` capability for managing per-user resource limits
- Introduce `chat_sessions` and `chat_messages` database tables
- Implement Server-Sent Events (SSE) streaming for real-time chat responses
- Integrate `async-openai` crate for SambaNova API communication
- Add rate limiting (20 messages/minute) and daily quotas (100 messages/day)
- Create frontend components for chat interface with markdown rendering
- Implement smart context windowing (last 20 messages) for conversation coherency

## Impact

**Affected specs:**
- `llm-chat` (NEW) - Core chat functionality with sessions, messages, and streaming
- `user-quotas` (NEW) - Resource limit management and quota tracking

**Affected code:**
- Backend: New handlers, models, services for chat functionality
- Backend: Database migrations for chat_sessions and chat_messages tables
- Backend: New dependency: async-openai = "0.20"
- Frontend: New chat components (ChatList, ChatWindow, MessageBubble)
- Frontend: Dependencies: react-markdown, remark-gfm, rehype-highlight already present
- Configuration: New environment variables for SambaNova API

**Infrastructure:**
- Leverages existing Valkey/Redis for rate limiting
- Uses existing PostgreSQL for chat data persistence
- Uses existing JWT authentication middleware

**Non-Breaking:**
- All changes are additive
- No modifications to existing APIs or database schema
- Feature can be enabled/disabled via environment variable
