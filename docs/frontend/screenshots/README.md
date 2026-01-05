# Frontend Screenshots

Visual documentation of UI components, themes, and features in Cobalt Stack.

## Table of Contents

- [Overview](#overview)
- [Themes](#themes)
- [Components](#components)
- [Features](#features)
- [Contributing Screenshots](#contributing-screenshots)

## Overview

This directory contains visual documentation to help developers and designers understand the UI implementation. Screenshots are organized by category and include both light and dark theme variants.

## Themes

Visual examples of theme variations:

### Light Theme

> **Note**: Screenshots to be added

Light theme provides a clean, bright interface optimized for daylight viewing.

### Dark Theme

> **Note**: Screenshots to be added

Dark theme reduces eye strain and provides better visibility in low-light conditions.

### Theme Comparison

| Component | Light Theme | Dark Theme |
|-----------|-------------|------------|
| Dashboard | *Coming soon* | *Coming soon* |
| Navigation | *Coming soon* | *Coming soon* |
| Forms | *Coming soon* | *Coming soon* |
| Buttons | *Coming soon* | *Coming soon* |

## Components

Visual reference for individual UI components:

### Layout Components

- **Navigation Bar** - *Screenshots coming soon*
- **Sidebar** - *Screenshots coming soon*
- **Footer** - *Screenshots coming soon*

### UI Components

- **Buttons** - *Screenshots coming soon*
  - Primary, secondary, danger variants
  - Sizes: small, medium, large
  - States: default, hover, active, disabled

- **Forms** - *Screenshots coming soon*
  - Input fields
  - Textareas
  - Select dropdowns
  - Checkboxes and radio buttons

- **Cards** - *Screenshots coming soon*
  - Default card
  - Card with header
  - Card with actions
  - Card with media

- **Modals** - *Screenshots coming soon*
  - Information modal
  - Confirmation modal
  - Form modal

### Data Display

- **Tables** - *Screenshots coming soon*
  - Simple table
  - Sortable table
  - Paginated table

- **Lists** - *Screenshots coming soon*
  - Basic list
  - List with icons
  - List with actions

## Features

Complete feature demonstrations:

### Authentication

- **Login Page** - *Screenshot coming soon*
- **Registration Page** - *Screenshot coming soon*
- **Password Reset** - *Screenshot coming soon*

### Dashboard

- **Overview Dashboard** - *Screenshot coming soon*
- **Statistics View** - *Screenshot coming soon*
- **Activity Feed** - *Screenshot coming soon*

### User Management

- **User List** - *Screenshot coming soon*
- **User Profile** - *Screenshot coming soon*
- **User Settings** - *Screenshot coming soon*

## Contributing Screenshots

### Guidelines

When adding screenshots to this directory:

1. **File Naming Convention**
   ```
   [component-name]-[variant]-[theme].png

   Examples:
   button-primary-light.png
   button-primary-dark.png
   dashboard-overview-light.png
   form-validation-dark.png
   ```

2. **Screenshot Requirements**
   - Use consistent browser window size (1920x1080 recommended)
   - Capture at 100% zoom level
   - Include relevant UI context
   - Show both light and dark themes
   - Demonstrate different states (hover, active, disabled)

3. **Directory Structure**
   ```
   screenshots/
   ├── themes/
   │   ├── light/
   │   └── dark/
   ├── components/
   │   ├── buttons/
   │   ├── forms/
   │   ├── cards/
   │   └── modals/
   └── features/
       ├── auth/
       ├── dashboard/
       └── user-management/
   ```

4. **Screenshot Descriptions**
   - Add a brief description in this README
   - Include component state (if applicable)
   - Note any special behaviors or interactions

### How to Capture Screenshots

```bash
# Using Playwright (recommended)
npm run screenshot

# Manual capture
# 1. Navigate to component in browser
# 2. Open DevTools (F12)
# 3. Toggle device toolbar (Ctrl+Shift+M)
# 4. Set viewport to 1920x1080
# 5. Capture screenshot (Ctrl+Shift+P -> "Screenshot")
```

### Automation

```typescript
// scripts/capture-screenshots.ts
import { chromium } from 'playwright';

async function captureScreenshots() {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1920, height: 1080 } });

  // Capture light theme
  await page.goto('http://localhost:3000');
  await page.screenshot({ path: 'screenshots/components/dashboard-light.png' });

  // Toggle dark theme
  await page.click('[data-testid="theme-toggle"]');
  await page.screenshot({ path: 'screenshots/components/dashboard-dark.png' });

  await browser.close();
}

captureScreenshots();
```

## Related Resources

- [Frontend Documentation](../README.md)
- [Component Documentation](../../guides/component-guide.md)
- [Design System](../../guides/design-system.md)
- [Contributing Guidelines](../../contributing/README.md)

---

**Note**: This screenshot gallery is under development. Screenshots will be added progressively as components are documented.
