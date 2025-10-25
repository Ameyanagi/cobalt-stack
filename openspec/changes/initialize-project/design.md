# Design: Project Initialization

## Context

This is the foundational change that establishes the entire project structure, tooling, and development workflow for the Cobalt Stack. All future development will build upon this foundation. The initialization must strictly follow DDD layered architecture and TDD practices as specified in `openspec/project.md`.

**Constraints:**
- Must support TDD workflow from day one (cargo-watch, continuous testing)
- Must enforce DDD layering (domain has zero infrastructure dependencies)
- Must use uv/uvx for Python tooling (cookiecutter, pytest-cookies)
- Must use Bun for frontend (future)
- Must work on Linux/macOS (primary targets)

**Stakeholders:**
- Developers using the template
- AI assistants implementing features following the spec
- Future contributors to the template project

## Goals / Non-Goals

**Goals:**
- Establish DDD-compliant directory structure for backend
- Set up TDD infrastructure with cargo-watch and cargo-tarpaulin
- Create one-command setup experience (`make setup`)
- Enable continuous testing during development
- Provide docker-compose for local database/cache services
- Document all setup steps clearly

**Non-Goals:**
- Not implementing actual business logic yet (just health check)
- Not setting up frontend structure yet (separate change)
- Not adding database migrations yet (separate change)
- Not implementing authentication yet (separate change)
- Not creating cookiecutter template yet (extract after working example is complete)

## Decisions

### Decision 1: Backend Code Organization

**What:** Organize backend code following the structure from the specification:
```
src/
├── main.rs
├── config/          # Configuration management
├── handlers/        # HTTP request handlers (health, auth, tasks)
├── models/          # SeaORM database models
├── services/        # Business logic layer
├── middleware/      # Request/response middleware (auth, logging)
├── utils/           # Utility functions (redis, error handling)
└── openapi/         # OpenAPI schema generation
```

With DDD principles applied within this structure:
- **handlers/**: Presentation layer (thin HTTP adapters)
- **services/**: Application + Domain logic (business rules)
- **models/**: Infrastructure layer (database entities via SeaORM)
- **middleware/**: Cross-cutting concerns

**Why:**
- Matches the detailed specification exactly
- Simpler initial structure, easier to understand
- Still maintains DDD separation of concerns
- SeaORM models naturally sit in infrastructure layer
- Services contain domain logic with clear boundaries

**Alternatives Considered:**
- Strict 4-layer DDD (domain/application/infrastructure/presentation) - Deferred: Too complex for initial setup, will refactor during implementation if needed
- Flat single-level structure - Rejected: Doesn't provide enough organization

### Decision 2: Test-First Implementation

**What:** Implement health check using strict TDD:
1. Write failing test for `/health` endpoint
2. Implement minimal code to pass
3. Refactor if needed
4. Ensure 100% coverage before commit

**Why:**
- Establishes TDD culture from first line of code
- Demonstrates correct testing patterns for future features
- Validates testing infrastructure works correctly
- project.md mandates TDD for all code

**Alternatives Considered:**
- Implementation first, tests later - Rejected: Violates TDD mandate
- Skip health check entirely - Rejected: Need minimal working endpoint to validate setup

### Decision 3: Makefile for Task Automation

**What:** Use GNU Make with targets:
- `setup`: Install dependencies, create .env files
- `dev`: Run backend with hot reload
- `test`: Run tests with coverage
- `build`: Build release binary
- `clean`: Remove build artifacts

**Why:**
- Available by default on Linux/macOS
- Simple, declarative syntax
- Industry standard for project automation
- Windows developers can use WSL

**Alternatives Considered:**
- `just` (Rust task runner) - Rejected: Extra dependency, less familiar
- npm scripts - Rejected: Requires Node.js for backend tasks
- Bash scripts - Rejected: Less declarative, harder to maintain

### Decision 4: Docker Compose for Services

**What:** Provide docker-compose.yml with:
- PostgreSQL 15 (exposed on 5432)
- Redis 7 (exposed on 6379, optional)
- Networks and volumes for persistence

**Why:**
- Consistent development environment
- Zero-config local setup
- Matches production infrastructure
- Easy teardown and reset

**Alternatives Considered:**
- System-installed PostgreSQL/Redis - Rejected: Version conflicts, harder cleanup
- SQLite for development - Rejected: Different SQL dialect from production
- Cloud-hosted databases - Rejected: Requires internet, costs money

### Decision 5: Cargo Workspace Structure

**What:** Single workspace with one binary crate initially:
```toml
[workspace]
members = ["backend"]

[workspace.dependencies]
# Shared dependency versions
```

**Why:**
- Allows future migration crate without duplication
- Centralizes dependency versions
- Supports future multi-crate architecture
- Cargo best practice for growing projects

**Alternatives Considered:**
- Single Cargo.toml at root - Rejected: Less flexible for future growth
- Multiple independent crates - Rejected: Overkill for current scope

## Risks / Trade-offs

### Risk 1: Cargo Watch Performance
**Risk:** cargo-watch may be slow on large projects with many dependencies

**Mitigation:**
- Start with minimal dependencies
- Use sccache for compilation caching (document in README)
- Consider bacon as alternative if cargo-watch becomes problem

### Risk 2: Docker Compose Resource Usage
**Risk:** Running PostgreSQL + Redis locally may use significant memory

**Mitigation:**
- Document how to run backend without Docker (use system PostgreSQL)
- Configure memory limits in docker-compose.yml
- Provide instructions to stop services when not needed

### Risk 3: DDD Overhead for Simple Features
**Risk:** Strict DDD layering may feel heavy for simple CRUD operations

**Mitigation:**
- Accept this trade-off for architectural consistency
- Show clear examples in initial implementation
- Document when to create domain services vs simple repository calls

### Risk 4: Testing Infrastructure Complexity
**Risk:** cargo-tarpaulin, cargo-watch, mockall may have learning curve

**Mitigation:**
- Provide clear examples in initial health check implementation
- Document testing patterns in README
- Use project.md TDD best practices section

## Migration Plan

This is initial setup, no migration needed.

**Rollback Plan:**
- Delete created directories
- Remove Cargo.toml, Makefile, docker-compose.yml
- Restore to empty repository state

## Open Questions

None - all decisions documented above.
