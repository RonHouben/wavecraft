# Implementation Plan: Dev Audio FFI Abstraction

## Overview

Remove `dev-audio.rs` from the user project template by moving audio processing into the CLI process via a C-ABI FFI vtable. The `wavecraft_plugin!` macro auto-generates the FFI export; the CLI loads the vtable from the user's cdylib (alongside existing parameter discovery), creates an `FfiProcessor` wrapper, and runs audio capture + processing in-process. The template is cleaned of all audio-dev dependencies, feature flags, and binary targets.

## Requirements

- Users must never see or maintain `dev-audio.rs` or audio-dev dependencies
- `wavecraft start` audio processing must work without any user action beyond `wavecraft_plugin!` macro usage
- Backward compatibility: older plugins without the vtable symbol continue to work (metering-only mode)
- All memory allocated inside the dylib must be freed inside the dylib (no cross-allocator issues)
- Panics in DSP code must not crash the CLI (`catch_unwind` at FFI boundaries)

## Architecture Changes

- **`wavecraft-protocol`**: New `dev_audio_ffi` module with `DevProcessorVTable` and constants
- **`wavecraft-macros`**: Generate `wavecraft_dev_create_processor` FFI export in `wavecraft_plugin!` macro
- **`wavecraft-bridge`**: Extend `PluginParamLoader` to optionally load the vtable symbol
- **`wavecraft-dev-server`**: New `DevAudioProcessor` trait, `FfiProcessor` wrapper, refactored `AudioServer`
- **CLI `start.rs`**: In-process audio via FFI, remove `try_start_audio_server()` / `has_audio_binary()`
- **CLI `Cargo.toml`**: Add `cpal` optional dependency with `audio-dev` default feature
- **Template**: Remove `dev-audio.rs`, `src/bin/` dir, optional deps, `[features]`, `[[bin]]` from `Cargo.toml.template`

## Implementation Steps

### Phase 1: FFI Contract (`wavecraft-protocol`)

#### Step 1.1 — Create `dev_audio_ffi` module
**File:** `engine/crates/wavecraft-protocol/src/dev_audio_ffi.rs`
- Action: Create new file with `DevProcessorVTable` (`#[repr(C)]`) struct, `DEV_PROCESSOR_VTABLE_VERSION` constant, and `DEV_PROCESSOR_SYMBOL` constant
- Fields: `version: u32`, `create`, `process`, `set_sample_rate`, `reset`, `drop` — all `extern "C" fn` pointers using `*mut c_void` and primitive types
- Include comprehensive doc comments per the design document
- Why: Defines the shared ABI contract between the macro-generated code (user dylib) and the CLI consumer
- Dependencies: None — this is the foundation
- Risk: Low

