# Implementation Progress — Runtime-Reorderable SignalChain

## Status Overview

| Phase | Description                                          | Status      |
| ----- | ---------------------------------------------------- | ----------- |
| 1     | Engine: Macro API change (`signals → processors`)    | ✅ Complete |
| 2     | Engine: Runtime order table + lock-free handoff      | ✅ Complete |
| 3     | Engine: Plugin state persistence                     | ✅ Complete |
| 4     | Engine: Crossfade (volume fade on reorder)           | ✅ Complete |
| 5     | IPC Protocol: New message types                      | ✅ Complete |
| 6     | IPC Bridge: New request handlers                     | ✅ Complete |
| 7     | UI Core: ProcessorOrderClient                        | ✅ Complete |
| 8     | UI Core: `useProcessorOrder` hook                    | ✅ Complete |
| 9     | UI Components: SignalChain component folder          | ✅ Complete |
| 10    | Integration: Wire into `sdk-template/ui/src/App.tsx` | ✅ Complete |

## Phase Details

### Phase 1 — Engine: Macro API change

**Goal**: Replace `signal: SignalChain![…]` DSL key with `processors: [A, B, C]`.

Files:

- `engine/crates/wavecraft-macros/src/plugin/parse.rs` — Replace `signal: Expr` with `processors: Vec<Type>`
- `engine/crates/wavecraft-macros/src/plugin.rs` — Remove `parse_signal_chain_processors` call
- `engine/crates/wavecraft-macros/src/plugin/codegen.rs` — `CodegenInput.signal_type` → `processors`
- `sdk-template/engine/src/lib.rs` — Update syntax

**Acceptance**: `cargo test -p wavecraft-macros` passes.

**Result**: `cargo test -p wavecraft-macros` — 14 tests passed, 0 failed (verified 28 Feb 2026).

---

### Phase 2 — Engine: Runtime order table + lock-free handoff

**Goal**: Add `ProcessorOrderController` (lock-free SPSC pending-order store) and generate per-processor struct fields + runtime dispatch.

Files:

- `engine/crates/wavecraft-bridge/src/order.rs` (NEW)
- `engine/crates/wavecraft-bridge/src/lib.rs`
- `engine/crates/wavecraft-macros/src/plugin/codegen.rs` (major)

**Acceptance**: Plugin compiles; audio processes through all processors in registration order by default.

---

### Phase 3 — Engine: Plugin state persistence

**Goal**: Persist `processorOrder` in plugin state chunk via `serialize_fields`/`deserialize_fields`.

Files:

- `engine/crates/wavecraft-macros/src/plugin/codegen.rs` (add serialize/deserialize)

**Acceptance**: Order round-trips through state save/restore.

> **Gap**: `deserialize_fields` silently falls back to registration order on invalid data. The `processorOrderRestoreFailed` IPC diagnostic (Phase 6) will surface this to the UI after Phase 6 is implemented.

---

### Phase 4 — Engine: Crossfade

**Goal**: Volume fade when order changes (256-sample fade-out → apply order → fade-in).

Files:

- `engine/crates/wavecraft-macros/src/plugin/codegen.rs` (add crossfade state + logic)

**Acceptance**: No click artifacts when reordering in a DAW; `REORDER_CROSSFADE_SAMPLES = 256` constant used.

---

### Phase 5 — IPC Protocol: New message types

**Goal**: Add `getProcessorOrder`/`setProcessorOrder` methods + `processorOrderChanged` notification types.

Files:

- `engine/crates/wavecraft-protocol/src/ipc/methods.rs` — Added:
  - `GetProcessorOrderResult`, `SetProcessorOrderParams`, `ProcessorOrderChangedNotification` structs
  - `METHOD_GET_PROCESSOR_ORDER`, `METHOD_SET_PROCESSOR_ORDER`, `NOTIFICATION_PROCESSOR_ORDER_CHANGED` constants
