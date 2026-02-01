# Implementation Plan: M5 Dead Code Cleanup

## Overview

Remove stale `#[allow(dead_code)]` suppressions and genuinely dead code from the editor modules. This cleanup reduces technical debt from when the React UI was behind a feature flag.

**Target:** 14 → 3 suppressions (79% reduction)  
**Version:** 0.2.2 (patch bump)  
**Risk:** Low — removing suppressions and unused code only

---

## Requirements

- Remove suppressions on code that IS being used (stale comments)
- Delete genuinely dead code (`EditorMessage` enum, `message_tx` field)
- Move debug-only utilities to test modules
- Update comments on legitimate suppressions to explain WHY they're kept
- All builds and tests must pass

---

## Architecture Changes

| File | Change Type |
|------|-------------|
| `plugin/src/editor/webview.rs` | Remove 3 suppressions, update 2 comments |
| `plugin/src/editor/assets.rs` | Remove 3 suppressions |
| `plugin/src/editor/mod.rs` | Delete `EditorMessage` enum, delete `message_tx` field |
| `plugin/src/editor/bridge.rs` | Remove 2 suppressions |
| `plugin/src/editor/windows.rs` | Update 1 comment |
| `desktop/src/assets.rs` | Refactor `list_assets` to test module |
| `engine/Cargo.toml` | Bump version 0.2.1 → 0.2.2 |

---

## Implementation Steps

### Phase 1: Remove Stale Suppressions in `webview.rs`

#### Step 1.1: Remove suppression from `WebViewConfig`
**File:** `engine/crates/plugin/src/editor/webview.rs`

**Action:** Delete line `#[allow(dead_code)] // Configuration struct for platform implementations`

**Why:** `WebViewConfig` is actively used by `create_webview()` and `create_macos_webview()`

**Risk:** Low

---

#### Step 1.2: Remove suppression from `create_ipc_handler`
**File:** `engine/crates/plugin/src/editor/webview.rs`

**Action:** Delete line `#[allow(dead_code)] // Will be used when WebView editor is re-enabled`

**Why:** Function is called from `macos.rs:87`

**Risk:** Low

---

#### Step 1.3: Remove suppression from `IPC_PRIMITIVES_JS`
**File:** `engine/crates/plugin/src/editor/webview.rs`

**Action:** Delete line `#[allow(dead_code)] // Used conditionally per platform`

**Why:** Constant is used in `macos.rs:173`

**Risk:** Low

---

### Phase 2: Remove Stale Suppressions in `assets.rs`

#### Step 2.1: Remove suppression from `UI_ASSETS`
**File:** `engine/crates/plugin/src/editor/assets.rs`

**Action:** Delete `#[allow(dead_code)] // Part of asset serving API, will be used when editor is re-enabled`

**Why:** Used by `get_asset()` which is called by the URL scheme handler

**Risk:** Low

---

#### Step 2.2: Remove suppression from `get_asset`
**File:** `engine/crates/plugin/src/editor/assets.rs`

**Action:** Delete `#[allow(dead_code)] // Part of asset serving API, will be used when editor is re-enabled`

**Why:** Function is called by macOS URL scheme handler

**Risk:** Low

---

#### Step 2.3: Remove suppression from `mime_type_from_path`
**File:** `engine/crates/plugin/src/editor/assets.rs`

**Action:** Delete `#[allow(dead_code)] // Helper for get_asset, will be used when editor is re-enabled`

**Why:** Called by `get_asset()` which is in use

**Risk:** Low

---

### Phase 3: Remove Stale Suppressions in `bridge.rs`

#### Step 3.1: Remove suppression from `PluginEditorBridge` struct
**File:** `engine/crates/plugin/src/editor/bridge.rs`

**Action:** Delete `#[allow(dead_code)] // Will be used when WebView editor is re-enabled`

**Why:** Struct is instantiated in `create_ipc_handler()`

**Risk:** Low

---

#### Step 3.2: Remove suppression from `PluginEditorBridge::new`
**File:** `engine/crates/plugin/src/editor/bridge.rs`

**Action:** Delete `#[allow(dead_code)] // Will be used when WebView editor is re-enabled`

**Why:** Constructor is called from `create_ipc_handler()`

**Risk:** Low

---

### Phase 4: Delete Dead Code in `mod.rs`

#### Step 4.1: Delete `EditorMessage` enum
**File:** `engine/crates/plugin/src/editor/mod.rs`

**Action:** Delete the entire `EditorMessage` enum (lines ~27-31)

```rust
// DELETE THIS:
#[allow(dead_code)] // Variants defined for future IPC use
#[derive(Debug, Clone)]
pub enum EditorMessage {
    ParamUpdate { id: String, value: f32 },
    ParamModulation { id: String, offset: f32 },
}
```

**Why:** Never used anywhere — speculative "future" code

**Risk:** Low — grep confirms no usages

---

#### Step 4.2: Delete `message_tx` field from `VstKitEditor`
**File:** `engine/crates/plugin/src/editor/mod.rs`

**Action:** 
1. Delete the field declaration:
   ```rust
   #[allow(dead_code)] // Kept for future IPC enhancement
   message_tx: Arc<Mutex<Option<std::sync::mpsc::Sender<EditorMessage>>>>,
   ```
2. Delete its initialization in `VstKitEditor::new()`:
   ```rust
   message_tx: Arc::new(Mutex::new(None)),
   ```

