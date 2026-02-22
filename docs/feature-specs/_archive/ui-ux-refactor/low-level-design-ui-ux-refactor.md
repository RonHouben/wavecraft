# Low-Level Design: UI/UX Refactor

## Related Documents

- [User Stories](./user-stories.md) — Requirements and acceptance criteria
- [UX Improvement Plan](./ux-improvement-plan.md) — Phased UX execution guidance
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Conventions hub
- [TypeScript Standards](../../architecture/coding-standards-typescript.md) — Component and class patterns
- [CSS & Styling Standards](../../architecture/coding-standards-css.md) — Token and theming rules
- [Roadmap](../../roadmap.md) — Project milestones

---

## Overview

This document defines the technical design decisions for the `ui-ux-refactor` feature. The refactor addresses six categories of architectural drift and UX inconsistency identified across prior architect, UX, and visual QA reviews:

1. Parameter state fan-out across multiple component layers
2. Blurred smart vs presentational component boundaries
3. Raw IPC method strings in UI code
4. Split resize ownership between declarative and legacy paths
5. Ad-hoc token usage and style leakage across components
6. Missing or inconsistent accessibility / interaction-state baselines

The design is **incremental and rollback-safe**: each migration slice maps to a standalone Phase and can be reverted independently without affecting other phases. No core audio or transport behavior changes.

**Scope:** `ui/packages/` and `sdk-template/ui/` only. Rust engine code is out of scope.

---

## 1. Current-State Architecture Summary

### 1.1 Layer Map (as-is)

```
┌────────────────────────────────────────────────────────────┐
│  sdk-template/ui/  (plugin app — per-plugin bundle)        │
│  • assembles smart container components                    │
│  • imports from @wavecraft/core and @wavecraft/components  │
└────────────────────┬───────────────────────────────────────┘
                     │ imports
        ┌────────────┴────────────┐
        │                         │
┌───────▼───────────┐   ┌─────────▼──────────────────────────┐
│ @wavecraft/core   │   │ @wavecraft/components               │
│ (ui/packages/core)│   │ (ui/packages/components)           │
│                   │   │                                    │
│ • IpcBridge       │   │ • Meter, ParameterSlider           │
│ • ParameterClient │   │ • ParameterGroup, VersionBadge     │
│ • MeterClient     │   │ • ToggleButton, selectable rows    │
│ • useParameter    │   │                                    │
│ • useAllParameters│   │ ⚠ Some components own local state  │
│ • useConnectionStatus│  that duplicates hook state         │
│ • useMeterFrame   │   │ ⚠ Several components reach upward  │
│                   │   │   into hook context implicitly     │
│ ⚠ Hooks expose   │   │ ⚠ Ad-hoc inline styles bypass      │
│   raw state to    │   │   token system (hardcoded values)  │
│   N consumers,    │   │ ⚠ Focus/interaction-state          │
│   causing fan-out │   │   treatment is inconsistent        │
└───────────────────┘   └────────────────────────────────────┘
```

### 1.2 Known Drift Points

| Area                          | Current Symptom                                                                                    | Impact                                            |
| ----------------------------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------------- |
| Parameter state               | Multiple components each subscribe to `useParameter` independently for the same param              | Redundant renders, fan-out risk                   |
| Smart/presentational boundary | Presentational components (`@wavecraft/components`) call hooks internally, coupling them to IPC    | Hard to test or reuse in isolation                |
| IPC method strings            | Raw string literals (`"getParameter"`, `"setParameter"`) scattered in component call sites         | Naming drift, typo risk, no editor assistance     |
| Resize ownership              | Both declarative (`ResizeObserver`-based) and legacy imperative paths active concurrently          | Double-resize events, jitter, ambiguous authority |
| Design tokens                 | `bg-[#2a2a2a]`, inline `style={{color: '#4a9eff'}}`, and similar ad-hoc values exist in components | Visual inconsistency, harder theming              |
| Interaction states            | Focus, hover, active, disabled states not consistently applied across control families             | Keyboard context loss, inconsistent feel          |

---

## 2. Target Architecture and Layer Boundaries

### 2.1 Layer Map (to-be)

