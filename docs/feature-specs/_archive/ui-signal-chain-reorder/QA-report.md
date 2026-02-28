# QA Report — Runtime-Reorderable SignalChain

## Status: APPROVED

## Summary

All High findings have been resolved and Finding 3 has been assessed as Not Applicable by design. The feature is ready for merge.

## Findings

### Finding 1: Real-time unsafe allocations in generated `process()` hot path

- **Severity**: High
- **Status**: ✅ Resolved
- **File**: `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- **Description**: The generated audio-thread loop allocates on the heap in the hot path:
  - Block-level `Vec` allocation for params (all parameter values collected into `Vec<f32>`)
  - Per-sample `Vec<Vec<f32>>` and `Vec<&mut [f32]>` allocations for channel slices
  - Oscilloscope snapshot `to_vec()`/`clone()` calls in `process()`
  These violate real-time constraints from the architecture coding standards (no allocations on the audio thread).
- **Fix**:
  - Added `__param_scratch: Vec<f32>` field to `__WavecraftPlugin`, pre-grown at construction to total param count. Per-block: `clear()` + `push()` loop — zero allocation after init.
  - Replaced per-sample `Vec<Vec<f32>>` + `Vec<&mut [f32]>` with stack-allocated `[f32; 2]` + `[&mut [f32]; 2]` arrays. Uses `split_at_mut` + inner block for borrow discipline.
  - Oscilloscope `to_vec()`/`clone()` retained with doc comment: `// Block-end snapshot: acceptable once-per-block allocation.`

### Finding 2: `InMemoryParameterHost::set_processor_order()` does not validate permutation

- **Severity**: High
- **Status**: ✅ Resolved
- **File**: `engine/crates/wavecraft-bridge/src/in_memory_host.rs`
- **Description**: Validation only checked for a non-empty input, then wrote blindly without enforcing valid slot indices, range, or duplicate prevention.
- **Fix**: Applied full permutation validation matching `ProcessorOrderAccess::set_order()`: parses each slot as `usize`, then checks index in-range and no duplicates via a `seen` bitvec. Returns `BridgeError::InvalidProcessorOrder` with a descriptive reason on any failure. Five new unit tests added to cover: empty order, valid permutation, out-of-range index, duplicate index, non-integer slot name.

### Finding 3: `processorOrderChanged` — notification after `getProcessorOrder`

- **Severity**: Medium
- **Status**: ✅ Not Applicable (by design)
- **File**: `engine/crates/wavecraft-bridge/src/handler.rs`
- **Assessment**: The LLD (`low-level-design-ui-signal-chain-reorder.md`) specifies that `processorOrderChanged` is emitted "after successful `setProcessorOrder`", on state restore, and on initial sync — never after GET. A notification after GET would be redundant (the GET response already carries current state) and could cause client subscription loops. The design intent is explicit: GET is for robust UI bootstrap, not to trigger side-effects.
- **Fix**: No code change required. A doc comment was added to `handle_get_processor_order` documenting this design decision and referencing the LLD.

### Finding 4: Unsafe blocks in FFI section lack local SAFETY rationale

- **Severity**: Medium
- **Status**: ✅ Resolved
- **File**: `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- **Description**: Multiple FFI unsafe usages in the dev-FFI vtable section (`wavecraft_dev_create_processor`) lacked local safety justification.
- **Fix**: Added focused SAFETY comments to all unsafe blocks in the vtable: instance casts (`Box::into_raw` / `&mut *` provenance), channel pointer reads (`*channels.add(N)` bounds), `from_raw_parts_mut` / `from_raw_parts` (non-null, caller-owned, in-bounds), and `Box::from_raw` (called exactly once, `into_raw` provenance).

## Approved Items

- `BridgeError::InvalidProcessorOrder` maps correctly to `ERROR_INVALID_PROCESSOR_ORDER`.
- `ProcessorOrderClient` / `useProcessorOrder` type flow is consistent (`string[]` end-to-end).
- Hook subscription cleanup is implemented (`return unsubscribe`) in `useProcessorOrder`.
- Optimistic update rollback logic exists and restores prior state on IPC failure.
- `SignalChain` enables keyboard accessibility sensor (`KeyboardSensor`).
- `Map<ProcessorId, ReactNode>` lookup table is memoized in `SignalChain.tsx`.
- Empty processors are handled gracefully via presentation fallback.
- Crossfade state-machine entry behavior appears coherent (no immediate first-block silence regression from default state).
- Lock-free SPSC ordering uses `Ordering::Release` for writes and `Ordering::Acquire` for reads — correct.
