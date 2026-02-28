---

# Implementation Plan: Runtime-Reorderable SignalChain

## Overview

Implement runtime processor reordering across engine, protocol, bridge, and UI using **Option A: generated static dispatch + runtime order table**. Replace compile-time `signal: SignalChain![...]` with `processors: [...]`, keep processor instance identity stable, apply reorder at audio block boundaries with lock-free handoff, persist order per preset, and expose a keyboard-accessible drag-and-drop SignalChain UI.

This plan is sequenced to land engine/runtime primitives first, then protocol/bridge contracts, then UI core and components, and finally sdk-template app integration.

## Prerequisites / Dependencies

- Approved architecture and UX decisions:
  - `docs/feature-specs/ui-signal-chain-reorder/low-level-design-ui-signal-chain-reorder.md`
  - `docs/feature-specs/ui-signal-chain-reorder/ui-design-signal-chain.md`
  - `docs/feature-specs/ui-signal-chain-reorder/user-stories.md`
- Existing code patterns validated:
  - Macro parsing/codegen: `engine/crates/wavecraft-macros/src/plugin/`
  - DSP combinators: `engine/crates/wavecraft-dsp/src/combinators/`
  - IPC protocol contracts: `engine/crates/wavecraft-protocol/src/ipc/{methods.rs,errors.rs}`
  - Bridge dispatch: `engine/crates/wavecraft-bridge/src/{handler.rs,host.rs,error.rs}`
  - UI IPC and hooks: `ui/packages/core/src/ipc/`, `ui/packages/core/src/hooks/`
  - Processor component composition: `ui/packages/components/src/processors/`
  - Template plugin definition: `sdk-template/engine/src/lib.rs`
- New npm deps (components package):
  - `@dnd-kit/core`
  - `@dnd-kit/sortable`
  - `@dnd-kit/utilities`

## Implementation Phases

### Phase 1 — Engine: Macro API Change

**Files to modify/create**

- `engine/crates/wavecraft-macros/src/plugin/parse.rs`
- `engine/crates/wavecraft-macros/src/plugin.rs`
- `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- `engine/crates/wavecraft-macros/src/plugin/runtime_params.rs`
- `engine/crates/wavecraft-macros/src/plugin/metadata.rs`
- `sdk-template/engine/src/lib.rs`

**Changes**

- Replace macro input field:
  - from `signal: SignalChain![...]`
  - to `processors: [A, B, C]`
- Update parser validation/error messages to require `processors`.
- Replace `parse_signal_chain_processors()` with parser for bracketed type list.
- Update codegen input and generated plugin shape so processor registrations are generated from `processors` list.
- Keep processor ID generation stable (same naming path used by `processor_info_entries` / `instance_id_prefixes`) to preserve parameter contracts.
- Update template plugin declaration to `processors: [...]`.

**Ordering constraints**

- Must complete before Phase 2 (runtime order table generation depends on parsed processor list).
- Must complete before Phase 10 (template integration depends on new DSL field).

**Estimated complexity:** High

---

### Phase 2 — Engine: Runtime Order Table + Lock-Free Handoff

**Files to modify/create**

- `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- `engine/crates/wavecraft-dsp/src/combinators/chain.rs` (augment helpers as needed)
- `engine/crates/wavecraft-dsp/src/combinators/mod.rs` (exports/doc updates)
- `engine/crates/wavecraft-dsp/src/lib.rs` (exports if new runtime-order helper is introduced)

**Changes**

- In generated plugin code:
  - Stop relying on nested compile-time `SignalChain` execution for runtime path.
  - Generate stable processor instance storage (one concrete instance per processor).
  - Generate canonical registration order IDs.
  - Generate runtime order table (slot indices) and a lock-free pending-order handoff mechanism (versioned snapshot).
  - Apply pending order swap only at audio block boundaries.
  - Generate static dispatch `match slot_idx { ... }` so no virtual dispatch is introduced.
- Keep parameter and bypass IDs tied to processor identity, not current order position.
- Ensure no allocation/lock in audio-thread reorder apply path.

**Ordering constraints**

- Depends on Phase 1.
- Must complete before Phase 4 (crossfade wraps reorder transition points).
- Must complete before Phase 6 (bridge handlers need runtime order setter/getter endpoints in engine path).

**Estimated complexity:** High

---

