# QA Report: Dev Audio FFI Abstraction

**Date**: 2026-02-08
**Reviewer**: QA Agent
**Status**: PASS

## Summary

| Severity | Count | Resolved |
|----------|-------|----------|
| Critical | 1 | 1 |
| Major | 1 | 1 |
| Medium | 1 | 1 |
| Minor | 2 | 1 (Finding 5 deferred — low risk) |
| Info | 5 | — |

**Overall**: PASS — All Critical/High/Medium findings resolved. Ready for Architect handoff.

## Automated Check Results

**Note:** Automated checks (linting, type-checking, tests) were run by the Tester agent via `cargo xtask ci-check` prior to QA review. Results documented in test-plan.md.

- Linting: ✅ PASSED — ESLint, Prettier, cargo fmt, Clippy all clean
- Tests: ✅ PASSED — 150+ engine tests, 28 UI tests

## Findings

| ID | Severity | Category | Description | Location | Recommendation |
|----|----------|----------|-------------|----------|----------------|
| 1 | Critical | Resource Lifecycle | `AudioHandle` dropped immediately — audio stream stops right after starting | `cli/src/commands/start.rs:310-313` | Store handle in a variable that lives until shutdown |
| 2 | Major | Documentation Correctness | Drop ordering comment is factually incorrect about Rust's drop semantics | `engine/crates/wavecraft-bridge/src/plugin_loader.rs:62-64` | Correct the comment or restructure field order |
| 3 | Medium | Robustness | Audio callback hardcodes stereo channel count regardless of device | `engine/crates/wavecraft-dev-server/src/audio_server.rs:100` | Use actual channel count from stream config |
| 4 | Minor | Safety Comments | `unsafe` blocks in vtable loading lack SAFETY comments | `engine/crates/wavecraft-bridge/src/plugin_loader.rs:143-146` | Add `// SAFETY:` annotations per coding standards |
| 5 | Minor | Test Isolation | Static atomics in tests could cause flaky behavior under parallel execution | `engine/crates/wavecraft-dev-server/src/ffi_processor.rs:103-110` | Consider per-test state or `serial_test` |

---

### Finding 1 — Critical: `AudioHandle` Dropped Immediately

**Location:** `cli/src/commands/start.rs` lines 310–313

```rust
let has_audio = runtime.block_on(async {
    let ws_handle = server.handle();
    try_start_audio_in_process(&loader, ws_handle, verbose).is_some()
});
```

**Problem:** `try_start_audio_in_process()` returns `Option<AudioHandle>`, but `.is_some()` immediately converts it to `bool`, dropping the `AudioHandle` at the end of the expression. The `AudioHandle` wraps a cpal `Stream` — dropping it stops audio capture. The audio stream effectively starts and stops within the same expression.

**Impact:** Audio processing stops immediately after starting. The `FfiProcessor` inside the cpal closure is also dropped, calling the vtable's `drop` function. The meter forwarding task (tokio::spawn) receives no more data and exits. In-process audio is non-functional.

**Why tests may have passed:** The test may have observed residual meter values cached in the UI from the brief window before the stream stopped, or a platform-specific cpal behavior on macOS where CoreAudio's cleanup has a race/delay allowing some callback invocations.

**Recommended fix:**

```rust
// Store the handle so the audio stream lives until shutdown
let audio_handle = runtime.block_on(async {
    let ws_handle = server.handle();
    try_start_audio_in_process(&loader, ws_handle, verbose)
});
let has_audio = audio_handle.is_some();
// audio_handle lives until end of run_dev_servers() —
// dropped BEFORE loader (reverse local variable order), which is correct.
```

This also preserves correct drop ordering: `audio_handle` (declared after `loader`) is dropped first (reverse declaration order for locals), so `FfiProcessor::drop()` executes while the Library is still loaded.

---

### Finding 2 — Major: Incorrect Drop Ordering Documentation

**Location:** `engine/crates/wavecraft-bridge/src/plugin_loader.rs` lines 62–64

```rust
/// # Drop Ordering
///
/// `_library` is listed first and is therefore dropped last (Rust drops
/// fields in declaration order).
```

**Problem:** The comment states `_library` is "dropped last" because it's "listed first." This is incorrect. Per the [Rust Reference](https://doc.rust-lang.org/reference/destructors.html): *"The fields of a struct are dropped in declaration order"* — meaning the first-declared field is dropped **first**, not last. (This is the opposite of local variables, which are dropped in reverse declaration order.)

