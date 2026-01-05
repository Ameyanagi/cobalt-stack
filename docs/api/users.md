# User API

User management endpoints for Cobalt Stack.

## Table of Contents

- [Overview](#overview)
- [Endpoints](#endpoints)
  - [GET /api/auth/me](#get-apiauthme)
- [User Model](#user-model)
- [Examples](#examples)

## Overview

User endpoints provide access to user account information and profile management. All endpoints require authentication.

### Authentication Required

All user endpoints require a valid JWT access token in the Authorization header:

```http
Authorization: Bearer <access_token>
```

## Endpoints

### GET /api/auth/me

Get current authenticated user information.

**Authentication**: Required

#### Request

```http
GET /api/auth/me
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### Response

**Status**: `200 OK`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "email": "alice@example.com",
  "email_verified": true,
  "role": "User"
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string (UUID) | Unique user identifier |
| `username` | string | User's username (3-50 characters) |
| `email` | string | User's email address |
| `email_verified` | boolean | Whether the email has been verified |
| `role` | string | User role: "User" or "Admin" |

#### Error Responses

**401 Unauthorized**

Missing or invalid authorization token:

```json
{
  "error": "Invalid token"
}
```

**404 Not Found**

User account no longer exists:

```json
{
  "error": "User not found"
}
```

#### Example

**cURL**:
```bash
curl -X GET http://localhost:8000/api/auth/me \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

**JavaScript**:
```javascript
const token = localStorage.getItem('access_token');

const response = await fetch('/api/auth/me', {
  headers: {
    'Authorization': `Bearer ${token}`,
  },
});

if (response.ok) {
  const user = await response.json();
  console.log('Current user:', user);
} else {
  const error = await response.json();
  console.error('Error:', error.error);
}
```

**Rust (reqwest)**:
```rust
use reqwest;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserResponse {
    id: String,
    username: String,
    email: String,
    email_verified: bool,
    role: String,
}

let client = reqwest::Client::new();
let response = client
    .get("http://localhost:8000/api/auth/me")
    .bearer_auth(&access_token)
    .send()
    .await?;

if response.status().is_success() {
    let user: UserResponse = response.json().await?;
    println!("Username: {}", user.username);
}
```

---

## User Model

### User Object

The user object represents an authenticated user account in the system.

#### Fields

| Field | Type | Nullable | Description |
|-------|------|----------|-------------|
| `id` | UUID | No | Primary key, unique identifier |
| `username` | string | No | Unique username (3-50 chars) |
| `email` | string | No | Unique email address |
| `email_verified` | boolean | No | Email verification status |
| `role` | UserRole | No | "User" or "Admin" |
| `created_at` | datetime | No | Account creation timestamp |
| `updated_at` | datetime | No | Last update timestamp |
| `disabled_at` | datetime | Yes | Account disable timestamp (null if active) |
| `last_login_at` | datetime | Yes | Last successful login timestamp |

#### UserRole Enum

| Value | Description |
|-------|-------------|
| `User` | Standard user with basic permissions |
| `Admin` | Administrator with full system access |

### User Lifecycle

```
┌─────────────┐
│  Register   │
└──────┬──────┘
       │
       ▼
┌──────────────────┐
│ email_verified   │
│ = false          │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Verify Email     │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ email_verified   │
│ = true           │
│ (Active User)    │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Admin Disables   │
│ disabled_at set  │
│ (Inactive)       │
└──────────────────┘
```

### Account Status

A user account can be in one of these states:

1. **Unverified**: `email_verified = false`, `disabled_at = null`
   - Can login and use basic features
   - May have limited access to some features

2. **Active**: `email_verified = true`, `disabled_at = null`
   - Full access to all features
   - Can perform all permitted operations

3. **Disabled**: `disabled_at != null`
   - Cannot login
   - All tokens are invalid
   - Admin action required to re-enable

## Examples

### Complete User Flow

#### 1. Register and Get User Info

```javascript
// Register
const registerResponse = await fetch('/api/auth/register', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  credentials: 'include',
  body: JSON.stringify({
    username: 'alice',
    email: 'alice@example.com',
    password: 'SecurePass123!',
  }),
});

const { access_token } = await registerResponse.json();
localStorage.setItem('access_token', access_token);

// Get user info
const userResponse = await fetch('/api/auth/me', {
  headers: {
    'Authorization': `Bearer ${access_token}`,
  },
});

const user = await userResponse.json();
console.log('User:', user);
// Output: { id: "...", username: "alice", email: "alice@example.com", email_verified: false, role: "User" }
```

#### 2. Check Email Verification Status

```javascript
async function isEmailVerified() {
  const token = localStorage.getItem('access_token');

  const response = await fetch('/api/auth/me', {
    headers: {
      'Authorization': `Bearer ${token}`,
    },
  });

  if (response.ok) {
    const user = await response.json();
    return user.email_verified;
  }

  return false;
}

// Usage
if (!await isEmailVerified()) {
  // Show verification reminder
  console.log('Please verify your email');
}
```

#### 3. Display User Profile

```javascript
async function displayUserProfile() {
  const token = localStorage.getItem('access_token');

  const response = await fetch('/api/auth/me', {
    headers: {
      'Authorization': `Bearer ${token}`,
    },
  });

  if (!response.ok) {
    window.location.href = '/login';
    return;
  }

  const user = await response.json();

  // Update UI
  document.getElementById('username').textContent = user.username;
  document.getElementById('email').textContent = user.email;

  if (user.email_verified) {
    document.getElementById('verification-badge').textContent = 'Verified ✓';
  } else {
    document.getElementById('verification-badge').innerHTML =
      '<a href="#" onclick="resendVerification()">Verify Email</a>';
  }

  if (user.role === 'Admin') {
    document.getElementById('admin-panel').style.display = 'block';
  }
}
```

#### 4. Handle Token Expiration

```javascript
async function fetchCurrentUser() {
  let token = localStorage.getItem('access_token');

  let response = await fetch('/api/auth/me', {
    headers: {
      'Authorization': `Bearer ${token}`,
    },
  });

  // If unauthorized, try to refresh token
  if (response.status === 401) {
    const refreshResponse = await fetch('/api/auth/refresh', {
      method: 'POST',
      credentials: 'include',
    });

    if (refreshResponse.ok) {
      const data = await refreshResponse.json();
      token = data.access_token;
      localStorage.setItem('access_token', token);

      // Retry with new token
      response = await fetch('/api/auth/me', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });
    } else {
      // Refresh failed, redirect to login
      window.location.href = '/login';
      return null;
    }
  }

  return response.ok ? await response.json() : null;
}
```

#### 5. React Hook Example

```javascript
import { useState, useEffect } from 'react';

function useCurrentUser() {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    async function fetchUser() {
      try {
        const token = localStorage.getItem('access_token');

        const response = await fetch('/api/auth/me', {
          headers: {
            'Authorization': `Bearer ${token}`,
          },
        });

        if (response.ok) {
          const data = await response.json();
          setUser(data);
        } else {
          setError('Failed to fetch user');
        }
      } catch (err) {
        setError(err.message);
      } finally {
        setLoading(false);
      }
    }

    fetchUser();
  }, []);

  return { user, loading, error };
}

// Usage in component
function UserProfile() {
  const { user, loading, error } = useCurrentUser();

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;
  if (!user) return <div>Not logged in</div>;

  return (
    <div>
      <h1>{user.username}</h1>
      <p>{user.email}</p>
      {!user.email_verified && (
        <p>Please verify your email</p>
      )}
    </div>
  );
}
```

## Future Endpoints

The following endpoints are planned for future releases:

- `PATCH /api/users/me` - Update user profile
- `PATCH /api/users/me/password` - Change password
- `DELETE /api/users/me` - Delete account
- `GET /api/users/me/sessions` - List active sessions
- `DELETE /api/users/me/sessions/:id` - Revoke session

## Related Documentation

- [Authentication API](./authentication.md)
- [Admin API](./admin.md)
- [API Reference](./reference.md)
- [Security Guide](../guides/security.md)
