# Implementation Plan: UI/UX Refactor — Final (Code-Minimization Focus)

**Status:** Ready for Execution  
**Date:** 2026-02-21  
**Scope:** `sdk-template/ui/src/**`, `ui/packages/components/src/**`, `ui/packages/core/src/**`  
**LLD Reference:** [low-level-design-ui-ux-refactor-final.md](./low-level-design-ui-ux-refactor-final.md)

---

## Related Documents

- [Low-Level Design](./low-level-design-ui-ux-refactor-final.md) — Design decisions, problem map, target architecture
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards — TypeScript & React](../../architecture/coding-standards-typescript.md) — React/TypeScript conventions
- [CSS & Styling Standards](../../architecture/coding-standards-css.md) — TailwindCSS and token conventions
- [Testing & Quality](../../architecture/coding-standards-testing.md) — Test patterns

---

## Execution Principle

**Delete and consolidate before adding anything new.** Every slice either reduces net LOC or reorganises code with a net-neutral footprint. No slice introduces new behaviour, new public API surface, or new abstractions beyond what is necessary to eliminate an identified duplication.

---

## Do Not Do — Scope Control

The following are explicitly out of scope. Do not implement them as part of any slice.

| Category | Prohibited action |
|---|---|
| New behaviour | No new features, component variants, or prop additions |
| New public API | No new exports in `index.ts`, no new package entry points |
| Engine / IPC | No changes to Rust engine, IPC protocol, `wavecraft-bridge`, or `wavecraft-protocol` |
| New test suites | No new test files; only update existing tests to track refactored internals |
| Roadmap edits | `docs/roadmap.md` is PO-owned; do not touch it |
| Archived specs | Nothing in `docs/feature-specs/_archive/` may be modified |
| CSS bundle target | Do not change the <10 KB gzipped CSS target or the Tailwind config token set |
| `compat.ts` removal | The compat shim lives for at least one full version cycle; do not remove it in this plan |
| New `contextShape` fields | `WavecraftProvider` context type shape is frozen for this PR batch |
| New subpath exports | Do not add a third entry point to `@wavecraft/core` or `@wavecraft/components` |

---

## Slice Overview and Dependency Order

```
S0  baseline checks          (no code changes)
 └─ S1  P0 token safety      (depends on S0 clean baseline)
     └─ S2  renderParameter  (depends on S1 classNames.ts additions)
         └─ S3  MeterChannel (can parallel with S2; depends on S1)
             └─ S4  compat shim + wrapper delete   (depends on S3 — MeterChannel must be stable first)
                 └─ S5  polling consolidation + alias test hygiene (can parallel with S4)
                     └─ S6  WavecraftProvider decomposition (depends on S5 — polling helper available)
                         └─ S7  template wrapper collapse   (depends on S4 — compat shim live)
```

All slices within the same indentation level that are marked "can parallel" may land in the same PR if the author chooses. Otherwise each slice is an independent, revertible commit.

---

## LOC Accounting Table

| Slice | Est. net LOC Δ | Cumulative net Δ |
|---|---|---|
| S0 | 0 | 0 |
| S1 | −10 to −15 | −10 to −15 |
| S2 | −35 to −40 | −45 to −55 |
| S3 | −45 to −55 | −90 to −110 |
| S4 | −40 to −50 | −130 to −160 |
| S5 | −30 to −40 | −160 to −200 |
| S6 | −50 to −70 (net; extracted LOC reorganised, not deleted) | −210 to −270 |
| S7 | −85 to −95 | **−295 to −365** |

**Cumulative target: −290 to −365 net LOC across all slices.**

---

## Verification Commands

Run after each slice before merging:

```bash
# Standard CI (phases 0–3: docs, UI build, lint+typecheck, tests)
cargo xtask ci-check

# Full CI (adds template validation + CD dry-run) — required for S4, S6, S7
cargo xtask ci-check --full

# Auto-fix lint issues if needed
cargo xtask ci-check --fix

# Manual grep guard — no inline literal survivors after S1
grep -rn '"rounded-lg border border-plugin-border bg-plugin-surface p-4"' ui/packages/components/src/
grep -rn '"text-sm font-semibold uppercase tracking-wider text-gray-400"' ui/packages/components/src/
grep -rn '"space-y-3"' ui/packages/components/src/

# Guard: no private modules leaked to barrel (after S5, S6)
grep -n '_usePollingSubscription\|_fetchController\|_writeReconciler\|_subscriptionWiring\|_valueHelpers' \
  ui/packages/core/src/index.ts
# Expected: zero matches

# Guard: compat shim resolves (after S4)
grep -n 'InputTrimProcessor\|OutputGainProcessor\|SoftClipProcessor\|ToneFilterProcessor\|OscillatorProcessor' \
  ui/packages/components/src/index.ts
# Expected: each name appears exactly once (via compat re-export)

# Guard: no bare transition classes missing motion-safe prefix (after S1)
grep -n 'transition-colors duration-' ui/packages/components/src/Meter.tsx
# Expected: all hits include motion-safe: prefix
```

