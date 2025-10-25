# Project Context

## Purpose

**Cobalt Stack** is a production-ready, full-stack template for building modern web applications with a Rust backend and Next.js frontend, using cookiecutter for project generation.

**Philosophy:** Build a working example application first (cobalt-stack-example), then extract it into a reusable cookiecutter template. This ensures all patterns are proven and battle-tested before becoming part of the template.

**Project Name Origin:** Named after cobalt, the element next to iron in the periodic table - symbolizing strength and reliability.

**Key Goals:**
- Provide a complete, working full-stack example that demonstrates all technologies integrated properly
- Extract proven patterns into a cookiecutter template for rapid project scaffolding
- Ensure type safety flows from backend to frontend via OpenAPI schema generation
- Support both monolithic and microservices deployment patterns
- Minimize boilerplate while maximizing developer experience

## Tech Stack

### Backend (Rust)
- **Runtime:** Tokio async runtime
- **Web Framework:** Axum 0.7+ (routing, middleware, handlers)
- **Database ORM:** SeaORM (PostgreSQL integration)
- **Caching:** Redis (optional, configurable feature)
- **Authentication:** JWT with bcrypt/argon2 password hashing
- **API Documentation:** utoipa (OpenAPI 3.0 schema generation)
- **Logging:** tracing + tracing-subscriber
- **Error Handling:** anyhow + thiserror
- **Serialization:** serde + serde_json
- **Container:** Distroless Docker images (gcr.io/distroless/cc-debian12)

### Frontend (TypeScript/Next.js)
- **Framework:** Next.js 14+ with App Router
- **Runtime:** Bun (preferred) or Node.js
- **Styling:** TailwindCSS
- **UI Components:** shadcn/ui
- **State Management:** @tanstack/react-query for server state
- **Forms:** react-hook-form + Zod validation
- **Authentication:** NextAuth.js (optional, configurable feature)
- **API Client:** Type-safe client generated from OpenAPI schema via openapi-typescript
- **Type Safety:** TypeScript strict mode enabled

### Development Tools
- **Template Engine:** Cookiecutter (Python-based, language-agnostic)
- **Python Package Manager:** uv/uvx (fast Python package installer and runner)
- **Build Tools:** Cargo (Rust), Bun/npm (Frontend)
- **Orchestration:** Docker Compose for local development
- **Task Runner:** Makefile for common commands
- **Testing:** pytest-cookies for template validation
- **CI/CD:** GitHub Actions (optional feature)

### Infrastructure
- **Database:** PostgreSQL 15+
- **Cache:** Redis 7+ (optional)
- **Reverse Proxy:** Traefik (future consideration)
- **Container Registry:** Docker Hub / GitHub Container Registry

## Project Conventions

### Code Style

