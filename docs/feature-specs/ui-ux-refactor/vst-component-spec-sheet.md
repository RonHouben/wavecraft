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
| Accessibility notes   | Use `tablist`/`tab` semantics and arrow key nav; preserve visible selected + focus distinction                                                                                                                             |

### 6.7 Graph Editor (EQ/filter/curve)

| Spec area             | Definition                                                                                                                                                                                                          |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | Min canvas 280x160; preferred 420x220; expanded 640x300                                                                                                                                                             |
| Label/value placement | Axis labels outer edges; point values in floating readout                                                                                                                                                           |
| Spacing               | toolbar to graph `space-3`; graph padding `space-3`                                                                                                                                                                 |
| States                | hover point enlargement; focus ring on selected node; active drag path highlight; disabled lock overlay; loading skeleton grid; error banner top; bypassed curve dim; armed write-mode badge; mapped node highlight |
| Accessibility notes   | Keyboard node selection + nudging; provide textual list fallback for nodes/values                                                                                                                                   |

### 6.8 Envelope / LFO Editor

| Spec area             | Definition                                                                                                                                                                                                                      |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | Compact 260x120; default 360x160; expanded 520x220                                                                                                                                                                              |
| Label/value placement | Mode/polarity controls top row; rate/depth values right-aligned                                                                                                                                                                 |
| Spacing               | control row gap `space-2`; editor margin `space-3`                                                                                                                                                                              |
| States                | active segment highlight; hover handle tooltips; focus-visible handles; disabled lock; loading shimmer curve; error invalid-shape alert; bypassed desaturated waveform; armed modulation-capture badge; mapped destination chip |
| Accessibility notes   | Expose alternate numeric editing for points/segments; avoid motion-heavy previews without reduced-motion fallback                                                                                                               |

### 6.9 Preset Browser

| Spec area             | Definition                                                                                                                                                                                                             |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | Sidebar 220/280px; list row heights 28/34/40px; modal width 640/840px                                                                                                                                                  |
| Label/value placement | Category left, preset name center-left, metadata right                                                                                                                                                                 |
| Spacing               | row horizontal padding `space-3`; section spacing `space-4`                                                                                                                                                            |
| States                | hover row tint; focus row ring; active selected row with accent bar; disabled unavailable rows; loading skeleton rows; error inline retry row; bypassed not applicable; armed favorite-write state; mapped macro badge |
| Accessibility notes   | Full keyboard list navigation and type-to-search; maintain visible active item + focus item differentiation                                                                                                            |

### 6.10 Macro Controls

| Spec area             | Definition                                                                                                                                                                                                    |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | 4/8 macro slots; each slot 72x72 (`sm`), 96x96 (`md`)                                                                                                                                                         |
| Label/value placement | Macro label above, control center, assignment count below                                                                                                                                                     |
| Spacing               | slot gap `space-3`; section padding `space-4`                                                                                                                                                                 |
| States                | hover highlight; focus ring; active animation (respect reduced motion); disabled muted; loading assignment spinner; error conflict badge; bypassed macro muted; armed learn border; mapped chain icon + count |
| Accessibility notes   | Announce mapping count and learn state textually, not by color alone                                                                                                                                          |

### 6.11 XY Pad

| Spec area             | Definition                                                                                                                                                                                 |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Size variants         | 160x160, 220x220, 320x320                                                                                                                                                                  |
| Label/value placement | Axis labels at edges; current X/Y values below in tabular text                                                                                                                             |
| Spacing               | label-to-pad `space-2`; value row margin-top `space-2`                                                                                                                                     |
| States                | hover crosshair; focus ring perimeter; active puck glow; disabled lock overlay; loading grid shimmer; error out-of-range warning; bypassed dim puck; armed capture dot; mapped axis badges |
| Accessibility notes   | Keyboard nudging (arrow keys), coarse/fine modifiers, textual coordinate entry fallback                                                                                                    |

### 6.12 Status Bar