- `engine/crates/wavecraft-protocol/src/ipc/errors.rs` — Added:
  - `ERROR_INVALID_PROCESSOR_ORDER = -32002`
  - `ERROR_PROCESSOR_ORDER_RESTORE_FAILED = -32003`
- `engine/crates/wavecraft-protocol/src/ipc.rs` — Updated `pub use` exports to re-export all new symbols.

**Result**: `cargo test -p wavecraft-protocol` — 25 tests + 2 doc-tests passed, 0 failed (verified 28 Feb 2026).

---

### Phase 6 — IPC Bridge: New request handlers

**Goal**: Wire `getProcessorOrder`/`setProcessorOrder` through bridge + handler + nih-plug bridge.

Files:

- `engine/crates/wavecraft-protocol/src/ipc/methods.rs` — Added `SetProcessorOrderResult {}` struct
- `engine/crates/wavecraft-protocol/src/ipc.rs` — Re-exported `SetProcessorOrderResult`
- `engine/crates/wavecraft-protocol/src/lib.rs` — All Phase 6 types added to root re-exports
- `engine/crates/wavecraft-bridge/src/error.rs` — Added `InvalidProcessorOrder { reason }` variant; `to_ipc_error()` maps it to `ERROR_INVALID_PROCESSOR_ORDER`
- `engine/crates/wavecraft-bridge/src/host.rs` — Added `get_processor_order`/`set_processor_order` default methods to `ParameterHost`; new `ProcessorOrderAccess` trait; forwarding in `Arc<T>` blanket impl
- `engine/crates/wavecraft-bridge/src/lib.rs` — Re-exported `ProcessorOrderAccess` + new protocol types
- `engine/crates/wavecraft-bridge/src/handler.rs` — Dispatch arms for `getProcessorOrder`/`setProcessorOrder`; `handle_get_processor_order`, `handle_set_processor_order`, `handle_json_multi` methods
- `engine/crates/wavecraft-bridge/src/in_memory_host.rs` — `processor_order: RwLock<Vec<String>>` field; `get_processor_order`/`set_processor_order` impl
- `engine/crates/wavecraft-nih_plug/src/lib.rs` — `BridgeError` + `ProcessorOrderAccess` re-exported in `__nih` module
- `engine/crates/wavecraft-nih_plug/src/editor/bridge.rs` — `P: Params + ProcessorOrderAccess` bound; `get_processor_order`/`set_processor_order` impl; test `ProcessorOrderAccess` impl
- `engine/crates/wavecraft-nih_plug/src/editor/webview.rs` — `WebViewConfig`, `create_webview`, `create_ipc_handler` all updated with `+ ProcessorOrderAccess` bound
- `engine/crates/wavecraft-nih_plug/src/editor/mod.rs` — `WavecraftEditor`, all impl blocks, `create_webview_editor` updated with `+ ProcessorOrderAccess` bound
- `engine/crates/wavecraft-nih_plug/src/editor/macos.rs` — `handle_json_multi` added to `JsonIpcHandler` trait + `IpcHandler<H>` override; multi-message loop in `userContentController_didReceiveScriptMessage`; all `P: Params` bounds updated to `P: Params + ProcessorOrderAccess`
- `engine/crates/wavecraft-macros/src/plugin/codegen.rs` — `ProcessorOrderAccess` impl generated for `__WavecraftParams`; `else { tracing::warn!(...) }` branch in `deserialize_fields`; fixed pre-existing unused-variable warning (`_proc_idx_usize`)

**Result**:

- `cargo test -p wavecraft-bridge` — 29 tests + 2 doc-tests passed, 0 failed
- `cargo test -p wavecraft-nih_plug` — 6 tests passed, 0 failed
- `cargo test -p wavecraft-macros` — 9 unit tests + 5 integration tests passed, 0 failed
- `cargo clippy -p wavecraft-bridge -p wavecraft-nih_plug -p wavecraft-macros -p wavecraft-protocol -- -D warnings` — clean, 0 warnings

