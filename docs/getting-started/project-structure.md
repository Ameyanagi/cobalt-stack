# Project Structure

Complete overview of the Cobalt Stack codebase organization.

## Table of Contents

- [Overview](#overview)
- [Root Directory](#root-directory)
- [Backend Structure](#backend-structure)
- [Frontend Structure](#frontend-structure)
- [Shared Resources](#shared-resources)
- [Configuration Files](#configuration-files)
- [Navigation Guide](#navigation-guide)

## Overview

Cobalt Stack follows a monorepo structure with clear separation between backend, frontend, and shared resources.

```
cobalt-stack/
├── backend/          # Rust backend services
├── frontend/         # React/Next.js frontend
├── shared/           # Shared types and utilities
├── docs/             # Documentation
├── scripts/          # Build and deployment scripts
└── docker-compose.yml
```

## Root Directory

```
cobalt-stack/
├── .env.example          # Environment template
├── .gitignore           # Git ignore rules
├── Cargo.toml           # Rust workspace config
├── Cargo.lock           # Rust dependency lock
├── docker-compose.yml   # Development services
├── docker-compose.prod.yml  # Production services
├── Makefile             # Convenience commands
├── package.json         # Node.js dependencies
├── README.md            # Project overview
└── IMPLEMENTATION_GUIDE.md  # Development guide
```

### Key Files

| File | Purpose |
|------|---------|
| `.env.example` | Environment variables template |
| `Cargo.toml` | Rust workspace and dependencies |
| `docker-compose.yml` | Docker service definitions |
| `Makefile` | Common development commands |
| `package.json` | Frontend dependencies |

## Backend Structure

```
backend/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library root
│   ├── api/             # API endpoints
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── users.rs
│   │   └── health.rs
│   ├── models/          # Data models
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   └── session.rs
│   ├── services/        # Business logic
│   │   ├── mod.rs
│   │   ├── auth_service.rs
│   │   └── user_service.rs
│   ├── db/              # Database layer
│   │   ├── mod.rs
│   │   ├── pool.rs
│   │   └── migrations/
│   ├── middleware/      # HTTP middleware
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   └── logging.rs
│   ├── utils/           # Utility functions
│   │   ├── mod.rs
│   │   ├── jwt.rs
│   │   └── validation.rs
│   └── config/          # Configuration
│       ├── mod.rs
│       └── settings.rs
├── tests/               # Integration tests
│   ├── api_tests.rs
│   └── common/
├── migrations/          # Database migrations
├── Cargo.toml          # Package config
└── Dockerfile          # Container image
```

### Backend Layers

#### API Layer (`src/api/`)
- HTTP endpoint handlers
- Request/response mapping
- Route definitions
- OpenAPI documentation

#### Service Layer (`src/services/`)
- Business logic
- Transaction management
- Domain operations
- Service coordination

#### Data Layer (`src/db/`)
- Database connections
- Query execution
- Transaction handling
- Migration management

#### Models (`src/models/`)
- Data structures
- Database mappings
- Serialization/deserialization
- Validation rules

## Frontend Structure

```
frontend/
├── src/
│   ├── app/             # Next.js app directory
│   │   ├── layout.tsx   # Root layout
│   │   ├── page.tsx     # Home page
│   │   ├── login/
│   │   ├── dashboard/
│   │   └── api/         # API routes
│   ├── components/      # React components
│   │   ├── ui/          # UI components
│   │   │   ├── Button.tsx
│   │   │   ├── Card.tsx
│   │   │   └── Input.tsx
│   │   ├── layout/      # Layout components
│   │   │   ├── Header.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── Footer.tsx
│   │   └── features/    # Feature components
│   │       ├── auth/
│   │       └── dashboard/
│   ├── hooks/           # Custom React hooks
│   │   ├── useAuth.ts
│   │   ├── useApi.ts
│   │   └── useTheme.ts
│   ├── lib/             # Libraries and utilities
│   │   ├── api.ts
│   │   ├── auth.ts
│   │   └── utils.ts
│   ├── types/           # TypeScript types
│   │   ├── api.ts
│   │   ├── user.ts
│   │   └── index.ts
│   ├── styles/          # Global styles
│   │   ├── globals.css
│   │   └── theme.css
│   └── contexts/        # React contexts
│       ├── AuthContext.tsx
│       └── ThemeContext.tsx
├── public/              # Static assets
│   ├── images/
│   ├── icons/
│   └── favicon.ico
├── tests/               # Test files
│   ├── components/
│   └── integration/
├── package.json         # Dependencies
├── tsconfig.json        # TypeScript config
├── next.config.js       # Next.js config
├── tailwind.config.js   # Tailwind CSS config
└── Dockerfile           # Container image
```

### Frontend Organization

#### App Directory (`src/app/`)
- Next.js 13+ app router
- Server and client components
- Route handlers
- Layouts and templates

#### Components (`src/components/`)
- **ui/**: Reusable UI components
- **layout/**: Page structure components
- **features/**: Business logic components

#### Hooks (`src/hooks/`)
- Custom React hooks
- Reusable stateful logic
- Side effect management

#### Types (`src/types/`)
- TypeScript type definitions
- API interfaces
- Shared types

## Shared Resources

```
shared/
├── types/               # Shared TypeScript/Rust types
│   ├── user.ts
│   └── api.ts
├── utils/               # Shared utilities
│   └── validation.ts
└── constants/           # Shared constants
    └── config.ts
```

### Shared Code

Shared between frontend and backend:
- API type definitions
- Validation schemas
- Constants and enums
- Utility functions

## Configuration Files

### Docker Configuration

```
├── docker-compose.yml       # Development environment
├── docker-compose.prod.yml  # Production environment
├── backend/Dockerfile       # Backend container
└── frontend/Dockerfile      # Frontend container
```

### Build Configuration

```
├── Cargo.toml              # Rust workspace
├── backend/Cargo.toml      # Backend dependencies
├── package.json            # Node.js dependencies
├── tsconfig.json           # TypeScript config
└── next.config.js          # Next.js config
```

### Development Tools

```
├── .gitignore             # Git ignore rules
├── .env.example           # Environment template
├── .prettierrc            # Code formatting
├── .eslintrc.json         # Linting rules
└── rust-toolchain.toml    # Rust version
```

## Navigation Guide

### Finding Code

| What You Need | Where to Look |
|---------------|---------------|
| API endpoints | `backend/src/api/` |
| Business logic | `backend/src/services/` |
| Database queries | `backend/src/db/` |
| Data models | `backend/src/models/` |
| React components | `frontend/src/components/` |
| Pages | `frontend/src/app/` |
| Custom hooks | `frontend/src/hooks/` |
| API client | `frontend/src/lib/api.ts` |
| Types | `frontend/src/types/` or `shared/types/` |

### Common Tasks

#### Add New API Endpoint

1. Define model in `backend/src/models/`
2. Add service logic in `backend/src/services/`
3. Create endpoint in `backend/src/api/`
4. Add types in `frontend/src/types/`
5. Create API client method in `frontend/src/lib/api.ts`

#### Add New Page

1. Create page in `frontend/src/app/[page-name]/page.tsx`
2. Add components in `frontend/src/components/features/[feature]/`
3. Create custom hooks in `frontend/src/hooks/`
4. Add types in `frontend/src/types/`

#### Add Database Migration

1. Create migration: `cd backend && cargo run --bin create-migration -- name`
2. Edit migration file in `backend/migrations/`
3. Run migration: `cargo run --bin migrate`

## Project Conventions

### Naming Conventions

**Backend (Rust)**
- Files: `snake_case.rs`
- Functions: `snake_case()`
- Types: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`

**Frontend (TypeScript)**
- Files: `PascalCase.tsx` (components), `camelCase.ts` (utilities)
- Components: `PascalCase`
- Functions: `camelCase()`
- Types: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`

### Directory Patterns

- One module per directory
- `mod.rs` / `index.ts` for directory exports
- Separate tests in `tests/` directory
- Keep related files together

### Import Order

**Rust**
```rust
// Standard library
use std::collections::HashMap;

// External crates
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

// Internal modules
use crate::models::User;
use crate::services::AuthService;
```

**TypeScript**
```typescript
// React
import React from 'react';

// External libraries
import { useRouter } from 'next/router';
import axios from 'axios';

// Internal components
import { Button } from '@/components/ui/Button';
import { useAuth } from '@/hooks/useAuth';

// Types
import type { User } from '@/types/user';

// Styles
import styles from './Component.module.css';
```

## Related Resources

- [Quick Start Guide](./quick-start.md)
- [Installation Guide](./installation.md)
- [Backend Documentation](../backend/README.md)
- [Frontend Documentation](../frontend/README.md)
- [API Reference](../api/README.md)
- [Contributing Guide](../contributing/README.md)

---

**Next Steps**: Ready to start coding? Check out our [Development Guides](../guides/README.md).