| Spec area             | Definition                                                                                                                                                                                                           |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Size variants         | Heights 24/28/32px                                                                                                                                                                                                   |
| Label/value placement | Left: transport/session; center: context; right: CPU/sample rate/latency/state icons                                                                                                                                 |
| Spacing               | section padding `space-2`/`space-3`; item gap `space-2`                                                                                                                                                              |
| States                | normal neutral; hover on clickable chips; focus ring for interactive areas; active chip accent; disabled muted; loading spinner chip; error danger strip; bypassed module badge; armed recording badge; mapped badge |
| Accessibility notes   | Use live region sparingly for important status changes only; avoid noisy announcements                                                                                                                               |

---

## 7) Theme Definitions

## 7.1 Dark Studio (default)

| Role token               | Suggested value |
| ------------------------ | --------------- |
| `--color-bg-canvas`      | `#13161A`       |
| `--color-bg-surface-1`   | `#1B2129`       |
| `--color-bg-surface-2`   | `#242C36`       |
| `--color-border-default` | `#334050`       |
| `--color-border-strong`  | `#48607A`       |
| `--color-text-primary`   | `#E8EEF5`       |
| `--color-text-secondary` | `#B9C7D8`       |
| `--color-text-muted`     | `#7D8DA1`       |
| `--color-accent`         | `#4AA3FF`       |
| `--color-accent-soft`    | `#1E3C5A`       |
| `--color-focus-ring`     | `#8BC3FF`       |
| `--color-success`        | `#36C27B`       |
| `--color-warning`        | `#F0B429`       |
| `--color-danger`         | `#FF5D6C`       |
| `--color-info`           | `#64B5FF`       |
| `--color-meter-safe`     | `#4BC96A`       |
| `--color-meter-warn`     | `#F2D94E`       |
| `--color-meter-clip`     | `#FF4A57`       |

## 7.2 Light Neutral

| Role token               | Suggested value |
| ------------------------ | --------------- |
| `--color-bg-canvas`      | `#F4F6F8`       |
| `--color-bg-surface-1`   | `#FFFFFF`       |
| `--color-bg-surface-2`   | `#EDEFF3`       |
| `--color-border-default` | `#CCD3DD`       |
| `--color-border-strong`  | `#A8B4C4`       |
| `--color-text-primary`   | `#1F2732`       |
| `--color-text-secondary` | `#455468`       |
| `--color-text-muted`     | `#6E7B8C`       |
| `--color-accent`         | `#2D7EEA`       |
| `--color-accent-soft`    | `#D9E8FF`       |
| `--color-focus-ring`     | `#2F80ED`       |
| `--color-success`        | `#238A55`       |
| `--color-warning`        | `#B87A00`       |
| `--color-danger`         | `#CC3D4C`       |
| `--color-info`           | `#2A7FD1`       |
| `--color-meter-safe`     | `#2FA55A`       |
| `--color-meter-warn`     | `#D2A900`       |
| `--color-meter-clip`     | `#D73A49`       |

### Theme role mapping guidance

- Maintain semantic parity: role token meaning must not change across themes.
- Validate contrast for text + icons in all key states.
- For warning/danger in light theme, increase border/icon weight when fill contrast is reduced.

---

## 8) Accessibility Notes (global)

- Use native semantic elements first (`button`, `input`, `select`, `tab` pattern).
- All interactive controls require keyboard operability and visible focus.
- Do not rely on color alone for state (add icon, text, pattern, or shape cue).
- Respect reduced motion (`prefers-reduced-motion`) for animations and meter embellishments.
- Provide alternative numeric/text entry for graph-like controls where feasible.

---

## 9) React/Tailwind Implementation Guidance

- Model state via data attributes or variant classes:
  - `data-state="active|disabled|loading|error"`
  - `data-plugin-state="bypassed|armed|mapped"`
- Compose shared primitives:
  - `ControlShell`, `ValueReadout`, `StateBadge`, `FocusRing`
- Keep token use semantic:
  - `bg-plugin-surface`, `text-plugin-primary`, `ring-plugin-focus`, `border-plugin-default`
- Prefer reusable variant maps over duplicated class strings.

### 9.1 Tailwind Token Mapping (aligned to `ui/tailwind.config.js`)

Use this table as the source-of-truth mapping when implementing React components with Tailwind utilities.

#### Semantic role → Tailwind theme key → utility examples

