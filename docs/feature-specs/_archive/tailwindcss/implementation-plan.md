# TailwindCSS Implementation Plan

## Overview

This plan implements TailwindCSS for the VstKit React UI, replacing all component CSS files with utility-first styling. The implementation follows 4 phases with 18 discrete steps.

**Estimated Total Time:** 3.5 hours  
**Prerequisites:** Node.js, npm, existing UI codebase  
**Reference:** [Low-Level Design](./low-level-design-tailwindcss.md) | [User Stories](./user-stories.md)

---

## Phase 1: Foundation Setup

### Step 1.1: Install Tailwind Dependencies
**File:** `ui/package.json`  
**Action:** Install TailwindCSS, PostCSS, and Autoprefixer

```bash
cd ui
npm install -D tailwindcss@^3.4.17 postcss@^8.4.49 autoprefixer@^10.4.20
```

**Why:** Core dependencies for Tailwind's build pipeline  
**Dependencies:** None  
**Risk:** Low

---

### Step 1.2: Create PostCSS Configuration
**File:** `ui/postcss.config.js` (new file)  
**Action:** Create PostCSS config to wire up Tailwind and Autoprefixer

```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
};
```

**Why:** Vite uses PostCSS automatically when config exists  
**Dependencies:** Step 1.1  
**Risk:** Low

---

### Step 1.3: Create Tailwind Configuration
**File:** `ui/tailwind.config.js` (new file)  
**Action:** Create Tailwind config with custom theme matching current design

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
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
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['SF Mono', 'Monaco', 'Courier New', 'monospace'],
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
    },
  },
  plugins: [],
};
```

**Why:** Custom theme tokens ensure visual parity with current design  
**Dependencies:** Step 1.1  
**Risk:** Low

---

### Step 1.4: Update Entry CSS with Tailwind Directives
**File:** `ui/src/index.css`  
**Action:** Replace current content with Tailwind directives and base layer styles

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
  /* Range input thumb - requires vendor prefixes */
  .slider-thumb::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    @apply w-[18px] h-[18px] rounded-full bg-accent cursor-pointer transition-colors duration-150;
  }

  .slider-thumb::-webkit-slider-thumb:hover {
    @apply bg-accent-light;
  }

  .slider-thumb::-moz-range-thumb {
    @apply w-[18px] h-[18px] rounded-full bg-accent cursor-pointer border-none transition-colors duration-150;
  }

  .slider-thumb::-moz-range-thumb:hover {
    @apply bg-accent-light;
  }
}
```

**Why:** Entry point for Tailwind; base layer handles global resets  
**Dependencies:** Steps 1.2, 1.3  
**Risk:** Low

---

### Step 1.5: Verify Build Pipeline
**Action:** Run build to confirm Tailwind processes correctly

```bash
npm run build
npm run dev  # Verify hot reload works
```

**Why:** Catch configuration issues before migrating components  
**Dependencies:** Step 1.4  
**Risk:** Low  
**Verification:** Build succeeds, dev server starts, page loads

---

## Phase 2: Global Styles Migration

### Step 2.1: Migrate App Component Styles
**File:** `ui/src/App.tsx`  
**Action:** Replace CSS class references with Tailwind utilities

**Current classes to migrate:**
| CSS Class | Tailwind Replacement |
|-----------|---------------------|
| `.app` | `flex flex-col min-h-full` |
| `.app-header` | `p-8 text-center bg-gradient-to-br from-plugin-surface to-plugin-dark border-b-2 border-plugin-border` |
| `.app-header h1` | `text-2xl mb-2 bg-gradient-to-r from-accent to-accent-light bg-clip-text text-transparent` |
| `.app-header p` | `text-gray-500 text-sm` |
| `.app-main` | `flex-1 p-8 max-w-3xl w-full mx-auto` |
| `.app-main section` | `mb-8` |
| `.app-main h2` | `text-xl mb-4 text-gray-200 border-b-2 border-plugin-border pb-2` |
| `.app-footer` | `p-4 text-center bg-plugin-surface border-t border-plugin-border text-gray-500 text-sm` |

**Why:** App.tsx is the root component; validates Tailwind works in JSX  
**Dependencies:** Step 1.5  
**Risk:** Low

---

