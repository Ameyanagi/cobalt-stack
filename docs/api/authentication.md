# Authentication API

Complete authentication API reference for Cobalt Stack.

## Table of Contents

- [Overview](#overview)
- [Endpoints](#endpoints)
  - [POST /api/auth/register](#post-apiauthregister)
  - [POST /api/auth/login](#post-apiauthlogin)
  - [POST /api/auth/refresh](#post-apiauthrefresh)
  - [POST /api/auth/logout](#post-apiauthlogout)
  - [GET /api/auth/me](#get-apiauthme)
  - [POST /api/auth/verify-email](#post-apiauthverify-email)
  - [POST /api/auth/send-verification](#post-apiauthsend-verification)
- [Security Features](#security-features)
- [Best Practices](#best-practices)

## Overview

The authentication system uses JWT (JSON Web Tokens) with a dual-token approach:

- **Access Token**: Short-lived (15 minutes), sent in response body
- **Refresh Token**: Long-lived (7 days), stored in HTTP-only cookie

### Authentication Flow

```
┌─────────────┐
│  Register   │
│   /Login    │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────┐
│ Access Token (15min)        │
│ Refresh Token (7d, cookie)  │
└──────┬──────────────────────┘
       │
       ▼
┌─────────────┐
│  Use API    │◄──── Authorization: Bearer <token>
└──────┬──────┘
       │
       ▼ (token expires)
┌─────────────┐
│  /refresh   │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────┐
│ New Access Token (15min)    │
│ New Refresh Token (7d)      │
└─────────────────────────────┘
```

## Endpoints

### POST /api/auth/register

Register a new user account.

#### Request

```http
POST /api/auth/register
Content-Type: application/json

{
  "username": "alice",
  "email": "alice@example.com",
  "password": "SecurePass123!"
}
```

#### Request Body

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `username` | string | Yes | 3-50 characters |
| `email` | string | Yes | Valid email format |
| `password` | string | Yes | 8-128 characters |

#### Response

**Status**: `200 OK`

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**Headers**:
```http
Set-Cookie: refresh_token=eyJ...; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=604800
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `access_token` | string | JWT access token for API authentication |
| `token_type` | string | Always "Bearer" |
| `expires_in` | integer | Token expiration in seconds (900 = 15 minutes) |

#### Error Responses

**400 Bad Request**
```json
{
  "error": "Invalid input: Username must be between 3 and 50 characters"
}
```

**409 Conflict**
```json
{
  "error": "User already exists"
}
```

#### Notes

- Email verification is sent automatically after registration
- User can login immediately but may have limited access until email is verified
- Refresh token is stored in HTTP-only cookie (not in response body)

#### Example

**cURL**:
```bash
curl -X POST http://localhost:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "username": "alice",
    "email": "alice@example.com",
    "password": "SecurePass123!"
  }'
```

**JavaScript**:
```javascript
const response = await fetch('/api/auth/register', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  credentials: 'include',
  body: JSON.stringify({
    username: 'alice',
    email: 'alice@example.com',
    password: 'SecurePass123!',
  }),
});

const data = await response.json();
localStorage.setItem('access_token', data.access_token);
```

---

### POST /api/auth/login

Authenticate existing user.

#### Request

```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "alice",
  "password": "SecurePass123!"
}
```

#### Request Body

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `username` | string | Yes | Non-empty |
| `password` | string | Yes | Non-empty |

#### Response

**Status**: `200 OK`

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**Headers**:
```http
Set-Cookie: refresh_token=eyJ...; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=604800
```

#### Error Responses

**400 Bad Request**
```json
{
  "error": "Invalid input: Username cannot be empty"
}
```

**401 Unauthorized**
```json
{
  "error": "Invalid credentials"
}
```

**429 Too Many Requests**
```json
{
  "error": "Too many login attempts"
}
```

#### Rate Limiting

- **Limit**: 5 attempts per 15 minutes per IP address
- **Scope**: Per IP address
- **Reset**: 15 minutes after first attempt

#### Example

**cURL**:
```bash
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "username": "alice",
    "password": "SecurePass123!"
  }'
```

**JavaScript**:
```javascript
const response = await fetch('/api/auth/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  credentials: 'include',
  body: JSON.stringify({
    username: 'alice',
    password: 'SecurePass123!',
  }),
});

if (response.ok) {
  const data = await response.json();
  localStorage.setItem('access_token', data.access_token);
}
```

---

### POST /api/auth/refresh

Refresh access token using refresh token from cookie.

#### Request

```http
POST /api/auth/refresh
Cookie: refresh_token=eyJ...
```

#### Response

**Status**: `200 OK`

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**Headers**:
```http
Set-Cookie: refresh_token=eyJ...; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=604800
```

#### Error Responses

**401 Unauthorized**
```json
{
  "error": "Invalid token"
}
```

#### Token Rotation

This endpoint implements automatic token rotation:

1. Validates the old refresh token
2. Revokes the old refresh token
3. Issues a new refresh token
4. Issues a new access token

This ensures that:
- Stolen refresh tokens become invalid after first use
- Each refresh operation extends the session by 7 days
- Token reuse is detected and blocked

#### Example

**cURL**:
```bash
curl -X POST http://localhost:8000/api/auth/refresh \
  -b cookies.txt \
  -c cookies.txt
```

**JavaScript**:
```javascript
const response = await fetch('/api/auth/refresh', {
  method: 'POST',
  credentials: 'include',
});

if (response.ok) {
  const data = await response.json();
  localStorage.setItem('access_token', data.access_token);
} else {
  // Refresh failed, redirect to login
  window.location.href = '/login';
}
```

---

### POST /api/auth/logout

Logout and invalidate tokens.

#### Request

```http
POST /api/auth/logout
Cookie: refresh_token=eyJ...
```

#### Response

**Status**: `200 OK`

```http
Set-Cookie: refresh_token=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0
```

#### Error Responses

**401 Unauthorized**
```json
{
  "error": "Invalid token"
}
```

#### Token Revocation

This endpoint:
1. Revokes the refresh token in the database
2. Clears the refresh token cookie
3. Client should discard the access token

Note: Access tokens cannot be revoked until expiry (15 minutes max).

#### Example

**cURL**:
```bash
curl -X POST http://localhost:8000/api/auth/logout \
  -b cookies.txt
```

**JavaScript**:
```javascript
await fetch('/api/auth/logout', {
  method: 'POST',
  credentials: 'include',
});

localStorage.removeItem('access_token');
window.location.href = '/login';
```

---

### GET /api/auth/me

Get current authenticated user information.

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
| `username` | string | User's username |
| `email` | string | User's email address |
| `email_verified` | boolean | Whether email is verified |
| `role` | string | User role: "User" or "Admin" |

#### Error Responses

**401 Unauthorized**
```json
{
  "error": "Invalid token"
}
```

#### Example

**cURL**:
```bash
curl -X GET http://localhost:8000/api/auth/me \
  -H "Authorization: Bearer <access_token>"
```

**JavaScript**:
```javascript
const token = localStorage.getItem('access_token');

const response = await fetch('/api/auth/me', {
  headers: {
    'Authorization': `Bearer ${token}`,
  },
});

const user = await response.json();
console.log('Current user:', user);
```

---

### POST /api/auth/verify-email

Verify email address with token from email.

#### Request

```http
POST /api/auth/verify-email
Content-Type: application/json

{
  "token": "abc123def456"
}
```

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `token` | string | Yes | Verification token from email |

#### Response

**Status**: `200 OK`

```json
{
  "message": "Email verified successfully"
}
```

#### Error Responses

**400 Bad Request**
```json
{
  "error": "Verification failed: Invalid or expired token"
}
```

#### Notes

- Tokens expire after 24 hours
- Tokens are single-use only
- Already verified emails return 400

#### Example

**cURL**:
```bash
curl -X POST http://localhost:8000/api/auth/verify-email \
  -H "Content-Type: application/json" \
  -d '{
    "token": "abc123def456"
  }'
```

**JavaScript**:
```javascript
const response = await fetch('/api/auth/verify-email', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    token: 'abc123def456',
  }),
});

const data = await response.json();
if (response.ok) {
  alert(data.message);
}
```

---

### POST /api/auth/send-verification

Resend verification email.

**Requires authentication**.

#### Request

```http
POST /api/auth/send-verification
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### Response

**Status**: `200 OK`

```json
{
  "message": "Verification email sent"
}
```

#### Error Responses

**400 Bad Request**
```json
{
  "error": "Invalid input: Email already verified"
}
```

**401 Unauthorized**
```json
{
  "error": "Invalid token"
}
```

#### Notes

- Only works for unverified emails
- Rate limited to prevent spam
- Old verification tokens remain valid until expiry

#### Example

**cURL**:
```bash
curl -X POST http://localhost:8000/api/auth/send-verification \
  -H "Authorization: Bearer <access_token>"
```

**JavaScript**:
```javascript
const token = localStorage.getItem('access_token');

const response = await fetch('/api/auth/send-verification', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
  },
});

const data = await response.json();
alert(data.message);
```

---

## Security Features

### JWT Tokens

- **Algorithm**: HS256 (HMAC-SHA256)
- **Access Token Expiry**: 15 minutes
- **Refresh Token Expiry**: 7 days
- **Token Rotation**: Automatic on refresh

### Password Security

- **Hashing**: Argon2id
- **Minimum Length**: 8 characters
- **Maximum Length**: 128 characters
- **Validation**: Server-side only

### Cookie Security

Refresh tokens use secure cookies:

- **HttpOnly**: JavaScript cannot access
- **Secure**: HTTPS only (production)
- **SameSite=Strict**: CSRF protection
- **Path=/**: Available to all routes

### Rate Limiting

- **Login**: 5 attempts per 15 minutes per IP
- **Register**: 3 attempts per hour per IP

### Token Revocation

- Refresh tokens stored in database
- Can be revoked via logout
- Automatic cleanup of expired tokens

## Best Practices

### Client Implementation

1. **Store Access Token Securely**
   - Use memory or sessionStorage (not localStorage for sensitive apps)
   - Clear on logout or page close

2. **Handle Token Refresh**
   ```javascript
   async function refreshAccessToken() {
     const response = await fetch('/api/auth/refresh', {
       method: 'POST',
       credentials: 'include',
     });

     if (response.ok) {
       const data = await response.json();
       return data.access_token;
     }

     throw new Error('Refresh failed');
   }
   ```

3. **Implement Retry Logic**
   ```javascript
   async function apiCall(url, options) {
     let response = await fetch(url, options);

     if (response.status === 401) {
       const newToken = await refreshAccessToken();
       options.headers.Authorization = `Bearer ${newToken}`;
       response = await fetch(url, options);
     }

     return response;
   }
   ```

4. **Handle Errors Gracefully**
   - Show user-friendly error messages
   - Redirect to login on authentication failure
   - Log errors for debugging

### Security Recommendations

1. **Use HTTPS in Production**
   - Protects tokens in transit
   - Required for secure cookies

2. **Implement CORS Properly**
   - Whitelist specific origins
   - Include credentials in requests

3. **Monitor for Suspicious Activity**
   - Track failed login attempts
   - Alert on unusual patterns
   - Log token usage

4. **Regular Token Rotation**
   - Access tokens expire in 15 minutes
   - Refresh tokens rotate on use
   - Old tokens are revoked

## Related Documentation

- [API Reference](./reference.md)
- [User Endpoints](./users.md)
- [Admin Endpoints](./admin.md)
- [Security Guide](../guides/security.md)
