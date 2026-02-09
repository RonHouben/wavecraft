# QA Report: Replace Python Validation with xtask

**Date**: 2026-02-09
**Reviewer**: QA Agent
**Status**: PASS

## Summary

| Severity | Count | Resolved |
|----------|-------|----------|
| Critical | 0 | — |
| High | 1 | ✅ |
| Medium | 2 | ✅ |
| Low | 2 | ✅ (Finding 4 acknowledged as N/A for small feature) |

**Overall**: PASS — All findings resolved

## Final Sign-off

**Date**: 2026-02-09
**Status**: ✅ APPROVED

All 4 actionable findings from the initial QA review have been fixed by the Coder and verified:

| Finding | Severity | Resolution | Verification |
|---------|----------|------------|--------------|
| #1 — `process::exit` → `anyhow::bail!()` | High | Fixed | No `process::exit` calls remain in `validate_cli_deps.rs` |
| #2 — Unit tests for `validate_dependency()` | Medium | Fixed | `test_validate_dependency_missing_crate_dir` and `test_validate_dependency_malformed_toml` added |
| #3 — `print_error_item()` in shared module | Medium | Fixed | Moved to `xtask::output` module in `lib.rs` |
| #5 — Absent `publish` key test | Low | Fixed | `test_publish_key_absent_is_publishable` added |

**Automated verification:**
- `cargo clippy -p xtask -- -D warnings` — ✅ Clean (zero warnings)
- `cargo test -p xtask -- validate_cli_deps` — ✅ 9/9 tests passed
- No new issues introduced by the fixes

**Handoff**: → **Architect** for architecture documentation review and updates, then → **PO** for roadmap update and feature spec archival.

## Automated Check Results

**Note:** Automated checks (linting, type-checking, tests) were run by the Tester agent via `cargo xtask ci-check` prior to QA review. Results documented in test-plan.md.

- Linting: ✅ PASSED
- Tests: ✅ PASSED (6 unit tests + full ci-check)

## Findings

| ID | Severity | Category | Description | Location | Recommendation |
|----|----------|----------|-------------|----------|----------------|
| 1 | High | Error Handling | `std::process::exit(1)` bypasses `anyhow::Result` error flow | `validate_cli_deps.rs:64,110` | Replace with `anyhow::bail!()` |
| 2 | Medium | Test Coverage | `validate_dependency()` function not directly unit-tested | `validate_cli_deps.rs` | Add tests for missing crate path and missing `publish` key |
| 3 | Medium | Consistency | Local `print_error_item()` duplicates lib-level pattern | `validate_cli_deps.rs:207-210` | Move to `xtask::output` module |
| 4 | Low | Documentation | Missing feature spec artifacts (low-level-design, implementation-plan) | `docs/feature-specs/replace-python-with-xtask/` | Add files per agent-development-flow.md |
| 5 | Low | Test Coverage | `publish` field absent from crate TOML not explicitly tested | `validate_cli_deps.rs` tests | Add test asserting no error when `publish` key is omitted |

## Detailed Findings

### Finding 1 — `std::process::exit(1)` bypasses error propagation (High)

**Location:** `engine/xtask/src/commands/validate_cli_deps.rs` lines 64 and 110

The `run()` function signature returns `Result<()>`, but on two error paths it calls `std::process::exit(1)` instead of returning an `Err`. This is the **only** xtask command module that does this — all other commands (`lint.rs`, `sign.rs`, `desktop.rs`, `validate_template.rs`, etc.) use `anyhow::bail!()` to propagate errors, which are then caught by `main.rs` line 404 (`if let Err(e) = result { ... std::process::exit(1); }`).

**Why this matters:**
- **Inconsistent contract**: The return type `Result<()>` promises to signal failure via `Err(...)`, but `process::exit` short-circuits the entire process, skipping `main()`'s error handler and any future cleanup logic.
- **Untestable**: `std::process::exit()` terminates the test process, making it impossible to write unit tests for the error paths of `run()`.
- **Coding standard violation**: Per `coding-standards-rust.md` xtask command conventions — "Use `anyhow::Result` for error propagation."

