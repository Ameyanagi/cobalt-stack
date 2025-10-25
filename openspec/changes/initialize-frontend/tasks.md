# Tasks: Initialize Frontend

**Change ID:** `initialize-frontend`

## Task List

### Phase 1: Frontend Foundation (2-3 hours)

- [ ] **Task 1.1:** Initialize Next.js 16 (latest) project with Bun
  - Run `bun create next-app@latest frontend --typescript --tailwind --app --src-dir`
  - Verify TypeScript strict mode enabled in tsconfig.json
  - Verify src/app directory structure created
  - Test: `cd frontend && bun run dev` starts successfully

- [ ] **Task 1.2:** Configure TailwindCSS
  - Verify tailwind.config.ts has correct content paths
  - Add custom color palette (optional, can use defaults)
  - Test: Create simple component with Tailwind classes, verify styles apply

- [ ] **Task 1.3:** Install and configure shadcn/ui
  - Run `bunx shadcn-ui@latest init`
  - Install button component: `bunx shadcn-ui@latest add button`
  - Install card component: `bunx shadcn-ui@latest add card`
  - Test: Import and render Button component in page.tsx

- [ ] **Task 1.4:** Set up basic app structure
  - Create `src/app/layout.tsx` with root layout
  - Create `src/app/page.tsx` as home page
  - Create `src/app/health/page.tsx` for health check demonstration
  - Add basic error boundary in layout
  - Test: Navigate to http://localhost:3000 and /health routes

- [ ] **Task 1.5:** Create environment configuration
  - Create `.env.local.example` with `NEXT_PUBLIC_API_URL=http://localhost:3002`
  - Create `.env.local` with same content (gitignored)
  - Add environment validation in `lib/env.ts`
  - Test: Access `process.env.NEXT_PUBLIC_API_URL` in component

### Phase 2: API Integration (1-2 hours)

- [ ] **Task 2.1:** Add OpenAPI schema generation to backend
  - Install utoipa-swagger-ui if not present
  - Add `/openapi.json` endpoint in `backend/src/main.rs`
  - Test: `curl http://localhost:3002/openapi.json` returns valid schema

- [ ] **Task 2.2:** Generate TypeScript types from OpenAPI
  - Install openapi-typescript: `bun add -D openapi-typescript`
  - Add script in frontend/package.json: `"generate-types": "openapi-typescript http://localhost:3002/openapi.json -o src/types/api.ts"`
  - Run generation: `bun run generate-types`
  - Test: `src/types/api.ts` exists with HealthResponse type

- [ ] **Task 2.3:** Create type-safe API client wrapper
  - Create `lib/api-client.ts` with fetch wrapper
  - Add error handling and response parsing
  - Add TypeScript types from generated schema
  - Test: Mock fetch and verify types work correctly

- [ ] **Task 2.4:** Set up React Query
  - Install dependencies: `bun add @tanstack/react-query`
  - Create `src/app/providers.tsx` with QueryClientProvider
  - Wrap app in layout.tsx with providers
  - Test: React Query devtools appear (dev mode only)

- [ ] **Task 2.5:** Create health check page
  - Create `src/app/health/page.tsx` using React Query
  - Implement useQuery hook to fetch from `/health`
  - Display status with Card and Button components
  - Add loading and error states
  - Test: Page shows "healthy" status when backend is running

### Phase 3: Docker Integration (1-2 hours)

- [ ] **Task 3.1:** Create frontend Dockerfile
  - Create `frontend/Dockerfile` with multi-stage build
  - Dev stage: Bun runtime with hot reload
  - Prod stage: Standalone output with minimal image
  - Test: `docker build -t cobalt-frontend frontend/`

- [ ] **Task 3.2:** Add frontend to docker-compose.yml
  - Add frontend service definition
  - Configure port mapping (3001:3000)
  - Add volume mounts for hot reload
  - Set environment variables
  - Add dependency on backend service
  - Test: `docker-compose up frontend` starts successfully

- [ ] **Task 3.3:** Update Makefile with frontend targets
  - Add `dev-frontend` target for local Bun development
  - Update `dev` target to start both backend and frontend
  - Add `build-frontend` for Docker image build
  - Add `test-frontend` for running tests
  - Test: `make dev-frontend` starts frontend locally

- [ ] **Task 3.4:** Create .dockerignore for frontend
  - Add node_modules, .next, .env.local, etc.
  - Test: Verify Docker build doesn't copy ignored files

### Phase 4: Testing & Documentation (1 hour)

- [ ] **Task 4.1:** Add frontend tests
  - Create `__tests__/health.test.tsx` for health check page
  - Test API client with mocked fetch
  - Run tests: `bun test`
  - Verify all tests pass

- [ ] **Task 4.2:** Update README.md
  - Add frontend setup instructions
  - Document environment variables
  - Add troubleshooting section for common issues
  - Document `make` commands for frontend
  - Test: Follow README from scratch to verify completeness

- [ ] **Task 4.3:** Verify full-stack integration
  - Run `make dev` to start all services
  - Access backend health: `curl http://localhost:3002/health`
  - Access frontend health page: http://localhost:3001/health
  - Verify frontend displays backend health status
  - Test hot reload: Edit frontend code, verify auto-refresh

- [ ] **Task 4.4:** Create git commit
  - Review all changes with `git diff`
  - Stage files: `git add frontend/ docker-compose.yml Makefile README.md`
  - Commit with message: `feat(frontend): initialize Next.js 14 with Bun and type-safe API client`
  - Verify clean working tree

## Dependencies Between Tasks

- Task 2.1 must complete before 2.2 (OpenAPI schema needed for type generation)
- Task 2.2 must complete before 2.3 (Types needed for API client)
- Task 2.3 must complete before 2.5 (API client needed for health page)
- Phase 1 must complete before Phase 2 (Frontend structure needed for API integration)
- Phase 2 must complete before Phase 3 (Working frontend needed for Docker)

## Parallel Work Opportunities

- Tasks 1.2 and 1.3 can be done in parallel (TailwindCSS and shadcn/ui are independent)
- Tasks 3.1 and 3.2 can be done in parallel (Dockerfile and docker-compose are independent)
- Tasks 3.3 and 3.4 can be done in parallel (Makefile and .dockerignore are independent)

## Validation Checklist

After all tasks complete, verify:

- [ ] `bun run dev` starts frontend on port 3000
- [ ] `docker-compose up` starts all services (postgres, redis, backend, frontend)
- [ ] http://localhost:3001/health shows backend health status
- [ ] TypeScript compilation has zero errors (`bun run build`)
- [ ] All tests pass (`bun test`)
- [ ] Hot reload works for code changes
- [ ] No `any` types in application code
- [ ] README accurately documents setup process

## Estimated Time

- **Minimum:** 5 hours (experienced developer, no issues)
- **Expected:** 6-7 hours (typical development pace)
- **Maximum:** 8 hours (includes troubleshooting and learning)

## Notes

- Bun is significantly faster than npm/pnpm - installations should be nearly instant
- Next.js 14 App Router is different from Pages Router - follow App Router patterns
- shadcn/ui components are copied into project (not npm dependency) - this is intentional
- Hot reload in Docker may be slower than local development - use `make dev-frontend` for faster iteration
