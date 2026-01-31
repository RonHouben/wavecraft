# TailwindCSS Implementation — Low-Level Design

## 1. Overview

This document defines the technical design for implementing TailwindCSS in the VstKit React UI, replacing the current CSS file approach with utility-first styling.

### 1.1 Goals

- **Consistency**: Unified styling approach across all components
- **Maintainability**: No scattered CSS files; styles live with components
- **Performance**: Purged CSS bundle < 10KB gzipped
- **Developer Experience**: Faster iteration with utility classes and excellent LLM/docs support

### 1.2 Non-Goals

- Theme switching (dark-only UI)
- Tailwind UI component library
- CSS-in-JS solutions

---

## 2. Architecture

### 2.1 Build Pipeline Integration

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Vite Build Pipeline                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  src/index.css ──► PostCSS ──► Tailwind ──► Autoprefixer ──► CSS   │
│       │                │                                            │
│       │                ├── tailwind.config.js (theme, content)      │
│       │                └── postcss.config.js (plugins)              │
│       │                                                             │
│       └── @tailwind base;                                           │
│           @tailwind components;                                     │
│           @tailwind utilities;                                      │
│                                                                     │
│  src/**/*.tsx ──► Vite ──► esbuild ──► JS bundle                   │
│       │                                                             │
│       └── className="bg-plugin-dark text-gray-200 ..."             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 File Structure (After Migration)

```
ui/
├── postcss.config.js          # NEW: PostCSS configuration
├── tailwind.config.js         # NEW: Tailwind configuration + theme
├── package.json               # Updated: new dependencies
├── vite.config.ts             # Unchanged (Vite has native PostCSS)
└── src/
    ├── index.css              # MODIFIED: Tailwind directives only
    ├── App.tsx                # MODIFIED: utility classes
    ├── App.css                # DELETED
    └── components/
        ├── Meter.tsx          # MODIFIED: utility classes
        ├── Meter.css          # DELETED
        ├── ParameterSlider.tsx
        ├── ParameterSlider.css # DELETED
        ├── ParameterToggle.tsx
        ├── ParameterToggle.css # DELETED
        ├── ResizeControls.tsx
        ├── ResizeControls.css  # DELETED
        ├── ResizeHandle.tsx
        ├── ResizeHandle.css    # DELETED
        ├── LatencyMonitor.tsx
        └── LatencyMonitor.css  # DELETED
```

---

## 3. Configuration Files

### 3.1 PostCSS Configuration

**File**: `ui/postcss.config.js`

```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
};
```

### 3.2 Tailwind Configuration

**File**: `ui/tailwind.config.js`

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        // Plugin UI color palette
        plugin: {
          dark: '#1a1a1a',
          surface: '#2a2a2a',
          border: '#444444',
        },
        accent: {
          DEFAULT: '#4a9eff',
          light: '#6bb0ff',
        },
        meter: {
          safe: '#4caf50',
          'safe-light': '#8bc34a',
          warning: '#ffeb3b',
          clip: '#ff1744',
          'clip-dark': '#d50000',
        },
        text: {
          primary: '#e0e0e0',
          secondary: '#999999',
          muted: '#666666',
        },
      },
      fontFamily: {
        sans: [
          '-apple-system',
          'BlinkMacSystemFont',
          'Segoe UI',
          'Roboto',
          'Oxygen',
          'Ubuntu',
          'Cantarell',
          'Fira Sans',
          'Droid Sans',
          'Helvetica Neue',
          'sans-serif',
        ],
        mono: ['Courier New', 'monospace'],
      },
      animation: {
        'clip-pulse': 'clip-pulse 0.5s ease-in-out infinite alternate',
      },
      keyframes: {
        'clip-pulse': {
          from: { opacity: '1' },
          to: { opacity: '0.7' },
        },
      },
      backgroundImage: {
        'header-gradient': 'linear-gradient(135deg, #2a2a2a 0%, #1a1a1a 100%)',
        'meter-gradient': 'linear-gradient(to right, #4caf50, #8bc34a)',
        'accent-gradient': 'linear-gradient(90deg, #4a9eff 0%, #6bb0ff 100%)',
      },
    },
  },
  plugins: [],
};
```

### 3.3 Entry CSS File

**File**: `ui/src/index.css`

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  body {
    @apply m-0 p-0 overflow-hidden;
  }

  #root {
    @apply w-full h-screen overflow-y-auto;
  }

  * {
    @apply box-border;
  }
}

@layer components {
  /* Range input thumb styling - requires vendor prefixes */
  .slider-thumb::-webkit-slider-thumb {
    @apply appearance-none w-[18px] h-[18px] rounded-full bg-accent cursor-pointer;
    @apply transition-colors duration-150 ease-out;
  }

  .slider-thumb::-webkit-slider-thumb:hover {
    @apply bg-accent-light;
  }

  .slider-thumb::-moz-range-thumb {
    @apply w-[18px] h-[18px] rounded-full bg-accent cursor-pointer border-none;
    @apply transition-colors duration-150 ease-out;
  }

  .slider-thumb::-moz-range-thumb:hover {
    @apply bg-accent-light;
  }
}
```

