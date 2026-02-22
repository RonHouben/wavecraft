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

This plan translates the low-level design and UX improvement plan into phased, coder-ready implementation tasks. Each phase produces independently verifiable, independently revertable diffs. No phase changes core audio, transport, or Rust engine behavior.

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

**Verification:** Before/after Playwright screenshots; `cargo xtask ci-check` passes.

---

### Phase 2 Risk Controls

| Risk                                                       | Control                                                                               |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| Token replacement changes visual weight in unintended ways | Before/after screenshot comparison per component; revert token change if diff is wrong |

### Phase 2 Rollback

- Each task is a per-component, per-file change. Revert individual files as needed.

### Phase 2 PR Slices

```
fix(ui): token normalization — bg and border values          ← Task 2.2
fix(ui): token normalization — text and inline style values  ← Task 2.3
fix(ui): hierarchy normalization — typography and spacing    ← Task 2.4
```

Task 2.1 (audit) is embedded in the Phase 2 kickoff work; no separate PR.

---

## Phase 3 — IPC Constants Migration

**Depends on:** Phase 0 complete.
**Parallel-safe with:** Phase 1, Phase 2.

### Objective

Eliminate raw IPC method string literals at UI call sites. Add a `IpcMethods`/`IpcEvents` constants export to `@wavecraft/core`; migrate all in-scope call sites to use it. Maps to **P2 User Story 5**.

### Tasks

#### 3.1 — Add `IpcMethods` and `IpcEvents` constants to `@wavecraft/core`

**Files affected:**

- `ui/packages/core/src/ipc/constants.ts` (new)
- `ui/packages/core/src/index.ts` (update exports)

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

- Export `IpcMethods`, `IpcEvents`, `IpcMethod`, `IpcEvent` from `index.ts`.

**Acceptance criteria:**

- [ ] `IpcMethods` and `IpcEvents` exported from `@wavecraft/core`.
- [ ] TypeScript types enforced (`IpcMethod`, `IpcEvent`).
- [ ] Existing build passes; no behavioral change.

**Verification:** `cargo xtask ci-check` passes.

---

#### 3.2 — Migrate in-scope call sites in `@wavecraft/core` to use constants

**Files affected:**

- `ui/packages/core/src/` — any internally-defined call sites using raw IPC string literals.

**Changes:**

- Replace any `bridge.invoke('getParameter', ...)`, `bridge.invoke('setParameter', ...)` etc. with `bridge.invoke(IpcMethods.GET_PARAMETER, ...)`.
- Do **not** modify `sdk-template/` call sites yet — those are Phase 4 scope.

**Acceptance criteria:**

- [ ] No raw IPC method strings at call sites within `ui/packages/core/src/` (except inside `ipc/constants.ts` itself).
- [ ] ESLint `no-restricted-syntax` rule from Phase 0.3 passes cleanly.

**Verification:** `cargo xtask ci-check` passes; grep check confirms zero violations.

---

#### 3.3 — Migrate in-scope call sites in `sdk-template/ui/` to use constants

**Files affected:**

- `sdk-template/ui/src/` — any components or hooks using raw IPC method string literals.

**Changes:**

- Import `IpcMethods`/`IpcEvents` from `@wavecraft/core`; replace all in-scope raw string literals.

**Acceptance criteria:**

- [ ] No raw IPC method strings at call sites within `sdk-template/ui/src/`.
- [ ] Build and tests pass.

**Verification:** `cargo xtask ci-check` passes.

---

### Phase 3 Risk Controls

| Risk                                       | Control                                                                  |
| ------------------------------------------ | ------------------------------------------------------------------------ |
| `IpcMethods` value differs from raw string | Constants values are identical strings; wire format is unchanged          |
| New raw strings introduced in later PRs    | ESLint `no-restricted-syntax` rule from Phase 0.3 is the guard           |

### Phase 3 Rollback

- Revert `ipc/constants.ts` and its exports; re-introduce string literals at call sites.

### Phase 3 PR Slices

```
feat(core): add IpcMethods and IpcEvents constants export     ← Task 3.1
refactor(core): migrate call sites to use IPC constants       ← Task 3.2
refactor(template): migrate template call sites to IPC consts ← Task 3.3
```

---

