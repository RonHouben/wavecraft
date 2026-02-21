# Low-Level Design: WavecraftProvider — Centralised Parameter State

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview, IPC/bridge contracts, real-time safety constraints
- [Coding Standards](../../architecture/coding-standards.md) — TypeScript/React conventions, naming, imports
- [SDK Architecture](../../architecture/sdk-architecture.md) — `@wavecraft/core` npm package structure, subpath exports
- [Development Workflows](../../architecture/development-workflows.md) — Browser dev mode, hot-reload, two-stage UI build
- [Plugin Formats](../../architecture/plugin-formats.md) — VST3/CLAP/AU host context relevant to parameter ownership
- [Roadmap](../../roadmap.md) — Milestone tracking

---

## 1. Problem Statement

The current `@wavecraft/core` SDK exposes parameter state through a set of independent hooks and singleton classes. Each hook manages its own fetch lifecycle, connection watching, and subscription to parameter-change notifications. This produces several compounding problems in practice:

1. **Multiple concurrent fetches.** `useParameter` and `useAllParameters` both call `ParameterClient.getAllParameters()` independently. A component tree that mixes both hooks triggers parallel network/IPC requests for the same data on every mount and reconnect.

2. **Inconsistent invalidation.** `useAllParameters` auto-invalidates on `parameterChanged` notifications and on hot-reload events. `useParameter` does not; it refetches the entire parameter list on `connected` state changes but does not react to per-parameter push updates without explicitly re-subscribing per hook instance.

3. **Manual reload after writes.** `SmartProcessor` (`sdk-template/ui/src/processors/SmartProcessor.tsx`) calls `ParameterClient.getInstance().setParameter(...)` directly and then calls `reload()` explicitly after every successful write. This pattern is brittle, duplicates error-handling boilerplate, and is incorrect when optimistic UI is desired.

4. **Scattered singleton access.** Components access `ParameterClient.getInstance()` directly to perform writes. This bypasses any future middleware layer (e.g., undo/redo, offline queueing, analytics) and makes testing harder.

5. **No shared loading/error surface.** There is no single boundary that can provide a "parameters unavailable" banner; every consumer must render its own loading and error states independently.

---

## 2. Goals

- Introduce a `WavecraftProvider` React context at the application root that owns the single global parameter fetch lifecycle.
- Ensure all parameter reads in the component tree derive from one cached dataset, eliminating duplicate IPC calls.
- Provide a context-aware `setParameter` that updates local state optimistically (or via confirmed server push) without requiring manual `reload()` calls in component code.
- Keep the public hook API (`useParameter`, `useAllParameters`, `useAllParametersFor`) source-compatible; migrate their implementations to read from context rather than from independent fetch state.
- Centralise hot-reload invalidation and `parameterChanged` notification handling once, in the provider.
- Establish a clear public/internal API boundary so application code never imports `ParameterClient` or `IpcBridge` directly.

## 3. Non-Goals

- Changing the JSON-RPC wire protocol or adding new IPC methods.
- Introducing a general-purpose state-management library (Redux, Zustand, Jotai, etc.).
- Implementing full undo/redo for parameter changes.
- Supporting multi-instance (multi-plugin) state within one provider; `WavecraftProvider` is scoped to a single plugin window.
- Modifying the Rust engine or `wavecraft-bridge`.

---

## 4. Current-State Analysis

### 4.1 Hook inventory

| Hook | File | Fetch strategy | Invalidation |
|---|---|---|---|
| `useAllParameters` | `ui/packages/core/src/hooks/useAllParameters.ts` | Fetches `getAllParameters()` on mount + reconnect; retries up to 3× with exponential backoff. Deduplicates concurrent fetches via `fetchingRef`. | Subscribes to `parameterChanged` (in-place update) and `IpcEvents.PARAMETERS_CHANGED` (full refetch via `reload`). |
| `useAllParametersFor` | `ui/packages/core/src/hooks/useAllParameterFor.ts` | Thin wrapper over `useAllParameters`; filters by `processorId` via `useMemo`. | Inherits parent's invalidation. |
| `useParameter` | `ui/packages/core/src/hooks/useParameter.ts` | Calls `getAllParameters()` independently and filters for `id`. | Refetches on `connected` transition and `parameterChanged` notification; does NOT share state with `useAllParameters`. |

