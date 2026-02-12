# Implementation Plan: SDK Example Plugin

## Overview

Enable `cargo xtask dev` to work from the SDK root by creating a minimal example plugin crate (`wavecraft-example`) that mirrors the template structure and modifying the project detection logic to support "SDK mode". This allows SDK developers to test UI and dev-server changes without creating a separate plugin project.

## Requirements

- SDK developers must be able to run `cargo xtask dev` from the SDK repository root
- The example plugin must use the same macros and patterns as the template (template parity)
- No breaking changes to existing `wavecraft start` flow for generated plugin projects
- The example crate must build with `_param-discovery` feature for parameter extraction
- All existing tests must continue passing with SDK mode detection properly distinguished

## Architecture Changes

- **New crate**: `engine/crates/wavecraft-example/` — minimal plugin using SDK macros
- **Modified**: `cli/src/project/detection.rs` — adds `sdk_mode` field and SDK workspace detection
- **Modified**: `cli/src/project/detection.rs` (tests) — updates SDK detection test expectations
- **No change**: `cli/src/project/dylib.rs` — existing workspace fallback handles SDK mode
- **No change**: `cli/src/commands/start.rs` — existing `--package` flag handles SDK mode

## Implementation Steps

### Phase 1: Create Example Plugin Crate

**Dependencies**: None

#### 1. **Create crate directory** (File: `engine/crates/wavecraft-example/`)

- Action: Create `engine/crates/wavecraft-example/` directory
- Why: Houses the example plugin implementation
- Dependencies: None
- Risk: Low

#### 2. **Create Cargo.toml** (File: `engine/crates/wavecraft-example/Cargo.toml`)

- Action: Create manifest with `[lib]` cdylib configuration, workspace dependencies
- Why: Defines the crate as a loadable plugin library
- Dependencies: Step 1
- Risk: Low
- Details:

  ```toml
  [package]
  name = "wavecraft-example"
  description = "Example Wavecraft plugin for SDK development and testing"
  publish = false
  version.workspace = true
  edition.workspace = true
  license.workspace = true
  authors.workspace = true
  repository.workspace = true

  [lib]
  name = "wavecraft_example"
  crate-type = ["cdylib"]

  [dependencies]
  wavecraft = { package = "wavecraft-nih_plug", path = "../wavecraft-nih_plug" }
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"
  log = "0.4"

  [features]
  default = []
  _param-discovery = []
  ```

#### 3. **Create plugin implementation** (File: `engine/crates/wavecraft-example/src/lib.rs`)

- Action: Write minimal plugin using `wavecraft_plugin!`, `wavecraft_processor!`, and `SignalChain![]` macros
- Why: Provides a working plugin that exercises the SDK's macro system
- Dependencies: Step 2
- Risk: Low
- Details:

  ```rust
  use wavecraft::prelude::*;

  wavecraft_processor!(InputGain => Gain);
  wavecraft_processor!(OutputGain => Gain);

  wavecraft_plugin! {
      name: "Wavecraft Example",
      signal: SignalChain![InputGain, OutputGain],
  }
  ```

#### 4. **Verify workspace inclusion** (File: `engine/Cargo.toml`)

- Action: Confirm `members = ["crates/*", "xtask"]` includes the new crate (no change needed)
- Why: The glob pattern auto-includes `wavecraft-example`
- Dependencies: Steps 1-3
- Risk: Low

#### 5. **Test crate compilation** (Command line)

- Action: Run `cargo build -p wavecraft-example` from SDK root
- Why: Validates the crate builds successfully
- Dependencies: Steps 1-4
- Risk: Low

#### 6. **Test param-discovery feature** (Command line)

- Action: Run `cargo build -p wavecraft-example --features _param-discovery`
- Why: Validates the fast-build path used by `wavecraft start`
- Dependencies: Step 5
- Risk: Low

### Phase 2: Modify Detection Logic

**Dependencies**: Phase 1 complete

#### 7. **Add sdk_mode field to ProjectMarkers** (File: `cli/src/project/detection.rs`)

- Action: Add `pub sdk_mode: bool` field to the `ProjectMarkers` struct
- Why: Tracks whether we're in SDK mode vs. a regular plugin project
- Dependencies: Phase 1
- Risk: Low
- Location: Line ~18 (in struct definition)