## Phase 4 — Smart/Presentational Split + Fan-out Reduction

**Depends on:** Phase 3 complete.
**Parallel-safe with:** Phase 5 (partially — per-surface tasks may interleave).

### Objective

Remove hook imports from `@wavecraft/components` presentational components. Lift parameter subscriptions to smart containers in `sdk-template/ui/`. Reduce duplicate state subscriptions. Maps to **P1 User Story 4**.

### Tasks

#### 4.1 — Inventory and map current state fan-out

**Files affected:**

- Read-only analysis pass.

**Steps:**

1. For each `@wavecraft/components` component, list all `use*` hook calls.
2. For each hook call, identify which smart container should own the subscription.
3. Produce a fan-out map: component → hook → smart container owner.
4. Commit to `docs/feature-specs/ui-ux-refactor/fan-out-inventory.md`.

**Acceptance criteria:**

- [ ] Fan-out map exists and is verified against the actual codebase.

---

#### 4.2 — Extract props interfaces for each presentational component

**Files affected:**

- `ui/packages/components/src/` — each component being migrated.

**Changes:**

- Define a clean `Props` interface for each component with no IPC or hook coupling (as per LLD Section 4.2).
- Add the `legacyProps` fallback mechanism described in LLD Section 6.3:

```typescript
interface BaseProps {
  value: number;
  onChange: (v: number) => void;
  disabled?: boolean;
}

interface LegacyProps {
  legacyProps: true;
  parameterId: string; // uses internal hook; remove in Phase 4 cleanup
}

type Props = BaseProps | LegacyProps;
```

**Acceptance criteria:**

- [ ] Clean `Props` interface defined for each component.
- [ ] `legacyProps` fallback path available and tested.
- [ ] TypeScript compiles cleanly.

---

#### 4.3 — Lift hook ownership to smart containers

**Files affected:**

- `sdk-template/ui/src/` — smart container components (e.g., `App.tsx`, processor-specific smartwrappers)
- `ui/packages/components/src/` — components being migrated (removed hook dependencies)

**Changes (per component):**

1. Remove hook calls from the presentational component.
2. Accept all data as typed props.
3. In the smart container: add `useParameter(id)` (or `useAllParameters()`) subscription; pass resolved values as props.

**Acceptance criteria:**

- [ ] No `use*` hook imports remain in `ui/packages/components/src/` files.
- [ ] All hook subscriptions are in `sdk-template/ui/src/` smart containers.
- [ ] ESLint `import/no-restricted-paths` rule from Phase 0.2 passes cleanly.
- [ ] Presentational components render correctly with props-only data (no hook fallback needed).

---

#### 4.4 — Remove `legacyProps` fallback and clean up

**Files affected:**

- All components migrated in 4.2 and 4.3.

**Changes:**

- Remove `legacyProps` interface branches and internal hook calls from each component.
- Confirm `import/no-restricted-paths` ESLint rule has no active suppressions remaining.

**Acceptance criteria:**

- [ ] No `legacyProps` or internal hook calls remain in `@wavecraft/components`.
- [ ] All ESLint `// eslint-disable` lines added during Phase 0/4 are removed.
- [ ] `cargo xtask ci-check --full` passes.

---

### Phase 4 Risk Controls

| Risk                                              | Control                                                                                           |
| ------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Smart container must subscribe to many parameters | Use `useAllParameters()` to get all params; filter locally; one subscription per surface           |
| Rendering regression during hook-lift             | `legacyProps` fallback allows side-by-side testing before cleanup                                |
| New hook-coupling introduced in components        | ESLint `import/no-restricted-paths` guard; blocked at PR review                                  |

### Phase 4 Rollback

- Revert to `legacyProps` path; defer cleanup to next cycle.

### Phase 4 PR Slices

```
refactor(components): define clean props interface + legacyProps fallback  ← Task 4.2
refactor(components+template): lift hook ownership to smart containers     ← Task 4.3
chore(components): remove legacyProps and clean up Phase 4 stubs           ← Task 4.4
```

---

## Phase 5 — Resize Ownership Unification

**Depends on:** Phase 4 smart containers established.
**Parallel-safe with:** Phase 4 per-surface tasks.

### Objective

