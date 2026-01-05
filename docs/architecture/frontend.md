# Frontend Architecture

Comprehensive guide to the Next.js frontend architecture, including component structure, routing, state management, and integration patterns.

## Technology Stack

### Core Framework

- **Next.js 15**: React framework with App Router
- **React 19**: UI library with Server Components
- **TypeScript 5**: Type safety and IntelliSense
- **Bun**: Fast JavaScript runtime and package manager

### Styling & UI

- **Tailwind CSS 3**: Utility-first CSS framework
- **shadcn/ui**: Accessible, customizable component library
- **Radix UI**: Unstyled accessible primitives
- **Lucide React**: Icon library
- **class-variance-authority**: Component variant management

### Forms & Validation

- **react-hook-form**: Performant form management
- **zod**: TypeScript-first schema validation
- **@hookform/resolvers**: Zod integration for react-hook-form

### HTTP & State

- **Fetch API**: Native HTTP client
- **React Context**: Global state management
- **React hooks**: Local state management
- **OpenAPI TypeScript**: Auto-generated API client types

## Project Structure

```
frontend/
├── src/
│   ├── app/                    # App Router pages
│   │   ├── layout.tsx          # Root layout (shared across pages)
│   │   ├── page.tsx            # Home page (/)
│   │   ├── globals.css         # Global styles (Tailwind)
│   │   ├── login/              # Login page route
│   │   │   └── page.tsx        # Login page component
│   │   ├── register/           # Register page route
│   │   │   └── page.tsx        # Register page component
│   │   ├── dashboard/          # Dashboard route (protected)
│   │   │   ├── layout.tsx      # Dashboard layout
│   │   │   └── page.tsx        # Dashboard page
│   │   └── admin/              # Admin routes (admin only)
│   │       ├── layout.tsx      # Admin layout
│   │       └── page.tsx        # Admin dashboard
│   ├── components/             # React components
│   │   ├── ui/                 # shadcn/ui components
│   │   │   ├── button.tsx      # Button component
│   │   │   ├── input.tsx       # Input component
│   │   │   ├── card.tsx        # Card component
│   │   │   └── ...             # Other UI primitives
│   │   ├── auth/               # Authentication components
│   │   │   ├── LoginForm.tsx   # Login form
│   │   │   └── RegisterForm.tsx # Register form
│   │   ├── layout/             # Layout components
│   │   │   ├── Header.tsx      # App header
│   │   │   ├── Sidebar.tsx     # Dashboard sidebar
│   │   │   └── Footer.tsx      # App footer
│   │   └── providers/          # Context providers
│   │       └── AuthProvider.tsx # Auth context provider
│   ├── lib/                    # Utility libraries
│   │   ├── api.ts              # API client functions
│   │   ├── auth.ts             # Auth utilities
│   │   └── utils.ts            # General utilities
│   ├── hooks/                  # Custom React hooks
│   │   ├── useAuth.tsx         # Auth hook
│   │   └── useUser.tsx         # User data hook
│   ├── types/                  # TypeScript types
│   │   ├── api.ts              # Auto-generated API types
│   │   └── index.ts            # Custom types
│   └── config/                 # Configuration
│       └── site.ts             # Site metadata
├── public/                     # Static assets
│   ├── favicon.ico             # Site icon
│   └── images/                 # Image assets
├── .env.local                  # Local environment variables
├── next.config.js              # Next.js configuration
├── tailwind.config.ts          # Tailwind configuration
├── components.json             # shadcn/ui configuration
├── tsconfig.json               # TypeScript configuration
├── package.json                # Dependencies
└── Dockerfile                  # Production Docker image
```

## Architecture Patterns

### App Router (Next.js 15)

**File-based Routing**: Directory structure defines routes

```
app/
├── page.tsx              → /
├── login/page.tsx        → /login
├── register/page.tsx     → /register
├── dashboard/
│   ├── page.tsx          → /dashboard
│   └── settings/
│       └── page.tsx      → /dashboard/settings
```

