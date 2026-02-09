# Implementation Progress: Replace Python Validation with xtask

## Status: 🟢 Complete (QA fixes applied)

## QA Fixes (2026-02-09)

- [x] **Finding 1 (High):** Replaced `std::process::exit(1)` with `anyhow::bail!()` in `validate_cli_deps.rs` — aligns with all other xtask commands
- [x] **Finding 2 (Medium):** Added unit tests for `validate_dependency()` — missing crate directory and malformed TOML paths
- [x] **Finding 3 (Medium):** Moved `print_error_item()` to `xtask::output` module in `lib.rs` alongside `print_success_item()`
- [x] **Finding 5 (Low):** Added `test_publish_key_absent_is_publishable` unit test verifying absent `publish` key is treated as publishable
- [x] All checks pass: `cargo clippy -p xtask -- -D warnings`, `cargo test -p xtask` (9 validate_cli_deps tests), `cargo xtask validate-cli-deps`, `cargo xtask ci-check`

## Progress Tracker

### Phase 1: Rust Implementation (xtask command)

- [x] **Step 1.1:** Create `validate_cli_deps.rs` module
  - File: `engine/xtask/src/commands/validate_cli_deps.rs`
  - Status: Complete

- [x] **Step 1.2:** Register the command in `mod.rs`
  - File: `engine/xtask/src/commands/mod.rs`
  - Status: Complete

- [x] **Step 1.3:** Add CLI variant and dispatch in `main.rs`
  - File: `engine/xtask/src/main.rs`
  - Status: Complete

- [x] **Step 1.4:** Add unit tests
  - File: `engine/xtask/src/commands/validate_cli_deps.rs`
  - Status: Complete — 6 tests (5 from plan + 1 extra inline table format test)

### Phase 2: Workflow Changes (CI/CD)

- [x] **Step 2.1:** Replace Python steps in `continuous-deploy.yml`
  - File: `.github/workflows/continuous-deploy.yml`
  - Status: Complete — replaced 2 Python heredoc steps with single xtask step

### Phase 3: Documentation Updates

- [x] **Step 3.1:** Update `cli/Cargo.toml` comment
  - File: `cli/Cargo.toml`
  - Status: Complete — added NOTE comment referencing xtask

- [x] **Step 3.2:** Update `docs/guides/ci-pipeline.md`
  - File: `docs/guides/ci-pipeline.md`
  - Status: Complete — added "CLI Dependency Validation" section

### Phase 4: Test Plan & Cleanup

- [x] **Step 4.1:** Create test plan for this feature
  - File: `docs/feature-specs/replace-python-with-xtask/test-plan.md`
  - Status: Complete

- [x] **Step 4.2:** Supersede old test plan with note
  - Status: N/A — `fix-cd-stale-python-validation` feature spec does not exist on main branch

## Verification Checklist

- [x] `cargo xtask validate-cli-deps` exits 0 on current repo
- [x] 6 unit tests pass (`cargo test -p xtask`)
- [x] `continuous-deploy.yml` contains zero Python heredoc scripts
- [x] `cargo xtask ci-check` passes
- [x] Documentation updated (ci-pipeline.md, cli/Cargo.toml)
- [x] Test plan ready for tester agent
