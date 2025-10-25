# Project Structure Specification

## ADDED Requirements

### Requirement: Backend Directory Structure
The backend SHALL be organized with clear separation of concerns following the specification structure with handlers, models, services, middleware, utils, and openapi modules.

#### Scenario: HTTP handlers are thin adapters
- **WHEN** implementing request handlers
- **THEN** handlers SHALL delegate business logic to services
- **AND** handlers SHALL only handle HTTP concerns (routing, validation, serialization)
- **AND** handlers SHALL be located in `src/handlers/` directory

#### Scenario: Business logic in services layer
- **WHEN** implementing business operations
- **THEN** logic SHALL be placed in service modules
- **AND** services SHALL be testable independently of HTTP layer
- **AND** services SHALL be located in `src/services/` directory

#### Scenario: Database models with SeaORM
- **WHEN** defining database entities
- **THEN** SeaORM entity models SHALL be used
- **AND** models SHALL be located in `src/models/` directory
- **AND** models SHALL match database schema from migrations

### Requirement: Migration Directory Structure
The backend SHALL include a separate migration crate for database schema management using SeaORM migrations.

#### Scenario: Migration crate structure
- **WHEN** setting up database migrations
- **THEN** migrations SHALL be in `backend/migration/` directory
- **AND** migration crate SHALL have its own Cargo.toml
- **AND** migration crate SHALL be a workspace member

#### Scenario: Migration file naming
- **WHEN** creating new migrations
- **THEN** files SHALL follow pattern `m{YYYYMMDD}_{NNNNNN}_{description}.rs`
- **AND** migrations SHALL be reversible (up and down functions)
- **AND** migrations SHALL be located in `migration/src/`

### Requirement: Cargo Workspace Configuration
The project SHALL use Cargo workspace to organize backend and migration crates with centralized dependency management.

#### Scenario: Workspace member compilation
- **WHEN** running `cargo build` from workspace root
- **THEN** backend and migration crates SHALL compile successfully
- **AND** shared dependencies SHALL use workspace-defined versions

#### Scenario: Dependency version consistency
- **WHEN** multiple crates depend on the same library
- **THEN** all crates SHALL use the version defined in `[workspace.dependencies]`
- **AND** version conflicts SHALL be detected at compile time

### Requirement: Development Tooling Setup
The project SHALL provide Makefile targets for common development tasks with one-command setup capability.

#### Scenario: Initial project setup
- **WHEN** developer runs `make setup`
- **THEN** all dependencies SHALL be installed
- **AND** environment files SHALL be created from examples
- **AND** development database SHALL be available

#### Scenario: Development server startup
- **WHEN** developer runs `make dev`
- **THEN** backend SHALL start with hot reload enabled
- **AND** changes to source files SHALL trigger automatic rebuild
- **AND** running tests SHALL execute automatically on changes

#### Scenario: Test execution with coverage
- **WHEN** developer runs `make test`
- **THEN** all tests SHALL execute
- **AND** coverage report SHALL be generated
- **AND** minimum coverage thresholds SHALL be enforced

### Requirement: TDD Infrastructure
The project SHALL provide continuous testing infrastructure with coverage reporting to support test-driven development workflow.

#### Scenario: Continuous testing during development
- **WHEN** developer runs `cargo watch -x test`
- **THEN** tests SHALL execute automatically on file changes
- **AND** test results SHALL display immediately
- **AND** compilation errors SHALL be shown clearly

#### Scenario: Coverage reporting
- **WHEN** running `cargo tarpaulin`
- **THEN** line coverage SHALL be reported for all source files
- **AND** uncovered lines SHALL be identified
- **AND** coverage percentage SHALL be calculated

#### Scenario: Test organization
- **WHEN** writing tests for domain logic
- **THEN** tests SHALL be co-located with domain code using `#[cfg(test)]`
- **WHEN** writing integration tests
- **THEN** tests SHALL be placed in `tests/` directory
- **AND** test helpers SHALL be available in `tests/common/`

### Requirement: Docker Compose Development Environment
The project SHALL provide Docker Compose configuration for running PostgreSQL and Redis services locally.

#### Scenario: Service startup
- **WHEN** developer runs `docker-compose up`
- **THEN** PostgreSQL 15 SHALL start on port 5432
- **AND** Redis 7 SHALL start on port 6379
- **AND** data SHALL persist across container restarts

#### Scenario: Service health check
- **WHEN** services are running
- **THEN** backend SHALL be able to connect to PostgreSQL
- **AND** backend SHALL be able to connect to Redis
- **AND** connection failures SHALL be logged clearly

#### Scenario: Service teardown
- **WHEN** developer runs `docker-compose down -v`
- **THEN** all containers SHALL stop
- **AND** volumes SHALL be removed
- **AND** development environment SHALL be reset to clean state

### Requirement: Health Check Endpoint (TDD Implementation)
The backend SHALL provide a health check endpoint implemented using test-driven development to validate infrastructure setup.

#### Scenario: Health check returns success
- **WHEN** GET request is made to `/health`
- **THEN** response status SHALL be 200 OK
- **AND** response body SHALL be valid JSON
- **AND** response SHALL contain `{"status": "healthy"}`

