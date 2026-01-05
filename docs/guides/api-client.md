# API Client Guide

Complete guide to using the API client in Cobalt Stack frontend.

## Table of Contents

- [Overview](#overview)
- [Basic Usage](#basic-usage)
- [Making API Calls](#making-api-calls)
- [Authentication](#authentication)
- [Error Handling](#error-handling)
- [Type Safety](#type-safety)
- [Advanced Patterns](#advanced-patterns)
- [Troubleshooting](#troubleshooting)

## Overview

Cobalt Stack includes a type-safe API client that:

- **Type-safe**: Generated TypeScript types from OpenAPI schema
- **Centralized**: Single configuration point for API calls
- **Error handling**: Consistent error response structure
- **Authentication**: Built-in token management
- **Environment-aware**: Uses environment-specific API URLs

### Architecture

```
Component
    ↓
API Client (Typed)
    ↓
Fetch API
    ↓
Backend API (Rust)
```

## Basic Usage

### API Client Setup

The API client is defined in `frontend/src/lib/api-client.ts`:

```typescript
import { env } from './env'
import type { paths, components } from '@/types/api'

export type ApiResponse<T> =
  | { success: true; data: T }
  | { success: false; error: string }

class ApiClient {
  private baseUrl: string

  constructor(baseUrl: string = env.apiUrl) {
    this.baseUrl = baseUrl
  }

  // HTTP methods defined here
}

export const apiClient = new ApiClient()
```

### Environment Configuration

Configure API URL in `frontend/.env.local`:

```bash
# Development
NEXT_PUBLIC_API_URL=http://localhost:2750

# Production
NEXT_PUBLIC_API_URL=https://api.yourapp.com
```

### Simple API Call

```tsx
import { apiClient } from '@/lib/api-client'

async function checkHealth() {
  const response = await apiClient.healthCheck()

  if (response.success) {
    console.log('API Status:', response.data.status)
  } else {
    console.error('Health check failed:', response.error)
  }
}
```

## Making API Calls

### GET Requests

```typescript
// In api-client.ts
class ApiClient {
  private async get<T>(path: string): Promise<ApiResponse<T>> {
    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        return {
          success: false,
          error: `HTTP ${response.status}: ${response.statusText}`,
        }
      }

      const data = await response.json()
      return { success: true, data }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      }
    }
  }

  // Public methods
  getUser = (id: string) => this.get<User>(`/api/users/${id}`)
  getUsers = () => this.get<User[]>('/api/users')
}
```

Usage:

```tsx
const response = await apiClient.getUser('123')

if (response.success) {
  const user = response.data
  console.log(user.username)
}
```

### POST Requests

```typescript
class ApiClient {
  private async post<T, D>(
    path: string,
    body: D
  ): Promise<ApiResponse<T>> {
    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
      })

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}))
        return {
          success: false,
          error: errorData.message || `HTTP ${response.status}`,
        }
      }

      const data = await response.json()
      return { success: true, data }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      }
    }
  }

  // Public methods
  login = (credentials: LoginRequest) =>
    this.post<LoginResponse, LoginRequest>('/api/auth/login', credentials)
}
```

Usage:

```tsx
const response = await apiClient.login({
  username: 'alice',
  password: 'password123',
})

if (response.success) {
  const { access_token, user } = response.data
  // Handle successful login
}
```

### PUT/PATCH Requests

```typescript
class ApiClient {
  private async put<T, D>(
    path: string,
    body: D
  ): Promise<ApiResponse<T>> {
    // Similar to POST but with PUT method
  }

  private async patch<T, D>(
    path: string,
    body: D
  ): Promise<ApiResponse<T>> {
    // Similar to POST but with PATCH method
  }

  // Public methods
  updateUser = (id: string, data: UpdateUserRequest) =>
    this.put<User, UpdateUserRequest>(`/api/users/${id}`, data)
}
```

### DELETE Requests

```typescript
class ApiClient {
  private async delete<T>(path: string): Promise<ApiResponse<T>> {
    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        method: 'DELETE',
      })

      if (!response.ok) {
        return {
          success: false,
          error: `HTTP ${response.status}: ${response.statusText}`,
        }
      }

      // Some DELETE requests return empty response
      const data = response.status === 204 ? null : await response.json()
      return { success: true, data }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      }
    }
  }

  // Public methods
  deleteUser = (id: string) => this.delete<void>(`/api/users/${id}`)
}
```

## Authentication

### Adding Authorization Header

```typescript
class ApiClient {
  private getAuthHeaders(token?: string): HeadersInit {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    }

    if (token) {
      headers['Authorization'] = `Bearer ${token}`
    }

    return headers
  }

  private async authenticatedGet<T>(
    path: string,
    token: string
  ): Promise<ApiResponse<T>> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'GET',
      headers: this.getAuthHeaders(token),
    })
    // ... rest of implementation
  }
}
```

### Using with Auth Context

```tsx
'use client'

import { useAuth } from '@/contexts/auth-context'
import { apiClient } from '@/lib/api-client'

export function UserProfile() {
  const { accessToken } = useAuth()
  const [profile, setProfile] = useState(null)

  useEffect(() => {
    async function fetchProfile() {
      if (!accessToken) return

      const response = await fetch(`${env.apiUrl}/api/auth/me`, {
        headers: {
          'Authorization': `Bearer ${accessToken}`,
        },
      })

      if (response.ok) {
        const data = await response.json()
        setProfile(data)
      }
    }

    fetchProfile()
  }, [accessToken])

  return <div>{profile?.username}</div>
}
```

### Authenticated API Client Methods

```typescript
class ApiClient {
  // Authenticated endpoints
  getProfile = (token: string) =>
    this.authenticatedGet<User>('/api/auth/me', token)

  updateProfile = (token: string, data: UpdateProfileRequest) =>
    this.authenticatedPut<User, UpdateProfileRequest>(
      '/api/auth/me',
      data,
      token
    )

  // Admin endpoints
  listUsers = (token: string) =>
    this.authenticatedGet<UserListResponse>('/api/admin/users', token)
}
```

### Including Cookies

For endpoints that use HttpOnly cookies (like refresh tokens):

```typescript
const response = await fetch(`${env.apiUrl}/api/auth/refresh`, {
  method: 'POST',
  credentials: 'include', // Important: Send/receive cookies
  headers: {
    'Content-Type': 'application/json',
  },
})
```

## Error Handling

### Consistent Error Response

```typescript
export type ApiResponse<T> =
  | { success: true; data: T }
  | { success: false; error: string }
```

### Handling Errors in Components

```tsx
async function submitForm() {
  setLoading(true)
  setError('')

  const response = await apiClient.createPost(formData)

  if (response.success) {
    // Success
    toast.success('Post created successfully')
    router.push(`/posts/${response.data.id}`)
  } else {
    // Error
    setError(response.error)
    toast.error(response.error)
  }

  setLoading(false)
}
```

### HTTP Status Code Handling

```typescript
private async handleResponse<T>(response: Response): Promise<ApiResponse<T>> {
  if (!response.ok) {
    // Parse error message from response
    const errorData = await response.json().catch(() => ({}))

    // Map status codes to user-friendly messages
    let error: string
    switch (response.status) {
      case 400:
        error = errorData.message || 'Invalid request'
        break
      case 401:
        error = 'Authentication required'
        break
      case 403:
        error = 'Access denied'
        break
      case 404:
        error = 'Resource not found'
        break
      case 429:
        error = 'Too many requests, please slow down'
        break
      case 500:
        error = 'Server error, please try again'
        break
      default:
        error = `HTTP ${response.status}: ${response.statusText}`
    }

    return { success: false, error }
  }

  const data = await response.json()
  return { success: true, data }
}
```

### Network Error Handling

```typescript
try {
  const response = await fetch(url, options)
  return await this.handleResponse(response)
} catch (error) {
  // Network errors, timeouts, CORS issues
  if (error instanceof TypeError) {
    return {
      success: false,
      error: 'Network error. Please check your connection.',
    }
  }

  return {
    success: false,
    error: error instanceof Error ? error.message : 'Unknown error',
  }
}
```

### Retry Logic

```typescript
async function fetchWithRetry<T>(
  fetcher: () => Promise<ApiResponse<T>>,
  maxRetries = 3,
  delay = 1000
): Promise<ApiResponse<T>> {
  let lastError: string = ''

  for (let i = 0; i < maxRetries; i++) {
    const response = await fetcher()

    if (response.success) {
      return response
    }

    lastError = response.error

    // Don't retry on client errors (4xx)
    if (response.error.includes('400') || response.error.includes('404')) {
      return response
    }

    // Wait before retry (exponential backoff)
    if (i < maxRetries - 1) {
      await new Promise((resolve) => setTimeout(resolve, delay * (i + 1)))
    }
  }

  return {
    success: false,
    error: `Failed after ${maxRetries} attempts: ${lastError}`,
  }
}
```

## Type Safety

### Generating Types from OpenAPI

Generate TypeScript types from OpenAPI schema:

```bash
# From frontend directory
make generate-types

# Or manually
cd frontend
bunx openapi-typescript ../openapi/schema.json -o src/types/api.ts
```

### Using Generated Types

```typescript
import type { components, paths } from '@/types/api'

// Component types
type User = components['schemas']['User']
type LoginRequest = components['schemas']['LoginRequest']
type LoginResponse = components['schemas']['LoginResponse']

// Path operation types
type GetUserResponse =
  paths['/api/users/{id}']['get']['responses']['200']['content']['application/json']

type PostLoginRequest =
  paths['/api/auth/login']['post']['requestBody']['content']['application/json']
```

### Type-Safe API Client

```typescript
import type { components } from '@/types/api'

type User = components['schemas']['User']
type HealthResponse = components['schemas']['HealthResponse']

class ApiClient {
  // Type-safe methods
  healthCheck = (): Promise<ApiResponse<HealthResponse>> =>
    this.get<HealthResponse>('/health')

  getUser = (id: string): Promise<ApiResponse<User>> =>
    this.get<User>(`/api/users/${id}`)
}
```

### Type Guards

```typescript
function isSuccessResponse<T>(
  response: ApiResponse<T>
): response is { success: true; data: T } {
  return response.success === true
}

// Usage
const response = await apiClient.getUser('123')

if (isSuccessResponse(response)) {
  // TypeScript knows response.data exists
  console.log(response.data.username)
}
```

## Advanced Patterns

### Custom Hooks

```typescript
// hooks/use-api.ts
import { useState, useEffect } from 'react'
import type { ApiResponse } from '@/lib/api-client'

export function useApi<T>(
  fetcher: () => Promise<ApiResponse<T>>,
  dependencies: any[] = []
) {
  const [data, setData] = useState<T | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let mounted = true

    async function fetch() {
      setLoading(true)
      setError(null)

      const response = await fetcher()

      if (!mounted) return

      if (response.success) {
        setData(response.data)
      } else {
        setError(response.error)
      }

      setLoading(false)
    }

    fetch()

    return () => {
      mounted = false
    }
  }, dependencies)

  return { data, error, loading }
}
```

Usage:

```tsx
function UserProfile({ userId }: { userId: string }) {
  const { data: user, error, loading } = useApi(
    () => apiClient.getUser(userId),
    [userId]
  )

  if (loading) return <div>Loading...</div>
  if (error) return <div>Error: {error}</div>
  if (!user) return <div>User not found</div>

  return <div>{user.username}</div>
}
```

### Mutation Hook

```typescript
export function useMutation<T, D>(
  mutator: (data: D) => Promise<ApiResponse<T>>
) {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const mutate = async (data: D): Promise<T | null> => {
    setLoading(true)
    setError(null)

    const response = await mutator(data)

    setLoading(false)

    if (response.success) {
      return response.data
    } else {
      setError(response.error)
      return null
    }
  }

  return { mutate, loading, error }
}
```

Usage:

```tsx
function CreatePostForm() {
  const { mutate, loading, error } = useMutation(apiClient.createPost)

  const handleSubmit = async (formData: CreatePostRequest) => {
    const post = await mutate(formData)

    if (post) {
      router.push(`/posts/${post.id}`)
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      {error && <div className="error">{error}</div>}
      <button type="submit" disabled={loading}>
        {loading ? 'Creating...' : 'Create Post'}
      </button>
    </form>
  )
}
```

### Request Cancellation

```typescript
export function useApiWithCancel<T>(
  fetcher: (signal: AbortSignal) => Promise<ApiResponse<T>>,
  dependencies: any[] = []
) {
  const [data, setData] = useState<T | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const controller = new AbortController()

    async function fetch() {
      setLoading(true)
      setError(null)

      const response = await fetcher(controller.signal)

      if (response.success) {
        setData(response.data)
      } else {
        setError(response.error)
      }

      setLoading(false)
    }

    fetch()

    return () => {
      controller.abort()
    }
  }, dependencies)

  return { data, error, loading }
}
```

### Caching

```typescript
class ApiClient {
  private cache = new Map<string, { data: any; timestamp: number }>()
  private cacheDuration = 5 * 60 * 1000 // 5 minutes

  private async getCached<T>(
    key: string,
    fetcher: () => Promise<ApiResponse<T>>
  ): Promise<ApiResponse<T>> {
    const cached = this.cache.get(key)

    if (cached && Date.now() - cached.timestamp < this.cacheDuration) {
      return { success: true, data: cached.data }
    }

    const response = await fetcher()

    if (response.success) {
      this.cache.set(key, {
        data: response.data,
        timestamp: Date.now(),
      })
    }

    return response
  }

  // Cached method
  getUser = (id: string) =>
    this.getCached(`user-${id}`, () =>
      this.get<User>(`/api/users/${id}`)
    )
}
```

## Troubleshooting

### CORS Errors

**Problem**: "blocked by CORS policy"

**Solutions**:
1. Verify backend CORS configuration allows frontend origin
2. Check `FRONTEND_URL` in backend `.env`
3. Use `credentials: 'include'` for cookies
4. Verify API URL doesn't have trailing slash mismatch

### 401 Unauthorized

**Problem**: Getting 401 on authenticated endpoints

**Solutions**:
1. Check Authorization header format: `Bearer <token>`
2. Verify token hasn't expired
3. Check token is being passed correctly
4. Test token with curl or Postman
5. Check auth middleware on backend

### Network Request Failed

**Problem**: Fetch fails with network error

**Solutions**:
1. Check backend is running
2. Verify API URL is correct
3. Check for typos in endpoint paths
4. Test with curl: `curl http://localhost:2750/health`
5. Check browser DevTools Network tab

### Type Errors

**Problem**: TypeScript errors with API responses

**Solutions**:
1. Regenerate types: `make generate-types`
2. Check OpenAPI schema is up to date
3. Verify import paths are correct
4. Restart TypeScript server
5. Check for breaking changes in API

### Response Not JSON

**Problem**: "Unexpected token < in JSON"

**Solutions**:
1. Backend returning HTML error page (check status code)
2. CORS preflight failure
3. Wrong API URL (hitting frontend instead)
4. Server error (check backend logs)
5. Check Content-Type header

## Best Practices

1. **Use type-safe methods**: Generate types from OpenAPI schema
2. **Handle all error cases**: Network, HTTP, and application errors
3. **Consistent error messages**: User-friendly error descriptions
4. **Loading states**: Show loading indicators during requests
5. **Request cancellation**: Cancel requests on component unmount
6. **Retry logic**: Retry failed requests with exponential backoff
7. **Caching**: Cache frequently accessed data
8. **Environment variables**: Use env-specific API URLs
9. **Authentication**: Securely manage tokens
10. **Testing**: Mock API responses in tests

## Related Documentation

- [Authentication Guide](./authentication.md) - JWT authentication
- [API Reference](../api/README.md) - Complete API documentation
- [Testing Guide](./testing.md) - Testing API calls
- [Frontend Architecture](../frontend/README.md) - Frontend structure