**Special Files**:
- `layout.tsx`: Shared layout (persists across navigations)
- `page.tsx`: Page content (unique to route)
- `loading.tsx`: Loading UI (Suspense boundary)
- `error.tsx`: Error UI (Error boundary)
- `not-found.tsx`: 404 UI

### Server Components (Default)

**Benefits**:
- Zero JavaScript shipped to client
- Direct database/API access (backend)
- Automatic code splitting
- Streaming with Suspense

**Example**:
```tsx
// app/dashboard/page.tsx (Server Component)
async function DashboardPage() {
  // Direct API call (server-side)
  const user = await fetchUser();

  return (
    <div>
      <h1>Welcome, {user.name}</h1>
    </div>
  );
}
```

### Client Components

**Usage**: Interactive UI, browser APIs, React hooks

**Declaration**: `"use client"` directive at top of file

**Example**:
```tsx
"use client";

import { useState } from "react";

export function Counter() {
  const [count, setCount] = useState(0);

  return (
    <button onClick={() => setCount(count + 1)}>
      Count: {count}
    </button>
  );
}
```

### Hybrid Composition

**Pattern**: Server Component wrapping Client Component

```tsx
// app/dashboard/page.tsx (Server Component)
import { UserProfile } from "@/components/UserProfile"; // Client Component

async function DashboardPage() {
  const user = await fetchUser(); // Server-side fetch

  return (
    <div>
      <UserProfile user={user} /> {/* Pass data to client */}
    </div>
  );
}
```

## Authentication Flow

### Auth Context Provider

**Location**: `src/components/providers/AuthProvider.tsx`

**Purpose**: Global authentication state

**Provided Values**:
- `user`: Current user object (or null)
- `isLoading`: Loading state
- `login`: Login function
- `logout`: Logout function
- `refreshToken`: Token refresh function

**Implementation**:
```tsx
"use client";

import { createContext, useContext, useState, useEffect } from "react";

type AuthContextType = {
  user: User | null;
  isLoading: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshToken: () => Promise<void>;
};

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Initialize: Check for existing token
  useEffect(() => {
    checkAuth();
  }, []);

  async function checkAuth() {
    try {
      const userData = await fetchCurrentUser();
      setUser(userData);
    } catch (error) {
      setUser(null);
    } finally {
      setIsLoading(false);
    }
  }

  async function login(email: string, password: string) {
    const response = await apiClient.post("/api/auth/login", { email, password });
    localStorage.setItem("access_token", response.access_token);
    setUser(response.user);
  }

  async function logout() {
    await apiClient.post("/api/auth/logout");
    localStorage.removeItem("access_token");
    setUser(null);
  }

  async function refreshToken() {
    const response = await apiClient.post("/api/auth/refresh");
    localStorage.setItem("access_token", response.access_token);
  }

  return (
    <AuthContext.Provider value={{ user, isLoading, login, logout, refreshToken }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) throw new Error("useAuth must be used within AuthProvider");
  return context;
}
```

### Protected Routes

**Pattern**: Middleware or layout-based protection

**Layout Protection** (Recommended):
```tsx
// app/dashboard/layout.tsx
import { redirect } from "next/navigation";
import { fetchCurrentUser } from "@/lib/api";

export default async function DashboardLayout({ children }: { children: React.ReactNode }) {
  try {
    const user = await fetchCurrentUser();
    if (!user) redirect("/login");
  } catch (error) {
    redirect("/login");
  }

  return (
    <div>
      <DashboardHeader />
      <main>{children}</main>
    </div>
  );
}
```

### Token Management

**Storage**:
- Access token: localStorage (or sessionStorage)
- Refresh token: HttpOnly cookie (managed by backend)

