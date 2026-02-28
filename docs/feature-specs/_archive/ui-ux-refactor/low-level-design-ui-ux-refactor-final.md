# Low-Level Design: UI/UX Refactor — Final (Code-Minimization Focus)

**Status:** Draft  
**Date:** 2026-02-21  
**Scope:** `sdk-template/ui/src/**`, `ui/packages/components/src/**`, `ui/packages/core/src/**`

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards — TypeScript & React](../../architecture/coding-standards-typescript.md) — Conventions for React code
- [CSS & Styling Standards](../../architecture/coding-standards-css.md) — TailwindCSS and token conventions
- [Roadmap](../../roadmap.md) — Project milestones

---

## 1. Executive Summary

**Directive: delete and consolidate before adding anything new.**

The recent UI sweeps have identified four concrete duplication clusters and one structural layer-confusion problem. This document defines the minimum set of changes required to eliminate them. No new features, no new abstractions introduced without a corresponding deletion. Every proposed change must reduce net line count, flatten tree depth, or remove a redundant export.

The guiding principle is: if two things look the same, one of them should not exist.

---

## 2. Current-State Problem Map

### 2.1 Thin Processor Wrapper Fan-out (`@wavecraft/components`)

**Files:**

- `ui/packages/components/src/InputTrimProcessor.tsx` — 13 LOC
- `ui/packages/components/src/OutputGainProcessor.tsx` — 12 LOC
- `ui/packages/components/src/OscillatorProcessor.tsx` — 15 LOC
- `ui/packages/components/src/SoftClipProcessor.tsx` — 14 LOC
- `ui/packages/components/src/ToneFilterProcessor.tsx` — 14 LOC

**Problem:** Each file is a one-expression wrapper: `<Processor id="<hardcoded>" title={title} parameters={parameters} />`. The props are `Omit<ProcessorProps, 'id'>`, meaning the caller still wires data and passes `parameters` in manually. These components add no behavior, no layout variation, and no type narrowing. They are pure namespace aliases for a string literal.

Additionally, because these wrappers still demand an external `parameters` prop — unlike the template's `SmartProcessor` which self-fetches — they are architecturally incomplete. A consumer who imports `OutputGainProcessor` from `@wavecraft/components` must still source and wire parameter data themselves, making the component's existence misleading.

### 2.2 Duplicate Parameter Renderer Logic

**Files:**

- `ui/packages/components/src/Processor.tsx` — lines 31–68
- `ui/packages/components/src/ParameterGroup.tsx` — lines 45–75

**Problem:** Both components implement an identical per-parameter type dispatch: `bool` → `<ParameterToggle>`, `enum` → `<ParameterSelect>`, `float` → `<ParameterSlider>`. The two implementations differ only in their outer container and header typography. The render-parameter logic is copy-pasted, not shared. Any future parameter type (e.g., `int`, `string`) must be added in both places independently.

**Shared heading class string** (`"text-sm font-semibold uppercase tracking-wider text-gray-400"`) appears verbatim in both files, compounding the drift risk.

### 2.3 Meter L/R Duplication and Reduced-Motion Inconsistency

**File:** `ui/packages/components/src/Meter.tsx` (198 LOC total)

**Problem — structural duplication:**

The component maintains two parallel namespaced state/ref pairs:
- `clippedL` / `clippedR` — `useState`
- `clipLTimeoutRef` / `clipRTimeoutRef` — `useRef`
- `peakLDb` / `peakRDb`, `rmsLDb` / `rmsRDb`, `peakLPercent` / `peakRPercent`, `rmsLPercent` / `rmsRPercent`
- Two identical `useEffect` bodies (lines 37–51, lines 53–67)
- Two structurally identical JSX channel blocks (L, lines 130–165; R, lines 167–197)

Each channel block is ~30 JSX lines. There is no extracted `MeterChannel` sub-component or data-driven loop.

**Problem — reduced-motion inconsistency:**

