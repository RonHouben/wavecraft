# Low-Level Design: M5 Dead Code Cleanup

## Overview

This document analyzes each `#[allow(dead_code)]` suppression in the codebase and provides recommendations for action. The goal is to reduce suppressions from 14 to ≤5, removing genuinely dead code while preserving legitimate platform-specific patterns.

---

## Analysis Summary

| Verdict | Count | Description |
|---------|-------|-------------|
| **REMOVE** | 10 | Code is dead or comments are stale |
| **KEEP** | 3 | Legitimate platform-specific patterns |
| **REFACTOR** | 1 | Can be restructured to avoid suppression |

**Target outcome:** 14 → 3 suppressions (79% reduction)

---

## Detailed Analysis

### File 1: `plugin/src/editor/webview.rs`

#### Suppression 1.1: `WebViewHandle::resize`
```rust
#[allow(dead_code)] // Platform trait completeness
fn resize(&self, width: u32, height: u32);
```

**Verdict: KEEP**

**Rationale:** This is a trait method that platform implementations must provide. Even though it's not called from the trait level, individual implementations (macOS, Windows) use it. The `resize()` method on `MacOSWebView` is called from the macOS module. This is legitimate Rust trait design.

**Action:** Update comment to be more precise:
```rust
/// Resize the WebView to the given dimensions.
/// Called by platform-specific code, not from trait consumers directly.
fn resize(&self, width: u32, height: u32);
```

---

#### Suppression 1.2: `WebViewHandle::close`
```rust
#[allow(dead_code)] // Platform trait completeness
fn close(&mut self);
```

**Verdict: KEEP**

**Rationale:** Same as above — trait method implemented by platforms, called during WebView cleanup. The macOS implementation has a working `close()` that removes the webview from superview.

**Action:** Same comment improvement as above.

---

#### Suppression 1.3: `WebViewConfig`
```rust
#[allow(dead_code)] // Configuration struct for platform implementations
pub struct WebViewConfig { ... }
```

**Verdict: REMOVE**

**Rationale:** `WebViewConfig` is actively used! It's passed to `create_webview()` and consumed by `create_macos_webview()`. The suppression is **stale** — this code is NOT dead.

**Action:** Remove the suppression entirely.

---

#### Suppression 1.4: `create_ipc_handler`
```rust
#[allow(dead_code)] // Will be used when WebView editor is re-enabled
pub fn create_ipc_handler(...) -> IpcHandler<PluginEditorBridge> { ... }
```

**Verdict: REMOVE**

**Rationale:** This function IS being used! It's called from `macos.rs:87`:
```rust
let handler = Arc::new(Mutex::new(super::webview::create_ipc_handler(...)));
```
The comment "when editor is re-enabled" is **stale** — the React UI editor IS the editor. The suppression is unnecessary.

**Action:** Remove the suppression and stale comment.

---

#### Suppression 1.5: `IPC_PRIMITIVES_JS`
```rust
#[allow(dead_code)] // Used conditionally per platform
pub const IPC_PRIMITIVES_JS: &str = include_str!("js/ipc-primitives-plugin.js");
```

**Verdict: REMOVE**

**Rationale:** This constant IS being used! It's referenced in `macos.rs:173`:
```rust
let ipc_primitives = NSString::from_str(super::webview::IPC_PRIMITIVES_JS);
```
The suppression is stale.

**Action:** Remove the suppression.

---

### File 2: `plugin/src/editor/assets.rs`

#### Suppression 2.1: `UI_ASSETS`
```rust
#[allow(dead_code)] // Part of asset serving API, will be used when editor is re-enabled
static UI_ASSETS: Dir = include_dir!("...");
```

**Verdict: REMOVE**

**Rationale:** `UI_ASSETS` is used by `get_asset()` which is called from `macos.rs` via the URL scheme handler. The comment is stale.

**Action:** Remove suppression and stale comment.

---

#### Suppression 2.2: `get_asset`
```rust
#[allow(dead_code)] // Part of asset serving API, will be used when editor is re-enabled
pub fn get_asset(path: &str) -> Option<...> { ... }
```

**Verdict: REMOVE**

**Rationale:** This function IS being used by the macOS URL scheme handler to serve assets. Comment is stale.

**Action:** Remove suppression and stale comment.

---

#### Suppression 2.3: `mime_type_from_path`
```rust
#[allow(dead_code)] // Helper for get_asset, will be used when editor is re-enabled
fn mime_type_from_path(path: &str) -> &'static str { ... }
```

**Verdict: REMOVE**

**Rationale:** Called by `get_asset()` which is in use. The helper is NOT dead.

**Action:** Remove suppression and stale comment.

---

### File 3: `plugin/src/editor/mod.rs`

#### Suppression 3.1: `EditorMessage` enum
```rust
#[allow(dead_code)] // Variants defined for future IPC use
#[derive(Debug, Clone)]
pub enum EditorMessage {
    ParamUpdate { id: String, value: f32 },
    ParamModulation { id: String, offset: f32 },
}
```

**Verdict: REMOVE (the entire enum)**

**Rationale:** This enum is defined but never used anywhere in the codebase. The "future IPC use" is speculative — we already have a working IPC system via the bridge. This is dead code.

**Action:** Delete the `EditorMessage` enum entirely.

---

#### Suppression 3.2: `message_tx` field
```rust
#[allow(dead_code)] // Kept for future IPC enhancement
message_tx: Arc<Mutex<Option<std::sync::mpsc::Sender<EditorMessage>>>>,
```

**Verdict: REMOVE**

**Rationale:** This field stores a sender for `EditorMessage`, which we're deleting. The "future IPC enhancement" is speculative and the existing IPC bridge already handles parameter updates. This is dead code.