### Step 2.2: Delete App.css
**File:** `ui/src/App.css`  
**Action:** Delete file, remove import from App.tsx

**Why:** No longer needed after migration  
**Dependencies:** Step 2.1  
**Risk:** Low  
**Verification:** UI renders correctly without App.css

---

## Phase 3: Component Migration

### Step 3.1: Migrate Meter Component
**File:** `ui/src/components/Meter.tsx`  
**Action:** Replace all CSS classes with Tailwind utilities

**Key mappings:**
| CSS Class | Tailwind |
|-----------|----------|
| `.meter` | `flex flex-col gap-2 p-4 bg-black/30 rounded-lg font-sans` |
| `.meter-header` | `flex items-center justify-between gap-2` |
| `.meter-label` | `text-xs font-semibold text-gray-500 uppercase tracking-wide` |
| `.meter-clip-indicator` | `text-[10px] font-bold text-white bg-meter-clip px-2 py-0.5 rounded cursor-pointer select-none animate-clip-pulse hover:bg-meter-clip-dark active:scale-95` |
| `.meter-channel` | `flex items-center gap-2` |
| `.meter-channel-label` | `w-4 text-[11px] font-semibold text-gray-300 text-center` |
| `.meter-bar-container` | `flex-1 h-6 relative` |
| `.meter-bar-bg` | `w-full h-full bg-[#222] rounded relative overflow-hidden transition-shadow duration-100` |
| `.meter-bar-bg.clipped` | Add `shadow-[inset_0_0_8px_rgba(255,23,68,0.8)]` |
| `.meter-bar-rms` | `absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe to-meter-safe-light transition-[width] duration-100` |
| `.meter-bar-peak` | `absolute left-0 top-0 h-full bg-gradient-to-r from-meter-safe via-meter-warning to-orange-500 opacity-60 transition-[width] duration-50` |
| `.meter-value` | `w-[60px] text-[11px] font-mono text-gray-300 text-right transition-colors duration-100` |
| `.meter-value.clipped` | Add `text-meter-clip font-semibold` |

**Why:** Most complex component; validates all Tailwind features work  
**Dependencies:** Step 1.5  
**Risk:** Medium (gradients, animations, conditional classes)

---

### Step 3.2: Delete Meter.css
**File:** `ui/src/components/Meter.css`  
**Action:** Delete file, remove import from Meter.tsx

**Dependencies:** Step 3.1  
**Risk:** Low

---

### Step 3.3: Migrate ParameterSlider Component
**File:** `ui/src/components/ParameterSlider.tsx`  
**Action:** Replace CSS classes with Tailwind utilities

**Key mappings:**
| CSS Class | Tailwind |
|-----------|----------|
| `.parameter-slider` | `p-4 border border-plugin-border rounded-lg bg-plugin-surface mb-4` |
| `.parameter-slider.loading` | Add `text-gray-500 italic` |
| `.parameter-slider.error` | `text-red-400 border-red-400` |
| `.parameter-header` | `flex justify-between items-center mb-2` |
| `.parameter-header label` | `font-semibold text-gray-200` |
| `.parameter-value` | `font-mono text-accent text-sm` |
| `.slider` | `w-full h-1.5 rounded-sm bg-plugin-border outline-none appearance-none slider-thumb` |

**Why:** User-facing control; validates range input styling  
**Dependencies:** Step 1.4 (slider-thumb component class)  
**Risk:** Medium (slider thumb pseudo-elements)

---

### Step 3.4: Delete ParameterSlider.css
**File:** `ui/src/components/ParameterSlider.css`  
**Action:** Delete file, remove import

**Dependencies:** Step 3.3  
**Risk:** Low

---

### Step 3.5: Migrate ParameterToggle Component
**File:** `ui/src/components/ParameterToggle.tsx`  
**Action:** Replace CSS classes with Tailwind utilities