So `_library` being the first field means it is dropped **first**, not last.

**Practical impact:** Currently none — `DevProcessorVTable` is `Copy` (no-op drop), and `Vec<ParameterInfo>` doesn't reference library symbols during deallocation. The real safety constraint (FfiProcessor must be dropped before PluginParamLoader) depends on the **caller** ensuring correct variable ordering, which Finding 1's fix addresses.

**Why this is Major:** A future developer may add a field with a non-trivial Drop that depends on the Library being loaded and rely on this incorrect comment. The comment actively misleads about Rust's safety invariants.

**Recommended fix — Option A (correct the comment):**

```rust
/// # Drop Ordering
///
/// Struct fields are dropped in declaration order (first declared = first
/// dropped). This is fine because all fields are either `Copy` (vtable)
/// or don't reference library symbols during drop (Vec<ParameterInfo>).
///
/// The critical ordering constraint is external: any `FfiProcessor`
/// created from the vtable must be dropped *before* this loader to
/// keep function pointers valid. The caller must ensure this via
/// local variable declaration order (later declared = first dropped).
```

**Recommended fix — Option B (reorder fields to match the stated invariant):**

Move `_library` to the last field position so it truly is dropped last:

```rust
pub struct PluginParamLoader {
    parameters: Vec<ParameterInfo>,
    dev_processor_vtable: Option<DevProcessorVTable>,
    _library: Library,  // Last field → dropped last
}
```

Option B is preferred because it makes the struct self-documenting and resilient to future field additions.

---

### Finding 3 — Medium: Hardcoded Stereo in Audio Callback

**Location:** `engine/crates/wavecraft-dev-server/src/audio_server.rs` line 100

```rust
let num_channels = 2usize;
let num_samples = data.len() / num_channels;
```

**Problem:** The audio callback hardcodes `num_channels = 2` regardless of the actual device configuration from `self.stream_config.channels()`. If the input device provides mono (1 channel) or surround (4+ channels), the deinterleaving math produces incorrect results:

- **Mono device (1 ch):** `num_samples` = `data.len() / 2` = half the actual samples. The second half of the buffer is treated as "right channel" data that doesn't exist.
- **4+ channel device:** Only the first 2 channels worth of interleaved data is processed, remaining data is ignored.

**Impact:** Audio glitches or incorrect metering on non-stereo input devices. Most dev setups use stereo microphones, so this is unlikely to trigger in practice but is a correctness issue.

**Recommended fix:**

```rust
let num_channels = self.stream_config.channels as usize;
// ... or pass channel count into the closure
```

Since `stream_config` is consumed by `build_input_stream`, capture the channel count before the closure:

```rust
let num_channels = self.stream_config.channels() as usize;
// ... inside closure:
// let num_channels = num_channels;  // captured from outer scope
```

---

### Finding 4 — Minor: Missing SAFETY Comments on `unsafe` Blocks

**Location:** `engine/crates/wavecraft-bridge/src/plugin_loader.rs` lines 143–146

```rust
let symbol: Symbol<DevProcessorVTableFn> =
    unsafe { library.get(b"wavecraft_dev_create_processor\0").ok()? };

let vtable = unsafe { symbol() };
```

**Problem:** Two `unsafe` blocks lack `// SAFETY:` annotations. Per coding standards: *"No unsafe Rust without safety comments."*

**Note:** This is consistent with pre-existing code in the same file (the parameter loading `unsafe` blocks at lines 79–106 also lack safety comments). However, newly added `unsafe` code should follow current standards.

**Recommended fix:**

```rust
// SAFETY: `library` is a valid loaded Library. If the symbol doesn't exist,
// `get()` returns Err which `.ok()?` converts to None (graceful fallback).
// The symbol type is trusted to match the macro-generated `extern "C"` function.
let symbol: Symbol<DevProcessorVTableFn> =
    unsafe { library.get(b"wavecraft_dev_create_processor\0").ok()? };

// SAFETY: The symbol points to a valid `extern "C"` function generated by
// `wavecraft_plugin!` macro (same ABI, same return type). The function
// only constructs and returns a DevProcessorVTable by value (no side effects
// beyond initialization). The Library remains loaded for this call.
let vtable = unsafe { symbol() };
```

---

### Finding 5 — Minor: Test Static Atomics May Cause Parallel Test Interference

**Location:** `engine/crates/wavecraft-dev-server/src/ffi_processor.rs` lines 103–110