#### 8. **Implement SDK mode detection** (File: `cli/src/project/detection.rs`)

- Action: Modify `detect()` function to check for `[workspace]` in `engine/Cargo.toml` and redirect `engine_dir` to `wavecraft-example` on detection
- Why: Enables the CLI to distinguish SDK repo from plugin project
- Dependencies: Step 7
- Risk: Medium (must handle all edge cases)
- Details:
  - After initial directory checks, call `is_sdk_repo(&engine_cargo_toml)?`
  - If true, validate `engine/crates/wavecraft-example/` exists
  - Set `engine_dir` to `engine/crates/wavecraft-example/`
  - Set `engine_cargo_toml` to `engine/crates/wavecraft-example/Cargo.toml`
  - Set `sdk_mode = true`
- Location: Lines ~45-65 (in `detect()` function)

#### 9. **Update normal project path** (File: `cli/src/project/detection.rs`)

- Action: Add `sdk_mode: false` to the final `Ok(Self { ... })` return in the normal (non-SDK) path
- Why: All `ProjectMarkers` instances must have `sdk_mode` set
- Dependencies: Steps 7-8
- Risk: Low
- Location: Line ~66 (final Ok return)

### Phase 3: Update Tests

**Dependencies**: Phase 2 complete

#### 10. **Update test_sdk_repo_detection** (File: `cli/src/project/detection.rs`)

- Action: Modify test to create `wavecraft-example` dir structure and assert `sdk_mode == true` instead of expecting an error
- Why: The test currently expects SDK detection to fail; now it should succeed
- Dependencies: Steps 7-9
- Risk: Medium (test behavior changes significantly)
- Details:

  ```rust
  // Create wavecraft-example/ structure
  fs::create_dir_all(tmp.path().join("engine/crates/wavecraft-example/src")).unwrap();
  fs::write(
      tmp.path().join("engine/crates/wavecraft-example/Cargo.toml"),
      "[package]\nname = \"wavecraft-example\"\n\n[lib]\nname = \"wavecraft_example\"\ncrate-type = [\"cdylib\"]"
  ).unwrap();
  fs::write(tmp.path().join("engine/crates/wavecraft-example/src/lib.rs"), "").unwrap();

  let result = ProjectMarkers::detect(tmp.path());
  assert!(result.is_ok());
  let markers = result.unwrap();
  assert!(markers.sdk_mode);
  assert_eq!(markers.engine_dir, tmp.path().join("engine/crates/wavecraft-example"));
  ```

- Location: Lines ~188-210

#### 11. **Add test_sdk_mode_missing_example** (File: `cli/src/project/detection.rs`)

- Action: Create new test that verifies error when `[workspace]` present but `wavecraft-example/` directory is missing
- Why: Validates error handling for incomplete SDK structure
- Dependencies: Steps 7-10
- Risk: Low
- Details:

  ```rust
  #[test]
  fn test_sdk_mode_missing_example() {
      let tmp = TempDir::new().unwrap();
      fs::create_dir_all(tmp.path().join("ui")).unwrap();
      fs::create_dir_all(tmp.path().join("engine")).unwrap();
      fs::write(tmp.path().join("ui/package.json"), "{}").unwrap();
      fs::write(
          tmp.path().join("engine/Cargo.toml"),
          "[workspace]\nmembers = [\"crates/*\"]",
      ).unwrap();
      // Note: NOT creating wavecraft-example/

      let result = ProjectMarkers::detect(tmp.path());
      assert!(result.is_err());
      assert!(result.unwrap_err().to_string().contains("wavecraft-example"));
  }
  ```

- Location: After `test_sdk_repo_detection` (new test)

#### 12. **Verify test_plugin_project_detection still passes** (File: `cli/src/project/detection.rs`)

- Action: Run test to confirm plugin projects with `[package]` are detected correctly with `sdk_mode = false`
- Why: Ensures no regression for normal plugin projects
- Dependencies: Steps 7-11
- Risk: Low
- Location: Lines ~212-218 (existing test)

### Phase 4: Integration Testing

**Dependencies**: Phase 3 complete

#### 13. **Test cargo xtask dev from SDK root** (Command line)

