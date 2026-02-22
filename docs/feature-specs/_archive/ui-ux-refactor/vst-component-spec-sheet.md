# VST Component Spec Sheet

## Purpose

This is a concrete visual spec for a reusable VST component library targeting React + Tailwind implementation in Wavecraft.

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — System context and constraints
- [Coding Standards](../../architecture/coding-standards.md) — Repo standards hub
- [TypeScript & React Standards](../../architecture/coding-standards-typescript.md) — Component conventions
- [CSS & Styling Standards](../../architecture/coding-standards-css.md) — Tailwind and token usage
- [Testing & Quality Standards](../../architecture/coding-standards-testing.md) — Verification baseline
- [Roadmap](../../roadmap.md) — Project context
- [VST UI Research Findings](./vst-ui-research-findings.md) — Source rationale

---

## 1) Visual Language

- **Style:** modern studio-instrument aesthetic; low-glare surfaces, high legibility, precise indicators.
- **Geometry:** rounded-rect containers + circular primary controls.
- **Density:** compact by default for plugin windows; allow comfortable variant in browser/dev mode.
- **Feedback:** all interactions must present immediate visual response within 1 frame.
- **Focus treatment:** visible, non-color-only cue (ring + outline contrast).

---

## 2) Tokenized Color Roles

> Role-first tokens; theme values map per theme section.

| Role token               | Usage                                       |
| ------------------------ | ------------------------------------------- |
| `--color-bg-canvas`      | Main plugin background                      |
| `--color-bg-surface-1`   | Primary card/panel background               |
| `--color-bg-surface-2`   | Secondary inset/controls tray               |
| `--color-border-default` | Neutral borders/dividers                    |
| `--color-border-strong`  | Emphasis borders, selected containers       |
| `--color-text-primary`   | Main text                                   |
| `--color-text-secondary` | Labels, helper text                         |
| `--color-text-muted`     | Disabled/inactive labels                    |
| `--color-accent`         | Primary action, selected, active highlights |
| `--color-accent-soft`    | Accent-tinted backgrounds                   |
| `--color-focus-ring`     | Keyboard focus indicator                    |
| `--color-success`        | Safe/OK states                              |
| `--color-warning`        | Warning thresholds                          |
| `--color-danger`         | Clip/error/critical states                  |
| `--color-info`           | Mapping/learn/automation info state         |
| `--color-meter-safe`     | Meter safe zone                             |
| `--color-meter-warn`     | Meter warning zone                          |
| `--color-meter-clip`     | Meter clip zone                             |

Tailwind mapping guideline:

- Map roles to utilities via theme extension (e.g., `bg-plugin-surface`, `text-plugin-primary`, `ring-plugin-focus`).
- Never use ad-hoc hex classes when a role token exists.

---

## 3) Typography Scale

| Token         | Size / line-height | Weight  | Primary usage                     |
| ------------- | ------------------ | ------- | --------------------------------- |
| `type-2xs`    | 10/12              | 500     | Dense micro labels, unit suffixes |
| `type-xs`     | 11/14              | 500     | Control labels                    |
| `type-sm`     | 12/16              | 500     | Secondary body, metadata          |
| `type-md`     | 14/18              | 500–600 | Panel titles, prominent values    |
| `type-lg`     | 16/22              | 600     | Section headers                   |
| `type-xl-num` | 20/24 tabular      | 700     | Critical numeric readout          |

Implementation notes:

- Use tabular numerals for values/meters/timers.
- Keep label text uppercase optional but consistent per surface.

---

## 4) Spacing, Radius, Shadow Scales

### Spacing scale (px)

| Token     |  px | Typical usage          |
| --------- | --: | ---------------------- |
| `space-1` |   4 | Tight icon/text gap    |
| `space-2` |   8 | Control inner spacing  |
| `space-3` |  12 | Label/value separation |
| `space-4` |  16 | Card padding baseline  |
| `space-5` |  20 | Group spacing          |
| `space-6` |  24 | Large section spacing  |

### Radius scale

| Token          |   px | Typical usage            |
| -------------- | ---: | ------------------------ |
| `radius-sm`    |    4 | Small pills/tags         |
| `radius-md`    |    8 | Buttons/input shells     |
| `radius-lg`    |   12 | Cards and modules        |
| `radius-xl`    |   16 | Browser panes / overlays |
| `radius-round` | 9999 | Pills/toggles            |

### Shadow scale

| Token          | Value intent     | Usage                     |
| -------------- | ---------------- | ------------------------- |
| `shadow-0`     | none             | Flat surfaces             |
| `shadow-1`     | subtle elevation | Default floating controls |
| `shadow-2`     | medium elevation | Popovers/browser          |
| `shadow-focus` | ring + glow      | Focus-visible state       |

