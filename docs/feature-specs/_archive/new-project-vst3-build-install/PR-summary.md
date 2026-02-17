## Summary

This feature adds a complete top-level bundle workflow for generated Wavecraft projects:

- `wavecraft bundle` (build + bundle)
- `wavecraft bundle --install` (build + bundle + local VST3 install on macOS)

The final implementation is now centered on `cli/src/commands/bundle_command.rs`, with CLI wiring in `cli/src/main.rs`/`cli/src/commands/mod.rs`, template/validation updates, and focused tests.

## Final Implementation Scope

### CLI / Command Flow

- Implemented bundle command in `cli/src/commands/bundle_command.rs`
- Wired `BundleCommand` through:
  - `cli/src/main.rs`
  - `cli/src/commands/mod.rs`
- Supports both build-only and install flows with command-accurate diagnostics
- Removes temporary `CARGO_MANIFEST_DIR` leakage during bundling to ensure execution against generated project workspace

### Validation and Template Paths

- Updated template validation + CI coverage:
  - `.github/workflows/template-validation.yml`
  - `engine/xtask/src/commands/validate_template.rs`
  - `sdk-template/engine/xtask/src/main.rs`
- Updated SDK template/build-related files used by generated projects

### Tests

- `cli/tests/bundle_command.rs`
  - verifies command help and context diagnostics
  - verifies invalid-context errors for both command variants
  - verifies project root detection from nested directories
- unit tests in `cli/src/commands/bundle_command.rs`
  - verify root detection and install error diagnostics/recovery messaging

### QA Follow-up Cleanup (this update)

- Corrected invalid-context wording to match the invoked command:
  - `wavecraft bundle`
  - `wavecraft bundle --install`
- Removed obsolete prune reference from `scripts/setup-dev-template.sh`
- Removed dead legacy file `cli/src/commands/bundle.rs`

## Validation

Executed locally:

- `cargo test --manifest-path cli/Cargo.toml --test bundle_command -- --nocapture`
- `cargo xtask validate-template`
- `cargo xtask ci-check --full`

All checks passed after QA follow-up updates.
