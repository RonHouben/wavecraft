# PR Summary: Processors-Crate Migration

## Summary
This PR introduces the new `wavecraft-processors` crate and migrates the SDK template engine from direct oscillator wiring to an `ExampleProcessor` integration. It updates affected engine/template wiring and accompanying SDK architecture/getting-started documentation for the new processor model.

## Changes

### Engine / Crates
- Added new crate: `engine/crates/wavecraft-processors/`
  - `Cargo.toml`
  - `src/lib.rs`
  - `src/oscillator.rs`
- Wired crate into workspace/dependency graph:
  - `engine/Cargo.toml`
  - `engine/Cargo.lock`
  - `engine/crates/wavecraft-nih_plug/Cargo.toml`
  - `engine/crates/wavecraft-nih_plug/src/lib.rs`
  - `engine/crates/wavecraft-nih_plug/src/prelude.rs`

### SDK Template
- Migrated template processor usage to `ExampleProcessor`:
  - `sdk-template/engine/src/lib.rs`
  - `sdk-template/engine/src/processors/example_processor.rs`
  - `sdk-template/engine/src/processors/mod.rs`
- Updated template UI usage references:
  - `sdk-template/ui/src/App.tsx`
  - `ui/packages/components/src/OscillatorControl.tsx`

### Documentation
- Updated architecture and getting-started docs to reflect processors crate migration:
  - `docs/architecture/sdk-architecture.md`
  - `docs/guides/sdk-getting-started.md`
  - `docs/architecture/coding-standards-rust.md`
- Added/updated feature test plan:
  - `docs/feature-specs/processors-crate-migration/test-plan.md`

## Commits (since merge base)
- `cae8b4f` feat: document processors-crate migration and update SDK architecture
- `a84a871` feat: update SDK documentation and examples, replace oscillator with example processor
- `1f741ec` feat: remove verbose flag from `wavecraft start` and standardize version flag

## Validation
Per feature test plan (`docs/feature-specs/processors-crate-migration/test-plan.md`):
- `cargo xtask ci-check`: PASS
- `cargo check -p wavecraft-processors`: PASS
- `cargo test -p wavecraft-processors`: PASS
- Template wiring and docs verification: PASS

## Notes
- This PR body is auto-generated from git history and changed files for the current branch scope.
