# TailwindCSS Implementation — User Stories

## Overview

This document captures the user stories for implementing TailwindCSS in the VstKit React UI. The goal is to replace the current CSS file approach with Tailwind's utility-first methodology.

---

## User Stories

### US-1: Install and Configure TailwindCSS

**As a** plugin UI developer  
**I want** TailwindCSS installed and configured in the Vite build pipeline  
**So that** I can use utility classes in React components immediately

#### Acceptance Criteria
- [ ] TailwindCSS v3.x installed via npm
- [ ] PostCSS configured with Tailwind and Autoprefixer
- [ ] `tailwind.config.js` created with appropriate content paths
- [ ] Tailwind directives added to entry CSS file
- [ ] `npm run build` succeeds with Tailwind processing
- [ ] `npm run dev` hot-reloads Tailwind changes

#### Notes
- Use Vite's native PostCSS support (no extra plugins needed)
- Configure content paths to scan `./src/**/*.{ts,tsx}`

---

### US-2: Define Plugin UI Design Tokens

**As a** plugin UI developer  
**I want** a custom Tailwind theme with audio plugin-appropriate design tokens  
**So that** the UI has consistent colors, spacing, and typography matching the current dark theme

#### Acceptance Criteria
- [ ] Custom color palette defined (dark backgrounds, accent blues, meter greens/reds)
- [ ] Typography scale matches current font sizes
- [ ] Spacing scale covers current padding/margin values
- [ ] Border radius tokens defined
- [ ] Box shadow tokens for depth effects
- [ ] Gradient utilities for header effects

#### Design Tokens to Migrate
| Current CSS | Tailwind Token |
|-------------|----------------|
| `#1a1a1a` | `bg-plugin-dark` |
| `#2a2a2a` | `bg-plugin-surface` |
| `#4a9eff` | `text-accent` / `bg-accent` |
| `#4caf50` | `bg-meter-safe` |
| `#ff1744` | `bg-meter-clip` |

---

### US-3: Migrate Global Styles

**As a** plugin UI developer  
**I want** global styles (`index.css`, `App.css`) converted to Tailwind  
**So that** the base styling uses the utility-first approach

#### Acceptance Criteria
- [ ] `index.css` reduced to Tailwind directives only
- [ ] Body/root styles expressed via Tailwind base layer or classes
- [ ] App layout classes migrated to `App.tsx`
- [ ] Header gradient effect preserved
- [ ] Footer styling migrated
- [ ] Original `App.css` deleted after migration

#### Files Affected
- `ui/src/index.css` — Keep (Tailwind entry point)
- `ui/src/App.css` — Delete after migration
- `ui/src/App.tsx` — Add utility classes

---

### US-4: Migrate Meter Component

**As a** plugin UI developer  
**I want** the Meter component styled with Tailwind utilities  
**So that** the meter visualization maintains its appearance without a separate CSS file

#### Acceptance Criteria
- [ ] All `.meter-*` classes replaced with Tailwind utilities
- [ ] Clip indicator animation preserved (use `@keyframes` in Tailwind config or `animate-*` utility)
- [ ] Gradient meter bar colors maintained
- [ ] Responsive/flexible layout preserved
- [ ] `Meter.css` deleted after migration

#### Technical Notes
- The clip pulse animation requires custom keyframes in `tailwind.config.js`
- Meter bar gradients can use Tailwind's gradient utilities

---

### US-5: Migrate Parameter Controls

**As a** plugin UI developer  
**I want** ParameterSlider and ParameterToggle styled with Tailwind  
**So that** parameter controls are consistent and maintainable

#### Acceptance Criteria
- [ ] ParameterSlider migrated — loading/error states, slider thumb styling
- [ ] ParameterToggle migrated (if applicable)
- [ ] Custom slider thumb styling via Tailwind's arbitrary values or plugin
- [ ] State variants (hover, active, disabled) use Tailwind modifiers
- [ ] `ParameterSlider.css` and `ParameterToggle.css` deleted

#### Technical Notes
- Range input styling requires `-webkit-slider-thumb` pseudo-elements
- May need `@layer components` for complex slider styling

---

### US-6: Migrate Utility Components

**As a** plugin UI developer  
**I want** ResizeControls, ResizeHandle, and LatencyMonitor styled with Tailwind  
**So that** all UI components use the same styling approach

#### Acceptance Criteria
- [ ] ResizeControls migrated
- [ ] ResizeHandle migrated (drag handle visual)
- [ ] LatencyMonitor migrated
- [ ] All three CSS files deleted after migration

---

### US-7: Optimize Production Bundle

**As a** plugin user  
**I want** the CSS bundle to only include used styles  
**So that** the plugin loads quickly with minimal overhead

#### Acceptance Criteria
- [ ] Production build uses Tailwind's purge/content scanning
- [ ] Final CSS bundle < 10KB (gzipped) for used utilities
- [ ] No unused Tailwind utilities in production bundle
- [ ] Source maps work correctly for debugging

#### Verification
```bash
npm run build
# Check dist/assets/*.css size
```

---

### US-8: Update Developer Documentation

**As a** plugin UI developer  
**I want** documentation on using Tailwind in VstKit  
**So that** I can follow project conventions when adding new UI components

#### Acceptance Criteria
- [ ] README or contributing guide updated with Tailwind usage
- [ ] Design token reference documented
- [ ] Example component styling patterns shown
- [ ] Prettier plugin for Tailwind class sorting configured (optional)

---

## Out of Scope

The following are explicitly **not** part of this feature:

- **Tailwind UI components** — We're not purchasing/using Tailwind UI library
- **CSS-in-JS migration** — We're staying with utility classes, not styled-components
- **Dark/light theme toggle** — Plugin uses dark theme only (standard for audio)
- **Tailwind v4** — Stick with v3.x for stability

---

## Priority Order

| Priority | Story | Rationale |
|----------|-------|-----------|
| 1 | US-1 | Foundation — nothing else works without this |
| 2 | US-2 | Design tokens enable consistent migration |
| 3 | US-3 | Global styles set the baseline |
| 4 | US-4 | Meter is the most complex component |
| 5 | US-5 | Parameter controls are user-facing |
| 6 | US-6 | Remaining utilities |
| 7 | US-7 | Optimization after migration |
| 8 | US-8 | Documentation last |

---

## Success Metrics

- **Zero CSS files** in `ui/src/components/` after migration
- **Build succeeds** with no CSS-related warnings
- **Visual parity** — UI looks identical before/after
- **Bundle size** — CSS < 10KB gzipped
- **Developer feedback** — Faster styling iteration

---

## References

- [TailwindCSS Documentation](https://tailwindcss.com/docs)
- [Vite + Tailwind Setup](https://tailwindcss.com/docs/guides/vite)
- Current CSS files: `ui/src/index.css`, `ui/src/App.css`, `ui/src/components/*.css`
