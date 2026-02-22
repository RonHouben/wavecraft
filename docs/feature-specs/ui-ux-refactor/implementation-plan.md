# Implementation Plan: UI/UX Refactor

## Related Documents

- [User Stories](./user-stories.md) — Requirements and acceptance criteria
- [Low-Level Design](./low-level-design-ui-ux-refactor-final.md) — Architecture decisions and hard boundary rules
- [UX Improvement Plan](./ux-improvement-plan.md) — Phased UX execution guidance
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Conventions hub
- [CSS & Styling Standards](../../architecture/coding-standards-css.md) — Token and theming rules
- [TypeScript Standards](../../architecture/coding-standards-typescript.md) — Component and class patterns

---

## Overview

This plan translates the low-level design and UX improvement plan into phased, coderready implementation tasks. Each phase produces independently verifiable, independently revertable diffs. No phase changes core audio, transport, or Rust engine behavior.

**Scope:** `ui/packages/` and `sdk-template/ui/` only.

**Strategy:** ship visible UX quality gains early (Phases 0–2), then stabilize architecture (Phases 3–5). Later phases are safe to defer; earlier phases are not blocked on them.

---

## Phase Map and Dependency Order

```
Phase 0: Baseline + Guardrails  ←────────── start here; unblocks everything
    │
    ├──► Phase 1: Focus/Interaction Consistency   (parallel-capable with Phase 2)
    │
    ├──► Phase 2: Token/Hierarchy Normalization   (parallel-capable with Phase 1)
    │
    └──► Phase 3: IPC Constants Migration  ←────── after Phase 0; independent of 1 & 2
         │
         ├──► Phase 4: Smart/Presentational Split + Fan-out Reduction
         │        (requires Phase 3 IpcMethods available; blocks Phase 5 on Slice C)
         │
         └──► Phase 5: Resize Ownership Unification
                  (requires Phase 4 smart containers exist; can run parallel to Phase 4)
```

### Parallelization Opportunities

| Parallel tracks               | Condition                                                          |
| ----------------------------- | ------------------------------------------------------------------ |
| Phase 1 + Phase 2             | Both begin after Phase 0 completes; no shared file dependencies    |
| Phase 3 alongside Phase 1/2   | `IpcConstants` addition is additive; no conflict with styling work |
| Phase 4 + Phase 5 (partially) | Per-surface tasks may interleave once Phase 3 is merged            |

---

## Phase 0 — Baseline Snapshot + Guardrails

### Objective

Establish a reproducible visual baseline and add static-analysis guardrails **before any code changes**. This is a prerequisite for all other phases.

### Tasks

#### 0.1 — Capture baseline screenshots

**Files affected:**

- `ui/` (read-only, no edits — screenshots captured against existing build)

**Steps:**

1. Run `cargo xtask dev` to start the dev servers.
2. Using the `playwright-mcp-ui-testing` skill, capture full-viewport screenshots of all primary surfaces: plugin root, slider controls, toggle buttons, selectable rows, meter display, version badge, any overlay/modal flows.
3. Save screenshots to `docs/feature-specs/ui-ux-refactor/visual-baseline/` (create directory).
4. Document any pre-existing visual QA caveats (focus, interaction states) in a brief `baseline-notes.md` in the same directory.

**Acceptance criteria:**

- [ ] Screenshot artifacts exist for every primary surface.
- [ ] Caveat list is documented and matches prior QA findings (focus visibility gaps, interaction-state inconsistencies).

**Verification:** Visual confirmation; artifacts committed and referenced in subsequent PRs.

---

#### 0.2 — Add ESLint guardrail: no `@wavecraft/core` imports in `@wavecraft/components`

**Files affected:**

- `ui/eslint.config.js`

**Changes:**

- Add `import/no-restricted-paths` rule that prevents any file under `ui/packages/components/` from importing from `ui/packages/core/`.

