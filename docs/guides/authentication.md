# Authentication Guide

Complete guide to implementing and using JWT authentication in Cobalt Stack.

## Table of Contents

- [Overview](#overview)
- [Authentication Flow](#authentication-flow)
- [Backend Implementation](#backend-implementation)
- [Frontend Implementation](#frontend-implementation)
- [Protected Routes](#protected-routes)
- [Token Refresh](#token-refresh)
- [Logout](#logout)
- [Troubleshooting](#troubleshooting)

## Overview

Cobalt Stack implements a secure JWT-based authentication system with:

- **Access Tokens**: Short-lived (30 minutes), stored in memory
- **Refresh Tokens**: Long-lived (7 days), stored in HttpOnly cookies
- **Automatic Token Refresh**: Tokens refresh automatically before expiry
- **Token Rotation**: Each refresh generates a new refresh token for enhanced security

### Security Features

- HMAC-SHA256 (HS256) signature algorithm
- Token expiration validation
- Refresh token rotation via unique token IDs (jti)
- HttpOnly cookies prevent XSS attacks
- Access tokens in memory prevent CSRF attacks

## Authentication Flow

```
┌─────────────┐                 ┌─────────────┐
│   Client    │                 │   Backend   │
└─────────────┘                 └─────────────┘
       │                               │
       │  POST /api/auth/login        │
       │  { username, password }      │
       │─────────────────────────────>│
       │                               │
       │                          ✓ Verify
       │                          credentials
       │                               │
       │  200 OK                       │
       │  { access_token, user }       │
       │  Set-Cookie: refresh_token    │
       │<─────────────────────────────│
       │                               │
       │  Store access_token           │
       │  in memory                    │
       │                               │
       │  Authenticated requests       │
       │  Authorization: Bearer token  │
       │─────────────────────────────>│
       │                               │
       │  POST /api/auth/refresh       │
       │  Cookie: refresh_token        │
       │─────────────────────────────>│
       │                               │
       │  200 OK                       │
       │  { access_token }             │
       │  Set-Cookie: new_refresh      │
       │<─────────────────────────────│
```

## Backend Implementation

### JWT Configuration

Configure JWT settings in your `.env` file:

```bash
# JWT secret key (REQUIRED in production)
JWT_SECRET=your_secure_secret_key_here_minimum_32_characters

# Access token lifetime (default: 30 minutes)
JWT_ACCESS_EXPIRY_MINUTES=30

# Refresh token lifetime (default: 7 days)
JWT_REFRESH_EXPIRY_DAYS=7
```

### Creating Tokens

```rust
use cobalt_stack::services::auth::jwt::{
    JwtConfig, create_access_token, create_refresh_token
};
use uuid::Uuid;

// Load configuration
let config = JwtConfig::from_env();

// Create access token
let user_id = Uuid::new_v4();
let username = "alice".to_string();
let access_token = create_access_token(user_id, username, &config)?;

// Create refresh token (returns token and jti)
let (refresh_token, jti) = create_refresh_token(user_id, &config)?;

// Store refresh token in database with jti
```

### Verifying Tokens

```rust
use cobalt_stack::services::auth::jwt::{
    verify_access_token, verify_refresh_token
};

// Verify access token
let claims = verify_access_token(&token, &config)?;
println!("User ID: {}", claims.sub);
println!("Username: {}", claims.username);

// Verify refresh token
let refresh_claims = verify_refresh_token(&token, &config)?;
println!("Token ID: {}", refresh_claims.jti);
```

### Login Endpoint Example

```rust
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Verify credentials
    let user = verify_password(&state.db, &req.username, &req.password).await?;

    // 2. Create tokens
    let access_token = create_access_token(user.id, user.username.clone(), &state.jwt_config)?;
    let (refresh_token, jti) = create_refresh_token(user.id, &state.jwt_config)?;

    // 3. Store refresh token in database
    store_refresh_token(&state.db, user.id, jti, expires_at).await?;

    // 4. Return access token and set refresh token cookie
    Ok((
        cookies.add(create_refresh_cookie(refresh_token)),
        Json(LoginResponse { access_token, user })
    ))
}
```

### Authentication Middleware

The auth middleware verifies the access token on protected routes:

```rust
use axum::{
    middleware::Next,
    extract::Request,
    http::StatusCode,
};

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Bearer token from Authorization header
    let token = extract_token(&req)?;

    // Verify token
    let claims = verify_access_token(&token, &state.jwt_config)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add user info to request extensions
    req.extensions_mut().insert(AuthUser {
        user_id: claims.sub,
        username: claims.username,
    });

    Ok(next.run(req).await)
}
```

## Frontend Implementation

### AuthProvider Setup

Wrap your app with the `AuthProvider` in `app/layout.tsx`:

```tsx
import { AuthProvider } from '@/contexts/auth-context'

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <AuthProvider>
          {children}
        </AuthProvider>
      </body>
    </html>
  )
}
```

### Login Component

```tsx
'use client'

import { useState } from 'react'
import { useAuth } from '@/contexts/auth-context'
import { useRouter } from 'next/navigation'
import { env } from '@/lib/env'

export function LoginForm() {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const { login } = useAuth()
  const router = useRouter()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    try {
      const response = await fetch(`${env.apiUrl}/api/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include', // Important: Send/receive cookies
        body: JSON.stringify({ username, password }),
      })

      if (!response.ok) {
        const data = await response.json()
        setError(data.message || 'Login failed')
        return
      }

      const data = await response.json()

      // Store access token and user in auth context
      login(data.access_token, data.user)

      // Redirect to dashboard
      router.push('/dashboard')
    } catch (err) {
      setError('Network error. Please try again.')
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      {error && <div className="error">{error}</div>}

      <input
        type="text"
        value={username}
        onChange={(e) => setUsername(e.target.value)}
        placeholder="Username"
        required
      />

      <input
        type="password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        placeholder="Password"
        required
      />

      <button type="submit">Login</button>
    </form>
  )
}
```

### Using the useAuth Hook

```tsx
'use client'

import { useAuth } from '@/contexts/auth-context'

export function UserProfile() {
  const { user, isAuthenticated, isLoading } = useAuth()

  if (isLoading) {
    return <div>Loading...</div>
  }

  if (!isAuthenticated) {
    return <div>Please log in</div>
  }

  return (
    <div>
      <h2>Welcome, {user?.username}!</h2>
      <p>Email: {user?.email}</p>
      <p>Role: {user?.role}</p>
      <p>Verified: {user?.email_verified ? 'Yes' : 'No'}</p>
    </div>
  )
}
```

## Protected Routes

### Backend: Route Protection

Apply the `auth_middleware` to protect routes:

```rust
use axum::{
    Router,
    routing::get,
    middleware,
};

let protected_routes = Router::new()
    .route("/api/profile", get(get_profile))
    .route("/api/settings", get(get_settings))
    .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));
```

### Frontend: Protected Pages

Create a protected route wrapper:

```tsx
// components/auth/protected-route.tsx
'use client'

import { useAuth } from '@/contexts/auth-context'
import { useRouter } from 'next/navigation'
import { useEffect } from 'react'

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAuth()
  const router = useRouter()

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.push('/login')
    }
  }, [isAuthenticated, isLoading, router])

  if (isLoading) {
    return <div>Loading...</div>
  }

  if (!isAuthenticated) {
    return null
  }

  return <>{children}</>
}
```

Use it in your pages:

```tsx
// app/dashboard/page.tsx
import { ProtectedRoute } from '@/components/auth/protected-route'