| Semantic role token      | Tailwind theme key                 | Utility class examples                                                                                                   |
| ------------------------ | ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `--color-bg-canvas`      | `colors.plugin.dark`               | `bg-plugin-dark`                                                                                                         |
| `--color-bg-surface-1`   | `colors.plugin.surface`            | `bg-plugin-surface`                                                                                                      |
| `--color-bg-surface-2`   | `colors.plugin.surface`            | `bg-plugin-surface/80`, `bg-plugin-surface` + `border border-plugin-border`                                              |
| `--color-border-default` | `colors.plugin.border`             | `border-plugin-border`, `divide-plugin-border`                                                                           |
| `--color-border-strong`  | `colors.plugin.border`             | `border-plugin-border/90`, `ring-1 ring-plugin-border`                                                                   |
| `--color-text-primary`   | _(base Tailwind scale)_            | `text-slate-100` (dark canvas), `text-slate-900` (light surface)                                                         |
| `--color-text-secondary` | _(base Tailwind scale)_            | `text-slate-300` (dark), `text-slate-600` (light)                                                                        |
| `--color-text-muted`     | _(base Tailwind scale)_            | `text-slate-500`                                                                                                         |
| `--color-accent`         | `colors.accent.DEFAULT`            | `text-accent`, `bg-accent`, `border-accent`, `ring-accent`                                                               |
| `--color-accent-soft`    | `colors.accent.light`              | `bg-accent-light/20`, `text-accent-light`                                                                                |
| `--color-focus-ring`     | `colors.accent.light`              | `focus-visible:ring-2 focus-visible:ring-accent-light focus-visible:ring-offset-2 focus-visible:ring-offset-plugin-dark` |
| `--color-success`        | `colors.meter.safe`                | `text-meter-safe`, `bg-meter-safe/15`, `border-meter-safe`                                                               |
| `--color-warning`        | `colors.meter.warning`             | `text-meter-warning`, `bg-meter-warning/15`, `border-meter-warning`                                                      |
| `--color-danger`         | `colors.meter.clip`                | `text-meter-clip`, `bg-meter-clip/15`, `border-meter-clip`                                                               |
| `--color-meter-safe`     | `colors.meter.safe`                | `from-meter-safe to-meter-safe-light`                                                                                    |
| `--color-meter-warn`     | `colors.meter.warning`             | `via-meter-warning`                                                                                                      |
| `--color-meter-clip`     | `colors.meter.clip` / `.clip-dark` | `to-meter-clip`, `shadow-[0_0_0_1px_theme(colors.meter.clip-dark)]`                                                      |
| `--color-info`           | `colors.accent.DEFAULT`            | `text-accent`, `bg-accent/10`, `border-accent/40`                                                                        |

Implementation note:

- `text-plugin-primary` and `ring-plugin-focus` are semantic aliases from this spec, not current theme keys. Until aliases are added, use the concrete mapped classes above.

#### Spacing / radius / shadow / type mappings

These roles should map to existing Tailwind scale names (no ad-hoc pixel values unless justified).

| Spec token     | Tailwind scale name | Utility examples                                     |
| -------------- | ------------------- | ---------------------------------------------------- |
| `space-1`      | `1`                 | `gap-1`, `px-1`, `mt-1`                              |
| `space-2`      | `2`                 | `gap-2`, `px-2`, `py-2`                              |
| `space-3`      | `3`                 | `gap-3`, `px-3`, `pt-3`                              |
| `space-4`      | `4`                 | `gap-4`, `p-4`, `mx-4`                               |
| `space-5`      | `5`                 | `gap-5`, `px-5`                                      |
| `space-6`      | `6`                 | `gap-6`, `p-6`, `mt-6`                               |
| `radius-sm`    | `rounded-sm`        | `rounded-sm`                                         |
| `radius-md`    | `rounded-md`        | `rounded-md`                                         |
| `radius-lg`    | `rounded-lg`        | `rounded-lg`                                         |
| `radius-xl`    | `rounded-xl`        | `rounded-xl`                                         |
| `radius-round` | `rounded-full`      | `rounded-full`                                       |
| `shadow-0`     | none                | `shadow-none`                                        |
| `shadow-1`     | small               | `shadow-sm`                                          |
| `shadow-2`     | medium              | `shadow`                                             |
| `shadow-focus` | ring emphasis       | `focus-visible:ring-2 ring-accent-light/70`          |
| `type-2xs`     | custom text size    | `text-[10px] leading-3 font-medium`                  |
| `type-xs`      | `text-xs`           | `text-xs leading-4 font-medium`                      |
| `type-sm`      | `text-sm`           | `text-sm leading-4 font-medium`                      |
| `type-md`      | base                | `text-base leading-5 font-medium`                    |
| `type-lg`      | lg                  | `text-lg leading-6 font-semibold`                    |
| `type-xl-num`  | xl + mono tabular   | `text-xl leading-6 font-bold font-mono tabular-nums` |

