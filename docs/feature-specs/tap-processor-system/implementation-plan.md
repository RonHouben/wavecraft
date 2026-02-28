# Implementation Plan: Tap Processor System

## Overview

This implementation introduces first-class tap processors in the plugin DSL and runtime, replaces processor-only ordering with a unified `SignalChainOrder` slot contract, and aligns Rust + UI contracts around a clean-break API (`getSignalChainOrder`, `setSignalChainOrder`, `signalChainOrderChanged`). It also formalizes oscilloscope frame handling as a separate data path parallel to metering and updates codegen so tap placement is derived from unified runtime order rather than hard-coded `OscilloscopeTap` detection.

## Requirements

- Add DSL support for `taps: [TypePath]` in `wavecraft_plugin!`.
- Introduce `TapProcessor` trait and make `OscilloscopeTap` implement it (not `Processor`).
- Replace processor-order IPC surface with unified signal-chain slot surface.
- Add/align Rust + TS `SignalChainSlot` typing and method/event constants.
- Ensure codegen emits `__tap_N` + per-tap scratch buffers and derives insertion points from runtime slot order.
- Keep processor catalog / generated processors list processor-only.
- Update UI API and component flow to use one N+T slot list and one mutation path.
- Remove all legacy processor-order paths (clean break).

## Architecture Changes

- **DSL parse + macro expansion**
  - `engine/crates/wavecraft-macros/src/plugin/parse.rs`
  - `engine/crates/wavecraft-macros/src/plugin.rs`
  - `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- **Core traits**
  - `engine/crates/wavecraft-dsp/src/traits.rs`
  - `engine/crates/wavecraft-core/src/prelude.rs`
- **Tap implementation**
  - `engine/crates/wavecraft-processors/src/oscilloscope.rs`
- **IPC protocol**
  - `engine/crates/wavecraft-protocol/src/ipc/methods.rs`
  - `engine/crates/wavecraft-protocol/src/ipc.rs`
  - `engine/crates/wavecraft-protocol/src/lib.rs`
- **Bridge + host contracts**
  - `engine/crates/wavecraft-bridge/src/host.rs`
  - `engine/crates/wavecraft-bridge/src/in_memory_host.rs`
  - `engine/crates/wavecraft-bridge/src/handler.rs`
  - `engine/crates/wavecraft-bridge/src/lib.rs`
- **Dev server + editor integration**
  - `dev-server/src/host.rs`
  - `dev-server/src/ws/mod.rs`
  - `engine/crates/wavecraft-nih_plug/src/editor/bridge.rs`
  - `engine/crates/wavecraft-nih_plug/src/lib.rs`
- **UI core API**
  - `ui/packages/core/src/ipc/constants.ts`
  - `ui/packages/core/src/ipc/ProcessorOrderClient.ts` (remove)
  - `ui/packages/core/src/ipc/SignalChainOrderClient.ts` (new)
  - `ui/packages/core/src/hooks/useProcessorOrder.ts` (remove)
  - `ui/packages/core/src/hooks/useSignalChainOrder.ts` (new)
  - `ui/packages/core/src/hooks/useOscilloscopeFrame.ts`
  - `ui/packages/core/src/types/signal-chain.ts` (new)
  - `ui/packages/core/src/index.ts`
- **UI components / template**
  - `ui/packages/components/src/signalChain/types.ts`
  - `ui/packages/components/src/signalChain/useSignalChainPresentation.ts`
  - `ui/packages/components/src/signalChain/SignalChain.tsx`
  - `ui/packages/components/src/TemplateApp.test.tsx`
  - `sdk-template/ui/src/App.tsx`
  - `sdk-template/engine/src/lib.rs`

---

## Implementation Steps

### Phase 1: Rust core — tap trait and payload baselines

#### Step 1 — Add `TapProcessor` trait

- **Files**: `engine/crates/wavecraft-dsp/src/traits.rs`, `engine/crates/wavecraft-core/src/prelude.rs`
- **Change**: Add `TapProcessor: Default + Send + 'static` with methods `set_sample_rate(&mut self, f32)`, `reset(&mut self)`, `observe_stereo(&mut self, &[f32], &[f32])`. Re-export in prelude.
- **Dependency**: None
- **Risk**: Low

