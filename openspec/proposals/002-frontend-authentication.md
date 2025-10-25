# Proposal: Frontend Authentication Implementation

**Status**: Proposed
**Created**: 2025-01-26
**Author**: AI Assistant
**Related**: Backend auth system (feature/auth-system merged to master)

## Summary

Implement complete authentication flow in Next.js 16 frontend with JWT token management, protected routes, and integration with backend auth API endpoints.

## Context

The backend authentication system has core functionality with:
- ✅ User registration (`POST /api/auth/register`)
- ✅ User login (`POST /api/auth/login`)
- ✅ Protected routes with JWT middleware (`GET /api/auth/me`)
- ✅ JWT dual-token pattern (access + refresh tokens generated)
- ✅ Comprehensive test coverage (59 tests passing)

**Backend Gaps Requiring Completion:**
- ⚠️ Refresh token only stored in database (not returned to client)
- ⚠️ `/api/auth/refresh` endpoint returns `NOT_IMPLEMENTED`
- ⚠️ `/api/auth/logout` endpoint returns `NOT_IMPLEMENTED`
- ⚠️ No HttpOnly cookie support for refresh tokens

**This proposal follows a backend-first approach:** Complete backend cookie support and token refresh/logout endpoints using TDD, then implement frontend integration.

## Goals

1. **Complete Auth Flow**: Registration, login, logout, session management
2. **Token Management**: Secure storage and automatic refresh of JWT tokens
3. **Protected Routes**: Client-side route protection and redirects
4. **User Context**: Global auth state management with React Context
5. **Auth UI Components**: Login form, registration form, user profile display
6. **Type Safety**: Full TypeScript integration with backend API types
7. **Error Handling**: User-friendly error messages and validation
8. **Testing**: Unit tests for auth hooks and integration tests for flows

## Non-Goals

- OAuth provider integration (Google, EntraID, GitHub) - future work
- Email verification flow - future work
- Password reset flow - future work
- Remember me / persistent login - future work
- Multi-factor authentication (MFA) - future work

## Architecture

### 1. Token Management Strategy

**Storage Approach**:
- **Access Token**: Memory-only (React state/context) - never in localStorage
- **Refresh Token**: HttpOnly cookie (backend-managed) - secure, prevents XSS

**Token Refresh Strategy**:
- Automatic refresh on 401 responses
- Proactive refresh before expiry (if access token includes exp claim)
- Retry failed requests after refresh

**Security Benefits**:
- XSS-resistant (access token in memory, refresh in HttpOnly cookie)
- CSRF-protected refresh endpoint
- No sensitive data in localStorage/sessionStorage

### 2. Directory Structure

```
frontend/src/
├── app/
│   ├── (auth)/              # Auth-related routes
│   │   ├── login/
│   │   │   └── page.tsx     # Login page
│   │   ├── register/
│   │   │   └── page.tsx     # Register page
│   │   └── layout.tsx       # Auth layout (no auth required)
│   ├── (protected)/         # Protected routes
│   │   ├── dashboard/
│   │   │   └── page.tsx
│   │   ├── profile/
│   │   │   └── page.tsx
│   │   └── layout.tsx       # Protected layout (requires auth)
│   └── layout.tsx           # Root layout with AuthProvider
├── components/
│   ├── auth/
│   │   ├── LoginForm.tsx
│   │   ├── RegisterForm.tsx
│   │   ├── LogoutButton.tsx
│   │   └── UserMenu.tsx
│   └── guards/
│       ├── AuthGuard.tsx    # Protect routes/components
│       └── GuestGuard.tsx   # Redirect if authenticated
├── contexts/
│   └── AuthContext.tsx      # Auth state and methods
├── hooks/
│   ├── useAuth.ts          # Access auth context
│   ├── useUser.ts          # Access current user
│   └── useProtectedRoute.ts # Route protection hook
├── lib/
│   ├── api/
│   │   ├── auth.ts         # Auth API client methods
│   │   └── client.ts       # Enhanced API client with token refresh
│   └── validation/
│       └── auth.ts         # Zod schemas for auth forms
└── types/
    └── auth.ts             # Auth-related types
```