```js
// Example rule shape (adapt to project ESLint config structure)
{
  rules: {
    'import/no-restricted-paths': ['error', {
      zones: [{
        target: './packages/components',
        from: './packages/core',
        message: 'Presentational components must not import from @wavecraft/core. Pass data via props.',
      }],
    }],
  }
}
```

**Acceptance criteria:**

- [ ] `cargo xtask ci-check` passes with the new rule active.
- [ ] Existing violations (if any) are listed in a follow-up comment and tracked for Phase 4.

**Verification:** `cargo xtask ci-check --skip-tests` completes without ESLint errors on current codebase (violations may be `// eslint-disable` annotated temporarily, with TODO referencing Phase 4).

---

#### 0.3 — Add ESLint guardrail: no raw IPC method strings outside `IpcBridge`

**Files affected:**

- `ui/eslint.config.js`

**Changes:**

- Add `no-restricted-syntax` rule targeting string literals matching known IPC method names (`"getParameter"`, `"setParameter"`, `"getMeterFrame"`, `"getAudioStatus"`, `"ping"`) at call sites other than `ui/packages/core/src/ipc/`.

**Acceptance criteria:**

- [ ] Rule activates without blocking current build (annotate existing violations with TODO for Phase 3).
- [ ] `cargo xtask ci-check` passes.

**Verification:** `npx eslint ui/` output reviewed; violations documented.

---

### Phase 0 Risk Controls

| Risk                                | Control                                                                                 |
| ----------------------------------- | --------------------------------------------------------------------------------------- |
| Playwright screenshots unavailable  | Use browser-dev mode (`cargo xtask dev`); record any limitations in `baseline-notes.md` |
| ESLint rule breaks existing imports | Add temporary `// eslint-disable-next-line` with `TODO(phase-4)` tracker comment        |

### Phase 0 Rollback

- Screenshots are additive (no code change).
- ESLint rule additions: revert changes to `eslint.config.js`.

### Phase 0 PR

```
chore(ui): add visual baseline snapshots and ESLint guardrails
```

---

## Phase 1 — Focus / Interaction-State Consistency

**Depends on:** Phase 0 complete (baseline captured, guardrails active).
**Parallel-safe with:** Phase 2, Phase 3.

### Objective

Make every interactive control expose clear, consistent focus and interaction states. Closes the highest-priority visual QA caveats. Maps to **P0 User Stories 1 and 2**.

### Tasks

#### 1.1 — Add shared `focusRingClass` utility

**Files affected:**

- `ui/packages/components/src/utils/classNames.ts` (new or existing)

**Changes:**

- Define and export:

```typescript
export const focusRingClass =
  'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-1 focus-visible:ring-offset-plugin-dark';
```

- Define and export shared interaction-state Tailwind class sets for buttons, toggles, sliders, and selectable rows. Example:

```typescript
export const interactionStateClass =
  'hover:brightness-110 active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed';
```

**Acceptance criteria:**

- [ ] `focusRingClass` and `interactionStateClass` are exported from `classNames.ts`.
- [ ] TypeScript compiles cleanly (`tsc --noEmit`).
- [ ] No existing component behavior changed by this task alone.

**Verification:** `cargo xtask ci-check --skip-tests` passes.

---

#### 1.2 — Apply `focusRingClass` to all keyboard-focusable controls

**Files affected (apply per component):**

- `ui/packages/components/src/` — `ParameterSlider`, `ToggleButton`, any selectable row/card components, version badge (if interactive)
- `sdk-template/ui/src/` — any bespoke interactive controls not using the base component set

**Changes per component:**

- Add `focusRingClass` to the Tailwind `className` string.
- Remove any ad-hoc `outline-none` or `:focus { outline: none }` overrides that suppress default focus rings without a visible alternative.

**Acceptance criteria:**

- [ ] Every keyboard-focusable control shows a visible focus ring during keyboard navigation.
- [ ] Focus ring is present in both light-mode and dark-mode (plugin-dark) contexts.
- [ ] No `outline-none` without a `focus-visible` ring replacement.

