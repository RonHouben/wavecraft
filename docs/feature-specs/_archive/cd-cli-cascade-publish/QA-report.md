# QA Report: CD CLI Cascade Publish

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

**Overall**: PASS — No Critical or High issues. All Low findings are minor observations.

## Automated Check Results

**Note:** Automated checks were run by the Tester agent via `cargo xtask ci-check` prior to QA review (documented in test-plan.md).

- Linting: ✅ PASSED (cargo fmt, clippy, ESLint, Prettier)
- Tests: ✅ PASSED (165 Rust tests, 28 UI tests)

## LLD Compliance Verification

All four changes specified in the LLD are correctly implemented:

| LLD Change | Status | Notes |
|------------|--------|-------|
| Change 1: `[auto-bump]` skip guard on `detect-changes` | ✅ Implemented | Exact match to spec |
| Change 1b: `any_sdk_changed` aggregate output | ✅ Implemented | ORs all 4 filter outputs correctly |
| Change 2: npm auto-bump (core + components) | ✅ Implemented | Three-step pattern: determine → bump → commit+push |
| Change 3: CLI auto-bump | ✅ Implemented | Same three-step pattern, uses `sed` with `[package]` scoping |
| Change 4: CLI cascade trigger | ✅ Implemented | `needs` all 4 jobs, `!cancelled()` + result checks |
| Coding standards update | ✅ Implemented | SDK Distribution Versioning section added |

## Implementation Phase Completion

All phases complete per implementation-progress.md:

- [x] Phase 1: Infinite Loop Guard + Aggregate Output (Steps 1.1, 1.2)
- [x] Phase 2: npm Package Auto-Bump (Steps 2.1, 2.2)
- [x] Phase 3: CLI Auto-Bump + Cascade Trigger (Steps 3.1, 3.2, 3.3)
- [x] Phase 4: Coding Standards Update (Step 4.1)

## Test Coverage

12/12 test cases PASS. Both bugs found during testing (sed pattern corruption, missing upstream failure guards) were fixed in commit `87bf1c6` and verified.

## Findings

| ID | Severity | Category | Description | Location | Status |
|----|----------|----------|-------------|----------|--------|
| 1 | Low | Implicit Dependency | `publish-cli` uses `npx --yes semver` for patch version computation without a `setup-node` step. Relies on Node.js being pre-installed on GitHub-hosted Ubuntu runners (documented behavior, but implicit). The npm publish jobs explicitly use `actions/setup-node@v4`. | `.github/workflows/continuous-deploy.yml` line 171 | **FIXED** — Added `actions/setup-node@v4` step. |
| 2 | Low | Inconsistency | `fetch-depth: 0` used in `publish-npm-components` checkout but not in `publish-cli` or `publish-npm-core`. All three jobs perform `git pull --rebase` and `git push` operations which work fine with shallow clones, so there is no functional impact. | `.github/workflows/continuous-deploy.yml` lines 393, 85, 289 | **FIXED** — Added `fetch-depth: 0` to `publish-cli` and `publish-npm-core`. |
| 3 | Low | Inconsistency | `publish-npm-components` uses `always()` while `publish-cli` uses `!cancelled()` for the cancellation-aware condition. The `!cancelled()` pattern (used in `publish-cli`) is the recommended approach — it respects manual workflow cancellation. | `.github/workflows/continuous-deploy.yml` lines 388, 68 | **FIXED** — Replaced `always()` with `!cancelled()`. |
| 4 | Low | Spec Deviation | LLD specifies manual bash arithmetic for CLI patch version computation (`cut -d. -f1/2/3` + arithmetic). Implementation uses `npx --yes semver "$PUBLISHED" -i patch` instead. | `.github/workflows/continuous-deploy.yml` line 171 | No action — acceptable deviation; `npx semver` is more robust. |

## Detailed Analysis

### Correctness

**Loop prevention:** The `[auto-bump]` commit message marker correctly prevents infinite loops. When an auto-bump commit triggers the CD workflow, `detect-changes` is skipped via the `if` condition, causing all downstream jobs to be skipped. This is well-designed and avoids the drawbacks of `[skip ci]` (which would suppress other workflows).

**Cascade trigger:** The `publish-cli` condition `!cancelled() && any_sdk_changed == 'true' && (result == 'success' || result == 'skipped')` for each upstream job is the correct GitHub Actions pattern for conditional fan-in. It ensures CLI only publishes when upstream jobs succeed or were legitimately skipped (not when they fail).

**Auto-bump pattern:** The three-step pattern (determine → bump → commit+push) with a "set final version" consolidation step is clean and DRY across all three jobs. The mutual exclusivity of `steps.version.outputs.version` and `steps.bump.outputs.version` is ensured by the step `if` conditions, making the string concatenation in "Set final version" correct.

**Sed scoping fix:** The `sed -i '/^\[package\]/,/^\[/{...}'` address range correctly restricts version replacement to the `[package]` section of `cli/Cargo.toml`, preventing the critical bug that was caught during testing.

**Git conflict prevention:** `git pull --rebase origin main` before each push correctly handles potential conflicts from parallel auto-bump commits across jobs. Since npm-core and npm-components modify different files, rebase merges cleanly.

### Security

- OIDC tokens used correctly for crates.io authentication
- npm provenance enabled for npm publish operations
- `GITHUB_TOKEN` (not a PAT) used for git push, which correctly prevents workflow re-trigger by default
- No secrets exposed in logs; version strings from registries are safe for shell interpolation

### Documentation

The coding standards addition is well-structured with:
- Two version domains table (Product vs Distribution)
- Clear 6-step explanation of how CI auto-bumping works
- Developer guidance on what to do / not do
- CI behavior documentation
- Infinite loop prevention rationale
- Correct placement between existing Versioning and Comments sections

## Architectural Concerns

None. The implementation makes no architectural changes — it modifies only the CD workflow YAML and documentation. Domain boundaries, code structure, and crate dependencies are unaffected.

## Handoff Decision

**Target Agent**: architect  
**Reasoning**: QA review passes with no Critical/High/Medium issues. The 4 Low findings are non-blocking. Implementation is complete, tested (12/12 PASS), and compliant with the LLD. Ready for architectural documentation review and subsequent PO handoff for roadmap update and archival.