Within the meter bar JSX, motion guard usage is mixed:
- RMS bar: `motion-safe:transition-[width] motion-safe:duration-100` ✅
- Peak bar: `motion-safe:transition-[width] motion-safe:duration-75` ✅
- Clip shadow: `motion-safe:transition-shadow motion-safe:duration-100` ✅
- dB readout: `transition-colors duration-100` ❌ — missing `motion-safe:` prefix

The dB readout color transition will animate even for users who prefer reduced motion.

### 2.4 Repeated Class Strings and Token Consistency Drift

**Files and occurrences:**

| Class string | Files |
|---|---|
| `"rounded-lg border border-plugin-border bg-plugin-surface p-4"` | `Processor.tsx` (line 24), `ParameterSlider.tsx` (line 52), `Meter.tsx` (line 105, line 116) |
| `"text-sm font-semibold uppercase tracking-wider text-gray-400"` | `Processor.tsx` (line 25), `ParameterGroup.tsx` (line 46) |
| `"space-y-2"` outer + `"space-y-3"` inner | `Processor.tsx` (lines 23, 29), `ParameterGroup.tsx` (lines 44, 49) |

These strings also bypass the `classNames.ts` utility pattern already established for `focusRingClass` and `interactionStateClass`, meaning they are invisible to global search-and-replace when tokens change.

### 2.5 Thin Processor Wrapper Fan-out (Template)

**Files:**

- `sdk-template/ui/src/processors/InputTrimProcessor.tsx` — 17 LOC
- `sdk-template/ui/src/processors/OutputGainProcessor.tsx` — 17 LOC
- `sdk-template/ui/src/processors/OscillatorProcessor.tsx` — 17 LOC
- `sdk-template/ui/src/processors/OscilloscopeProcessor.tsx` — similar
- `sdk-template/ui/src/processors/SoftClipProcessor.tsx` — 17 LOC
- `sdk-template/ui/src/processors/ToneFilterProcessor.tsx` — 17 LOC
- `sdk-template/ui/src/processors/ExampleProcessor.tsx` — 17 LOC

**Problem:** Each file is a one-expression pass-through to `SmartProcessor` with a hardcoded `id` and `title`. The `SmartProcessor` abstraction already accepts `id` and `title` directly, so all named wrappers are unnecessary indirections in the template tree.

`App.tsx` in the template imports **7 separate named components** from 7 separate files; it could instead import `SmartProcessor` once and render it 7 times with inline `id`/`title` props, eliminating all 7 wrapper files.

### 2.6 Deprecated Hook/Export Aliases in Core

**File:** `ui/packages/core/src/hooks/useAllParameterFor.ts`

Two parallel exports remain active:
- `useParametersForProcessor` (canonical)
- `useAllParametersFor` (deprecated alias, same implementation)
- `UseParametersForProcessorResult` (canonical type)
- `UseAllParameterForResult` (deprecated alias type)

Both are re-exported in `ui/packages/core/src/index.ts`. The deprecated pair is marked with `@deprecated` JSDoc in the hook file, but the `index.ts` exports carry no deprecation annotation at the barrel level, making the alias as discoverable as the canonical form.

### 2.7 `@wavecraft/core` — Barrel Complexity and Polling Duplication

#### Barrel Export Discoverability

**File:** `ui/packages/core/src/index.ts`

The barrel re-exports every public symbol flat. With no grouping, `@deprecated` annotations on the implementation file do not surface when a consumer auto-completes from `@wavecraft/core`, because the barrel re-export line itself carries no annotation. This makes deprecated aliases (`useAllParametersFor`, `UseAllParameterForResult`) as discoverable as canonical exports. The barrel also does not distinguish between the primary entrypoint (`.`) and the meters subpath (`./meters`), increasing the risk of future accidental exports into the wrong subpath.

#### Deprecated Alias Shape and Test Coupling

**Files:**

- `ui/packages/core/src/hooks/useAllParameterFor.ts` — exports both the canonical `useParametersForProcessor` and the deprecated alias `useAllParametersFor` from the same module
- `ui/packages/core/src/hooks/useAllParameterFor.test.ts` — test assertions reference the deprecated alias name directly, coupling the test file to the symbol that is slated for removal