**Action:** Delete the `message_tx` field from `VstKitEditor`.

---

### File 4: `plugin/src/editor/bridge.rs`

#### Suppression 4.1: `PluginEditorBridge` struct
```rust
#[allow(dead_code)] // Will be used when WebView editor is re-enabled
pub struct PluginEditorBridge { ... }
```

**Verdict: REMOVE**

**Rationale:** `PluginEditorBridge` IS being used! It's instantiated in `create_ipc_handler()` and used throughout the macOS WebView integration. Comment is stale.

**Action:** Remove suppression and stale comment.

---

#### Suppression 4.2: `PluginEditorBridge::new`
```rust
#[allow(dead_code)] // Will be used when WebView editor is re-enabled
pub fn new(...) -> Self { ... }
```

**Verdict: REMOVE**

**Rationale:** Same as above — the constructor IS being called. Comment is stale.

**Action:** Remove suppression and stale comment.

---

### File 5: `plugin/src/editor/windows.rs`

#### Suppression 5.1: `WindowsWebView::hwnd`
```rust
#[allow(dead_code)] // Stored for future window operations (resize, focus management)
hwnd: HWND,
```

**Verdict: KEEP**

**Rationale:** The `hwnd` field stores the Windows handle for the WebView parent window. While not currently used for resize (we use the controller's `SetBounds`), it may be needed for:
- Focus management
- Window positioning
- Platform-specific operations

This is **legitimate forward design** for Windows support that we may implement later.

**Action:** Keep suppression, update comment:
```rust
/// Parent window handle, retained for potential focus/positioning operations.
#[allow(dead_code)]
hwnd: HWND,
```

---

### File 6: `desktop/src/assets.rs`

#### Suppression 6.1: `list_assets`
```rust
#[allow(dead_code)]
pub fn list_assets() -> Vec<String> { ... }
```

**Verdict: REFACTOR → REMOVE**

**Rationale:** This is a debugging utility that's never called. It has a test that exercises it, but the function itself is unused in production.

**Options:**
1. **Delete it** — it's truly unused
2. **Move to `#[cfg(test)]`** — make it test-only
3. **Keep with suppression** — least desirable

**Action:** Move `list_assets()` and `collect_paths()` inside the `#[cfg(test)]` module. They're only used by tests.

---

## Implementation Plan

### Phase 1: Remove Stale Suppressions (Low Risk)

These are suppressions on code that IS being used — the comments are just wrong.

| File | Item | Action |
|------|------|--------|
| `webview.rs` | `WebViewConfig` | Remove `#[allow(dead_code)]` |
| `webview.rs` | `create_ipc_handler` | Remove suppression + stale comment |
| `webview.rs` | `IPC_PRIMITIVES_JS` | Remove suppression |
| `assets.rs` | `UI_ASSETS` | Remove suppression + stale comment |
| `assets.rs` | `get_asset` | Remove suppression + stale comment |
| `assets.rs` | `mime_type_from_path` | Remove suppression + stale comment |
| `bridge.rs` | `PluginEditorBridge` | Remove suppression + stale comment |
| `bridge.rs` | `PluginEditorBridge::new` | Remove suppression + stale comment |

**Verification:** `cargo clippy --workspace -- -D warnings` should pass.

---

### Phase 2: Delete Dead Code

These are genuinely dead items that should be removed.

| File | Item | Action |
|------|------|--------|
| `mod.rs` | `EditorMessage` enum | Delete entirely |
| `mod.rs` | `message_tx` field | Delete from `VstKitEditor` struct |

**Dependencies:**
- Deleting `EditorMessage` may require removing the import of `std::sync::mpsc::Sender`
- Deleting `message_tx` requires removing its initialization in `VstKitEditor::new()`

**Verification:** Full build + tests pass.

---

### Phase 3: Refactor Debug Utilities

| File | Item | Action |
|------|------|--------|
| `desktop/src/assets.rs` | `list_assets` | Move to `#[cfg(test)]` module |
| `desktop/src/assets.rs` | `collect_paths` | Move with `list_assets` |

**Verification:** Tests still pass.

---

### Phase 4: Update Legitimate Suppressions

These suppressions are valid but need better comments.

| File | Item | New Comment |
|------|------|-------------|
| `webview.rs` | `resize` trait method | "Called by platform implementations, not trait consumers" |
| `webview.rs` | `close` trait method | Same as above |
| `windows.rs` | `hwnd` field | "Retained for future focus/positioning operations" |

---

## Verification Checklist

After all changes:

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo xtask bundle` succeeds
- [ ] Plugin loads in Ableton Live (manual test)

---

## Final Suppression Count

| Category | Count |
|----------|-------|
| **Platform trait methods** | 2 (resize, close) |
| **Windows-specific storage** | 1 (hwnd) |

**Total:** 3 suppressions (was 14)

---

## Architectural Notes

### Why Some Suppressions Are Legitimate

Rust's dead code analysis doesn't understand:

1. **Trait methods called on concrete types** — If you define a trait method and only call it on `MacOSWebView`, Rust doesn't see that as "using" the trait method
2. **Platform-conditional compilation** — Code inside `#[cfg(target_os = "windows")]` isn't analyzed when compiling on macOS
3. **Fields used for lifecycle** — Storing a handle just to keep it alive isn't detected as "usage"

These are cases where `#[allow(dead_code)]` is appropriate.

### Why Most Suppressions Here Are Wrong

The existing suppressions were added during development when:
- The egui fallback was still present (React editor was behind a feature flag)
- Code was genuinely not yet wired up

Now that React UI is the default, most of this code IS being used. The suppressions and comments are **technical debt from an earlier state**.

---

## Handoff

→ **Planner**: Create implementation plan with step-by-step tasks
