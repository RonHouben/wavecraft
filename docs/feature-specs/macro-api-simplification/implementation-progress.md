# Implementation Progress: Macro API Simplification

**Feature:** Macro API Simplification  
**Target Version:** 0.9.0  
**Status:** In Progress (Phase 1 & 2 Complete)  
**Started:** 2026-02-08

---

## Overview

This document tracks the implementation progress of the Macro API Simplification feature. Each task corresponds to a step in the implementation plan.

---

## Phase 1: Core Macro Changes (HIGH PRIORITY) ✅

### Proc-Macro Simplification

- [x] **Task 1.1**: Simplify `PluginDef` struct
  - File: `engine/crates/wavecraft-macros/src/plugin.rs`
  - Remove `vendor`, `url`, `email` fields
  - Make `krate` field optional (default: `::wavecraft`)
  
- [x] **Task 1.2**: Update `Parse` implementation
  - File: `engine/crates/wavecraft-macros/src/plugin.rs`
  - Remove property parsing for vendor/url/email
  - Make `crate` property optional with `::wavecraft` default
  - Update error messages

- [x] **Task 1.3**: Implement metadata derivation
  - File: `engine/crates/wavecraft-macros/src/plugin.rs`
  - Derive vendor from `CARGO_PKG_AUTHORS`
  - Derive URL from `CARGO_PKG_HOMEPAGE` / `CARGO_PKG_REPOSITORY`
  - Parse email from authors field
  - Add fallback defaults

- [x] **Task 1.4**: Update VST3 ID generation
  - File: `engine/crates/wavecraft-macros/src/plugin.rs`
  - Change `generate_vst3_id()` to use package name instead of vendor
  - Update hash input: `format!("{}{}", package_name, name)`

- [x] **Task 1.5**: Update CLAP ID generation
  - File: `engine/crates/wavecraft-macros/src/plugin.rs`
  - Use package name: `format!("com.{}", package_name.replace('-', "_"))`

- [x] **Task 1.6**: Add signal validation
  - File: `engine/crates/wavecraft-macros/src/plugin.rs`
  - Detect bare processors (identifiers)
  - Emit helpful compile error guiding to `SignalChain![]`

- [x] **Task 1.7**: Update macro docstring
  - File: `engine/crates/wavecraft-macros/src/lib.rs`
  - Update examples to show minimal API
  - Show `SignalChain![]` usage
  - Document metadata derivation

---

## Phase 2: SignalChain Macro Rename (MEDIUM PRIORITY) ✅

### Declarative Macro Updates

- [x] **Task 2.1**: Create `SignalChain!` macro
  - File: `engine/crates/wavecraft-dsp/src/combinators/mod.rs`
  - Copy implementation from `Chain!`
  - Update docstrings with examples

- [x] **Task 2.2**: Deprecate `Chain!` macro
  - File: `engine/crates/wavecraft-dsp/src/combinators/mod.rs`
  - Add `#[deprecated]` attribute
  - Delegate to `SignalChain!`

- [x] **Task 2.3**: Export `SignalChain!` at crate root
  - File: `engine/crates/wavecraft-dsp/src/lib.rs`
  - Add to public exports

- [x] **Task 2.4**: Update core prelude
  - File: `engine/crates/wavecraft-core/src/prelude.rs`
  - Re-export `SignalChain`

- [x] **Task 2.5**: Update nih_plug prelude
  - File: `engine/crates/wavecraft-nih_plug/src/prelude.rs`
  - Verify `SignalChain` available via `wavecraft::prelude::*` (auto via core prelude)

---

## Phase 3: CLI Template Updates (MEDIUM PRIORITY)

### Template Modernization

- [ ] **Task 3.1**: Update template plugin code
  - File: `cli/sdk-templates/new-project/react/engine/src/lib.rs`
  - Use simplified `wavecraft_plugin!` API
  - Use `SignalChain![]` wrapper