**Access Token Usage**:
```typescript
// lib/api.ts
export async function apiClient(endpoint: string, options: RequestInit = {}) {
  const token = localStorage.getItem("access_token");

  const response = await fetch(`${API_URL}${endpoint}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      "Authorization": token ? `Bearer ${token}` : "",
      ...options.headers,
    },
    credentials: "include", // Include cookies (refresh token)
  });

  // Handle 401 (expired token)
  if (response.status === 401) {
    // Attempt token refresh
    const refreshed = await refreshToken();
    if (refreshed) {
      // Retry original request with new token
      return apiClient(endpoint, options);
    } else {
      // Refresh failed, redirect to login
      window.location.href = "/login";
    }
  }

  return response.json();
}
```

**Automatic Token Refresh**:
```typescript
async function refreshToken(): Promise<boolean> {
  try {
    const response = await fetch(`${API_URL}/api/auth/refresh`, {
      method: "POST",
      credentials: "include", // Send refresh token cookie
    });

    if (response.ok) {
      const data = await response.json();
      localStorage.setItem("access_token", data.access_token);
      return true;
    }
    return false;
  } catch (error) {
    return false;
  }
}
```

## Form Management

### react-hook-form + Zod Integration

**Benefits**:
- Type-safe validation
- Minimal re-renders
- Built-in error handling
- Async validation support

**Example: Login Form**

```tsx
"use client";

import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";

// Validation schema
const loginSchema = z.object({
  email: z.string().email("Invalid email address"),
  password: z.string().min(8, "Password must be at least 8 characters"),
});

type LoginFormData = z.infer<typeof loginSchema>;

export function LoginForm() {
  const { register, handleSubmit, formState: { errors, isSubmitting } } = useForm<LoginFormData>({
    resolver: zodResolver(loginSchema),
  });

  async function onSubmit(data: LoginFormData) {
    try {
      await login(data.email, data.password);
      router.push("/dashboard");
    } catch (error) {
      console.error("Login failed:", error);
    }
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <div>
        <label htmlFor="email">Email</label>
        <input
          id="email"
          type="email"
          {...register("email")}
          aria-invalid={errors.email ? "true" : "false"}
        />
        {errors.email && <span>{errors.email.message}</span>}
      </div>

      <div>
        <label htmlFor="password">Password</label>
        <input
          id="password"
          type="password"
          {...register("password")}
          aria-invalid={errors.password ? "true" : "false"}
        />
        {errors.password && <span>{errors.password.message}</span>}
      </div>

      <button type="submit" disabled={isSubmitting}>
        {isSubmitting ? "Logging in..." : "Login"}
      </button>
    </form>
  );
}
```

### Form Validation Patterns

**Client-Side Validation**:
- Zod schema validation
- Real-time field validation
- Custom validation rules

**Server-Side Validation**:
- Backend validates all inputs
- Frontend displays server errors
- Prevents malicious requests

**Error Handling**:
```tsx
// Display server errors
const { setError } = useForm();

try {
  await submitForm(data);
} catch (error) {
  if (error.field) {
    setError(error.field, { message: error.message });
  } else {
    // Global error
    toast.error(error.message);
  }
}
```

## Component Architecture

### UI Components (shadcn/ui)

**Philosophy**: Copy-paste, not npm install

**Structure**: Each component is a standalone file

**Example: Button Component**
```tsx
// components/ui/button.tsx
import { cva, type VariantProps } from "class-variance-authority";

const buttonVariants = cva(
  "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/90",
        destructive: "bg-destructive text-destructive-foreground hover:bg-destructive/90",
        outline: "border border-input hover:bg-accent hover:text-accent-foreground",
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-9 rounded-md px-3",
        lg: "h-11 rounded-md px-8",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {}

export function Button({ variant, size, className, ...props }: ButtonProps) {
  return (
    <button
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  );
}
```

**Usage**:
```tsx
<Button variant="default">Click me</Button>
<Button variant="outline" size="sm">Small button</Button>
<Button variant="destructive">Delete</Button>
```

### Feature Components

**Pattern**: Compose UI components into features

**Example: User Card**
```tsx
// components/UserCard.tsx
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

interface UserCardProps {
  user: User;
  onEdit: () => void;
  onDelete: () => void;
}

export function UserCard({ user, onEdit, onDelete }: UserCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{user.name}</CardTitle>
      </CardHeader>
      <CardContent>
        <p>Email: {user.email}</p>
        <p>Role: {user.role}</p>
        <div className="flex gap-2 mt-4">
          <Button onClick={onEdit}>Edit</Button>
          <Button variant="destructive" onClick={onDelete}>Delete</Button>
        </div>
      </CardContent>
    </Card>
  );
}
```

### Layout Components

**Pattern**: Reusable layout structures

**Example: Dashboard Layout**
```tsx
// components/layout/DashboardLayout.tsx
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";

