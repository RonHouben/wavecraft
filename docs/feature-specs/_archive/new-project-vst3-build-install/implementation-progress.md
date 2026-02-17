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

- (None from the prior deferred list — all prior-pass deferred items completed in this pass.)

## Pass 2 Scope Completed

- Extended generated template `xtask` contract in `sdk-template/engine/xtask/src/main.rs`:
  - Added `bundle --install` support with staged execution (`bundle` first, then install only on success).
  - Added macOS-first VST3 install flow to `~/Library/Audio/Plug-Ins/VST3`.
  - Added deterministic replacement behavior for existing installed plugin bundles.
  - Added robust diagnostics for:
    - missing bundled artifact under `target/bundled`
    - destination directory creation and copy failures
    - remediation hints (permissions, close DAW, retry)
  - Kept dry-run/config validation path via `bundle --check`, including `--install` validation messaging.

- Hardened template validation checks in `engine/xtask/src/commands/validate_template.rs`:
  - Asserts generated `cargo xtask bundle --help` includes `--install`.
  - Validates `cargo xtask bundle --check --install` command path.
  - Added captured-output assertion helpers for stronger contract checks.

- Hardened CI template-validation workflow in `.github/workflows/template-validation.yml`:
  - Added explicit command-surface check for `--install` in generated xtask help.
  - Switched dry-run contract validation to `cargo xtask bundle --check --install`.

- Aligned docs in `docs/guides/sdk-getting-started.md`:
  - Canonical user-facing build/install flow is now `wavecraft bundle --install`.
  - Added bundle/install CLI reference section.
  - Updated DAW testing and troubleshooting instructions to prefer canonical flow.
  - Kept `cargo xtask install` labeled as advanced/internal.

## Validation Run in This Session

- `cargo fmt --manifest-path engine/xtask/Cargo.toml`
- `cargo fmt --manifest-path sdk-template/engine/xtask/Cargo.toml`
- `cargo test --manifest-path engine/xtask/Cargo.toml validate_template`
- `cargo check --manifest-path sdk-template/engine/xtask/Cargo.toml`
- `cargo xtask validate-template`
- `cargo clippy --manifest-path engine/xtask/Cargo.toml --all-targets -- -D warnings`
- `cargo clippy --manifest-path sdk-template/engine/xtask/Cargo.toml --all-targets -- -D warnings`

All passed for this pass scope.

## Deferred Enhancement Implemented (Install Failure Path Tests)

- Implemented deterministic failure-path tests in generated template xtask install logic (`sdk-template/engine/xtask/src/main.rs`) by introducing filesystem-operation injection for install internals.
- Added automated tests for:
  - destination directory create failure (permission-denied simulation)
  - bundle copy failure (file-lock-like simulation)
  - existing-bundle replace failure (file-lock-like simulation)
- Hardened diagnostics to include operation, path, OS error text, and remediation hints in user-facing install errors.
- Updated template validation command (`engine/xtask/src/commands/validate_template.rs`) to run generated xtask unit tests, enforcing this coverage in local CI-equivalent validation.

### Validation Run for Deferred Enhancement

- `cargo fmt --manifest-path sdk-template/engine/xtask/Cargo.toml`
- `cargo fmt --manifest-path engine/xtask/Cargo.toml`
- `cargo test --manifest-path sdk-template/engine/xtask/Cargo.toml`
- `cargo clippy --manifest-path sdk-template/engine/xtask/Cargo.toml --all-targets -- -D warnings`
- `cargo test --manifest-path engine/xtask/Cargo.toml`
- `cargo clippy --manifest-path engine/xtask/Cargo.toml --all-targets -- -D warnings`

All passed for this deferred enhancement scope.

## QA Remediation Pass (2026-02-17)

- Resolved docs/CLI behavior mismatch for `wavecraft bundle`:
  - `wavecraft bundle` now delegates build-only flow to `cargo xtask bundle`.
  - `wavecraft bundle --install` remains canonical install flow and delegates to `cargo xtask bundle --install`.
- Added positive-path CLI delegation test coverage in `cli/tests/bundle_command.rs`:
  - verifies `wavecraft bundle --install` delegates exact args `xtask bundle --install`
  - verifies delegation runs from resolved project root (even when invoked from nested directory).
- Resolved artifact-path contract drift in feature specs by aligning to runtime semantics:
  - source path is `target/bundled` relative to generated project root.
  - updated low-level design and implementation plan accordingly.

### Validation Run for QA Remediation

- `cargo fmt --manifest-path cli/Cargo.toml`
- `cargo test --manifest-path cli/Cargo.toml --test bundle_command -- --nocapture`
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`

All passed for this remediation scope.
