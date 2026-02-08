# PR Summary: Extend `cargo xtask clean` to Comprehensively Clean Workspace

## Overview

Milestone 15 (Developer Tooling Polish) — Extended `cargo xtask clean` command to comprehensively clean the entire workspace with clear output and disk space reporting.

**Feature:** Comprehensive workspace cleanup  
**Type:** Enhancement  
**Milestone:** M15 (Developer Tooling Polish)  
**Version:** 0.8.6 (patch)

## Summary

This PR extends the `cargo xtask clean` command to clean all build artifacts across the entire Wavecraft workspace, not just the engine. The command now cleans 7 directories with clear output showing what was cleaned and how much disk space was reclaimed.

**User Value:**
- Single command to reclaim disk space across entire workspace
- Clear feedback showing exactly what was cleaned and space recovered
- Idempotent operation (no errors on missing directories)
- Significantly improved developer experience

**Key Improvements:**
- Cleans `cli/target/` (CLI build artifacts)
- Cleans `ui/dist/` (Vite build outputs)
- Cleans `ui/coverage/` (test coverage artifacts)
- Cleans `target/tmp/` (test plugin artifacts)
- Cleans `bundled/` (VST3/CLAP bundles)
- Cleans AU wrapper build directory (macOS)
- Shows size per directory + total disk space reclaimed

## Changes Made

### Engine / Build System

**`engine/xtask/src/commands/clean.rs`** (376 lines added)
- Added `CleanedItem` struct to track path and size
- Added `dir_size()` helper: Recursive size calculation for directories/files
- Added `format_size()` helper: Human-readable formatting (GB/MB/KB)
- Added `remove_dir()` helper: Idempotent removal with size tracking
- Extended `run()` function: Orchestrates cleanup of 7 directories
- Added 8 comprehensive unit tests: `test_format_size`, `test_dir_size_*`, `test_remove_dir_*`

**`engine/xtask/Cargo.toml`**
- Added `tempfile = "3"` dev-dependency for test fixtures

**Version Coordination** (0.8.6 across workspace)
- `engine/Cargo.toml` — Workspace version bump
- `engine/crates/wavecraft-bridge/Cargo.toml`
- `engine/crates/wavecraft-dev-server/Cargo.toml`
- `engine/crates/wavecraft-dsp/Cargo.toml`
- `engine/crates/wavecraft-macros/Cargo.toml`
- `engine/crates/wavecraft-metering/Cargo.toml`
- `engine/crates/wavecraft-protocol/Cargo.toml`

### Documentation

**`docs/architecture/high-level-design.md`**
- Updated clean command description with comprehensive cleanup details

**`docs/roadmap.md`**
- Added Milestone 15: Developer Tooling Polish
- Updated progress to 88% (15/17 milestones)
- Marked M15 as complete with full task breakdown
- Renumbered User Testing (M15→M16), V1.0 Release (M16→M17)
- Added changelog entry for M15 completion

**`docs/backlog.md`**
- Removed workspace cleanup task (promoted to roadmap as M15)

### Feature Specification (Archived)

**All files in `docs/feature-specs/_archive/workspace-cleanup/`:**
- `user-stories.md` — 4 user stories with acceptance criteria
- `implementation-progress.md` — Complete task tracking
- `test-plan.md` — 12 test cases (all passed)
- `QA-report.md` — 0 issues found, approved for release
- `architectural-review.md` — Full compliance analysis, approved for merge

## Testing

### Automated Tests
- ✅ **8 new unit tests** in `engine/xtask/src/commands/clean.rs`
  - `test_format_size` — Human-readable size formatting
  - `test_dir_size_empty_dir` — Empty directory handling
  - `test_dir_size_single_file` — Single file calculation
  - `test_dir_size_multiple_files` — Multiple file aggregation
  - `test_dir_size_nested_dirs` — Recursive directory traversal
  - `test_dir_size_nonexistent` — Returns 0 for missing paths
  - `test_remove_dir_success` — Tracks removed directory size
  - `test_remove_dir_nonexistent` — Returns None for missing dirs

- ✅ **106 engine tests passing** (98 existing + 8 new)
- ✅ **51 UI tests passing** (no changes to UI)
- ✅ **All linting checks passed** (cargo fmt, clippy, ESLint, Prettier)

### Manual Testing
- ✅ **12/12 test cases passed (100%)**
  - TC-001: Clean all directories when all exist
  - TC-002: Clean with some directories missing (idempotent)
  - TC-003: Clean with all directories missing (graceful)
  - TC-004: Verbose flag shows detailed output
  - TC-005: Help flag displays usage
  - TC-006: Integration with existing xtask (bundle → clean)
  - TC-007: Minimal output format validation
  - TC-008: Verbose output format validation
  - TC-009: Disk space reporting accuracy
  - TC-010: Error handling (permission denied)
  - TC-011: Clean after full build (realistic scenario)
  - TC-012: Multiple consecutive runs (idempotent)

### Quality Assurance
- ✅ **QA Review**: 0 Critical/High/Medium/Low issues (approved)
- ✅ **Architectural Review**: Fully compliant with all conventions (approved)

## Related Documentation

- [User Stories](./user-stories.md) — Requirements and acceptance criteria
- [Implementation Progress](./implementation-progress.md) — Task tracking
- [Test Plan](./test-plan.md) — Comprehensive test coverage
- [QA Report](./QA-report.md) — Quality review findings
- [Architectural Review](./architectural-review.md) — Architectural compliance analysis
- [High-Level Design](../../architecture/high-level-design.md) — Updated build system section
- [Roadmap](../../roadmap.md) — Milestone 15 completion

## Commits

```
5439281 PO: Add Milestone 15 (Developer Tooling Polish) - workspace cleanup
```

## Checklist

- [x] Code follows project coding standards
- [x] All tests passing (automated + manual: 100%)
- [x] Documentation updated (high-level-design.md, roadmap.md)
- [x] QA review completed and approved (0 issues)
- [x] Architectural review completed and approved (fully compliant)
- [x] Feature spec archived to `docs/feature-specs/_archive/workspace-cleanup/`
- [x] Version bumped appropriately (0.8.6, patch)
- [x] No breaking changes
- [x] Ready to merge to main

## Deployment Notes

This is a developer tooling enhancement with no impact on end users or production plugins. The change only affects the `cargo xtask clean` command used during development.

**Post-merge actions:**
- None required — this is a development-only enhancement

**Risk Assessment:** Low
- Changes isolated to xtask command
- No changes to production code (engine, UI, or CLI)
- Idempotent operation (safe to run multiple times)
- Comprehensive test coverage (8 unit tests + 12 manual tests)

## Performance Impact

**Build time:** No change (this is a cleanup command, not invoked during builds)  
**Developer productivity:** **Positive** — Single command now cleans entire workspace, saving time

**Typical space reclaimed:** 2-3 GB (verified in testing with full workspace build)