**Verification:** Manual keyboard pass (Tab through all controls) in browser-dev mode; before/after Playwright screenshots compared.

---

#### 1.3 — Apply shared interaction-state classes to core control set

**Files affected:**

- `ui/packages/components/src/` — `ParameterSlider`, `ToggleButton`, selectable row/card components, primary action buttons

**Changes per component:**

- Apply `interactionStateClass` (or component-appropriate subset) to each control's root or trigger element.
- Verify that focus + disabled, active + hover, and selected + hover compound states are visually unambiguous.

**Acceptance criteria:**

- [ ] Hover, active, focus, and disabled states are visually distinct for each control in the core set.
- [ ] Compound states do not produce conflicting visual results.
- [ ] No regressions in controls not included in this task.

**Verification:** Playwright screenshots for each control; keyboard + mouse interaction spot-check.

---

#### 1.4 — Validate keyboard flow and tab order for primary surfaces

**Files affected:**

- Read-only verification pass; no code changes expected unless gaps are found.

**Steps:**

1. Tab through all interactive elements in the primary plugin surface in browser-dev mode.
2. Confirm Enter/Space activates custom controls; Escape closes any overlays.
3. Confirm no positive `tabIndex` values exist without documented justification.

**Acceptance criteria:**

- [ ] Tab order is logical and matches visual top-to-bottom, left-to-right flow.
- [ ] All custom interactive controls respond to keyboard activation.
- [ ] No keyboard traps.

**Verification:** `ui-accessibility-review` skill checklist executed; findings logged.

---

#### 1.5 — Validate reduced-motion baseline for changed transitions

**Files affected:**

- `ui/packages/components/src/` — any component modified in 1.2/1.3 with transitions.

**Changes:**

- Wrap any new CSS transitions or `transition-*` Tailwind utilities in `motion-safe:` prefix.
- Example: `motion-safe:transition-transform` instead of `transition-transform`.

**Acceptance criteria:**

- [ ] All new/modified transitions use `motion-safe:` prefix.
- [ ] Browser devtools `prefers-reduced-motion: reduce` simulation confirms transitions are suppressed.

**Verification:** Browser devtools simulation pass.

---

### Phase 1 Risk Controls

| Risk                                         | Control                                                                                       |
| -------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Focus ring breaks in WKWebView               | Test inside plugin host after Phase 1 merge; record results in test plan                      |
| Adjacent untouched component visually shifts | Before/after screenshot diff; revert class change to `focusRingClass` only (no layout impact) |

### Phase 1 Rollback

- Revert changes to `classNames.ts`; all per-component class additions are pure Tailwind string changes.

### Phase 1 PR Slices

```
feat(ui): add shared focusRingClass and interactionStateClass utilities       ← Task 1.1
feat(ui): apply focus ring to all keyboard-focusable controls                  ← Task 1.2
feat(ui): apply interaction-state classes to core control set                  ← Task 1.3
```

Tasks 1.4 and 1.5 are verification steps embedded in the above PRs' test passes; no separate PR needed.

---

## Phase 2 — Token / Hierarchy Normalization

**Depends on:** Phase 0 complete.
**Parallel-safe with:** Phase 1, Phase 3.

### Objective

Replace ad-hoc color/spacing/typography values with design tokens and improve visual hierarchy in targeted surfaces. Maps to **P1 User Story 3**.

### Tasks

#### 2.1 — Audit and inventory ad-hoc token violations

**Files affected:**

- `ui/packages/components/src/` — full grep scan
- `sdk-template/ui/src/` — full grep scan

**Steps:**

1. Run: `grep -rn "bg-\[#" ui/packages/ sdk-template/ui/src/`
2. Run: `grep -rn "style={{" ui/packages/ sdk-template/ui/src/` to find inline style objects with color/spacing overrides.
3. Produce a prioritized list: per-component, violation type, replacement token.

**Acceptance criteria:**

- [ ] Inventory list exists with at least severity tiers (blocking / warning / documented exception).
- [ ] List is committed to `docs/feature-specs/ui-ux-refactor/token-audit.md`.