```
┌────────────────────────────────────────────────────────────┐
│  sdk-template/ui/  (plugin app)                            │
│  • smart containers — own IPC, state side-effects         │
│  • pass typed props/callbacks down to presentational cmps  │
└────────────────────┬───────────────────────────────────────┘
                     │ typed props only
        ┌────────────┴────────────┐
        │                         │
┌───────▼───────────┐   ┌─────────▼──────────────────────────┐
│ @wavecraft/core   │   │ @wavecraft/components               │
│                   │   │                                    │
│ Parameter state   │   │ Presentational-first:              │
│ lives HERE only   │   │ • receive value/onChange/disabled  │
│ (single authority)│   │   as props                         │
│                   │   │ • own NO IPC or hook imports        │
│ IpcConstants      │   │ • own focus/interaction-state      │
│ enum/object:      │   │   via shared Tailwind variants      │
│  Methods.GET_PARAM│   │ • token-compliant styling only     │
│  Methods.SET_PARAM│   │ • pass resize events upward        │
│  Events.AUDIO_ST  │   │   via callback; no resize logic    │
│  (…)              │   │   inside presentational layer      │
└───────────────────┘   └────────────────────────────────────┘
```

### 2.2 Hard Boundary Rules

The following rules are **architecture requirements** for this refactor and all future UI work in scope:

| Rule                                                            | Package                 | Enforcement                                                          |
| --------------------------------------------------------------- | ----------------------- | -------------------------------------------------------------------- |
| Parameter state has a single subscription point per surface     | `@wavecraft/core` hooks | Smart container subscribes; passes props to children                 |
| Presentational components import nothing from `@wavecraft/core` | `@wavecraft/components` | ESLint `import/no-restricted-paths` rule (add to `eslint.config.js`) |
| IPC method names are referenced only via `IpcConstants`         | `@wavecraft/core`       | TypeScript type; raw strings trigger ESLint `no-restricted-syntax`   |
| Resize authority is singular per surface                        | App / smart container   | One `ResizeObserver` owner per surface; declarative preferred        |
| All styling uses design tokens                                  | Both packages           | `design-token-compliance` skill gate on PR                           |
| Focus/interaction states use shared Tailwind variants           | `@wavecraft/components` | Shared `focusRing`, `interactionState` class-sets                    |

---

## 3. Data / State Model Changes

### 3.1 Parameter State Authority

**Current:** Individual presentational or intermediate components each call `useParameter(id)` independently.

**Target:** Smart container (in `sdk-template/ui/`) calls `useParameter(id)` (or `useAllParameters()`) **once per surface**, then passes `value`, `onChange`, `disabled`, and metadata as typed props.

```typescript
// ✅ Smart container (sdk-template/ui/)
function GainSection() {
  const { param, setValue } = useParameter('gain');
  return (
    <ParameterSlider
      id="gain"
      name={param?.name ?? 'Gain'}
      value={param?.value ?? 0}
      min={param?.minValue ?? 0}
      max={param?.maxValue ?? 1}
      disabled={!param}
      onChange={setValue}
    />
  );
}

// ✅ Presentational component (@wavecraft/components) — no hook imports
export function ParameterSlider({ id, name, value, min, max, disabled, onChange }: Props) {
  return (
    <input
      type="range"
      aria-label={name}
      value={value}
      min={min}
      max={max}
      disabled={disabled}
      onChange={(e) => onChange(parseFloat(e.target.value))}
      className={cn('slider-thumb', focusRingClass, disabled && 'opacity-50 cursor-not-allowed')}
    />
  );
}
```

### 3.2 No New State Abstractions

This refactor does **not** introduce a global state manager (Redux, Zustand, Jotai, etc.). The `@wavecraft/core` hook layer (`useParameter`, `useAllParameters`) remains the single state boundary. Fan-out is reduced by lifting subscriptions to smart containers, not by adding a new store.

---

## 4. API / Contract Decisions

### 4.1 IPC Constants Object

A typed constants object (or `const enum`) is added to `@wavecraft/core` to eliminate raw string method names at call sites.

