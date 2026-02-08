## Summary

Replace the template-embedded `dev-audio.rs` binary with an FFI/dlopen approach for dev audio processing. The CLI now loads the user's DSP processor from their compiled cdylib via a C-ABI FFI vtable (`DevProcessorVTable`), running audio capture in-process via cpal. Users never see or touch audio capture code — the SDK template is cleaner and the developer experience is significantly improved.

**Key outcomes:**
- Deleted `dev-audio.rs` from the SDK template
- Removed 6 optional dependencies, `[features]` section, and `[[bin]]` section from template `Cargo.toml`
- Audio processing runs in-process in the CLI (no spawned subprocess)
- Backward compatible: plugins without vtable gracefully fall back to metering-only mode

## Changes

- **Engine/DSP**:
  - `wavecraft-protocol`: New `DevProcessorVTable` FFI contract with versioning (`dev_audio_ffi.rs`)
  - `wavecraft-macros`: `wavecraft_plugin!` macro now generates vtable FFI exports with `catch_unwind` guards
  - `wavecraft-bridge`: `PluginParamLoader` extended with optional vtable loading and proper drop ordering
  - `wavecraft-dev-server`: New `DevAudioProcessor` trait, `FfiProcessor` wrapper, refactored `AudioServer` to accept `Box<dyn DevAudioProcessor>`
  - `wavecraft-nih_plug`: Re-exports for `DevProcessorVTable` and `DEV_PROCESSOR_VTABLE_VERSION`

- **CLI**:
  - `start.rs`: Replaced subprocess spawning with in-process FFI audio via cpal
  - `Cargo.toml`: Added `cpal` behind `audio-dev` default feature flag
  - Template: Removed `dev-audio.rs`, cleaned up `Cargo.toml.template`

- **Documentation**:
  - `high-level-design.md`: Added "Dev Audio via FFI" section, updated crate tables and DSL architecture
  - `coding-standards.md`: Added "FFI Safety Patterns" section (catch_unwind, memory ownership, SAFETY comments)
  - `roadmap.md`: Marked feature complete
  - Feature spec archived to `_archive/dev-audio-ffi/`

## Commits

- `e06c423` feat: Implement Dev Audio FFI Abstraction and QA Findings Resolution
- `76ee431` feat: add SDK Audio Architecture Gaps to backlog and update implementation progress with completed test cases
- `acf9fbe` feat: implement Dev Audio FFI abstraction

## Related Documentation

- [Low-Level Design](../_archive/dev-audio-ffi/low-level-design-dev-audio-ffi.md)
- [Implementation Plan](../_archive/dev-audio-ffi/implementation-plan.md)
- [QA Report](../_archive/dev-audio-ffi/QA-report.md)
- [Test Plan](../_archive/dev-audio-ffi/test-plan.md)

## Testing

- [x] CI passes: `cargo xtask ci-check` (162 engine + 28 UI tests)
- [x] Linting passes: ESLint, Prettier, cargo fmt, clippy
- [x] Manual testing: 6/6 tests pass (FFI symbol, audio handle, backward compat, template clean)
- [x] Regression testing: 6/6 after QA fix cycle
- [x] QA review: PASS (after fix cycle for 1 Critical + 1 Major finding)

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated (HLD + coding standards)
- [x] No linting errors
- [x] QA sign-off received
- [x] Feature spec archived
