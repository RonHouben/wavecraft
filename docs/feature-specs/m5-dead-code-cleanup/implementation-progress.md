# Implementation Progress: M5 Dead Code Cleanup

## Status: 🚧 Ready for Implementation

---

## Task Checklist

### Phase 1: Remove Stale Suppressions in `webview.rs`
- [ ] 1.1 Remove suppression from `WebViewConfig`
- [ ] 1.2 Remove suppression from `create_ipc_handler`
- [ ] 1.3 Remove suppression from `IPC_PRIMITIVES_JS`

### Phase 2: Remove Stale Suppressions in `assets.rs`
- [ ] 2.1 Remove suppression from `UI_ASSETS`
- [ ] 2.2 Remove suppression from `get_asset`
- [ ] 2.3 Remove suppression from `mime_type_from_path`

### Phase 3: Remove Stale Suppressions in `bridge.rs`
- [ ] 3.1 Remove suppression from `PluginEditorBridge` struct
- [ ] 3.2 Remove suppression from `PluginEditorBridge::new`

### Phase 4: Delete Dead Code in `mod.rs`
- [ ] 4.1 Delete `EditorMessage` enum
- [ ] 4.2 Delete `message_tx` field from `VstKitEditor`
- [ ] 4.3 Clean up unused imports (if any)

### Phase 5: Refactor Debug Utility
- [ ] 5.1 Move `list_assets` and `collect_paths` to test module

### Phase 6: Update Legitimate Suppressions
- [ ] 6.1 Update `resize` trait method comment
- [ ] 6.2 Update `close` trait method comment
- [ ] 6.3 Update `hwnd` field comment

### Phase 7: Version Bump
- [ ] 7.1 Bump version to 0.2.2

### Phase 8: Verification
- [ ] 8.1 Run formatter (`cargo fmt`)
- [ ] 8.2 Run clippy (`cargo clippy --workspace -- -D warnings`)
- [ ] 8.3 Run tests (`cargo xtask test`)
- [ ] 8.4 Build plugin (`cargo xtask bundle`)
- [ ] 8.5 Manual test in Ableton Live

---

## Metrics

| Metric | Before | After | Target |
|--------|--------|-------|--------|
| `#[allow(dead_code)]` count | 14 | — | 3 |
| Stale "re-enable" comments | ~4 | — | 0 |
| Build warnings | 0 | — | 0 |
| Test failures | 0 | — | 0 |

---

## Notes

_Implementation notes will be added here as work progresses._