---

## 4. Component Migration Patterns

### 4.1 Migration Strategy

Each component follows this pattern:

1. **Identify CSS selectors** in the component's CSS file
2. **Map to Tailwind utilities** or custom theme tokens
3. **Apply classes** directly in JSX
4. **Verify visual parity** in browser
5. **Delete CSS file** and remove import

### 4.2 Example: Meter Component

**Before** (`Meter.css` excerpt):
```css
.meter {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 8px;
}

.meter-clip-indicator {
  font-size: 10px;
  font-weight: 700;
  color: #fff;
  background: #ff1744;
  padding: 2px 8px;
  border-radius: 3px;
  animation: clip-pulse 0.5s ease-in-out infinite alternate;
}
```

**After** (`Meter.tsx` excerpt):
```tsx
<div className="flex flex-col gap-2 p-4 bg-black/30 rounded-lg font-sans">
  {/* ... */}
  <button
    className="text-[10px] font-bold text-white bg-meter-clip 
               px-2 py-0.5 rounded cursor-pointer select-none
               animate-clip-pulse hover:bg-meter-clip-dark 
               active:scale-95"
    onClick={onClipReset}
  >
    CLIP
  </button>
</div>
```

### 4.3 CSS-to-Tailwind Mapping Reference

| CSS Property | Tailwind Utility |
|--------------|------------------|
| `display: flex` | `flex` |
| `flex-direction: column` | `flex-col` |
| `gap: 8px` | `gap-2` |
| `padding: 16px` | `p-4` |
| `background: #1a1a1a` | `bg-plugin-dark` |
| `border-radius: 8px` | `rounded-lg` |
| `font-size: 12px` | `text-xs` |
| `font-weight: 600` | `font-semibold` |
| `color: #999` | `text-text-secondary` |
| `text-transform: uppercase` | `uppercase` |
| `letter-spacing: 0.5px` | `tracking-wide` |

### 4.4 Complex Patterns

#### Gradient Backgrounds
```tsx
// Header gradient
<header className="bg-header-gradient border-b-2 border-plugin-border">

// Meter bar gradient
<div className="bg-meter-gradient" style={{ width: `${percent}%` }} />
```

#### Conditional Classes
```tsx
// Loading/error states
<div className={`p-4 rounded-lg border ${
  isLoading ? 'text-text-muted italic' :
  hasError ? 'text-red-400 border-red-400' :
  'border-plugin-border bg-plugin-surface'
}`}>
```

#### Pseudo-element Effects (Meter Clip Glow)
```tsx
// Use Tailwind's shadow utilities
<div className={`rounded ${isClipped ? 'shadow-[inset_0_0_8px_rgba(255,23,68,0.8)]' : ''}`}>
```