**Location:** `ui/packages/core/src/ipc/constants.ts`

```typescript
// Exported from @wavecraft/core public API

export const IpcMethods = {
  GET_PARAMETER: 'getParameter',
  SET_PARAMETER: 'setParameter',
  GET_METER_FRAME: 'getMeterFrame',
  GET_AUDIO_STATUS: 'getAudioStatus',
  PING: 'ping'
} as const;

export type IpcMethod = (typeof IpcMethods)[keyof typeof IpcMethods];

export const IpcEvents = {
  AUDIO_STATUS_CHANGED: 'audioStatusChanged',
  PARAM_UPDATE: 'paramUpdate',
  METER_FRAME: 'meterFrame'
} as const;

export type IpcEvent = (typeof IpcEvents)[keyof typeof IpcEvents];
```

**Migration:** `IpcBridge.invoke(method, ...)` call sites in the UI replace raw strings with `IpcMethods.*` references. `ParameterClient` and `MeterClient` are updated internally; external call sites update to use the exported constants when they construct direct `IpcBridge` calls.

**No breaking changes:** The underlying string values are unchanged; this is a reference discipline change, not a wire-format change.

### 4.2 Presentational Component Props Contract

All `@wavecraft/components` components that currently call hooks internally get a **clean props interface**:

| Prop                   | Type                                              | Description                   |
| ---------------------- | ------------------------------------------------- | ----------------------------- |
| `value`                | `number \| string \| boolean`                     | Controlled value              |
| `onChange`             | `(v: T) => void`                                  | Controlled change handler     |
| `disabled`             | `boolean`                                         | Disables interaction          |
| `aria-label` \| `name` | `string`                                          | Accessible label              |
| `onResize`             | `(entry: ResizeObserverEntry) => void` (optional) | Resize callback, owned upward |

Components may **not** add IPC or hook imports. Props must be sufficient for full render without side-effects.

### 4.3 Resize Ownership Contract

**Rule:** Exactly one `ResizeObserver` instance is active per surface. The **smart container** owns the observer and passes `onResize` as a prop to any child that needs to respond to size changes.

```typescript
// ✅ Smart container owns resize
function PluginRoot() {
  const ref = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 0, height: 0 });

  useEffect(() => {
    if (!ref.current) return;
    const ro = new ResizeObserver(([entry]) => {
      setSize({ width: entry.contentRect.width, height: entry.contentRect.height });
    });
    ro.observe(ref.current);
    return () => ro.disconnect();
  }, []);

  return <div ref={ref}><SomePresComponent size={size} /></div>;
}

// ❌ Presentational component must NOT own ResizeObserver
export function Meter({ ... }) {
  // No new ResizeObserver here
}
```

**Legacy fallback:** If an existing component currently owns a `ResizeObserver` and its surface is not in Phase 1 scope, the observer is **gated behind a `legacyResize` prop** that defaults to `false` once the declarative path is active. This provides a rollback point without immediate breakage.

---

## 5. Accessibility and Token Constraints

These constraints apply as **hard gates** in PR review for all changes in this feature. They are not optional polish.

### 5.1 Accessibility (A11y) Requirements

| Requirement          | Standard               | Implementation                                                                                 |
| -------------------- | ---------------------- | ---------------------------------------------------------------------------------------------- |
| Focus visibility     | WCAG 2.1 AA — SC 2.4.7 | `:focus-visible` ring on all interactive controls via shared `focusRingClass` Tailwind variant |
| Keyboard operability | WCAG 2.1 AA — SC 2.1.1 | All custom controls respond to `Enter`/`Space`; `Escape` closes overlays                       |
| Focus order          | WCAG 2.1 AA — SC 2.4.3 | Logical DOM order; no `tabIndex > 0` unless explicitly justified                               |
| Accessible names     | WCAG 2.1 AA — SC 4.1.2 | All controls have `aria-label` or associated `<label>`                                         |
| Contrast             | WCAG 2.1 AA — SC 1.4.3 | Token-based colors; design token table maintains ≥4.5:1 ratio for text                         |
| Reduced motion       | WCAG 2.1 AA — SC 2.3.3 | All new transitions guarded by `motion-safe:` or `@media (prefers-reduced-motion)`             |
| Semantic elements    | WCAG best practice     | Native elements preferred; ARIA only when semantics cannot be achieved natively                |

