.PHONY: setup dev dev-backend dev-frontend test build build-frontend docker-build clean help migrate seed-admin generate-openapi generate-types lint fmt fmt-check typecheck ci ci-frontend ci-all check fix

## Default target
.DEFAULT_GOAL := help

## help: Display this help message
help:
	@echo "Cobalt Stack - Development Commands"
	@echo ""
	@echo "Setup:"
	@echo "  make setup          - Initial project setup (copy .env files, install dependencies)"
	@echo ""
	@echo "Development:"
	@echo "  make dev            - Run full stack with docker-compose"
	@echo "  make dev-backend    - Run backend with hot reload (cargo-watch)"
	@echo "  make dev-frontend   - Run frontend dev server (bun)"
	@echo ""
	@echo "Testing:"
	@echo "  make test           - Run all tests with coverage"
	@echo "  make test-watch     - Run tests in watch mode"
	@echo ""
	@echo "Building:"
	@echo "  make build          - Build release binary"
	@echo "  make build-frontend - Build frontend for production"
	@echo "  make docker-build   - Build Docker images"
	@echo ""
	@echo "Database:"
	@echo "  make migrate        - Run database migrations"
	@echo "  make seed-admin     - Create initial admin user"
	@echo ""
	@echo "OpenAPI:"
	@echo "  make generate-openapi - Generate OpenAPI schema"
	@echo "  make generate-types   - Generate TypeScript types from OpenAPI schema"
	@echo ""
	@echo "Code Quality:"
	@echo "  make lint           - Run linting (backend: clippy, frontend: biome)"
	@echo "  make fmt            - Format code (backend: cargo fmt, frontend: biome)"
	@echo "  make fmt-check      - Check code formatting"
	@echo "  make typecheck      - Run type checking (backend + frontend)"
	@echo "  make check          - Run all quality checks (typecheck + lint)"
	@echo "  make fix            - Auto-fix all formatting and linting issues"
	@echo "  make ci             - Run backend CI checks (fmt-check + lint + test)"
	@echo "  make ci-frontend    - Run frontend CI checks (typecheck + lint)"
	@echo "  make ci-all         - Run all CI checks (backend + frontend)"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean          - Clean build artifacts and stop containers"

## setup: Initial project setup
setup:
	@echo "🚀 Setting up Cobalt Stack..."
	@if [ ! -f backend/.env ]; then cp backend/.env.example backend/.env; echo "✅ Created backend/.env"; fi
	@if [ ! -f frontend/.env.local ]; then cp frontend/.env.local.example frontend/.env.local; echo "✅ Created frontend/.env.local"; fi
	@echo "📦 Installing Rust dependencies..."
	@cd backend && cargo build
	@echo "📦 Installing frontend dependencies..."
	@cd frontend && bun install
	@echo "✅ Setup complete! Run 'make dev' to start development."

## dev: Run full stack with docker-compose
dev:
	@echo "🐳 Starting full stack with docker-compose..."
	docker-compose up

## dev-backend: Run backend with hot reload
dev-backend:
	@echo "🦀 Starting backend with cargo-watch..."
	@if ! command -v cargo-watch &> /dev/null; then \
		echo "Installing cargo-watch..."; \
		cargo install cargo-watch; \
	fi
	cd backend && cargo watch -x run

## dev-frontend: Run frontend dev server
dev-frontend:
	@echo "⚡ Starting frontend dev server with bun..."
	@cd frontend && bun run dev

## test: Run all tests
test:
	@echo "🧪 Running tests..."
	@cd backend && cargo test

## test-watch: Run tests in watch mode
test-watch:
	@echo "🧪 Running tests in watch mode..."
	@if ! command -v cargo-watch &> /dev/null; then \
		echo "Installing cargo-watch..."; \
		cargo install cargo-watch; \
	fi
	@cd backend && cargo watch -x test

## test-coverage: Run tests with coverage
test-coverage:
	@echo "📊 Running tests with coverage..."
	@if ! command -v cargo-tarpaulin &> /dev/null; then \
		echo "Installing cargo-tarpaulin..."; \
		cargo install cargo-tarpaulin; \
	fi
	@cd backend && cargo tarpaulin --out Html --output-dir coverage

## build: Build release binary
build:
	@echo "🔨 Building release binary..."
	@cd backend && cargo build --release
	@echo "✅ Binary built: backend/target/release/cobalt-stack-backend"