**Key mappings:**
| CSS Class | Tailwind |
|-----------|----------|
| `.parameter-toggle` | `flex items-center justify-between p-4 border border-plugin-border rounded-lg bg-plugin-surface mb-4` |
| `.parameter-toggle.loading` | Add `text-gray-500 italic` |
| `.parameter-toggle.error` | `text-red-400 border-red-400` |
| `.parameter-toggle label` | `font-semibold text-gray-200` |
| `.toggle-button` | `relative w-[50px] h-[26px] rounded-full border-none cursor-pointer transition-colors duration-200 outline-none` |
| `.toggle-button.off` | `bg-gray-600` |
| `.toggle-button.on` | `bg-accent` |
| `.toggle-button:hover.off` | `hover:bg-gray-500` |
| `.toggle-button:hover.on` | `hover:bg-accent-light` |
| `.toggle-indicator` | `absolute top-[3px] w-5 h-5 rounded-full bg-white transition-[left] duration-200` |
| `.toggle-button.off .toggle-indicator` | `left-[3px]` |
| `.toggle-button.on .toggle-indicator` | `left-[27px]` |

**Why:** Simpler toggle validates state-based styling  
**Dependencies:** Step 1.5  
**Risk:** Low

---

### Step 3.6: Delete ParameterToggle.css
**File:** `ui/src/components/ParameterToggle.css`  
**Action:** Delete file, remove import

**Dependencies:** Step 3.5  
**Risk:** Low

---

### Step 3.7: Migrate ResizeControls Component
**File:** `ui/src/components/ResizeControls.tsx`  
**Action:** Replace CSS classes with Tailwind utilities

**Key mappings:**
| CSS Class | Tailwind |
|-----------|----------|
| `.resize-controls` | `p-5 bg-black/5 rounded-lg` |
| `.resize-controls h3` | `m-0 mb-4 text-sm font-semibold uppercase tracking-wide text-black/70` |
| `.resize-presets` | `grid grid-cols-2 gap-2.5 mb-4` |
| `.resize-button` | `flex flex-col items-center justify-center p-3 bg-white border border-gray-300 rounded-md cursor-pointer transition-all duration-200 font-medium text-gray-800 hover:bg-gray-100 hover:border-blue-500 hover:-translate-y-px hover:shadow-md disabled:opacity-50 disabled:cursor-not-allowed` |
| `.resize-button .size-label` | `text-[11px] text-gray-500 mt-1` |
| `.resize-status` | `px-3 py-2 rounded text-sm text-center bg-black/5 text-gray-500` |
| `.resize-status.success` | `bg-green-500/10 text-green-600` |
| `.resize-status.error` | `bg-red-500/10 text-red-500` |

**Why:** Button grid validates responsive layout utilities  
**Dependencies:** Step 1.5  
**Risk:** Low

**Note:** This component uses light-mode colors (white bg, black text) — different from the dark plugin theme. Verify this is intentional or adjust to match dark theme.

---

### Step 3.8: Delete ResizeControls.css
**File:** `ui/src/components/ResizeControls.css`  
**Action:** Delete file, remove import

**Dependencies:** Step 3.7  
**Risk:** Low

---

### Step 3.9: Migrate ResizeHandle Component
**File:** `ui/src/components/ResizeHandle.tsx`  
**Action:** Replace CSS classes with Tailwind utilities

**Key mappings:**
| CSS Class | Tailwind |
|-----------|----------|
| `.resize-handle` | `fixed bottom-0 right-0 w-6 h-6 flex items-center justify-center cursor-nwse-resize z-[9999] select-none border-none bg-transparent p-0 transition-colors duration-150 hover:bg-white/5` |
| `.resize-handle.dragging` | Add `bg-white/10` |
| `.resize-grip-icon` | `text-white/30 transition-colors duration-150` |
| `.resize-handle:hover .resize-grip-icon` | Use group: `group-hover:text-white/60` |
| `.resize-handle.dragging .resize-grip-icon` | Conditional: `text-white/80` when dragging |

**Why:** Fixed positioning, z-index, cursor styles  
**Dependencies:** Step 1.5  
**Risk:** Low

---

### Step 3.10: Delete ResizeHandle.css
**File:** `ui/src/components/ResizeHandle.css`  
**Action:** Delete file, remove import

**Dependencies:** Step 3.9  
**Risk:** Low

---

### Step 3.11: Migrate LatencyMonitor Component
**File:** `ui/src/components/LatencyMonitor.tsx`  
**Action:** Replace CSS classes with Tailwind utilities

