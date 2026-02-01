# Low-Level Design: Make React UI Default

> **Feature:** Remove the `webview_editor` feature flag and make React UI the only plugin editor implementation.

---

## 1. Executive Summary

This is a **removal and simplification** effort. The `webview_editor` feature flag served as scaffolding during development to allow building the plugin without the React UI. Now that the React UI has been validated in production (Ableton Live on macOS), the feature flag is technical debt that adds conditional compilation complexity.

**Scope:**
- Remove the `webview_editor` feature flag from Cargo configuration
- Remove all `#[cfg(feature = "webview_editor")]` conditionals
- Remove the legacy egui fallback editor
- Update build tooling to always build React UI
- Update documentation to remove feature flag references

**Out of Scope:**
- Removing `nih_plug_egui` from workspace dependencies (may be needed for future alternative UIs)
- Modifying the WebView/React UI implementation itself
- Windows/Linux support changes

---

## 2. Current State Analysis

### 2.1 Feature Flag Definition

**File:** [engine/crates/plugin/Cargo.toml](../../../engine/crates/plugin/Cargo.toml)

```toml
[features]
default = []
# Enable WebView-based editor (React UI)
webview_editor = []
```

The feature is empty (no dependencies)—it's purely a compile-time switch.

### 2.2 Conditional Compilation Sites

**File:** [engine/crates/plugin/src/lib.rs](../../../engine/crates/plugin/src/lib.rs)

| Line | Condition | Purpose |
|------|-----------|---------|
| 17-18 | `#[cfg(feature = "webview_editor")]` | Import `create_webview_editor` |
| 20-21 | `#[cfg(not(feature = "webview_editor"))]` | Import `create_editor` (egui fallback) |
| 74-76 | `#[cfg(feature = "webview_editor")]` | Use WebView editor in `editor()` method |
| 79-81 | `#[cfg(not(feature = "webview_editor"))]` | Use egui editor in `editor()` method |

### 2.3 Legacy egui Editor

**File:** [engine/crates/plugin/src/editor/egui.rs](../../../engine/crates/plugin/src/editor/egui.rs)

This is a placeholder/fallback UI that:
- Uses `nih_plug_egui` for rendering
- Provides basic gain slider
- Was intended as temporary during WebView development
- Has `#[allow(dead_code)]` annotations indicating it's only used conditionally

**File:** [engine/crates/plugin/src/editor/mod.rs](../../../engine/crates/plugin/src/editor/mod.rs)

Lines 181-186 define `create_editor()` which wraps the egui editor:

```rust
pub fn create_editor(
    params: Arc<VstKitParams>,
    _meter_consumer: Arc<Mutex<MeterConsumer>>,
) -> Option<Box<dyn Editor>> {
    egui::create_egui_editor(params)
}
```

### 2.4 `#[allow(dead_code)]` Annotations

Several modules have dead code annotations because they're only used when `webview_editor` is enabled:

| File | Items |
|------|-------|
| [editor/assets.rs](../../../engine/crates/plugin/src/editor/assets.rs) | `UI_ASSETS`, `get_asset()`, `mime_type_from_path()` |
| [editor/webview.rs](../../../engine/crates/plugin/src/editor/webview.rs) | `WebViewHandle` trait |
| [editor/mod.rs](../../../engine/crates/plugin/src/editor/mod.rs) | `EditorMessage`, `VstKitEditor`, `create_webview_editor()` |

### 2.5 Build System References

**File:** [engine/xtask/src/commands/bundle.rs](../../../engine/xtask/src/commands/bundle.rs)

Lines 27-58: Conditional UI build logic:
```rust
// If webview_editor feature is enabled, build the UI assets first
if features.contains(&"webview_editor") {
    print_status("Building React UI assets...");
    // ... npm run build ...
}
```

**File:** [engine/xtask/src/commands/release.rs](../../../engine/xtask/src/commands/release.rs)

Line 15: Hardcoded feature in release workflow:
```rust
bundle::run_with_features(BuildMode::Release, None, &["webview_editor"], verbose)?;
```

### 2.6 CI/CD References

**File:** [.github/workflows/ci.yml](../../../.github/workflows/ci.yml)

Line 232: Build command with feature flag:
```yaml
- name: Build plugin bundles
  run: cargo xtask bundle --release --features webview_editor
```

**File:** [.github/workflows/release.yml](../../../.github/workflows/release.yml)

Line 51: Release build command:
```yaml
run: |
  cd engine
  cargo xtask bundle --release --features webview_editor
```

### 2.7 Documentation References

| Document | Lines | Content |
|----------|-------|---------|
| [README.md](../../../README.md) | 99, 118-129 | Feature flag usage examples |
| [high-level-design.md](../../architecture/high-level-design.md) | 228, 231, 234, 247, 312 | Build commands with feature flag |
| [macos-signing.md](../../guides/macos-signing.md) | 94 | Build command with feature flag |