**Verification:** File created; Coder uses it as the task list for 2.2 and 2.3.

---

#### 2.2 — Replace ad-hoc background and border token violations

**Files affected (based on 2.1 audit):**

- Components using `bg-[#...]` or `border-[#...]` for values that map to existing theme tokens.

**Allowed replacements:**
| Ad-hoc pattern | Replacement token |
|----------------|-------------------|
| `bg-[#2a2a2a]` or similar | `bg-plugin-dark` or `bg-plugin-surface` |
| `border-[#...]` | `border-plugin-border` |
| `bg-[#4a9eff]` or accent-like | `bg-accent` |

**Acceptance criteria:**

- [ ] Zero `bg-[#...]` or `border-[#...]` ad-hoc color values remain in touched files.
- [ ] Visual output is equivalent (before/after Playwright comparison).

**Verification:** Before/after screenshots; `cargo xtask ci-check` passes.

---

#### 2.3 — Replace ad-hoc text/color and inline style overrides

**Files affected (based on 2.1 audit):**

- Components with `style={{ color: '#...' }}` or `text-[#...]`.

**Allowed replacements:**
| Ad-hoc pattern | Replacement token |
|----------------|-------------------|
| `style={{ color: '#4a9eff' }}` | `text-accent` |
| `text-[#aaa]` | `text-gray-400` (or appropriate Tailwind scale) |

**Acceptance criteria:**

- [ ] No inline `style={{ color: ... }}` overrides that map to existing tokens in touched files.
- [ ] Visual regression check passes.

**Verification:** Before/after screenshots; ESLint (no new inline styles introduced).

---

#### 2.4 — Improve hierarchy: typography and spacing normalization

**Files affected:**

- Identified "flat hierarchy" surfaces from UX findings (refer to `ux-improvement-plan.md` Section 3).

**Changes:**

- Use Tailwind typography scale to differentiate labels from values (e.g., `text-xs` labels, `text-sm` values).
- Apply consistent `gap-*` / `p-*` spacing between logical groups.
- Improve emphasis with `font-semibold` for section titles; `font-normal` for secondary text.

**Acceptance criteria:**

- [ ] Hierarchy improvements are visible (grouped controls have logical spacing and label differentiation).
- [ ] All new spacing and type values use Tailwind scale (no ad-hoc values).
- [ ] No regressions in overall layout.

**Verification:** Before/after screenshots; `design-token-compliance` skill review on PR.

---

### Phase 2 Risk Controls

| Risk                                        | Control                                                                             |
| ------------------------------------------- | ----------------------------------------------------------------------------------- |
| Token substitution causes visual regression | Before/after screenshot diff required; revert per-component (non-breaking)          |
| Token doesn't match visual intent           | Document exception in PR with rationale; add `// token-exception: <reason>` comment |

### Phase 2 Rollback

- Per-component PR structure means each file change is independently revertable.
- No API surface, state, or behavior changes.

### Phase 2 PR Slices

```
chore(ui): token audit inventory — add token-audit.md                         ← Task 2.1
fix(ui): replace ad-hoc background/border colors with design tokens            ← Task 2.2
fix(ui): replace ad-hoc text/inline style overrides with design tokens         ← Task 2.3
fix(ui): normalize typography and spacing hierarchy in targeted surfaces        ← Task 2.4
```

---

## Phase 3 — IPC Constants Migration

**Depends on:** Phase 0 complete.
**Parallel-safe with:** Phase 1, Phase 2.
**Must complete before:** Phase 4 (smart containers will reference `IpcMethods`).

### Objective

Eliminate raw IPC method strings at UI call sites by adding a canonical `IpcMethods` / `IpcEvents` constants object to `@wavecraft/core`. Maps to **P2 User Story 5**.

### Tasks

#### 3.1 — Add `IpcMethods` and `IpcEvents` constants to `@wavecraft/core`

**Files affected:**