**Rust Backend:**
- Follow official Rust style guide (rustfmt default configuration)
- Use `cargo clippy` for linting (deny warnings in CI)
- Naming: snake_case for functions/variables, PascalCase for types/structs
- Module organization: handlers, models, services, middleware, utils
- Error types: Use thiserror for domain errors, anyhow for application errors
- Async: All I/O operations must be async with Tokio
- Comments: Prefer self-documenting code; use doc comments (///) for public APIs

**TypeScript Frontend:**
- Follow Next.js App Router conventions
- Use ESLint + Prettier (config-next preset)
- Naming: camelCase for functions/variables, PascalCase for components/types
- Component organization: One component per file, co-locate styles
- Type safety: No `any` types in application code (use `unknown` with type guards)
- Imports: Use absolute imports with `@/` prefix for src directory
- Comments: JSDoc for exported functions and components

**Shared:**
- Line length: 100 characters max (Rust), 120 characters max (TypeScript)
- Indentation: 4 spaces (Rust), 2 spaces (TypeScript/JSON/YAML)
- File naming: snake_case.rs (Rust), kebab-case.tsx (Frontend components)
- Git: Conventional Commits (feat:, fix:, docs:, refactor:, test:, chore:)

### Architecture Patterns

**Backend Architecture:**
- **Domain-Driven Design (DDD) - MANDATORY:**
  - **Domain Layer:** Core business logic, entities, value objects, domain events
  - **Application Layer:** Use cases, application services, DTOs
  - **Infrastructure Layer:** Database implementations, external services, frameworks
  - **Presentation Layer:** HTTP handlers, API contracts (thin layer)
  - **Strict Separation:** Domain layer has zero dependencies on infrastructure
  - **Ubiquitous Language:** Code must reflect business domain terminology
- **Test-Driven Development (TDD) - MANDATORY:**
  - **Red-Green-Refactor:** Write failing test → implement minimum code → refactor
  - **No Production Code Without Tests:** Every function/method must have tests first
  - **Test Coverage:** 100% for domain logic, 80%+ for application layer
  - **Test Pyramid:** Unit tests (domain) → Integration tests (application) → E2E tests (API)
- **Layered Architecture:**
  - Handlers (HTTP layer) → Application Services → Domain Services → Entities/Value Objects
  - Clean separation of concerns: handlers are adapters, domain is pure business logic
  - Infrastructure adapters implement domain interfaces (Repository pattern)
- **Dependency Injection:** Use Axum's State for infrastructure dependencies (DB pool, Redis client)
- **Middleware Chain:** CORS → Logging → Auth → Request handlers
- **Error Handling:** Domain errors → Application errors → HTTP errors (proper mapping)
- **Configuration:** Environment-based config loaded at startup via dotenvy

**Frontend Architecture:**
- **App Router:** Use Next.js 14+ app directory structure
- **Server Components:** Default to server components, use client components only when needed
- **API Layer:** Type-safe client wrapper around fetch with error handling
- **State Management:** React Query for server state, React Context for UI state
- **Component Hierarchy:** pages → layouts → features → UI components
- **Data Fetching:** Server components fetch data directly; client components use React Query

**Cross-Stack Integration:**
- **OpenAPI Contract-First:** Backend generates schema → Frontend consumes types
- **Type Flow:** Rust structs (with utoipa) → OpenAPI YAML → TypeScript types
- **API Versioning:** Not implemented initially; add /v1 prefix when needed
- **Error Format:** Consistent JSON structure across all endpoints

### Testing Strategy

**Backend Testing (TDD Approach - MANDATORY):**
- **TDD Workflow:** All code MUST be written test-first (Red → Green → Refactor)
- **Unit Tests (Domain Layer):**
  - Test entities, value objects, domain services in isolation
  - Pure business logic with zero infrastructure dependencies
  - Coverage Target: 100% (no exceptions)
  - Location: domain/tests/ or inline with #[cfg(test)]
- **Integration Tests (Application Layer):**
  - Test use cases and application services
  - Mock infrastructure dependencies (repositories, external services)
  - Coverage Target: 80%+ for application layer
  - Location: tests/ directory or application/tests/
- **API Tests (Presentation Layer):**
  - Test HTTP endpoints with TestRequest
  - Use test database with transactions rolled back after each test
  - Verify request/response contracts match OpenAPI schema
  - Location: tests/api/ directory
- **Mocking Strategy:**
  - Use trait-based abstractions for infrastructure (Repository trait)
  - Use mockall or manual test doubles for mocking
  - Never mock domain logic (test real implementations)
- **Testing Tools:**
  - cargo test (unit/integration tests)
  - cargo-tarpaulin (coverage reporting)
  - cargo-watch -x test (continuous testing during TDD)

**Frontend Testing:**
- **Unit Tests:** Jest + React Testing Library for components
- **Integration Tests:** Playwright for E2E user flows
- **Coverage Target:** 60%+ code coverage for critical paths
- **API Mocking:** MSW (Mock Service Worker) for API responses
- **Accessibility:** axe-core integration in component tests

**Template Testing:**
- **Generation Tests:** pytest-cookies validates template generates correctly
- **Compilation Tests:** Generated projects must compile without errors
- **Integration Tests:** Docker Compose must bring up all services successfully
- **Option Matrix:** Test all combinations of optional features (Redis y/n, NextAuth y/n, etc.)

### Git Workflow

**Branching Strategy:**
- **main:** Production-ready code (protected, requires PR)
- **feature/\*:** New features (branch from main, merge via PR)
- **fix/\*:** Bug fixes (branch from main, merge via PR)
- **docs/\*:** Documentation updates
- **No develop branch:** Keep it simple, main is the source of truth

**Commit Conventions:**
- Use Conventional Commits specification
- Format: `<type>(<scope>): <description>`
- Types: feat, fix, docs, refactor, test, chore, perf, ci
- Examples:
  - `feat(backend): add JWT authentication middleware`
  - `fix(frontend): resolve type error in TaskList component`
  - `docs(readme): update setup instructions`

**PR Process:**
- All changes via pull requests (no direct commits to main)
- PR must pass CI checks (build, test, lint)
- Require 1 approval for merge (small projects; increase for teams)
- Squash and merge to keep clean history

**Release Process:**
- Semantic versioning (MAJOR.MINOR.PATCH)
- Tag releases on main branch
- Changelog generated from conventional commits

## Domain Context

**Problem Domain:**
This is a **meta-project** that creates other projects. The domain knowledge required is:

1. **Full-Stack Development Patterns:**
   - RESTful API design principles
   - JWT authentication flows and token management
   - Database schema design and migrations
   - Type-safe API contracts via OpenAPI

2. **Cookiecutter Templating:**
   - Jinja2 templating syntax for dynamic file generation
   - Conditional file inclusion based on user choices
   - Pre/post-generation hooks for validation and setup automation
   - Variable interpolation and default value handling

3. **Developer Experience:**
   - One-command setup and execution (make setup, make dev)
   - Hot reload workflows for rapid iteration
   - Clear error messages and troubleshooting guides
   - Minimal cognitive load for common tasks

4. **Production Readiness:**
   - Security best practices (CORS, CSRF, input validation)
   - Performance optimization (connection pooling, caching, lazy loading)
   - Observability (structured logging, health checks, metrics)
   - Container optimization (multi-stage builds, distroless images)

**Example Application Domain:**
The working example (cobalt-stack-example) implements a **Task Manager** or **Todo API** to demonstrate:
- User registration and authentication
- CRUD operations on a simple resource (tasks)
- Ownership and authorization (users can only access their own tasks)
- Full-stack data flow from database → API → UI

## Important Constraints

### Technical Constraints

1. **Rust Version:** Minimum 1.75 (for latest Axum features)
2. **PostgreSQL:** Minimum 15 (for latest SeaORM compatibility)
3. **Next.js:** Version 14+ required (App Router is mandatory)
4. **Docker:** Multi-stage builds required; distroless runtime mandatory for security
5. **Type Safety:** Zero tolerance for `any` types in frontend; must use OpenAPI-generated types
6. **Environment Variables:** All secrets must come from .env files (never hardcoded)

### Development Constraints

1. **Build Order Dependency:**
   - Backend must build first to generate OpenAPI schema
   - Frontend type generation depends on schema.yaml existence
   - Use `make generate-openapi` to manually trigger regeneration

2. **Database Migrations:**
   - Always use SeaORM migrations (no manual SQL schema changes)
   - Migrations must be reversible (implement both up and down)
   - Test migrations against production-like data volumes

3. **API Compatibility:**
   - Backward compatibility required for API changes
   - Breaking changes require API versioning (/v1 → /v2)
   - Deprecation warnings must precede breaking changes by 1 minor version

### Business Constraints

1. **Open Source:** Project must remain MIT licensed
2. **Template Usability:** Generated projects must work with zero configuration changes
3. **Documentation Quality:** README must enable setup by someone unfamiliar with the stack
4. **Security Defaults:** Generated projects must be secure by default (no test credentials, etc.)

### Performance Constraints

1. **Cold Start:** Backend must start in <5 seconds (local), <15 seconds (Docker)
2. **API Response:** 95th percentile latency <200ms for authenticated endpoints
3. **Frontend Build:** Production build must complete in <2 minutes
4. **Docker Image Size:** Backend image <50MB (distroless achieves this)

## External Dependencies

### Required Services

1. **PostgreSQL Database:**
   - Used by: Backend (SeaORM)
   - Configuration: DATABASE_URL environment variable
   - Connection pooling: Managed by SeaORM
   - Default port: 5432

2. **Redis Cache (Optional):**
   - Used by: Backend (session storage, caching)
   - Configuration: REDIS_URL environment variable
   - Connection: Managed via redis-rs with connection manager
   - Default port: 6379
   - Conditional: Only included if `use_redis=y` in cookiecutter

### Development Dependencies

1. **Cargo (Rust Toolchain):**
   - Required version: 1.75+
   - Used for: Backend compilation, testing, linting
   - Installation: rustup recommended

2. **Bun or Node.js:**
   - Bun preferred (faster install/build times)
   - Node.js 18+ supported as fallback
   - Used for: Frontend compilation, type generation, testing

3. **uv/uvx (Python Package Management):**
   - uv: Fast Python package installer (replaces pip/pip-tools)
   - uvx: Run Python tools without installation (replaces pipx)
   - Used for: Cookiecutter template generation, pytest-cookies testing
   - Installation: `curl -LsSf https://astral.sh/uv/install.sh | sh`
   - Benefits: 10-100x faster than pip, reliable dependency resolution

4. **Docker & Docker Compose:**
   - Used for: Local development orchestration, production containers
   - Required for: Full-stack development experience
   - Optional: Can run backend/frontend separately without Docker

5. **Make:**
   - Used for: Task automation (setup, dev, test, build commands)
   - Available on: Linux/macOS by default; Windows via WSL or chocolatey

### External APIs/Services

**None in base template** - The example application is self-contained. However, the template is designed to easily integrate:

- Authentication providers (Auth0, Clerk, Supabase Auth)
- Email services (SendGrid, AWS SES)
- Storage services (AWS S3, Cloudflare R2)
- Monitoring (Sentry, DataDog)

### OpenAPI Tooling

1. **utoipa (Rust):**
   - Generates OpenAPI 3.0 schema from Rust code
   - Annotations: `#[utoipa::path]` for endpoints, `#[derive(ToSchema)]` for types

2. **openapi-typescript (Node.js):**
   - Generates TypeScript types from OpenAPI schema
   - Command: `openapi-typescript schema.yaml -o types/api.ts`

3. **Swagger UI:**
   - Embedded in backend at `/swagger-ui` endpoint
   - Auto-generated interactive API documentation

### CI/CD (Optional Feature)

**GitHub Actions Workflows:**
- `ci.yml`: Build, test, lint on every PR
- `release.yml`: Build Docker images and create GitHub releases
- `template-test.yml`: Validate cookiecutter template generates correctly

**Required Secrets:**
- `DOCKER_USERNAME`, `DOCKER_PASSWORD`: For image publishing
- None required for base testing workflows

---

## Additional Notes for AI Assistants

### Implementation Philosophy

1. **Build Working Example First:**
   - Create cobalt-stack-example as a complete, functional application
   - Test every feature end-to-end before extracting to template
   - Document all setup steps and troubleshooting

2. **Extract to Template Second:**
   - Copy working example to cookiecutter structure
   - Replace hardcoded values with Jinja2 variables
   - Add conditionals for optional features
   - Automate setup via post-generation hooks

3. **Validate Template Third:**
   - Write pytest-cookies tests for all option combinations
   - Verify generated projects compile and run
   - Test Docker builds and compose orchestration

### Key Files to Understand

- `backend/src/main.rs`: Application entry point, router setup
- `backend/src/openapi/mod.rs`: OpenAPI schema generation
- `frontend/src/lib/api-client.ts`: Type-safe API client wrapper
- `docker-compose.yml`: Local development orchestration
- `Makefile`: Common commands and workflows
- `cookiecutter.json`: Template configuration and user prompts (after extraction)

### Backend DDD Project Structure

```
backend/
├── src/
│   ├── domain/                    # Pure business logic (zero dependencies)
│   │   ├── mod.rs
│   │   ├── task/                  # Task aggregate
│   │   │   ├── mod.rs
│   │   │   ├── entity.rs          # Task entity
│   │   │   ├── value_objects.rs   # TaskStatus, TaskPriority, etc.
│   │   │   ├── repository.rs      # TaskRepository trait (interface)
│   │   │   ├── service.rs         # Domain services (if needed)
│   │   │   └── events.rs          # Domain events (TaskCreated, etc.)
│   │   └── user/                  # User aggregate
│   │       ├── mod.rs
│   │       ├── entity.rs
│   │       ├── value_objects.rs
│   │       ├── repository.rs
│   │       └── events.rs
│   ├── application/               # Use cases and application services
│   │   ├── mod.rs
│   │   ├── task/
│   │   │   ├── mod.rs
│   │   │   ├── create_task.rs     # CreateTaskUseCase
│   │   │   ├── list_tasks.rs      # ListTasksUseCase
│   │   │   ├── update_task.rs     # UpdateTaskUseCase
│   │   │   └── delete_task.rs     # DeleteTaskUseCase
│   │   ├── user/
│   │   │   ├── register_user.rs   # RegisterUserUseCase
│   │   │   └── authenticate.rs    # AuthenticateUseCase
│   │   └── dto/                   # Data Transfer Objects
│   │       ├── task.rs
│   │       └── user.rs
│   ├── infrastructure/            # External implementations
│   │   ├── mod.rs
│   │   ├── persistence/
│   │   │   ├── mod.rs
│   │   │   ├── models.rs          # SeaORM models
│   │   │   ├── task_repository.rs # TaskRepository implementation
│   │   │   └── user_repository.rs
│   │   ├── cache/
│   │   │   └── redis.rs           # Redis cache implementation
│   │   └── config/
│   │       └── mod.rs             # Configuration management
│   ├── presentation/              # HTTP handlers (thin layer)
│   │   ├── mod.rs
│   │   ├── routes.rs              # Router setup
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   ├── task.rs            # Task HTTP handlers
│   │   │   ├── user.rs            # User HTTP handlers
│   │   │   └── health.rs          # Health check
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   └── auth.rs            # JWT middleware
│   │   └── openapi/
│   │       └── mod.rs             # OpenAPI schema generation
│   ├── shared/                    # Shared utilities
│   │   ├── mod.rs
│   │   ├── errors.rs              # Error types and conversions
│   │   └── utils.rs
│   └── main.rs                    # Application entry point
├── tests/                         # Integration and E2E tests
│   ├── api/
│   │   ├── task_api_test.rs
│   │   └── user_api_test.rs
│   └── common/
│       └── test_helpers.rs
└── Cargo.toml
```

### Common Tasks (TDD Workflow)

- **Add New Feature (TDD + DDD):**
  1. **Red:** Write failing domain test for new entity/value object/service
  2. **Green:** Implement minimum domain logic to pass test
  3. **Refactor:** Clean up domain code
  4. **Red:** Write failing application service test (use case)
  5. **Green:** Implement application service
  6. **Refactor:** Optimize application layer
  7. **Red:** Write failing API test
  8. **Green:** Implement HTTP handler (thin adapter)
  9. Annotate with utoipa → rebuild → types regenerate

- **Add Database Table (DDD Repository Pattern):**
  1. Create migration → run migration
  2. **Red:** Write test for repository interface behavior
  3. **Green:** Create SeaORM model in infrastructure/persistence
  4. **Green:** Implement repository trait from domain layer
  5. Wire up dependency injection in main.rs

- **Add UI Component:** Use shadcn/ui CLI → customize → integrate with API client

- **Add Optional Feature (Template):** Use Jinja2 conditionals → test both enabled/disabled states

### TDD Best Practices

- **Run tests continuously:** `cargo watch -x test` during development
- **Write smallest possible test:** One assertion per test when possible
- **Test behavior, not implementation:** Focus on what code does, not how
- **Keep tests fast:** Domain tests should run in milliseconds
- **Use descriptive test names:** `test_task_creation_fails_when_title_is_empty`
- **Follow AAA pattern:** Arrange → Act → Assert

### Debugging Tips

- Backend logs: Check tracing output for request/response details
- Frontend types: Regenerate with `npm run generate-types` after backend changes
- Database issues: Check migrations with `sea-orm-cli migrate status`
- Docker issues: Use `docker-compose logs [service]` to inspect container output