---

### Phase 7 — UI Core: ProcessorOrderClient

**Goal**: Singleton `ProcessorOrderClient` for `getProcessorOrder`/`setProcessorOrder` IPC calls.

Files:

- `ui/packages/core/src/ipc/constants.ts` — Added `GET_PROCESSOR_ORDER`, `SET_PROCESSOR_ORDER` to `IpcMethods`; added `PROCESSOR_ORDER_CHANGED` to `IpcEvents`
- `ui/packages/core/src/ipc/ProcessorOrderClient.ts` (NEW) — Singleton pattern with `getProcessorOrder()`, `setProcessorOrder()`, `onProcessorOrderChanged()` methods
- `ui/packages/core/src/index.ts` — Exported `ProcessorOrderClient` + types

**Result**: `npm run typecheck` — no errors. `npm test` — 218 tests passed (verified 28 Feb 2026).

---

### Phase 8 — UI Core: `useProcessorOrder` hook

**Goal**: React hook with optimistic updates + rollback on error.

Files:

- `ui/packages/core/src/hooks/useProcessorOrder.ts` (NEW) — `useProcessorOrder(isDraggingRef?)` hook; optimistic update on `setOrder`, rollback on IPC error; suppresses order-changed notifications during active drag
- `ui/packages/core/src/index.ts` — Exported `useProcessorOrder` + `UseProcessorOrderResult`

**Result**: Part of same typecheck/test run as Phase 7.

---

### Phase 9 — UI Components: SignalChain component folder

**Goal**: Drag-and-drop SignalChain component using `@dnd-kit`.

Files:

- `ui/packages/components/src/signalChain/types.ts` (NEW) — `SignalChainProcessorEntry { id, component }` interface
- `ui/packages/components/src/signalChain/useSignalChainPresentation.ts` (NEW) — `useSortedProcessors(processors, order)` hook; uses `order` as authoritative; loading fallback when `order` is empty
- `ui/packages/components/src/signalChain/SignalChainItem.tsx` (NEW) — `useSortable`-based draggable wrapper with accessible drag handle (keyboard + pointer)
- `ui/packages/components/src/signalChain/SignalChain.tsx` (NEW) — `DndContext` + `SortableContext`, `DragOverlay`, `PointerSensor` + `KeyboardSensor`; null-component slots preserved in order without rendering
- `ui/packages/components/src/signalChain/index.ts` (NEW) — Barrel exports
- `ui/packages/components/package.json` — Added `@dnd-kit/core@6.3.1`, `@dnd-kit/sortable@10.0.0`, `@dnd-kit/utilities@3.2.2`
- `ui/packages/components/src/index.ts` — Exported `SignalChain`, `SignalChainProps`, `SignalChainProcessorEntry`

**Result**: Part of same typecheck/test run as Phase 7.

---

### Phase 10 — Integration

**Goal**: Replace static processor grid in `sdk-template/ui/src/App.tsx` with `<SignalChain>`.

Files:

- `sdk-template/ui/src/App.tsx` — Replaced responsive grid (Row/Col) with `<SignalChain processors={processorEntries} />`; all 8 processors hardcoded in Rust registration slot order; `example_processor` uses `component: null` (placeholder for user's custom DSP)
- `ui/packages/components/src/TemplateApp.test.tsx` — Updated mock: added `useProcessorOrder`, mocked `SignalChain` to avoid `@dnd-kit` dual-React instance in tests, removed stale grid CSS assertions

**Result**: `npm run typecheck` — exit 0. `npm test` — 218/218 tests passed (verified 28 Feb 2026).

---

## Completion Checklist

- [x] Phases 1–10 implementation complete
- [x] `npm run typecheck` passes (exit 0)
- [x] `npm test` — 218/218 tests passed
- [ ] `cargo xtask ci-check` passes (full pipeline)
- [ ] PR created
