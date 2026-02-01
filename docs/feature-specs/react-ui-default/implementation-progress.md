# Implementation Progress: Make React UI Default

> **Feature:** Remove the `webview_editor` feature flag and make React UI the only plugin editor implementation.

---

## Status: 🟡 Not Started

**Branch:** `feature/react-ui-default`  
**Started:** -  
**Last Updated:** -

---

## Task Checklist

### Phase 1: Remove Feature Flag Definition and Legacy Code

- [ ] **1.1** Delete `engine/crates/plugin/src/editor/egui.rs`
- [ ] **1.2** Remove `webview_editor` feature from `engine/crates/plugin/Cargo.toml`
- [ ] **1.3** Remove `nih_plug_egui` dependency from `engine/crates/plugin/Cargo.toml`

**Checkpoint:** `cargo check -p vstkit` should fail (code still references feature)

---

### Phase 2: Remove Conditional Compilation from Plugin Source

- [ ] **2.1** Simplify imports in `engine/crates/plugin/src/lib.rs`
- [ ] **2.2** Simplify `editor()` method in `engine/crates/plugin/src/lib.rs`
- [ ] **2.3** Remove `dead_code` annotation from `meter_consumer` field
- [ ] **2.4** Remove `mod egui` and `create_editor()` from `engine/crates/plugin/src/editor/mod.rs`
- [ ] **2.5** Remove `dead_code` annotations from `editor/mod.rs`
- [ ] **2.6** Remove `dead_code` annotations from `editor/assets.rs`
- [ ] **2.7** Remove `dead_code` annotations from `editor/webview.rs`

**Checkpoint:** `cargo check -p vstkit` should succeed

---

### Phase 3: Update Build System

- [ ] **3.1** Remove feature check from `engine/xtask/src/commands/bundle.rs`
- [ ] **3.2** Update `engine/xtask/src/commands/release.rs` to not pass feature flag

**Checkpoint:** `cargo xtask bundle` should build React UI without flags

---

### Phase 4: Update CI/CD Workflows

- [ ] **4.1** Update `.github/workflows/ci.yml` build command
- [ ] **4.2** Update `.github/workflows/release.yml` build command

**Checkpoint:** CI pipeline should pass after push

---

### Phase 5: Update Documentation

- [ ] **5.1** Update `README.md` - remove feature flag references
- [ ] **5.2** Update `docs/architecture/high-level-design.md` - remove feature flag from examples
- [ ] **5.3** Update `docs/guides/macos-signing.md` - remove feature flag from examples

**Checkpoint:** Documentation reflects current commands

---

## Final Verification

- [ ] `cargo xtask bundle` produces plugin with React UI
- [ ] `cargo xtask bundle --release` produces plugin with React UI
- [ ] Plugin loads in Ableton Live with React UI
- [ ] `grep -r "webview_editor"` returns only archived docs
- [ ] All CI checks pass

---

## Notes

_Add implementation notes, issues encountered, and decisions made here._

---

## Handoff

When complete:
1. Mark all tasks as done
2. Update status to ✅ Complete
3. Hand off to **Tester** for manual verification