- `ui/packages/core/src/ipc/constants.ts` (new file)
- `ui/packages/core/src/index.ts` (add export)

**Changes:**

```typescript
// ui/packages/core/src/ipc/constants.ts
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

**Acceptance criteria:**

- [ ] `IpcMethods` and `IpcEvents` are exported from `@wavecraft/core` public API.
- [ ] TypeScript types `IpcMethod` and `IpcEvent` are available.
- [ ] No existing runtime behavior changes; this is a pure addition.
- [ ] `cargo xtask ci-check` passes.

**Verification:** `tsc --noEmit`; `cargo xtask ci-check`.

---

#### 3.2 — Migrate internal `@wavecraft/core` call sites to use constants

**Files affected:**

- `ui/packages/core/src/ipc/` — `IpcBridge.ts`, `ParameterClient.ts`, `MeterClient.ts` (any that reference raw string method names internally)

**Changes:**

- Replace `'getParameter'` → `IpcMethods.GET_PARAMETER`, etc. inside the core package.

**Acceptance criteria:**

- [ ] No raw IPC method strings remain inside `ui/packages/core/src/` (excluding `constants.ts` itself).
- [ ] All tests pass; no behavioral change.

**Verification:** `grep -rn '"getParameter"\|"setParameter"\|"getMeterFrame"\|"getAudioStatus"\|"ping"' ui/packages/core/src/` returns only the constants definition file.

---

#### 3.3 — Migrate external call sites in `sdk-template/ui/`

**Files affected:**

- `sdk-template/ui/src/` — any component or hook calling `IpcBridge.invoke(...)` with a raw string.

**Changes:**

- Import `IpcMethods` from `@wavecraft/core` at call sites; replace raw strings.
- Remove ESLint `// eslint-disable` annotations added in Phase 0 (Task 0.3) for these files.

**Acceptance criteria:**

- [ ] Zero raw IPC method strings at non-constants call sites (`grep` check passes cleanly).
- [ ] No new ESLint `no-restricted-syntax` violations.

**Verification:** `cargo xtask ci-check`; grep verification identical to 3.2.

---

### Phase 3 Risk Controls

| Risk                                 | Control                                                                                            |
| ------------------------------------ | -------------------------------------------------------------------------------------------------- |
| Constants string value typo          | TypeScript `as const` ensures type safety; unit test `IpcMethods.GET_PARAMETER === 'getParameter'` |
| Partial migration leaves mixed usage | Merge 3.2 + 3.3 in sequence; grep check in PR description required before merge                    |

### Phase 3 Rollback

- Remove `constants.ts` and the `index.ts` re-export; restore raw strings. No runtime wire-format changes occurred.

### Phase 3 PR Slices

```
feat(core): add IpcMethods and IpcEvents constants                             ← Task 3.1
refactor(core): migrate internal IPC call sites to use IpcMethods              ← Task 3.2
refactor(ui): migrate sdk-template IPC call sites to use IpcMethods            ← Task 3.3
```

---

## Phase 4 — Smart / Presentational Split + Fan-out Reduction

**Depends on:** Phase 0 + Phase 3 complete.
**Partially parallel with:** Phase 5 (per-surface tasks may interleave).

### Objective

Lift hook subscriptions out of presentational components into smart containers. Eliminate duplicate `useParameter` subscriptions for the same param ID. Maps to **P1 User Story 4** and the fan-out concern in **P1 User Story 3**.

### Tasks

#### 4.1 — Identify fan-out surfaces and components with internal hook usage

**Files affected:**

- `ui/packages/components/src/` — audit for `useParameter`, `useAllParameters`, `useMeterFrame` import usage
- `sdk-template/ui/src/` — identify surfaces with multiple `useParameter(id)` calls for the same ID

**Steps:**

1. Run: `grep -rn "useParameter\|useAllParameters\|useMeterFrame" ui/packages/components/`
2. Document results in `docs/feature-specs/ui-ux-refactor/fan-out-inventory.md`.

**Acceptance criteria:**