#### Step 1.2 — Register module in `wavecraft-protocol` lib.rs
**File:** `engine/crates/wavecraft-protocol/src/lib.rs`
- Action: Add `pub mod dev_audio_ffi;` and re-export `DevProcessorVTable`, `DEV_PROCESSOR_VTABLE_VERSION`, `DEV_PROCESSOR_SYMBOL`
- Why: Makes the contract accessible to both `wavecraft-macros` (via `wavecraft-nih_plug`'s `__internal` module) and `wavecraft-bridge` (CLI-side loader)
- Dependencies: Step 1.1
- Risk: Low

### Phase 2: Macro Code Generation (`wavecraft-macros`)

#### Step 2.1 — Add vtable FFI export to `wavecraft_plugin!` macro
**File:** `engine/crates/wavecraft-macros/src/plugin.rs`
- Action: In `wavecraft_plugin_impl()`, add a new `#[unsafe(no_mangle)] pub extern "C" fn wavecraft_dev_create_processor() -> ...` block after the existing `wavecraft_get_params_json` / `wavecraft_free_string` exports
- The generated code creates inner `extern "C"` functions (`create`, `process`, `set_sample_rate`, `reset`, `drop_fn`) that wrap the concrete `__ProcessorType`
- Each inner function must be wrapped in `std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { ... }))` to prevent panics from unwinding across the FFI boundary
- The `process` function builds `Vec<&mut [f32]>` from raw `*mut *mut f32` pointers and calls `Processor::process()` with a default `Transport` and default `Params`
- Reference the vtable types via `#krate::__internal::DevProcessorVTable` (same pattern as existing `ParameterInfo` references)
- Why: Auto-generates the FFI export so users don't write any audio binary code
- Dependencies: Step 1.2
- Risk: Medium — proc-macro code generation with unsafe FFI; needs thorough testing

#### Step 2.2 — Expose vtable types through `wavecraft-nih_plug` `__internal` module
**File:** `engine/crates/wavecraft-nih_plug/src/lib.rs`
- Action: Re-export `DevProcessorVTable`, `DEV_PROCESSOR_VTABLE_VERSION` from `wavecraft_protocol` via the existing `__internal` module (used by generated macro code)
- Why: The macro-generated code references types via `#krate::__internal::*`; this is the existing pattern for `ParameterInfo`, `serde_json`, etc.
- Dependencies: Step 1.2
- Risk: Low

### Phase 3: CLI-Side Plugin Loader (`wavecraft-bridge`)

#### Step 3.1 — Extend `PluginParamLoader` with vtable loading
**File:** `engine/crates/wavecraft-bridge/src/plugin_loader.rs`
- Action:
  - Add `use wavecraft_protocol::{DevProcessorVTable, DEV_PROCESSOR_VTABLE_VERSION};` import
  - Add `dev_processor_vtable: Option<DevProcessorVTable>` field to `PluginParamLoader`
  - Add `try_load_processor_vtable(library: &Library) -> Option<DevProcessorVTable>` private method that:
    1. Tries `library.get(b"wavecraft_dev_create_processor\0")`
    2. If not found → returns `None` (graceful backward compat)
    3. Calls the function, checks `vtable.version == DEV_PROCESSOR_VTABLE_VERSION`
    4. If version mismatch → logs warning, returns `None`
    5. Otherwise returns `Some(vtable)`
  - Call `try_load_processor_vtable` in `load()` and store the result
  - Add `pub fn dev_processor_vtable(&self) -> Option<&DevProcessorVTable>` accessor
- Why: Extends existing FFI loader to also discover the audio processor without a separate binary
- Dependencies: Step 1.2
- Risk: Low — additive change to existing loader, graceful fallback on missing symbol

#### Step 3.2 — Update `wavecraft-bridge` exports
**File:** `engine/crates/wavecraft-bridge/src/lib.rs`
- Action: Ensure `DevProcessorVTable` is re-exported if needed by CLI consumers (or CLI imports it directly from `wavecraft-protocol`)
- Why: CLI needs access to the vtable type for `FfiProcessor` construction
- Dependencies: Step 3.1
- Risk: Low

### Phase 4: Audio Server Refactoring (`wavecraft-dev-server`)

#### Step 4.1 — Add `DevAudioProcessor` trait and `FfiProcessor` wrapper
**File:** `engine/crates/wavecraft-dev-server/src/ffi_processor.rs` (new file)
- Action: Create file with:
  - `DevAudioProcessor` trait: `process(&mut self, channels: &mut [&mut [f32]])`, `set_sample_rate(&mut self, f32)`, `reset(&mut self)`
  - `FfiProcessor` struct: holds `*mut c_void` instance and `DevProcessorVTable` copy
  - `unsafe impl Send for FfiProcessor {}` with justification comment
  - `impl DevAudioProcessor for FfiProcessor` dispatching through vtable function pointers
  - `impl Drop for FfiProcessor` calling `(self.vtable.drop)(self.instance)`
  - Unit tests with mock vtable function pointers: lifecycle (create/drop), process dispatching, null-safety
- Why: Adapts the FFI vtable to a safe Rust trait the audio server can use
- Dependencies: Step 1.1
- Risk: Medium — unsafe code, but well-bounded and testable

#### Step 4.2 — Refactor `AudioServer` to use `DevAudioProcessor`
**File:** `engine/crates/wavecraft-dev-server/src/audio_server.rs`
- Action:
  - Replace the generic `AudioServer<P: Processor>` with `AudioServer<P: DevAudioProcessor>` (or a non-generic version taking `Box<dyn DevAudioProcessor>`)
  - Remove the dependency on `wavecraft_dsp::Processor` from audio_server
  - Update the `build_input_stream` callback to call `DevAudioProcessor::process()` instead of `Processor::process()`
  - The callback still computes meters and sends updates via the existing WebSocket client
  - Adjust the `AudioConfig` struct: remove `websocket_url` (audio runs in-process, the CLI owns the WS server)
  - Instead, the audio server communicates meters back via a channel (`tokio::sync::mpsc`) or a meter callback
- Why: Decouples the audio server from compile-time knowledge of the processor type
- Dependencies: Step 4.1
- Risk: Medium — significant refactor of existing code; preserve meter pipeline

#### Step 4.3 — Register new module in `wavecraft-dev-server` lib.rs
**File:** `engine/crates/wavecraft-dev-server/src/lib.rs`
- Action: Add `pub mod ffi_processor;` (possibly gated behind `#[cfg(feature = "audio")]`)
- Dependencies: Step 4.1
- Risk: Low

#### Step 4.4 — Update `wavecraft-dev-server/Cargo.toml`
**File:** `engine/crates/wavecraft-dev-server/Cargo.toml`
- Action: Add `wavecraft-protocol` to the audio feature deps if not already there (needed for `DevProcessorVTable`)
- Why: `FfiProcessor` needs the vtable type
- Dependencies: Step 4.1
- Risk: Low

### Phase 5: CLI Integration (`wavecraft start`)

#### Step 5.1 — Add `cpal` dependency to CLI
**File:** `cli/Cargo.toml`
- Action:
  - Add `cpal = { version = "0.15", optional = true }`
  - Add `[features]` section: `default = ["audio-dev"]`, `audio-dev = ["cpal", "wavecraft-dev-server/audio"]`
- Why: CLI now runs audio capture in-process; cpal is needed for OS audio input
- Dependencies: Step 4.2
- Risk: Low — additive dependency change

#### Step 5.2 — Rewrite audio start logic in `start.rs`
**File:** `cli/src/commands/start.rs`
- Action:
  - **Remove:** `try_start_audio_server()` function (~60 lines)
  - **Remove:** `has_audio_binary()` function (~10 lines)
  - **Add:** New function `try_start_audio_in_process(loader: &PluginLoader, ws_port: u16, verbose: bool) -> Option<AudioHandle>` that:
    1. Gets `loader.dev_processor_vtable()` — if None, prints info message and returns None
    2. Creates `FfiProcessor::new(vtable)`
    3. Calls `ffi_processor.set_sample_rate(sample_rate)` with the system default sample rate
    4. Creates `AudioServer::new(ffi_processor, config)`
    5. Spawns the audio server on the existing tokio runtime as a background task
    6. Returns a handle that can be used for graceful shutdown
  - **Update:** `run_dev_servers()` to call `try_start_audio_in_process` instead of `try_start_audio_server`
  - **Simplify:** `wait_for_shutdown()` — remove `Option<Child>` for audio process; audio stops when the tokio runtime is dropped
  - Gate the audio logic behind `#[cfg(feature = "audio-dev")]`
- Why: Core integration point — replaces separate process with in-process FFI
- Dependencies: Steps 3.1, 4.2, 5.1
- Risk: High — central orchestration logic; must maintain existing WS server + UI server lifecycle

#### Step 5.3 — Update imports and shutdown handling
**File:** `cli/src/commands/start.rs`
- Action:
  - Remove `use std::process::{Child, Command, Stdio}` if no longer needed (UI server still uses `Command`)
  - Update `wait_for_shutdown()` signature: remove `audio_server: Option<Child>` parameter
  - Simplify the main loop: no audio child process monitoring
- Dependencies: Step 5.2
- Risk: Low

### Phase 6: Template Cleanup

#### Step 6.1 — Remove `dev-audio.rs` from template
**File:** `cli/sdk-templates/new-project/react/engine/src/bin/dev-audio.rs`
- Action: Delete file
- Why: SDK owns the audio binary now; user never sees it
- Dependencies: Step 5.2
- Risk: Low

#### Step 6.2 — Remove `src/bin/` directory from template
**Path:** `cli/sdk-templates/new-project/react/engine/src/bin/`
- Action: Delete directory
- Why: No binary targets remain in the template
- Dependencies: Step 6.1
- Risk: Low

#### Step 6.3 — Clean up template `Cargo.toml`
**File:** `cli/sdk-templates/new-project/react/engine/Cargo.toml.template`
- Action: Remove:
  - `wavecraft-dsp = { ... optional = true }` dependency
  - `wavecraft-dev-server = { ... optional = true }` dependency
  - `cpal = { ... optional = true }` dependency
  - `anyhow = { ... optional = true }` dependency
  - `env_logger = { ... optional = true }` dependency
  - `tokio = { ... optional = true }` dependency
  - `[features]` section entirely
  - `[[bin]]` section entirely
  - Comment about DSP crate being needed for dev-audio binary
- Why: ~15 fewer lines; zero optional dependencies; clean template
- Dependencies: Steps 6.1, 6.2
- Risk: Low

### Phase 7: Testing and Validation

#### Step 7.1 — Unit tests for `DevProcessorVTable` and `FfiProcessor`
**File:** `engine/crates/wavecraft-dev-server/src/ffi_processor.rs` (tests section)
- Action: Write tests with mock vtable functions:
  - `test_ffi_processor_lifecycle`: create → process → drop
  - `test_ffi_processor_process_dispatches_correctly`: verify process passes correct pointers
  - `test_ffi_processor_drop_calls_vtable_drop`: verify cleanup
  - `test_ffi_processor_null_safety`: verify create returns null handling
- Dependencies: Step 4.1
- Risk: Low

#### Step 7.2 — Unit tests for vtable version checking
**File:** `engine/crates/wavecraft-bridge/src/plugin_loader.rs` (tests section)
- Action: Add test for `try_load_processor_vtable` version mismatch handling
- Dependencies: Step 3.1
- Risk: Low

#### Step 7.3 — Compile-test macro vtable generation
**File:** Existing macro test infrastructure (or `target/tmp/test-macro-api/`)
- Action: Scaffold a test plugin with `wavecraft create`, build it, and verify the `wavecraft_dev_create_processor` symbol is present using `nm` or equivalent
- Dependencies: Step 2.1
- Risk: Low

#### Step 7.4 — Template validation: clean scaffold
- Action: Run `cargo run --manifest-path cli/Cargo.toml -- create TestClean --output target/tmp/test-clean`, verify:
  - No `src/bin/` directory exists
  - No audio-dev deps in Cargo.toml
  - No `[features]` section
  - No `[[bin]]` section
  - `cargo build --lib` succeeds
- Dependencies: Step 6.3
- Risk: Low

#### Step 7.5 — End-to-end: `wavecraft start` with audio
- Action: Manual test:
  1. Scaffold test plugin
  2. Run `wavecraft start`
  3. Verify audio server starts in-process (log output shows vtable loaded)
  4. Verify meters appear in browser UI
  5. Verify Ctrl+C cleanly shuts everything down
- Dependencies: All steps
- Risk: Medium — end-to-end test depends on audio hardware

#### Step 7.6 — Backward compatibility: old plugin without vtable
- Action: Build a plugin without the vtable export (simulate older SDK), run `wavecraft start`, verify graceful fallback to metering-only mode
- Dependencies: Steps 3.1, 5.2
- Risk: Low

#### Step 7.7 — CI validation
- Action: Run `cargo xtask ci-check` to verify all lint + tests pass across the workspace
- Dependencies: All steps
- Risk: Low

## Testing Strategy

- **Unit tests:**
  - `ffi_processor.rs`: Mock vtable function pointers, lifecycle, dispatch
  - `plugin_loader.rs`: Version mismatch, missing symbol
- **Integration tests:**
  - Macro generates symbol → `nm` validates
  - CLI loads vtable from real cdylib
  - Template validation (scaffold + build)
- **Manual tests:**
  - End-to-end `wavecraft start` with audio input
  - Backward compatibility with older plugins
  - Ctrl+C shutdown behavior

## Risks & Mitigations

- **Risk:** Panic across FFI boundary causes UB
  - Mitigation: `catch_unwind` in every generated vtable function (Step 2.1)

- **Risk:** Cross-allocator memory corruption
  - Mitigation: All alloc/free happens inside the dylib via vtable functions; CLI never deallocates processor memory

- **Risk:** Library unloaded before processor dropped
  - Mitigation: `PluginLoader` struct owns `Library` by value, dropped last (LIFO); documented invariant

- **Risk:** Breaking change for existing SDK users
  - Mitigation: Vtable symbol is optional; CLI falls back gracefully (Step 3.1)

- **Risk:** cpal compiles to larger CLI binary
  - Mitigation: Behind `audio-dev` feature flag, default-on; opt out with `--no-default-features`

## Success Criteria

- [ ] `dev-audio.rs` does not exist in scaffolded projects
- [ ] Template `Cargo.toml` has no optional audio deps, no `[features]`, no `[[bin]]`
- [ ] `wavecraft start` detects vtable from cdylib and runs audio in-process
- [ ] Older plugins without vtable export → graceful metering-only fallback
- [ ] `catch_unwind` prevents DSP panics from crashing CLI
- [ ] All existing CI checks pass (`cargo xtask ci-check`)
- [ ] Meters visible in browser UI during `wavecraft start` with audio
