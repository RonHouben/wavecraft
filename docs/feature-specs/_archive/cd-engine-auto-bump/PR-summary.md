# PR Summary — CD Engine Auto-Bump

## Summary

This PR implements automatic patch version bumping for engine crates in the CD pipeline, eliminating the manual versioning requirement that was causing deployment failures. The solution includes:

1. **Fixed change detection** to include all engine paths
2. **Auto-bump workflow** that queries crates.io and bumps versions when needed
3. **Registry validation guardrail** that verifies engine crate availability before CLI publish
4. **CLI compatibility documentation** showing patch bumps work automatically via semver

All 4 phases from the implementation plan are complete, with one adjustment discovered during testing (removed conflicting `--no-git-commit` flag).

## Changes

### Build/CI Pipeline (`.github/workflows/continuous-deploy.yml`)

- **Fixed `detect-changes` job**: Added missing paths:
  - `engine/crates/wavecraft-dev-server/**`
  - `engine/Cargo.toml`
- **Rewrote `publish-engine` job** with auto-bump flow:
  - Added Node.js setup for `npx semver`
  - Added version determination step (queries crates.io sparse index)
  - Added auto-bump logic using `cargo ws version custom`
  - Removed all `--from-git` flags from publish steps
  - Added `outputs.version` for downstream jobs
- **Updated `publish-cli` job**: Added `--check-registry` flag to validation step

### Engine — xtask (`engine/xtask/`)

- **Dependencies added** (`Cargo.toml`):
  - `ureq = "2"` — HTTP client for crates.io queries
  - `semver = "1"` — Version compatibility checks
- **CLI flag added** (`main.rs`): `--check-registry` option for `validate-cli-deps`
- **Registry validation** (`validate_cli_deps.rs`):
  - `crate_index_prefix()` — Generates crates.io sparse index URL paths
  - `check_registry_availability()` — Queries crates.io and validates semver compatibility
  - `ValidateCliDepsConfig.check_registry` — New config field
  - Integration into `run()` function with verbose output
- **Unit tests added**:
  - `test_crate_index_prefix` — URL path generation (1-4+ char names)
  - `test_check_registry_config_default` — Config defaults
  - `test_check_registry_availability_real_crate` — Integration test (ignored)

### Documentation

- **Implementation Progress** (`docs/feature-specs/cd-engine-auto-bump/implementation-progress.md`):
  - Complete phase-by-phase documentation
  - Key implementation decisions and findings
  - Success criteria checklist

## Architecture Decisions

### Version Drift Is Intentional

The source version on `main` stays at a baseline (e.g., `0.11.0`) while published versions increment (`0.11.1`, `0.11.2`, ...). This matches the existing behavior for CLI and NPM packages.

**Why:** Reduces merge conflicts and keeps development simple. The CD pipeline owns version bumping, not developers.

### CLI Compatibility Works Automatically

Cargo interprets `version = "0.11.0"` as `^0.11.0` (caret requirement), which allows any patch bump without CLI changes. Only minor/major engine bumps require manual CLI dependency updates.

### Removed `--no-git-commit` Flag

Discovered during Phase 2.0 verification that `cargo ws version` cannot use both `--no-git-commit` and `--no-git-push` together (they conflict). The tool now creates the commit automatically with its default message.

### Registry Check Is Opt-In

The `--check-registry` flag is only used in CI, not during local development, to avoid unnecessary network calls.

## Commits

```
2abffa8 feat: implement CD engine auto-bump with registry validation
```

## Related Documentation

- [Low-Level Design](../cd-engine-auto-bump/low-level-design-cd-engine-auto-bump.md) — Technical design
- [Implementation Plan](../cd-engine-auto-bump/implementation-plan.md) — Step-by-step plan
- [Implementation Progress](../cd-engine-auto-bump/implementation-progress.md) — Completed work
- [Coding Standards — Rust](../../architecture/coding-standards-rust.md) — Rust conventions
- [Versioning and Distribution](../../architecture/versioning-and-distribution.md) — Version flow

## Testing

### ✅ Completed

- [x] All unit tests pass (`cargo test validate_cli_deps`)
- [x] New tests added for registry check functions
- [x] `cargo ws version` behavior verified locally
- [x] Code follows Rust coding standards

### 🔄 Manual Testing Required (Post-Merge)

1. **Change detection**: Push a change to only `wavecraft-dev-server` → Verify `detect-changes` outputs `engine=true`
2. **Auto-bump flow**: Trigger CD → Verify `publish-engine` bumps and publishes successfully
3. **CLI compatibility**: Verify `publish-cli` succeeds with new engine versions
4. **Registry validation**: Verify `--check-registry` catches unavailable crates

## Quality Checklist

- [x] Follows implementation plan
- [x] Code adheres to Rust coding standards
- [x] Unit tests added and passing
- [x] Documentation complete (implementation progress)
- [x] No breaking changes
- [x] Ready for CI pipeline testing

## Notes for Reviewers

- This PR **does not bump any versions** in source code — version bumping is handled by the CD pipeline at publish time
- The implementation progress doc has a detailed finding from Phase 2.0 about the `--no-git-commit` flag conflict
- Phase 3 is analysis-only (no code changes) — documented in implementation progress
- The registry check uses crates.io's sparse index API (no git clone overhead)