### 4.2 ParameterClient and IpcBridge access

`ParameterClient` and `IpcBridge` are singletons exported from `ui/packages/core/src/index.ts` as part of the "advanced use" tier. Application code in `sdk-template/ui/src/processors/SmartProcessor.tsx` accesses `ParameterClient.getInstance()` directly:

```tsx
// sdk-template/ui/src/processors/SmartProcessor.tsx (current)
const client = ParameterClient.getInstance();

onChange: async (value) => {
  await client.setParameter(param.id, value);
  await reload(); // <-- manual cache bust after write
}
```

This pattern leaks IPC mechanics into view components and is the primary source of boilerplate and inconsistency.

### 4.3 Loading/error handling

Each hook maintains independent `isLoading` and `error` state. A component tree that uses `useAllParametersFor` in four `SmartProcessor` instances renders four independent loading skeletons and four independent error states, even though they all depend on the same underlying fetch.

### 4.4 Hot-reload path

`useAllParameters` subscribes to `IpcEvents.PARAMETERS_CHANGED` inside a `useEffect`. Every mounted instance of `useAllParameters` fires a `reload()` on the same hot-reload event. Deduplication in `fetchingRef` prevents parallel requests from a single hook instance, but multiple independent hook instances (e.g., `useParameter` + `useAllParameters` in the same tree) still each enqueue a fetch.

---

## 5. Target Architecture

### 5.1 Overview

```
┌───────────────────────────────────────────────────────────┐
│                     WavecraftProvider                     │
│                                                           │
│  ParameterStateContext                                    │
│  ┌──────────────────────────────────────────────────┐    │
│  │  params: ParameterInfo[]                         │    │
│  │  isLoading: boolean                              │    │
│  │  error: Error | null                             │    │
│  │  setParameter(id, value): Promise<void>          │    │
│  │  reload(): Promise<void>                         │    │
│  └──────────────────────────────────────────────────┘    │
│                                                           │
│  Owns:                                                    │
│  • Single getAllParameters() fetch + retry                │
│  • onParameterChanged subscription (in-place update)     │
│  • PARAMETERS_CHANGED hot-reload subscription            │
│  • Connection-aware refetch                               │
└───────────────────────────────────────────────────────────┘
                            │ context
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
       useParameter  useAllParameters  useAllParametersFor
       (reads from   (reads from       (reads from
        context)      context)          context + filters)
              │
              ▼
       SmartProcessor / other components
       (read via hook, write via setParameter from context)
```

### 5.2 WavecraftProvider contract

```tsx
// Conceptual public interface — not prescriptive of implementation file

interface ParameterStateContext {
  /** Flat list of all parameters, normalised. Always in sync with engine. */
  params: readonly ParameterInfo[];
  /** True while the initial fetch or a reload is in-flight. */
  isLoading: boolean;
  /** Last fetch or transport error, or null. */
  error: Error | null;
  /** Set a parameter value. Applies optimistic update; confirms via push. */
  setParameter: (id: ParameterId, value: ParameterValue) => Promise<void>;
  /** Force a full refetch (e.g., after structural change). */
  reload: () => Promise<void>;
}

interface WavecraftProviderProps {
  children: React.ReactNode;
}

export function WavecraftProvider({ children }: WavecraftProviderProps): JSX.Element;
```

`WavecraftProvider` is a zero-config component. It does not accept an `IpcBridge` prop; it resolves the singleton internally. This mirrors the existing hook pattern and avoids introducing dependency-injection plumbing before it is needed (see §9 Open Decisions).

### 5.3 setParameter semantics

The context `setParameter` implementation follows an **optimistic-then-confirmed** strategy:

1. Apply the new value to the local `params` array immediately (no loading state change).
2. Call `ParameterClient.setParameter(id, value)` over IPC.
3. On success: the engine pushes a `parameterChanged` notification; the provider's `onParameterChanged` handler applies the confirmed server value in-place. If the value is unchanged (common case), the state update is a no-op.
4. On error: roll back the optimistic update and surface the error via a returned `Promise` rejection. The component is responsible for presenting feedback.