- Action: From SDK root, run `cargo xtask dev` and verify servers start without errors
- Why: End-to-end validation of SDK mode detection, build, param loading, server startup
- Dependencies: Steps 1-12
- Risk: Medium (full integration test)
- Expected output:
  - Project detection succeeds
  - `cargo build -p wavecraft-example --features _param-discovery` runs
  - Dylib found at `engine/target/debug/libwavecraft_example.dylib`
  - Parameters extracted (InputGain, OutputGain)
  - WebSocket server starts on port 9000
  - Vite UI server starts on port 5173

#### 14. **Test parameter extraction** (Browser + WebSocket)

- Action: Connect browser to `http://localhost:5173`, verify IPC works and InputGain/OutputGain parameters are visible
- Why: Validates parameter discovery and IPC for the example plugin
- Dependencies: Step 13
- Risk: Low (uses existing infrastructure)

#### 15. **Test hot-reload** (Command line + editor)

- Action: Edit `engine/crates/wavecraft-example/src/lib.rs`, verify file watcher triggers rebuild
- Why: Confirms file watching works for the example crate
- Dependencies: Steps 13-14
- Risk: Low
- Details: Add a comment or change plugin name, verify console shows "Rebuild triggered"

#### 16. **Test normal plugin project unchanged** (New temp project)

- Action: Run `wavecraft create TestPlugin --output target/tmp/test-plugin`, then `wavecraft start` from the generated project
- Why: Regression test — ensures generated projects still work correctly
- Dependencies: Phase 3
- Risk: Medium (critical regression check)

### Phase 5: CI & Documentation

**Dependencies**: Phase 4 complete

#### 17. **Add clippy check for wavecraft-example** (File: `.github/workflows/ci.yml` or xtask)

- Action: Add `cargo clippy -p wavecraft-example -- -D warnings` to lint pipeline
- Why: Enforces code quality for the example crate
- Dependencies: Phase 4
- Risk: Low
- Location: Existing lint step in CI workflow

#### 18. **Update CHANGELOG.md** (File: `CHANGELOG.md`)

- Action: Add entry for SDK example plugin feature under unreleased section
- Why: Documents the new capability
- Dependencies: Phase 4
- Risk: Low
- Entry: "SDK: `cargo xtask dev` now works from SDK root using `wavecraft-example` crate"

#### 19. **Document SDK dev workflow** (File: `docs/guides/sdk-getting-started.md` or README)

- Action: Add section explaining `cargo xtask dev` for SDK developers
- Why: Makes the feature discoverable
- Dependencies: Phase 4
- Risk: Low
- Content:

  ````markdown
  ## SDK Development Workflow

  When working on Wavecraft SDK itself, you can test changes without creating a separate plugin:

  ```bash
  # From SDK root
  cargo xtask dev
  ```
  ````

  This uses the built-in `wavecraft-example` plugin crate to demonstrate UI and dev-server features.

  ```

  ```

#### 20. **Run full test suite** (Command line)

- Action: Execute `cargo test --workspace` from SDK root
- Why: Final validation that all tests pass with changes
- Dependencies: Steps 1-19
- Risk: Low

## Testing Strategy

### Unit Tests

| Test                            | File                           | What It Validates                                                  |
| ------------------------------- | ------------------------------ | ------------------------------------------------------------------ |
| `test_sdk_repo_detection`       | `cli/src/project/detection.rs` | SDK workspace detection with `wavecraft-example` present (updated) |
| `test_sdk_mode_missing_example` | `cli/src/project/detection.rs` | Error when SDK detected but example crate missing (new)            |
| `test_plugin_project_detection` | `cli/src/project/detection.rs` | Normal plugin projects return `sdk_mode = false` (existing)        |
| `test_project_detection_valid`  | `cli/src/project/detection.rs` | Existing validation still works (existing)                         |

### Integration Tests

| Test                       | How                                                            | What It Validates                                        |
| -------------------------- | -------------------------------------------------------------- | -------------------------------------------------------- |
| SDK dev server startup     | `cargo xtask dev` from SDK root                                | Full flow: detection → build → param load → server start |
| Parameter extraction       | Browser WebSocket connection                                   | Example plugin params visible in UI                      |
| Hot-reload                 | Edit `wavecraft-example/src/lib.rs`                            | File watcher triggers rebuild                            |
| Normal plugin unchanged    | `wavecraft create` + `wavecraft start`                         | Generated projects work correctly (regression)           |
| Build with param-discovery | `cargo build -p wavecraft-example --features _param-discovery` | Fast build path works                                    |

