# User Stories: Make React UI Default

> **Feature:** Remove the `webview` feature flag and make React UI the only plugin editor implementation.

---

## Overview

The React UI has been validated in production (Ableton Live on macOS). The `webview` feature flag was scaffolding during development — it's time to remove it and establish React UI as the default (and only) editor implementation.

This is a **cleanup/simplification** effort, not new functionality.

---

## User Stories

### US-1: React UI Builds by Default

**As a** VstKit developer  
**I want** the plugin to build with React UI by default  
**So that** I don't need to pass special feature flags to get the production editor

#### Acceptance Criteria
- [ ] `cargo xtask bundle` produces a plugin with React UI without any feature flags
- [ ] No `--features webview` flag required in build commands
- [ ] CI pipeline builds React UI without special configuration

---

### US-2: Remove `webview` Feature Flag

**As a** VstKit maintainer  
**I want** the `webview` feature flag removed from the codebase  
**So that** there's only one code path and no conditional compilation complexity

#### Acceptance Criteria
- [ ] `webview` feature removed from `engine/crates/plugin/Cargo.toml`
- [ ] All `#[cfg(feature = "webview")]` conditionals removed
- [ ] All `#[cfg(not(feature = "webview"))]` code blocks evaluated and either kept unconditionally or removed
- [ ] No references to `webview` feature in any `Cargo.toml` files

---

### US-3: Remove Legacy Native UI Code

**As a** VstKit maintainer  
**I want** the old nih-plug native UI code removed  
**So that** the codebase is simpler and there's no dead code to maintain

#### Acceptance Criteria
- [ ] Old native nih-plug editor implementation removed (if it exists separately)
- [ ] Any placeholder/fallback UI code removed
- [ ] No orphaned modules or dead code paths related to non-React UI
- [ ] Code compiles and tests pass after removal

---

### US-4: Update Documentation

**As a** VstKit developer reading the docs  
**I want** documentation to reflect that React UI is the default  
**So that** I'm not confused by references to feature flags or alternative UIs

#### Acceptance Criteria
- [ ] README.md updated — no mention of `webview` feature flag for basic usage
- [ ] Build instructions simplified (no feature flags needed)
- [ ] High-level design doc updated if it references the feature flag
- [ ] Any xtask command help text updated

---

### US-5: Update xtask Build Commands

**As a** VstKit developer  
**I want** xtask commands to build React UI by default  
**So that** the build system reflects the new default

#### Acceptance Criteria
- [ ] `cargo xtask bundle` builds with React UI (no flags)
- [ ] `cargo xtask desktop` builds with React UI (no flags)
- [ ] Any `--features webview` arguments removed from xtask internals
- [ ] Help text doesn't mention webview feature

---

## Out of Scope

- Adding new UI features
- Changing UI behavior
- Windows/Linux WebView support (still not a priority)
- Performance optimization (separate milestone item)

---

## Technical Notes

### Investigation Needed

The Architect should investigate:
1. What code is conditionally compiled with `#[cfg(feature = "webview")]`?
2. Is there a separate native UI implementation, or just a placeholder?
3. What modules become dead code after removing the feature flag?
4. Are there any runtime checks for the feature that need removal?

### Expected Impact

- **Simpler `Cargo.toml`** — fewer features to manage
- **Cleaner code** — no conditional compilation for UI choice
- **Smaller binary** — if native UI code is removed (marginal)
- **Clearer architecture** — one path, one implementation

---

## Definition of Done

- [ ] All user stories' acceptance criteria met
- [ ] `cargo xtask bundle` produces working plugin with React UI (no flags)
- [ ] Plugin loads and works in Ableton Live (regression test)
- [ ] All unit tests pass
- [ ] CI pipeline passes
- [ ] No compiler warnings related to dead code
- [ ] Documentation updated

