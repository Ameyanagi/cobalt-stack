# Cobalt Stack

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Next.js](https://img.shields.io/badge/Next.js-16-black.svg)](https://nextjs.org/)
[![Documentation](https://img.shields.io/badge/docs-latest-brightgreen.svg)](./docs)

Modern full-stack application template with Rust backend (Axum + SeaORM) and Next.js 16 frontend (Bun + React Query + shadcn/ui).

**Repository**: [https://github.com/Ameyanagi/cobalt-stack](https://github.com/Ameyanagi/cobalt-stack)

## Features

- **Modern Rust Backend**: Axum 0.7+ with SeaORM for type-safe database access
- **Next.js 16 Frontend**: App Router with TypeScript and shadcn/ui components
- **Type Safety**: OpenAPI schema generation with automatic TypeScript type generation
- **Authentication**: JWT-based auth with email verification and role-based access control
- **Admin System**: Complete admin dashboard with user management
- **Testing**: TDD workflow with cargo-tarpaulin coverage reporting
- **Docker Ready**: Optimized multi-stage builds with BuildKit cache mounts
- **Development Tools**: Hot reload, code generation, linting, and formatting

## Quick Start

### Prerequisites

- **Rust** 1.75+ ([install](https://rustup.rs/))
- **Bun** 1.2+ ([install](https://bun.sh/))
- **Docker** & **Docker Compose** ([install](https://docs.docker.com/get-docker/))
- **Make** (usually pre-installed on macOS/Linux)

### Installation

```bash
# Clone the repository
git clone https://github.com/Ameyanagi/cobalt-stack
cd cobalt-stack

# Setup environment and install dependencies
make setup

# Start all services with Docker Compose
make dev
```

Access the application:
- **Frontend**: http://localhost:3001
- **Backend API**: http://localhost:3002/health
- **Swagger UI**: http://localhost:3002/swagger-ui

### Initial Admin Setup

```bash
# Run database migrations
make migrate

# Create initial admin user (development only)
make seed-admin
# Email: admin@example.com | Password: admin123
```

## Documentation

Comprehensive guides and references for all aspects of the project:

- **[Getting Started](docs/getting-started/)** - Installation, setup, and first steps
- **[Backend Documentation](docs/backend/)** - Rust backend architecture and API development
- **[Frontend Documentation](docs/frontend/)** - Next.js frontend development and components
- **[API Reference](docs/api/)** - Complete API endpoint documentation
- **[Deployment Guide](docs/deployment/)** - Production deployment and configuration
- **[Contributing Guide](docs/contributing/)** - Development workflow and contribution guidelines

## Development

### Running Services

```bash
# Full stack with Docker Compose
make dev

# Backend only (hot reload)
make dev-backend

# Frontend only (dev server)
make dev-frontend
```

### Testing

```bash
# Run all tests
make test

# Watch mode (auto-rerun)
make test-watch

# Coverage report
make test-coverage
```

### Code Quality

```bash
# Lint, format, and test
make ci

# Individual commands
make lint      # Run clippy
make fmt       # Format code
make fmt-check # Check formatting
```

### Type Generation

```bash
# Generate OpenAPI schema from backend
make generate-openapi

# Generate TypeScript types from schema
make generate-types
```

## Architecture

### Backend Stack

- **Framework**: Axum 0.7+ (async web framework)
- **ORM**: SeaORM 1.1+ with PostgreSQL
- **Runtime**: Tokio async runtime
- **API Docs**: utoipa for OpenAPI 3.0 generation
- **Testing**: cargo-test with 100% coverage requirement for domain logic

### Frontend Stack

- **Framework**: Next.js 16 App Router
- **Runtime**: Bun 1.2+
- **UI Components**: shadcn/ui + Radix UI + TailwindCSS 4.1
- **State Management**: React Query 5.x
- **Type Safety**: TypeScript strict mode with OpenAPI-generated types

### Design Principles

- **Domain-Driven Design**: Clear separation between handlers, services, and models
- **Type Safety**: End-to-end type safety from database to frontend
- **Test-Driven Development**: Red → Green → Refactor workflow
- **OpenSpec Workflow**: Proposal-based change management for significant features

## Project Structure

```
cobalt-stack/
├── backend/              # Rust backend (Axum + SeaORM)
│   ├── src/
│   │   ├── handlers/    # HTTP request handlers
│   │   ├── services/    # Business logic (DDD)
│   │   ├── models/      # Database models
│   │   └── middleware/  # Auth and middleware
│   ├── migration/       # Database migrations
│   └── tests/           # Integration tests
├── frontend/            # Next.js frontend (Bun + React)
│   ├── src/
│   │   ├── app/        # App Router pages
│   │   ├── components/ # shadcn/ui components
│   │   └── lib/        # API client and utilities
├── openapi/            # Generated OpenAPI schema
├── docs/               # Project documentation
├── docker-compose.yml  # Development environment
└── Makefile           # Task automation
```

## Configuration

### Docker Environment

Copy `.env.example` to `.env` and customize:

```bash
# Service Ports
FRONTEND_PORT=2727
BACKEND_PORT=2750
POSTGRES_PORT=2800
REDIS_PORT=2900

# API Configuration
NEXT_PUBLIC_API_URL=http://localhost:2750

# Database
POSTGRES_DB=cobalt_dev
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
```

### Production Environment

Required variables for `docker-compose.prod.yml`:

```bash
DATABASE_PASSWORD  # PostgreSQL password
REDIS_PASSWORD     # Redis password
JWT_SECRET         # JWT signing secret
EMAIL_MOCK=false   # Enable real SMTP
```

See [Deployment Guide](docs/deployment/) for complete production setup.

## License

MIT License - see [LICENSE](./LICENSE) file for details.

Copyright (c) 2025 Cobalt Stack Contributors

## Contributing

1. Review the [Contributing Guide](docs/contributing/)
2. Create OpenSpec proposal for significant changes
3. Follow TDD workflow (Red → Green → Refactor)
4. Ensure `make ci` passes before submitting PR

## Resources

- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [SeaORM Guide](https://www.sea-ql.org/SeaORM/docs/index/)
- [Next.js Documentation](https://nextjs.org/docs)
- [OpenSpec Workflow](./openspec/README.md)

---

**Built with Rust, Axum, SeaORM, and Next.js**
