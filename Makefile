.PHONY: setup dev dev-backend dev-frontend test build build-frontend docker-build clean help migrate generate-openapi generate-types

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
	@echo ""
	@echo "OpenAPI:"
	@echo "  make generate-openapi - Generate OpenAPI schema"
	@echo "  make generate-types   - Generate TypeScript types from OpenAPI schema"
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

## generate-openapi: Generate OpenAPI schema
generate-openapi:
	@echo "📝 Generating OpenAPI schema..."
	@cd backend && cargo run
	@echo "✅ OpenAPI schema generated at openapi/schema.json"

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

## lint: Run clippy linter
lint:
	@echo "🔍 Running clippy..."
	@cd backend && cargo clippy -- -D warnings

## fmt: Format code
fmt:
	@echo "✨ Formatting code..."
	@cd backend && cargo fmt

## fmt-check: Check code formatting
fmt-check:
	@echo "🔍 Checking code formatting..."
	@cd backend && cargo fmt --check

## ci: Run CI checks (fmt, lint, test)
ci: fmt-check lint test
	@echo "✅ All CI checks passed"
