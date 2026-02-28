## Summary

Add a full-stack processor catalog across engine, dev-server, SDK template, and UI, including consolidation of built-in DSP processors into `wavecraft-processors` and introduction of Dev FFI v2 parameter injection at block boundaries.

This PR also updates macro/codegen and bridge loading paths to support the new processor model, and includes associated tests and architecture/workflow documentation updates.

## Changes

### Engine / DSP / Core

- Consolidated built-in processors into `engine/crates/wavecraft-processors/`.
  - Moved `gain` and `passthrough` from `wavecraft-dsp` to `wavecraft-processors`.
  - Added new processors: `saturator` and `unified_filter`.
- Updated DSP combinators and tests:
  - `chain.rs`, `chain_macro.rs`, `chain_param_extraction.rs`.
  - Removed legacy `wavecraft-dsp/src/processor.rs` and `builtins/mod.rs`.
- Updated plugin/codegen integration:
  - `wavecraft-macros` plugin codegen updates.
  - `wavecraft-core` macro/prelude exports and processor macro tests.
- Updated bridge/plugin loading integration:
  - `wavecraft-bridge/src/plugin_loader.rs` and related Cargo manifests.
- Updated protocol contract:
  - `wavecraft-protocol/src/dev_audio_ffi.rs` and crate versions/locks.

### Dev Server / Audio Runtime

- Introduced Dev FFI v2 parameter injection support:
  - `dev-server/src/audio/ffi_processor.rs`
  - `dev-server/src/audio/server/input_pipeline.rs`
  - `dev-server/src/audio/server/output_modifiers.rs`
  - `dev-server/src/audio/atomic_params.rs`
- Updated CLI/dev runtime wiring:
  - `cli/src/commands/start/audio_runtime.rs`

### UI / Template

- Updated template app and generated processor metadata wiring:
  - `sdk-template/ui/src/App.tsx`
  - `sdk-template/ui/src/generated/processors.ts`
- Updated template engine signal-chain usage:
  - `sdk-template/engine/src/lib.rs`
- Added/updated UI test coverage:
  - `ui/packages/components/src/TemplateApp.test.tsx`

### Documentation

- Updated architecture/workflow docs for processor and Dev FFI changes.
- Added archived feature-spec docs for processor consolidation and Dev FFI progress.
- Updated roadmap entries corresponding to this work.

## Commits

- `cb4be18` docs: update development workflows and archive implementation progress for Dev FFI v2
- `48aced2` feat: introduce Dev FFI v2 with parameter injection support
- `0f912be` feat: Consolidate built-in processors into wavecraft-processors and remove legacy stubs from wavecraft-dsp
- `2428933` feat: Implement new DSP processors and update tests

## Changed Files Snapshot

- **50 files changed**
- **1708 insertions**, **406 deletions**

## Related Documentation

- `docs/architecture/development-workflows.md`
- `docs/architecture/sdk-architecture.md`
- `docs/architecture/coding-standards-rust.md`
- `docs/feature-specs/dev-ffi-parameter-injection/implementation-progress.md`
- `docs/feature-specs/_archive/processors-crate-consolidation/*`

## Testing / Validation Checklist

- [ ] Engine unit/integration tests pass
- [ ] UI tests pass
- [ ] Macro/codegen paths validated for processor catalog changes
- [ ] Dev FFI v2 parameter injection verified in dev runtime
- [ ] Template app processor controls render as expected
- [ ] No regressions in bridge/plugin loader behavior

## PR Checklist

- [x] Merge-base reviewed against `origin/main`
- [x] Commits and changed files analyzed
- [x] PR summary generated from actual branch diff
- [ ] Reviewer verification complete