This eliminates the `reload()` call in `SmartProcessor` and prevents a full refetch on every write.

### 5.4 Hook taxonomy after migration

| Export | Status | Implementation source |
|---|---|---|
| `useParameter(id)` | **Public** | Reads `params` from `ParameterStateContext`, filters for `id` |
| `useAllParameters()` | **Public** | Returns `{ params, isLoading, error, reload }` directly from context |
| `useAllParametersFor(processorId)` | **Public** | Reads from context, filters via `selectProcessorParams` (unchanged logic) |
| `useParameterState` (internal) | **Internal** | Low-level context accessor; not exported from `index.ts` |
| `ParameterClient` | **Advanced** | Still exported for escape-hatch use; not recommended in application code |
| `IpcBridge` | **Advanced** | Unchanged |

The public hook signatures remain identical. No breaking changes to the `@wavecraft/core` package public API.

### 5.5 Context shape and file location

```
ui/packages/core/src/
├── context/
│   ├── WavecraftProvider.tsx       # Provider component + context creation
│   ├── ParameterStateContext.ts    # Context type definition and createContext call
│   └── useParameterState.ts        # Internal context accessor hook
```

The `WavecraftProvider` is exported from `ui/packages/core/src/index.ts` under the existing "React Hooks (primary API)" section. Its type is also exported. `useParameterState` is intentionally **not** exported from `index.ts`.

---

## 6. Fetch and Invalidation Semantics

### 6.1 Initial fetch

On mount, `WavecraftProvider` sets `isLoading = true` and awaits the first successful `getAllParameters()`. The existing retry logic (3 attempts, 500ms base backoff) from `useAllParameters` is lifted verbatim into the provider. The 15-second connection timeout behaviour is preserved.

### 6.2 Connection-change invalidation

| Transition | Behaviour |
|---|---|
| `disconnected → connected` | Clear error; set `isLoading = true`; fetch. |
| `connected → disconnected` | Keep `params` stale (better than empty); set `isLoading = true` (signals pending refetch). |

### 6.3 Per-parameter push update

`ParameterClient.onParameterChanged` fires when the engine pushes a `parameterChanged` notification. The provider applies an in-place immutable update:

```ts
setParams(prev => prev.map(p => p.id === changedId ? { ...p, value } : p));
```

This is a direct copy of the existing `updateParameterValue` helper in `useAllParameters.ts`. Consumers re-render only if the value changed (React bailout applies at slice level).

### 6.4 Structural invalidation (hot-reload)

`IpcEvents.PARAMETERS_CHANGED` signals that the parameter schema has changed (e.g., after a Rust hot-reload during development). The provider responds with a full `reload()`. This triggers `isLoading = true` on the context, which propagates to all consumers simultaneously — one loading gate instead of N.

### 6.5 Write-side invalidation

After `setParameter` succeeds:
- The optimistic update is already applied (§5.3).
- No explicit reload is needed.
- The engine confirms via `parameterChanged` notification, which is handled by the push handler (§6.3).

If the IPC call fails:
- Roll back the optimistic state.
- Do not reload (write failed; the engine state is unchanged).
- Reject the promise so the calling component can render feedback.

---

## 7. Public vs Internal API Boundaries

### 7.1 Exported from `@wavecraft/core` (`index.ts`)

```ts
// Provider
export { WavecraftProvider } from './context/WavecraftProvider';
export type { WavecraftProviderProps } from './context/WavecraftProvider';

// Existing hooks (re-exported unchanged — implementations updated internally)
export { useParameter } from './hooks/useParameter';
export { useAllParameters } from './hooks/useAllParameters';
export { useAllParametersFor } from './hooks/useAllParameterFor';
```

### 7.2 Not exported (internal)

```ts
// Internal — context accessor
// useParameterState is used only inside core hooks
import { useParameterState } from '../context/useParameterState';
```

### 7.3 Deprecation path for direct ParameterClient usage