#### State class patterns (baseline + plugin-specific)

Use data attributes for deterministic styling and accessibility-friendly state signaling:

- Base selector contract:
  - `data-state="default|hover|focus|active|disabled|loading|error"`
  - `data-plugin-state="bypassed|armed|mapped"`

Recommended class patterns:

- `default`: `bg-plugin-surface border border-plugin-border text-slate-100`
- `hover`: `hover:bg-plugin-surface/90 hover:border-accent/40`
- `focus-visible`: `focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-light focus-visible:ring-offset-2 focus-visible:ring-offset-plugin-dark`
- `active`: `data-[state=active]:bg-accent/20 data-[state=active]:border-accent`
- `disabled`: `disabled:opacity-50 disabled:cursor-not-allowed disabled:saturate-50`
- `error`: `data-[state=error]:border-meter-clip data-[state=error]:ring-1 data-[state=error]:ring-meter-clip/60`
- `loading`: `data-[state=loading]:animate-pulse aria-busy:true`
- `bypassed`: `data-[plugin-state=bypassed]:opacity-60 data-[plugin-state=bypassed]:grayscale`
- `armed`: `data-[plugin-state=armed]:border-meter-warning data-[plugin-state=armed]:ring-1 data-[plugin-state=armed]:ring-meter-warning/50`
- `mapped`: `data-[plugin-state=mapped]:border-accent data-[plugin-state=mapped]:shadow-sm`

Accessibility reinforcement rules for states:

- Never rely on color-only cues for `error`, `armed`, `mapped`, or `clip`; include icon/text/badge.
- Preserve visible focus in all themes and states (including `disabled` siblings).
- Keep `aria-pressed`, `aria-busy`, and `aria-invalid` synced with visual state.

#### Dark/light theme application approach

Current Tailwind config provides plugin-dark-first tokens. For implementation consistency:

1. Default to dark studio classes (`bg-plugin-dark`, `bg-plugin-surface`, `border-plugin-border`).
2. Apply theme switching at root (`html[data-theme='dark'|'light']`) using CSS variables for text neutrals and any future semantic aliases.
3. Keep component class names semantic and stable; swap actual values via theme layer rather than rewriting component classes.
4. Validate contrast for both themes, especially warning/danger text and focus rings.

Suggested pattern:

- Keep utilities for structural roles (`bg-plugin-surface`, `border-plugin-border`, `text-accent`).
- For light theme neutrals, use CSS-variable-backed utilities or `dark:` variants to avoid duplicated component markup.

---

## 10) Visual QA Acceptance Checklist

### Must-pass criteria

- [ ] Every component supports required baseline states: default/hover/focus/active/disabled/loading/error.
- [ ] Plugin-specific states are visible and distinguishable: bypassed/armed/mapped.
- [ ] Label/value placement follows the spec for each component type.
- [ ] Size variants render without clipping at minimum plugin size.
- [ ] Color contrast and focus visibility are acceptable in both Dark Studio and Light Neutral themes.
- [ ] Keyboard operation works for all interactive controls, including graph/XY alternatives.
- [ ] Reduced-motion mode removes non-essential motion while preserving clarity.
- [ ] Meter warning/clip cues are perceivable with non-color reinforcement.
- [ ] No ad-hoc visual values where role tokens exist.
- [ ] Visual regressions checked in affected processor screens.

### Nice-to-have checks

- [ ] Micro-interaction timing is consistent across controls.
- [ ] Numeric formatting is consistent for value readouts.
- [ ] Loading/error copy is concise and contextual.

---

## 11) Handoff Notes

- This sheet is implementation-oriented and intended for component library build-out.
- If a component requires deviation (space, color, behavior), document rationale in the feature folder before merging.
