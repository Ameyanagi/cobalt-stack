# Admin Dashboard Guide

Complete guide to using and building admin features in Cobalt Stack.

## Table of Contents

- [Overview](#overview)
- [Admin Role Management](#admin-role-management)
- [Admin Middleware](#admin-middleware)
- [Creating the Admin User](#creating-the-admin-user)
- [Frontend Admin Routes](#frontend-admin-routes)
- [User Management Features](#user-management-features)
- [Building Custom Admin Features](#building-custom-admin-features)
- [Troubleshooting](#troubleshooting)

## Overview

Cobalt Stack includes a role-based access control (RBAC) system with two roles:

- **User**: Standard user with normal privileges
- **Admin**: Administrative user with elevated privileges

### Admin Features

- User management (view, edit, disable users)
- Role assignment
- Account status management
- Admin-only routes and components
- Protected admin endpoints

## Admin Role Management

### User Roles

The system uses an enum for roles defined in the database:

```sql
CREATE TYPE user_role AS ENUM ('user', 'admin');
```

### Database Schema

Users table includes role and status fields:

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'user',
    disabled_at TIMESTAMPTZ,
    email_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Admin Middleware

### Backend Middleware

The admin middleware verifies that the authenticated user has admin role:

```rust
// backend/src/middleware/admin.rs
use crate::middleware::auth::AuthUser;
use crate::models::{prelude::*, sea_orm_active_enums::UserRole};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::sync::Arc;

/// Admin middleware - requires admin role
/// Must be used AFTER auth_middleware
pub async fn admin_middleware(
    State(db): State<Arc<DatabaseConnection>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract AuthUser from request extensions (set by auth_middleware)
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?
        .clone();

    // Fetch user from database
    let user = Users::find_by_id(auth_user.user_id)
        .one(db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check admin role
    if user.role != UserRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check if account is disabled
    if user.disabled_at.is_some() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}
```

### Applying Admin Middleware

Apply admin middleware to protected routes:

```rust
use axum::{Router, routing::get, middleware};

// Admin routes require both auth and admin middleware
let admin_routes = Router::new()
    .route("/api/admin/users", get(list_users))
    .route("/api/admin/users/:id", get(get_user).put(update_user))
    .route("/api/admin/users/:id/disable", post(disable_user))
    .route("/api/admin/users/:id/enable", post(enable_user))
    .layer(middleware::from_fn_with_state(
        state.clone(),
        admin_middleware,
    ))
    .layer(middleware::from_fn_with_state(
        state.clone(),
        auth_middleware,
    ));
```

**Important**: Admin middleware must come AFTER auth middleware in the layer stack.

## Creating the Admin User

### Using the Seed Script

The easiest way to create an admin user is using the provided seed script:

```bash
# From project root
make seed-admin

# Or directly
cd backend
cargo run --bin seed-admin
```

The script will prompt for:
- Username
- Email
- Password

Default admin credentials (for development):
```
Username: admin
Email: admin@example.com
Password: admin123
```

### Manual Admin User Creation

To create an admin user programmatically:

```rust
use cobalt_stack::services::auth::password::hash_password;
use cobalt_stack::models::{users, sea_orm_active_enums::UserRole};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

async fn create_admin_user(
    db: &DatabaseConnection,
    username: &str,
    email: &str,
    password: &str,
) -> Result<users::Model> {
    // Hash password
    let password_hash = hash_password(password)?;

    // Create admin user
    let admin = users::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(username.to_string()),
        email: Set(email.to_string()),
        password_hash: Set(password_hash),
        role: Set(UserRole::Admin),
        email_verified: Set(true),
        disabled_at: Set(None),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    let admin = admin.insert(db).await?;
    Ok(admin)
}
```

### Promoting Existing Users

To promote an existing user to admin:

```sql
UPDATE users
SET role = 'admin'
WHERE username = 'existing_user';
```

Or programmatically:

```rust
use sea_orm::{EntityTrait, Set};

async fn promote_to_admin(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<()> {
    let user = Users::find_by_id(user_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    let mut active_user: users::ActiveModel = user.into();
    active_user.role = Set(UserRole::Admin);
    active_user.update(db).await?;

    Ok(())
}
```

## Frontend Admin Routes

### Protecting Admin Routes

Create an admin route wrapper:

```tsx
// components/auth/admin-route.tsx
'use client'

import { useAuth } from '@/contexts/auth-context'
import { useRouter } from 'next/navigation'
import { useEffect } from 'react'

export function AdminRoute({ children }: { children: React.ReactNode }) {
  const { user, isAuthenticated, isLoading } = useAuth()
  const router = useRouter()

  useEffect(() => {
    if (!isLoading) {
      if (!isAuthenticated) {
        router.push('/login')
      } else if (user?.role !== 'admin') {
        router.push('/dashboard')
      }
    }
  }, [isAuthenticated, isLoading, user, router])

  if (isLoading) {
    return <div>Loading...</div>
  }

  if (!isAuthenticated || user?.role !== 'admin') {
    return null
  }

  return <>{children}</>
}
```

### Admin Page Example

```tsx
// app/admin/page.tsx
import { AdminRoute } from '@/components/auth/admin-route'
import { AdminDashboard } from '@/components/admin/dashboard'

export default function AdminPage() {
  return (
    <AdminRoute>
      <div className="container mx-auto py-8">
        <h1 className="text-3xl font-bold mb-6">Admin Dashboard</h1>
        <AdminDashboard />
      </div>
    </AdminRoute>
  )
}
```

### Conditional UI Elements

Show/hide UI elements based on role:

```tsx
'use client'

import { useAuth } from '@/contexts/auth-context'
import Link from 'next/link'

export function Navigation() {
  const { user, isAuthenticated } = useAuth()

  return (
    <nav>
      <Link href="/">Home</Link>
      {isAuthenticated && (
        <>
          <Link href="/dashboard">Dashboard</Link>
          {user?.role === 'admin' && (
            <Link href="/admin">Admin Panel</Link>
          )}
        </>
      )}
    </nav>
  )
}
```

## User Management Features

### List Users Endpoint

```rust
#[derive(Serialize)]
struct UserListResponse {
    users: Vec<UserSummary>,
    total: u64,
}

#[derive(Serialize)]
struct UserSummary {
    id: Uuid,
    username: String,
    email: String,
    role: UserRole,
    email_verified: bool,
    disabled: bool,
    created_at: DateTime<Utc>,
}

async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<UserListResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    // Get users with pagination
    let users = Users::find()
        .paginate(&state.db, per_page)
        .fetch_page(page - 1)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get total count
    let total = Users::find()
        .count(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Map to response format
    let user_summaries = users
        .into_iter()
        .map(|u| UserSummary {
            id: u.id,
            username: u.username,
            email: u.email,
            role: u.role,
            email_verified: u.email_verified,
            disabled: u.disabled_at.is_some(),
            created_at: u.created_at.into(),
        })
        .collect();

    Ok(Json(UserListResponse {
        users: user_summaries,
        total,
    }))
}
```

### Disable User Endpoint

```rust
async fn disable_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Get user
    let user = Users::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Don't allow disabling if already disabled
    if user.disabled_at.is_some() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Update user
    let mut active_user: users::ActiveModel = user.into();
    active_user.disabled_at = Set(Some(Utc::now().into()));
    active_user.update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
```

### Enable User Endpoint

```rust
async fn enable_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user = Users::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active_user: users::ActiveModel = user.into();
    active_user.disabled_at = Set(None);
    active_user.update(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
```

### User Management Component

```tsx
'use client'

import { useState, useEffect } from 'react'
import { useAuth } from '@/contexts/auth-context'
import { env } from '@/lib/env'

interface User {
  id: string
  username: string
  email: string
  role: 'admin' | 'user'
  email_verified: boolean
  disabled: boolean
  created_at: string
}

export function UserManagement() {
  const [users, setUsers] = useState<User[]>([])
  const [loading, setLoading] = useState(true)
  const { accessToken } = useAuth()

  useEffect(() => {
    fetchUsers()
  }, [])

  const fetchUsers = async () => {
    try {
      const response = await fetch(`${env.apiUrl}/api/admin/users`, {
        headers: {
          'Authorization': `Bearer ${accessToken}`,
        },
      })

      if (response.ok) {
        const data = await response.json()
        setUsers(data.users)
      }
    } catch (error) {
      console.error('Failed to fetch users:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleDisableUser = async (userId: string) => {
    try {
      const response = await fetch(
        `${env.apiUrl}/api/admin/users/${userId}/disable`,
        {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${accessToken}`,
          },
        }
      )

      if (response.ok) {
        fetchUsers() // Refresh list
      }
    } catch (error) {
      console.error('Failed to disable user:', error)
    }
  }

  const handleEnableUser = async (userId: string) => {
    try {
      const response = await fetch(
        `${env.apiUrl}/api/admin/users/${userId}/enable`,
        {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${accessToken}`,
          },
        }
      )

      if (response.ok) {
        fetchUsers() // Refresh list
      }
    } catch (error) {
      console.error('Failed to enable user:', error)
    }
  }

  if (loading) {
    return <div>Loading users...</div>
  }

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold">User Management</h2>

      <div className="overflow-x-auto">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Username
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Email
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Role
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Status
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {users.map((user) => (
              <tr key={user.id}>
                <td className="px-6 py-4 whitespace-nowrap">
                  {user.username}
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  {user.email}
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <span
                    className={`px-2 py-1 text-xs rounded ${
                      user.role === 'admin'
                        ? 'bg-purple-100 text-purple-800'
                        : 'bg-gray-100 text-gray-800'
                    }`}
                  >
                    {user.role}
                  </span>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <span
                    className={`px-2 py-1 text-xs rounded ${
                      user.disabled
                        ? 'bg-red-100 text-red-800'
                        : 'bg-green-100 text-green-800'
                    }`}
                  >
                    {user.disabled ? 'Disabled' : 'Active'}
                  </span>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  {user.disabled ? (
                    <button
                      onClick={() => handleEnableUser(user.id)}
                      className="text-green-600 hover:text-green-900"
                    >
                      Enable
                    </button>
                  ) : (
                    <button
                      onClick={() => handleDisableUser(user.id)}
                      className="text-red-600 hover:text-red-900"
                    >
                      Disable
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
```

## Building Custom Admin Features

### Admin Statistics Dashboard

```tsx
'use client'

import { useState, useEffect } from 'react'
import { useAuth } from '@/contexts/auth-context'
import { env } from '@/lib/env'

interface Stats {
  total_users: number
  active_users: number
  verified_users: number
  admin_users: number
}

export function AdminStats() {
  const [stats, setStats] = useState<Stats | null>(null)
  const { accessToken } = useAuth()

  useEffect(() => {
    fetchStats()
  }, [])

  const fetchStats = async () => {
    try {
      const response = await fetch(`${env.apiUrl}/api/admin/stats`, {
        headers: {
          'Authorization': `Bearer ${accessToken}`,
        },
      })

      if (response.ok) {
        const data = await response.json()
        setStats(data)
      }
    } catch (error) {
      console.error('Failed to fetch stats:', error)
    }
  }

  if (!stats) {
    return <div>Loading stats...</div>
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
      <StatCard
        title="Total Users"
        value={stats.total_users}
        icon="👥"
      />
      <StatCard
        title="Active Users"
        value={stats.active_users}
        icon="✅"
      />
      <StatCard
        title="Verified Users"
        value={stats.verified_users}
        icon="📧"
      />
      <StatCard
        title="Admin Users"
        value={stats.admin_users}
        icon="👑"
      />
    </div>
  )
}

function StatCard({ title, value, icon }: { title: string; value: number; icon: string }) {
  return (
    <div className="bg-white p-6 rounded-lg shadow">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-gray-600">{title}</p>
          <p className="text-2xl font-bold mt-1">{value}</p>
        </div>
        <div className="text-4xl">{icon}</div>
      </div>
    </div>
  )
}
```

### Backend Stats Endpoint

```rust
#[derive(Serialize)]
struct AdminStats {
    total_users: u64,
    active_users: u64,
    verified_users: u64,
    admin_users: u64,
}

async fn get_admin_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AdminStats>, StatusCode> {
    let total_users = Users::find()
        .count(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_users = Users::find()
        .filter(users::Column::DisabledAt.is_null())
        .count(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let verified_users = Users::find()
        .filter(users::Column::EmailVerified.eq(true))
        .count(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let admin_users = Users::find()
        .filter(users::Column::Role.eq(UserRole::Admin))
        .count(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AdminStats {
        total_users,
        active_users,
        verified_users,
        admin_users,
    }))
}
```

### Activity Log

For audit purposes, consider adding an activity log:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub admin_id: Uuid,
    pub action: String,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
}

async fn log_admin_action(
    db: &DatabaseConnection,
    admin_id: Uuid,
    user_id: Uuid,
    action: &str,
    details: Option<&str>,
) -> Result<()> {
    let log = activity_logs::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        admin_id: Set(admin_id),
        action: Set(action.to_string()),
        details: Set(details.map(|s| s.to_string())),
        created_at: Set(Utc::now().into()),
    };

    log.insert(db).await?;
    Ok(())
}
```

## Troubleshooting

### 403 Forbidden on Admin Routes

**Problem**: Admin user getting 403 errors

**Solutions**:
1. Verify user role is actually 'admin' in database
2. Check admin middleware is applied correctly
3. Ensure auth middleware runs before admin middleware
4. Check if user account is disabled
5. Verify token contains correct user_id

### Cannot Create Admin User

**Problem**: Seed script fails or admin creation errors

**Solutions**:
1. Check database connection is working
2. Verify migrations have been run
3. Ensure user_role enum exists in database
4. Check for unique constraint violations (username/email)
5. Review password hashing is working

### Admin Panel Not Loading

**Problem**: Admin pages show loading or redirect incorrectly

**Solutions**:
1. Check user role in auth context
2. Verify AdminRoute wrapper is applied
3. Check browser console for errors
4. Ensure API endpoints return correct role
5. Test authentication is working

### Middleware Order Issues

**Problem**: Admin middleware not working correctly

**Solutions**:
1. Verify middleware order: auth → admin
2. Check AuthUser is in request extensions
3. Ensure state is passed correctly
4. Review middleware function signatures
5. Test with simple logging middleware

### Database Role Mismatch

**Problem**: Role appears different in frontend vs database

**Solutions**:
1. Check database enum matches code enum
2. Verify API response serialization
3. Test direct database query
4. Check for caching issues
5. Restart backend after role changes

## Best Practices

1. **Never trust client-side role checks**: Always verify on backend
2. **Log admin actions**: Maintain audit trail of admin activities
3. **Rate limit admin endpoints**: Prevent abuse even by admins
4. **Separate admin UI**: Keep admin interface distinct from user interface
5. **Implement 2FA for admins**: Add extra security for admin accounts
6. **Regular admin audits**: Review admin accounts periodically
7. **Principle of least privilege**: Grant admin only when necessary
8. **Disable not delete**: Disable accounts instead of deleting
9. **Monitor admin activity**: Alert on suspicious admin actions
10. **Secure admin credentials**: Use strong passwords and secure storage

## Related Documentation

- [Authentication Guide](./authentication.md) - User authentication system
- [API Client Guide](./api-client.md) - Making authenticated API calls
- [Database Guide](./database.md) - Database setup and migrations
- [API Reference](../api/README.md) - Complete API documentation
- [Backend Middleware](../backend/README.md#middleware) - Middleware documentation