### 3. Core Components

#### AuthContext (`contexts/AuthContext.tsx`)

```typescript
interface AuthContextValue {
  // State
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;

  // Actions
  login: (username: string, password: string) => Promise<void>;
  register: (data: RegisterRequest) => Promise<void>;
  logout: () => Promise<void>;
  refreshUser: () => Promise<void>;
}
```

**Responsibilities**:
- Manage access token in memory
- Store user data in context state
- Provide authentication methods
- Handle token refresh automatically
- Initialize auth state on mount

#### API Client (`lib/api/client.ts`)

**Enhanced Features**:
- Automatic `Authorization` header injection
- 401 response interceptor
- Token refresh logic with retry
- Request queuing during refresh

**Token Refresh Flow**:
```typescript
1. Request fails with 401
2. Attempt token refresh (POST /api/auth/refresh)
3. On success: Update access token, retry original request
4. On failure: Clear auth state, redirect to login
```

#### Auth Guard (`components/guards/AuthGuard.tsx`)

```typescript
interface AuthGuardProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  redirect?: string; // Default: '/login'
}
```

**Behavior**:
- Show loading state while checking auth
- Redirect to login if not authenticated
- Render children if authenticated

### 4. API Integration

#### Auth API Methods (`lib/api/auth.ts`)

```typescript
export const authApi = {
  // Register new user
  register: async (data: RegisterRequest): Promise<AuthResponse> => {
    return apiClient.post('/api/auth/register', data);
  },

  // Login with credentials
  login: async (username: string, password: string): Promise<AuthResponse> => {
    return apiClient.post('/api/auth/login', { username, password });
  },

  // Get current user (requires auth)
  me: async (): Promise<UserResponse> => {
    return apiClient.get('/api/auth/me');
  },

  // Refresh access token (uses HttpOnly cookie)
  refresh: async (): Promise<AuthResponse> => {
    return apiClient.post('/api/auth/refresh', {}, {
      credentials: 'include' // Send cookies
    });
  },

  // Logout (revoke tokens)
  logout: async (): Promise<void> => {
    return apiClient.post('/api/auth/logout');
  },
};
```

### 5. Form Validation

**Use Zod for runtime validation**:

```typescript
// lib/validation/auth.ts
import { z } from 'zod';

export const loginSchema = z.object({
  username: z.string().min(1, 'Username is required'),
  password: z.string().min(1, 'Password is required'),
});

export const registerSchema = z.object({
  username: z.string()
    .min(3, 'Username must be at least 3 characters')
    .max(50, 'Username must not exceed 50 characters'),
  email: z.string().email('Invalid email format'),
  password: z.string()
    .min(8, 'Password must be at least 8 characters')
    .max(128, 'Password must not exceed 128 characters'),
});
```

**Integration with React Hook Form**:
- Use `@hookform/resolvers/zod` for schema validation
- Provide real-time field validation
- Display user-friendly error messages

## Implementation Plan

### Phase 0: Backend Cookie & Token Management (TDD) - **PREREQUISITE**

**Branch**: Continue on `feature/frontend-auth` (includes backend changes)

**Objective**: Complete backend refresh token and logout functionality with HttpOnly cookie support following TDD methodology.

#### 0.1: Refresh Token Cookie Support

**Write Tests First:**
```rust
// backend/src/handlers/auth.rs - Add to tests module

#[tokio::test]
async fn test_login_sets_refresh_token_cookie() {
    // Test that login response includes Set-Cookie header
    // Verify cookie attributes: HttpOnly, Secure, SameSite=Strict
}

#[tokio::test]
async fn test_register_sets_refresh_token_cookie() {
    // Test that register response includes Set-Cookie header
}
```