#### Step 2 — Move `OscilloscopeTap` to `TapProcessor` contract

- **File**: `engine/crates/wavecraft-processors/src/oscilloscope.rs`
- **Change**: Remove `impl Processor for OscilloscopeTap`. Add `impl TapProcessor for OscilloscopeTap` mapping `set_sample_rate` → `set_sample_rate_hz`, `observe_stereo` → `capture_stereo`.
- **Dependency**: Step 1
- **Risk**: Medium (temporary compile break until Phase 3)

#### Step 3 — Confirm/align `OscilloscopeFrame` protocol shape

- **Files**: `engine/crates/wavecraft-protocol/src/ipc/methods.rs`, `ui/packages/core/src/types/oscilloscope.ts`
- **Change**: Ensure Rust/TS schema parity for `OscilloscopeFrame`. Rename fields if LLD shape differs from current shape.
- **Dependency**: None
- **Risk**: Medium (UI rendering/tests depend on shape)

---

### Phase 2: IPC contract — replace processor order with signal chain order

#### Step 4 — Define unified slot model in protocol

- **Files**: `engine/crates/wavecraft-protocol/src/ipc/methods.rs`, `engine/crates/wavecraft-protocol/src/ipc.rs`, `engine/crates/wavecraft-protocol/src/lib.rs`
- **Change**:
  - Add `SlotType` enum (`Processor` | `Tap`), `SignalChainSlot { id: String, slot_type: SlotType }`, result/param/notification types for signal-chain order.
  - Add constants `METHOD_GET_SIGNAL_CHAIN_ORDER`, `METHOD_SET_SIGNAL_CHAIN_ORDER`, `NOTIFICATION_SIGNAL_CHAIN_ORDER_CHANGED`.
  - Remove legacy processor-order structs/constants/exports.
- **Dependency**: None
- **Risk**: Medium (many downstream compile refs)

#### Step 5 — Update `ParameterHost` trait surface

- **File**: `engine/crates/wavecraft-bridge/src/host.rs`
- **Change**: Replace `get_processor_order`/`set_processor_order` with `get_signal_chain_order`/`set_signal_chain_order` using `Vec<SignalChainSlot>`. Replace `ProcessorOrderAccess` with `SignalChainOrderAccess`.
- **Dependency**: Step 4
- **Risk**: Medium (trait bounds propagate widely)

#### Step 6 — Implement unified order validation in `InMemoryParameterHost`

- **File**: `engine/crates/wavecraft-bridge/src/in_memory_host.rs`
- **Change**: Replace `processor_order: RwLock<Vec<String>>` with `signal_chain_order: RwLock<Vec<SignalChainSlot>>`. Implement strict validation: exact permutation, type/id consistency, no duplicates, no omissions.
- **Dependency**: Step 5
- **Risk**: Medium

#### Step 7 — Switch IPC handler dispatch + notifications

- **File**: `engine/crates/wavecraft-bridge/src/handler.rs`
- **Change**: Replace `getProcessorOrder`/`setProcessorOrder` match arms with `getSignalChainOrder`/`setSignalChainOrder`. Emit `signalChainOrderChanged` on successful set.
- **Dependency**: Steps 4, 5
- **Risk**: Medium

#### Step 8 — Update bridge crate public re-exports

- **File**: `engine/crates/wavecraft-bridge/src/lib.rs`
- **Change**: Remove legacy processor-order exports, expose signal-chain equivalents.
- **Dependency**: Steps 4–7
- **Risk**: Low

#### Step 9 — Update dev-server host delegation

- **File**: `dev-server/src/host.rs`
- **Change**: Replace passthrough methods to new signal-chain methods.
- **Dependency**: Steps 5, 6
- **Risk**: Low

#### Step 10 — Update plugin editor bridge