---

## Acceptance Matrix

### Automated (every slice)

| Check | Tool | Pass criteria |
|---|---|---|
| Type safety | `tsc --noEmit` | Zero errors |
| Lint | ESLint + `@typescript-eslint` | Zero errors, zero warnings |
| Formatting | Prettier | No diff |
| Unit tests — components | Vitest | All existing tests pass; no test file deleted |
| Unit tests — core | Vitest | All existing tests pass |
| `data-testid` snapshot | `Meter.test.tsx`, `ParameterSlider.test.tsx` | No `data-testid` regressions |
| Full CI suite | `cargo xtask ci-check` | All 6 phases pass |
| Template validation | `cargo xtask ci-check --full` | Template compiles and clippy passes (required S4+) |

### Manual / A11y (final integration check before S7 merge)

| Check | Steps | Pass criteria |
|---|---|---|
| Meter channel parity | Load plugin, generate audio signal | Both L and R channels animate; clip indicators fire independently; reset button clears each independently |
| Reduced-motion | Enable macOS "Reduce Motion"; reload UI | Meter bars, dB readout, and all animated properties do not transition |
| Parameter control rendering | Load processor with `float`, `bool`, and `enum` params | All three control types render correctly via shared `renderParameter` |
| Deprecated wrapper compat | Import `InputTrimProcessor` from `@wavecraft/components` in test harness | Renders identically to `<Processor id="input_trim" .../>` |
| Template App.tsx | Run `wavecraft start` on generated template plugin | All processors render; no console errors |
| Focus traversal | Tab through all parameter controls | Focus ring visible on every interactive control; no focus trap |
| Keyboard clip reset | Focus meter clip indicator button; press Enter/Space | Clip state resets; no mouse required |

### Provider Parity (S6 only)

| Check | Method | Pass criteria |
|---|---|---|
| Existing provider tests | Vitest on `ui/packages/core/src/context/*.test.*` | All pass without modification |
| Rollback race invariant | Simulate concurrent write + failure | Pre-write value restored even when a concurrent write lands between issue and failure acknowledgement |
| Hook behaviour parity | Full hook test suite (`useParameter`, `useAllParameters`, `useConnectionStatus`) | Zero behaviour regressions; returned values and update cadence identical |
| No barrel leakage | Grep private module names in `index.ts` | Zero matches |
| Provider prop stability | `tsc --noEmit` on a consumer with current prop signature | Zero type errors |

---

## S0 — Baseline Checks

### Objective

Confirm the repository is in a clean, passing state before any code changes land. Document current LOC per affected file as a diff baseline.

### Files Expected to Change

None.

### Implementation Checklist

- [ ] Run `cargo xtask ci-check` — all phases green
- [ ] Run `cargo xtask ci-check --full` — template validation and CD dry-run green
- [ ] Record current line counts for affected files (use `wc -l` or IDE) as a reference:
  - `ui/packages/components/src/Meter.tsx`
  - `ui/packages/components/src/Processor.tsx`
  - `ui/packages/components/src/ParameterGroup.tsx`
  - `ui/packages/components/src/ParameterSlider.tsx`
  - `ui/packages/components/src/utils/classNames.ts`
  - `ui/packages/core/src/index.ts`
  - `ui/packages/core/src/context/WavecraftProvider.tsx`
  - `sdk-template/ui/src/App.tsx`
- [ ] Verify no uncommitted local changes in the above files

### Tests to Add/Update

None.

### Risks + Rollback

No risk. No changes made. If CI is red, do not proceed to S1 until fixed.

### Completion Criteria

`cargo xtask ci-check --full` passes on the unmodified `main` branch.

---

## S1 — P0 Correctness / Token Safety

### Objective

Fix the `motion-safe:` omission on the dB readout in `Meter.tsx` and replace all four recurring inline class-string literals with named constants from `classNames.ts`. Zero behaviour change.

### Files Expected to Change

| File | Change type |
|---|---|
| `ui/packages/components/src/utils/classNames.ts` | Add three new exported constants |
| `ui/packages/components/src/Meter.tsx` | Replace inline literals; add `motion-safe:` to dB readout |
| `ui/packages/components/src/Processor.tsx` | Replace inline literals |
| `ui/packages/components/src/ParameterGroup.tsx` | Replace inline literals |
| `ui/packages/components/src/ParameterSlider.tsx` | Replace inline literals |
| `ui/packages/core/src/index.ts` | Add `@deprecated` JSDoc + `// compat: remove after next minor` comment to `useAllParametersFor` and `UseAllParameterForResult` re-export lines |