- [ ] Inventory exists listing every presentational component with internal hook usage.
- [ ] Each duplicate subscription (same param ID, multiple subscribers in one surface) is listed.

---

#### 4.2 — Extract hook calls from presentational components — per component

**Files affected (per component identified in 4.1):**

- `ui/packages/components/src/<Component>.tsx` — remove hook imports and calls
- `sdk-template/ui/src/<Container>.tsx` — add smart container that owns hook + passes props

**Changes pattern per component:**

1. Add the clean props interface to the presentational component (see LLD Section 4.2):
   - `value`, `onChange`, `disabled`, `aria-label` / `name`
   - `data-*` passthrough as needed
2. Remove `useParameter` (or other hook) import and call from inside the component.
3. Add (or extend) a smart container in `sdk-template/ui/` that calls the hook and passes props down.
4. Annotate removed hook usage with `// Lifted to <ContainerName> — Phase 4` comment for traceability during transition.

**Acceptance criteria (per component):**

- [ ] Component file has zero imports from `@wavecraft/core`.
- [ ] ESLint `import/no-restricted-paths` rule produces zero violations for the file.
- [ ] Smart container test: render the component in isolation (Vitest/React Testing Library) with mock props — works without IPC context.
- [ ] No behavioral regression in plugin-host and browser-dev mode.

**Verification:** ESLint pass; unit test passes; Playwright visual regression check.

---

#### 4.3 — Remove `legacyProps` gate once all consumers are migrated (per surface)

**Files affected:**

- `ui/packages/components/src/<Component>.tsx` — remove `legacyProps` fallback and associated code.

**Changes:**

- Once all call sites of a component pass the new clean props, delete the legacy hook path entirely.

**Acceptance criteria:**

- [ ] No `legacyProps` prop or internal hook call remains in the component.
- [ ] `cargo xtask ci-check` passes with no TypeScript errors from removed prop.

**Verification:** `cargo xtask ci-check`; Playwright before/after.

---

### Phase 4 Risk Controls

| Risk                                                    | Control                                                                                     |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| Presentational component re-imports a hook accidentally | ESLint `import/no-restricted-paths` rule (added Phase 0) catches it immediately             |
| Duplicate subscriber not lifted (missed fan-out)        | Smart container checklist: confirm exactly one `useParameter(id)` per surface per param ID  |
| Isolation test missing                                  | PR gate: each extracted component must have at least one Vitest render test with mock props |

### Phase 4 Rollback

- While `legacyProps` gate exists (Task 4.2 in progress), rollback = set `legacyProps: true` at the consumer.
- After Task 4.3, rollback = revert the component extraction PR.

### Phase 4 PR Slices

```
chore(ui): fan-out inventory — add fan-out-inventory.md                        ← Task 4.1
refactor(components): extract <ParameterSlider> to clean props interface        ← Task 4.2 (per component)
refactor(components): extract <ToggleButton> to clean props interface           ← Task 4.2 (per component)
refactor(components): remove legacyProps gate from <ParameterSlider>            ← Task 4.3 (per component)
```

---

## Phase 5 — Resize Ownership Unification

**Depends on:** Phase 4 smart containers in place (containers are the correct ownership site).
**Parallel-safe with:** Phase 4 per-surface tasks once first smart container exists.

### Objective

Establish singular `ResizeObserver` authority per surface. Eliminate duplicate or conflicting resize paths. Maps to **P2 User Story 5**.

### Tasks

#### 5.1 — Identify all active `ResizeObserver` and resize-event sites

**Files affected:**

- `ui/packages/components/src/` — grep for `ResizeObserver`
- `sdk-template/ui/src/` — grep for `ResizeObserver`

**Steps:**

1. Run: `grep -rn "ResizeObserver" ui/packages/ sdk-template/ui/src/`
2. Classify each occurrence as: smart-container-owned (correct), presentational-owned (needs migration), or legacy (needs gate).

**Acceptance criteria:**

- [ ] Inventory exists in `docs/feature-specs/ui-ux-refactor/resize-inventory.md`.

