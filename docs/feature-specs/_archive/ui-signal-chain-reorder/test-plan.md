# Test Plan — Runtime-Reorderable SignalChain

## Scope

Validation of phases 1–10 of the `ui-signal-chain-reorder` feature on branch `feature/ui-signal-chain-reorder`.

Date: 28 February 2026  
Workspace: `/Users/ronhouben/code/private/wavecraft`

## Automated Test Results

### cargo xtask ci-check

Status: ✅ PASS  
Time: ~69s  
UI tests: 218/218 passed  
Engine tests: all passed  
Lint + typecheck: passed

### Phase-level crate test results

| Crate              | Tests           | Status |
| ------------------ | --------------- | ------ |
| wavecraft-macros   | 14              | ✅     |
| wavecraft-protocol | 27              | ✅     |
| wavecraft-bridge   | 29+2 (doctests) | ✅     |
| wavecraft-nih_plug | 6               | ✅     |
| UI (vitest)        | 218             | ✅     |

## Visual Verification

Opened `http://localhost:5173` in VS Code Simple Browser during `cargo xtask dev`.

Observed:

- App loads and renders without startup errors.
- Processor UI sections are visible: Test Tone, Input Trim, Passthrough, Tone Filter, Saturator, Output Gain, Oscilloscope.
- Signal-chain content appears in single ordered flow (consistent with vertical stack design spec).
- Drag-related icon marker is present in rendered output.
- Interactive drag-to-reorder recommended as manual follow-up (headless session limitation).

### Screenshot evidence

- Screenshot capture via macOS `screencapture` failed in headless environment.
- Page snapshot artifact: `artifacts/ui-visual-validation/ui-signal-chain-reorder-page-source.html`

## Test Cases

### TC-1: Macro API compilation

**Phase**: 1  
**Expected**: `processors: [...]` syntax compiles; 14 macro tests pass.  
**Result**: ✅ PASS

### TC-2: Runtime order table + lock-free handoff

**Phase**: 2  
**Expected**: Plugin compiles with per-processor struct fields and runtime dispatch.  
**Result**: ✅ PASS (build succeeds)

### TC-3: Plugin state persistence

**Phase**: 3  
**Expected**: `serialize_fields`/`deserialize_fields` generate correctly in params struct.  
**Result**: ✅ PASS (build + nih_plug tests pass)

### TC-4: Crossfade on reorder

**Phase**: 4  
**Expected**: `__CROSSFADE_SAMPLES = 256` constant, `__cf_pos`/`__cf_dir` state generated; no blocking in audio thread.  
**Result**: ✅ PASS (build succeeds; DAW artifact testing is manual / out-of-scope for CI)

### TC-5: IPC protocol message types

**Phase**: 5  
**Expected**: `getProcessorOrder`/`setProcessorOrder` methods and notification types compile and test.  
**Result**: ✅ PASS (27 protocol tests)

### TC-6: IPC bridge handlers

**Phase**: 6  
**Expected**: `ProcessorOrderAccess` trait impl; bridge dispatch arms work.  
**Result**: ✅ PASS (bridge + nih_plug tests)

### TC-7: ProcessorOrderClient

**Phase**: 7  
**Expected**: TypeScript singleton compiles with correct IPC method names.  
**Result**: ✅ PASS (typecheck + vitest)

### TC-8: useProcessorOrder hook

**Phase**: 8  
**Expected**: Hook with optimistic update/rollback; `isDraggingRef` deferral during active drag.  
**Result**: ✅ PASS (vitest)

### TC-9: SignalChain component

**Phase**: 9  
**Expected**: `@dnd-kit`-based vertical list renders with drag handles; keyboard accessible.  
**Result**: ✅ PASS automated; visual render verified in running app (interactive drag is a manual follow-up)

### TC-10: App.tsx integration

**Phase**: 10  
**Expected**: SignalChain renders in place of static processor grid in running app.  
**Result**: ✅ PASS (ordered processor presentation visible in running app)

## Known Gaps / Out of Scope

- **`processorOrderRestoreFailed` IPC notification**: `deserialize_fields` currently logs via `eprintln!` when invalid state is found; IPC notification to the UI is a follow-up task (tracked in `implementation-progress.md`).
- **DAW crossfade artifact validation**: Audio click artifact testing requires manual DAW testing; documented as manual follow-up.
- **End-to-end drag-to-reorder with engine**: Full roundtrip (drag UI → IPC → engine order change → audio processed in new order) requires manual DAW testing.

## Issues Found

No blocking issues. One observation:

1. **Visual evidence capture limitation (headless environment)**
   - Severity: Low
   - Description: OS screenshot capture is unavailable in automated sessions (`screencapture` fails). Page-source artifact was captured instead.
   - Impact: Does not affect automated or functional validation.