### Phase 3 — Engine: Plugin State Persistence (per-preset order)

**Files to modify/create**

- `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- `engine/crates/wavecraft-nih_plug/src/editor/mod.rs` (only if editor wiring needs order-state injection)
- `engine/crates/wavecraft-nih_plug/src/editor/{macos.rs,windows/mod.rs}` (only if bootstrap notification path is added there)
- New (recommended) in macros crate:
  - `engine/crates/wavecraft-macros/src/plugin/state.rs` (JSON schema + validation helpers)

**Changes**

- Add persistence payload schema:
  - `{ "version": 1, "processorOrder": string[] }`
- Store/recover order as non-automatable plugin state chunk data (per preset).
- On restore:
  - validate full permutation (all known IDs, no duplicates, no unknown IDs)
  - apply valid order
  - fallback to registration order on invalid/missing state
- Emit order-changed sync after restore so UI bootstraps correctly.
- Keep state schema versioned for future migrations.

**Ordering constraints**

- Depends on Phase 2 runtime order representation.
- Must complete before final manual DAW roundtrip testing.

**Estimated complexity:** High

---

### Phase 4 — Engine: Crossfade on Reorder

**Files to modify/create**

- `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- Optional helper file if extracted:
  - `engine/crates/wavecraft-dsp/src/combinators/reorder_crossfade.rs`
  - with exports in `engine/crates/wavecraft-dsp/src/combinators/mod.rs` and `lib.rs`

**Changes**

- Add hardcoded crossfade constant:
  - `REORDER_CROSSFADE_SAMPLES: usize = 256`
- Preallocate scratch buffers at init (no audio-thread allocation).
- On order change application:
  - render old order and new order buffers for transition region
  - blend via linear crossfade for 256 samples
- Ensure behavior is deterministic when reorder events arrive rapidly.

**Ordering constraints**

- Depends on Phase 2.
- Should complete before protocol/UI phases to validate engine behavior independently.

**Estimated complexity:** Medium-High

---

### Phase 5 — IPC Protocol: New Message Types

**Files to modify/create**

- `engine/crates/wavecraft-protocol/src/ipc/methods.rs`
- `engine/crates/wavecraft-protocol/src/ipc/errors.rs`
- `engine/crates/wavecraft-protocol/src/ipc.rs`
- `engine/crates/wavecraft-protocol/src/lib.rs`

**Changes**

- Add method constants:
  - `getProcessorOrder`
  - `setProcessorOrder`
- Add notification constant:
  - `processorOrderChanged`
- Add typed contracts:
  - `GetProcessorOrderResult { order: String[] }`
  - `SetProcessorOrderParams { order: String[] }`
  - `SetProcessorOrderResult {}`
  - `ProcessorOrderChangedNotification { order: String[] }`
- Add new error code + constructor for `invalidProcessorOrder`.
- Re-export all new contracts in `ipc.rs` and `lib.rs`.
- Add/extend serialization tests.

**Ordering constraints**

- Must complete before Phase 6 bridge handlers.
- Must complete before Phase 7 UI client typing.

**Estimated complexity:** Medium

---

### Phase 6 — IPC Bridge: New Request Handlers

**Files to modify/create**

- `engine/crates/wavecraft-bridge/src/host.rs`
- `engine/crates/wavecraft-bridge/src/handler.rs`
- `engine/crates/wavecraft-bridge/src/error.rs`
- `engine/crates/wavecraft-bridge/src/in_memory_host.rs`
- `engine/crates/wavecraft-nih_plug/src/editor/bridge.rs`
- Optional: `engine/crates/wavecraft-bridge/src/lib.rs` re-export updates

**Changes**

- Extend `ParameterHost` trait with:
  - `get_processor_order() -> Vec<String>` (or equivalent typed alias)
  - `set_processor_order(order: &[String]) -> Result<(), BridgeError>`
- Add handler dispatch arms for new protocol methods.
- Add permutation validation failure mapping to `invalidProcessorOrder`.
- Add in-memory host implementation for dev/test mode.
- Wire nih-plug editor bridge to engine-side order controller so requests mutate runtime order and reads return current order.
- Add handler tests following existing `handle_*` test style.

**Ordering constraints**

- Depends on Phase 5 protocol contracts.
- Depends on Phase 2 runtime order setter/getter in generated plugin/engine path.
- Must complete before Phase 7–8 UI core work.

