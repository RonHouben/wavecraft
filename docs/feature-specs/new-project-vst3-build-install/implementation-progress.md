# Implementation Progress — new-project-vst3-build-install

## Date

2026-02-17

## Kickoff Scope Completed

- Added top-level CLI command surface for canonical user command:
  - `wavecraft bundle --install`
- Added command routing in:
  - `cli/src/main.rs`
  - `cli/src/commands/mod.rs`
- Implemented `BundleCommand` in:
  - `cli/src/commands/bundle.rs`
- Implemented project-context/root validation with actionable diagnostics:
  - resolves Wavecraft project root from current directory or ancestors
  - errors clearly outside valid context
- Implemented delegation to generated project command:
  - `cargo xtask bundle --install`
  - preserves delegated stdout/stderr output
  - propagates non-zero exit with delegated command context
- Updated `wavecraft create` post-create guidance to include canonical command.
- Added tests for new command surface and behavior:
  - command/help visibility
  - `--install` requirement
  - invalid-context diagnostics
  - ancestor root detection path

## Deferred to Next Pass

- Extend generated template `xtask` contract to fully support/standardize `bundle --install` implementation details in `sdk-template/engine/xtask/src/main.rs`.
- Add/expand template validation checks for this contract in `engine/xtask/src/commands/validate_template.rs` and CI template-validation workflow.
- Update getting-started docs to fully align canonical install workflow where needed.

## Validation Run in This Session

- `cargo fmt --manifest-path cli/Cargo.toml`
- `cargo test --manifest-path cli/Cargo.toml`
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`

All passed for CLI scope.
