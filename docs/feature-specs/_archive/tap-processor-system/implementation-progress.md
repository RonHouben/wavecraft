# Implementation Progress: Tap Processor System

## Status Summary

| Phase                                                                   | Steps       | Status      |
| ----------------------------------------------------------------------- | ----------- | ----------- |
| Phase 1: Rust core — tap trait and payload baselines                    | Steps 1–3   | ✅ Complete |
| Phase 2: IPC contract — replace processor order with signal chain order | Steps 4–11  | ✅ Complete |
| Phase 3: Macro DSL + codegen for taps and unified order resolution      | Steps 12–18 | ✅ Complete |
| Phase 4: UI core API                                                    | Steps 19–24 | ✅ Complete |
| Phase 5: UI SignalChain component and template wiring                   | Steps 25–29 | ✅ Complete |
| Phase 6: Cleanup                                                        | Steps 30–32 | ✅ Complete |

> **Note:** Step 3 (OscilloscopeFrame schema alignment) and Step 11 (WebSocket broadcast comment update) are included as part of Phase 1+2 scope.

---

## Completed Steps (Phase 1 + Phase 2)

### Phase 1: Rust core

- [x] **Step 1** — Add `TapProcessor` trait
  - `engine/crates/wavecraft-dsp/src/traits.rs` — trait added (`Default + Send + 'static`, methods: `set_sample_rate`, `reset`, `observe_stereo`)
  - `engine/crates/wavecraft-dsp/src/lib.rs` — `TapProcessor` added to `pub use`
  - `engine/crates/wavecraft-core/src/prelude.rs` — `TapProcessor` re-exported from `wavecraft_dsp`

- [x] **Step 2** — Move `OscilloscopeTap` to `TapProcessor` contract
  - `engine/crates/wavecraft-processors/src/oscilloscope.rs` — removed `impl Processor`, added `impl TapProcessor` with `observe_stereo` → `capture_stereo` mapping; tests updated to use `tap.observe_stereo(&left, &right)`

- [x] **Step 3** _(partial)_ — No protocol shape changes required; `OscilloscopeFrame` contract unchanged

### Phase 2: IPC contract

- [x] **Step 4** — Define unified slot model in protocol
  - `engine/crates/wavecraft-protocol/src/ipc/methods.rs` — Added `SlotType`, `SignalChainSlot`, `GetSignalChainOrderResult { slots }`, `SetSignalChainOrderParams { slots }`, `SetSignalChainOrderResult`, `SignalChainOrderChangedNotification { slots }`; constants `METHOD_GET_SIGNAL_CHAIN_ORDER`, `METHOD_SET_SIGNAL_CHAIN_ORDER`, `NOTIFICATION_SIGNAL_CHAIN_ORDER_CHANGED`; removed all legacy processor-order types/constants
  - `engine/crates/wavecraft-protocol/src/ipc/errors.rs` — Added `ERROR_INVALID_SIGNAL_CHAIN_ORDER = -32002`, `ERROR_SIGNAL_CHAIN_ORDER_RESTORE_FAILED = -32003`
  - `engine/crates/wavecraft-protocol/src/ipc.rs` — Fixed syntax, updated pub use
  - `engine/crates/wavecraft-protocol/src/lib.rs` — Updated exports to signal-chain types

- [x] **Step 5** — Update `ParameterHost` trait surface
  - `engine/crates/wavecraft-bridge/src/host.rs` — Replaced `ProcessorOrderAccess` with `SignalChainOrderAccess`; methods use `Vec<SignalChainSlot>`; `Arc<T>` blanket impl updated; `ParameterHost` default methods `get_signal_chain_order`/`set_signal_chain_order` added

- [x] **Step 6** — Implement unified order validation in `InMemoryParameterHost`
  - `engine/crates/wavecraft-bridge/src/in_memory_host.rs` — Field renamed `signal_chain_order: RwLock<Vec<SignalChainSlot>>`; validation: no empty IDs, no duplicate IDs; 5 new tests covering valid slots, empty id rejection, duplicate id rejection, mixed slot types

- [x] **Step 7** — Switch IPC handler dispatch + notifications
  - `engine/crates/wavecraft-bridge/src/handler.rs` — Dispatch arms renamed to `METHOD_GET_SIGNAL_CHAIN_ORDER`/`METHOD_SET_SIGNAL_CHAIN_ORDER`; `handle_get/set_signal_chain_order` methods; `handle_json_multi` emits `NOTIFICATION_SIGNAL_CHAIN_ORDER_CHANGED` with `SignalChainOrderChangedNotification { slots }`

- [x] **Step 8** — Update bridge crate public re-exports
  - `engine/crates/wavecraft-bridge/src/lib.rs` — Removed legacy processor-order exports; exposed `SignalChainOrderAccess`, `SignalChainSlot`, `SlotType`, signal-chain IPC types

