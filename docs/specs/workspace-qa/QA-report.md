# QA Report: Workspace Full Scan

**Date**: 2026-01-31
**Reviewer**: QA Agent
**Status**: FAIL

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High | 3 |
| Medium | 0 |
| Low | 0 |

**Overall**: FAIL (3 High severity issues found)

## Automated Check Results

### cargo fmt --check
❌ **FAILED** - Multiple formatting issues in `xtask` crate:
- `xtask/src/commands/notarize.rs` - Line formatting issues
- `xtask/src/commands/release.rs` - Import ordering
- `xtask/src/commands/sign.rs` - Line formatting, import ordering
- `xtask/src/commands/test.rs` - Line formatting
- `xtask/src/main.rs` - Import ordering, line formatting

### cargo clippy --workspace -- -D warnings
❌ **FAILED** - 2 Clippy errors in `xtask` crate:

```
error: the borrowed expression implements the required traits
  --> xtask/src/commands/bundle.rs:36:19
   |
36 |             .args(&["run", "build"])
   |                   ^^^^^^^^^^^^^^^^^ help: change this to: `["run", "build"]`

error: explicit call to `.into_iter()` in function argument accepting `IntoIterator`
  --> xtask/src/commands/bundle.rs:95:66
   |
95 |     if let Err(e) = nih_plug_xtask::main_with_args(package_name, bundle_args.into_iter()) {
   |                                                                  ^^^^^^^^^^^------------
   |                                                                             |
   |                                                                             help: consider removing the `.into_iter()`
```

### cargo test --workspace
❌ **FAILED** - Build error in `desktop` crate:

```
error: proc macro panicked
  --> crates/desktop/src/assets.rs:13:25
   |
13 | static UI_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../../ui/dist");
   |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: message: "/Users/.../ui/dist" is not a directory
```

The `ui/dist` directory does not exist. This is expected during development when the UI hasn't been built, but prevents workspace-wide compilation.

### UI TypeScript Check
✅ **PASSED** - `npm run typecheck` completed successfully.

## Findings

| ID | Severity | Category | Description | Location | Recommendation |
|----|----------|----------|-------------|----------|----------------|
| 1 | High | Code Quality | Formatting violations in xtask crate - imports not sorted, lines need reformatting | `xtask/src/commands/*.rs`, `xtask/src/main.rs` | Run `cargo fmt` to auto-fix |
| 2 | High | Code Quality | Clippy lints - needless borrow and useless conversion | `xtask/src/commands/bundle.rs:36,95` | Remove `&` prefix from array literal; remove `.into_iter()` call |
| 3 | High | Build Failure | Missing `ui/dist` directory causes `desktop` crate to fail compilation | `engine/crates/desktop/src/assets.rs:13` | Build UI first with `cd ui && npm run build`, or make `include_dir!` conditional |

## Architectural Concerns

> ⚠️ **The following items require architect review before implementation.**

None identified - all issues are code quality fixes.

## Handoff Decision

**Target Agent**: coder
**Reasoning**: All findings are straightforward code quality issues:
1. Formatting can be auto-fixed with `cargo fmt`
2. Clippy warnings have explicit fix suggestions
3. The `ui/dist` issue is a build order dependency, not an architectural problem

The coder should:
1. Run `cargo fmt` in the `engine/` directory
2. Apply the two Clippy fixes to `xtask/src/commands/bundle.rs`
3. (Optional) Consider making the `desktop` crate build conditional on UI assets existing
