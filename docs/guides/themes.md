# Themes Guide

Complete guide to using and customizing the theme system in Cobalt Stack.

## Table of Contents

- [Overview](#overview)
- [Using Themes](#using-themes)
- [Available Themes](#available-themes)
- [Switching Themes](#switching-themes)
- [Customizing Colors](#customizing-colors)
- [Creating New Themes](#creating-new-themes)
- [Dark Mode](#dark-mode)
- [Troubleshooting](#troubleshooting)

## Overview

Cobalt Stack includes a flexible theme system that supports:

- **Multiple color themes**: Cobalt (default), Nature, Violet Bloom
- **Light/Dark modes**: Each theme supports both light and dark variants
- **CSS variables**: Easy customization using CSS custom properties
- **Persistent preferences**: Theme choices saved to localStorage
- **Type-safe**: Full TypeScript support for theme configuration

### Theme Architecture

```
Theme System
├── ThemeProvider (Context)
│   ├── Current theme state
│   ├── Current mode (light/dark)
│   └── Theme switching logic
├── Theme Configuration
│   ├── Theme definitions
│   ├── Color palettes
│   └── Preview colors
└── CSS Variables
    ├── Light mode colors
    └── Dark mode colors
```

## Using Themes

### ThemeProvider Setup

The theme system is initialized in your root layout:

```tsx
// app/layout.tsx
import { ThemeProvider } from '@/contexts/theme-context'

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <ThemeProvider>
          {children}
        </ThemeProvider>
      </body>
    </html>
  )
}
```

### useTheme Hook

Access theme functionality in any component:

```tsx
'use client'

import { useTheme } from '@/contexts/theme-context'

export function MyComponent() {
  const { theme, mode, setTheme, setMode, toggleMode } = useTheme()

  return (
    <div>
      <p>Current theme: {theme}</p>
      <p>Current mode: {mode}</p>

      <button onClick={() => setTheme('nature')}>
        Switch to Nature Theme
      </button>

      <button onClick={toggleMode}>
        Toggle {mode === 'light' ? 'Dark' : 'Light'} Mode
      </button>
    </div>
  )
}
```

## Available Themes

### 1. Cobalt (Default)

Professional cobalt blue theme with clean, modern aesthetics.

**Colors**:
- Primary: `#0047AB` (Cobalt Blue)
- Secondary: `#6B8EC9` (Light Blue)
- Accent: `#2E5A9E` (Deep Blue)

**Best for**: Corporate, professional, technical applications

```tsx
setTheme('default')
```

### 2. Nature

Calming green tones inspired by nature and growth.

**Colors**:
- Primary: `#22c55e` (Emerald Green)
- Secondary: `#84cc16` (Lime)
- Accent: `#10b981` (Teal)

**Best for**: Health, wellness, environmental, outdoor applications

```tsx
setTheme('nature')
```

### 3. Violet Bloom

Elegant purple palette with soft, sophisticated edges.

**Colors**:
- Primary: `#8b5cf6` (Violet)
- Secondary: `#a855f7` (Purple)
- Accent: `#c084fc` (Light Purple)

**Best for**: Creative, luxury, artistic, feminine applications

```tsx
setTheme('violet-bloom')
```

## Switching Themes

### Theme Toggle Component

```tsx
'use client'

import { useTheme } from '@/contexts/theme-context'
import { Button } from '@/components/ui/button'

export function ThemeToggle() {
  const { theme, setTheme } = useTheme()

  const themes = [
    { id: 'default', name: 'Cobalt', icon: '🔵' },
    { id: 'nature', name: 'Nature', icon: '🌿' },
    { id: 'violet-bloom', name: 'Violet', icon: '🌸' },
  ]

  return (
    <div className="flex gap-2">
      {themes.map((t) => (
        <Button
          key={t.id}
          onClick={() => setTheme(t.id)}
          variant={theme === t.id ? 'default' : 'outline'}
        >
          {t.icon} {t.name}
        </Button>
      ))}
    </div>
  )
}
```

### Theme Selector Dropdown

```tsx
'use client'

import { useTheme } from '@/contexts/theme-context'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { themes } from '@/lib/theme-config'

export function ThemeSelector() {
  const { theme, setTheme } = useTheme()

  return (
    <Select value={theme} onValueChange={setTheme}>
      <SelectTrigger className="w-[180px]">
        <SelectValue placeholder="Select theme" />
      </SelectTrigger>
      <SelectContent>
        {Object.values(themes).map((t) => (
          <SelectItem key={t.id} value={t.id}>
            <div className="flex items-center gap-2">
              <div
                className="h-4 w-4 rounded-full"
                style={{ backgroundColor: t.preview.primary }}
              />
              {t.name}
            </div>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
```

### Theme Previews

Show visual previews of themes:

```tsx
'use client'

import { useTheme } from '@/contexts/theme-context'
import { themes } from '@/lib/theme-config'

export function ThemePreviewer() {
  const { theme: currentTheme, setTheme } = useTheme()

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {Object.values(themes).map((theme) => (
        <button
          key={theme.id}
          onClick={() => setTheme(theme.id)}
          className={`p-4 border rounded-lg text-left transition-all ${
            currentTheme === theme.id
              ? 'border-primary ring-2 ring-primary'
              : 'border-gray-200 hover:border-gray-300'
          }`}
        >
          <div className="flex gap-2 mb-3">
            <div
              className="h-8 w-8 rounded"
              style={{ backgroundColor: theme.preview.primary }}
            />
            <div
              className="h-8 w-8 rounded"
              style={{ backgroundColor: theme.preview.secondary }}
            />
            <div
              className="h-8 w-8 rounded"
              style={{ backgroundColor: theme.preview.accent }}
            />
          </div>

          <h3 className="font-bold text-lg">{theme.name}</h3>
          <p className="text-sm text-gray-600">{theme.description}</p>

          {currentTheme === theme.id && (
            <div className="mt-2 text-xs text-primary font-medium">
              ✓ Active
            </div>
          )}
        </button>
      ))}
    </div>
  )
}
```

## Customizing Colors

### Understanding CSS Variables

Themes use CSS variables defined in `app/globals.css`:

```css
@layer base {
  :root {
    /* Light mode variables */
    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;
    --primary: 221.2 83.2% 53.3%;
    --primary-foreground: 210 40% 98%;
    /* ... more variables */
  }

  .dark {
    /* Dark mode variables */
    --background: 222.2 84% 4.9%;
    --foreground: 210 40% 98%;
    --primary: 217.2 91.2% 59.8%;
    --primary-foreground: 222.2 47.4% 11.2%;
    /* ... more variables */
  }
}
```

### Modifying Existing Themes

To adjust colors in an existing theme, edit `app/globals.css`:

```css
/* Customize default theme */
:root {
  --primary: 210 100% 50%;  /* Adjust primary color */
  --accent: 200 100% 40%;   /* Adjust accent color */
}

/* Customize dark mode */
.dark {
  --primary: 210 100% 60%;
  --accent: 200 100% 50%;
}
```

**HSL Format**: Colors use HSL (Hue Saturation Lightness) format:
- `210` = Hue (0-360)
- `100%` = Saturation (0-100%)
- `50%` = Lightness (0-100%)

### Theme-Specific Variables

Add theme-specific colors using data attributes:

```css
/* Nature theme customization */
[data-theme="nature"] {
  --primary: 142 71% 45%;  /* Emerald green */
  --accent: 84 81% 44%;    /* Lime green */
}

[data-theme="nature"].dark {
  --primary: 142 71% 55%;
  --accent: 84 81% 54%;
}

/* Violet Bloom theme */
[data-theme="violet-bloom"] {
  --primary: 258 90% 66%;  /* Violet */
  --accent: 270 95% 75%;   /* Light purple */
}

[data-theme="violet-bloom"].dark {
  --primary: 258 90% 76%;
  --accent: 270 95% 85%;
}
```

### Using Theme Colors in Components

```tsx
// Use CSS variables in Tailwind
<div className="bg-primary text-primary-foreground">
  Primary colored box
</div>

<div className="border-accent bg-accent/10">
  Accent colored border
</div>

// Use CSS variables in inline styles
<div style={{ color: 'hsl(var(--primary))' }}>
  Custom styled element
</div>
```

## Creating New Themes

### Step 1: Define Theme Configuration

Add your theme to `frontend/src/lib/theme-config.ts`:

```typescript
export type ThemeName = 'default' | 'nature' | 'violet-bloom' | 'ocean';

export const themes: Record<ThemeName, Theme> = {
  // ... existing themes

  'ocean': {
    id: 'ocean',
    name: 'Ocean',
    description: 'Serene blue and teal inspired by the sea',
    preview: {
      primary: '#0891b2',  // Cyan 600
      secondary: '#06b6d4', // Cyan 500
      accent: '#14b8a6'     // Teal 500
    }
  }
}
```

### Step 2: Define CSS Variables

Add theme-specific variables to `app/globals.css`:

```css
/* Ocean theme - Light mode */
[data-theme="ocean"] {
  --background: 0 0% 100%;
  --foreground: 222.2 84% 4.9%;

  --primary: 188 94% 37%;        /* Cyan 600 */
  --primary-foreground: 0 0% 100%;

  --secondary: 189 94% 43%;      /* Cyan 500 */
  --secondary-foreground: 0 0% 100%;

  --accent: 173 80% 40%;         /* Teal 500 */
  --accent-foreground: 0 0% 100%;

  --muted: 210 40% 96.1%;
  --muted-foreground: 215.4 16.3% 46.9%;

  --card: 0 0% 100%;
  --card-foreground: 222.2 84% 4.9%;

  --border: 214.3 31.8% 91.4%;
  --input: 214.3 31.8% 91.4%;
  --ring: 188 94% 37%;
}

/* Ocean theme - Dark mode */
[data-theme="ocean"].dark {
  --background: 222.2 84% 4.9%;
  --foreground: 210 40% 98%;

  --primary: 188 94% 47%;        /* Brighter for dark mode */
  --primary-foreground: 222.2 84% 4.9%;

  --secondary: 189 94% 53%;
  --secondary-foreground: 222.2 84% 4.9%;

  --accent: 173 80% 50%;
  --accent-foreground: 222.2 84% 4.9%;

  --muted: 217.2 32.6% 17.5%;
  --muted-foreground: 215 20.2% 65.1%;

  --card: 222.2 84% 4.9%;
  --card-foreground: 210 40% 98%;

  --border: 217.2 32.6% 17.5%;
  --input: 217.2 32.6% 17.5%;
  --ring: 188 94% 47%;
}
```

### Step 3: Update Type Definitions

Ensure TypeScript recognizes the new theme:

```typescript
// The type is already union type, just add to the config
export type ThemeName = 'default' | 'nature' | 'violet-bloom' | 'ocean';
```

### Step 4: Test Your Theme

```tsx
// Test component
export function TestTheme() {
  const { setTheme } = useTheme()

  return (
    <button onClick={() => setTheme('ocean')}>
      Test Ocean Theme
    </button>
  )
}
```

## Dark Mode

### Light/Dark Mode Toggle

The included theme toggle component:

```tsx
// components/theme/theme-toggle.tsx
'use client'

import { Moon, Sun } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useTheme } from '@/contexts/theme-context'

export function ThemeToggle() {
  const { mode, toggleMode } = useTheme()

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={toggleMode}
      aria-label={`Switch to ${mode === 'light' ? 'dark' : 'light'} mode`}
    >
      {mode === 'light' ? (
        <Moon className="h-5 w-5" />
      ) : (
        <Sun className="h-5 w-5" />
      )}
    </Button>
  )
}
```

### Programmatic Mode Control

```tsx
const { mode, setMode } = useTheme()

// Set specific mode
setMode('dark')
setMode('light')

// Toggle between modes
toggleMode()

// Check current mode
if (mode === 'dark') {
  console.log('Dark mode active')
}
```

### Mode-Specific Styles

```tsx
// Conditional classes based on mode
<div className={mode === 'dark' ? 'bg-gray-900' : 'bg-white'}>
  Mode-aware component
</div>

// CSS with dark: modifier
<div className="bg-white dark:bg-gray-900">
  Automatically switches with mode
</div>
```

### System Preference Detection

To detect and use system color preference:

```tsx
useEffect(() => {
  // Detect system preference
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

  // Set initial mode based on system preference
  if (mediaQuery.matches) {
    setMode('dark')
  }

  // Listen for changes
  const handler = (e: MediaQueryListEvent) => {
    setMode(e.matches ? 'dark' : 'light')
  }

  mediaQuery.addEventListener('change', handler)
  return () => mediaQuery.removeEventListener('change', handler)
}, [])
```

## Troubleshooting

### Theme Not Applying

**Problem**: Theme changes but colors don't update

**Solutions**:
1. Check `data-theme` attribute on `<html>` element
2. Verify CSS variables are defined in `globals.css`
3. Clear browser cache and reload
4. Check for CSS specificity conflicts
5. Inspect element to verify CSS variables are being applied

### Dark Mode Not Working

**Problem**: Dark mode toggle doesn't change appearance

**Solutions**:
1. Verify `.dark` class is added to `<html>` element
2. Check dark mode variables are defined in CSS
3. Ensure Tailwind `dark:` variants are configured
4. Check `darkMode: 'class'` in `tailwind.config.ts`
5. Clear localStorage and retry

### Theme Not Persisting

**Problem**: Theme resets on page reload

**Solutions**:
1. Check localStorage is enabled in browser
2. Verify ThemeProvider is wrapping app correctly
3. Check for localStorage errors in console
4. Test in incognito mode (localStorage works differently)
5. Check browser privacy settings

### Flash of Unstyled Content (FOUC)

**Problem**: Brief flash of wrong theme on load

**Solutions**:
1. ThemeProvider prevents rendering until mounted
2. Add `suppressHydrationWarning` to `<html>` tag
3. Consider using cookies for SSR theme persistence
4. Add inline script in `<head>` to set theme immediately

```tsx
// app/layout.tsx
<html lang="en" suppressHydrationWarning>
```

### CSS Variables Not Working

**Problem**: Custom CSS variables not applying

**Solutions**:
1. Verify HSL format: `hue saturation% lightness%`
2. Use `hsl(var(--variable))` in custom CSS
3. Check variable names match exactly (case-sensitive)
4. Verify variables are in correct scope (`:root`, `.dark`, `[data-theme]`)
5. Test with browser DevTools to inspect computed values

### Type Errors with New Themes

**Problem**: TypeScript errors when adding new theme

**Solutions**:
1. Update `ThemeName` type union
2. Add theme to `themes` Record
3. Restart TypeScript server
4. Check for typos in theme ID
5. Verify all required theme properties are defined

## Best Practices

1. **Consistent naming**: Use kebab-case for theme IDs
2. **Semantic variables**: Use purpose-based variables (--primary, --accent)
3. **Test both modes**: Always test light and dark modes
4. **Accessibility**: Ensure sufficient color contrast (WCAG AA minimum)
5. **Performance**: Use CSS variables for instant theme switching
6. **Type safety**: Keep TypeScript types in sync with theme definitions
7. **Documentation**: Document custom themes and their use cases
8. **Fallbacks**: Provide sensible defaults for all CSS variables
9. **Testing**: Test themes across different browsers
10. **User preference**: Remember user's theme choice

## Related Documentation

- [Frontend Architecture](../frontend/README.md) - Frontend structure and components
- [API Client Guide](./api-client.md) - API integration
- [Testing Guide](./testing.md) - Testing themes and UI components