- [x] **Step 9** — Update dev-server host delegation
  - `dev-server/src/host.rs` — Delegation methods `get_signal_chain_order`/`set_signal_chain_order` added

- [x] **Step 10** — Update plugin editor bridge
  - `engine/crates/wavecraft-nih_plug/src/editor/bridge.rs` — `SignalChainOrderAccess` trait bounds; methods use `Vec<SignalChainSlot>`; test impl updated
  - `engine/crates/wavecraft-nih_plug/src/editor/mod.rs` — All 5 `ProcessorOrderAccess` → `SignalChainOrderAccess`
  - `engine/crates/wavecraft-nih_plug/src/editor/webview.rs` — All 4 occurrences renamed
  - `engine/crates/wavecraft-nih_plug/src/editor/macos.rs` — All 7 occurrences renamed
  - `engine/crates/wavecraft-nih_plug/src/lib.rs` — `TapProcessor` added to DSP re-exports; `SignalChainOrderAccess`, `SignalChainSlot`, `SlotType` added to `__nih` module

- [x] **Step 11** — Update WebSocket broadcast references
  - `dev-server/src/ws/mod.rs` — Comment updated from `processorOrderChanged` → `signalChainOrderChanged`

- [x] **Extra (compile-fix)** — Update macro codegen to implement `SignalChainOrderAccess`
  - `engine/crates/wavecraft-macros/src/plugin/codegen.rs` — `impl SignalChainOrderAccess` replaces `impl ProcessorOrderAccess`; `get_order` returns `Vec<SignalChainSlot>` wrapping stored u8 indices as `SlotType::Processor`; `set_order` accepts `Vec<SignalChainSlot>`, extracts `.id`, validates integer permutation; all `InvalidProcessorOrder` → `InvalidSignalChainOrder`

---

## Completed Steps (Phase 3)

### Phase 3: Macro DSL + codegen

- [x] **Step 12** — Extend macro parser for `taps`
  - `engine/crates/wavecraft-macros/src/plugin/parse.rs` — Optional `taps: [TypePath]` parse branch with duplicate detection and unknown-field diagnostics updated

- [x] **Step 13** — Thread taps through macro expansion pipeline
  - `engine/crates/wavecraft-macros/src/plugin.rs` — `PluginDef` and `CodegenInput` extended with `taps: Vec<Type>`

- [x] **Step 14** — Add compile-time tap/processor separation guards
  - `engine/crates/wavecraft-macros/src/plugin/codegen.rs` — Compile-time checks that tap types implement `TapProcessor` and processor types implement `Processor`; `compile_error!` emitted when a tap appears in `processors`; hard-coded `OscilloscopeTap` string-match detection removed

- [x] **Step 15** — Generate tap fields and per-tap scratch buffers
  - `engine/crates/wavecraft-macros/src/plugin/codegen.rs` — Emits `__tap_0`, `__tap_1`, ... fields; `__tap_0_scratch_l/r`, `__tap_1_scratch_l/r`, ... preallocated buffers; removed `__osc_scratch_l/r` special-case fields

- [x] **Step 16** — Replace pending-order storage with slot-aware resolved state
  - `engine/crates/wavecraft-macros/src/plugin/codegen.rs` — Slot-aware model stores canonical `Vec<SignalChainSlot>`; resolves at control boundary into processor execution order and per-tap insertion boundaries; audio thread reads pre-resolved state lock-free via `__resolved_order` atomic

- [x] **Step 17** — Integrate tap capture in process loop
  - `engine/crates/wavecraft-macros/src/plugin/codegen.rs` — Captures input for taps at boundary 0 before any processing; captures after each processor stage for taps at that boundary; calls `observe_stereo` on scratch slices after all processor stages; no allocations on audio thread

- [x] **Step 18** — Keep processor catalog processor-only
  - `engine/crates/wavecraft-macros/src/plugin/codegen.rs` — `wavecraft_get_processors_json` and generated processor info remain processor-only; taps do not appear in processor discovery surfaces

---

## Completed Steps (Phase 4)

### Phase 4: UI core API

- [x] **Step 19** — Introduce TS signal-chain slot types
  - `ui/packages/core/src/types/signal-chain.ts` — `SlotType` and `SignalChainOrder = { id: ProcessorId | AudioSignalTapId; type: 'processor' | 'tap' }` added and exported

- [x] **Step 20** — Replace IPC constants
  - `ui/packages/core/src/ipc/constants.ts` — `GET_SIGNAL_CHAIN_ORDER`, `SET_SIGNAL_CHAIN_ORDER`, `SIGNAL_CHAIN_ORDER_CHANGED` added; processor-order constants removed