export function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen">
      <Sidebar />
      <div className="flex-1 flex flex-col">
        <Header />
        <main className="flex-1 overflow-y-auto p-6">
          {children}
        </main>
      </div>
    </div>
  );
}
```

## API Integration

### Type-Safe API Client

**OpenAPI Code Generation**:
```bash
bunx openapi-typescript ../openapi/schema.json -o src/types/api.ts
```

**Generated Types**:
```typescript
// types/api.ts (auto-generated)
export interface paths {
  "/api/auth/login": {
    post: {
      requestBody: {
        content: {
          "application/json": {
            email: string;
            password: string;
          };
        };
      };
      responses: {
        200: {
          content: {
            "application/json": {
              access_token: string;
              user: components["schemas"]["User"];
            };
          };
        };
      };
    };
  };
}
```

**Usage**:
```typescript
// lib/api.ts
import type { paths } from "@/types/api";

type LoginRequest = paths["/api/auth/login"]["post"]["requestBody"]["content"]["application/json"];
type LoginResponse = paths["/api/auth/login"]["post"]["responses"]["200"]["content"]["application/json"];

export async function login(data: LoginRequest): Promise<LoginResponse> {
  const response = await apiClient("/api/auth/login", {
    method: "POST",
    body: JSON.stringify(data),
  });
  return response;
}
```

### API Client Functions

**Centralized API calls** (Location: `lib/api.ts`):

```typescript
const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:2750";

// Auth
export async function login(email: string, password: string) { ... }
export async function register(email: string, password: string) { ... }
export async function logout() { ... }
export async function getCurrentUser() { ... }
export async function verifyEmail(code: string) { ... }

// Admin
export async function listUsers(page: number, limit: number) { ... }
export async function getUser(id: string) { ... }
export async function disableUser(id: string) { ... }
export async function enableUser(id: string) { ... }
export async function getStats() { ... }
```

## Styling Strategy

### Tailwind CSS

**Configuration** (`tailwind.config.ts`):
```typescript
export default {
  content: [
    "./src/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        // ... more colors
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
};
```

**CSS Variables** (`app/globals.css`):
```css
@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;
    --primary: 222.2 47.4% 11.2%;
    --primary-foreground: 210 40% 98%;
    /* ... more variables */
  }

  .dark {
    --background: 222.2 84% 4.9%;
    --foreground: 210 40% 98%;
    /* ... dark mode overrides */
  }
}
```

### Component Styling

**Utility Classes**:
```tsx
<div className="flex items-center justify-between p-4 bg-white rounded-lg shadow-md">
  <h2 className="text-2xl font-bold text-gray-900">Title</h2>
  <Button className="ml-4">Action</Button>
</div>
```

**Responsive Design**:
```tsx
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
  {/* Responsive grid: 1 col on mobile, 2 on tablet, 3 on desktop */}
</div>
```

## State Management

### Local State (useState)

**Usage**: Component-specific state

```tsx
const [count, setCount] = useState(0);
const [isOpen, setIsOpen] = useState(false);
```

### Context API (Global State)

**Usage**: Shared state across components

**Example: Theme Context**
```tsx
const ThemeContext = createContext<{ theme: string; setTheme: (theme: string) => void }>(undefined);