### Manual Testing Checklist

- [ ] `cargo xtask dev` starts both servers successfully
- [ ] Browser can connect to `http://localhost:5173`
- [ ] InputGain and OutputGain parameters appear in UI
- [ ] Parameter changes send IPC messages
- [ ] Editing `lib.rs` triggers rebuild (watch console)
- [ ] `wavecraft create TestPlugin --output target/tmp/test-plugin` works
- [ ] `wavecraft start` works from generated project
- [ ] `cargo test --workspace` passes all tests
- [ ] `cargo clippy -p wavecraft-example -- -D warnings` passes

## Risks & Mitigations

| #   | Risk                                     | Impact                                                  | Likelihood | Mitigation                                                                                                                 |
| --- | ---------------------------------------- | ------------------------------------------------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------- |
| 1   | **Workspace target dir confusion**       | Dylib not found after build                             | Low        | `resolve_debug_dir()` already has two-level fallback. Verified by reading `dylib.rs` lines 78-98.                          |
| 2   | **Test expectations change**             | `test_sdk_repo_detection` fails unexpectedly            | Medium     | Update test carefully. Create `wavecraft-example` structure in test. Assert `sdk_mode == true` and correct paths.          |
| 3   | **Example crate drifts from template**   | Example no longer representative                        | Medium     | CI clippy check on example crate. Document invariant: example MUST mirror template. Manual review during template updates. |
| 4   | **sdk_mode flag leaks**                  | Generated projects incorrectly detect SDK mode          | Low        | Only `true` when `[workspace]` present. Generated projects have `[package]`. Add integration test in step 16.              |
| 5   | **Cargo package flag fails**             | Build runs wrong package or fails to find workspace     | Low        | `--package wavecraft-example` is explicit. Cargo walks up to workspace root. Standard behavior.                            |
| 6   | **Hot-reload doesn't catch SDK changes** | Developers edit `wavecraft-core`, rebuild not triggered | Medium     | Document limitation: manual restart needed for SDK crate changes. Future: extend watcher. Out of scope.                    |

## Success Criteria

- [ ] `cargo xtask dev` runs successfully from SDK root without errors
- [ ] Example plugin builds with `_param-discovery` feature
- [ ] Parameters (InputGain, OutputGain) are extracted and visible in browser UI
- [ ] Hot-reload triggers rebuild when `wavecraft-example/src/lib.rs` is edited
- [ ] All unit tests pass (`cargo test --workspace`)
- [ ] Integration test: generated plugin projects remain unaffected
- [ ] CI includes clippy check for `wavecraft-example`
- [ ] Documentation updated with SDK dev workflow

## Rollout Plan

### Pre-Merge Validation

1. Run full test suite locally: `cargo test --workspace`
2. Test SDK mode: `cargo xtask dev` from SDK root
3. Test normal mode: `wavecraft create TestPlugin --output target/tmp/test && cd target/tmp/test && wavecraft start`
4. Verify clippy: `cargo clippy -p wavecraft-example -- -D warnings`

### Post-Merge Monitoring

1. CI pipeline must pass (lint, test, template-validation)
2. Manual smoke test: SDK developer runs `cargo xtask dev` on clean clone
3. Check for GitHub issues related to `wavecraft start` failures

### Rollback Plan

If SDK mode causes issues:

1. Remove `engine/crates/wavecraft-example/` directory
2. Revert changes to `cli/src/project/detection.rs`
3. Re-run test suite to confirm normal operation restored

---

## Notes

- **No changes to dylib.rs or start.rs**: Existing abstractions (`engine_dir`, `--package` flag, workspace fallback) handle SDK mode without modification
- **Template parity**: The example plugin MUST use the same macros and patterns as the template. Any template updates require corresponding example updates.
- **Future enhancement**: Extend file watcher to observe all `engine/crates/*/src/` directories for fuller hot-reload coverage during SDK development.
- **Windows/Linux**: Not validated (consistent with macOS-first development constraint)

---