**Recommendation:** Replace both `std::process::exit(1)` calls with `anyhow::bail!()`:
```rust
// Line 64: no deps found
anyhow::bail!("No wavecraft-* dependencies found in cli/Cargo.toml");

// Line 110: validation errors
anyhow::bail!("{} validation error(s) found", errors.len());
```

### Finding 2 — `validate_dependency()` lacks direct unit tests (Medium)

**Location:** `engine/xtask/src/commands/validate_cli_deps.rs` — `validate_dependency()` function

The `validate_dependency()` function performs file-system operations (checking crate path existence, reading and parsing crate TOML, checking `publish` field). The existing test `test_detect_unpublishable_crate` only tests the TOML parsing logic inline (duplicating the check logic) rather than calling `validate_dependency()` itself.

**Untested paths:**
1. Crate directory does not exist → should produce "crate Cargo.toml not found" error
2. Crate TOML is malformed → should produce "failed to parse" error
3. Crate TOML has no `publish` field at all → should pass (current code correctly handles this via `Some(false)` check, but it's not verified by tests)

**Recommendation:** Add targeted tests using `tempdir` for the file-system dependent paths, or at minimum add a synthetic TOML test that verifies `publish` absent is treated as publishable.

### Finding 3 — `print_error_item()` should live in shared output module (Medium)

**Location:** `engine/xtask/src/commands/validate_cli_deps.rs` lines 207–210

A local `print_error_item()` function is defined in this module while the symmetric `print_success_item()` is imported from `xtask::output::*`. Both follow the same pattern (`"  {marker} {text}"`) but output to different streams (`eprintln!` vs `println!`).

**Recommendation:** Move `print_error_item()` to `xtask::output` alongside `print_success_item()` for consistency. This makes the pattern reusable by other commands.

### Finding 4 — Missing feature spec documents (Low)

**Location:** `docs/feature-specs/replace-python-with-xtask/`

Per `agent-development-flow.md`, the standard feature spec folder should contain `low-level-design-{feature}.md` and `implementation-plan.md`. The directory currently only has `implementation-progress.md` and `test-plan.md`.

**Recommendation:** Add the missing documents, or if they were intentionally omitted for this small feature, document that decision in `implementation-progress.md`.

### Finding 5 — Missing test for `publish` key absent scenario (Low)

**Location:** `engine/xtask/src/commands/validate_cli_deps.rs` tests

The code correctly treats a missing `publish` field as publishable (only `publish = false` triggers an error). However, no test explicitly verifies this behavior. A regression that changes the `Some(false)` check to `None` would silently break the logic for crates that don't set `publish` at all (which is the common case).

**Recommendation:** Add a test:
```rust
#[test]
fn test_publish_key_absent_is_publishable() {
    let crate_toml = r#"
[package]
name = "wavecraft-foo"
version = "0.1.0"
"#;
    let parsed: toml::Value = crate_toml.parse().unwrap();
    let publish = parsed
        .get("package")
        .and_then(|p| p.get("publish"))
        .and_then(|p| p.as_bool());
    assert_eq!(publish, None, "Absent publish key should be None, not Some(false)");
}
```

## Architectural Concerns

None. The implementation correctly follows the xtask command pattern documented in `coding-standards-rust.md`:
- Module under `commands/` with `pub fn run()` entry point
- Registered in `commands/mod.rs` and wired in `main.rs`
- Uses `ValidateCliDepsConfig` struct for configuration
- TOML parsing handles both inline and section-style formats

The CI workflow change is minimal and correct — single step with proper `working-directory: engine` and all prerequisites (Rust toolchain, system deps) installed prior.

## Handoff Decision

**Target Agent**: Architect
**Reasoning**: All QA findings resolved, 0 Critical/High issues remaining. Feature is APPROVED. Architect should review implementation against architectural decisions, update `docs/architecture/` if needed, then hand off to PO for roadmap update and spec archival.