Both the type alias (`UseAllParameterForResult`) and the hook alias (`useAllParametersFor`) are re-exported without `@deprecated` from `index.ts`, giving consumers no IDE signal that the barrel-level export is non-canonical.

#### Duplicate Polling/Subscription Mechanics

**Files:**

- `ui/packages/core/src/hooks/useMeterFrame.ts`
- `ui/packages/core/src/hooks/useLatencyMonitor.ts`
- `ui/packages/core/src/hooks/useOscilloscopeFrame.ts`
- `ui/packages/core/src/hooks/useAudioStatus.ts`

Each of these hooks independently implements a subscribe-on-mount / unsubscribe-on-cleanup pattern with an interval or callback registration, similar effect dependency arrays, and similar null-guard patterns around the IPC bridge. The mechanics are not shared; changes to connection-aware polling behaviour (e.g., pausing when disconnected) must be applied to each file independently.

### 2.8 WavecraftProvider Complexity

**File:** `ui/packages/core/src/context/WavecraftProvider.tsx`

**Problem — mixed responsibilities in one file:**

`WavecraftProvider` conflates five distinct concerns without any internal boundary between them:

1. **Fetch lifecycle** — mounting, triggering `attemptFetch`, abort-on-unmount guard.
2. **Reconnect / timeout loop** — exponential-backoff retry, `retryTimeoutRef`, connection-state polling.
3. **Optimistic writes and rollback race handling** — pending-write tracking, rollback on `setParameter` failure, ordering of concurrent write acknowledgements.
4. **Notification subscriptions** — `parameterChanged` and `PARAMETERS_CHANGED` bridge event wiring and cleanup.
5. **Value normalization / update helpers** — per-parameter coerce, merge-into-state, and display-value derivation.

All five live in the same component function body, making the component body deep and hard to audit in isolation.

**Problem — helper sprawl:**

The following private functions are defined inline, with no grouping or co-location signal:

- `attemptFetch` — issues the fetch, wires the abort signal, handles the happy path.
- `handleSuccessResult` / `handleStopResult` — divergent post-fetch state transitions.
- Normalization helpers — duplicate value-coerce logic (e.g., float clamp, bool cast).
- Rollback helpers — restore optimistic state on write failure; rely on closure-captured snapshot of pre-write state.

The helpers are interspersed with effect hooks, increasing cognitive overhead when tracing a single data path from bottom to top.

**Problem — repeated guards:**

Identical `if (!mounted || !isConnected)` early-exits appear at the top of multiple effects and callbacks. The mounted-ref pattern (`useRef<boolean>`) is reproduced locally instead of using a shared utility.

---

## 3. Target Architecture

### 3.1 Shared Parameter Renderer

Extract the parameter-type dispatch into a single utility function or component in `@wavecraft/components`:

```
ui/packages/components/src/utils/renderParameter.tsx   (new)
```

Both `Processor` and `ParameterGroup` delegate rendering to this shared function. The outer container and header remain each component's own concern.

### 3.2 Consolidated Meter Channel

Extract the per-channel state/ref/effect/JSX into a private `MeterChannel` component within `Meter.tsx`. The parent `Meter` drives a data array of two entries (`[{ side: 'L', peak, rms, clipped }, { side: 'R', ... }]`) and maps over them.

### 3.3 Component-Level Class Constants

Add named constants to `ui/packages/components/src/utils/classNames.ts` for recurring structural class strings:

```typescript
export const surfaceCardClass =
  'rounded-lg border border-plugin-border bg-plugin-surface p-4';

export const sectionHeadingClass =
  'text-sm font-semibold uppercase tracking-wider text-gray-400';

export const parameterListClass = 'space-y-3';
```

### 3.4 Remove `@wavecraft/components` Thin Wrappers

Delete the five ID-alias wrappers from `@wavecraft/components`: `InputTrimProcessor`, `OutputGainProcessor`, `SoftClipProcessor`, `ToneFilterProcessor`, `OscillatorProcessor`. Re-export `Processor` and `SmartProcessor`-pattern documentation as the canonical composition path. Compatibility aliases (see §5) are maintained in a `compat.ts` re-export shim for one version cycle.

