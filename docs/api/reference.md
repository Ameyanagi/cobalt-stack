# API Reference

Comprehensive API reference for Cobalt Stack REST API.

## Table of Contents

- [Overview](#overview)
- [Base URL](#base-url)
- [Authentication](#authentication)
- [Request/Response Format](#requestresponse-format)
- [Error Handling](#error-handling)
- [HTTP Status Codes](#http-status-codes)
- [Pagination](#pagination)
- [Filtering and Sorting](#filtering-and-sorting)
- [Rate Limiting](#rate-limiting)
- [Code Examples](#code-examples)

## Overview

The Cobalt Stack API is a RESTful API that uses JSON for serialization and JWT tokens for authentication. All API endpoints are prefixed with `/api` and follow REST conventions.

### API Characteristics

- **Protocol**: HTTPS (HTTP in development)
- **Format**: JSON
- **Authentication**: JWT Bearer tokens
- **Versioning**: Not yet implemented (future: `/api/v1`)

## Base URL

```
# Development
http://localhost:8000/api

# Production
https://your-domain.com/api
```

## Authentication

Most endpoints require authentication via JWT Bearer tokens.

### Authentication Flow

1. **Register** or **Login** to receive an access token
2. Include the token in the `Authorization` header
3. Refresh token automatically via HTTP-only cookie

### Authorization Header Format

```http
Authorization: Bearer <access_token>
```

### Token Types

| Token Type | Storage | Expiration | Purpose |
|------------|---------|------------|---------|
| Access Token | Client (memory/localStorage) | 15 minutes | API authentication |
| Refresh Token | HTTP-only cookie | 7 days | Token renewal |

### Token Lifecycle

```
Register/Login → Access Token (15min) + Refresh Token (7d)
     ↓
Access Token expires → Call /api/auth/refresh
     ↓
New Access Token (15min) + New Refresh Token (7d)
```

## Request/Response Format

### Request Format

All POST/PATCH requests must include:

```http
Content-Type: application/json
```

Request body must be valid JSON:

```json
{
  "username": "alice",
  "password": "SecurePass123!"
}
```

### Response Format

#### Success Response

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "email": "alice@example.com",
  "email_verified": false,
  "role": "User"
}
```

#### Error Response

All errors follow this standard format:

```json
{
  "error": "Error message describing what went wrong"
}
```

## Error Handling

### Standard Error Response

All API errors return a consistent JSON structure:

```json
{
  "error": "Human-readable error message"
}
```

### Error Categories

| Category | HTTP Status | Description |
|----------|-------------|-------------|
| Authentication | 401 | Invalid credentials, expired/invalid token |
| Authorization | 403 | Insufficient permissions, email not verified |
| Validation | 400 | Invalid input, missing required fields |
| Not Found | 404 | Resource does not exist |
| Conflict | 409 | Resource already exists (e.g., duplicate username) |
| Rate Limit | 429 | Too many requests |
| Server Error | 500 | Internal server error |

### Example Error Responses

#### 400 Bad Request
```json
{
  "error": "Invalid input: Username must be between 3 and 50 characters"
}
```

#### 401 Unauthorized
```json
{
  "error": "Invalid credentials"
}
```

#### 403 Forbidden
```json
{
  "error": "Email not verified"
}
```

#### 404 Not Found
```json
{
  "error": "User not found"
}
```

#### 409 Conflict
```json
{
  "error": "User already exists"
}
```

#### 429 Too Many Requests
```json
{
  "error": "Too many login attempts"
}
```

#### 500 Internal Server Error
```json
{
  "error": "Database operation failed"
}
```

## HTTP Status Codes

### Success Codes

| Code | Name | Description |
|------|------|-------------|
| 200 | OK | Request succeeded |
| 201 | Created | Resource created successfully |
| 204 | No Content | Request succeeded with no response body |

### Client Error Codes

| Code | Name | Description |
|------|------|-------------|
| 400 | Bad Request | Invalid input or malformed request |
| 401 | Unauthorized | Authentication required or failed |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource does not exist |
| 409 | Conflict | Resource conflict (duplicate) |
| 422 | Unprocessable Entity | Validation failed |
| 429 | Too Many Requests | Rate limit exceeded |

### Server Error Codes

| Code | Name | Description |
|------|------|-------------|
| 500 | Internal Server Error | Server encountered an error |
| 502 | Bad Gateway | Upstream service failure |
| 503 | Service Unavailable | Service temporarily unavailable |
| 504 | Gateway Timeout | Upstream timeout |

## Pagination

List endpoints support pagination for large datasets.

### Pagination Parameters

| Parameter | Type | Default | Max | Description |
|-----------|------|---------|-----|-------------|
| `page` | integer | 1 | N/A | Page number (1-based) |
| `per_page` | integer | 20 | 100 | Items per page |

### Pagination Request

```http
GET /api/admin/users?page=2&per_page=50
```

### Pagination Response

```json
{
  "users": [
    { "id": "...", "username": "alice" },
    { "id": "...", "username": "bob" }
  ],
  "total": 150,
  "page": 2,
  "per_page": 50,
  "total_pages": 3
}
```

### Pagination Metadata

| Field | Type | Description |
|-------|------|-------------|
| `total` | integer | Total number of items |
| `page` | integer | Current page number |
| `per_page` | integer | Items per page |
| `total_pages` | integer | Total number of pages |

## Filtering and Sorting

### Filtering

Admin endpoints support filtering via query parameters:

```http
GET /api/admin/users?role=admin&email_verified=true
```

#### Available Filters

| Parameter | Type | Description |
|-----------|------|-------------|
| `role` | string | Filter by user role (`admin`, `user`) |
| `email_verified` | boolean | Filter by email verification status |
| `search` | string | Search username or email (partial match) |

### Sorting

Currently, list endpoints are sorted by:
- **Users**: `created_at` descending (newest first)

Future versions may support custom sorting.

## Rate Limiting

API endpoints are protected by rate limiting to prevent abuse.

### Rate Limit Rules

| Endpoint | Limit | Window | Scope |
|----------|-------|--------|-------|
| `/api/auth/login` | 5 requests | 15 minutes | Per IP address |
| `/api/auth/register` | 3 requests | 1 hour | Per IP address |
| Other endpoints | No limit | N/A | N/A |

### Rate Limit Headers

When rate limited, the API returns:

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json

{
  "error": "Too many login attempts"
}
```

### Best Practices

- Implement exponential backoff on 429 errors
- Cache access tokens to reduce /auth/login calls
- Use refresh token endpoint instead of re-authenticating

## Code Examples

### cURL

#### Register User
```bash
curl -X POST http://localhost:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "email": "alice@example.com",
    "password": "SecurePass123!"
  }'
```

#### Login
```bash
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "username": "alice",
    "password": "SecurePass123!"
  }'
```

#### Get Current User
```bash
curl -X GET http://localhost:8000/api/auth/me \
  -H "Authorization: Bearer <access_token>"
```

#### Refresh Token
```bash
curl -X POST http://localhost:8000/api/auth/refresh \
  -b cookies.txt \
  -c cookies.txt
```

### JavaScript (Fetch API)

#### Register User
```javascript
const response = await fetch('http://localhost:8000/api/auth/register', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    username: 'alice',
    email: 'alice@example.com',
    password: 'SecurePass123!',
  }),
  credentials: 'include', // Important for cookies
});

const data = await response.json();
if (response.ok) {
  localStorage.setItem('access_token', data.access_token);
} else {
  console.error('Registration failed:', data.error);
}
```

#### Login
```javascript
const response = await fetch('http://localhost:8000/api/auth/login', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    username: 'alice',
    password: 'SecurePass123!',
  }),
  credentials: 'include',
});

const data = await response.json();
if (response.ok) {
  localStorage.setItem('access_token', data.access_token);
}
```

#### Authenticated Request
```javascript
const token = localStorage.getItem('access_token');

const response = await fetch('http://localhost:8000/api/auth/me', {
  method: 'GET',
  headers: {
    'Authorization': `Bearer ${token}`,
  },
});

const user = await response.json();
```

#### Refresh Token
```javascript
const response = await fetch('http://localhost:8000/api/auth/refresh', {
  method: 'POST',
  credentials: 'include',
});

const data = await response.json();
if (response.ok) {
  localStorage.setItem('access_token', data.access_token);
}
```

#### Error Handling
```javascript
async function apiRequest(url, options = {}) {
  const token = localStorage.getItem('access_token');

  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token ? `Bearer ${token}` : '',
      ...options.headers,
    },
    credentials: 'include',
  });

  if (response.status === 401) {
    // Try to refresh token
    const refreshResponse = await fetch('/api/auth/refresh', {
      method: 'POST',
      credentials: 'include',
    });

    if (refreshResponse.ok) {
      const data = await refreshResponse.json();
      localStorage.setItem('access_token', data.access_token);

      // Retry original request
      return apiRequest(url, options);
    } else {
      // Redirect to login
      window.location.href = '/login';
    }
  }

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Request failed');
  }

  return response.json();
}
```

### Rust (reqwest)

#### Register User
```rust
use reqwest;
use serde_json::json;

let client = reqwest::Client::new();
let response = client
    .post("http://localhost:8000/api/auth/register")
    .json(&json!({
        "username": "alice",
        "email": "alice@example.com",
        "password": "SecurePass123!"
    }))
    .send()
    .await?;

if response.status().is_success() {
    let auth: AuthResponse = response.json().await?;
    println!("Access token: {}", auth.access_token);
}
```

#### Login
```rust
let response = client
    .post("http://localhost:8000/api/auth/login")
    .json(&json!({
        "username": "alice",
        "password": "SecurePass123!"
    }))
    .send()
    .await?;

let auth: AuthResponse = response.json().await?;
let access_token = auth.access_token;
```

#### Authenticated Request
```rust
let response = client
    .get("http://localhost:8000/api/auth/me")
    .bearer_auth(&access_token)
    .send()
    .await?;

let user: UserResponse = response.json().await?;
```

## Related Documentation

- [Authentication Endpoints](./authentication.md)
- [User Endpoints](./users.md)
- [Admin Endpoints](./admin.md)
- [Quick Start Guide](../getting-started/quick-start.md)