**Implementation:**
- [ ] Add `axum-extra` dependency for cookie support
- [ ] Update `AuthResponse` to NOT include refresh_token in JSON
- [ ] Modify `register` handler to set HttpOnly cookie with refresh token
- [ ] Modify `login` handler to set HttpOnly cookie with refresh token
- [ ] Configure cookie attributes: `http_only=true`, `secure=true`, `same_site=Strict`, `path=/`, `max_age=7 days`
- [ ] Update OpenAPI documentation to reflect cookie usage
- [ ] Run tests: `cargo test --lib handlers::auth` - Should pass

#### 0.2: Token Refresh Endpoint Implementation

**Write Tests First:**
```rust
#[tokio::test]
async fn test_refresh_token_with_valid_cookie() {
    // Mock valid refresh token in cookie
    // Call /api/auth/refresh
    // Assert: Returns new access token
    // Assert: Returns new refresh token cookie (rotation)
}

#[tokio::test]
async fn test_refresh_token_without_cookie() {
    // Call /api/auth/refresh without cookie
    // Assert: Returns 401 Unauthorized
}

#[tokio::test]
async fn test_refresh_token_with_invalid_cookie() {
    // Mock invalid/expired refresh token
    // Call /api/auth/refresh
    // Assert: Returns 401 Unauthorized
}

#[tokio::test]
async fn test_refresh_token_with_revoked_token() {
    // Mock revoked refresh token (not in database)
    // Call /api/auth/refresh
    // Assert: Returns 401 Unauthorized
}
```

**Implementation:**
- [ ] Extract refresh token from `Cookie` header using `axum-extra::extract::CookieJar`
- [ ] Validate refresh token JWT signature and expiry
- [ ] Query database to verify refresh token exists and isn't revoked
- [ ] Generate new access token
- [ ] Rotate refresh token (revoke old, create new) for security
- [ ] Return new access token in JSON
- [ ] Set new refresh token as HttpOnly cookie
- [ ] Run tests: `cargo test --lib handlers::auth::test_refresh` - Should pass

#### 0.3: Logout Endpoint Implementation

**Write Tests First:**
```rust
#[tokio::test]
async fn test_logout_revokes_refresh_token() {
    // Mock user with active refresh token cookie
    // Call /api/auth/logout
    // Assert: Refresh token revoked in database
    // Assert: Cookie cleared (Max-Age=0)
}

#[tokio::test]
async fn test_logout_without_cookie() {
    // Call /api/auth/logout without cookie
    // Assert: Returns 200 OK (idempotent)
}

#[tokio::test]
async fn test_logout_requires_authentication() {
    // Call /api/auth/logout without access token
    // Assert: Middleware returns 401 Unauthorized
}
```

**Implementation:**
- [ ] Apply auth middleware to `/api/auth/logout` route (already done)
- [ ] Extract refresh token from cookie
- [ ] Revoke refresh token in database (set `revoked_at` timestamp)
- [ ] Clear refresh token cookie (set `Max-Age=0`)
- [ ] Return 200 OK
- [ ] Run tests: `cargo test --lib handlers::auth::test_logout` - Should pass

#### 0.4: Integration & Cleanup

**Deliverables:**
- [ ] Update `Cargo.toml` with `axum-extra = { version = "0.9", features = ["cookie"] }`
- [ ] Run full test suite: `cargo test --lib` - All tests should pass
- [ ] Manual testing with curl:
  ```bash
  # Login and capture cookie
  curl -v -X POST http://localhost:3000/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username":"test","password":"test123"}' \
    -c cookies.txt

  # Refresh token using cookie
  curl -v -X POST http://localhost:3000/api/auth/refresh \
    -b cookies.txt \
    -c cookies.txt

  # Logout
  curl -v -X POST http://localhost:3000/api/auth/logout \
    -H "Authorization: Bearer <access_token>" \
    -b cookies.txt
  ```
- [ ] Commit Phase 0: "feat(auth): Implement refresh token cookies and logout with TDD"

**Estimated Time**: 1-2 days

---

### Phase 1: Core Auth Infrastructure (Foundation)