**Key mappings:**
| CSS Class | Tailwind |
|-----------|----------|
| `.latency-monitor` | `p-4 border border-plugin-border rounded-lg bg-plugin-surface mb-4` |
| `.latency-monitor h3` | `m-0 mb-3 text-base text-gray-200 font-semibold` |
| `.metrics` | `grid grid-cols-2 gap-2` |
| `.metric` | `flex justify-between p-2 bg-plugin-dark rounded` |
| `.metric .label` | `text-gray-500 text-sm` |
| `.metric .value` | `text-accent font-mono text-sm font-semibold` |
| `.status` | `mt-3 text-center text-sm font-semibold` |
| `.status .good` | `text-green-400` |
| `.status .warning` | `text-yellow-400` |
| `.status .poor` | `text-red-400` |

**Why:** Grid layout, status colors  
**Dependencies:** Step 1.5  
**Risk:** Low

---

### Step 3.12: Delete LatencyMonitor.css
**File:** `ui/src/components/LatencyMonitor.css`  
**Action:** Delete file, remove import

**Dependencies:** Step 3.11  
**Risk:** Low

---

## Phase 4: Optimization & Documentation

### Step 4.1: Verify Production Bundle Size
**Action:** Build and check CSS output size

```bash
npm run build
ls -la dist/assets/*.css
# Target: < 10KB gzipped
gzip -c dist/assets/*.css | wc -c
```

**Why:** Ensure Tailwind purge is working correctly  
**Dependencies:** All Phase 3 steps  
**Risk:** Low

---

### Step 4.2: Install Prettier Tailwind Plugin (Optional)
**File:** `ui/package.json`  
**Action:** Add Prettier plugin for class sorting

```bash
npm install -D prettier-plugin-tailwindcss@^0.6.11
```

Update `.prettierrc` or `prettier.config.js` if needed:
```json
{
  "plugins": ["prettier-plugin-tailwindcss"]
}
```

**Why:** Consistent class ordering improves readability  
**Dependencies:** All Phase 3 steps  
**Risk:** Low

---

### Step 4.3: Run Full Lint and Format
**Action:** Ensure code passes all checks

```bash
npm run lint
npm run format
npm run typecheck
```

**Why:** Validate no issues introduced  
**Dependencies:** Step 4.2  
**Risk:** Low

---

### Step 4.4: Test in WebView Context
**Action:** Build full plugin UI and test in desktop app

```bash
cargo xtask desktop --build-ui
```

**Why:** Validate styling works in WKWebView (WebKit) context  
**Dependencies:** Step 4.1  
**Risk:** Medium (WebKit CSS compatibility)

---

### Step 4.5: Update Documentation
**File:** `ui/README.md` or new `CONTRIBUTING.md`  
**Action:** Add Tailwind usage guidelines

**Content to add:**
- How to use custom theme tokens
- Component styling conventions
- Link to Tailwind docs

**Why:** Enable other developers to maintain consistency  
**Dependencies:** Step 4.4  
**Risk:** Low

---

## Verification Checklist

After completing all steps:

- [ ] `npm run build` succeeds
- [ ] `npm run dev` hot-reloads Tailwind changes
- [ ] `npm run lint` passes
- [ ] `npm run typecheck` passes
- [ ] Production CSS < 10KB gzipped
- [ ] No CSS files in `ui/src/components/`
- [ ] No `App.css` file
- [ ] Visual parity with original design
- [ ] `cargo xtask desktop --build-ui` works
- [ ] Meter animations work (clip pulse)
- [ ] Slider thumb styling works
- [ ] Toggle indicator positioning works

---

## Risk Summary

| Step | Risk Level | Mitigation |
|------|------------|------------|
| 3.1 Meter | Medium | Test animations/gradients before deleting CSS |
| 3.3 ParameterSlider | Medium | Verify slider thumb in multiple browsers |
| 4.4 WebView Test | Medium | Test early, adjust if WebKit issues |
| All others | Low | Standard Tailwind patterns |

---

## Rollback Plan

1. **Git branch**: All work in feature branch `feature/tailwindcss`
2. **Keep CSS until verified**: Don't delete CSS files until visual parity confirmed
3. **Incremental commits**: Commit after each component migration
4. **Easy revert**: `git checkout main -- ui/src/` restores all files