### 3.5 Collapse Template Processor Files

Delete all six named processor wrappers from `sdk-template/ui/src/processors/` (keep `SmartProcessor.tsx` and `ExampleProcessor.tsx` as the illustrative template entry point). Update `App.tsx` to use `SmartProcessor` directly with inline `id`/`title`.

### 3.6 Deprecated API Drain in Core

Add `@deprecated` JSDoc to the re-exports of `useAllParametersFor` and `UseAllParameterForResult` in `ui/packages/core/src/index.ts`. Mark for removal in the next minor version.

### 3.7 WavecraftProvider Internal Decomposition

**Public API surface is unchanged.** No new exports. No hook signature changes. This is a file-internal restructuring only.

Split the implementation into private co-located modules within `ui/packages/core/src/context/`:

```
ui/packages/core/src/context/
  WavecraftProvider.tsx           (public — unchanged API surface)
  _fetchController.ts             (private — fetch lifecycle, retry/backoff, abort)
  _writeReconciler.ts             (private — optimistic writes, rollback, race ordering)
  _subscriptionWiring.ts          (private — parameterChanged + PARAMETERS_CHANGED wiring)
  _valueHelpers.ts                (private — normalization, coerce, display-value derivation)
```

All four private modules are `_`-prefixed to signal non-export intent. None appear in `index.ts` or any subpath export.

`WavecraftProvider.tsx` becomes a thin orchestrator: it imports and composes the four modules, owns React context creation and the `<Provider>` render, and remains the only file in the folder that a consumer ever interacts with.

**Rollback boundary:** each private module can be inlined again independently if extraction introduces unexpected closure or ordering issues. The public component is unaffected in either direction.

---

## 4. Prioritized Plan

### P0 — Correctness / Token Safety (Do First)

These items fix bugs or silent drift that can affect user experience today.

| # | Change | Files touched | Est. net LOC Δ |
|---|--------|---------------|----------------|
| P0-1 | Add `motion-safe:` to dB readout `transition-colors` in `Meter.tsx` | `Meter.tsx` | −0 (+1 qualifier, net neutral) |
| P0-2 | Extract `surfaceCardClass`, `sectionHeadingClass`, `parameterListClass` into `classNames.ts`; replace all occurrences in `Processor.tsx`, `ParameterGroup.tsx`, `ParameterSlider.tsx`, `Meter.tsx` | `classNames.ts`, `Processor.tsx`, `ParameterGroup.tsx`, `ParameterSlider.tsx`, `Meter.tsx` | −10 to −15 |
| P0-3 | Add `@deprecated` JSDoc to barrel re-exports of `useAllParametersFor` and `UseAllParameterForResult` in `ui/packages/core/src/index.ts`; add inline comment `// compat: remove after next minor` as a removal-gate marker | `ui/packages/core/src/index.ts` | +2 doc lines |

**Expected net P0 reduction:** −10 to −15 LOC. Zero behavior change.

---

### P1 — Active Duplication (High Value)

These items eliminate the largest duplication clusters with low API-surface risk.

| # | Change | Files touched | Est. net LOC Δ |
|---|--------|---------------|----------------|
| P1-1 | Extract shared `renderParameter(param, key)` utility into `utils/renderParameter.tsx`; replace duplicate dispatch in `Processor.tsx` (lines 31–68) and `ParameterGroup.tsx` (lines 45–75) | `utils/renderParameter.tsx` (new ~30 LOC), `Processor.tsx` (−35), `ParameterGroup.tsx` (−30) | −35 to −40 |
| P1-2 | Extract private `MeterChannel` into `Meter.tsx`; replace two parallel state/ref/effect/JSX channel blocks with a mapped array | `Meter.tsx` | −45 to −55 |
| P1-3 | Delete 5 thin wrappers from `@wavecraft/components` (`InputTrimProcessor`, `OutputGainProcessor`, `SoftClipProcessor`, `ToneFilterProcessor`, `OscillatorProcessor`); add backward-compatible re-exports in `compat.ts` shim | 5 deleted files (~65 LOC), `compat.ts` (new ~25 LOC), `index.ts` (−5 direct exports, +1 compat re-export) | −40 to −50 |
| P1-4 | Extract a private internal helper (e.g., `hooks/_usePollingSubscription.ts`, `_`-prefixed to signal non-public) consolidating the subscribe-on-mount / unsubscribe-on-cleanup mechanics shared by `useMeterFrame`, `useLatencyMonitor`, `useOscilloscopeFrame`, and `useAudioStatus`; **not exported from `index.ts`** | `hooks/_usePollingSubscription.ts` (new, ~25 LOC), 4 existing hook files refactored | −25 to −35 |

