# Implementation Plan: WavecraftProvider Parameter State

## Related Documents

- [Low-Level Design](./low-level-design-wavecraft-provider-parameter-state.md) — Architecture and design decisions
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards — TypeScript & React](../../architecture/coding-standards-typescript.md) — Conventions for hooks and components
- [Roadmap](../../roadmap.md) — Milestone tracking

---

## Overview

This plan introduces a centralized parameter state owner (`WavecraftProvider`) to consolidate duplicate IPC fetch paths, standardize write routing through context actions, and align hook naming to processor-centric semantics — all without breaking existing consumers.

---

## Implementation Sequence Summary

1. Add `WavecraftProvider` and context primitives (non-breaking).
2. Rewire existing hooks to consume provider state (remove duplicate fetch paths).
3. Migrate app/template usage away from direct `ParameterClient` writes.
4. Add/adjust tests to lock behavior and prevent regressions.
5. Run repo-standard verification gates (`ci-check`, sync checks, targeted tests).

---

## Scope and Constraints

- Docs-only planning artifact; no code changes in this step.
- Canonical provider naming: **`WavecraftProvider`**.
- Do **not** modify:
  - `docs/roadmap.md`
  - any file under `docs/feature-specs/_archive/`

---

## Explicit Migration Map

| Old API | New API | Notes |
|---|---|---|
| `useAllParametersFor` | `useParametersForProcessor` | Alias shim kept during transition |
| Direct `ParameterClient` usage in components | Hook/context actions via `WavecraftProvider` | Route writes through provider actions |

> **Migration policy:** Introduce alias/shim first (non-breaking), then migrate call sites, then remove old paths in a final cleanup PR.

---

## Dependency Ordering

1. Provider/context foundation (safe additive work).
2. Hook internals migration to provider (public API stable).
3. Consumer migration (`SmartProcessor` and related app surfaces).
4. Naming migration (`useAllParametersFor` → `useParametersForProcessor`) with compatibility layer.
5. Cleanup and enforcement (remove direct client usage patterns where appropriate).

---

## Phase 1 — Provider Foundation (PR 1)

### Objective

Introduce centralized parameter state ownership using `WavecraftProvider` without breaking existing hooks.

### Expected Files to Touch

- `ui/packages/core/src/context/ParameterStateContext.ts` *(new)*
- `ui/packages/core/src/context/WavecraftProvider.tsx` *(new)*
- `ui/packages/core/src/context/useParameterState.ts` *(new)*
- `ui/packages/core/src/index.ts` *(export provider and public types)*
- `sdk-template/ui/src/App.tsx` *(wrap app root with provider)*

### Tasks

- [ ] Create context types and provider state contract (`params`, `isLoading`, `error`, `reload`, `setParameter`).
- [ ] Port fetch/retry/invalidation behavior into provider.
- [ ] Add provider subscriptions for:
  - parameter push updates
  - hot-reload parameter schema invalidation
  - reconnect-triggered refresh
- [ ] Export `WavecraftProvider` from package entrypoint.
- [ ] Place `WavecraftProvider` at template app root.

### Tests to Add / Update

- Add: `ui/packages/core/src/context/WavecraftProvider.test.tsx`
  - Initial load
  - Reconnect refresh
  - Push-update mutation
  - Hot-reload invalidation
  - Unmount safety for in-flight fetches
- Ensure no regressions in existing hook tests.

### Risks / Rollback

- **Risk:** Provider mounted incorrectly can break hook consumers.
- **Rollback:** Keep existing hook logic untouched in this phase; if needed, revert root wrapping and provider exports only.

### Completion Criteria

- [ ] Provider compiles and mounts in template app.
- [ ] Existing public hooks still function unchanged.
- [ ] New provider tests pass.

---

## Phase 2 — Hook Internals Migration (PR 2)

### Objective

Move parameter read lifecycle to provider-backed state while preserving hook API stability.

### Expected Files to Touch

- `ui/packages/core/src/hooks/useAllParameters.ts`
- `ui/packages/core/src/hooks/useParameter.ts`
- `ui/packages/core/src/hooks/useAllParameterFor.ts` *(or the file owning `useAllParametersFor`)*
- `ui/packages/core/src/hooks/*.test.ts*` *(related hook tests)*
- `ui/packages/core/src/index.ts` *(if export surface adjustments are needed)*

### Tasks

- [ ] Refactor `useAllParameters` to consume provider context.
- [ ] Refactor `useParameter` to derive from shared provider state.
- [ ] Keep current signatures and return contracts stable.
- [ ] Ensure duplicate fetches are eliminated via provider centralization.
- [ ] Verify error/loading propagation is shared and consistent.

### Tests to Add / Update

- Update hook tests to include a provider wrapper harness.
- Add assertion that mixed hook usage causes a single logical parameter fetch cycle.
- Add regression test for reconnect + stale state refresh.

### Risks / Rollback

- **Risk:** Subtle behavior drift in loading/error semantics.
- **Rollback:** Retain previous hook implementations on a branch; revert individual hook migrations if regressions appear.

### Completion Criteria