- **Files**: `engine/crates/wavecraft-nih_plug/src/editor/bridge.rs`, `engine/crates/wavecraft-nih_plug/src/lib.rs`
- **Change**: Rename `ProcessorOrderAccess` → `SignalChainOrderAccess` trait bounds. Bridge methods map to new getter/setter signatures.
- **Dependency**: Step 5
- **Risk**: Medium

#### Step 11 — Update WebSocket broadcast references

- **File**: `dev-server/src/ws/mod.rs`
- **Change**: Update any `processorOrderChanged` references to `signalChainOrderChanged`.
- **Dependency**: Step 7
- **Risk**: Low

---

### Phase 3: Macro DSL + codegen for taps and unified order resolution

#### Step 12 — Extend macro parser for `taps`

- **File**: `engine/crates/wavecraft-macros/src/plugin/parse.rs`
- **Change**: Add optional `taps: [Type]` parse branch (default empty). Duplicate tap type detection. Update unknown-field diagnostics.
- **Dependency**: None
- **Risk**: Low

#### Step 13 — Thread taps through macro expansion pipeline

- **File**: `engine/crates/wavecraft-macros/src/plugin.rs`
- **Change**: Extend `PluginDef` and `CodegenInput` to carry `taps: Vec<Type>`.
- **Dependency**: Step 12
- **Risk**: Low

#### Step 14 — Add compile-time tap/processor separation guards

- **File**: `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- **Change**: Generate compile-time checks requiring tap types satisfy `TapProcessor` and processor types satisfy `Processor`. Emit `compile_error!` when a tap appears in `processors`. Remove hard-coded `OscilloscopeTap` string-match detection.
- **Dependency**: Steps 1, 13
- **Risk**: Medium

#### Step 15 — Generate tap fields and per-tap scratch buffers

- **File**: `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- **Change**: Emit `__tap_0`, `__tap_1`, ... fields. Emit `__tap_0_scratch_l/r`, `__tap_1_scratch_l/r`, ... preallocated buffers. Remove `__osc_scratch_l/r` interim special-case fields.
- **Dependency**: Step 14
- **Risk**: Medium

#### Step 16 — Replace pending-order storage with slot-aware resolved state

- **File**: `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- **Change**: Replace `u8` permutation-only storage with a slot-aware model that stores the canonical `Vec<SignalChainSlot>` and resolves at control boundary into: (a) processor execution order, (b) per-tap insertion boundaries. Audio thread reads pre-resolved state lock-free.
- **Dependency**: Steps 4, 5, 15
- **Risk**: High (core runtime behavior)

#### Step 17 — Integrate tap capture in process loop

- **File**: `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- **Change**: At block start, capture input for taps at boundary 0. After each processor stage, capture for taps at that boundary. After processor stages, call each tap's `observe_stereo` on scratch slices.
- **Dependency**: Step 16
- **Risk**: High (RT correctness, multi-tap ordering)

#### Step 18 — Keep processor catalog processor-only

