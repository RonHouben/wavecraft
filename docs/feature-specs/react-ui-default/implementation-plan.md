# Implementation Plan: Make React UI Default

> **Feature:** Remove the `webview_editor` feature flag and make React UI the only plugin editor implementation.

---

## Overview

This plan removes the `webview_editor` feature flag and all associated conditional compilation, making React UI the default (and only) plugin editor. This is a cleanup/simplification effort—no new functionality is added.

**Total Tasks:** 18 tasks across 5 phases  
**Estimated Complexity:** Low-Medium (straightforward removal, but touches many files)

---

## Requirements Summary

From [user-stories.md](./user-stories.md):
- US-1: React UI builds by default (no feature flags needed)
- US-2: Remove `webview_editor` feature flag from codebase
- US-3: Remove legacy egui fallback editor code
- US-4: Update documentation
- US-5: Update xtask build commands

---

## Architecture Changes

Based on [low-level-design-react-ui-default.md](./low-level-design-react-ui-default.md):

| Category | Files Affected |
|----------|----------------|
| Delete | `engine/crates/plugin/src/editor/egui.rs` |
| Cargo config | `engine/crates/plugin/Cargo.toml` |
| Plugin source | `lib.rs`, `editor/mod.rs`, `editor/assets.rs`, `editor/webview.rs` |
| Build system | `xtask/commands/bundle.rs`, `xtask/commands/release.rs` |
| CI/CD | `.github/workflows/ci.yml`, `.github/workflows/release.yml` |
| Documentation | `README.md`, `high-level-design.md`, `macos-signing.md` |

---

## Implementation Steps

### Phase 1: Remove Feature Flag Definition and Legacy Code

**Goal:** Remove the source of conditional compilation and delete unused code.

---

#### Task 1.1: Delete egui Fallback Editor

**File:** `engine/crates/plugin/src/editor/egui.rs`

- **Action:** Delete the entire file
- **Why:** This is the legacy fallback UI that is no longer needed
- **Dependencies:** None (start here)
- **Risk:** Low
- **Verification:** File no longer exists

---

#### Task 1.2: Remove `webview_editor` Feature from Cargo.toml

**File:** `engine/crates/plugin/Cargo.toml`

- **Action:** Remove the `webview_editor = []` line from `[features]` section
- **Why:** The feature flag definition must be removed
- **Dependencies:** Task 1.1 (egui code deleted first)
- **Risk:** Low
- **Verification:** `cargo check -p vstkit` fails if any code still references the feature

**Before:**
```toml
[features]
default = []
# Enable WebView-based editor (React UI)
webview_editor = []
# Enable runtime allocation detection on audio thread (debug builds)
assert_process_allocs = ["nih_plug/assert_process_allocs"]
```

**After:**
```toml
[features]
default = []
# Enable runtime allocation detection on audio thread (debug builds)
assert_process_allocs = ["nih_plug/assert_process_allocs"]
```

---

#### Task 1.3: Remove `nih_plug_egui` Dependency

**File:** `engine/crates/plugin/Cargo.toml`

- **Action:** Remove `nih_plug_egui.workspace = true` from `[dependencies]`
- **Why:** No longer needed with egui fallback deleted
- **Dependencies:** Task 1.1
- **Risk:** Low
- **Verification:** `cargo check -p vstkit` succeeds without egui dependency

---

### Phase 2: Remove Conditional Compilation from Plugin Source

**Goal:** Remove all `#[cfg(feature = "webview_editor")]` conditionals and dead_code annotations.

---

#### Task 2.1: Simplify Imports in lib.rs

**File:** `engine/crates/plugin/src/lib.rs`

- **Action:** Replace conditional imports with direct import of `create_webview_editor`
- **Why:** Only WebView editor exists now
- **Dependencies:** Phase 1 complete
- **Risk:** Low
- **Verification:** `cargo check -p vstkit` succeeds

**Before (lines 17-21):**
```rust
#[cfg(feature = "webview_editor")]
use crate::editor::create_webview_editor;

#[cfg(not(feature = "webview_editor"))]
use crate::editor::create_editor;
```

**After:**
```rust
use crate::editor::create_webview_editor;
```

---

#### Task 2.2: Simplify editor() Method in lib.rs

**File:** `engine/crates/plugin/src/lib.rs`

