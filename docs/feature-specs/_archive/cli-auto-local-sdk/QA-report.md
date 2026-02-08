# QA Report: Auto-Detect Local SDK for Development

**Date**: 2026-02-08
**Reviewer**: QA Agent
**Status**: PASS

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High | 0 |
| Medium | 1 |
| Low | 1 |

**Overall**: PASS (0 Critical/High issues)

## Automated Check Results

**Note:** Automated checks (linting, type-checking, tests) were run by the Tester agent via `cargo xtask ci-check` prior to QA review. Results documented in test-plan.md.

- Linting: ✅ PASSED (Rust fmt, Clippy, ESLint, Prettier)
- Tests: ✅ PASSED (164 engine + 28 UI + 32 CLI unit tests)

## Scope of Review

### Changed Files

| File | Change Type | Lines Changed |
|------|-------------|---------------|
| `cli/src/sdk_detect.rs` | New (169 lines) | Detection module with 9 unit tests |
| `cli/src/main.rs` | Modified (+1 line) | `mod sdk_detect;` registration |
| `cli/src/commands/create.rs` | Modified (+22 lines) | Auto-detection wiring in `execute()` |
| `cli/src/commands/start.rs` | Modified (formatting) | `cargo fmt` changes only, no logic |
| `docs/guides/sdk-getting-started.md` | Modified (+25 lines) | SDK Development section added |
| `docs/feature-specs/cli-auto-local-sdk/implementation-plan.md` | New | Feature specification |
| `docs/feature-specs/cli-auto-local-sdk/implementation-progress.md` | New | Progress tracking |
| `docs/feature-specs/cli-auto-local-sdk/test-plan.md` | New | Test results (9/9 pass) |

### Related Files Reviewed (Unchanged)

| File | Reason |
|------|--------|
| `cli/src/commands/new.rs` | Near-duplicate of `create.rs` — checked for impact |
| `cli/src/commands/mod.rs` | Verified module exports |

## Findings

| ID | Severity | Category | Description | Location | Recommendation |
|----|----------|----------|-------------|----------|----------------|
| 1 | Medium | Code Quality / Dead Code | `new.rs` is a 146-line near-duplicate of `create.rs` that is NOT exported from `commands/mod.rs` and NOT imported in `main.rs`. It contains a duplicated `find_local_sdk_path()` function and lacks the new auto-detection logic. If ever re-enabled without updating, it would silently use git tags instead of auto-detecting. | `cli/src/commands/new.rs` | Delete this dead file. It serves no purpose — the `create` command handles all project creation. If preserved intentionally for a future `new` subcommand, it should be tracked as tech debt and tagged with a `TODO` comment. |
| 2 | Low | Edge Case (Documented) | `is_cargo_run_binary()` checks for any path component == `"target"`. A user whose home or project directory is literally named `target` (e.g., `/home/target/projects/wavecraft/bin/wav`) would trigger a false positive on the first check. | `cli/src/sdk_detect.rs:57` | Acceptable risk. The dual-check with `find_monorepo_root()` requiring the SDK marker (`engine/crates/wavecraft-nih_plug/Cargo.toml`) makes a true false positive virtually impossible. This edge case is already documented in the implementation plan's Risks section. No code change recommended. |

## Code Quality Assessment

### `sdk_detect.rs` — Excellent

- **Documentation**: Module-level doc comment explains the full detection algorithm. All three functions have clear `///` doc comments.
- **Error handling**: Uses `Option` chaining with `?` (via `.ok()`) throughout. Any failure in `current_exe()`, `canonicalize()`, or filesystem checks results in a safe `None` return (falls back to git tags).
- **Separation of concerns**: Three focused functions — `detect_sdk_repo()` (orchestrator), `is_cargo_run_binary()` (heuristic check), `find_monorepo_root()` (filesystem walk).
- **Symlink handling**: `canonicalize()` on the exe path resolves symlinks before inspection.
- **Test coverage**: 9 unit tests covering debug/release/workspace target paths, installed binary paths (`~/.cargo/bin`, `/usr/local/bin`), monorepo found/not-found with real filesystem via `TempDir`.
- **`unwrap()` usage**: Only in `#[cfg(test)]` code — acceptable per coding standards.

### `create.rs` Changes — Clean

- **3-tier precedence** is clear and correctly ordered: (1) explicit `--local-sdk` → `find_local_sdk_path()`, (2) auto-detect → `sdk_detect::detect_sdk_repo()`, (3) `None` → git tags.
- **User communication**: Informative notice with actionable escape hatch ("install via: cargo install wavecraft").
- **No structural changes**: `CreateCommand` struct untouched; behavior change is contained in `execute()`.

### `start.rs` Changes — Formatting Only

- All changes are `cargo fmt` reformatting (import ordering, line wrapping). No logic changes. Verified via diff inspection.

### Documentation — Accurate

- New "SDK Development" section in `sdk-getting-started.md` correctly documents when detection triggers/doesn't, and how to override.

## Checklist Results

### Real-Time Safety
- N/A — CLI code, not audio thread code.

### Domain Separation
- [x] Detection logic isolated in `sdk_detect.rs` (no template or validation concerns)
- [x] Integration point in `create.rs` is minimal (one `if/else if/else` block)
- [x] No changes to template generation logic — the existing `TemplateVariables` and `apply_local_dev_overrides` handle path deps

### Security
- [x] No hardcoded secrets
- [x] No user input passed to shell commands
- [x] Filesystem inspection is read-only (no writes, no modifications to existing files)

### Code Quality
- [x] Functions under 30 lines
- [x] Clear naming following Rust conventions
- [x] Public API documented with `///`
- [x] Tests exist for all new public functions
- [x] No dead code in new files

## Architectural Concerns

No architectural concerns. The implementation:
- Uses runtime binary path detection (no build scripts needed)
- Follows the existing `--local-sdk` code path for template generation
- Does not modify the IPC, DSP, or UI layers

## Handoff Decision

**Target Agent**: architect
**Reasoning**: QA review PASSED with 0 Critical/High issues. The one Medium finding (dead `new.rs` file) is pre-existing technical debt, not introduced by this feature. Implementation is complete, well-tested (9/9 manual TCs + 192 automated tests), and quality-verified. Ready for architectural documentation review and roadmap update.

**Note for architect**: Finding #1 (`new.rs` dead code) is pre-existing and unrelated to this feature. It should be tracked separately as a cleanup task rather than blocking this feature.