- **File**: `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- **Change**: Ensure `wavecraft_get_processors_json` and generated processor info remain processor-only. Do not leak taps into processor discovery surfaces.
- **Dependency**: Steps 13–17
- **Risk**: Low

---

### Phase 4: UI core API

#### Step 19 — Introduce TS signal-chain slot types

- **Files**: `ui/packages/core/src/types/signal-chain.ts` (new), `ui/packages/core/src/index.ts`
- **Change**: Add `SlotType` and `SignalChainOrder = { id: ProcessorId | AudioSignalTapId; type: 'processor' | 'tap' }` and export publicly.
- **Dependency**: Step 4
- **Risk**: Low

#### Step 20 — Replace IPC constants

- **File**: `ui/packages/core/src/ipc/constants.ts`
- **Change**: Add `GET_SIGNAL_CHAIN_ORDER`, `SET_SIGNAL_CHAIN_ORDER`, `SIGNAL_CHAIN_ORDER_CHANGED`. Remove processor-order constants.
- **Dependency**: Step 4
- **Risk**: Low

#### Step 21 — Replace `ProcessorOrderClient` with `SignalChainOrderClient`

- **Files**: `ui/packages/core/src/ipc/SignalChainOrderClient.ts` (new), `ui/packages/core/src/ipc/ProcessorOrderClient.ts` (remove)
- **Change**: Implement `getSignalChainOrder(): Promise<SignalChainOrder[]>`, `setSignalChainOrder(slots: SignalChainOrder[]): Promise<void>`, and subscription to `signalChainOrderChanged`.
- **Dependency**: Steps 19, 20
- **Risk**: Medium (public API break)

#### Step 22 — Replace `useProcessorOrder` with `useSignalChainOrder`

- **Files**: `ui/packages/core/src/hooks/useSignalChainOrder.ts` (new), `ui/packages/core/src/hooks/useProcessorOrder.ts` (remove)
- **Change**: Preserve optimistic update + rollback behavior and drag-guard semantics. Return `SignalChainOrder[]` instead of index strings.
- **Dependency**: Step 21
- **Risk**: Medium

#### Step 23 — Align `useOscilloscopeFrame` contract

- **Files**: `ui/packages/core/src/hooks/useOscilloscopeFrame.ts`, `ui/packages/core/src/types/oscilloscope.ts`
- **Change**: Ensure hook uses canonical method constant and expected contract fields after Step 3 schema alignment.
- **Dependency**: Step 3
- **Risk**: Low

#### Step 24 — Update package barrel + remove legacy exports

- **File**: `ui/packages/core/src/index.ts`
- **Change**: Remove all `ProcessorOrder*` exports. Export `SignalChainOrderClient`, `useSignalChainOrder`, slot types.
- **Dependency**: Steps 21, 22
- **Risk**: Low

---

### Phase 5: UI SignalChain component and template wiring

#### Step 25 — Expand component-level signal chain types

- **File**: `ui/packages/components/src/signalChain/types.ts`
- **Change**: Replace processor-only entry model with slot-aware entry model carrying `id`, `type`, render component, and stable DnD id strategy.
- **Dependency**: Steps 19, 22
- **Risk**: Medium

#### Step 26 — Update sorting/presentation logic

- **File**: `ui/packages/components/src/signalChain/useSignalChainPresentation.ts`
- **Change**: Sort by `SignalChainOrder[]` order. Handle unknown slots robustly.
- **Dependency**: Step 25
- **Risk**: Low

#### Step 27 — Refactor `SignalChain` DnD mutation path

- **File**: `ui/packages/components/src/signalChain/SignalChain.tsx`
- **Change**: Replace `useProcessorOrder` with `useSignalChainOrder`. On drop, reorder slot objects and call `setSignalChainOrder(slots)`. Maintain drag-guard suppression.
- **Dependency**: Steps 22, 26
- **Risk**: Medium

#### Step 28 — Template app integration

- **Files**: `sdk-template/ui/src/App.tsx`, `ui/packages/components/src/TemplateApp.test.tsx`
- **Change**: Provide unified slot entries (processors + taps) to `SignalChain`. Update test mocks for renamed hook/client.
- **Dependency**: Step 27
- **Risk**: Medium

#### Step 29 — Template engine DSL migration

- **File**: `sdk-template/engine/src/lib.rs`
- **Change**: Move `OscilloscopeTap` from `processors` list to `taps: [OscilloscopeTap]`.
- **Dependency**: Steps 12+
- **Risk**: Low

---

### Phase 6: Cleanup

#### Step 30 — Remove all legacy processor-order APIs

- **Files**: all legacy references in `engine/crates/wavecraft-bridge/**`, `engine/crates/wavecraft-protocol/**`, `engine/crates/wavecraft-nih_plug/**`, `ui/packages/core/**`, `ui/packages/components/**`, `dev-server/**`
- **Change**: Delete old constants, methods, DTOs, events, traits, and tests. No compatibility alias.
- **Dependency**: Phases 2–5
- **Risk**: Medium (grep-driven pass required)

#### Step 31 — Update protocol/bridge re-export surfaces

- **Files**: `engine/crates/wavecraft-protocol/src/ipc.rs`, `engine/crates/wavecraft-protocol/src/lib.rs`, `engine/crates/wavecraft-bridge/src/lib.rs`
- **Change**: Remove old exports, expose only new signal-chain contract.
- **Dependency**: Step 30
- **Risk**: Low

#### Step 32 — Final naming + persisted field key review

- **File**: `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- **Change**: Replace persisted state key `"processorOrder"` with `"signalChainOrder"`. Remove stale log strings referencing old names.
- **Dependency**: Steps 16, 30
- **Risk**: Medium (intentional state restore break)

---

## Testing Strategy

### Rust unit/integration

- `engine/crates/wavecraft-macros`: parser tests for `taps`, duplicate taps, and invalid tap-in-processors compile errors; codegen behavior tests for generated tap fields and order-resolution wiring.
- `engine/crates/wavecraft-bridge`: `InMemoryParameterHost` validation tests for slot permutation, type mismatch, omissions, duplicates; `IpcHandler` tests for new method names and changed notification payload.
- `engine/crates/wavecraft-nih_plug`: bridge trait-bound compile tests and order passthrough tests.
- `engine/crates/wavecraft-processors`: `OscilloscopeTap` tests adapted to `TapProcessor` API.
- `dev-server`: host delegation tests to new methods.

### TypeScript / Vitest

- `ui/packages/core`: `SignalChainOrderClient` unit tests; `useSignalChainOrder` optimistic update/rollback tests; `useOscilloscopeFrame` polling tests after contract alignment.
- `ui/packages/components`: `SignalChain` reorder tests using slot objects; `TemplateApp.test.tsx` mock updates.

### End-to-end verification sequence

1. Build Rust crates after Phase 2 (contract compiles end-to-end).
2. Build macro consumers after Phase 3 (`sdk-template/engine` compiles with `taps` field).
3. Build UI packages after Phases 4–5.
4. Run `cargo xtask ci-check`.
5. Manual runtime check: reorder processor/tap slots from UI; verify `signalChainOrderChanged` notifications; verify oscilloscope capture position changes with slot moves.

---

## Risks & Mitigations

| Risk                                                              | Mitigation                                                                                                        |
| ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| Macro refactor introduces RT regressions                          | Keep capture resolution on control path; no allocations in `process`; add targeted macro-generated behavior tests |
| API break leaves stale call sites                                 | Grep-driven cleanup pass for all legacy names before final validation                                             |
| Slot validation source-of-truth ambiguity in host implementations | Define declared slot catalog ownership explicitly in Phase 2 before coding                                        |
| Oscilloscope payload mismatch between Rust/TS                     | Lock schema in protocol (Step 3) first, then update TS types/hooks in same phase                                  |

---

## Success Criteria

- [ ] `wavecraft_plugin!` accepts `taps` and rejects tap types in `processors`.
- [ ] `OscilloscopeTap` no longer implements `Processor`; it implements `TapProcessor`.
- [ ] Legacy processor-order methods/events/types are fully removed.
- [ ] New signal-chain methods/events are the only runtime ordering API.
- [ ] Codegen emits `__tap_N` + scratch buffers and derives insertion boundaries from unified slots.
- [ ] UI `SignalChain` uses one N+T slot model and one set mutation path.
- [ ] `sdk-template/engine/src/lib.rs` uses `taps: [OscilloscopeTap]`.
- [ ] Full checks pass (`cargo xtask ci-check`) with updated tests.

---

## Related Documents

- [Low-Level Design](./low-level-design-tap-processor-system.md) — architecture and contract specifications
- [High-Level Design](../../architecture/high-level-design.md) — system overview
- [Coding Standards](../../architecture/coding-standards.md) — conventions
- [Declarative Plugin DSL](../../architecture/declarative-plugin-dsl.md) — macro system context
