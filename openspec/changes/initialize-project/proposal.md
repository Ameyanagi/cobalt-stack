# Initialize Cobalt Stack Project

## Why

The Cobalt Stack project needs a proper foundation with correctly structured directories, configuration files, and tooling setup to support TDD/DDD backend development and Next.js frontend development. Without this initialization, we cannot begin building the working example application that will later be extracted into a cookiecutter template.

## What Changes

- Create backend project structure following DDD layered architecture (domain, application, infrastructure, presentation)
- Initialize Rust workspace with Cargo configuration
- Set up basic Makefile for common development tasks
- Create .gitignore for Rust and Node.js projects
- Add .env.example files for backend and frontend
- Create docker-compose.yml for local development orchestration
- Initialize backend with minimal health check endpoint (TDD approach)
- Set up testing infrastructure (cargo-tarpaulin, cargo-watch)
- Create README.md with setup instructions

## Impact

- **Affected specs**: Creates new `project-structure` capability
- **Affected code**: Initializes entire codebase structure
- **Dependencies**: Requires Rust 1.75+, Bun/Node.js 18+, Docker, Make, uv/uvx
- **Timeline**: 1-2 days to complete initialization and validate all tools work correctly