---

## 5) Unified State Model

All interactive components support baseline and plugin-specific states.

### Baseline interaction states

- `default`
- `hover`
- `focus-visible`
- `active`
- `disabled`
- `loading`
- `error`

### Plugin-specific semantic states

- `bypassed` (component or module bypassed)
- `armed` (ready for capture/automation/record)
- `mapped` (MIDI/host mapping active)

### State precedence (highest first)

`error` > `disabled` > `loading` > `active` > `focus-visible` > `hover` > `default`

Plugin-specific badges (`bypassed`, `armed`, `mapped`) layer on top of baseline state with icon + text cue.

---

## 6) Component Specs (visual + interaction)

### 6.1 Rotary Knob

| Spec area             | Definition                                                                                                                                                                                        |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | `sm` 32px, `md` 44px, `lg` 56px diameter                                                                                                                                                          |
| Label/value placement | Label top-centered (`type-xs`), knob center, value below (`type-sm` tabular)                                                                                                                      |
| Spacing               | label→knob: `space-2`; knob→value: `space-2`; component margin: `space-3`                                                                                                                         |
| States                | default indicator arc; hover halo; focus ring; active stronger arc; disabled low contrast; loading spinner overlay; error red ring; bypassed desaturated; armed amber dot; mapped blue chain icon |
| Accessibility notes   | Keyboard step controls with arrow keys; Shift+arrow fine step; clear visible focus; `aria-valuemin/max/now` and labelledby                                                                        |

### 6.2 Linear Fader

| Spec area             | Definition                                                                                                                                                                                      |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | Vertical: 120/160/220px track; Horizontal: 120/180/240px track                                                                                                                                  |
| Label/value placement | Label above track, value at end-cap or below, unit suffix aligned                                                                                                                               |
| Spacing               | thumb clearance `space-2`; label gap `space-2`; grouped channels gap `space-4`                                                                                                                  |
| States                | hover thumb highlight; focus ring on track; active thumb glow; disabled muted track; loading skeleton thumb; error danger border; bypassed striped overlay; armed amber edge; mapped info badge |
| Accessibility notes   | Ensure hit area >= 24px for thumb; keyboard increments/decrements; page up/down coarse step                                                                                                     |

### 6.3 Button (momentary/latch)

| Spec area             | Definition                                                                                                                                                                                                                |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | `sm` h-24, `md` h-32, `lg` h-40; min width 56/72/88px                                                                                                                                                                     |
| Label/value placement | Center label; optional left icon with `space-1` gap                                                                                                                                                                       |
| Spacing               | horizontal padding 10/12/16px per size                                                                                                                                                                                    |
| States                | hover tint; focus ring; active pressed shadow-inset; disabled opacity+cursor; loading spinner replacing icon; error uses danger role; bypassed uses muted + strike icon; armed uses warning border; mapped uses info tint |
| Accessibility notes   | Native `<button>` only; pressed state via `aria-pressed` for toggled mode                                                                                                                                                 |

### 6.4 Toggle / Switch

| Spec area             | Definition                                                                                                                                                                                           |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | `sm` 28x16, `md` 36x20, `lg` 44x24                                                                                                                                                                   |
| Label/value placement | Label right of switch (`space-2`); optional value badge trailing                                                                                                                                     |
| Spacing               | group row gap `space-3`                                                                                                                                                                              |
| States                | On/off track colors; hover thumb lift; focus ring around control; disabled muted; loading shimmer; error border; bypassed = forced off with bypass badge; armed = small amber pip; mapped = blue pip |
| Accessibility notes   | Use checkbox/switch semantics; ensure programmatic name includes linked label                                                                                                                        |

### 6.5 Meter (peak/RMS)

| Spec area             | Definition                                                                                                                                                                                                                                                                |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | Vertical widths 8/12/16px; heights 80/140/220px; horizontal heights 8/12/16px                                                                                                                                                                                             |
| Label/value placement | Channel label under meter; peak hold numeric top-right                                                                                                                                                                                                                    |
| Spacing               | channel gap `space-2`; block padding `space-3`                                                                                                                                                                                                                            |
| States                | safe/warn/clip color zones; hover optional readout emphasis; focus (if interactive) ring; disabled frozen/muted; loading pulsing placeholders; error hashed danger overlay; bypassed dim meter; armed border pulse; mapped not applicable except meter source mapping tag |
| Accessibility notes   | Non-color cues for clip (icon/text); provide text fallback values for screen reader regions                                                                                                                                                                               |

### 6.6 Tabs / Segmented Control

