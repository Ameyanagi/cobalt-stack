# Component Organization

## Table of Contents
- [Overview](#overview)
- [Component Structure](#component-structure)
- [shadcn/ui Integration](#shadcnui-integration)
- [Custom Component Guidelines](#custom-component-guidelines)
- [Component Composition](#component-composition)
- [Visual Examples](#visual-examples)

## Overview

Cobalt Stack uses a hierarchical component architecture combining shadcn/ui primitives with custom domain components. Components are organized by responsibility and follow consistent patterns for maintainability and reusability.

## Component Structure

### Directory Organization

```
components/
├── admin/              # Admin-specific features
│   └── user-table.tsx
├── auth/               # Authentication features
│   ├── login-form.tsx
│   ├── register-form.tsx
│   └── unverified-email-banner.tsx
├── theme/              # Theme-related components
│   ├── theme-toggle.tsx
│   └── theme-selector.tsx
└── ui/                 # shadcn/ui primitives
    ├── button.tsx
    ├── input.tsx
    ├── card.tsx
    └── ...
```

### Component Categories

| Category | Purpose | Examples | Client/Server |
|----------|---------|----------|---------------|
| UI | Reusable primitives | Button, Input, Card | Client |
| Feature | Domain-specific | LoginForm, UserTable | Client |
| Layout | Page structure | DashboardLayout | Server/Client |
| Theme | Theme system | ThemeSelector, ThemeToggle | Client |

## shadcn/ui Integration

### Component Installation

shadcn/ui components are installed individually and live in `components/ui/`:

```bash
# Install a component
npx shadcn-ui@latest add button

# Install multiple components
npx shadcn-ui@latest add button input card
```

### Base Components

Core UI primitives provided by shadcn/ui:

```typescript
// components/ui/button.tsx
import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'

const buttonVariants = cva(
  'inline-flex items-center justify-center rounded-md transition-colors',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        destructive: 'bg-destructive text-destructive-foreground',
        outline: 'border border-input hover:bg-accent',
        ghost: 'hover:bg-accent hover:text-accent-foreground',
      },
      size: {
        default: 'h-10 px-4 py-2',
        sm: 'h-9 px-3',
        lg: 'h-11 px-8',
        icon: 'h-10 w-10',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  }
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button'
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    )
  }
)
Button.displayName = 'Button'
```

### Available UI Components

| Component | Purpose | Composition |
|-----------|---------|-------------|
| Button | Actions and triggers | Radix Slot |
| Input | Text input | Native input |
| Card | Content containers | div composition |
| Form | Form management | React Hook Form |
| Dropdown | Menu selections | Radix Dropdown |
| Dialog | Modal interactions | Radix Dialog |
| Sheet | Side panels | Radix Dialog |
| Separator | Visual dividers | Radix Separator |

## Custom Component Guidelines

### Feature Components

Feature components combine UI primitives with business logic:

```typescript
// components/auth/login-form.tsx
'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { useAuth } from '@/contexts/auth-context'

export function LoginForm() {
  const router = useRouter()
  const { login } = useAuth()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    setIsLoading(true)
    setError(null)

    const formData = new FormData(e.currentTarget)
    const username = formData.get('username') as string
    const password = formData.get('password') as string

    try {
      const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      })

      if (!response.ok) {
        throw new Error('Invalid credentials')
      }

      const data = await response.json()
      login(data.access_token, data.user)
      router.push('/dashboard')
    } catch (err) {
      setError('Invalid username or password')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle>Login</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <label htmlFor="username">Username</label>
            <Input
              id="username"
              name="username"
              required
              disabled={isLoading}
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="password">Password</label>
            <Input
              id="password"
              name="password"
              type="password"
              required
              disabled={isLoading}
            />
          </div>
          {error && (
            <div className="text-sm text-destructive">{error}</div>
          )}
          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? 'Logging in...' : 'Login'}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}
```

### Theme Components

Theme components manage application appearance:

```typescript
// components/theme/theme-selector.tsx
'use client'

import { Palette } from 'lucide-react'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Button } from '@/components/ui/button'
import { useTheme } from '@/contexts/theme-context'
import { themes, type ThemeName } from '@/lib/theme-config'

export function ThemeSelector() {
  const { theme: currentTheme, setTheme } = useTheme()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" aria-label="Select theme">
          <Palette className="h-5 w-5" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-56">
        <DropdownMenuLabel>Choose Theme</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {Object.values(themes).map((theme) => (
          <DropdownMenuItem
            key={theme.id}
            onClick={() => setTheme(theme.id)}
            className="cursor-pointer"
          >
            <div className="flex items-center gap-3 w-full">
              <div className="flex gap-1">
                <div
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: theme.preview.primary }}
                />
                <div
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: theme.preview.secondary }}
                />
                <div
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: theme.preview.accent }}
                />
              </div>
              <div className="flex-1">
                <div className="font-medium">{theme.name}</div>
                <div className="text-xs text-muted-foreground">
                  {theme.description}
                </div>
              </div>
              {currentTheme === theme.id && (
                <div className="text-primary">✓</div>
              )}
            </div>
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
```

### Component Patterns

#### Container/Presentation Pattern

Separate logic from presentation:

```typescript
// Container Component (logic)
export function UserTableContainer() {
  const { data, isLoading } = useQuery({
    queryKey: ['users'],
    queryFn: fetchUsers,
  })

  if (isLoading) return <LoadingSpinner />

  return <UserTable data={data} />
}

// Presentation Component (UI)
interface UserTableProps {
  data: User[]
}

export function UserTable({ data }: UserTableProps) {
  return (
    <table>
      {/* Render data */}
    </table>
  )
}
```

#### Compound Components Pattern

Create flexible component APIs:

```typescript
// Card compound component
export function Card({ children, ...props }: CardProps) {
  return <div {...props}>{children}</div>
}

Card.Header = function CardHeader({ children }: { children: ReactNode }) {
  return <div className="card-header">{children}</div>
}

Card.Body = function CardBody({ children }: { children: ReactNode }) {
  return <div className="card-body">{children}</div>
}

Card.Footer = function CardFooter({ children }: { children: ReactNode }) {
  return <div className="card-footer">{children}</div>
}

// Usage
<Card>
  <Card.Header>Title</Card.Header>
  <Card.Body>Content</Card.Body>
  <Card.Footer>Actions</Card.Footer>
</Card>
```

## Component Composition

### Composing UI Components

Build complex interfaces from simple primitives:

```typescript
// Simple Dialog composition
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'

export function DeleteUserDialog({ userId }: { userId: string }) {
  const [open, setOpen] = useState(false)
  const mutation = useMutation({
    mutationFn: deleteUser,
    onSuccess: () => setOpen(false),
  })

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="destructive">Delete</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Confirm Deletion</DialogTitle>
        </DialogHeader>
        <p>Are you sure you want to delete this user?</p>
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            onClick={() => mutation.mutate(userId)}
          >
            Delete
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
```

### Form Composition

Combine form components with validation:

```typescript
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import * as z from 'zod'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

const formSchema = z.object({
  username: z.string().min(3).max(20),
  email: z.string().email(),
  password: z.string().min(8),
})

export function RegisterForm() {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  })

  const onSubmit = (data: z.infer<typeof formSchema>) => {
    // Handle submission
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="username"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Username</FormLabel>
              <FormControl>
                <Input {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="email"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Email</FormLabel>
              <FormControl>
                <Input type="email" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Password</FormLabel>
              <FormControl>
                <Input type="password" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type="submit" className="w-full">
          Register
        </Button>
      </form>
    </Form>
  )
}
```

## Visual Examples

**Note**: Component screenshots are available in `docs/frontend/screenshots/components/`. Screenshots include:
- Button variants (default, destructive, outline, ghost)
- Form components (input, textarea, select, checkbox)
- Card layouts (basic, with header, with footer)
- Dialog and sheet examples
- Theme selector interface
- Authentication forms

## Best Practices

### Component Design
- Keep components focused on single responsibility
- Use TypeScript for all components
- Export types alongside components
- Document complex component APIs

### Composition
- Prefer composition over prop drilling
- Use compound components for flexible APIs
- Extract reusable patterns to custom components
- Leverage shadcn/ui primitives

### Performance
- Use React.memo for expensive renders
- Optimize re-renders with proper key props
- Lazy load heavy components
- Split client/server components appropriately

### Accessibility
- Use semantic HTML elements
- Include ARIA labels and roles
- Ensure keyboard navigation
- Test with screen readers

## Related Documentation
- [Architecture](./architecture.md)
- [State Management](./state-management.md)
- [Theme System](./themes.md)
- [Testing](./testing.md)