- **Action:** Replace conditional editor creation with direct call to `create_webview_editor`
- **Why:** Only one editor implementation now
- **Dependencies:** Task 2.1
- **Risk:** Low
- **Verification:** `cargo check -p vstkit` succeeds

**Before (lines ~71-82):**
```rust
fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    #[cfg(feature = "webview_editor")]
    {
        create_webview_editor(self.params.clone(), self.meter_consumer.clone())
    }

    #[cfg(not(feature = "webview_editor"))]
    {
        create_editor(self.params.clone(), self.meter_consumer.clone())
    }
}
```

**After:**
```rust
fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    create_webview_editor(self.params.clone(), self.meter_consumer.clone())
}
```

---

#### Task 2.3: Remove dead_code Annotation from meter_consumer

**File:** `engine/crates/plugin/src/lib.rs`

- **Action:** Remove `#[allow(dead_code)]` from `meter_consumer` field (lines ~32-35)
- **Why:** Field is now always used (not conditional)
- **Dependencies:** Task 2.2
- **Risk:** Low
- **Verification:** No warnings about unused field

---

#### Task 2.4: Remove mod egui and create_editor() from editor/mod.rs

**File:** `engine/crates/plugin/src/editor/mod.rs`

- **Action:** 
  1. Remove `mod egui;` declaration
  2. Remove `create_editor()` function
- **Why:** egui module deleted, fallback function no longer needed
- **Dependencies:** Phase 1 complete
- **Risk:** Low
- **Verification:** `cargo check -p vstkit` succeeds

---

#### Task 2.5: Remove dead_code Annotations from editor/mod.rs

**File:** `engine/crates/plugin/src/editor/mod.rs`

- **Action:** Remove all `#[allow(dead_code)]` annotations from `EditorMessage`, `VstKitEditor`, `create_webview_editor()`
- **Why:** These items are now always used
- **Dependencies:** Task 2.4
- **Risk:** Low
- **Verification:** `cargo check -p vstkit` succeeds with no dead_code warnings

---

#### Task 2.6: Remove dead_code Annotations from editor/assets.rs

**File:** `engine/crates/plugin/src/editor/assets.rs`

- **Action:** Remove `#[allow(dead_code)]` from `UI_ASSETS`, `get_asset()`, `mime_type_from_path()`
- **Why:** These items are now always used
- **Dependencies:** Phase 1 complete
- **Risk:** Low
- **Verification:** No warnings

---

#### Task 2.7: Remove dead_code Annotations from editor/webview.rs

**File:** `engine/crates/plugin/src/editor/webview.rs`

- **Action:** Remove `#[allow(dead_code)]` from `WebViewHandle` trait
- **Why:** Trait is now always used
- **Dependencies:** Phase 1 complete
- **Risk:** Low
- **Verification:** No warnings

---

### Phase 3: Update Build System

**Goal:** Make xtask commands build React UI unconditionally.

---

#### Task 3.1: Remove Feature Check from bundle.rs

**File:** `engine/xtask/src/commands/bundle.rs`

- **Action:** Remove the `if features.contains(&"webview_editor")` check; always build UI
- **Why:** React UI should always be built
- **Dependencies:** Phase 2 complete
- **Risk:** Medium (affects build pipeline)
- **Verification:** `cargo xtask bundle` builds React UI without flags

**Before (lines 27-56):**
```rust
// If webview_editor feature is enabled, build the UI assets first
if features.contains(&"webview_editor") {
    print_status("Building React UI assets...");
    // ... build logic ...
}
```

**After:**
```rust
// Build the React UI assets
print_status("Building React UI assets...");
// ... build logic (unconditional) ...
```

---

#### Task 3.2: Update release.rs to Not Pass Feature Flag

**File:** `engine/xtask/src/commands/release.rs`

- **Action:** Change `bundle::run_with_features(..., &["webview_editor"], ...)` to `bundle::run(...)`
- **Why:** Feature flag no longer exists
- **Dependencies:** Task 3.1
- **Risk:** Low
- **Verification:** `cargo xtask release` works (dry run)

---

### Phase 4: Update CI/CD Workflows

**Goal:** Remove feature flags from CI build commands.

---

#### Task 4.1: Update ci.yml Build Command

**File:** `.github/workflows/ci.yml`

- **Action:** Remove `--features webview_editor` from build command (line ~232)
- **Why:** Feature no longer exists
- **Dependencies:** Phase 3 complete
- **Risk:** Low
- **Verification:** CI pipeline passes (after push)