| P1-5 | Split `WavecraftProvider.tsx` internals into four `_`-prefixed private modules (`_fetchController.ts`, `_writeReconciler.ts`, `_subscriptionWiring.ts`, `_valueHelpers.ts`); `WavecraftProvider.tsx` becomes a thin orchestrator; **no change to public API or Context type** | `ui/packages/core/src/context/WavecraftProvider.tsx` (−60 to −80 net after delegation), 4 new private files (~120 LOC total extracted) | −50 to −70 net (extracted code is reorganised, not deleted, but provider body shrinks significantly) |

**Expected net P1 reduction:** −195 to −250 LOC.

---

### P2 — Template Cleanup and API Hygiene (Lower Urgency)

| # | Change | Files touched | Est. net LOC Δ |
|---|--------|---------------|----------------|
| P2-1 | Delete 6 thin template processor wrappers from `sdk-template/ui/src/processors/`; update `App.tsx` to use `SmartProcessor` directly | 6 deleted files (~102 LOC), `App.tsx` (rewire imports, ~+15 LOC change) | −85 to −95 |
| P2-2 | Add `@deprecated` barrel annotation to `useAllParametersFor` / `UseAllParameterForResult` in `index.ts` | `ui/packages/core/src/index.ts` | −0 (doc-only) |
| P2-3 | Remove `OscilloscopeProcessor` from `@wavecraft/components` if it duplicates the template version (verify — may be intentionally SDK-provided) | TBD after verification | TBD |
| P2-4 | Update `useAllParameterFor.test.ts` to reference canonical `useParametersForProcessor`; retain a single alias import only as an explicit backwards-compat assertion (smoke-test that the alias still resolves) | `ui/packages/core/src/hooks/useAllParameterFor.test.ts` | −0 to −5 |
| P2-5 | Document staged alias drain criteria in `CHANGELOG`: `useAllParametersFor` / `UseAllParameterForResult` are eligible for hard removal after the next minor version bump; removal is blocked until the `// compat: remove after next minor` marker inserted in P0-3 is actioned | `CHANGELOG` | doc-only |

**Expected net P2 reduction:** −85 to −100 LOC (excluding P2-3 pending verification).

---

### Total Expected Net Reduction

| Priority | Range |
|----------|-------|
| P0 | −10 to −15 LOC |
| P1 | −195 to −250 LOC |
| P2 | −85 to −100 LOC |
| **Total** | **−290 to −365 LOC** |

---

## 5. API Compatibility Guardrails

All public exports from `@wavecraft/components` are part of the SDK's published API surface. Deletion without a compatibility bridge is a breaking change.

**Strategy: one-cycle deprecation shim.**

For every deleted named processor export from `@wavecraft/components`, add a re-export to a new `ui/packages/components/src/compat.ts` file that:
1. Re-exports the symbol by its original name via `Processor` with the ID pre-bound.
2. Adds a `@deprecated` JSDoc comment to the re-export.
3. Is included in `index.ts` under its existing export name.

```typescript
// ui/packages/components/src/compat.ts

/** @deprecated Use <Processor id="input_trim" .../> directly. */
export function InputTrimProcessor(props: Omit<ProcessorProps, 'id'>) {
  return <Processor id="input_trim" {...props} />;
}
// ... repeat for OutputGain, SoftClip, ToneFilter, Oscillator
```