`ParameterClient` remains exported in the "advanced use" tier. Application code (`SmartProcessor`, user plugin UIs) should migrate from:

```tsx
const client = ParameterClient.getInstance();
await client.setParameter(id, value);
await reload();
```

to:

```tsx
const { setParameter } = useAllParametersFor(processorId);
// or at component level:
const { setParameter } = useAllParameters(); // if useAllParameters exposes setParameter
```

`ParameterClient` is not removed; no deprecation warning is added yet. This is a style boundary enforced by convention and documented here.

---

## 8. Phased Migration

### Phase 1 — Introduce `WavecraftProvider` and context (non-breaking)

**Scope:** `ui/packages/core/src/context/`

- Create `ParameterStateContext.ts`, `WavecraftProvider.tsx`, `useParameterState.ts`.
- `WavecraftProvider` implements the full fetch/retry/invalidation/push-update lifecycle (logic ported from `useAllParameters.ts`).
- Export `WavecraftProvider` from `index.ts`.
- No existing hooks are changed.
- Add `<WavecraftProvider>` to `sdk-template/ui/src/App.tsx` (or equivalent root).
- **Verification:** Existing behaviour is unchanged. `WavecraftProvider` fetches once; independent hooks continue fetching independently (temporarily duplicated; resolved in Phase 2).

### Phase 2 — Migrate hooks to consume context

**Scope:** `ui/packages/core/src/hooks/`

- `useAllParameters`: replace internal fetch state with `useParameterState()`; keep the same return type and `reload` reference.
- `useAllParametersFor`: already delegates to `useAllParameters`; no change needed beyond Phase 2a.
- `useParameter`: replace internal `getAllParameters()` call with a context read + `useMemo` filter; remove independent `onParameterChanged` subscription.
- **Verification:** Only one `getAllParameters()` call in network trace on load and on reconnect. All hooks reflect the same `isLoading` / `error` state simultaneously.

### Phase 3 — Remove direct ParameterClient writes from application code

**Scope:** `sdk-template/ui/src/processors/SmartProcessor.tsx`

- Replace `ParameterClient.getInstance().setParameter(...)` + `reload()` with a `setParameter` obtained from context (via `useAllParametersFor` or a dedicated `useSetParameter` helper if desired).
- **Verification:** No `ParameterClient.getInstance()` calls in `sdk-template/ui/src/` (enforced by eslint rule if desired; see §10 Open Decisions).

### Phase 4 — (Optional) Add shared loading/error boundary

- Introduce an `<ParametersErrorBoundary>` or equivalent wrapper that reads context `isLoading` and `error` and renders a single fallback UI, removing per-component loading skeletons in `SmartProcessor`.
- This phase is optional and can be deferred.

---

## 9. Testing and Verification

### 9.1 Unit tests — `WavecraftProvider`

File: `ui/packages/core/src/context/WavecraftProvider.test.tsx`

| Scenario | Assertion |
|---|---|
| Initial mount with connected transport | `getAllParameters` called once; context `isLoading` transitions to `false` |
| Mount while transport disconnected | `isLoading` remains `true`; fetch triggered on connect event |
| `parameterChanged` notification received | Corresponding param in context updated in-place; no refetch |
| `PARAMETERS_CHANGED` event received | Full reload triggered; `isLoading` briefly `true` |
| `setParameter` success with push confirmation | Optimistic value applied; confirmed value replaces it |
| `setParameter` failure | Rolled back to pre-call value; promise rejects |
| Unmount during in-flight fetch | No `setState` after unmount (no React warning) |
| 15-second connection timeout | Error surfaced; `isLoading` set to `false` |

### 9.2 Unit tests — updated hooks

| Hook | Test change |
|---|---|
| `useAllParameters` | Existing tests pass without modification (same return shape) |
| `useParameter` | Verify single `getAllParameters` call when both `useParameter` and `useAllParameters` are mounted in same tree (requires wrapping tree in `WavecraftProvider` in test) |

### 9.3 Integration test — duplicate fetch prevention