- [ ] Hook APIs unchanged for consumers.
- [ ] Duplicate IPC fetch behavior removed and covered by tests.
- [ ] Hook test suite green.

---

## Phase 3 — Consumer Migration Away from Direct `ParameterClient` (PR 3)

### Objective

Remove direct component writes via `ParameterClient` and route all writes through provider actions/hooks.

### Expected Files to Touch

- `sdk-template/ui/src/processors/SmartProcessor.tsx`
- Potentially related processor/container components under `sdk-template/ui/src/processors/`
- Optional: `ui/eslint.config.js` *(only if introducing a targeted restriction rule)*

### Tasks

- [ ] Replace direct `ParameterClient` write paths with hook/context action calls.
- [ ] Remove manual post-write `reload()` patterns where provider state now handles consistency.
- [ ] Standardize component error handling around provider action promises.
- [ ] Confirm no direct `ParameterClient.getInstance()` usage remains in migrated UI surfaces.

### Tests to Add / Update

- Update processor component tests (or add new tests) to assert:
  - Action-based write call path is used
  - No manual reload dependency for routine writes
- Add regression test for failed write rollback behavior in UI state.

### Risks / Rollback

- **Risk:** Write feedback timing differences (optimistic vs. confirmed update).
- **Rollback:** Temporarily keep fallback action path that can trigger `reload()` if push update semantics fail in edge hosts.

### Completion Criteria

- [ ] `SmartProcessor` no longer uses direct `ParameterClient` writes.
- [ ] Write flow is action/hook/context based.
- [ ] UI behavior remains functionally equivalent or improved.

---

## Phase 4 — Naming Migration + Compatibility Shim (PR 4)

### Objective

Migrate API naming to processor-centric semantics while preserving upgrade safety.

### Expected Files to Touch

- `ui/packages/core/src/hooks/useAllParameterFor.ts` *(or the owning file)*
- `ui/packages/core/src/index.ts`
- `sdk-template/ui/src/**/*.{ts,tsx}` *(consumer call-site migration)*
- Non-archived documentation that references old naming:
  - `docs/feature-specs/wavecraft-provider-parameter-state/*`
  - Other relevant non-archived docs, if applicable

### Tasks

- [ ] Introduce `useParametersForProcessor` as the canonical hook.
- [ ] Keep `useAllParametersFor` as a temporary alias wrapper.
- [ ] Migrate all internal/template call sites to `useParametersForProcessor`.
- [ ] Add deprecation note in docs/changelog context for the old name.
- [ ] Define removal target for the alias (future milestone; not in this PR unless explicitly approved).

### Tests to Add / Update

- Add compatibility tests:
  - Old alias returns identical data/action contract as new hook.
  - New canonical hook behaves identically.
- Update snapshots and types where hook names appear.

### Risks / Rollback

- **Risk:** Downstream SDK consumers rely on the old hook name.
- **Rollback:** Keep alias indefinitely through the next minor cycle; postpone hard removal.

### Completion Criteria

- [ ] Canonical naming available and used in template/app code.
- [ ] Backward compatibility maintained via alias.
- [ ] Tests validate parity between old and new names.

---

## Phase 5 — Hardening, Verification, and Optional Guardrails (PR 5)

### Objective

Lock in behavior, improve migration safety, and ensure CI-grade confidence across the full change.

### Expected Files to Touch

- `ui/packages/core/src/context/WavecraftProvider.test.tsx`
- `ui/packages/core/src/hooks/*.test.ts*`
- `sdk-template/ui/src/**/*.test.ts*`
- Optional: `ui/eslint.config.js` *(restricted direct-client import rules)*

### Tasks

- [ ] Add cross-layer regression tests (provider + hooks + processor usage).
- [ ] Optional: enforce no direct `ParameterClient` imports in template app code via ESLint.
- [ ] Validate documentation references for naming migration accuracy.
- [ ] Final pass for error/loading consistency and reconnect behavior.

### Tests to Add / Update

Regression matrix:

- [ ] Reconnect flow
- [ ] Hot-reload parameter schema change
- [ ] Single-parameter push update
- [ ] Failed write handling
- [ ] Alias hook parity (`useAllParametersFor` == `useParametersForProcessor`)
- [ ] If lint guardrail added: lint test run confirms restriction behavior.

### Risks / Rollback

- **Risk:** Overly strict lint guardrails block valid advanced scenarios.
- **Rollback:** Narrow lint rule scope to `sdk-template/ui/src/` only, or set rule to warning-level initially.

### Completion Criteria

- [ ] All targeted regressions covered in tests.
- [ ] CI/local verification commands pass.
- [ ] Migration is stable and reviewable in small, understandable PR history.

---

## Verification Commands (Repo-Aligned)

Run in this order during each implementation PR:

```bash
# UI tests
cargo xtask test --ui

# Lint and type checking
cargo xtask lint --ui

# Standard CI checks (docs, UI build, lint+typecheck, tests)
cargo xtask ci-check

# Verify UI package version sync
cargo xtask sync-ui-versions --check
```

For broader confidence before merging to `main`:

```bash
cargo xtask ci-check --full
```

Optional manual/visual verification of UI behavior:

```bash
cargo xtask dev
```