### Implementation Checklist

**`classNames.ts` additions:**
- [ ] Add `export const surfaceCardClass = 'rounded-lg border border-plugin-border bg-plugin-surface p-4';`
- [ ] Add `export const sectionHeadingClass = 'text-sm font-semibold uppercase tracking-wider text-gray-400';`
- [ ] Add `export const parameterListClass = 'space-y-3';`

**`Meter.tsx`:**
- [ ] Find the dB readout JSX element — locate `transition-colors duration-100` without a `motion-safe:` prefix
- [ ] Replace with `motion-safe:transition-colors motion-safe:duration-100`
- [ ] Replace all occurrences of `'rounded-lg border border-plugin-border bg-plugin-surface p-4'` with `surfaceCardClass`
- [ ] Import `surfaceCardClass` from `../utils/classNames`

**`Processor.tsx`:**
- [ ] Replace `'rounded-lg border border-plugin-border bg-plugin-surface p-4'` with `surfaceCardClass`
- [ ] Replace `'text-sm font-semibold uppercase tracking-wider text-gray-400'` with `sectionHeadingClass`
- [ ] Replace relevant `'space-y-3'` outer/inner usages with `parameterListClass`
- [ ] Import new constants from `./utils/classNames`

**`ParameterGroup.tsx`:**
- [ ] Replace `'text-sm font-semibold uppercase tracking-wider text-gray-400'` with `sectionHeadingClass`
- [ ] Replace relevant `'space-y-3'` usages with `parameterListClass`
- [ ] Import new constants from `./utils/classNames`

**`ParameterSlider.tsx`:**
- [ ] Replace `'rounded-lg border border-plugin-border bg-plugin-surface p-4'` with `surfaceCardClass`
- [ ] Import new constants from `./utils/classNames`

**`ui/packages/core/src/index.ts`:**
- [ ] Locate re-export of `useAllParametersFor` — add `/** @deprecated Use useParametersForProcessor instead. */` JSDoc above the line
- [ ] Locate re-export of `UseAllParameterForResult` — add `/** @deprecated Use UseParametersForProcessorResult instead. */` JSDoc above the line
- [ ] Add inline comment `// compat: remove after next minor` on the same line as each deprecated re-export

**Post-edit verification:**
- [ ] Run post-edit grep guards (see Verification Commands section) — zero inline literal survivors
- [ ] Run `cargo xtask ci-check`

### Tests to Add/Update

No new tests required. If existing snapshot tests assert on class names directly, update the expected class-string values to match the constants (behaviour is unchanged; only the source reference changes). Search `*.test.*` files for occurrences of the replaced literal strings and update accordingly.

### Risks + Rollback

| Risk | Mitigation |
|---|---|
| A grep miss leaves an unreplaced literal | Run post-edit grep guards before committing |
| `motion-safe:` prefix changes dB readout behaviour on non-reduced-motion users | No — the `motion-safe:` prefix enables the transition for standard users; it only suppresses animation for users with reduce-motion enabled. Behaviour is equivalent or improved |

Rollback: revert this commit. No downstream dependency.

### Completion Criteria

- `cargo xtask ci-check` passes
- Zero inline occurrences of the four replaced class strings in `ui/packages/components/src/`
- `Meter.tsx` dB readout contains `motion-safe:transition-colors`
- `index.ts` deprecated re-exports carry `@deprecated` JSDoc and `// compat:` comment

**Net LOC Δ: −10 to −15 (class literal deduplication) + ~2 doc lines (deprecation comments) ≈ −8 to −13**

---

## S2 — Shared Parameter Renderer Extraction

### Objective

Extract the duplicated per-parameter type-dispatch logic from `Processor.tsx` and `ParameterGroup.tsx` into a single shared utility function `renderParameter`. Eliminate ~65 duplicated lines across the two files.

### Files Expected to Change

| File | Change type |
|---|---|
| `ui/packages/components/src/utils/renderParameter.tsx` | **New file** (~30 LOC) |
| `ui/packages/components/src/Processor.tsx` | Replace per-param dispatch block with `renderParameter` call (−35 LOC) |
| `ui/packages/components/src/ParameterGroup.tsx` | Replace per-param dispatch block with `renderParameter` call (−30 LOC) |

### Implementation Checklist

**`utils/renderParameter.tsx` (new):**
- [ ] Create the file
- [ ] Define and export `function renderParameter(param: ParameterInfo, key: string): React.ReactNode`
- [ ] Implement the `bool` → `<ParameterToggle>`, `enum` → `<ParameterSelect>`, `float` → `<ParameterSlider>` dispatch (exact copy of one of the two existing implementations)
- [ ] Add a fallback `default` branch: `return null;` (with an optional `console.warn` in dev for unknown types)
- [ ] Do **not** export from `index.ts` — this is a component-internal utility