## build-frontend: Build frontend for production
build-frontend:
	@echo "🔨 Building frontend for production..."
	@cd frontend && bun run build
	@echo "✅ Frontend built in: frontend/.next"

## docker-build: Build Docker image
docker-build:
	@echo "🐳 Building Docker image with BuildKit..."
	DOCKER_BUILDKIT=1 docker-compose build

## docker-build-prod: Build production Docker image
docker-build-prod:
	@echo "🐳 Building production Docker image..."
	DOCKER_BUILDKIT=1 docker-compose -f docker-compose.prod.yml build

## migrate: Run database migrations
migrate:
	@echo "🗄️  Running database migrations..."
	@if ! command -v sea-orm-cli &> /dev/null; then \
		echo "Installing sea-orm-cli..."; \
		cargo install sea-orm-cli; \
	fi
	@cd backend && sea-orm-cli migrate up

## seed-admin: Create initial admin user
seed-admin:
	@echo "🌱 Creating admin user..."
	@cd backend && cargo run --bin seed-admin

## generate-openapi: Generate OpenAPI schema
generate-openapi:
	@echo "📝 Generating OpenAPI schema..."
	@cd backend && cargo run --release --bin generate_openapi

## generate-types: Generate TypeScript types from OpenAPI schema
generate-types:
	@echo "🔧 Generating TypeScript types..."
	@cd frontend && bunx openapi-typescript ../openapi/schema.json -o src/types/api.ts
	@echo "✅ TypeScript types generated at frontend/src/types/api.ts"

## clean: Clean build artifacts and stop containers
clean:
	@echo "🧹 Cleaning up..."
	@cd backend && cargo clean
	@rm -rf frontend/.next frontend/node_modules
	@docker-compose down -v
	@echo "✅ Cleanup complete"

## lint: Run linting (backend + frontend)
lint:
	@echo "🔍 Running linting checks..."
	@echo "📦 Backend (clippy)..."
	@cd backend && cargo clippy --all-features --all-targets -- -D warnings
	@echo "📦 Frontend (biome)..."
	@cd frontend && bun run lint
	@echo "✅ Linting complete"

## fmt: Format code (backend + frontend)
fmt:
	@echo "✨ Formatting code..."
	@echo "🦀 Backend (cargo fmt)..."
	@cd backend && cargo fmt
	@echo "📦 Frontend (biome)..."
	@cd frontend && bun run format
	@echo "✅ Formatting complete"

## fmt-check: Check code formatting (backend + frontend)
fmt-check:
	@echo "🔍 Checking code formatting..."
	@echo "🦀 Backend (cargo fmt)..."
	@cd backend && cargo fmt --check
	@echo "📦 Frontend (biome)..."
	@cd frontend && bun run lint
	@echo "✅ Format check complete"

## typecheck: Run type checking (backend + frontend)
typecheck:
	@echo "🔍 Running type checks..."
	@echo "🦀 Backend (cargo check)..."
	@cd backend && cargo check --all-features
	@echo "📦 Frontend (tsc)..."
	@cd frontend && bun run typecheck
	@echo "✅ Type checking complete"

## check: Run all quality checks (typecheck + lint)
check: typecheck lint
	@echo "✅ All quality checks passed"

## fix: Auto-fix all formatting and linting issues
fix:
	@echo "🔧 Auto-fixing all issues..."
	@echo "🦀 Backend (cargo fmt + clippy --fix)..."
	@cd backend && cargo fmt
	@cd backend && cargo clippy --fix --allow-dirty --allow-staged --all-features --all-targets
	@echo "📦 Frontend (biome)..."
	@cd frontend && bun run lint:fix
	@cd frontend && bun run format
	@echo "✅ Auto-fix complete"

## ci: Run backend CI checks (fmt-check + lint + test)
ci:
	@echo "🔍 Running backend CI checks..."
	@cd backend && cargo fmt --check
	@cd backend && cargo clippy --all-features --all-targets -- -D warnings
	@cd backend && cargo test
	@echo "✅ Backend CI checks passed"

## ci-frontend: Run frontend CI checks (typecheck + lint)
ci-frontend:
	@echo "🔍 Running frontend CI checks..."
	@cd frontend && bun run typecheck
	@cd frontend && bun run lint
	@echo "✅ Frontend CI checks passed"

## ci-all: Run all CI checks (backend + frontend)
ci-all: ci ci-frontend
	@echo "✅ All CI checks passed"
