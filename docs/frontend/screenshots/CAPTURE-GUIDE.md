# Screenshot Capture Guide

Comprehensive guide for capturing screenshots for the Cobalt Stack frontend documentation.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Screenshot Requirements](#screenshot-requirements)
- [Required Screenshots](#required-screenshots)
- [Capture Methods](#capture-methods)
- [Image Optimization](#image-optimization)
- [Quality Checklist](#quality-checklist)

## Prerequisites

Before capturing screenshots, ensure:

1. **Application Running**
   ```bash
   # Start the frontend application
   cd frontend
   npm run dev
   # Application should be running at http://localhost:3000
   ```

2. **Clean Browser State**
   - Clear cache and cookies
   - Disable browser extensions that might interfere
   - Use incognito/private mode for consistency

3. **Test Data**
   - Use consistent test data for reproducibility
   - Seed the database with sample data if needed
   ```bash
   npm run seed-data
   ```

## Quick Start

```bash
# Automated screenshot capture (recommended)
npm run screenshots

# Or use the Playwright script
npx playwright test screenshots.spec.ts

# Or capture manually using browser tools
```

## Screenshot Requirements

### Technical Specifications

| Specification | Value | Notes |
|--------------|-------|-------|
| **Resolution** | 1920x1080 | Standard desktop viewport |
| **Format** | PNG | Lossless compression |
| **Color Space** | sRGB | Web standard |
| **DPI** | 96 | Standard screen DPI |
| **Zoom Level** | 100% | No browser zoom |
| **File Size** | < 500KB | After optimization |

### Naming Convention

```
[category]-[component-name]-[variant]-[theme].png

Examples:
- theme-cobalt-light.png
- theme-cobalt-dark.png
- component-button-primary-light.png
- component-button-disabled-dark.png
- feature-login-form-light.png
```

### Directory Structure

```
screenshots/
├── themes/                    # Theme showcase screenshots
│   ├── cobalt-light.png
│   ├── cobalt-dark.png
│   ├── nature-light.png
│   ├── nature-dark.png
│   ├── violet-bloom-light.png
│   └── violet-bloom-dark.png
├── components/                # Individual component screenshots
│   ├── button-variants.png
│   ├── button-states.png
│   ├── form-elements.png
│   ├── form-validation.png
│   ├── card-components.png
│   ├── modal-types.png
│   ├── navigation.png
│   ├── table-simple.png
│   ├── table-sortable.png
│   └── list-variants.png
└── features/                  # Feature workflow screenshots
    ├── authentication-flow.png
    ├── login-page.png
    ├── registration-page.png
    ├── password-reset.png
    ├── admin-dashboard.png
    ├── user-profile.png
    └── user-settings.png
```

## Required Screenshots

### Theme Screenshots (6 total)

Each theme needs screenshots in both light and dark variants:

#### 1. Cobalt Theme (Light)
- **Filename**: `themes/cobalt-light.png`
- **URL**: `http://localhost:3000?theme=cobalt&mode=light`
- **Description**: Default Cobalt theme with light mode
- **Highlights**: Blue accent colors, clean typography

#### 2. Cobalt Theme (Dark)
- **Filename**: `themes/cobalt-dark.png`
- **URL**: `http://localhost:3000?theme=cobalt&mode=dark`
- **Description**: Default Cobalt theme with dark mode
- **Highlights**: Dark backgrounds, reduced eye strain

#### 3. Nature Theme (Light)
- **Filename**: `themes/nature-light.png`
- **URL**: `http://localhost:3000?theme=nature&mode=light`
- **Description**: Nature-inspired theme with earthy tones
- **Highlights**: Green accent colors, natural palette

#### 4. Nature Theme (Dark)
- **Filename**: `themes/nature-dark.png`
- **URL**: `http://localhost:3000?theme=nature&mode=dark`
- **Description**: Nature theme optimized for dark mode
- **Highlights**: Deep forest colors, organic feel

#### 5. Violet Bloom Theme (Light)
- **Filename**: `themes/violet-bloom-light.png`
- **URL**: `http://localhost:3000?theme=violet-bloom&mode=light`
- **Description**: Vibrant violet-based theme
- **Highlights**: Purple accent colors, modern aesthetic

#### 6. Violet Bloom Theme (Dark)
- **Filename**: `themes/violet-bloom-dark.png`
- **URL**: `http://localhost:3000?theme=violet-bloom&mode=dark`
- **Description**: Violet theme with dark backgrounds
- **Highlights**: Rich purples, elegant contrast

### Component Screenshots (10+ total)

#### Button Variants
- **Filename**: `components/button-variants.png`
- **URL**: `http://localhost:3000/components/buttons`
- **Capture Area**: Button showcase section
- **Include**: Primary, secondary, danger, ghost, link variants
- **States**: Default, hover (use browser DevTools to trigger)

#### Button States
- **Filename**: `components/button-states.png`
- **URL**: `http://localhost:3000/components/buttons`
- **Capture Area**: Button states section
- **Include**: Default, hover, active, disabled, loading states

#### Form Elements
- **Filename**: `components/form-elements.png`
- **URL**: `http://localhost:3000/components/forms`
- **Capture Area**: Form elements showcase
- **Include**: Text input, textarea, select, checkbox, radio, switch

#### Form Validation
- **Filename**: `components/form-validation.png`
- **URL**: `http://localhost:3000/components/forms`
- **Capture Area**: Form with validation errors
- **Include**: Error states, success states, warning states

#### Card Components
- **Filename**: `components/card-components.png`
- **URL**: `http://localhost:3000/components/cards`
- **Capture Area**: Card variations showcase
- **Include**: Default card, card with header, card with actions, card with image

#### Modal Types
- **Filename**: `components/modal-types.png`
- **URL**: `http://localhost:3000/components/modals`
- **Capture**: Multiple modal examples (composite screenshot)
- **Include**: Information, confirmation, form modals

#### Navigation
- **Filename**: `components/navigation.png`
- **URL**: `http://localhost:3000`
- **Capture Area**: Full navigation bar and sidebar
- **Include**: Logo, menu items, user dropdown, mobile toggle

#### Table (Simple)
- **Filename**: `components/table-simple.png`
- **URL**: `http://localhost:3000/components/tables`
- **Capture Area**: Basic table example
- **Include**: Headers, rows, data cells

#### Table (Sortable)
- **Filename**: `components/table-sortable.png`
- **URL**: `http://localhost:3000/components/tables`
- **Capture Area**: Table with sorting enabled
- **Include**: Sort indicators, sorted column

#### List Variants
- **Filename**: `components/list-variants.png`
- **URL**: `http://localhost:3000/components/lists`
- **Capture Area**: Different list styles
- **Include**: Basic list, list with icons, list with actions

### Feature Screenshots (7+ total)

#### Authentication Flow
- **Filename**: `features/authentication-flow.png`
- **Composite**: Create a flow diagram screenshot
- **Include**: Login → Registration → Email Verification → Password Reset
- **Tool**: Capture each step and create composite

#### Login Page
- **Filename**: `features/login-page.png`
- **URL**: `http://localhost:3000/auth/login`
- **Capture**: Full login page
- **Include**: Form, social login buttons, links

#### Registration Page
- **Filename**: `features/registration-page.png`
- **URL**: `http://localhost:3000/auth/register`
- **Capture**: Full registration form
- **Include**: All form fields, validation

#### Password Reset
- **Filename**: `features/password-reset.png`
- **URL**: `http://localhost:3000/auth/reset-password`
- **Capture**: Password reset flow
- **Include**: Email input, confirmation message

#### Admin Dashboard
- **Filename**: `features/admin-dashboard.png`
- **URL**: `http://localhost:3000/admin`
- **Authentication**: Login as admin user
- **Capture**: Full dashboard view
- **Include**: Statistics, charts, recent activity

#### User Profile
- **Filename**: `features/user-profile.png`
- **URL**: `http://localhost:3000/profile`
- **Authentication**: Login as regular user
- **Capture**: User profile page
- **Include**: Avatar, user info, edit options

#### User Settings
- **Filename**: `features/user-settings.png`
- **URL**: `http://localhost:3000/settings`
- **Authentication**: Login as regular user
- **Capture**: Settings page
- **Include**: Theme selector, preferences, account settings

## Capture Methods

### Method 1: Automated Playwright Script (Recommended)

Create `scripts/capture-screenshots.ts`:

```typescript
import { chromium, Page } from 'playwright';
import { mkdir } from 'fs/promises';
import path from 'path';

const SCREENSHOT_DIR = './docs/frontend/screenshots';
const BASE_URL = 'http://localhost:3000';

interface Screenshot {
  filename: string;
  url: string;
  selector?: string;
  action?: (page: Page) => Promise<void>;
}

const screenshots: Screenshot[] = [
  // Themes
  { filename: 'themes/cobalt-light.png', url: `${BASE_URL}?theme=cobalt&mode=light` },
  { filename: 'themes/cobalt-dark.png', url: `${BASE_URL}?theme=cobalt&mode=dark` },
  { filename: 'themes/nature-light.png', url: `${BASE_URL}?theme=nature&mode=light` },
  { filename: 'themes/nature-dark.png', url: `${BASE_URL}?theme=nature&mode=dark` },
  { filename: 'themes/violet-bloom-light.png', url: `${BASE_URL}?theme=violet-bloom&mode=light` },
  { filename: 'themes/violet-bloom-dark.png', url: `${BASE_URL}?theme=violet-bloom&mode=dark` },

  // Components
  {
    filename: 'components/button-variants.png',
    url: `${BASE_URL}/components/buttons`,
    selector: '[data-testid="button-variants"]'
  },
  {
    filename: 'components/form-elements.png',
    url: `${BASE_URL}/components/forms`,
    selector: '[data-testid="form-elements"]'
  },
  // Add more screenshots...
];

async function captureScreenshots() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1920, height: 1080 },
    deviceScaleFactor: 1,
  });

  const page = await context.newPage();

  // Ensure directories exist
  await mkdir(path.join(SCREENSHOT_DIR, 'themes'), { recursive: true });
  await mkdir(path.join(SCREENSHOT_DIR, 'components'), { recursive: true });
  await mkdir(path.join(SCREENSHOT_DIR, 'features'), { recursive: true });

  for (const screenshot of screenshots) {
    console.log(`Capturing: ${screenshot.filename}`);

    await page.goto(screenshot.url, { waitUntil: 'networkidle' });

    // Wait for any custom action
    if (screenshot.action) {
      await screenshot.action(page);
    }

    // Wait for page to be fully loaded
    await page.waitForTimeout(1000);

    const screenshotPath = path.join(SCREENSHOT_DIR, screenshot.filename);

    if (screenshot.selector) {
      // Capture specific element
      const element = await page.locator(screenshot.selector);
      await element.screenshot({ path: screenshotPath });
    } else {
      // Capture full page
      await page.screenshot({ path: screenshotPath, fullPage: true });
    }

    console.log(`✓ Saved: ${screenshot.filename}`);
  }

  await browser.close();
  console.log('Screenshot capture complete!');
}

captureScreenshots().catch(console.error);
```

**Usage**:
```bash
# Install Playwright if needed
npm install -D playwright

# Run the script
npx ts-node scripts/capture-screenshots.ts
```

### Method 2: Browser DevTools

1. **Open Application**: Navigate to `http://localhost:3000`
2. **Open DevTools**: Press `F12` or `Ctrl+Shift+I` (Windows/Linux) / `Cmd+Option+I` (Mac)
3. **Toggle Device Toolbar**: Press `Ctrl+Shift+M` (Windows/Linux) / `Cmd+Shift+M` (Mac)
4. **Set Viewport**: Select "Responsive" and set to 1920x1080
5. **Capture Screenshot**:
   - Press `Ctrl+Shift+P` (Windows/Linux) / `Cmd+Shift+P` (Mac)
   - Type "Screenshot"
   - Choose "Capture full size screenshot" or "Capture node screenshot"

### Method 3: Browser Extensions

Recommended extensions:

- **Awesome Screenshot** (Chrome/Firefox)
- **Nimbus Screenshot** (Chrome/Firefox)
- **FireShot** (Chrome/Firefox)

**Steps**:
1. Install extension from browser store
2. Navigate to the page you want to capture
3. Click extension icon
4. Choose capture type (full page, visible area, or selected area)
5. Save to the appropriate directory

### Method 4: Manual Screenshots

**macOS**:
```bash
# Full screen
Cmd + Shift + 3

# Selected area
Cmd + Shift + 4
```

**Windows**:
```bash
# Full screen
PrtScn

# Active window
Alt + PrtScn

# Snipping Tool
Win + Shift + S
```

**Linux**:
```bash
# GNOME
gnome-screenshot

# KDE
spectacle

# Command line
import screenshot.png  # ImageMagick
```

## Image Optimization

After capturing screenshots, optimize them for web delivery:

### Using ImageMagick

```bash
# Install ImageMagick
# Ubuntu/Debian: sudo apt-get install imagemagick
# macOS: brew install imagemagick
# Windows: choco install imagemagick

# Optimize single image
convert input.png -strip -quality 85 -resize 1920x1080 output.png

# Batch optimize
find docs/frontend/screenshots -name "*.png" -exec mogrify -strip -quality 85 {} \;
```

### Using pngquant

```bash
# Install pngquant
# Ubuntu/Debian: sudo apt-get install pngquant
# macOS: brew install pngquant
# Windows: choco install pngquant

# Optimize with quality 80-95
pngquant --quality=80-95 --ext .png --force screenshot.png

# Batch optimize
find docs/frontend/screenshots -name "*.png" -exec pngquant --quality=80-95 --ext .png --force {} \;
```

### Using Online Tools

- **TinyPNG**: https://tinypng.com/ (Batch upload up to 20 images)
- **Squoosh**: https://squoosh.app/ (Advanced compression options)
- **Compress PNG**: https://compresspng.com/ (Free batch compression)

### Optimization Script

Create `scripts/optimize-screenshots.sh`:

```bash
#!/bin/bash

SCREENSHOT_DIR="./docs/frontend/screenshots"

echo "Optimizing screenshots..."

# Find all PNG files and optimize
find "$SCREENSHOT_DIR" -name "*.png" | while read -r file; do
  original_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)

  # Optimize with pngquant
  pngquant --quality=80-95 --ext .png --force "$file"

  optimized_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
  savings=$((original_size - optimized_size))
  percent=$((savings * 100 / original_size))

  echo "✓ $(basename "$file"): ${original_size}B → ${optimized_size}B (saved ${percent}%)"
done

echo "Optimization complete!"
```

**Usage**:
```bash
chmod +x scripts/optimize-screenshots.sh
./scripts/optimize-screenshots.sh
```

## Quality Checklist

Before committing screenshots, verify:

### Technical Quality
- [ ] Resolution is 1920x1080
- [ ] File format is PNG
- [ ] File size is under 500KB
- [ ] No browser UI visible (unless intentional)
- [ ] No personal information visible
- [ ] Consistent zoom level (100%)

### Visual Quality
- [ ] Text is readable and sharp
- [ ] Colors are accurate
- [ ] No artifacts or compression issues
- [ ] Proper lighting/contrast
- [ ] UI elements are properly aligned

### Content Quality
- [ ] Correct theme applied
- [ ] Test data is appropriate
- [ ] All required elements visible
- [ ] No error messages (unless demonstrating errors)
- [ ] States are correct (hover, active, etc.)

### Documentation
- [ ] Filename follows naming convention
- [ ] Placed in correct directory
- [ ] Referenced in README.md
- [ ] Description added if needed
- [ ] Alt text provided for accessibility

## Troubleshooting

### Application Won't Start
```bash
# Check if port 3000 is in use
lsof -i :3000  # macOS/Linux
netstat -ano | findstr :3000  # Windows

# Kill process if needed
kill -9 <PID>

# Restart application
npm run dev
```

### Screenshots Are Blurry
- Ensure browser zoom is at 100%
- Check device pixel ratio in DevTools
- Use high DPI settings if available
- Capture at higher resolution and downscale

### Colors Don't Match
- Use sRGB color space
- Disable browser color management
- Check monitor color calibration
- Use consistent browser for all captures

### File Sizes Too Large
- Run optimization scripts
- Reduce color depth if acceptable
- Check for unnecessary metadata
- Consider JPEG for photographic content (not UI)

## Automation Integration

### GitHub Actions Workflow

Create `.github/workflows/screenshots.yml`:

```yaml
name: Update Screenshots

on:
  workflow_dispatch:  # Manual trigger
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday

jobs:
  capture-screenshots:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install dependencies
        run: |
          cd frontend
          npm ci

      - name: Install Playwright
        run: npx playwright install --with-deps

      - name: Start application
        run: |
          cd frontend
          npm run dev &
          sleep 10

      - name: Capture screenshots
        run: npx ts-node scripts/capture-screenshots.ts

      - name: Optimize images
        run: |
          sudo apt-get install -y pngquant
          ./scripts/optimize-screenshots.sh

      - name: Commit and push
        run: |
          git config --local user.email "action@github.com"
          git config --local user.name "GitHub Action"
          git add docs/frontend/screenshots/
          git commit -m "chore: update screenshots" || echo "No changes"
          git push
```

## Related Documentation

- [Frontend README](../README.md)
- [Theme Documentation](../themes.md)
- [Component Guide](../../guides/components.md)
- [Testing Guide](../testing.md)

---

**Status**: Ready for screenshot capture
**Last Updated**: 2025-10-27
**Maintainer**: Cobalt Stack Team