A Vitest test mounts a component tree containing:
- `<WavecraftProvider>`
  - `<SmartProcessor id="processorA" />`
  - `<SmartProcessor id="processorB" />`
  - A component using `useParameter("processorA_gain")`

Assert that `ParameterClient.getAllParameters` is called **exactly once** on initial mount.

### 9.4 CI verification

`cargo xtask ci-check` covers lint, type-check, and Vitest. No additional CI steps are required for this feature.

---

## 10. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Context consumer re-renders on every param update | Medium | Medium | `useAllParametersFor` already uses `useMemo` to slice the param list; consumers only re-render when their slice changes. `useParameter` filters to a single object; reference equality prevents unnecessary re-renders if the value is unchanged. |
| Optimistic rollback causes visual flicker | Low | Low | Optimistic writes are fast (IPC roundtrip <10ms in practice); confirmed push arrives quickly and is identity-equal, producing no DOM change. |
| Provider missing from tree causes hook crash | Medium | High | `useParameterState` throws a descriptive error if context is `undefined`. Development-time only; SDK template always includes `<WavecraftProvider>`. |
| Simultaneous migration across Phase 1→2 leaves a window of duplicate fetches | Medium | Low | Phase 1 is explicitly non-breaking; duplicate fetches are temporary and bounded. Phase 2 removes them. |
| `reload` reference identity changes cause downstream `useEffect` re-runs | Low | Low | `reload` is created with `useCallback` inside the provider and has stable identity across renders (dependency: `fetchParameters`, itself stable). |
| Hot-reload during `setParameter` in-flight | Low | Medium | `PARAMETERS_CHANGED` triggers a full reload which replaces all `params`, including any pending optimistic state. The in-flight `setParameter` IPC call completes independently; its push confirmation hits the fresh `params` array. No data corruption; worst case is a visible flicker. Document as known behaviour. |

---

## 11. Open Decisions

1. **Should `useAllParameters` expose `setParameter`?**
   Returning `setParameter` from `useAllParameters` (and therefore from `useAllParametersFor`) removes any need for components to access context directly or import `ParameterClient`. This would make Phase 3 trivial. Downside: changes the return type of `useAllParameters`, which is a minor breaking change if downstream plugin code destructs the result. Decision needed before Phase 2.

2. **ESLint rule to ban `ParameterClient.getInstance()` in `sdk-template/ui/`?**
   An ESLint `no-restricted-imports` rule on `ParameterClient` in `sdk-template/ui/src/` would enforce the architectural boundary automatically. Adds friction for advanced use cases; may be too aggressive before the migration is complete. Defer to after Phase 3.

3. **Provider injection via props vs. singleton resolution?**
   Currently `WavecraftProvider` resolves `ParameterClient.getInstance()` internally. Accepting an optional `client` prop would enable cleaner unit testing without mocking the singleton. Adds API surface. Decision: defer; use `vi.mock` in tests for now.

4. **`WavecraftProvider` location in `sdk-template/ui/`?**
   It should wrap the entire plugin UI (above routing, if any, and above all processor components). The natural mount point is `sdk-template/ui/src/App.tsx`. If the template gains a router in the future, the provider should remain above it. Confirm placement during Phase 1 implementation.

5. **Multi-window / multi-instance support?**
   If a future plugin architecture exposes multiple independent plugin windows (e.g., a detached editor), each window needs its own `WavecraftProvider` instance. The singleton `ParameterClient` would need to be scopeable first. Out of scope for this feature; noted here for architectural awareness.

---

## Implementation Notes

- The fetch/retry logic in `WavecraftProvider` is moved verbatim from `useAllParameters.ts` (`attemptFetch`, `handleSuccessResult`, `handleStopResult`, constants). Once the migration is complete, `useAllParameters.ts` becomes a thin wrapper and the duplicated logic can be deleted.
- The `updateParameterValue` helper (`useAllParameters.ts` line ≈82) is promoted to a shared utility in `context/` or `utils/` during the migration.
- `useAllParameterFor.ts` (note: singular) exports `useAllParametersFor` (plural). The file name / export name inconsistency pre-dates this feature; it should be noted but is not in scope to fix here.