| Spec area             | Definition                                                                                                                                                                                                                 |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | `sm` h-28, `md` h-32, `lg` h-36                                                                                                                                                                                            |
| Label/value placement | Label centered; optional count badge trailing                                                                                                                                                                              |
| Spacing               | item gap `space-1`; bar padding `space-1`                                                                                                                                                                                  |
| States                | selected accent fill; hover surface raise; focus ring per tab; active press effect; disabled muted; loading ghost tabs; error tab with indicator dot; bypassed tab section desaturation; armed tab badge; mapped tab badge |
| Accessibility notes   | Use `role="tablist"` / `role="tab"` / `role="tabpanel"` pattern; arrow-key navigation within tablist; space/enter to activate tab                                                                                         |

### 6.7 Graph Editor (EQ/Filter curve editor)

| Spec area             | Definition                                                                                                                                                               |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Size variants         | Responsive width 100%; heights 140/180/240px                                                                                                                             |
| Label/value placement | Frequency axis bottom (x), magnitude axis left (y); current selected node readout top or overlay tooltip                                                                |
| Spacing               | axis label gap `space-2`; outer padding `space-3`                                                                                                                        |
| States                | node default/hover highlight ring; active drag cursor; selected node accent fill; disabled frozen (no interaction); loading skeleton curve; error domain constraint cues |
| Accessibility notes   | Provide alternative text-based parameter control for screen reader / keyboard-only users; graph edits are progressive enhancement                                        |

### 6.8 Envelope / LFO Editor

| Spec area             | Definition                                                                                                                             |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | Responsive width; height 80/120/160px                                                                                                  |
| Label/value placement | Shape name or rate display above or inside; segments labeled on hover                                                                  |
| Spacing               | handle clearance `space-2`; outer padding `space-3`                                                                                    |
| States                | handle hover/focus ring; active drag; bipolar zero-crossing accent line; bypassed desaturated outline; armed pulsing edge              |
| Accessibility notes   | Provide discrete parameter fallbacks (attack, decay, sustain, release as numeric inputs); graph is progressive enhancement              |

### 6.9 Preset Browser

| Spec area             | Definition                                                                                                                                                      |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | Panel overlay or inline; min-width 220px, resizable                                                                                                             |
| Label/value placement | Category list left panel; preset list right; search top                                                                                                         |
| Spacing               | row height `space-8` (32px); list item padding h: `space-3` v: `space-2`                                                                                       |
| States                | row hover surface; row selected accent; active (auditioned) indicator dot; disabled row muted; loading skeleton rows; error badge on failed preset; mapped row tag |
| Accessibility notes   | List items as `button` or `option`; search input labeled; keyboard nav with arrow keys; Enter to load preset; result count announced to screen reader           |

### 6.10 Macro / XY Pad

| Spec area             | Definition                                                                                                             |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Size variants         | `sm` 80px², `md` 120px², `lg` 160px²; fullscreen variant                                                               |
| Label/value placement | Axis labels bottom / right; cursor position readout overlay                                                            |
| Spacing               | outer padding `space-3`; label gap `space-2`                                                                           |
| States                | default crosshair; hover highlight ring; focus border ring; active drag tracking; disabled frozen; armed pulse border  |
| Accessibility notes   | Provide discrete X/Y numeric inputs as accessible alternative; XY surface as progressive enhancement                  |

### 6.11 Status Bar / Telemetry

| Spec area             | Definition                                                                                                              |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Size variants         | Full-width strip; height `space-6` (24px) compact; `space-8` (32px) comfortable                                        |
| Label/value placement | Left: transport status + connection, Center: CPU/voice info, Right: automation/mapping mode indicators                 |
| Spacing               | item gap `space-4`; icon+label gap `space-1`                                                                            |
| States                | default text; hover tooltip activation; armed indicators amber; mapping mode icon + info color; error critical red text |
| Accessibility notes   | Provide live region (`aria-live="polite"`) for connection and error state changes                                       |

---

## 7) Theme: Dark (Primary)

> Named `dark`. All color role tokens below.