**`Processor.tsx`:**
- [ ] Import `renderParameter` from `./utils/renderParameter`
- [ ] Locate the per-parameter dispatch block (lines 31–68 in original)
- [ ] Replace the full dispatch block with `{parameters.map((param) => renderParameter(param, param.id))}`
- [ ] Remove any no-longer-needed local imports (`ParameterToggle`, `ParameterSelect`, `ParameterSlider` if they move into `renderParameter`)

**`ParameterGroup.tsx`:**
- [ ] Import `renderParameter` from `./utils/renderParameter`
- [ ] Locate the per-parameter dispatch block (lines 45–75 in original)
- [ ] Replace with `{parameters.map((param) => renderParameter(param, param.id))}`
- [ ] Remove any no-longer-needed local imports

**Post-edit:**
- [ ] Confirm both files still compile with `tsc --noEmit`
- [ ] Visually verify that a float, bool, and enum parameter all render correctly (manual test or snapshot)

### Tests to Add/Update

- Check `Processor.test.*` and `ParameterGroup.test.*` for any assertions on the dispatch tree — update `renderParameter` unit-level description if tests are present
- If no dedicated tests exist for the dispatch logic, add a minimal unit test in `renderParameter.test.tsx` covering all three parameter types and the `null` fallback. This is the one permissible new test file because the extracted function is new and previously untested in isolation.

### Risks + Rollback

| Risk | Mitigation |
|---|---|
| `renderParameter` key strategy differs from one of the two original implementations | Copy the key strategy from the canonical implementation; verify both files produced identical React keys |
| Future `int` or `string` parameter type added by only one caller | Adding to `renderParameter` covers both callers automatically — this is the desired outcome |

Rollback: delete `renderParameter.tsx`; restore inline dispatch blocks in both files. No downstream impact outside these three files.

### Completion Criteria

- `utils/renderParameter.tsx` exists and exports `renderParameter`
- Neither `Processor.tsx` nor `ParameterGroup.tsx` contains an inline `bool/enum/float` dispatch
- `cargo xtask ci-check` passes
- All three parameter types render in manual test

**Net LOC Δ: −35 to −40**

---

## S3 — Meter Channel Consolidation

### Objective

Extract the per-channel state/ref/effect/JSX from `Meter.tsx` into a private `MeterChannel` sub-component defined within the same file. Replace the two parallel channel blocks with a two-element data array mapped over `MeterChannel`. Eliminate ~50 duplicated lines.

### Files Expected to Change

| File | Change type |
|---|---|
| `ui/packages/components/src/Meter.tsx` | Extract `MeterChannel`; replace two parallel blocks with mapped array (−45 to −55 LOC net) |

No other files change.

### Implementation Checklist

**Define `MeterChannel` inside `Meter.tsx` (not exported):**
- [ ] Define interface `MeterChannelProps { side: 'L' | 'R'; peakDb: number; rmsDb: number; }` (or derive from existing meter state shape)
- [ ] Move the per-channel `useState` pair (`clipped`, `clipTimeoutRef`) into `MeterChannel`
- [ ] Move the per-channel `useEffect` body (signal subscription or derived value update) into `MeterChannel`
- [ ] Move the per-channel JSX block (~30 lines) into `MeterChannel`'s return
- [ ] Retain `data-testid="meter-L"` and `data-testid="meter-R"` on the channel root elements — pass `data-testid={`meter-${side}`}` as a prop or derive from `side`
- [ ] Retain the clip reset `<button>` with its `title` attribute and `onClick` handler inside `MeterChannel`

**Parent `Meter` component:**
- [ ] Remove the two parallel state pairs and keep only shared parent state (e.g., subscription hook, overall peak array)
- [ ] Replace the two JSX channel blocks with:
  ```tsx
  {(['L', 'R'] as const).map((side) => (
    <MeterChannel key={side} side={side} peakDb={...} rmsDb={...} />
  ))}
  ```
- [ ] Ensure all `motion-safe:` prefixes from S1 are preserved in the extracted JSX

**Post-edit:**
- [ ] Confirm `data-testid="meter-L"` and `data-testid="meter-R"` still exist in the rendered output
- [ ] Confirm clip indicator fires and resets independently per channel in manual test

### Tests to Add/Update

- `Meter.test.tsx`: verify `data-testid="meter-L"` and `data-testid="meter-R"` still present after extraction
- If any existing tests assert on the clip state per channel, update them to use the `data-testid` selectors rather than component internals

### Risks + Rollback

| Risk | Mitigation |
|---|---|
| `MeterChannel` extraction changes timing or ordering of clip state per channel | Scope state and refs inside `MeterChannel`; do not lift to parent. Clip state is per-channel by design |
| Visual layout differs after extraction | Extracted JSX is identical to original; no class strings change |