```rust
static CREATE_CALLED: AtomicBool = AtomicBool::new(false);
static PROCESS_CALLED: AtomicBool = AtomicBool::new(false);
// ... 5 more static atomics
```

**Problem:** All four tests share the same set of static atomics. Rust runs tests in parallel by default. If two tests execute concurrently, they could observe flag changes from each other between `reset_flags()` and the assertion, causing flaky failures.

**Practical impact:** Low with only 4 tests in this module. However, adding more tests increases the risk.

**Recommended fix:** Use `std::sync::Mutex<MockState>` per-test or restructure tests to use local mock state. Alternatively, annotate the test module with `#[serial_test::serial]` to enforce sequential execution.

---

## Positive Observations

| Area | Assessment |
|------|------------|
| **`catch_unwind` coverage** | ✅ All 5 `extern "C"` vtable functions (`create`, `process`, `set_sample_rate`, `reset`, `drop_fn`) are wrapped in `catch_unwind`. No panic can unwind across FFI. |
| **Version checking** | ✅ `try_load_processor_vtable()` checks `vtable.version == DEV_PROCESSOR_VTABLE_VERSION` before accepting the vtable. Mismatches log a `tracing::warn!` with upgrade guidance and return `None`. |
| **Memory ownership** | ✅ All alloc/dealloc happens inside the dylib via vtable functions. `create()` uses `Box::into_raw`, `drop_fn()` uses `Box::from_raw`. CLI never frees processor memory. |
| **Feature gating** | ✅ Audio code properly gated: `ffi_processor` and `audio_server` behind `#[cfg(feature = "audio")]` in dev-server, `try_start_audio_in_process` behind `#[cfg(feature = "audio-dev")]` in CLI, with fallback stubs for disabled features. |
| **Logging** | ✅ Engine crates use `tracing` (info, warn, error). CLI uses `println!` for user-facing output only — appropriate for CLI context. No `println!` in engine crates. |
| **Null pointer guards** | ✅ `FfiProcessor::new()` returns `None` if `create()` returns null. Macro-generated `process()` guards against null `instance` and `channels` pointers. `drop_fn()` checks for null before calling `Box::from_raw`. |
| **Template cleanup** | ✅ `dev-audio.rs` deleted, no `src/bin/` directory, no optional audio deps, no `[features]`, no `[[bin]]` in template `Cargo.toml`. |
| **Backward compatibility** | ✅ Missing vtable symbol → `library.get(...).ok()?` returns `None` → graceful fallback to metering-only mode with informational message. |
| **VTable ABI correctness** | ✅ `DevProcessorVTable` is `#[repr(C)]` with `extern "C"` fn pointers and `u32`. Return by value from `extern "C"` function is well-defined. Function signatures match between macro-generated code and vtable definition. |

## Architectural Concerns

> ⚠️ **The following items require architect review before implementation.**

None — the architecture follows the approved low-level design. The findings are implementation-level issues, not architectural concerns.

## Handoff Decision

**Target Agent**: Architect
**Reasoning**: All Critical/Major/Medium findings resolved, no new issues. Implementation complete and quality verified. Ready for architectural documentation review, then PO for roadmap update and spec archival.

**Priority order:**
1. **Finding 1 (Critical):** Fix `AudioHandle` lifetime — store in named variable
2. **Finding 2 (Major):** Correct drop ordering — reorder fields (Option B preferred) and fix comment
3. **Finding 3 (Medium):** Use actual channel count from stream config
4. **Finding 4 (Minor):** Add SAFETY comments to `unsafe` blocks
5. **Finding 5 (Minor):** Address test isolation (can be deferred)

---

## Re-Review: Final Sign-Off

**Date**: 2026-02-08
**Trigger**: Coder fixed Findings 1–4. Tester verified all fixes pass (see Regression Tests RT-001 through RT-006 in test-plan.md). QA re-reviews for sign-off.

### Finding 1 — Critical: AudioHandle Lifetime — ✅ RESOLVED

**Verified at**: `cli/src/commands/start.rs` lines 309–318

The `AudioHandle` is now stored in a named variable `_audio_handle` (line 314), and `.is_some()` is called separately (line 318) instead of consuming the handle inline. The `_audio_handle` variable lives until the end of `run_dev_servers()` scope, keeping the cpal audio stream alive.