export function ThemeProvider({ children }) {
  const [theme, setTheme] = useState("light");

  return (
    <ThemeContext.Provider value={{ theme, setTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}
```

### Server State (React Query - Future)

**Benefits**:
- Automatic caching
- Background refetching
- Optimistic updates
- Pagination support

## Performance Optimization

### Code Splitting

**Automatic** (Next.js):
- Each page is a separate bundle
- Shared code extracted to chunks

**Manual**:
```tsx
import dynamic from "next/dynamic";

const HeavyComponent = dynamic(() => import("./HeavyComponent"), {
  loading: () => <p>Loading...</p>,
  ssr: false, // Client-side only
});
```

### Image Optimization

**Next.js Image Component**:
```tsx
import Image from "next/image";

<Image
  src="/logo.png"
  alt="Logo"
  width={200}
  height={100}
  priority // Load immediately (above-the-fold)
/>
```

**Benefits**:
- Automatic WebP/AVIF conversion
- Lazy loading by default
- Responsive srcset generation

### Caching Strategies

**Server Components** (Automatic):
- Static rendering (cached indefinitely)
- Dynamic rendering (per-request)

**Client-Side Caching**:
- Browser cache (HTTP headers)
- Service workers (future PWA)

## Error Handling

### Error Boundaries

**App-Level** (`app/error.tsx`):
```tsx
"use client";

export default function Error({ error, reset }: { error: Error; reset: () => void }) {
  return (
    <div>
      <h2>Something went wrong!</h2>
      <button onClick={reset}>Try again</button>
    </div>
  );
}
```

### API Error Handling

**Pattern**: Centralized error handling

```typescript
class ApiError extends Error {
  constructor(public status: number, public message: string) {
    super(message);
  }
}

async function apiClient(endpoint: string, options: RequestInit) {
  const response = await fetch(`${API_URL}${endpoint}`, options);

  if (!response.ok) {
    const error = await response.json();
    throw new ApiError(response.status, error.message);
  }

  return response.json();
}
```

**Usage with Toast Notifications**:
```tsx
try {
  await login(email, password);
  toast.success("Login successful!");
} catch (error) {
  if (error instanceof ApiError) {
    toast.error(error.message);
  } else {
    toast.error("An unexpected error occurred");
  }
}
```

## Testing Strategy

### Unit Tests (Future)

**Framework**: Vitest or Jest

**Coverage**:
- Utility functions
- Custom hooks
- Form validation

### Component Tests (Future)

**Framework**: React Testing Library

**Coverage**:
- Component rendering
- User interactions
- State changes

### E2E Tests (Future)

**Framework**: Playwright

**Coverage**:
- Authentication flows
- Form submissions
- Navigation

## Deployment

### Build Configuration

**next.config.js**:
```javascript
module.exports = {
  output: "standalone", // Optimize for Docker
  experimental: {
    serverActions: true,
  },
};
```

### Environment Variables

**Build-time** (embedded in bundle):
- `NEXT_PUBLIC_API_URL`: Backend API URL

**Runtime** (server-only):
- No sensitive runtime variables (stateless frontend)

### Docker Build

**Multi-stage Dockerfile**:
1. **deps**: Install dependencies
2. **builder**: Build Next.js app
3. **runner**: Run production server (minimal image)

**Image Size**: ~200MB (Bun runtime + built app)

## Accessibility

### ARIA Labels

```tsx
<button aria-label="Close modal" onClick={onClose}>
  <X className="h-4 w-4" />
</button>
```

### Keyboard Navigation

```tsx
<div
  role="button"
  tabIndex={0}
  onKeyDown={(e) => e.key === "Enter" && handleClick()}
  onClick={handleClick}
>
  Clickable div
</div>
```

### Form Accessibility

```tsx
<label htmlFor="email">Email</label>
<input
  id="email"
  type="email"
  aria-describedby="email-error"
  aria-invalid={errors.email ? "true" : "false"}
/>
{errors.email && <span id="email-error" role="alert">{errors.email.message}</span>}
```

## Future Enhancements

- **Progressive Web App (PWA)**: Offline support, installable
- **Internationalization (i18n)**: Multi-language support
- **Dark Mode**: Theme switching
- **Real-time Updates**: WebSocket integration
- **Advanced Caching**: React Query or SWR
- **Analytics**: User behavior tracking
- **SEO Optimization**: Metadata, sitemap, robots.txt

## References

- [Next.js Documentation](https://nextjs.org/docs)
- [React Documentation](https://react.dev)
- [shadcn/ui Documentation](https://ui.shadcn.com)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [react-hook-form Documentation](https://react-hook-form.com)