**Why:** Stores sender for deleted `EditorMessage` — dead code

**Dependencies:** Must be done after Step 4.1

**Risk:** Low

---

#### Step 4.3: Clean up unused import
**File:** `engine/crates/plugin/src/editor/mod.rs`

**Action:** Check if `std::sync::mpsc::Sender` import is still needed. If not, remove it.

**Why:** `EditorMessage` deletion may leave orphaned import

**Risk:** Low — compiler will tell us

---

### Phase 5: Refactor Debug Utility in `desktop/src/assets.rs`

#### Step 5.1: Move `list_assets` to test module
**File:** `engine/crates/desktop/src/assets.rs`

**Action:** 
1. Remove `#[allow(dead_code)]` from `list_assets`
2. Remove `pub` from `list_assets` 
3. Move `list_assets()` and `collect_paths()` inside `#[cfg(test)] mod tests`

**Before:**
```rust
#[allow(dead_code)]
pub fn list_assets() -> Vec<String> { ... }

fn collect_paths(dir: &Dir, prefix: &str, paths: &mut Vec<String>) { ... }

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_list_assets() {
        let assets = list_assets();
        ...
    }
}
```

**After:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn list_assets() -> Vec<String> { ... }
    
    fn collect_paths(dir: &Dir, prefix: &str, paths: &mut Vec<String>) { ... }
    
    #[test]
    fn test_list_assets() {
        let assets = list_assets();
        ...
    }
}
```

**Why:** Only used by tests — make it test-only code

**Risk:** Low

---

### Phase 6: Update Legitimate Suppressions

#### Step 6.1: Update `resize` trait method comment
**File:** `engine/crates/plugin/src/editor/webview.rs`

**Action:** Replace comment:
```rust
// Before:
#[allow(dead_code)] // Platform trait completeness
fn resize(&self, width: u32, height: u32);

// After:
/// Resize the WebView to the given dimensions.
/// 
/// Note: Called by platform implementations, not from trait consumers.
/// The allow(dead_code) suppresses false positive from Rust's analysis.
#[allow(dead_code)]
fn resize(&self, width: u32, height: u32);
```

**Why:** Explain WHY the suppression is legitimate

**Risk:** None

---

#### Step 6.2: Update `close` trait method comment
**File:** `engine/crates/plugin/src/editor/webview.rs`

**Action:** Same pattern as Step 6.1

**Risk:** None

---

#### Step 6.3: Update `hwnd` field comment
**File:** `engine/crates/plugin/src/editor/windows.rs`

**Action:** Replace comment:
```rust
// Before:
#[allow(dead_code)] // Stored for future window operations (resize, focus management)
hwnd: HWND,

// After:
/// Parent window handle, retained for potential focus/positioning operations.
/// Note: Currently unused but required for future Windows platform features.
#[allow(dead_code)]
hwnd: HWND,
```

**Why:** Clearer explanation of why it's kept

**Risk:** None

---

### Phase 7: Version Bump

#### Step 7.1: Bump version to 0.2.2
**File:** `engine/Cargo.toml`

**Action:** Change `version = "0.2.1"` to `version = "0.2.2"`

**Why:** Patch release for housekeeping cleanup

**Risk:** None

---

### Phase 8: Verification

#### Step 8.1: Run formatter
```bash
cd engine && cargo fmt
```

---

#### Step 8.2: Run clippy
```bash
cd engine && cargo clippy --workspace -- -D warnings
```

**Expected:** No warnings, no errors

---

#### Step 8.3: Run tests
```bash
cargo xtask test
```

**Expected:** All tests pass (UI + Engine)

---

#### Step 8.4: Build plugin
```bash
cargo xtask bundle
```

**Expected:** VST3 and CLAP bundles created successfully

---

#### Step 8.5: Manual test (Tester phase)
- Load plugin in Ableton Live
- Verify UI renders correctly
- Verify parameter automation works
- Verify meters display correctly

---

## Testing Strategy

| Test Type | Scope | Expected Result |
|-----------|-------|-----------------|
| `cargo fmt --check` | All engine code | No formatting issues |
| `cargo clippy` | All engine code | No warnings |
| `cargo test --workspace` | Engine unit tests | All pass |
| `npm test` | UI unit tests | All pass |
| `cargo xtask bundle` | Build system | Bundles created |
| Manual: Ableton Live | Plugin functionality | Works as before |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Removing suppression reveals actual dead code | Low | Low | Run clippy after each phase; fix or re-add suppression |
| Deleting `EditorMessage` breaks something | Very Low | Medium | Grep confirms zero usages |
| Test failures after changes | Low | Low | Incremental changes, verify after each phase |

---

## Success Criteria

- [ ] `#[allow(dead_code)]` count reduced from 14 to 3
- [ ] No stale "will be used when re-enabled" comments remain
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `npm test` passes
- [ ] Plugin loads and works in Ableton Live
- [ ] Version bumped to 0.2.2

---

## Estimated Effort

| Phase | Effort | Notes |
|-------|--------|-------|
| Phase 1-3: Remove stale suppressions | 10 min | Mechanical deletions |
| Phase 4: Delete dead code | 10 min | Slightly more careful |
| Phase 5: Refactor debug utility | 10 min | Move code to test module |
| Phase 6: Update comments | 5 min | Documentation only |
| Phase 7-8: Version + verification | 15 min | Build + test |

**Total:** ~50 minutes

---

## Handoff

→ **Coder**: Implement according to this plan