Establish singular, deterministic resize authority in smart containers. Remove competing `ResizeObserver` instances from presentational components. Clarify IPC-facing window resize notification path. Maps to **P2 User Story 5**.

### Tasks

#### 5.1 — Inventory resize ownership per surface

**Files affected:**

- Read-only analysis pass.

**Steps:**

1. Grep for `ResizeObserver` and `resize` event listeners in `ui/packages/` and `sdk-template/ui/src/`.
2. Map: surface → current owner(s) → target smart container owner.
3. Commit map to `docs/feature-specs/ui-ux-refactor/resize-inventory.md`.

---

#### 5.2 — Add declarative resize ownership to smart containers

**Files affected:**

- `sdk-template/ui/src/` — relevant smart container (e.g., `App.tsx` or root container).

**Changes:**

- Add single `ResizeObserver` instance managed in the smart container root.
- Pass `onResize` callback as prop to any child requiring size-aware rendering.
- Gate legacy path with `legacyResize: true` prop on affected components.

**Acceptance criteria:**

- [ ] Single `ResizeObserver` instance per surface.
- [ ] Resize events flow as props; no resize observer in presentational layer.

---

#### 5.3 — Remove legacy resize path

**Files affected:**

- Components previously using internal resize logic.

**Changes:**

- Set `legacyResize` default to `false`; delete `legacyResize` path.
- Remove any `ResizeObserver` instances from presentational components.

**Acceptance criteria:**

- [ ] No `ResizeObserver` or `resize` event listeners in `ui/packages/components/src/`.
- [ ] `cargo xtask ci-check --full` passes.

---

### Phase 5 Risk Controls

| Risk                                                              | Control                                                                    |
| ----------------------------------------------------------------- | -------------------------------------------------------------------------- |
| Resize jitter during transition                                   | `legacyResize: true` keeps old path active until declarative path verified |
| WKWebView and browser-dev mode have different resize event timing | Test in both modes before removing legacy path                             |

### Phase 5 Rollback

- Set `legacyResize: true`; revert to prior path.

### Phase 5 PR Slices

```
feat(template): add declarative ResizeObserver in smart container root     ← Task 5.2
refactor(components): remove legacy resize paths and legacyResize prop     ← Task 5.3
```

---

## Implementation Risk Summary

| Risk                                        | Phase     | Mitigation                                                         |
| ------------------------------------------- | --------- | ------------------------------------------------------------------ |
| Focus ring breaks in WKWebView              | 1         | Test in plugin host after Phase 1; record in test plan             |
| Token substitution changes visual weight    | 2         | Per-component before/after screenshot comparison                   |
| `IpcMethods` constant value differs         | 3         | Values are identical strings; no wire format change                |
| Hook-lift causes rendering regression       | 4         | `legacyProps` fallback; `cargo xtask ci-check` gate               |
| Resize jitter during observer transition    | 5         | `legacyResize: true` gate; validate in both modes                  |
| New coupling regressions introduced post-PR | Ongoing   | ESLint `import/no-restricted-paths` + `no-restricted-syntax` rules |

---

## PR Strategy

Each phase maps to one or more small, focused PRs (see per-phase PR Slices). Merged incrementally, targeting `main`:

1. **Phase 0 PR** — visual baseline + guardrails (no runtime behavior change)
2. **Phase 1 PRs** — focus rings + interaction states
3. **Phase 2 PRs** — token normalization (parallel to Phase 1)
4. **Phase 3 PRs** — IPC constants
5. **Phase 4 PRs** — smart/presentational split, hook lift, cleanup
6. **Phase 5 PRs** — resize unification

---

## Definition of Done

Feature is considered done when:

- [ ] All six phases completed for in-scope surfaces
- [ ] Every PR has passed `cargo xtask ci-check`
- [ ] Visual QA caveats for focus and interaction states are closed (evidence in test plan)
- [ ] Keyboard + a11y pass completed and documented
- [ ] ESLint rules — no active suppressions remaining in `@wavecraft/components`
- [ ] No `@wavecraft/core` imports remain in `@wavecraft/components`
- [ ] No raw IPC method strings at call sites
- [ ] Single `ResizeObserver` authority per surface confirmed
- [ ] All phases independently revertable confirmed
- [ ] `test-plan.md` updated with final test results and release recommendation