**Deliverables**:
- [ ] TypeScript types for auth (`types/auth.ts`)
- [ ] Auth API client methods (`lib/api/auth.ts`)
- [ ] Enhanced API client with token refresh (`lib/api/client.ts`)
- [ ] Zod validation schemas (`lib/validation/auth.ts`)

**Testing**:
- Unit tests for API client token refresh logic
- Unit tests for validation schemas

### Phase 2: Auth Context & State Management

**Deliverables**:
- [ ] AuthContext with provider (`contexts/AuthContext.tsx`)
- [ ] useAuth hook (`hooks/useAuth.ts`)
- [ ] useUser hook (`hooks/useUser.ts`)
- [ ] Initialize auth state on app load
- [ ] Auto-refresh token logic

**Testing**:
- Unit tests for AuthContext actions
- Test token refresh scenarios
- Test logout clearing state

### Phase 3: Route Protection & Guards

**Deliverables**:
- [ ] AuthGuard component (`components/guards/AuthGuard.tsx`)
- [ ] GuestGuard component (`components/guards/GuestGuard.tsx`)
- [ ] useProtectedRoute hook (`hooks/useProtectedRoute.ts`)
- [ ] Protected route layout (`app/(protected)/layout.tsx`)
- [ ] Public route layout (`app/(auth)/layout.tsx`)

**Testing**:
- Test redirect behavior
- Test loading states
- Test authenticated/unauthenticated scenarios

### Phase 4: Auth UI Components

**Deliverables**:
- [ ] LoginForm component with validation (`components/auth/LoginForm.tsx`)
- [ ] RegisterForm component with validation (`components/auth/RegisterForm.tsx`)
- [ ] LogoutButton component (`components/auth/LogoutButton.tsx`)
- [ ] UserMenu component (`components/auth/UserMenu.tsx`)
- [ ] Login page (`app/(auth)/login/page.tsx`)
- [ ] Register page (`app/(auth)/register/page.tsx`)

**Design**:
- Use shadcn/ui components (already installed)
- Consistent error message display
- Loading states for async operations
- Accessible form controls

**Testing**:
- Component tests for forms
- Test validation error display
- Test submission handling

### Phase 5: Protected Pages & Features

**Deliverables**:
- [ ] Dashboard page (`app/(protected)/dashboard/page.tsx`)
- [ ] Profile page (`app/(protected)/profile/page.tsx`)
- [ ] Update navigation with auth-aware links
- [ ] Display user info in header

**Testing**:
- Integration tests for complete auth flows
- Test navigation between protected routes
- Test logout from protected pages

### Phase 6: Error Handling & Polish

**Deliverables**:
- [ ] Centralized error handling
- [ ] User-friendly error messages
- [ ] Toast notifications for auth events
- [ ] Handle network errors gracefully
- [ ] Loading skeletons for auth checks

**Testing**:
- Test error scenarios (network failures, 401s, etc.)
- Test user feedback for all auth operations

### Phase 7: Documentation & Finalization

**Deliverables**:
- [ ] Frontend authentication documentation
- [ ] Usage examples for developers
- [ ] Environment variable documentation
- [ ] Deployment notes for auth

## Technical Decisions

### 1. Token Storage: Memory + HttpOnly Cookies

**Decision**: Store access tokens in memory (React state) and refresh tokens in HttpOnly cookies.

**Rationale**:
- **Security**: Protects against XSS attacks (most common web vulnerability)
- **XSS Protection**: Access tokens in memory can't be stolen by malicious scripts
- **CSRF Protection**: Refresh endpoint validates origin and uses SameSite cookies
- **Best Practice**: Recommended by OWASP and auth0

**Trade-offs**:
- Access tokens lost on page refresh (acceptable - refresh token handles this)
- Requires backend cookie support (already configured with CORS credentials)

### 2. React Context vs State Management Library

**Decision**: Use React Context for auth state.

**Rationale**:
- **Simplicity**: Auth state is relatively simple and doesn't require heavy state management
- **Built-in**: No additional dependencies
- **Performance**: Auth state changes infrequently
- **Future**: Can migrate to Zustand/Redux if needed