Rollback: inline `MeterChannel` back into the parent body. The public `Meter` component is unchanged.

### Completion Criteria

- `Meter.tsx` has a single private `MeterChannel` component
- No parallel `clippedL`/`clippedR`, `clipLTimeoutRef`/`clipRTimeoutRef`, or duplicate `useEffect` bodies in parent
- Both `data-testid="meter-L"` and `data-testid="meter-R"` pass in automated tests
- `cargo xtask ci-check` passes

**Net LOC Δ: −45 to −55**

---

## S4 — Components Wrapper Deletion + Compat Shim

### Objective

Delete the five thin ID-alias processor wrappers from `@wavecraft/components`. Add a `compat.ts` backward-compatibility shim that re-exports each wrapper as a `@deprecated` function. Maintain all five export names in `index.ts` via the shim.

### Files Expected to Change

| File | Change type |
|---|---|
| `ui/packages/components/src/InputTrimProcessor.tsx` | **Deleted** |
| `ui/packages/components/src/OutputGainProcessor.tsx` | **Deleted** |
| `ui/packages/components/src/SoftClipProcessor.tsx` | **Deleted** |
| `ui/packages/components/src/ToneFilterProcessor.tsx` | **Deleted** |
| `ui/packages/components/src/OscillatorProcessor.tsx` | **Deleted** |
| `ui/packages/components/src/compat.ts` | **New file** (~30 LOC) |
| `ui/packages/components/src/index.ts` | Remove 5 direct exports; add single `compat.ts` re-export block |

### Implementation Checklist

**`compat.ts` (new):**
- [ ] Create `ui/packages/components/src/compat.ts`
- [ ] Import `Processor` and `ProcessorProps` from `./Processor`
- [ ] Define and export each deprecated wrapper function:
  ```typescript
  /** @deprecated Use <Processor id="input_trim" .../> directly. */
  export function InputTrimProcessor(props: Omit<ProcessorProps, 'id'>) {
    return <Processor id="input_trim" {...props} />;
  }
  ```
- [ ] Repeat for `OutputGainProcessor` (`id="output_gain"`), `SoftClipProcessor` (`id="soft_clip"`), `ToneFilterProcessor` (`id="tone_filter"`), `OscillatorProcessor` (`id="oscillator"`)
- [ ] Each function carries a `@deprecated` JSDoc comment
- [ ] File does **not** export anything else

**Delete files:**
- [ ] Delete `InputTrimProcessor.tsx`
- [ ] Delete `OutputGainProcessor.tsx`
- [ ] Delete `SoftClipProcessor.tsx`
- [ ] Delete `ToneFilterProcessor.tsx`
- [ ] Delete `OscillatorProcessor.tsx`

**`index.ts` update:**
- [ ] Remove the five individual named export lines for the deleted files
- [ ] Add `export { InputTrimProcessor, OutputGainProcessor, SoftClipProcessor, ToneFilterProcessor, OscillatorProcessor } from './compat';`
- [ ] Verify the five names still resolve when imported from `@wavecraft/components`

**Post-edit:**
- [ ] Run `cargo xtask ci-check --full` (template validation required)
- [ ] Verify deprecated wrapper compat manual acceptance check (see Acceptance Matrix)

### Tests to Add/Update

- Add a smoke-test assertion in the component test suite confirming that importing `InputTrimProcessor` from `@wavecraft/components` renders without error and produces the same output as `<Processor id="input_trim" .../>`. This directly tests the compat shim.

### Risks + Rollback

| Risk | Mitigation |
|---|---|
| A consumer outside the monorepo hard-imports one of the deleted files directly (`import ... from '@wavecraft/components/InputTrimProcessor'`) | The subpath is not in `package.json` `exports`; barrel import path `@wavecraft/components` still resolves via shim |
| Compat ID string differs from original hardcoded ID in deleted wrapper | Verify by reading each deleted file before deleting; copy the exact `id` string |
| Template validation CI fails because `sdk-template` imported from `@wavecraft/components` | `sdk-template` uses `SmartProcessor`, not these wrappers; verify with `grep -rn 'from.*@wavecraft/components'` in `sdk-template/ui/src/` before delete |

Rollback: restore the five deleted files from git; remove the `compat.ts` shim; restore direct exports in `index.ts`.

### Completion Criteria

- Five wrapper files deleted from `ui/packages/components/src/`
- `compat.ts` exists with five `@deprecated` re-export functions
- All five names still export from `@wavecraft/components` barrel
- Compatible import renders correctly (manual check)
- `cargo xtask ci-check --full` passes

**Net LOC Δ: −40 to −50**

---

## S5 — Core Polling/Subscription Consolidation + Alias Test Decoupling

### Objective

Two independent tasks in one slice (can be split if preferred):

