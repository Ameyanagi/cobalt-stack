# Backend Documentation

Comprehensive documentation for the Cobalt Stack Rust backend services.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
- [API Services](#api-services)
- [Database](#database)
- [Testing](#testing)
- [Documentation](#documentation)

## Overview

The Cobalt Stack backend is built with:
- **Rust** - Core programming language
- **Actix-web** - Web framework
- **PostgreSQL** - Database
- **Docker** - Containerization

## Architecture

### Project Structure

```
backend/
├── src/
│   ├── api/          # API endpoints
│   ├── models/       # Data models
│   ├── services/     # Business logic
│   ├── db/           # Database layer
│   └── main.rs       # Application entry point
├── tests/            # Integration tests
└── Cargo.toml        # Dependencies
```

### Design Patterns

- **Repository Pattern**: Data access abstraction
- **Service Layer**: Business logic separation
- **Dependency Injection**: Loose coupling
- **Error Handling**: Result types and custom errors

## Getting Started

### Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- Docker (optional)

### Development Setup

```bash
# Install dependencies
cargo build

# Run migrations
cargo run --bin migrate

# Start development server
cargo run

# Run tests
cargo test
```

## API Services

### Available Services

- **Authentication Service** - User authentication and authorization
- **User Service** - User management
- **Data Service** - Core data operations

For detailed API documentation, see [API Reference](../api/README.md).

## Database

### Migrations

```bash
# Create new migration
cargo run --bin create-migration <name>

# Run migrations
cargo run --bin migrate

# Rollback migration
cargo run --bin rollback
```

### Schema

Database schema documentation is available in the [API Reference](../api/README.md).

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test '*'
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## Documentation

### Rust Documentation

Generate and view comprehensive Rust documentation:

```bash
# Generate documentation
cargo doc --no-deps --open

# Generate with private items
cargo doc --no-deps --document-private-items --open
```

See [Rust Documentation Guide](./rust-doc-guide.md) for more details.

### Code Documentation Standards

- Document all public APIs
- Include examples in doc comments
- Document error conditions
- Explain non-obvious implementation details

## Related Resources

- [Getting Started Guide](../getting-started/quick-start.md)
- [API Reference](../api/README.md)
- [Deployment Guide](../deployment/README.md)
- [Troubleshooting](../troubleshooting/README.md)