### 2.8 Workspace Dependencies

**File:** [engine/Cargo.toml](../../../engine/Cargo.toml)

Line 14: `nih_plug_egui` workspace dependency (used by egui fallback)

**File:** [engine/crates/plugin/Cargo.toml](../../../engine/crates/plugin/Cargo.toml)

Line 26: `nih_plug_egui.workspace = true`

---

## 3. Design Decisions

### 3.1 Remove vs Deprecate

**Decision:** **Remove immediately.** No deprecation period.

**Rationale:**
- The feature flag is internal tooling, not a public API
- The egui fallback was always a placeholder
- No external consumers depend on it
- Keeping dead code paths increases maintenance burden

### 3.2 nih_plug_egui Dependency

**Decision:** **Keep in workspace, remove from plugin crate.**

**Rationale:**
- The egui code (`editor/egui.rs`) will be deleted
- The plugin crate no longer needs `nih_plug_egui`
- However, keeping it in workspace dependencies doesn't hurt and may be useful for future alternative UI backends
- If we want to be thorough, we can remove it from workspace too, but it's not required

### 3.3 Bundle Command Default Behavior

**Decision:** **Always build React UI.** Remove the conditional check.

The current behavior is:
- Without `webview_editor`: Skip UI build, produce plugin with egui editor
- With `webview_editor`: Build React UI, embed assets, produce plugin with WebView editor

The new behavior is:
- Always build React UI, embed assets, produce plugin with WebView editor

---

## 4. Implementation Plan

### 4.1 Files to Modify

#### Engine Crate Configuration

| File | Change |
|------|--------|
| `engine/crates/plugin/Cargo.toml` | Remove `webview_editor` feature, remove `nih_plug_egui` dependency |

#### Plugin Source Code

| File | Change |
|------|--------|
| `engine/crates/plugin/src/lib.rs` | Remove cfg conditionals, use WebView editor unconditionally |
| `engine/crates/plugin/src/editor/mod.rs` | Remove `create_editor()` function, remove `mod egui`, remove dead_code annotations |
| `engine/crates/plugin/src/editor/assets.rs` | Remove dead_code annotations |
| `engine/crates/plugin/src/editor/webview.rs` | Remove dead_code annotations |

#### Build System

| File | Change |
|------|--------|
| `engine/xtask/src/commands/bundle.rs` | Remove feature check, always build UI |
| `engine/xtask/src/commands/release.rs` | Remove explicit feature flag (no longer needed) |

#### CI/CD

| File | Change |
|------|--------|
| `.github/workflows/ci.yml` | Remove `--features webview_editor` from build command |
| `.github/workflows/release.yml` | Remove `--features webview_editor` from build command |

#### Documentation

| File | Change |
|------|--------|
| `README.md` | Remove feature flag documentation, simplify build examples |
| `docs/architecture/high-level-design.md` | Update build commands to remove feature flags |
| `docs/guides/macos-signing.md` | Update build commands to remove feature flags |

### 4.2 Files to Delete

| File | Reason |
|------|--------|
| `engine/crates/plugin/src/editor/egui.rs` | Legacy fallback UI, no longer needed |

### 4.3 Code Changes Detail

#### 4.3.1 `engine/crates/plugin/Cargo.toml`

**Before:**
```toml
[features]
default = []
# Enable WebView-based editor (React UI)
webview_editor = []
# Enable runtime allocation detection on audio thread (debug builds)
assert_process_allocs = ["nih_plug/assert_process_allocs"]

[dependencies]
...
nih_plug_egui.workspace = true
```

**After:**
```toml
[features]
default = []
# Enable runtime allocation detection on audio thread (debug builds)
assert_process_allocs = ["nih_plug/assert_process_allocs"]

[dependencies]
...
# nih_plug_egui removed (egui fallback editor deleted)
```

#### 4.3.2 `engine/crates/plugin/src/lib.rs`

**Before:**
```rust
#[cfg(feature = "webview_editor")]
use crate::editor::create_webview_editor;

#[cfg(not(feature = "webview_editor"))]
use crate::editor::create_editor;

// ... in editor() method:
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
use crate::editor::create_webview_editor;

// ... in editor() method:
fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    create_webview_editor(self.params.clone(), self.meter_consumer.clone())
}
```

#### 4.3.3 `engine/crates/plugin/src/editor/mod.rs`

**Remove:**
- `mod egui;` declaration
- `create_editor()` function (lines 181-186)
- All `#[allow(dead_code)]` annotations related to webview_editor

#### 4.3.4 `engine/xtask/src/commands/bundle.rs`

**Before:**
```rust
// If webview_editor feature is enabled, build the UI assets first
if features.contains(&"webview_editor") {
    print_status("Building React UI assets...");
    // ... build UI ...
}
```

**After:**
```rust
// Always build UI assets for the React editor
print_status("Building React UI assets...");
// ... build UI ...
```

Also remove comments referencing the feature flag.

