# Proposal: Initialize Frontend (Next.js + Bun)

**Change ID:** `initialize-frontend`
**Status:** Draft
**Created:** 2025-10-25
**Author:** AI Assistant

## Summary

Initialize the Next.js 16 (latest) frontend with Bun runtime, TailwindCSS, shadcn/ui, and type-safe API integration with the existing Rust backend. This establishes the foundation for building the full-stack Cobalt Stack application with proper Docker integration and development workflows.

## Motivation

The backend (Rust + Axum + SeaORM) is fully operational with a health check endpoint and Docker Compose setup. The next logical step is to initialize the frontend to:

1. Complete the full-stack architecture defined in `openspec/project.md`
2. Enable type-safe frontend-backend communication via OpenAPI-generated types
3. Provide a modern development experience with hot reload and component library
4. Maintain consistency with TDD principles on the backend (though less strict for UI)
5. Support both local and Docker-based development workflows

## Goals

### Primary Goals

1. **Next.js App Router Setup:** Initialize Next.js 16 (latest) with App Router architecture and src directory
2. **Type-Safe API Client:** Generate TypeScript types from backend OpenAPI schema and create API client
3. **UI Foundation:** Integrate TailwindCSS and shadcn/ui for component development
4. **Docker Integration:** Add frontend service to docker-compose.yml with hot reload
5. **Development Workflow:** Enable `make dev-frontend` for local development with Bun

### Non-Goals

- Authentication UI (NextAuth.js) - will be added in future proposals
- Complex state management beyond React Query - not needed yet
- E2E testing with Playwright - will be added when we have features to test
- Production optimization (code splitting, lazy loading) - future enhancement

## Proposed Changes

### Change Overview

This proposal introduces three interconnected capabilities:

1. **Frontend Foundation** (`specs/frontend-foundation/spec.md`)
   - Next.js 16 (latest) project initialization with Bun
   - src directory structure with app router
   - TailwindCSS configuration
   - shadcn/ui integration
   - Basic app structure (app router, layouts, pages)
   - Development server with hot reload

2. **Frontend API Integration** (`specs/frontend-api-integration/spec.md`)
   - OpenAPI schema generation from backend
   - TypeScript type generation with openapi-typescript
   - Type-safe API client wrapper
   - React Query setup for server state management
   - Health check page demonstrating API integration

3. **Frontend Docker Integration** (`specs/frontend-docker/spec.md`)
   - Multi-stage Dockerfile for frontend (dev + prod)
   - Docker Compose service configuration
   - Volume mounts for hot reload in development
   - Makefile targets for frontend operations

### Dependencies

- **Backend OpenAPI Generation:** Backend must expose `/openapi.json` endpoint (or generate schema.yaml)
- **Port Allocation:** Frontend requires port 3001 (maps to internal 3000)
- **Docker Network:** Uses existing `cobalt-network` from docker-compose.yml

### Sequencing

**Phase 1** (Foundation): Initialize Next.js, install dependencies, basic structure
**Phase 2** (API Integration): Generate types, create API client, add health check page
**Phase 3** (Docker): Containerize frontend, update docker-compose.yml, add Make targets

All phases can be implemented sequentially in a single development session.

## Alternatives Considered

### Alternative 1: Use npm/pnpm instead of Bun
**Rejected:** Bun provides significantly faster install times (10-100x) and better developer experience. Project spec explicitly prefers Bun.

### Alternative 2: Use Vite + React instead of Next.js
**Rejected:** Next.js App Router provides better SSR/SSG capabilities and aligns with project.md tech stack decision.

### Alternative 3: Manual API types without OpenAPI
**Rejected:** Violates type safety principle and creates maintenance burden. OpenAPI contract-first is a core architectural decision.

### Alternative 4: Defer Docker integration
**Rejected:** Docker integration is essential for full-stack development parity with backend.

## Impact Analysis

### User Impact
- **Developers:** Can now build UI components and connect to backend API
- **End Users:** No impact yet (no user-facing features in this proposal)

### System Impact
- **Performance:** Frontend dev server startup ~1-2s (Bun is fast)
- **Docker:** Additional container (+1 frontend service, ~200MB image for dev)
- **Build Time:** Full stack build increases by ~30-60s for frontend

### Migration Impact
- **None:** This is net-new functionality, no existing code to migrate

## Testing Strategy

### Frontend Unit Tests
- Component tests with Jest + React Testing Library (minimal for now)
- API client wrapper tests (mock fetch responses)

### Integration Tests
- Health check page successfully fetches from `/health` endpoint
- Type generation succeeds when backend schema changes

### Docker Tests
- `docker-compose up` brings up all services (postgres, redis, backend, frontend)
- Frontend accessible at http://localhost:3001
- Hot reload works when editing frontend code

## Open Questions

1. **TypeScript Strictness:** Should we enable all strict flags from day one?
   - **Recommendation:** Yes, align with project.md requirement for strict mode

2. **Component Library Scope:** Which shadcn/ui components to install initially?
   - **Recommendation:** Install only basics (button, card) initially, add more as needed

3. **Environment Variables:** How should frontend access API URL?
   - **Recommendation:** Use `NEXT_PUBLIC_API_URL` env var with sensible defaults

4. **Error Boundaries:** Should we add global error boundary now or later?
   - **Recommendation:** Add basic error boundary in root layout from the start

## Success Criteria

### Definition of Done

- [ ] Next.js app starts successfully with `bun run dev`
- [ ] TailwindCSS styles apply correctly
- [ ] At least one shadcn/ui component renders (button or card)
- [ ] TypeScript types generated from backend OpenAPI schema
- [ ] API client successfully calls `/health` endpoint
- [ ] Health check page displays `{"status":"healthy"}` response
- [ ] Frontend accessible at http://localhost:3001 via Docker Compose
- [ ] Hot reload works for code changes
- [ ] `make dev` starts both backend and frontend
- [ ] All TypeScript compiles without errors (strict mode enabled)
- [ ] Basic README documentation updated with frontend setup instructions

### Acceptance Criteria

- Frontend follows Next.js App Router conventions (app directory structure)
- No `any` types in application code (strictly typed)
- Environment variables properly configured (.env.local.example created)
- Docker image builds successfully in under 2 minutes
- Developer can run `make setup && make dev` and see working full-stack app

## Timeline Estimate

- **Phase 1 (Foundation):** 2-3 hours
- **Phase 2 (API Integration):** 1-2 hours
- **Phase 3 (Docker):** 1-2 hours
- **Testing & Documentation:** 1 hour
- **Total:** 5-8 hours for complete implementation

## References

- Project spec: `openspec/project.md` (Frontend Architecture section)
- Backend implementation: `backend/src/main.rs`, `backend/src/handlers/health.rs`
- Docker setup: `docker-compose.yml`
- Next.js 16 docs: https://nextjs.org/docs
- shadcn/ui docs: https://ui.shadcn.com/docs
- openapi-typescript: https://openapi-ts.dev/

## Related Changes

- **Depends on:** Backend OpenAPI generation (future enhancement to add `/openapi.json` endpoint)
- **Enables:** Future authentication UI, task management UI, user dashboard
- **Blocks:** Nothing (this is foundational work)