---

## 5. Dependencies

### 5.1 New Dependencies

```json
{
  "devDependencies": {
    "tailwindcss": "^3.4.17",
    "postcss": "^8.4.49",
    "autoprefixer": "^10.4.20"
  }
}
```

### 5.2 Optional (Recommended)

```json
{
  "devDependencies": {
    "prettier-plugin-tailwindcss": "^0.6.11"
  }
}
```

This plugin auto-sorts Tailwind classes in a consistent order.

---

## 6. Implementation Phases

### Phase 1: Foundation (US-1, US-2)
**Estimated: 30 minutes**

1. Install dependencies: `npm install -D tailwindcss postcss autoprefixer`
2. Initialize Tailwind: `npx tailwindcss init -p`
3. Configure `tailwind.config.js` with custom theme
4. Update `index.css` with Tailwind directives
5. Verify build works: `npm run build`

### Phase 2: Global Styles (US-3)
**Estimated: 30 minutes**

1. Migrate `App.css` styles to `App.tsx`
2. Add base layer styles to `index.css`
3. Delete `App.css`
4. Visual verification

### Phase 3: Component Migration (US-4, US-5, US-6)
**Estimated: 2 hours**

| Component | Complexity | Notes |
|-----------|------------|-------|
| Meter | High | Custom animation, gradients |
| ParameterSlider | Medium | Range input pseudo-elements |
| ParameterToggle | Low | Simple checkbox styling |
| ResizeControls | Low | Button grid |
| ResizeHandle | Low | Drag indicator |
| LatencyMonitor | Low | Debug display |

Order: Start with `Meter` (most complex), then work down.

### Phase 4: Optimization & Docs (US-7, US-8)
**Estimated: 30 minutes**

1. Verify production bundle size
2. Add Prettier plugin (optional)
3. Update README with Tailwind usage guidelines

---

## 7. Testing Strategy

### 7.1 Visual Verification

For each migrated component:

1. **Before screenshot**: Capture current appearance
2. **After screenshot**: Capture with Tailwind classes
3. **Diff check**: Ensure visual parity

### 7.2 Build Verification

```bash
# Development build
npm run dev
# Verify hot reload works with class changes

# Production build
npm run build
# Check dist/assets/*.css size (target: <10KB gzipped)

# Type check
npm run typecheck
# Ensure no TypeScript errors
```

### 7.3 Browser Testing

Test in WebView context:
```bash
cargo xtask desktop --build-ui
# Verify UI renders correctly in wry WebView
```

---

## 8. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Visual regressions | High | Side-by-side comparison before deleting CSS |
| Slider thumb styling breaks | Medium | Keep `@layer components` fallback for pseudo-elements |
| Bundle size increases | Low | Tailwind purges unused; monitor with `npm run build` |
| WebView CSS compatibility | Medium | WKWebView uses modern WebKit; low risk |

---

## 9. Rollback Plan

If issues arise mid-migration:

1. **Git revert**: All changes in single feature branch
2. **Keep CSS files**: Don't delete until full visual verification
3. **Parallel styles**: Can temporarily have both CSS imports and Tailwind classes

---

## 10. Success Criteria

- [ ] All 6 component CSS files deleted
- [ ] `App.css` deleted
- [ ] `npm run build` succeeds
- [ ] Production CSS < 10KB gzipped
- [ ] UI visually identical to before migration
- [ ] `npm run lint` passes
- [ ] Works in desktop WebView (`cargo xtask desktop`)

---

## 11. References

- [Tailwind CSS Docs](https://tailwindcss.com/docs)
- [Vite + Tailwind Guide](https://tailwindcss.com/docs/guides/vite)
- [Tailwind Configuration](https://tailwindcss.com/docs/configuration)
- User Stories: [user-stories.md](./user-stories.md)