**Before:**
```yaml
run: cargo xtask bundle --release --features webview_editor
```

**After:**
```yaml
run: cargo xtask bundle --release
```

---

#### Task 4.2: Update release.yml Build Command

**File:** `.github/workflows/release.yml`

- **Action:** Remove `--features webview_editor` from build command (line ~51)
- **Why:** Feature no longer exists
- **Dependencies:** Phase 3 complete
- **Risk:** Low
- **Verification:** Release workflow syntax is valid

**Before:**
```yaml
run: |
  cd engine
  cargo xtask bundle --release --features webview_editor
```

**After:**
```yaml
run: |
  cd engine
  cargo xtask bundle --release
```

---

### Phase 5: Update Documentation

**Goal:** Remove all references to the `webview_editor` feature flag.

---

#### Task 5.1: Update README.md

**File:** `README.md`

- **Action:** 
  1. Remove feature flag documentation section
  2. Simplify build examples to not include `--features webview_editor`
  3. Update any references implying React UI is optional
- **Why:** Feature flag no longer exists
- **Dependencies:** Phase 4 complete (so we can verify builds work first)
- **Risk:** Low
- **Verification:** README reflects current build commands

---

#### Task 5.2: Update high-level-design.md

**File:** `docs/architecture/high-level-design.md`

- **Action:** Update build command examples to remove `--features webview_editor`
- **Why:** Commands should match reality
- **Dependencies:** None (can run in parallel with 5.1)
- **Risk:** Low
- **Verification:** Commands in doc work

---

#### Task 5.3: Update macos-signing.md

**File:** `docs/guides/macos-signing.md`

- **Action:** Update build command to remove `--features webview_editor`
- **Why:** Commands should match reality
- **Dependencies:** None (can run in parallel with 5.1)
- **Risk:** Low
- **Verification:** Commands in doc work

---

## Verification Checkpoints

After each phase, verify the build still works:

| Checkpoint | Command | Expected |
|------------|---------|----------|
| After Phase 1 | `cargo check -p vstkit` | ❌ Fails (references to feature) |
| After Phase 2 | `cargo check -p vstkit` | ✅ Succeeds |
| After Phase 3 | `cargo xtask bundle` | ✅ Builds with React UI |
| After Phase 4 | Push to branch | ✅ CI passes |
| After Phase 5 | Manual review | ✅ Docs accurate |

---

## Final Verification

| Test | Method | Expected |
|------|--------|----------|
| Default bundle | `cargo xtask bundle` | Plugin with React UI |
| Release bundle | `cargo xtask bundle --release` | Plugin with React UI |
| Plugin in DAW | Load in Ableton Live | React UI appears |
| Grep check | `grep -r "webview_editor" --include="*.rs" --include="*.toml" --include="*.yml" --include="*.md" .` | Only archived docs |
| Old flag rejected | `cargo check -p vstkit --features webview_editor` | Error: unknown feature |

---

## Testing Strategy

### Build Tests
- `cargo xtask bundle` succeeds
- `cargo xtask bundle --release` succeeds
- `cargo xtask desktop` succeeds (if applicable)

### Plugin Tests
- VST3 loads in Ableton Live
- CLAP loads in Bitwig Studio
- Parameter sync works
- Meters animate

### CI/CD Tests
- All CI checks pass on feature branch
- Release workflow syntax validates

---

## Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Developer muscle memory (`--features webview_editor`) | Low | Clear error message, updated docs |
| CI cache stale | Low | Cache keyed on Cargo.lock, self-correcting |
| Missed reference | Very Low | Grep verification, PR review |
| Build breaks | Low | Verification checkpoints after each phase |

---

## Success Criteria

From [user-stories.md](./user-stories.md):

- [ ] `cargo xtask bundle` produces plugin with React UI (no flags)
- [ ] All `#[cfg(feature = "webview_editor")]` conditionals removed
- [ ] `egui.rs` deleted
- [ ] No `webview_editor` in any Cargo.toml
- [ ] CI/CD passes without feature flags
- [ ] Documentation updated

---

## Related Documents

- [User Stories](./user-stories.md) — Feature requirements
- [Low-Level Design](./low-level-design-react-ui-default.md) — Technical design details
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
- [Roadmap](../../roadmap.md) — Project milestones