**Shared focus ring class (defined in `ui/packages/components/src/utils/classNames.ts`):**

```typescript
export const focusRingClass =
  'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-1 focus-visible:ring-offset-plugin-dark';
```

### 5.2 Design Token Constraints

All new or modified styling in scope **must** use the following token sources. Ad-hoc values require a documented exception comment in the PR.

| Category           | Allowed sources                                                             |
| ------------------ | --------------------------------------------------------------------------- |
| Background         | `bg-plugin-dark`, `bg-plugin-surface` (Tailwind theme tokens)               |
| Border             | `border-plugin-border`                                                      |
| Text               | `text-accent`, `text-accent-light`, `text-gray-*` (Tailwind scale)          |
| Metering           | `bg-meter-safe`, `bg-meter-warning`, `bg-meter-clip`                        |
| Interaction states | Tailwind state variants: `hover:`, `active:`, `disabled:`, `focus-visible:` |
| Spacing            | Tailwind spacing scale (`gap-*`, `p-*`, `m-*`)                              |
| Typography         | Tailwind type scale (`text-sm`, `text-base`, `font-semibold`, etc.)         |

**Prohibited:**

- Inline `style={{ color: '#...' }}` for tokens that exist in the theme
- `bg-[#...]` hardcoded arbitrary colors
- Component-scoped CSS files (`MyComponent.css`)

**Exceptions** (custom CSS remains valid):

- Vendor-prefixed slider thumb pseudo-elements (`::-webkit-slider-thumb`)
- `@keyframes`-based animations not expressible in Tailwind utilities

---

## 6. Migration Strategy and Rollback Points

The migration is structured into three independent slices, each with a defined rollback point.

### 6.1 Migration Slices

| Slice                              | Phase     | Scope                                                                             | Rollback mechanism                                                      |
| ---------------------------------- | --------- | --------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| **A — Focus & Interaction States** | Phase 1   | Add `focusRingClass` and shared interaction-state variants to existing components | Revert `classNames.ts` changes; no API surface changed                  |
| **B — Token Normalization**        | Phase 1–2 | Replace ad-hoc color/spacing values with design tokens in touched surfaces        | Per-component PR; non-breaking visual-only changes                      |
| **C — Smart/Presentational Split** | Phase 3   | Lift hook calls out of presentational components; add clean props contract        | Feature-flagged via `legacyProps` opt-in during transition              |
| **D — IPC Constants**              | Phase 3   | Add `IpcMethods`/`IpcEvents` exports; migrate call sites                          | Old strings still work at runtime; migration is reference-only          |
| **E — Resize Ownership**           | Phase 3   | Introduce declarative `ResizeObserver` in smart containers; gate legacy path      | `legacyResize` prop allows fallback until surface migration is complete |

### 6.2 Rollback Points

Each slice is **independently revertable**:

- **Slices A + B** (styling): pure Tailwind class changes. Reverting to prior classes restores previous appearance with no state or API impact.
- **Slice C** (smart/presentational): until legacy hook usage is removed, the old path remains active via the `legacyProps` interface flag.
- **Slice D** (IPC constants): constants are pure TypeScript additions; string values are unchanged. Reverting means removing the constants file; call sites retain string literals.
- **Slice E** (resize): the `legacyResize` bool prop defaults to `true` during migration. Setting it to `false` activates the declarative path. Reverting means setting it back to `true`.

### 6.3 PR Structure

Each slice targets its own minimal-scope PR:

1. `feat(ui): add shared focusRingClass and interaction-state variants` (Slice A)
2. `fix(ui): token normalization pass — surface X` (Slice B, per surface)
3. `refactor(core): add IpcMethods and IpcEvents constants` (Slice D, non-breaking)
4. `refactor(components): extract hook calls; add clean props contract` (Slice C, per component)
5. `refactor(ui): unify resize ownership — surface X` (Slice E, per surface)

---

## 7. Risks and Mitigations