| Role                     | Value (named Tailwind extension) | Notes                             |
| ------------------------ | -------------------------------- | --------------------------------- |
| `--color-bg-canvas`      | `#1a1a1a` / `plugin-dark`        | Main background                   |
| `--color-bg-surface-1`   | `#2a2a2a` / `plugin-surface`     | Cards, panels                     |
| `--color-bg-surface-2`   | `#333333`                        | Inset trays, header bars          |
| `--color-border-default` | `#3a3a3a` / `plugin-border`      | Default dividers                  |
| `--color-border-strong`  | `#555555`                        | Emphasis/selected borders         |
| `--color-text-primary`   | `#e0e0e0`                        | Main text                         |
| `--color-text-secondary` | `#aaaaaa`                        | Labels, metadata                  |
| `--color-text-muted`     | `#666666`                        | Disabled/inactive                 |
| `--color-accent`         | `#4a9eff` / `accent`             | Primary highlights                |
| `--color-accent-soft`    | `#1c3a5e`                        | Accent-tinted bg                  |
| `--color-focus-ring`     | `#4a9eff` / `accent`             | Focus indicator                   |
| `--color-success`        | `#22c55e`                        | OK / safe                         |
| `--color-warning`        | `#f59e0b`                        | Warning                           |
| `--color-danger`         | `#ef4444`                        | Error / clip                      |
| `--color-info`           | `#38bdf8`                        | Mapping/learn info                |
| `--color-meter-safe`     | `#22c55e` / `meter-safe`         | Safe zone                         |
| `--color-meter-warn`     | `#f59e0b` / `meter-warning`      | Warn zone                         |
| `--color-meter-clip`     | `#ef4444` / `meter-clip`         | Clip zone                         |

---

## 8) Accessibility Acceptance Notes

| Check                    | Requirement                                                |
| ------------------------ | ---------------------------------------------------------- |
| Color contrast (text)    | WCAG AA ≥ 4.5:1 for normal text; 3:1 for large text        |
| Color contrast (UI)      | WCAG AA ≥ 3:1 for interactive boundaries                   |
| Focus visibility         | Non-color-only cue; ring contrast ≥ 3:1 against background |
| Reduced motion           | All transitions guarded by `motion-safe:` or media query   |
| Keyboard operability     | All interactive elements reachable and activatable         |
| Accessible names         | All controls have visible label or `aria-label`            |
| Plugin-specific states   | `bypassed`, `armed`, `mapped` use icon+text, not color only |

---

## 9) React / Tailwind Implementation Guidance

### Class Pattern

```typescript
// ✅ Correct
import { focusRingClass, interactionStateClass } from './utils/classNames';

const buttonClass = cn(
  'rounded-md border px-3 py-1.5 text-sm font-medium',
  'bg-plugin-surface border-plugin-border text-plugin-primary',
  interactionStateClass,
  focusRingClass,
  disabled && 'opacity-50 cursor-not-allowed'
);
```

### Avoid

```typescript
// ❌ Do not use ad-hoc hex colors
className="bg-[#2a2a2a] text-[#e0e0e0] border-[#3a3a3a]"

// ❌ Do not suppress focus without replacement
className="focus:outline-none"

// ❌ Do not use inline style for token-covered values
style={{ color: '#4a9eff' }}
```

### State Model Pattern

```typescript
// ✅ Layered state composition
cn(
  baseClasses,               // layout and container
  defaultStateClasses,       // default appearance
  'hover:...',               // hover layer
  'focus-visible:...',       // focus layer
  isSelected && 'bg-accent', // selected semantic layer
  isDisabled && 'opacity-50 cursor-not-allowed', // disabled semantic layer
  isError && 'border-danger' // error semantic layer
)
```

---

## 10) Visual QA Acceptance Checklist

Before marking any component spec as ready:

- [ ] All role token usages mapped to Tailwind theme extensions (no ad-hoc hex classes)
- [ ] All 7 baseline states implemented and verified visually: default, hover, focus-visible, active, disabled, loading, error
- [ ] Plugin-specific states (bypassed, armed, mapped) implemented for relevant components
- [ ] Accessibility notes applied: keyboard operability, `aria-*`, visible focus, non-color state cues
- [ ] Reduced motion verified in browser devtools simulation (`prefers-reduced-motion: reduce`)
- [ ] Before/after Playwright screenshots for each state demonstrating intended appearance
- [ ] Component renders identically as props-only surface (no internal hook dependencies)

---

## 11) Handoff Notes

### For Coder

- All visual specs in this document are guidelines for React/Tailwind implementation, not prescriptive CSS.
- Tailwind utilities should be drawn from the existing `tailwind.config.js` theme tokens (see [CSS & Styling Standards](../../architecture/coding-standards-css.md)).
- Component implementations go in `ui/packages/components/src/`; smart container wiring in `sdk-template/ui/src/`.
- Follow the smart/presentational boundary rules from [low-level-design-ui-ux-refactor.md](./low-level-design-ui-ux-refactor.md) Section 2.2.

### For Tester / QA

- Use this spec as the visual acceptance baseline for any UI/UX component work.
- State model (Section 5) defines the expected interactions for all components.
- Accessibility notes per component define the minimum testable a11y bar.
- Visual QA acceptance checklist (Section 10) is the gate for component acceptance.
