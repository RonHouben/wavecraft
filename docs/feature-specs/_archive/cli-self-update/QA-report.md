# QA Report: CLI Self-Update

**Date**: 2026-02-08
**Reviewer**: QA Agent
**Status**: PASS

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 4 |
| Info | 3 |

**Overall**: PASS — No Critical, High, or Medium issues. All findings are Low or informational.

## Automated Check Results

**Note:** Automated checks (linting, type-checking, tests) were run by the Tester agent via `cargo xtask ci-check` prior to QA review. Results documented in test-plan.md.

- Linting: ✅ PASSED (cargo fmt, clippy, ESLint, Prettier)
- Tests: ✅ PASSED (155 engine + 28 UI + 7 CLI unit + 2 CLI integration)

## Findings

| ID | Severity | Category | Description | Location | Recommendation |
|----|----------|----------|-------------|----------|----------------|
| 1 | Low | Error Handling | `get_installed_version()` does not check `output.status.success()` before parsing stdout. If `wavecraft --version` exits non-zero, an empty/garbage string is returned as `Ok(version)` | [update.rs](../../cli/src/commands/update.rs#L113-L126) | Add an exit status check before parsing. The caller already handles `Err`, so returning `bail!()` on non-zero exit would be correct. |
| 2 | Low | Test Coverage | No unit test for `get_installed_version()` version parsing logic. The `strip_prefix("wavecraft ").unwrap_or(...)` path at line 121-123 is untested. | [update.rs](../../cli/src/commands/update.rs#L112-L126) | Add a unit test that validates the parsing (e.g., extract a `parse_version_output(stdout: &str) -> String` helper and test it). |
| 3 | Low | Test Coverage | No unit test for `print_summary()` logic branches. The function has 4 distinct output paths (all success, CLI failed + project success, project errors bail, CLI failed + no project) — none are tested. | [update.rs](../../cli/src/commands/update.rs#L172-L210) | Consider extracting the decision logic into a pure function that returns an enum (e.g., `SummaryOutcome`) and testing that. Low priority — the manual testing in TC-004 through TC-008 covers the end-to-end behavior. |
| 4 | Low | Test Coverage | Project detection unit tests (`test_detects_engine_only`, `test_detects_ui_only`, `test_detects_both`) only test `Path::exists()` on temp directories. They do not exercise `update_project_deps()` itself and do not cover the "no project markers" case. | [update.rs](../../cli/src/commands/update.rs#L273-L320) | Add a test that verifies the "neither engine nor ui" path. The existing tests are useful but test `std::path::Path::exists()` rather than the function's behavior. |
| 5 | Info | Code Quality | `project_errors.clone()` in `print_summary()` creates an unnecessary heap copy of the errors Vec. Could use a reference instead. | [update.rs](../../cli/src/commands/update.rs#L183-L186) | Negligible for a CLI tool. Could pattern-match with `ref errors` or take ownership via `project: ProjectUpdateResult` (not `&ProjectUpdateResult`). No action needed. |
| 6 | Info | UX | No timeout on `cargo install` subprocess. If the command hangs (network stall, compilation freeze), `wavecraft update` blocks indefinitely. | [update.rs](../../cli/src/commands/update.rs#L49-L52) | Acceptable for v0.9.1. Consider wrapping with a timeout in a future version (e.g., using `tokio` or spawning a thread with `recv_timeout`). The LLD acknowledges the 30-60s compile time as expected. |
| 7 | Info | Documentation | Implementation deviates from LLD by simplifying `SelfUpdateResult` enum variants. LLD designed `Updated { old_version, new_version }` and `Failed(String)` with data, but implementation uses unit variants. | [update.rs](../../cli/src/commands/update.rs#L8-L16) vs [LLD section 3.1](./low-level-design-cli-self-update.md) | Valid simplification — version information is printed directly in `update_cli()` rather than stored in the enum. Behavior matches requirements. Test plan already notes this. |

## Detailed Analysis

### Real-Time Safety

N/A — This is CLI code, not audio-thread code. No real-time safety concerns.

### Domain Separation

✅ All changes are contained within `cli/` crate. No engine, UI, or bridge code was modified. Project detection uses filesystem checks only (no imports from engine crates for this logic).

### Error Handling

✅ Well-designed two-phase error model:
- Phase 1 (CLI self-update) **never** propagates errors — all failures captured as `SelfUpdateResult::Failed` and printed as warnings. This ensures Phase 2 always runs.
- Phase 2 (project deps) collects errors independently for Rust and npm updates. Only `bail!`s at the end if errors exist.
- `print_summary()` correctly handles all 6 combinations of (CLI success/fail/updated) × (project/no-project).

One minor gap: Finding #1 (`get_installed_version` exit status). The impact is minimal since the caller handles the `Err` case gracefully (line 92-96 falls back to printing "updated" without version).

### Security

✅ No security concerns:
- Uses `Command::new()` with explicit args — no shell injection risk.
- `cargo install wavecraft` downloads from crates.io (same trust model as initial installation).
- No credential handling or secret exposure.
- No user input is passed to subprocess commands.

### Coding Standards Compliance

| Check | Status | Notes |
|-------|--------|-------|
| `///` doc comments on all functions | ✅ | All 7 functions documented |
| `anyhow::Result` error handling | ✅ | Consistent with project patterns |
| snake_case functions | ✅ | `update_cli`, `is_already_up_to_date`, `get_installed_version`, etc. |
| PascalCase enums | ✅ | `SelfUpdateResult`, `ProjectUpdateResult` |
| No `unwrap()` in production code | ✅ | Only `unwrap_or` at line 123 (safe fallback), `unwrap()` only in `#[cfg(test)]` |
| No `unsafe` code | ✅ | None |
| `println!` for CLI output | ✅ | Coding standards explicitly allow `println!` for CLI commands |
| Functions under 50 lines | ✅ | `update_cli()` is ~50 lines (largest), all others well under |

### Test Adequacy

| Area | Coverage | Assessment |
|------|----------|------------|
| `is_already_up_to_date()` parsing | 4 unit tests | Good — covers true, false, empty, prefixed |
| Project marker detection | 3 unit tests | Adequate — tests filesystem detection |
| Help text | 2 integration tests | Good — validates user-facing documentation |
| Network-dependent flows | 4 ignored integration tests | Appropriate — tests exist but are `#[ignore]` for CI |
| Manual end-to-end | 8 test cases (TC-001 to TC-008) | Thorough — covers all user stories |
| `get_installed_version()` parsing | 0 unit tests | Gap (Finding #2) |
| `print_summary()` logic | 0 unit tests | Gap (Finding #3) |

### User Story Verification

| Story | Criteria Met | Notes |
|-------|-------------|-------|
| US-1: Self-Update CLI First | ✅ All verifiable criteria met | "Updated to X.Y.Z (was A.B.C)" path untestable without publishing newer version |
| US-2: Work from Any Directory | ✅ All criteria met | Confirmed via TC-004 (from /tmp), TC-005 (from repo root), TC-006 (from generated project) |
| US-3: Graceful Error Handling | ✅ All criteria met | Code review (TC-008) confirms Phase 1 failure never blocks Phase 2 |
| US-4: Version Change Notification | ✅ All verifiable criteria met | Re-run hint verified in code review; version display path untestable |
| US-5: Updated Help Text | ✅ All criteria met | TC-002 and TC-003 confirm |

### Template Fix (Separate Commit)

The `cli/src/template/mod.rs` change is a clean, minimal test fix:
1. Removes trailing whitespace on one line (cosmetic)
2. Adds missing `wavecraft-dev-server` dependency to test input data

This correctly fixes the pre-existing `test_apply_local_dev_overrides` failure without modifying production logic.

## Architectural Concerns

None. The implementation cleanly extends the existing `update` command without introducing new dependencies, new crates, or architectural changes. The two-phase pattern with independent error handling is well-suited for the CLI domain.

## Handoff Decision

**Target Agent**: Architect
**Reasoning**: All findings are Low/Info severity — no code fixes required before merge. The implementation is complete, tests pass, and quality is verified. Ready for architectural documentation review and PO handoff.

**Optional improvements** (Findings #1-#4) can be addressed in a follow-up patch if desired, but are not blocking.
