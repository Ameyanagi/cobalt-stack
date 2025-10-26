# Admin API

Administrative endpoints for user management in Cobalt Stack.

## Table of Contents

- [Overview](#overview)
- [Authentication](#authentication)
- [Endpoints](#endpoints)
  - [GET /api/admin/stats](#get-apiadminstats)
  - [GET /api/admin/users](#get-apiadminusers)
  - [GET /api/admin/users/:id](#get-apiadminusersid)
  - [PATCH /api/admin/users/:id/disable](#patch-apiadminusersiddisable)
  - [PATCH /api/admin/users/:id/enable](#patch-apiadminusersidenable)
- [Models](#models)
- [Examples](#examples)

## Overview

Admin endpoints provide administrative capabilities for user management, including:

- Viewing system statistics
- Listing and searching users
- Viewing detailed user information
- Disabling and enabling user accounts

### Access Control

All admin endpoints require:
1. Valid JWT authentication token
2. User role must be "Admin"

Non-admin users will receive a `403 Forbidden` error.

## Authentication

Admin endpoints require both authentication and authorization:

```http
Authorization: Bearer <access_token>
```

The access token must belong to a user with role "Admin".

### Example Authorization Check

```javascript
const token = localStorage.getItem('access_token');

const response = await fetch('/api/admin/stats', {
  headers: {
    'Authorization': `Bearer ${token}`,
  },
});

if (response.status === 403) {
  console.error('Admin access required');
}
```

## Endpoints

### GET /api/admin/stats

Get system-wide statistics.

**Authentication**: Required (Admin only)

#### Request

```http
GET /api/admin/stats
Authorization: Bearer <access_token>
```

#### Response

**Status**: `200 OK`

```json
{
  "total_users": 150,
  "verified_users": 120,
  "admin_users": 3,
  "disabled_users": 5
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `total_users` | integer | Total number of user accounts |
| `verified_users` | integer | Users with verified emails |
| `admin_users` | integer | Users with admin role |
| `disabled_users` | integer | Disabled user accounts |

#### Error Responses

**401 Unauthorized**
```json
{
  "error": "Invalid token"
}
```

**403 Forbidden**
```json
{
  "error": "Admin access required"
}
```

#### Example

**cURL**:
```bash
curl -X GET http://localhost:8000/api/admin/stats \
  -H "Authorization: Bearer <access_token>"
```

**JavaScript**:
```javascript
const response = await fetch('/api/admin/stats', {
  headers: {
    'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
  },
});

const stats = await response.json();
console.log(`Total users: ${stats.total_users}`);
console.log(`Verified: ${stats.verified_users} (${Math.round(stats.verified_users/stats.total_users*100)}%)`);
```

---

### GET /api/admin/users

List all users with pagination and filtering.

**Authentication**: Required (Admin only)

#### Request

```http
GET /api/admin/users?page=1&per_page=20&role=user&email_verified=true&search=alice
Authorization: Bearer <access_token>
```

#### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number (1-based) |
| `per_page` | integer | 20 | Items per page (1-100) |
| `role` | string | - | Filter by role: "admin" or "user" |
| `email_verified` | boolean | - | Filter by verification status |
| `search` | string | - | Search username or email (partial match) |

#### Response

**Status**: `200 OK`

```json
{
  "users": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "alice",
      "email": "alice@example.com",
      "role": "User",
      "email_verified": true,
      "disabled_at": null,
      "last_login_at": "2025-10-27T10:30:00Z",
      "created_at": "2025-10-20T08:00:00Z",
      "updated_at": "2025-10-27T10:30:00Z"
    }
  ],
  "total": 150,
  "page": 1,
  "per_page": 20,
  "total_pages": 8
}
```

#### Response Fields

**Top Level**:
| Field | Type | Description |
|-------|------|-------------|
| `users` | array | Array of user objects |
| `total` | integer | Total number of users matching filters |
| `page` | integer | Current page number |
| `per_page` | integer | Items per page |
| `total_pages` | integer | Total number of pages |

**User Object**:
| Field | Type | Description |
|-------|------|-------------|
| `id` | string (UUID) | Unique user identifier |
| `username` | string | User's username |
| `email` | string | User's email address |
| `role` | string | "User" or "Admin" |
| `email_verified` | boolean | Email verification status |
| `disabled_at` | datetime/null | When account was disabled (null if active) |
| `last_login_at` | datetime/null | Last successful login timestamp |
| `created_at` | datetime | Account creation timestamp |
| `updated_at` | datetime | Last update timestamp |

#### Error Responses

**400 Bad Request**
```json
{
  "error": "Invalid role parameter"
}
```

**401 Unauthorized**
```json
{
  "error": "Invalid token"
}
```

**403 Forbidden**
```json
{
  "error": "Admin access required"
}
```

#### Example

**cURL**:
```bash
# List all users
curl -X GET "http://localhost:8000/api/admin/users?page=1&per_page=50" \
  -H "Authorization: Bearer <access_token>"

# Filter by role
curl -X GET "http://localhost:8000/api/admin/users?role=admin" \
  -H "Authorization: Bearer <access_token>"

# Search users
curl -X GET "http://localhost:8000/api/admin/users?search=alice" \
  -H "Authorization: Bearer <access_token>"
```

**JavaScript**:
```javascript
// Fetch first page of users
const response = await fetch('/api/admin/users?page=1&per_page=20', {
  headers: {
    'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
  },
});

const data = await response.json();
console.log(`Showing ${data.users.length} of ${data.total} users`);

// Search for users
async function searchUsers(query) {
  const response = await fetch(`/api/admin/users?search=${encodeURIComponent(query)}`, {
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
    },
  });

  return await response.json();
}
```

---

### GET /api/admin/users/:id

Get detailed information about a specific user.

**Authentication**: Required (Admin only)

#### Request

```http
GET /api/admin/users/550e8400-e29b-41d4-a716-446655440000
Authorization: Bearer <access_token>
```

#### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | User's unique identifier |

#### Response

**Status**: `200 OK`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "email": "alice@example.com",
  "role": "User",
  "email_verified": true,
  "disabled_at": null,
  "last_login_at": "2025-10-27T10:30:00Z",
  "created_at": "2025-10-20T08:00:00Z",
  "updated_at": "2025-10-27T10:30:00Z"
}
```

#### Error Responses

**401 Unauthorized**
```json
{
  "error": "Invalid token"
}
```

**403 Forbidden**
```json
{
  "error": "Admin access required"
}
```

**404 Not Found**
```json
{
  "error": "User not found"
}
```

#### Example

**cURL**:
```bash
curl -X GET http://localhost:8000/api/admin/users/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <access_token>"
```

**JavaScript**:
```javascript
async function getUserDetails(userId) {
  const response = await fetch(`/api/admin/users/${userId}`, {
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
    },
  });

  if (response.ok) {
    return await response.json();
  }

  throw new Error('Failed to fetch user details');
}
```

---

### PATCH /api/admin/users/:id/disable

Disable a user account (soft delete).

**Authentication**: Required (Admin only)

#### Request

```http
PATCH /api/admin/users/550e8400-e29b-41d4-a716-446655440000/disable
Authorization: Bearer <access_token>
```

#### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | User's unique identifier |

#### Response

**Status**: `200 OK`

```json
{
  "message": "User disabled successfully"
}
```

#### Error Responses

**400 Bad Request**
```json
{
  "error": "User already disabled"
}
```

**401 Unauthorized**
```json
{
  "error": "Invalid token"
}
```

**403 Forbidden**
```json
{
  "error": "Admin access required"
}
```

**404 Not Found**
```json
{
  "error": "User not found"
}
```

#### Effects

When a user is disabled:
1. `disabled_at` timestamp is set to current time
2. User cannot login
3. All active sessions become invalid
4. Refresh tokens are revoked
5. User data is preserved (soft delete)

#### Example

**cURL**:
```bash
curl -X PATCH http://localhost:8000/api/admin/users/550e8400-e29b-41d4-a716-446655440000/disable \
  -H "Authorization: Bearer <access_token>"
```

**JavaScript**:
```javascript
async function disableUser(userId) {
  const response = await fetch(`/api/admin/users/${userId}/disable`, {
    method: 'PATCH',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
    },
  });

  if (response.ok) {
    const data = await response.json();
    alert(data.message);
    return true;
  }

  const error = await response.json();
  alert(`Error: ${error.error}`);
  return false;
}
```

---

### PATCH /api/admin/users/:id/enable

Enable a previously disabled user account.

**Authentication**: Required (Admin only)

#### Request

```http
PATCH /api/admin/users/550e8400-e29b-41d4-a716-446655440000/enable
Authorization: Bearer <access_token>
```

#### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | User's unique identifier |

#### Response

**Status**: `200 OK`

```json
{
  "message": "User enabled successfully"
}
```

#### Error Responses

**400 Bad Request**
```json
{
  "error": "User already enabled"
}
```

**401 Unauthorized**
```json
{
  "error": "Invalid token"
}
```

**403 Forbidden**
```json
{
  "error": "Admin access required"
}
```

**404 Not Found**
```json
{
  "error": "User not found"
}
```

#### Effects

When a user is enabled:
1. `disabled_at` is set to NULL
2. User can login again
3. User must authenticate to get new tokens

#### Example

**cURL**:
```bash
curl -X PATCH http://localhost:8000/api/admin/users/550e8400-e29b-41d4-a716-446655440000/enable \
  -H "Authorization: Bearer <access_token>"
```

**JavaScript**:
```javascript
async function enableUser(userId) {
  const response = await fetch(`/api/admin/users/${userId}/enable`, {
    method: 'PATCH',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
    },
  });

  if (response.ok) {
    const data = await response.json();
    alert(data.message);
    return true;
  }

  const error = await response.json();
  alert(`Error: ${error.error}`);
  return false;
}
```

---

## Models

### AdminUserResponse

Extended user information visible to administrators.

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "email": "alice@example.com",
  "role": "User",
  "email_verified": true,
  "disabled_at": null,
  "last_login_at": "2025-10-27T10:30:00Z",
  "created_at": "2025-10-20T08:00:00Z",
  "updated_at": "2025-10-27T10:30:00Z"
}
```

### AdminStatsResponse

System-wide statistics.

```json
{
  "total_users": 150,
  "verified_users": 120,
  "admin_users": 3,
  "disabled_users": 5
}
```

### UserListResponse

Paginated list of users.

```json
{
  "users": [/* array of AdminUserResponse */],
  "total": 150,
  "page": 1,
  "per_page": 20,
  "total_pages": 8
}
```

## Examples

### Admin Dashboard Component

```javascript
async function loadAdminDashboard() {
  const token = localStorage.getItem('access_token');

  // Fetch statistics
  const statsResponse = await fetch('/api/admin/stats', {
    headers: { 'Authorization': `Bearer ${token}` },
  });

  if (statsResponse.status === 403) {
    window.location.href = '/';
    return;
  }

  const stats = await statsResponse.json();

  // Display stats
  document.getElementById('total-users').textContent = stats.total_users;
  document.getElementById('verified-users').textContent = stats.verified_users;
  document.getElementById('admin-users').textContent = stats.admin_users;
  document.getElementById('disabled-users').textContent = stats.disabled_users;

  // Fetch recent users
  const usersResponse = await fetch('/api/admin/users?page=1&per_page=10', {
    headers: { 'Authorization': `Bearer ${token}` },
  });

  const usersData = await usersResponse.json();
  displayUsers(usersData.users);
}
```

### User Management Table

```javascript
class UserTable {
  constructor(containerId) {
    this.container = document.getElementById(containerId);
    this.currentPage = 1;
    this.perPage = 20;
    this.filters = {};
  }

  async loadUsers() {
    const token = localStorage.getItem('access_token');
    const params = new URLSearchParams({
      page: this.currentPage,
      per_page: this.perPage,
      ...this.filters,
    });

    const response = await fetch(`/api/admin/users?${params}`, {
      headers: { 'Authorization': `Bearer ${token}` },
    });

    if (!response.ok) {
      throw new Error('Failed to load users');
    }

    const data = await response.json();
    this.renderTable(data);
  }

  renderTable(data) {
    const html = `
      <table>
        <thead>
          <tr>
            <th>Username</th>
            <th>Email</th>
            <th>Role</th>
            <th>Verified</th>
            <th>Status</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          ${data.users.map(user => `
            <tr>
              <td>${user.username}</td>
              <td>${user.email}</td>
              <td>${user.role}</td>
              <td>${user.email_verified ? '✓' : '✗'}</td>
              <td>${user.disabled_at ? 'Disabled' : 'Active'}</td>
              <td>
                <button onclick="viewUser('${user.id}')">View</button>
                ${user.disabled_at
                  ? `<button onclick="enableUser('${user.id}')">Enable</button>`
                  : `<button onclick="disableUser('${user.id}')">Disable</button>`
                }
              </td>
            </tr>
          `).join('')}
        </tbody>
      </table>
      <div class="pagination">
        Page ${data.page} of ${data.total_pages}
        <button onclick="userTable.prevPage()" ${data.page === 1 ? 'disabled' : ''}>Previous</button>
        <button onclick="userTable.nextPage()" ${data.page === data.total_pages ? 'disabled' : ''}>Next</button>
      </div>
    `;

    this.container.innerHTML = html;
  }

  prevPage() {
    if (this.currentPage > 1) {
      this.currentPage--;
      this.loadUsers();
    }
  }

  nextPage() {
    this.currentPage++;
    this.loadUsers();
  }

  setFilter(key, value) {
    if (value) {
      this.filters[key] = value;
    } else {
      delete this.filters[key];
    }
    this.currentPage = 1;
    this.loadUsers();
  }
}

// Initialize
const userTable = new UserTable('user-table-container');
userTable.loadUsers();
```

### User Search

```javascript
async function searchUsers(query) {
  const token = localStorage.getItem('access_token');

  const response = await fetch(`/api/admin/users?search=${encodeURIComponent(query)}`, {
    headers: { 'Authorization': `Bearer ${token}` },
  });

  const data = await response.json();
  return data.users;
}

// Usage with debounce
let searchTimeout;
document.getElementById('search-input').addEventListener('input', (e) => {
  clearTimeout(searchTimeout);

  searchTimeout = setTimeout(async () => {
    const results = await searchUsers(e.target.value);
    displaySearchResults(results);
  }, 300);
});
```

## Related Documentation

- [Authentication API](./authentication.md)
- [User API](./users.md)
- [API Reference](./reference.md)
- [Admin Guide](../guides/admin.md)
