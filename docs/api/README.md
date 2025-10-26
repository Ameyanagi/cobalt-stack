# API Reference

Complete API documentation for Cobalt Stack.

## Table of Contents

- [Overview](#overview)
- [Authentication](#authentication)
- [Endpoints](#endpoints)
- [Request/Response Format](#requestresponse-format)
- [Error Handling](#error-handling)
- [Rate Limiting](#rate-limiting)

## Overview

The Cobalt Stack API is a RESTful API built with Rust/Actix-web.

**Base URL**: `http://localhost:8080/api/v1`

**Content Type**: `application/json`

## Authentication

> **Note**: Detailed authentication documentation coming soon

### JWT Authentication

All protected endpoints require a JWT token in the Authorization header.

## Endpoints

> **Note**: Detailed endpoint documentation coming soon

### Health Check

```http
GET /api/health
```

### Authentication Endpoints

```http
POST /api/v1/auth/register
POST /api/v1/auth/login
POST /api/v1/auth/logout
POST /api/v1/auth/refresh
```

### User Endpoints

```http
GET    /api/v1/users
GET    /api/v1/users/:id
POST   /api/v1/users
PUT    /api/v1/users/:id
DELETE /api/v1/users/:id
```

## Request/Response Format

> **Note**: Detailed format documentation coming soon

## Error Handling

> **Note**: Error handling documentation coming soon

## Rate Limiting

> **Note**: Rate limiting documentation coming soon

---

**Related Resources**:
- [Backend Documentation](../backend/README.md)
- [Getting Started](../getting-started/quick-start.md)
- [Troubleshooting](../troubleshooting/README.md)