| Risk                                                                         | Likelihood | Impact     | Mitigation                                                                                          |
| ---------------------------------------------------------------------------- | ---------- | ---------- | --------------------------------------------------------------------------------------------------- |
| Coupling regression: presentational component accidentally re-imports a hook | Medium     | High       | ESLint `import/no-restricted-paths` rule; PR gate via `design-token-compliance` skill               |
| Token substitution causes subtle visual regression                           | Medium     | Low–Medium | Before/after Playwright screenshots per PR; visual diff review required                             |
| Incomplete fan-out lift leaves duplicate subscribers                         | Medium     | Medium     | Smart container checklist in implementation plan; tester validates no duplicate renders             |
| Resize jitter during `legacyResize` transition                               | Low        | Medium     | Keep surfaces on `legacyResize: true` until both paths are validated in browser-dev and plugin-host |
| Focus styling breaks in WKWebView                                            | Low        | High       | Test critical focus indicators inside plugin host, not only browser-dev                             |
| IPC string drift re-emerges                                                  | Low        | Medium     | `no-restricted-syntax` ESLint rule prevents new raw string call sites post-migration                |
| A11y regression in reduced-motion context                                    | Low        | Medium     | All new transitions use `motion-safe:` prefix; checked in Tester verification pass                  |

---

## 8. Test and Verification Expectations

### 8.1 Automated Checks (per PR)

- `cargo xtask ci-check` — full lint/type-check/test pass required
- `tsc --noEmit` — no new TypeScript errors
- ESLint — no violations of `import/no-restricted-paths` or `no-restricted-syntax` rules

### 8.2 Visual Verification (per Phase)

- **Before/after Playwright screenshots** captured for every changed primary surface using the `playwright-mcp-ui-testing` skill.
- Screenshot comparison must show focus indicator improvement (Slice A), token consistency (Slice B), and no unintended regressions in adjacent unmodified surfaces.
- Caveat closure: QA caveats from prior visual review must be explicitly re-checked and documented as resolved.

### 8.3 Keyboard / A11y Pass (per Phase)

Using the `ui-accessibility-review` skill checklist:

- [ ] Tab order is logical across all changed surfaces
- [ ] Enter/Space activates custom interactive controls
- [ ] Escape closes applicable overlay/modal interactions
- [ ] Focus ring is visible (keyboard-only navigation pass; non-color indicator)
- [ ] All changed controls have accessible names
- [ ] Contrast meets ≥4.5:1 for text elements in normal and hover/focus states
- [ ] `prefers-reduced-motion` tested in browser devtools for new transitions

### 8.4 Regression Guard

- Unmodified surfaces: baseline Playwright screenshots must remain unchanged (no unintended side-effects from shared class changes).
- Core audio behavior: no functional parameter or metering regressions; verified via existing Rust + UI unit test suite.
- Plugin host mode: resize and focus behavior validated in WKWebView context where feasible.

### 8.5 Structural Boundary Verification (Slice C / D / E)

- Confirm: no `@wavecraft/core` imports remain inside `@wavecraft/components` after Slice C.
- Confirm: no raw IPC method strings remain in component files after Slice D (grep check).
- Confirm: no more than one active `ResizeObserver` per surface after Slice E.

---

## 9. Out-of-Scope

- Rust engine / DSP changes
- New product features beyond UX quality
- Transport or IPC wire-format changes
- Plugin format changes (VST3/CLAP/AU)
- Full-system visual rebranding

---

## Definition of Done

- [ ] All migration slices (A–E) completed for in-scope surfaces
- [ ] `cargo xtask ci-check` passes (including lint, type-check, and tests)
- [ ] Visual QA caveats for focus and interaction states are closed with evidence
- [ ] Keyboard + a11y pass completed and documented in `test-plan.md`
- [ ] No `@wavecraft/core` imports present in `@wavecraft/components` (Slice C complete)
- [ ] No raw IPC method strings at non-`IpcBridge` call sites (Slice D complete)
- [ ] Single `ResizeObserver` authority per surface confirmed (Slice E complete)
- [ ] Rollback paths documented in implementation plan and verified revertable