1. Extract a private `_usePollingSubscription` hook consolidating the subscribe-on-mount / unsubscribe-on-cleanup pattern shared across four core hooks.
2. Update `useAllParameterFor.test.ts` to reference canonical `useParametersForProcessor`; retain one explicit alias assertion for backward-compat smoke-testing.

### Files Expected to Change

| File | Change type |
|---|---|
| `ui/packages/core/src/hooks/_usePollingSubscription.ts` | **New file** (~25 LOC) — private, `_`-prefixed |
| `ui/packages/core/src/hooks/useMeterFrame.ts` | Refactored to use `_usePollingSubscription` |
| `ui/packages/core/src/hooks/useLatencyMonitor.ts` | Refactored to use `_usePollingSubscription` |
| `ui/packages/core/src/hooks/useOscilloscopeFrame.ts` | Refactored to use `_usePollingSubscription` |
| `ui/packages/core/src/hooks/useAudioStatus.ts` | Refactored to use `_usePollingSubscription` |
| `ui/packages/core/src/hooks/useAllParameterFor.test.ts` | Update assertions to reference canonical name; add one alias resolution smoke-test |

**Not changed:**
- `ui/packages/core/src/index.ts` — `_usePollingSubscription` must **not** be added here

### Implementation Checklist

**`_usePollingSubscription.ts` (new):**
- [ ] Create `ui/packages/core/src/hooks/_usePollingSubscription.ts`
- [ ] Define the generalised subscribe-on-mount / unsubscribe-on-cleanup pattern:
  - Accepts: `subscribe: () => (() => void)` (a function that registers a callback and returns an unsubscriber), `deps: readonly unknown[]`
  - Wraps the pattern in a `useEffect` with cleanup
  - Includes null-guard for the IPC bridge if needed (or delegates the guard into the subscribe callback)
- [ ] File is `_`-prefixed; add a comment `// private — do not export from index.ts`
- [ ] Do **not** add to `index.ts`

**Refactor existing hooks:**
- [ ] `useMeterFrame.ts`: replace manual subscribe/unsubscribe `useEffect` body with `_usePollingSubscription` call
- [ ] `useLatencyMonitor.ts`: same
- [ ] `useOscilloscopeFrame.ts`: same
- [ ] `useAudioStatus.ts`: same
- [ ] Verify each refactored hook has identical exported signature and return shape

**`useAllParameterFor.test.ts`:**
- [ ] Update test descriptions and primary assertions to reference `useParametersForProcessor` (canonical)
- [ ] Add a single test case: `it('useAllParametersFor alias resolves to same hook', ...)` that imports the deprecated alias and asserts it produces identically-shaped output — this is the explicit backward-compat smoke-test
- [ ] Remove any test assertions that only exist as a side-effect of the deprecated name (not as intentional compat assertions)

**Guard check:**
- [ ] Run `grep -n '_usePollingSubscription' ui/packages/core/src/index.ts` — must return zero matches

### Tests to Add/Update

- `useAllParameterFor.test.ts` (updated): canonical name as primary; alias assertion isolated in one explicit test case
- Existing tests for all four polling hooks must pass without modification to verify behaviour parity after refactor

### Risks + Rollback

| Risk | Mitigation |
|---|---|
| One of the four hooks has a subtle difference in its subscribe/unsubscribe lifecycle that the generic helper does not accommodate | Implement `_usePollingSubscription` to be flexible (callback-based subscribe, not opinionated) to accommodate differences; fallback: leave that one hook un-refactored rather than forcing it |
| Test changes accidentally remove a compat assertion | The explicit `'alias resolves to same hook'` test case is the required compat assertion; do not remove it |

Rollback: inline the `useEffect` pattern back into each hook; delete `_usePollingSubscription.ts`. Revert test changes separately.

### Completion Criteria

- `_usePollingSubscription.ts` exists and is not in `index.ts`
- All four polling hooks pass their existing test suites
- `useAllParameterFor.test.ts` uses canonical name as primary with one explicit alias smoke-test
- `cargo xtask ci-check` passes

**Net LOC Δ: −30 to −40**

---

## S6 — WavecraftProvider Internal Decomposition (Private Modules Only)

### Objective

Split the `WavecraftProvider.tsx` implementation into four `_`-prefixed private co-located modules. The public API surface, context shape, component signature, and all consumer-facing hooks are **completely unchanged**. Net effect: a shorter, auditable provider body; no behaviour change.

### Files Expected to Change

| File | Change type |
|---|---|
| `ui/packages/core/src/context/WavecraftProvider.tsx` | Reduced to thin orchestrator (~60–80 LOC removed; delegated to private modules) |
| `ui/packages/core/src/context/_fetchController.ts` | **New private file** — fetch lifecycle, retry/backoff, abort |
| `ui/packages/core/src/context/_writeReconciler.ts` | **New private file** — optimistic writes, rollback, race ordering |
| `ui/packages/core/src/context/_subscriptionWiring.ts` | **New private file** — `parameterChanged` + `PARAMETERS_CHANGED` bridge event wiring |
| `ui/packages/core/src/context/_valueHelpers.ts` | **New private file** — normalization, coerce, display-value derivation |