**Estimated complexity:** High

---

### Phase 7 — UI Core: ProcessorOrderClient

**Files to modify/create**

- `ui/packages/core/src/ipc/constants.ts`
- `ui/packages/core/src/ipc/ProcessorOrderClient.ts` (new)
- `ui/packages/core/src/types/processorOrder.ts` (new)
- `ui/packages/core/src/index.ts` (exports)
- Optional tests:
  - `ui/packages/core/src/ipc/ProcessorOrderClient.test.ts`

**Changes**

- Add IPC constants:
  - `GET_PROCESSOR_ORDER`
  - `SET_PROCESSOR_ORDER`
  - `PROCESSOR_ORDER_CHANGED`
- Create standalone `ProcessorOrderClient` singleton (do not extend `ParameterClient`).
- Implement methods:
  - `getProcessorOrder()`
  - `setProcessorOrder(order)`
  - `onOrderChanged(cb)`
- Export types and client from package root.

**Ordering constraints**

- Depends on Phase 5 + Phase 6 availability.
- Must complete before Phase 8 hook.

**Estimated complexity:** Medium

---

### Phase 8 — UI Core: useProcessorOrder Hook

**Files to modify/create**

- `ui/packages/core/src/hooks/useProcessorOrder.ts` (new)
- `ui/packages/core/src/index.ts` (exports)
- Optional tests:
  - `ui/packages/core/src/hooks/useProcessorOrder.test.ts`

**Changes**

- Implement hook API from UI spec:
  - input: `fallbackOrder`, `isDraggingRef`
  - output: `order`, `isLoading`, `error`, `reorder(from, to)`
- Bootstrapping behavior:
  - use fallback order immediately
  - fetch authoritative order via `getProcessorOrder`
- Reorder behavior:
  - optimistic local update
  - send full permutation via `setProcessorOrder`
  - rollback/reconcile on error
- Notification behavior:
  - subscribe to `processorOrderChanged`
  - defer incoming notifications while dragging (buffer pending order)
  - flush deferred order on drag end
- Keep hook isolated from parameter state provider internals.

**Ordering constraints**

- Depends on Phase 7 client.
- Must complete before Phase 9 SignalChain component.

**Estimated complexity:** Medium-High

---

### Phase 9 — UI Components: SignalChain Component Folder

**Files to modify/create**

- `ui/packages/components/src/signalChain/SignalChain.tsx` (new)
- `ui/packages/components/src/signalChain/SignalChainItem.tsx` (new)
- `ui/packages/components/src/signalChain/SignalChainSkeleton.tsx` (new)
- `ui/packages/components/src/signalChain/DragHandleIcon.tsx` (new)
- `ui/packages/components/src/signalChain/index.ts` (new)
- `ui/packages/components/src/index.ts` (re-exports)
- `ui/packages/components/package.json` (add `@dnd-kit/core`, `@dnd-kit/sortable`, `@dnd-kit/utilities`)
- Optional tests:
  - `ui/packages/components/src/signalChain/SignalChain.test.tsx`
  - `ui/packages/components/src/signalChain/SignalChainItem.test.tsx`

**Changes**

- Implement vertical sortable list with `@dnd-kit`:
  - pointer + keyboard sensors
  - drag overlay
  - sortable context
- Props: `processors: SignalChainProcessorEntry[]` (each entry: `{ id: ProcessorId, component: React.ReactNode }`)
- `SignalChain` owns `isDraggingRef` and passes to `useProcessorOrder`.
- Add ARIA instructions/announcements and keyboard interaction behavior per UI spec.
- Reuse design tokens/utilities (`focusRingClass`, tokenized classes) per UI spec.
- Render loading/error/empty states per UI spec.
- Barrel `signalChain/index.ts` exports: `SignalChain`, `SignalChainItem`, `SignalChainProcessorEntry`, `SignalChainProps`, `SignalChainItemProps`.

**Ordering constraints**

- Depends on Phase 8 hook.
- Must complete before Phase 10 app integration.

**Estimated complexity:** Medium-High

---

### Phase 10 — Integration: Wire SignalChain into sdk-template App

**Files to modify/create**

- `sdk-template/ui/src/App.tsx`
- `sdk-template/ui/src/components/` (optional component map helper if extracted)
- `sdk-template/engine/src/lib.rs` (final consistency check after Phase 1 migration)

