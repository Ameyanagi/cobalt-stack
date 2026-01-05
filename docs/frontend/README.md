# Frontend Documentation

Comprehensive documentation for the Cobalt Stack React/Next.js frontend.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
- [Components](#components)
- [Styling](#styling)
- [State Management](#state-management)
- [Testing](#testing)
- [Visual Documentation](#visual-documentation)

## Overview

The Cobalt Stack frontend is built with:
- **React** - UI library
- **Next.js** - React framework
- **TypeScript** - Type safety
- **Tailwind CSS** - Styling

## Architecture

### Project Structure

```
frontend/
├── src/
│   ├── components/    # React components
│   ├── pages/         # Next.js pages
│   ├── styles/        # CSS and styling
│   ├── hooks/         # Custom React hooks
│   ├── utils/         # Utility functions
│   └── types/         # TypeScript types
├── public/            # Static assets
└── tests/             # Test files
```

### Design Patterns

- **Component Composition**: Building complex UIs from simple components
- **Custom Hooks**: Reusable logic extraction
- **Context API**: Global state management
- **CSS Modules**: Scoped styling

## Getting Started

### Prerequisites

- Node.js 18+
- npm or yarn
- Modern browser

### Development Setup

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Run production build
npm start

# Run tests
npm test
```

### Environment Configuration

Create `.env.local` file:

```bash
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_APP_NAME=Cobalt Stack
```

## Components

### Component Structure

```tsx
// components/MyComponent/MyComponent.tsx
import React from 'react';
import styles from './MyComponent.module.css';

interface MyComponentProps {
  title: string;
  onAction: () => void;
}

/**
 * MyComponent - Brief description
 *
 * @param props - Component props
 * @returns React component
 */
export const MyComponent: React.FC<MyComponentProps> = ({ title, onAction }) => {
  return (
    <div className={styles.container}>
      <h2>{title}</h2>
      <button onClick={onAction}>Action</button>
    </div>
  );
};
```

### Component Categories

- **Layout Components**: Page structure and navigation
- **UI Components**: Buttons, forms, cards, etc.
- **Feature Components**: Business logic components
- **Utility Components**: Error boundaries, loading states

## Styling

### Tailwind CSS

```tsx
// Using Tailwind classes
export const Button = () => (
  <button className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600">
    Click Me
  </button>
);
```

### CSS Modules

```tsx
// Using CSS modules
import styles from './Button.module.css';

export const Button = () => (
  <button className={styles.button}>
    Click Me
  </button>
);
```

### Theme System

Theme configuration with dark/light mode support:

```tsx
// Using theme context
import { useTheme } from '@/contexts/ThemeContext';

export const ThemedComponent = () => {
  const { theme, toggleTheme } = useTheme();

  return (
    <div className={theme === 'dark' ? 'dark-mode' : 'light-mode'}>
      <button onClick={toggleTheme}>Toggle Theme</button>
    </div>
  );
};
```

## State Management

### React Context

```tsx
// contexts/AppContext.tsx
import { createContext, useContext, useState } from 'react';

interface AppState {
  user: User | null;
  setUser: (user: User | null) => void;
}

const AppContext = createContext<AppState | undefined>(undefined);

export const AppProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);

  return (
    <AppContext.Provider value={{ user, setUser }}>
      {children}
    </AppContext.Provider>
  );
};

export const useApp = () => {
  const context = useContext(AppContext);
  if (!context) throw new Error('useApp must be used within AppProvider');
  return context;
};
```

### Custom Hooks

```tsx
// hooks/useApi.ts
import { useState, useEffect } from 'react';

export const useApi = <T,>(url: string) => {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    fetch(url)
      .then(res => res.json())
      .then(setData)
      .catch(setError)
      .finally(() => setLoading(false));
  }, [url]);

  return { data, loading, error };
};
```

## Testing

### Component Tests

```tsx
// components/Button/Button.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { Button } from './Button';

describe('Button', () => {
  it('renders with text', () => {
    render(<Button>Click Me</Button>);
    expect(screen.getByText('Click Me')).toBeInTheDocument();
  });

  it('calls onClick handler', () => {
    const handleClick = jest.fn();
    render(<Button onClick={handleClick}>Click Me</Button>);

    fireEvent.click(screen.getByText('Click Me'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });
});
```

### Running Tests

```bash
# Run all tests
npm test

# Run with coverage
npm run test:coverage

# Run in watch mode
npm run test:watch

# Run E2E tests
npm run test:e2e
```

## Visual Documentation

Explore visual documentation of UI components, themes, and features:

- **[Screenshot Gallery](./screenshots/README.md)** - Visual component reference
- **[Theme Variations](./screenshots/themes/)** - Light/dark mode examples
- **[Component Library](./screenshots/components/)** - Individual components
- **[Feature Demos](./screenshots/features/)** - Complete features

## Code Documentation

### TSDoc Comments

```tsx
/**
 * Formats a date string for display
 *
 * @param date - Date to format
 * @param format - Format string (default: 'YYYY-MM-DD')
 * @returns Formatted date string
 *
 * @example
 * ```tsx
 * const formatted = formatDate(new Date(), 'MM/DD/YYYY');
 * // Returns: "10/26/2025"
 * ```
 */
export const formatDate = (date: Date, format = 'YYYY-MM-DD'): string => {
  // Implementation
};
```

### Component Documentation

```tsx
/**
 * Card component for displaying content in a contained box
 *
 * @component
 * @example
 * ```tsx
 * <Card title="My Card" variant="primary">
 *   <p>Card content</p>
 * </Card>
 * ```
 */
export const Card: React.FC<CardProps> = ({ title, children, variant = 'default' }) => {
  // Implementation
};
```

## Related Resources

- [Getting Started Guide](../getting-started/quick-start.md)
- [API Reference](../api/README.md)
- [Deployment Guide](../deployment/README.md)
- [Troubleshooting](../troubleshooting/README.md)