export default function DashboardPage() {
  return (
    <ProtectedRoute>
      <div>
        <h1>Dashboard</h1>
        <p>This page requires authentication</p>
      </div>
    </ProtectedRoute>
  )
}
```

## Token Refresh

### Automatic Refresh

The `AuthProvider` automatically refreshes tokens 5 minutes before expiry:

```tsx
// In auth-context.tsx
useEffect(() => {
  if (!isAuthenticated || !accessToken) return

  const REFRESH_BEFORE_EXPIRY = 25 * 60 * 1000 // 25 minutes

  const scheduleRefresh = () => {
    setTimeout(async () => {
      const success = await refreshToken()
      if (success) {
        scheduleRefresh() // Schedule next refresh
      }
    }, REFRESH_BEFORE_EXPIRY)
  }

  scheduleRefresh()
}, [accessToken, isAuthenticated])
```

### Manual Refresh

You can manually trigger a refresh:

```tsx
const { refreshToken } = useAuth()

const handleRefresh = async () => {
  const success = await refreshToken()
  if (success) {
    console.log('Token refreshed successfully')
  } else {
    console.log('Refresh failed, please log in again')
  }
}
```

### Backend Refresh Endpoint

```rust
async fn refresh_token(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> Result<Json<RefreshResponse>, StatusCode> {
    // 1. Extract refresh token from cookie
    let refresh_token = cookies
        .get("refresh_token")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .value();

    // 2. Verify refresh token
    let claims = verify_refresh_token(refresh_token, &state.jwt_config)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 3. Check if token is in database and not revoked
    verify_refresh_token_in_db(&state.db, claims.jti).await?;

    // 4. Create new tokens
    let user = get_user(&state.db, claims.sub).await?;
    let new_access = create_access_token(user.id, user.username, &state.jwt_config)?;
    let (new_refresh, new_jti) = create_refresh_token(user.id, &state.jwt_config)?;

    // 5. Rotate refresh token (invalidate old, store new)
    rotate_refresh_token(&state.db, claims.jti, new_jti).await?;

    // 6. Return new tokens
    Ok((
        cookies.add(create_refresh_cookie(new_refresh)),
        Json(RefreshResponse { access_token: new_access })
    ))
}
```

## Logout

### Frontend Logout

```tsx
'use client'

import { useAuth } from '@/contexts/auth-context'
import { Button } from '@/components/ui/button'

export function LogoutButton() {
  const { logout } = useAuth()

  const handleLogout = async () => {
    await logout()
    // Redirect to login page
    window.location.href = '/login'
  }

  return (
    <Button onClick={handleLogout}>
      Logout
    </Button>
  )
}
```

### Backend Logout Endpoint

```rust
async fn logout(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> StatusCode {
    // Extract refresh token from cookie
    if let Some(refresh_cookie) = cookies.get("refresh_token") {
        let token = refresh_cookie.value();

        // Verify and revoke token
        if let Ok(claims) = verify_refresh_token(token, &state.jwt_config) {
            let _ = revoke_refresh_token(&state.db, claims.jti).await;
        }
    }

    // Clear refresh token cookie
    cookies.remove(Cookie::named("refresh_token"));

    StatusCode::OK
}
```

## Troubleshooting

### Token Expired Errors

**Problem**: Frequent "Token expired" errors

**Solutions**:
1. Check if automatic refresh is working
2. Verify JWT_ACCESS_EXPIRY_MINUTES in backend `.env`
3. Check browser console for refresh errors
4. Ensure credentials: 'include' in fetch requests

### Unauthorized Errors

**Problem**: Getting 401 Unauthorized on protected routes

**Solutions**:
1. Verify Authorization header format: `Bearer <token>`
2. Check if token is being sent with request
3. Verify JWT_SECRET matches between requests
4. Check if token is expired using jwt.io

### CORS Issues with Cookies

**Problem**: Refresh token cookie not being set/sent

**Solutions**:
1. Ensure backend CORS allows credentials:
   ```rust
   CorsLayer::new()
       .allow_credentials(true)
       .allow_origin(Origin::exact(frontend_url))
   ```
2. Frontend must use `credentials: 'include'`
3. Frontend and backend must be on allowed origins

### Token Refresh Loop

**Problem**: Token keeps refreshing continuously

**Solutions**:
1. Check refresh timing (should be 25 minutes, not 30)
2. Verify refresh endpoint isn't triggering auth middleware
3. Check for errors in refresh token endpoint

### Invalid Signature Errors

**Problem**: JWT verification fails with invalid signature

**Solutions**:
1. Verify JWT_SECRET is identical in all environments
2. Check if secret has whitespace or special characters
3. Ensure secret is at least 32 characters long
4. Restart backend after changing JWT_SECRET

## Best Practices

1. **Never log tokens**: Tokens contain sensitive information
2. **Use HTTPS in production**: Prevents token interception
3. **Implement token rotation**: Refresh tokens should be single-use
4. **Set appropriate expiry times**: Balance security and UX
5. **Validate tokens server-side**: Never trust client claims
6. **Store access tokens in memory**: Prevents XSS attacks
7. **Use HttpOnly cookies for refresh tokens**: Prevents JavaScript access
8. **Implement logout everywhere**: Clear tokens on all devices
9. **Monitor failed auth attempts**: Detect brute force attacks
10. **Use secure JWT secrets**: Minimum 32 characters, randomly generated

## Related Documentation

- [Email Verification Guide](./email-verification.md) - Email verification after registration
- [Admin Dashboard Guide](./admin-dashboard.md) - Admin role authentication
- [API Client Guide](./api-client.md) - Making authenticated API calls
- [API Reference](../api/README.md) - Complete API documentation
- [Backend Auth Module](../backend/README.md#authentication) - Backend architecture details