---

#### 5.2 — Migrate presentational `ResizeObserver` usage to smart containers

**Files affected (per surface):**

- `ui/packages/components/src/<Component>.tsx` — remove `ResizeObserver` construction
- `sdk-template/ui/src/<Container>.tsx` — add `ResizeObserver` + pass `onResize` prop down

**Changes pattern:**

1. Add `onResize?: (entry: ResizeObserverEntry) => void` prop to the presentational component.
2. Hook into the existing smart container's resize ownership (from Phase 4 container) to call the prop.
3. Remove the `ResizeObserver` construction from the presentational component.
4. Gate legacy path: add `legacyResize?: boolean` prop that defaults to `false` once declarative path is active.

**Acceptance criteria (per surface):**

- [ ] Only one `ResizeObserver` instance active per surface.
- [ ] Resize behavior verified in browser-dev mode: no jitter; correct dimensions propagated.
- [ ] `legacyResize` prop defaults to `false` and legacy path is documented.

**Verification:** Manual resize test in browser-dev mode; Playwright screenshot stability check.

---

#### 5.3 — Remove `legacyResize` gates and legacy observer construction

**Files affected:**

- `ui/packages/components/src/<Component>.tsx` — remove `legacyResize` prop and legacy `ResizeObserver` usage.

**Changes:**

- Delete the `legacyResize` prop fallback code.
- Final state: component accepts `onResize` callback; constructs no observer itself.

**Acceptance criteria:**

- [ ] No `ResizeObserver` construction inside `ui/packages/components/`.
- [ ] Resize ownership is singular and exclusively in smart containers.
- [ ] Plugin host resize behavior validated in WKWebView context where feasible.

**Verification:** `grep -rn "new ResizeObserver" ui/packages/components/` returns zero results; Playwright resize test.

---

### Phase 5 Risk Controls

| Risk                                           | Control                                                                                          |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| Resize jitter during `legacyResize` transition | Keep surfaces on `legacyResize: true` until both browser-dev and plugin-host paths are validated |
| Missing `onResize` call in smart container     | Test: resize browser window and verify component receives dimensions correctly                   |
| WKWebView behaves differently                  | Test in plugin host before removing `legacyResize` gate                                          |

### Phase 5 Rollback

- Set `legacyResize: true` at consumer to restore legacy observer path.
- After Task 5.3: revert component PR.

### Phase 5 PR Slices

```
chore(ui): resize ownership inventory — add resize-inventory.md                ← Task 5.1
refactor(components): gate <Meter> ResizeObserver behind legacyResize prop      ← Task 5.2 (per surface)
refactor(ui): unify resize ownership for plugin root surface                   ← Task 5.2 (per surface)
refactor(components): remove legacyResize gate from <Meter>                    ← Task 5.3 (per surface)
```

---

## Risk Controls Summary

| Risk                                      | Phase | Control                                                                 |
| ----------------------------------------- | ----- | ----------------------------------------------------------------------- |
| Focus ring breaks WKWebView               | 1     | Post-merge plugin-host test; record in test plan                        |
| Visual regression from token swap         | 2     | Before/after Playwright screenshots required per PR                     |
| IPC string typo in constants              | 3     | `as const` + unit test (`IpcMethods.GET_PARAMETER === 'getParameter'`)  |
| Presentational re-import of hook          | 4     | ESLint `import/no-restricted-paths` (Phase 0 guardrail)                 |
| Duplicate subscriber not lifted           | 4     | Smart container checklist; grep for duplicate `useParameter(id)`        |
| Resize jitter during transition           | 5     | `legacyResize: true` gate; validate in both browser-dev and plugin-host |
| A11y regression in reduced-motion context | 1     | All new transitions use `motion-safe:`; devtools simulation check       |
| IPC string drift re-emerges               | 3     | `no-restricted-syntax` ESLint rule (Phase 0 guardrail) post Phase 3     |

---

## PR Slicing Strategy

