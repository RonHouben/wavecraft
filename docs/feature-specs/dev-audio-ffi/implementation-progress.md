# Implementation Progress: Dev Audio FFI Abstraction

## Phase 1: FFI Contract (`wavecraft-protocol`)
- [x] 1.1 — Create `dev_audio_ffi` module with `DevProcessorVTable` struct
- [x] 1.2 — Register module and re-exports in `wavecraft-protocol/src/lib.rs`

## Phase 2: Macro Code Generation (`wavecraft-macros`)
- [x] 2.1 — Add vtable FFI export to `wavecraft_plugin!` macro (with `catch_unwind`)
- [x] 2.2 — Expose vtable types through `wavecraft-nih_plug` `__internal` module

## Phase 3: CLI-Side Plugin Loader (`wavecraft-bridge`)
- [x] 3.1 — Extend `PluginParamLoader` with optional vtable loading + version check
- [x] 3.2 — Update `wavecraft-bridge` exports if needed

## Phase 4: Audio Server Refactoring (`wavecraft-dev-server`)
- [x] 4.1 — Create `ffi_processor.rs`: `DevAudioProcessor` trait + `FfiProcessor` wrapper + unit tests
- [x] 4.2 — Refactor `AudioServer` to use `DevAudioProcessor` (remove `Processor` generic)
- [x] 4.3 — Register `ffi_processor` module in `wavecraft-dev-server/src/lib.rs`
- [x] 4.4 — Update `wavecraft-dev-server/Cargo.toml` if needed

## Phase 5: CLI Integration (`wavecraft start`)
- [x] 5.1 — Add `cpal` optional dependency + `audio-dev` feature to `cli/Cargo.toml`
- [x] 5.2 — Rewrite audio start logic: remove `try_start_audio_server` / `has_audio_binary`, add `try_start_audio_in_process`
- [x] 5.3 — Simplify `wait_for_shutdown()` (remove audio child process handling)

## Phase 6: Template Cleanup
- [x] 6.1 — Delete `cli/sdk-templates/.../src/bin/dev-audio.rs`
- [x] 6.2 — Delete `cli/sdk-templates/.../src/bin/` directory
- [x] 6.3 — Clean up template `Cargo.toml.template` (remove audio deps, features, [[bin]])

## Phase 7: Testing and Validation
- [x] 7.1 — Unit tests for `FfiProcessor` (mock vtable, lifecycle, dispatch)
- [x] 7.2 — Unit tests for vtable version checking in plugin loader
- [x] 7.3 — Compile-test: macro generates `wavecraft_dev_create_processor` symbol
- [x] 7.4 — Template validation: clean scaffold (no bin/, no audio deps)
- [x] 7.5 — End-to-end: `wavecraft start` with audio input + meters in browser
- [x] 7.6 — Backward compatibility: old plugin without vtable → graceful fallback
- [x] 7.7 — CI validation: `cargo xtask ci-check` passes