**Drop ordering verified**: `loader` is declared at line 274, `_audio_handle` at line 314. Rust drops local variables in reverse declaration order, so `_audio_handle` (containing the `FfiProcessor`) is dropped first, while the Library in `loader` is still loaded — vtable function pointers remain valid during `FfiProcessor::drop()`.

Comment at lines 309–313 accurately documents this invariant.

### Finding 2 — Major: Drop Ordering — ✅ RESOLVED

**Verified at**: `engine/crates/wavecraft-bridge/src/plugin_loader.rs` lines 58–78

Option B was implemented (preferred approach). `_library` is now the **last** field in `PluginParamLoader`:

```rust
pub struct PluginParamLoader {
    parameters: Vec<ParameterInfo>,           // dropped 1st
    dev_processor_vtable: Option<DevProcessorVTable>, // dropped 2nd (Copy, no-op)
    _library: Library,                        // dropped 3rd (last)
}
```

Doc comments are now correct: line 62 states "Struct fields are dropped in declaration order (first declared = first dropped)" and line 64 states "`_library` is the **last** field so it is dropped last." External invariant (callers must drop `FfiProcessor` before this loader) is documented at lines 70–73. Field-level comment on `_library` reinforces the constraint.

### Finding 3 — Medium: Channel Handling — ✅ RESOLVED

**Verified at**: `engine/crates/wavecraft-dev-server/src/audio_server.rs` lines 88–115

Channel count is now read from the actual stream config: `let num_channels = self.stream_config.channels as usize;` (line 90), captured before `self.stream_config` is consumed by `build_input_stream`.

Robustness guards present:
- `num_channels.max(1)` prevents division by zero (line 100)
- Early return on `num_samples == 0 || num_channels == 0` (lines 102–103)
- Mono handling: duplicates left channel to right when `num_channels == 1` (lines 112–114)

### Finding 4 — Minor: SAFETY Comments — ✅ RESOLVED

**Verified at**: `engine/crates/wavecraft-bridge/src/plugin_loader.rs`

All 6 `unsafe` blocks now have comprehensive `// SAFETY:` annotations per coding standards:

1. `Library::new()` (lines 87–89) — documents inherent unsafety of dynamic loading
2. `library.get("wavecraft_get_params_json")` (lines 92–93) — documents symbol ABI contract
3. `library.get("wavecraft_free_string")` (lines 100–101) — documents symbol ABI contract
4. Combined FFI calls block (lines 107–115) — documents null check, `CStr::from_ptr`, and deallocation matching
5. `library.get("wavecraft_dev_create_processor")` (lines 172–177) — documents graceful `.ok()?` fallback
6. `symbol()` call (lines 181–184) — documents ABI match and no side effects

The Coder also retroactively added SAFETY comments to the pre-existing `unsafe` blocks (items 1–4 above), which was not strictly required but improves overall code quality.

### Finding 5 — Minor: Test Static Atomics — Deferred (Acceptable)

This finding was explicitly deferred in the original handoff ("can be deferred"). The risk is low with only 4 tests in the module, and the tests currently pass reliably. This can be addressed in a future cleanup pass if the test suite grows.

### New Issues Introduced — None

Reviewed all three modified files for regressions:
- No new `unwrap()` or `expect()` calls on fallible operations
- No new `unsafe` blocks without SAFETY comments
- No hardcoded values replacing the channel count fix
- No changes to public API surface
- Drop ordering is consistent between struct fields and local variables
- Feature gates (`#[cfg(feature = "audio-dev")]` / `#[cfg(feature = "audio")]`) remain correct

### Automated Test Confirmation

Per Tester's regression results (test-plan.md RT-001): `cargo xtask ci-check` passes — 162 engine tests, 28 UI tests, all linting clean. No new warnings.

---

## Final Assessment

| Finding | Severity | Status |
|---------|----------|--------|
| 1 — AudioHandle lifetime | Critical | ✅ Resolved |
| 2 — Drop ordering | Major | ✅ Resolved |
| 3 — Channel handling | Medium | ✅ Resolved |
| 4 — SAFETY comments | Minor | ✅ Resolved |
| 5 — Test static atomics | Minor | ⏸️ Deferred (low risk) |

**Status: PASS**

All Critical, Major, and Medium findings are resolved. Code quality meets project standards. No new issues introduced by the fixes.

**Ready for Architect handoff** — No architectural concerns. Implementation follows the approved low-level design. Architect should review for documentation updates and hand off to PO for roadmap update and spec archival.