- [x] **Step 21** — Replace `ProcessorOrderClient` with `SignalChainOrderClient`
  - `ui/packages/core/src/ipc/SignalChainOrderClient.ts` — New client: `getSignalChainOrder()`, `setSignalChainOrder(slots)`, subscription to `signalChainOrderChanged`
  - `ui/packages/core/src/ipc/ProcessorOrderClient.ts` — Removed

- [x] **Step 22** — Replace `useProcessorOrder` with `useSignalChainOrder`
  - `ui/packages/core/src/hooks/useSignalChainOrder.ts` — Optimistic update + rollback, drag-guard semantics, returns `SignalChainOrder[]`
  - `ui/packages/core/src/hooks/useProcessorOrder.ts` — Removed

- [x] **Step 23** — Align `useOscilloscopeFrame` contract
  - `ui/packages/core/src/hooks/useOscilloscopeFrame.ts` — Uses canonical method constant; contract fields aligned with Rust `OscilloscopeFrame`

- [x] **Step 24** — Update package barrel + remove legacy exports
  - `ui/packages/core/src/index.ts` — Removed all `ProcessorOrder*` exports; exported `SignalChainOrderClient`, `useSignalChainOrder`, slot types, `OscilloscopeFrame`, `useOscilloscopeFrame`

---

## Completed Steps (Phase 5)

### Phase 5: UI SignalChain component and template wiring

- [x] **Step 25** — Expand component-level signal chain types
  - `ui/packages/components/src/signalChain/types.ts` — Slot-aware entry model with `id`, `type`, render component, stable DnD id

- [x] **Step 26** — Update sorting/presentation logic
  - `ui/packages/components/src/signalChain/useSignalChainPresentation.ts` — Sorted by `SignalChainOrder[]`; unknown slots handled robustly

- [x] **Step 27** — Refactor `SignalChain` DnD mutation path
  - `ui/packages/components/src/signalChain/SignalChain.tsx` — Uses `useSignalChainOrder`; on drop calls `setSignalChainOrder(slots)` with reordered slot objects; drag-guard suppression maintained

- [x] **Step 28** — Template app integration
  - `sdk-template/ui/src/App.tsx` — Unified slot entries (processors + taps) provided to `SignalChain`
  - `ui/packages/components/src/TemplateApp.test.tsx` — Mocks updated for renamed hook/client

- [x] **Step 29** — Template engine DSL migration
  - `sdk-template/engine/src/lib.rs` — `OscilloscopeTap` moved from `processors` list to `taps: [OscilloscopeTap]`

---

## Completed Steps (Phase 6)

### Phase 6: Cleanup

- [x] **Step 30** — Remove all legacy processor-order APIs
  - `engine/crates/wavecraft-bridge/src/handler.rs` — Stale doc comment updated: `setProcessorOrder`/`processorOrderChanged` → `setSignalChainOrder`/`signalChainOrderChanged`
  - `docs/architecture/high-level-design.md` — npm package listing, IPC method listing, and example JSON payloads updated to signal-chain API
  - No live code references to legacy names remain in engine, dev-server, or UI packages

- [x] **Step 31** — Update protocol/bridge re-export surfaces
  - `engine/crates/wavecraft-protocol/src/ipc.rs` — Exports `METHOD_GET_SIGNAL_CHAIN_ORDER`, `METHOD_SET_SIGNAL_CHAIN_ORDER`, `NOTIFICATION_SIGNAL_CHAIN_ORDER_CHANGED`, `SignalChainSlot`, `SlotType`, and related result/param/notification types; no legacy processor-order exports
  - `engine/crates/wavecraft-protocol/src/lib.rs` — Same signal-chain types re-exported at crate root
  - `engine/crates/wavecraft-bridge/src/lib.rs` — Exports `SignalChainOrderAccess`, `SignalChainSlot`, `SlotType`, signal-chain IPC types; no legacy exports

- [x] **Step 32** — Final naming + persisted field key review
  - `engine/crates/wavecraft-macros/src/plugin/codegen.rs` — Persisted state key is `"signalChainOrder"`; stale "(replaces legacy 'processorOrder')" comment annotation removed
  - `dev-server/src/ws/mod.rs` — No legacy stale string literals (confirmed by grep)
  - `engine/crates/wavecraft-bridge/src/handler.rs` — No legacy stale string literals (confirmed by grep)

---

## Final State

- **Tests:** 218 passing (`cargo xtask ci-check`)
- **Legacy references remaining in live code:** 0
- **Legacy references in archived specs (`docs/feature-specs/_archive/`):** preserved intentionally (read-only historical record)
- **Legacy references in roadmap changelog:** preserved intentionally (historical milestone record)