#### 4.3.5 `engine/xtask/src/commands/release.rs`

**Before:**
```rust
bundle::run_with_features(BuildMode::Release, None, &["webview_editor"], verbose)?;
```

**After:**
```rust
bundle::run(BuildMode::Release, None, verbose)?;
```

---

## 5. Testing Strategy

### 5.1 Build Verification

| Test | Command | Expected Result |
|------|---------|-----------------|
| Default bundle | `cargo xtask bundle` | Produces plugin with React UI |
| Release bundle | `cargo xtask bundle --release` | Produces plugin with React UI |
| Debug bundle | `cargo xtask bundle --debug` | Produces plugin with React UI |
| Desktop POC | `cargo xtask desktop` | Builds and runs with React UI |

### 5.2 Plugin Verification

| Test | Method | Expected Result |
|------|--------|-----------------|
| VST3 loads in Ableton | Load `vstkit.vst3` in Ableton Live | Plugin loads, React UI appears |
| CLAP loads in Bitwig | Load `vstkit.clap` in Bitwig Studio | Plugin loads, React UI appears |
| Parameter sync | Move gain slider in UI | DAW automation lane follows |
| Automation playback | Record automation, play back | UI follows automation |
| Meters work | Play audio through plugin | Meters animate correctly |

### 5.3 CI/CD Verification

| Test | Method | Expected Result |
|------|--------|-----------------|
| CI builds pass | Push to feature branch | All CI checks green |
| Release workflow | Tag and push | Release artifacts produced |

### 5.4 No Feature Flag Errors

| Test | Command | Expected Result |
|------|---------|-----------------|
| Old flag rejected | `cargo xtask bundle -f webview_editor` | Error: unknown feature |
| Cargo check | `cargo check -p vstkit --features webview_editor` | Error: unknown feature |

---

## 6. Risks and Mitigation

### 6.1 Risk: Breaking Existing Developer Workflows

**Risk:** Developers with muscle memory for `--features webview_editor` will get errors.

**Mitigation:**
- Clear error message if unknown feature is passed
- Updated documentation with new (simpler) commands
- Announcement in commit message and PR description

**Severity:** Low (easy to adapt)

### 6.2 Risk: Build Fails Without `ui/dist`

**Risk:** If `ui/dist` doesn't exist and someone runs a raw cargo build (not via xtask), compilation will fail.

**Mitigation:**
- This is existing behavior (unchanged)
- The `include_dir!` macro requires the directory to exist
- xtask always builds UI first, so normal workflows work
- Documentation clarifies that `cargo xtask bundle` is the correct entry point

**Severity:** Low (existing behavior, well-documented)

### 6.3 Risk: CI Caching Issues

**Risk:** CI cache may have stale artifacts compiled with the old feature flag.

**Mitigation:**
- Rust-cache is keyed by `Cargo.lock`, which will change when `Cargo.toml` changes
- If issues occur, cache invalidation via workflow change

**Severity:** Low (self-correcting)

### 6.4 Risk: Missed Reference to Feature Flag

**Risk:** A reference to `webview_editor` in documentation or comments is missed.

**Mitigation:**
- Comprehensive grep search for `webview_editor` and `webview` terms
- PR review checklist
- Post-merge grep verification

**Severity:** Very Low (cosmetic only)

---

## 7. Migration Checklist

### Pre-Implementation
- [ ] Verify React UI works in current main branch
- [ ] Create feature branch (already done: `feature/react-ui-default`)

### Implementation
- [ ] Delete `engine/crates/plugin/src/editor/egui.rs`
- [ ] Update `engine/crates/plugin/Cargo.toml`
- [ ] Update `engine/crates/plugin/src/lib.rs`
- [ ] Update `engine/crates/plugin/src/editor/mod.rs`
- [ ] Update `engine/crates/plugin/src/editor/assets.rs`
- [ ] Update `engine/crates/plugin/src/editor/webview.rs`
- [ ] Update `engine/xtask/src/commands/bundle.rs`
- [ ] Update `engine/xtask/src/commands/release.rs`
- [ ] Update `.github/workflows/ci.yml`
- [ ] Update `.github/workflows/release.yml`
- [ ] Update `README.md`
- [ ] Update `docs/architecture/high-level-design.md`
- [ ] Update `docs/guides/macos-signing.md`

### Verification
- [ ] `cargo xtask bundle` succeeds (no flags)
- [ ] `cargo xtask bundle --release` succeeds
- [ ] Plugin loads in Ableton Live
- [ ] React UI appears and functions
- [ ] All CI checks pass
- [ ] Grep for `webview_editor` returns only archived docs

### Post-Implementation
- [ ] Merge to main
- [ ] Update roadmap (PO responsibility)
- [ ] Archive feature spec folder (PO responsibility)

---

## 8. Related Documents

- [User Stories](./user-stories.md) — Feature requirements
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
- [Roadmap](../../roadmap.md) — Project milestones