The shim is removed in the next minor version after template consumers have migrated to direct `Processor`/`SmartProcessor` usage.

For `useAllParametersFor` in `@wavecraft/core`: the existing `@deprecated` annotation in the implementation file is sufficient. Add the same annotation to the barrel re-export in `index.ts`.

**Hard rule:** No symbol removal without: (a) a deprecation shim live for at least one version, and (b) a migration note in `CHANGELOG`.

### `@wavecraft/core` Provider Refactor Guardrails

The `WavecraftProvider` decomposition (§3.7 / P1-5) is strictly internal to `@wavecraft/core`:

- **No new public exports.** None of the four private modules (`_fetchController`, `_writeReconciler`, `_subscriptionWiring`, `_valueHelpers`) may appear in `index.ts`, in `package.json` `exports`, or in any subpath export map.
- **No behavior drift for hooks.** All hooks that consume `WavecraftContext` (`useParameter`, `useAllParameters`, `useConnectionStatus`, etc.) must exhibit identical behavior before and after P1-5 lands. The context shape and value derivation are unchanged.
- **Rollback race behavior preserved.** The write-rollback ordering guarantee — that a failed `setParameter` restores the pre-write snapshot even when a concurrent successful write has landed — must be explicitly preserved in `_writeReconciler.ts`. See §7 acceptance checks.
- **No change to `WavecraftProvider` props or Context type.** The component signature (`children`) and the context object shape are frozen for this PR.

### `@wavecraft/core` Entrypoint Stability

The two package export entrypoints — `.` (primary barrel) and `./meters` (audio math utilities) — are stable and must not be modified or supplemented by this refactor:

- **Do not add a third entrypoint.** The polling helper extracted in P1-4 (`_usePollingSubscription.ts`) is a `_`-prefixed private module; it must not appear in `package.json` `exports`, in `index.ts`, or in any subpath export map.
- **Do not remove any currently exported symbol in this PR.** Deprecated symbols receive a `@deprecated` JSDoc annotation in `index.ts` (P0-3); hard removal is gated on the criteria in P2-5.
- **No new public API surface.** The internal consolidation of polling mechanics is an implementation detail. Consumers retain the same hook signatures and import paths after P1-4.

---

## 6. Accessibility and Design-Token Acceptance Gates

A change in this refactor is accepted only when all of the following pass:

### Accessibility Gates

- All interactive elements (`<button>`, `<input>`, parameter controls) retain `focusRingClass` from `utils/classNames.ts`. **No bare `outline-none` or `focus:outline-none` without a visible replacement.**
- Reduced-motion: every animated property in `Meter.tsx`, `ParameterSlider.tsx`, `ConnectionStatus.tsx` uses `motion-safe:` prefix. No bare `transition-*` or `duration-*` classes on decorative animations.
- All `<input type="range">` elements retain an associated `<label>` element with matching `htmlFor`/`id` pairing.
- Clip reset button in `Meter` retains `title` attribute and keyboard activation.
- Extracted `MeterChannel` component: each channel `<div>` retains its `data-testid` (`meter-L`, `meter-R`) and `aria`-compatible structure.

### Design-Token Gates

- No raw hex colors (`#...`) or raw Tailwind palette shades (`gray-400`, `orange-500`) outside of `tailwind.config.js` token definitions — **except** for `gray-400` used as a de-emphasis color which is an accepted convention until a semantic token is defined.
- `surfaceCardClass`, `sectionHeadingClass`, `parameterListClass` (new constants) are the only source of their respective class strings across the component tree. No occurrences remain as inline string literals.
- `focusRingClass` and `interactionStateClass` remain the only source of focus/interaction styling.
- Meter gradient stops (`from-meter-safe`, `to-meter-safe-light`, `via-meter-warning`) continue to reference token names, not raw colors.

---

## 7. Verification Matrix

### Automated