**Not changed:**
- `ui/packages/core/src/index.ts` — none of the four private modules appear here
- `package.json` `exports` — no new entry point
- Any consumer of `WavecraftProvider` or `WavecraftContext`

### Implementation Checklist

**Pre-work:**
- [ ] Read `WavecraftProvider.tsx` in full and map each helper/effect to one of the four module categories
- [ ] Identify all closure captures per category — each private module must receive required state/refs as parameters rather than implicitly capturing from the provider closure

**`_fetchController.ts` (new):**
- [ ] Extract `attemptFetch`, `handleSuccessResult`, `handleStopResult`, and the reconnect/retry/backoff loop
- [ ] Export as typed functions or a factory function accepting required context values
- [ ] No direct React imports (no `useState`, `useRef`) — accepts refs/setters as arguments
- [ ] Add comment `// private — do not export from index.ts`

**`_writeReconciler.ts` (new):**
- [ ] Extract optimistic write tracking, rollback on `setParameter` failure, and concurrent write ordering logic
- [ ] Preserve the rollback race invariant: a failed write restores the pre-write snapshot even when a concurrent successful write has landed between issue and failure
- [ ] Export as typed functions or a class/object factory
- [ ] Add comment `// private — do not export from index.ts`

**`_subscriptionWiring.ts` (new):**
- [ ] Extract `parameterChanged` subscription registration and `PARAMETERS_CHANGED` bridge event wiring
- [ ] Export a `wireSubscriptions(bridge, onChanged, mounted)` function (or similar)
- [ ] Add comment `// private — do not export from index.ts`

**`_valueHelpers.ts` (new):**
- [ ] Extract all normalization helpers: float clamp, bool cast, enum coerce, display-value derivation, merge-into-state
- [ ] Export as pure functions (no side effects, no React hooks)
- [ ] Add comment `// private — do not export from index.ts`

**`WavecraftProvider.tsx` (updated):**
- [ ] Import the four private modules
- [ ] Replace inline implementations with calls to imported functions
- [ ] Retain React context creation and the `<Provider>` render
- [ ] Retain the `if (!mounted || !isConnected)` guard — or extract to a shared `useMountedConnection()` internal utility if the duplication within the provider warrants it (within-file extraction only)
- [ ] Provider component body targets ~80–120 LOC after extraction

**Guard checks:**
- [ ] `grep -n '_fetchController\|_writeReconciler\|_subscriptionWiring\|_valueHelpers' ui/packages/core/src/index.ts` — zero matches
- [ ] `grep -n 'providerProps\|children.*Provider' ui/packages/core/src/index.ts` — confirm `WavecraftProvider` export is unchanged

### Tests to Add/Update

- Do **not** modify `WavecraftProvider` tests. All existing tests must pass without change — this is the primary verification that behaviour is preserved.
- If any test currently tests a private helper by importing it directly (verify: there should be none), move the assertion to test via the public provider interface instead.

### Risks + Rollback

| Risk | Mitigation |
|---|---|
| Closure capture removed by extraction causes stale ref or state in `_writeReconciler` | Pass all required refs/setters explicitly as arguments; write unit-level assertions for the rollback race invariant before extracting |
| `_fetchController` reconnect loop loses access to a closure variable | Audit all closure captures before extraction; use a returned function pair (start/stop) pattern if needed |
| One private module's ordering assumption diverges from provider's expected call sequence | Keep the call site order in `WavecraftProvider.tsx` identical to the original effect ordering; document the required call sequence at the top of each private module |

Rollback: inline all four private modules back into `WavecraftProvider.tsx` from git. The public component is unaffected. Each private module can be inlined independently.

### Completion Criteria

- Four `_`-prefixed private modules exist in `ui/packages/core/src/context/`
- None of the four appear in `index.ts` or any export map
- `WavecraftProvider.tsx` body is materially shorter; all behaviour is delegated
- All Provider Parity acceptance checks pass (see Acceptance Matrix — S6)
- `cargo xtask ci-check --full` passes

**Net LOC Δ: −50 to −70 (extracted code reorganised, not deleted; provider body shrinks significantly)**

---

## S7 — Template Wrapper Collapse in `sdk-template` App

### Objective

Delete the six thin named processor wrappers from `sdk-template/ui/src/processors/` and collapse `App.tsx` to use `SmartProcessor` directly with inline `id`/`title` props. `ExampleProcessor.tsx` is retained as the illustrative template entry point (or renamed if appropriate).

### Files Expected to Change