**Changes**

- Replace static processor layout with `<SignalChain processors={[...]} />`.
- Build processor entry array from existing component instances and canonical processor IDs.
- Keep expanded processor cards and bypass toggles intact via existing processor components.
- Ensure `processors` array is defined outside render or wrapped in `useMemo`.
- Validate generated processor IDs align with array entry IDs.

**Ordering constraints**

- Depends on Phases 1 and 9.
- Final integration gate before end-to-end testing.

**Estimated complexity:** Medium

---

## Risk Register

| Risk                                                     | Impact | Likelihood | Mitigation                                                                                        |
| -------------------------------------------------------- | ------ | ---------- | ------------------------------------------------------------------------------------------------- |
| Macro/codegen complexity regression                      | High   | Medium     | Land Phase 1+2 in small compileable increments; add compile-time tests for parser/codegen output. |
| Reorder causes audio artifacts under load                | High   | Medium     | Phase 4 crossfade + stress tests at small buffers (32/64).                                        |
| Order persistence mismatch between UI and engine         | High   | Medium     | Single canonical validation function reused at load, set, and restore points.                     |
| IPC contract drift between Rust and TS                   | Medium | Medium     | Phase 5 first; consume shared names in TS constants and typed clients only.                       |
| Keyboard accessibility regressions in drag flow          | Medium | Medium     | dnd-kit keyboard sensor + ARIA announcements + explicit keyboard tests.                           |
| Race between drag interaction and external notifications | Medium | High       | Deferred notification buffer in `useProcessorOrder` using `isDraggingRef`.                        |
| State-chunk API uncertainty in nih-plug integration      | Medium | Medium     | Resolve in coder spike early in Phase 3.                                                          |

## Testing Notes

- **Rust protocol/bridge**: extend tests in `wavecraft-protocol/src/ipc.rs` and `wavecraft-bridge/src/handler.rs`
- **Engine runtime**: reorder permutation validation, lock-free swap, rapid-reorder stress, crossfade transition assertions
- **UI core**: unit test `ProcessorOrderClient` invoke/subscription behavior; unit test `useProcessorOrder` optimistic reorder + rollback + deferred notifications
- **UI components**: keyboard drag/drop behavior; loading/error/empty-state rendering
- **Template**: validate `sdk-template` builds cleanly after `processors:` migration; run `cargo clippy` on generated code
- **Manual DAW validation** (macOS + Ableton):
  - Reorder during playback (no click/pop)
  - Save project / reload (order restored per preset)
  - Bypass still works pre/post reorder
- **Quality gates**: `cargo xtask ci-check` + UI tests + template validation path

## Resolved Decisions for Coder

The following questions from planning have been answered. Coders should treat these as firm decisions.

1. **nih-plug state persistence hook location** _(resolved by Coder spike during Phase 3)_
   Resolve during Phase 3 implementation by inspecting the nih-plug `Plugin` trait state hooks. No user input required.

2. **Engine↔editor order controller plumbing** _(resolved: option A)_
   Order state lives **inside the generated plugin struct**. The editor bridge receives a reference to it. No separate runtime-order object.

3. **ProcessorId typing in Rust protocol** _(resolved by Coder during Phase 5)_
   Keep `String` for protocol structs (consistent with current pattern). Introduce typed alias internally in the bridge layer only if it reduces noise — wire format stays `String`.

4. **Crossfade boundary behavior for short blocks** _(resolved by Coder during Phase 4)_
   Use carry-over crossfade state across callbacks when block size < 256 samples. Track remaining crossfade samples in the plugin struct; continue blend in the next callback until exhausted.

5. **Fallback behavior on invalid persisted order** _(resolved: emit diagnostic)_
   Fall back to registration order AND emit a `processorOrderRestoreFailed` diagnostic notification so the UI can optionally surface a warning. The notification payload should include the reason (e.g. `"unknownProcessorIds"` or `"invalidPermutation"`).

## References

- [Low-Level Design](./low-level-design-ui-signal-chain-reorder.md)
- [UI Design Specification](./ui-design-signal-chain.md)
- [User Stories](./user-stories.md)
- [Coding Standards — Rust](../../architecture/coding-standards-rust.md)
- [Coding Standards — TypeScript & React](../../architecture/coding-standards-typescript.md)