| Check | Tool | Pass criteria |
|-------|------|---------------|
| Type safety | `tsc --noEmit` | Zero errors |
| Lint | ESLint + `@typescript-eslint` | Zero errors, zero warnings |
| Formatting | Prettier | No diff |
| Unit tests — components | Vitest | All existing tests pass (no deletions of test files) |
| Unit tests — core | Vitest | All existing tests pass |
| Accessibility snapshot | Existing `Meter.test.tsx`, `ParameterSlider.test.tsx` | No `data-testid` regressions |
| CI full suite | `cargo xtask ci-check` | All 6 phases pass |

### Manual

| Check | Steps | Pass criteria |
|-------|-------|---------------|
| Meter channel parity | Load plugin, generate audio signal; observe L and R meters | Both channels animate, clip indicator fires independently, reset works |
| Reduced-motion | Enable "Reduce Motion" in macOS Accessibility settings; reload plugin UI | Meter bars and dB readout do not animate; no transition fires |
| Parameter rendering | Load a processor with float, bool, and enum parameters | All three control types render correctly |
| Deprecated wrapper compatibility | Import `InputTrimProcessor` from `@wavecraft/components` in a test harness | Renders identically to `<Processor id="input_trim" .../>` |
| Template App.tsx | Run `wavecraft start` on generated template plugin | All processors render; no console errors |
| Focus traversal | Tab through all parameter controls | Focus ring visible on each control; no trap |

### Provider Parity Acceptance Checks (P1-5)

Before P1-5 is considered complete, all of the following must pass in addition to the automated matrix above:

| Check | Method | Pass criteria |
|-------|--------|---------------|
| Existing `WavecraftProvider` tests | Vitest (`ui/packages/core/src/context/*.test.*`) | All tests pass without modification |
| Rollback race invariant | Existing race-condition test case (simulate concurrent write + failure) | Pre-write value is restored even when a concurrent write lands between issue and failure |
| Hook behavior parity | Run full hook test suite (`useParameter`, `useAllParameters`, `useConnectionStatus`) | Zero behavior regressions; returned values and update cadence identical |
| No new barrel exports | Grep `src/context/_*.ts` names in `index.ts` | Zero matches |
| `WavecraftProvider` props unchanged | `tsc --noEmit` on a consumer that uses current prop signature | Zero type errors |

---

### Rollback Strategy

All P0/P1/P2 changes are scoped to `ui/` and `sdk-template/ui/`. The Rust engine is untouched.

1. No structural changes are merged without a passing `cargo xtask ci-check --full` run.
2. Each priority tier (P0, P1, P2) is a separate commit or PR. Revert granularity is per-tier.
3. The `compat.ts` shim ensures that any consumer using the old named exports will continue to compile and run correctly after P1 lands. There is no window of breaking change.
4. If `MeterChannel` extraction introduces visual regression, the extracted component is inlined again; the only change is locality, not behavior.

---

## 8. Risks and Non-Goals

### Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `MeterChannel` extraction changes timing of clip state per channel | Low | Medium — clip indicator mismatch | Keep state and refs scoped to the extracted component, not lifted to parent; test independently |
| Template App.tsx rewrite breaks generated plugin compile | Low | High — template validation CI fails | Run `cargo xtask ci-check --full` (includes template validation phase) before merge |
| Deprecated shim is missed and symbol removed early | Low | Medium — SDK breaking change | Add lint rule or CI check for direct imports of deprecated names |
| Token constant extraction misses an occurrence | Low | Low — visual inconsistency survives | Follow up with `grep -rn` scan for literal strings post-merge |

### Non-Goals

- **No new components.** This refactor does not introduce new abstractions. `renderParameter` is a utility function extracted from existing logic, not a new API.
- **No behavior changes.** All rendering outcomes must be pixel-identical before and after.
- **No Rust engine changes.** The IPC layer, parameter protocol, and metering pipeline are out of scope.
- **No roadmap changes.** The roadmap is PO-owned and is not modified by this design.
- **No archive modifications.** Archived feature specs are read-only.
- **No new test files.** Existing tests are updated to reflect refactored internals (e.g., new `data-testid` locations if any shift), but no new test suites are required by this refactor alone.
- **No CSS bundle size target changes.** The current <10 KB gzipped target is maintained; class constant extraction has negligible impact on bundle size.
