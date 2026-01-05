# Cobalt Stack Documentation

Welcome to the Cobalt Stack documentation! This is your central hub for all documentation resources.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../LICENSE)
[![Documentation](https://img.shields.io/badge/docs-latest-brightgreen.svg)](.)

## Table of Contents

- [Getting Started](#getting-started)
- [Architecture](#architecture)
- [Backend Documentation](#backend-documentation)
- [Frontend Documentation](#frontend-documentation)
- [API Reference](#api-reference)
- [Guides & Tutorials](#guides--tutorials)
- [Deployment](#deployment)
- [Contributing](#contributing)
- [Troubleshooting](#troubleshooting)
- [Changelog](#changelog)
- [Quick Links](#quick-links)

## Getting Started

New to Cobalt Stack? Start here:

- **[Quick Start Guide](./getting-started/quick-start.md)** - Get up and running in 5 minutes
- **[Installation Guide](./getting-started/installation.md)** - Detailed installation instructions
- **[Project Structure](./getting-started/project-structure.md)** - Understanding the codebase

### First Steps
1. [Prerequisites](./getting-started/installation.md#prerequisites)
2. [Clone and Setup](./getting-started/installation.md#installation)
3. [Environment Configuration](./getting-started/installation.md#environment-setup)
4. [Run Your First Command](./getting-started/quick-start.md#running-the-application)

## Architecture

System architecture and design:

- **[Architecture Overview](./architecture/overview.md)** - High-level system design
- **[Backend Architecture](./architecture/backend.md)** - Rust backend structure
- **[Frontend Architecture](./architecture/frontend.md)** - Next.js frontend design

## Backend Documentation

Rust backend services and API implementation:

- **[Backend Overview](./backend/README.md)** - Architecture and structure
- **[API Handlers](./backend/api-handlers.md)** - Request handlers and routing
- **[Database Layer](./backend/database.md)** - Database models and queries
- **[Services](./backend/services.md)** - Business logic layer
- **[Models](./backend/models.md)** - Data structures and types
- **[Testing](./backend/testing.md)** - Backend testing strategies
- **[Rust Documentation Guide](./backend/rust-doc-guide.md)** - How to use cargo doc

### Quick Navigation
- [Authentication Implementation](./backend/api-handlers.md#authentication)
- [Database Migrations](./backend/database.md#migrations)
- [Error Handling](./backend/api-handlers.md#error-handling)

## Frontend Documentation

React/Next.js frontend documentation:

- **[Frontend Overview](./frontend/README.md)** - Architecture and components
- **[Architecture](./frontend/architecture.md)** - Next.js App Router patterns
- **[Components](./frontend/components.md)** - UI component library
- **[State Management](./frontend/state-management.md)** - React Query patterns
- **[API Client](./frontend/api-client.md)** - Type-safe API integration
- **[Theme System](./frontend/themes.md)** - Multi-theme support
- **[Testing](./frontend/testing.md)** - Frontend testing strategies
- **[Screenshots](./frontend/screenshots/README.md)** - Visual gallery of UI components
- **[Screenshot Capture Guide](./frontend/screenshots/CAPTURE-GUIDE.md)** - How to capture screenshots

### Quick Navigation
- [Component Examples](./frontend/components.md#component-showcase)
- [Theme Configuration](./frontend/themes.md#theme-configuration)
- [API Integration](./frontend/api-client.md#type-safe-api-client)
- [Testing Patterns](./frontend/testing.md#testing-patterns)

## API Reference

RESTful API endpoints and specifications:

- **[API Overview](./api/README.md)** - API documentation hub
- **[API Reference](./api/reference.md)** - Complete endpoint reference
- **[Authentication Endpoints](./api/authentication.md)** - Auth API documentation
- **[User Endpoints](./api/users.md)** - User management API
- **[Admin Endpoints](./api/admin.md)** - Admin-only API endpoints

### Quick Access
- [OpenAPI Specification](../backend/openapi.json)
- [Authentication Flow](./api/authentication.md#authentication-flow)
- [Error Codes](./api/reference.md#error-codes)

## Guides & Tutorials

Step-by-step tutorials and how-to guides:

- **[Guides Overview](./guides/README.md)** - Available guides and tutorials
- **[Authentication Guide](./guides/authentication.md)** - Implementing authentication
- **[Database Guide](./guides/database.md)** - Working with PostgreSQL
- **[Admin Dashboard Guide](./guides/admin-dashboard.md)** - Building admin features
- **[API Client Guide](./guides/api-client.md)** - Using the API client
- **[Email Verification](./guides/email-verification.md)** - Email verification flow
- **[Theme Customization](./guides/themes.md)** - Creating custom themes
- **[Testing Guide](./guides/testing.md)** - Writing and running tests

### Popular Guides
- [Adding a New API Endpoint](./guides/api-client.md#adding-endpoints)
- [Creating a Custom Theme](./guides/themes.md#creating-themes)
- [Setting Up Email](./guides/email-verification.md#smtp-configuration)

## Deployment

Production deployment and infrastructure:

- **[Deployment Overview](./deployment/README.md)** - Deployment strategies
- **[Docker Deployment](./deployment/docker.md)** - Containerized deployment
- **[Production Setup](./deployment/production.md)** - Production best practices
- **[Environment Variables](./deployment/environment-variables.md)** - Configuration reference
- **[Monitoring](./deployment/monitoring.md)** - Application monitoring

### Quick Deploy
- [Docker Compose](./deployment/docker.md#docker-compose)
- [Environment Setup](./deployment/environment-variables.md#production-variables)
- [Health Checks](./deployment/monitoring.md#health-checks)

## Contributing

Help improve Cobalt Stack:

- **[Contributing Guidelines](./contributing/README.md)** - How to contribute
- **[Code Style Guide](./contributing/code-style.md)** - Coding standards
- **[Pull Request Process](./contributing/pull-requests.md)** - PR guidelines
- **[Testing Requirements](./contributing/testing-requirements.md)** - Test standards

### Get Involved
- [Good First Issues](https://github.com/your-org/cobalt-stack/labels/good-first-issue)
- [Development Setup](./contributing/README.md#development-setup)
- [Commit Guidelines](./contributing/code-style.md#commit-messages)

## Troubleshooting

Common issues and solutions:

- **[Troubleshooting Guide](./troubleshooting/README.md)** - Solutions to common problems

### Quick Fixes
- [Database Connection Issues](./troubleshooting/README.md#database-issues)
- [Authentication Problems](./troubleshooting/README.md#authentication-issues)
- [Build Errors](./troubleshooting/README.md#build-errors)

## Changelog

- **[Version History](./CHANGELOG.md)** - Track changes and releases

---

## Quick Links

### Project Resources
- **[Main README](../README.md)** - Project overview and introduction
- **[Implementation Guide](../IMPLEMENTATION_GUIDE.md)** - Development roadmap
- **[LICENSE](../LICENSE)** - MIT License terms

### Development
- **[Local Development Setup](./getting-started/installation.md#development-setup)** - Set up your dev environment
- **[Running Tests](./guides/testing.md#running-tests)** - Test commands and strategies
- **[Code Generation](./backend/rust-doc-guide.md#generating-documentation)** - Generate API docs

### Configuration
- **[Environment Variables](./deployment/environment-variables.md)** - All configuration options
- **[Database Setup](./guides/database.md#setup)** - PostgreSQL configuration
- **[SMTP Configuration](./guides/email-verification.md#smtp-configuration)** - Email setup

### Common Tasks
- **[Adding API Endpoints](./guides/api-client.md#adding-endpoints)** - Extend the API
- **[Creating Components](./frontend/components.md#creating-components)** - Build UI components
- **[Writing Tests](./guides/testing.md#writing-tests)** - Test best practices
- **[Deploying](./deployment/production.md#deployment-checklist)** - Production deployment

## Search Tips

### Finding Information
- Use browser search (Ctrl+F / Cmd+F) within documentation pages
- Check the table of contents at the top of each document
- Look for "Quick Navigation" sections for common tasks
- Use the directory structure to browse by category

### Navigation Shortcuts
```
docs/
├── getting-started/     → New users start here
├── backend/             → Rust backend docs
├── frontend/            → Next.js frontend docs
├── api/                 → API reference
├── guides/              → Step-by-step tutorials
├── deployment/          → Production deployment
└── troubleshooting/     → Problem solving
```

### Common Search Terms
- **Authentication**: [Backend](./backend/api-handlers.md#authentication) | [Frontend](./guides/authentication.md)
- **Database**: [Models](./backend/models.md) | [Migrations](./backend/database.md#migrations)
- **Theming**: [System](./frontend/themes.md) | [Guide](./guides/themes.md)
- **Testing**: [Backend](./backend/testing.md) | [Frontend](./frontend/testing.md)
- **API**: [Reference](./api/reference.md) | [Client](./frontend/api-client.md)

## Documentation Standards

This documentation follows these principles:

- **Keep it current**: Update docs when code changes
- **Be concise**: Respect the reader's time
- **Show examples**: Code examples over lengthy explanations
- **Link generously**: Connect related topics
- **Test accuracy**: All code samples should work
- **Visual aids**: Use diagrams for complex concepts

### Documentation Quality

We maintain high documentation standards:
- ✅ 49 documentation files
- ✅ 23 Mermaid diagrams
- ✅ ~200+ code samples
- ✅ Comprehensive API reference
- ✅ Step-by-step guides
- ✅ Troubleshooting support

### Contributing to Documentation

Found an issue or want to improve the docs?
- **[Contributing Guidelines](./contributing/README.md)** - How to contribute
- **[Validation Checklist](./.validation-checklist.md)** - Quality standards
- **[Issue Tracker](https://github.com/your-org/cobalt-stack/issues)** - Report problems

---

**Documentation Version:** 1.0.0
**Last Updated:** 2025-10-27
**License:** MIT
**Maintained by:** Cobalt Stack Team