**Trade-offs**:
- Not ideal for frequently-changing state (not an issue for auth)
- Context re-renders can be mitigated with proper memoization

### 3. Form Validation: Zod + React Hook Form

**Decision**: Use Zod for schema validation with React Hook Form.

**Rationale**:
- **Type Safety**: Zod schemas generate TypeScript types
- **Runtime Validation**: Catches validation errors at runtime
- **DX**: Excellent developer experience with type inference
- **Consistency**: Same validation rules as backend (can share schemas)

**Trade-offs**:
- Additional bundle size (acceptable - Zod is tree-shakeable)

### 4. Route Protection: Layout-based Guards

**Decision**: Use Next.js 14+ route groups with layout-based protection.

**Rationale**:
- **Declarative**: Protection is clear from directory structure
- **Efficient**: Single auth check per layout
- **Next.js Patterns**: Follows Next.js 14+ best practices
- **Scalable**: Easy to add new protected routes

**Trade-offs**:
- Requires understanding of Next.js route groups
- Migration from older patterns may require refactoring

## Security Considerations

### XSS Protection
- ✅ Access tokens stored in memory (not localStorage)
- ✅ Refresh tokens in HttpOnly cookies
- ✅ Input sanitization with validation schemas
- ✅ CSP headers (should be configured in backend)

### CSRF Protection
- ✅ SameSite cookie attribute for refresh tokens
- ✅ Origin validation in backend
- ✅ Custom headers (Authorization) for protected requests

### Token Security
- ✅ Short-lived access tokens (30 minutes)
- ✅ Longer-lived refresh tokens (7 days)
- ✅ Token rotation on refresh
- ✅ Secure token transmission (HTTPS in production)

### Session Management
- ✅ Logout clears all tokens and state
- ✅ Automatic logout on token expiry
- ✅ Manual logout button in UI
- ✅ Logout on multiple tabs (use BroadcastChannel API - future)

## Testing Strategy

### Unit Tests
- Auth API client methods
- Token refresh logic
- Validation schemas
- Auth context actions
- Custom hooks (useAuth, useUser)

### Component Tests
- LoginForm validation and submission
- RegisterForm validation and submission
- AuthGuard redirect behavior
- UserMenu display and interactions

### Integration Tests
- Complete registration flow
- Complete login flow
- Protected route access
- Token refresh during request
- Logout flow

### E2E Tests (Optional)
- Full user journey: register → login → access protected page → logout
- Token refresh during long session
- Multiple tab scenarios

## Success Criteria

1. ✅ Users can register new accounts with validation
2. ✅ Users can login with username/password
3. ✅ Protected routes redirect to login when unauthenticated
4. ✅ Access tokens automatically refresh on expiry
5. ✅ Users can logout and clear session
6. ✅ Auth state persists across page refreshes (via refresh token)
7. ✅ User information displays correctly after login
8. ✅ Forms show validation errors clearly
9. ✅ Loading states display during auth operations
10. ✅ All tests passing with >80% coverage

## Future Enhancements

### Phase 8: OAuth Integration (Future)
- Google OAuth login
- Microsoft EntraID login
- GitHub OAuth login
- Social account linking

### Phase 9: Advanced Features (Future)
- Email verification flow
- Password reset flow
- Remember me / persistent login
- Multi-factor authentication (MFA)
- Session management (active sessions list)
- Account deletion

### Phase 10: Performance & UX (Future)
- Optimistic UI updates
- Background token refresh
- Multi-tab synchronization (BroadcastChannel API)
- Progressive Web App (PWA) support

## References

- Backend auth implementation: `feature/auth-system` (merged to master)
- OWASP Auth Cheatsheet: https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html
- JWT Best Practices: https://datatracker.ietf.org/doc/html/rfc8725
- Next.js 14 Auth Patterns: https://nextjs.org/docs/app/building-your-application/authentication
- React Context Best Practices: https://react.dev/learn/passing-data-deeply-with-context