| File | Change type |
|---|---|
| `sdk-template/ui/src/processors/InputTrimProcessor.tsx` | **Deleted** |
| `sdk-template/ui/src/processors/OutputGainProcessor.tsx` | **Deleted** |
| `sdk-template/ui/src/processors/OscillatorProcessor.tsx` | **Deleted** |
| `sdk-template/ui/src/processors/OscilloscopeProcessor.tsx` | **Deleted** (after confirming it is not a non-trivial component) |
| `sdk-template/ui/src/processors/SoftClipProcessor.tsx` | **Deleted** |
| `sdk-template/ui/src/processors/ToneFilterProcessor.tsx` | **Deleted** |
| `sdk-template/ui/src/App.tsx` | Replace 7 named imports with single `SmartProcessor` import; render inline |

**Retained (do not delete):**
- `sdk-template/ui/src/processors/SmartProcessor.tsx` — canonical self-fetching pattern
- `sdk-template/ui/src/processors/ExampleProcessor.tsx` — illustrative template for SDK users learning to add a custom processor

### Implementation Checklist

**Pre-work:**
- [ ] Read each wrapper file and confirm it is purely a pass-through to `SmartProcessor` with a hardcoded `id` and `title` — no additional logic, layout, or props
- [ ] Confirm `OscilloscopeProcessor.tsx` is also a thin pass-through (verify before deleting — if it contains non-trivial rendering, do not delete it and remove it from this slice's scope)

**Delete files:**
- [ ] Delete `InputTrimProcessor.tsx`  
- [ ] Delete `OutputGainProcessor.tsx`  
- [ ] Delete `OscillatorProcessor.tsx`  
- [ ] Delete `OscilloscopeProcessor.tsx` (if confirmed thin pass-through)  
- [ ] Delete `SoftClipProcessor.tsx`  
- [ ] Delete `ToneFilterProcessor.tsx`

**`App.tsx` update:**
- [ ] Remove 6–7 named processor imports
- [ ] Add single `import { SmartProcessor } from './processors/SmartProcessor';`
- [ ] Replace each `<InputTrimProcessor .../>`, `<OutputGainProcessor .../>`, etc. with:
  ```tsx
  <SmartProcessor id="input_trim" title="Input Trim" />
  <SmartProcessor id="output_gain" title="Output Gain" />
  {/* etc. */}
  ```
- [ ] Verify the `id` values match the exact processor IDs used in the Rust engine for each processor
- [ ] Retain `ExampleProcessor` import and render if it exists in the current `App.tsx`

**Post-edit:**
- [ ] Run `cargo xtask ci-check --full` — template validation phase must pass
- [ ] Run clippy on the generated template project:
  ```bash
  cargo run --manifest-path cli/Cargo.toml -- create TestPlugin --output target/tmp/test-plugin
  cd target/tmp/test-plugin/engine && cargo clippy --all-targets -- -D warnings
  cd ../.. && rm -rf target/tmp/test-plugin
  ```
- [ ] Run `wavecraft start` on generated template plugin — all processors render; no console errors

### Tests to Add/Update

No new test files. If template validation tests assert on specific component files by name, update the expected file list to reflect the deletions.

### Risks + Rollback

| Risk | Mitigation |
|---|---|
| `App.tsx` inline `id` string differs from the Rust-side processor ID | Cross-reference the generated `ui/src/generated/processors.ts` or the Rust processor registration for exact ID values before updating `App.tsx` |
| Template validation CI fails due to missing import | `cargo xtask ci-check --full` catches this before merge |
| `OscilloscopeProcessor.tsx` is non-trivial | Read the file before deleting; if non-trivial, remove it from this slice's scope |

Rollback: restore deleted files from git; revert `App.tsx`. No downstream impact outside the template.

### Completion Criteria

- Six (or five if `OscilloscopeProcessor` is non-trivial) processor wrapper files deleted from `sdk-template/ui/src/processors/`
- `App.tsx` imports `SmartProcessor` once; renders each processor with inline `id`/`title`
- `cargo xtask ci-check --full` passes (template validation phase green)
- `wavecraft start` on generated template plugin — all processors render

**Net LOC Δ: −85 to −95**

---

## Final Cumulative Target

| Slice | Net LOC Δ | Cumulative |
|---|---|---|
| S0 | 0 | 0 |
| S1 | −8 to −13 | −8 to −13 |
| S2 | −35 to −40 | −43 to −53 |
| S3 | −45 to −55 | −88 to −108 |
| S4 | −40 to −50 | −128 to −158 |
| S5 | −30 to −40 | −158 to −198 |
| S6 | −50 to −70 | −208 to −268 |
| S7 | −85 to −95 | **−293 to −363** |

Target: **−290 to −365 net LOC** across all slices. Zero behaviour change. Zero new public API. Zero Rust engine changes.