- [ ] **Task 3.2**: Update template Cargo.toml
  - File: `cli/sdk-templates/new-project/react/engine/Cargo.toml.template`
  - Add `authors` field
  - Add `homepage` field

- [ ] **Task 3.3**: Update template variables
  - File: `cli/src/template/variables.rs`
  - Add `author_name`, `author_email`, `homepage_url` variables
  - Add prompts/defaults for new fields

---

## Phase 4: Documentation Updates (LOW PRIORITY)

### Public Documentation

- [ ] **Task 4.1**: Update High-Level Design
  - File: `docs/architecture/high-level-design.md`
  - Replace `Chain!` with `SignalChain!`
  - Update DSL examples

- [ ] **Task 4.2**: Update Coding Standards
  - File: `docs/architecture/coding-standards.md`
  - Document new macro API
  - Explain metadata derivation
  - Show `SignalChain!` usage patterns

- [ ] **Task 4.3**: Update README
  - File: `README.md`
  - Update quick start examples
  - Show minimal plugin definition

- [ ] **Task 4.4**: Create migration guide
  - File: `docs/MIGRATION-0.8.md`
  - Document breaking changes
  - Provide migration steps
  - Explain VST3 ID impact

---

## Phase 5: Testing (CONTINUOUS — HIGHEST PRIORITY)

### Test Coverage

- [ ] **Task 5.1**: Create macro expansion tests
  - File: `engine/crates/wavecraft-macros/tests/plugin_macro.rs` (new)
  - Test minimal plugin compiles
  - Test metadata derivation
  - Test bare processor error message
  - Test VST3/CLAP ID generation
  - Test `crate` property default

- [ ] **Task 5.2**: Update CLI template tests
  - File: `cli/tests/template_generation.rs`
  - Verify generated plugin uses `SignalChain!`
  - Verify no vendor/url/email in macro invocation
  - Verify Cargo.toml has metadata fields

- [ ] **Task 5.3**: Manual integration tests
  - Generate plugin: `wavecraft create test-macro-api-simplification --output target/tmp/test-plugin`
  - Build: `cargo xtask bundle`
  - Install: `cargo xtask install`
  - Load in Ableton Live
  - Verify plugin metadata displays correctly
  - Test multiple processors
  - Verify `Chain!` deprecation warning
  - Verify bare processor error message

- [ ] **Task 5.4**: CI validation
  - Run `cargo xtask ci-check`
  - All tests pass
  - No unexpected warnings
  - Linting passes

---

## Version Bump

- [ ] **Task 6.1**: Update version to 0.8.0
  - File: `engine/Cargo.toml`
  - Update `[workspace.package]` version
  - Verify UI displays new version (VersionBadge)

---

## Rollback Plan

If critical issues arise during implementation:

1. **Partial rollback**: Revert specific commits while keeping passing changes
2. **Full rollback**: `git revert` all commits in feature branch
3. **Version strategy**: If already merged to main, release 0.7.2 as hotfix branch

**Blocking issues that require rollback:**
- VST3 ID generation breaks plugin loading
- Cargo env vars unavailable in certain build environments
- Macro expansion errors in valid user code
- DAW crashes when loading plugins with new metadata

---

## Notes

### Decisions Made

- **Metadata source**: Using Cargo env vars (`CARGO_PKG_*`) instead of reading `Cargo.toml` directly
- **VST3 ID strategy**: Using package name instead of vendor (breaking change, acceptable for 0.8.0)
- **`Chain!` deprecation**: Soft deprecation with warning in 0.8.0, hard removal in 0.9.0
- **`crate` property**: Made optional (not removed) for power users

### Blockers

_(None currently)_

### Questions

_(None currently)_

---

## Progress Summary

**Total Tasks**: 27  
**Completed**: 0  
**In Progress**: 0  
**Blocked**: 0  
**Remaining**: 27

**Last Updated**: 2026-02-08