Each PR must:

1. Map to exactly one migration slice (A–E from LLD Section 6.1) or one phase task.
2. Include a "before/after behavior" note in the PR description.
3. Pass `cargo xtask ci-check` with zero new violations.
4. Reference the task number (e.g., `Phase 1 — Task 1.2`) in the PR description.

**Merge order constraint:**

```
Phase 0 PRs → merge first (unblocks all)
Phase 1 + 2 + 3 PRs → can merge in any order relative to each other
Phase 4 PRs → merge after Phase 3 is merged
Phase 5 PRs → merge after Phase 4 smart containers for relevant surfaces
```

**PR naming convention:**

| Slice                 | Prefix                  |
| --------------------- | ----------------------- |
| A (Focus/Interaction) | `feat(ui):`             |
| B (Token)             | `fix(ui):`              |
| C (Smart/Pres split)  | `refactor(components):` |
| D (IPC constants)     | `refactor(core):`       |
| E (Resize)            | `refactor(ui):`         |
| Inventory/docs        | `chore(ui):`            |

---

## Coder / Tester Handoff Checklist

### Coder Pre-Handoff (each PR)

- [ ] `cargo xtask ci-check` passes (lint, type-check, tests).
- [ ] No new TypeScript errors (`tsc --noEmit`).
- [ ] No new ESLint violations (especially `import/no-restricted-paths` and `no-restricted-syntax`).
- [ ] Before/after Playwright screenshots captured for any visual change.
- [ ] PR description includes: task reference, files changed, rollback method, and behavior notes.
- [ ] No `@wavecraft/core` imports added to `ui/packages/components/` (Phase 4+).
- [ ] No raw IPC method strings added outside `ui/packages/core/src/ipc/` (Phase 3+).
- [ ] Any new transitions use `motion-safe:` prefix.
- [ ] Any ad-hoc style values include a `// token-exception: <reason>` comment.

### Tester Verification (each Phase)

- [ ] `cargo xtask ci-check` passes on the merged phase branch.
- [ ] Playwright before/after screenshots compared; no unintended regressions in adjacent surfaces.
- [ ] Keyboard-only navigation pass: Tab order, Enter/Space activation, no traps.
- [ ] Focus ring visible for all interactive controls in primary surfaces.
- [ ] `prefers-reduced-motion` devtools simulation: new transitions suppressed.
- [ ] Core interaction states (hover/focus/active/disabled) visually distinct and consistent.
- [ ] Token audit: no new `bg-[#...]` or inline `style={{ color: ... }}` in changed files.
- [ ] Phase 4: confirm zero `@wavecraft/core` imports in `@wavecraft/components` (ESLint clean).
- [ ] Phase 4: confirm no duplicate `useParameter(id)` subscriptions for same param in same surface.
- [ ] Phase 3+: `grep` for raw IPC strings returns zero results outside `constants.ts`.
- [ ] Phase 5: `grep` for `new ResizeObserver` in `ui/packages/components/` returns zero results.
- [ ] WKWebView plugin-host: focus ring and resize behavior validated (Phases 1 and 5).
- [ ] Test plan (`test-plan.md`) updated with phase results and caveat closure evidence.

---

## Definition of Done

This feature is complete when all of the following are true:

- [ ] Phases 0–5 tasks are implemented and merged.
- [ ] All acceptance criteria across phases are verified and signed off in `test-plan.md`.
- [ ] Visual QA caveats for focus and interaction states are closed with before/after screenshot evidence.
- [ ] Zero `@wavecraft/core` imports remain inside `ui/packages/components/`.
- [ ] Zero raw IPC method strings at non-`IpcBridge` call sites (grep confirms).
- [ ] Single `ResizeObserver` authority per surface confirmed (grep confirms).
- [ ] `cargo xtask ci-check` passes on main branch post-merge of all slices.
- [ ] Keyboard + a11y pass documented in `test-plan.md` for all changed surfaces.
- [ ] Rollback paths documented and verified revertable per phase.