#### Scenario: Health check implementation follows TDD
- **WHEN** implementing health check
- **THEN** failing test SHALL be written first
- **AND** minimal implementation SHALL make test pass
- **AND** code coverage SHALL be 100% for health check handler

#### Scenario: Health check is independently testable
- **WHEN** testing health check handler
- **THEN** test SHALL NOT require database connection
- **AND** test SHALL NOT require Redis connection
- **AND** test SHALL execute in milliseconds

### Requirement: OpenAPI Schema Generation Infrastructure
The project SHALL provide infrastructure for generating OpenAPI 3.0 schema from Rust code using utoipa annotations.

#### Scenario: OpenAPI module structure
- **WHEN** setting up OpenAPI generation
- **THEN** OpenAPI code SHALL be in `src/openapi/mod.rs`
- **AND** schema generation function SHALL be provided
- **AND** schema SHALL be exported to `shared/openapi/schema.yaml`

#### Scenario: Swagger UI endpoint
- **WHEN** backend is running in development
- **THEN** Swagger UI SHALL be accessible at `/swagger-ui`
- **AND** Swagger UI SHALL display generated API documentation
- **AND** Swagger UI SHALL allow interactive testing of endpoints

#### Scenario: Schema export during build
- **WHEN** running `cargo build`
- **THEN** OpenAPI schema SHALL be generated automatically
- **AND** schema file SHALL be written to shared directory
- **AND** schema SHALL be valid OpenAPI 3.0 format

### Requirement: Core Rust Dependencies
The backend SHALL include core dependencies for web framework, async runtime, database, and serialization.

#### Scenario: Web framework dependencies
- **WHEN** building backend
- **THEN** Axum 0.7+ SHALL be included for HTTP routing
- **AND** Tokio 1.x SHALL be included for async runtime with full features
- **AND** Tower middleware SHALL be included for CORS and tracing

#### Scenario: Database dependencies
- **WHEN** setting up database layer
- **THEN** SeaORM 1.0+ SHALL be included with PostgreSQL support
- **AND** SeaORM SHALL use Tokio runtime
- **AND** SeaORM SHALL include macros for derive functionality

#### Scenario: Serialization and logging
- **WHEN** handling JSON and logging
- **THEN** serde and serde_json SHALL be included for JSON serialization
- **AND** tracing and tracing-subscriber SHALL be included for structured logging
- **AND** dotenvy SHALL be included for environment variable loading

#### Scenario: Development dependencies
- **WHEN** setting up testing infrastructure
- **THEN** cargo-tarpaulin SHALL be available for coverage
- **AND** cargo-watch SHALL be available for continuous testing
- **AND** test dependencies SHALL not be included in release builds

### Requirement: Environment Configuration
The project SHALL use environment variables for configuration with example files provided for development setup.

#### Scenario: Environment file template
- **WHEN** developer runs `make setup`
- **THEN** `.env` file SHALL be created from `.env.example`
- **AND** all required variables SHALL be documented
- **AND** sensible development defaults SHALL be provided

#### Scenario: Environment variables loaded at startup
- **WHEN** backend starts
- **THEN** environment variables SHALL be loaded from `.env` file
- **AND** missing required variables SHALL cause startup failure
- **AND** startup failure message SHALL indicate which variable is missing

#### Scenario: Secrets not committed to version control
- **WHEN** checking repository contents
- **THEN** `.env` files SHALL be in `.gitignore`
- **AND** only `.env.example` files SHALL be committed
- **AND** example files SHALL NOT contain real secrets

### Requirement: Code Quality Standards
The project SHALL enforce Rust code quality standards using rustfmt and clippy with strict linting rules.

#### Scenario: Code formatting enforcement
- **WHEN** running `cargo fmt --check`
- **THEN** all code SHALL conform to rustfmt defaults
- **AND** 4-space indentation SHALL be used
- **AND** 100-character line length limit SHALL be enforced

#### Scenario: Linting enforcement
- **WHEN** running `cargo clippy -- -D warnings`
- **THEN** no clippy warnings SHALL be present
- **AND** all clippy suggestions SHALL be resolved
- **AND** build SHALL fail if warnings are present

#### Scenario: Pre-commit validation
- **WHEN** code is ready for commit
- **THEN** `cargo fmt` SHALL pass
- **AND** `cargo clippy` SHALL pass with no warnings
- **AND** `cargo test` SHALL pass with 100% domain coverage

### Requirement: Documentation Standards
The project SHALL provide comprehensive documentation for setup, development workflow, and troubleshooting.

#### Scenario: Setup documentation completeness
- **WHEN** new developer reads README.md
- **THEN** all prerequisites SHALL be listed with versions
- **AND** setup steps SHALL be numbered and complete
- **AND** one-command setup SHALL be documented

#### Scenario: Development workflow documentation
- **WHEN** developer wants to start development
- **THEN** README SHALL explain how to run backend
- **AND** README SHALL explain how to run tests
- **AND** README SHALL explain TDD workflow

#### Scenario: Troubleshooting documentation
- **WHEN** developer encounters common issues
- **THEN** README SHALL provide troubleshooting section
- **AND** common errors SHALL be documented with solutions
- **AND** links to detailed documentation SHALL be provided
