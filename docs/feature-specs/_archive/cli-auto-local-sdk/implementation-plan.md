# Implementation Plan: Auto-Detect Local SDK for Development

## Overview

When `wavecraft create` is run from a source checkout of the SDK (e.g., via `cargo run -p wavecraft`), the generated plugin references a git tag (`wavecraft-cli-v{VERSION}`) that doesn't exist yet — the tag is only created when the CD pipeline publishes the CLI. This causes `wavecraft start` (which runs `cargo build`) to fail with a "tag not found" error. The fix is to automatically detect when the CLI is running from the SDK repository and behave as if `--local-sdk` was passed.

## Problem

```
wavecraft create myPlugin    → generates engine/Cargo.toml with tag = "wavecraft-cli-v0.8.5"
cd myPlugin && wavecraft start → cargo build fails: tag "wavecraft-cli-v0.8.5" not found
```

**Root cause:** CLI version (0.8.5) → `SDK_VERSION = "wavecraft-cli-v0.8.5"` → embedded in template Cargo.toml → tag doesn't exist until CD pipeline publishes.

## Requirements

- `wavecraft create` must auto-detect SDK development mode when run from the wavecraft monorepo
- When in SDK dev mode, generate path dependencies (same as `--local-sdk`) instead of git tag dependencies
- Print a notice to the user so behavior is transparent
- End users installing via `cargo install wavecraft` must be completely unaffected
- Explicit `--local-sdk` flag must continue to work as before
- Existing tests must continue to pass

## Architecture Changes

### Compile-Time Approach

The key insight is that we can detect at **compile time** whether the CLI is being built from within the wavecraft monorepo workspace. When built via `cargo run -p wavecraft` from the repo root, `CARGO_MANIFEST_DIR` points to `<repo>/cli/`. The parent directory contains `engine/crates/` — the SDK marker.

We embed the resolved monorepo root path at compile time. At runtime, we check if that monorepo root still exists and contains the expected structure. If so, we auto-enable local SDK mode.

### Why compile-time + runtime validation?

- **Compile-time only** (embed a flag like `IS_DEV_BUILD`): Would be wrong for `cargo install --path cli` — the user intends to install, not develop.
- **Runtime only** (search upward for `engine/crates/`): Would work but is fragile — the installed binary has no relationship to a repo.
- **Compile-time path + runtime validation**: The compiled binary knows where the repo was at build time. At runtime, it checks if that location still has the SDK structure. This correctly handles:
  - `cargo run -p wavecraft` → detects repo, auto-enables local SDK ✅
  - `cargo install wavecraft` from crates.io → `CARGO_MANIFEST_DIR` points to cargo registry, no SDK structure → normal mode ✅
  - `cargo install --path cli` → `CARGO_MANIFEST_DIR` points to real repo but the binary runs from `~/.cargo/bin` — the repo path may still exist, but we add a safeguard: also check that the **current binary location** is within or built from the repo (via checking the build output dir matches).

### Simpler Alternative: Check CLI binary path at runtime

Actually, the simplest and most reliable approach:

1. At runtime, check if the **running binary's path** is inside a cargo `target/` directory (which means it was run via `cargo run`, not installed).
2. If so, walk up from the binary location to find the monorepo root (look for `engine/crates/wavecraft-nih_plug/Cargo.toml`).
3. If found, auto-enable local SDK mode.

This is reliable because:
- `cargo run -p wavecraft` → binary at `<repo>/cli/target/debug/wavecraft` or `<repo>/target/debug/wavecraft` → walk up → find `engine/crates/` ✅
- `cargo install wavecraft` → binary at `~/.cargo/bin/wavecraft` → no `target/` in path → normal mode ✅
- `cargo install --path cli` → binary at `~/.cargo/bin/wavecraft` → same as above ✅

**Decision: Use the runtime binary path approach (Simpler Alternative).** No build script needed. The detection logic is straightforward and testable.

## Implementation Steps

### Phase 1: SDK Repository Detection Module

1. **Create `cli/src/sdk_detect.rs`** — New module for SDK repo detection
   - Action: Add function `detect_sdk_repo() -> Option<PathBuf>` that:
     1. Gets the current binary path via `std::env::current_exe()`
     2. Checks if any ancestor directory contains `target/debug` or `target/release` (indicating `cargo run`)
     3. From that ancestor, looks for `engine/crates/wavecraft-nih_plug/Cargo.toml`
     4. If found, returns `Some(path_to_engine_crates)`; otherwise `None`
   - Add function `is_cargo_run_binary() -> bool` that checks if the binary path contains a `target/` segment (heuristic for `cargo run` vs installed binary)
   - Why: Isolated, testable detection logic that can be unit tested
   - Dependencies: None
   - Risk: Low — pure path inspection, no side effects

2. **Add unit tests for `sdk_detect.rs`**
   - Action: Test the `is_cargo_run_binary()` heuristic with known paths
   - Test both positive case (binary in `target/debug/`) and negative case (`~/.cargo/bin/`)
   - Why: Critical logic must be well-tested to avoid false positives for end users
   - Dependencies: Step 1
   - Risk: Low

### Phase 2: Wire Detection into `create` Command

3. **Register the new module** (File: `cli/src/main.rs`)
   - Action: Add `mod sdk_detect;` declaration
   - Dependencies: Step 1
   - Risk: Low

4. **Update `CreateCommand::execute()`** (File: `cli/src/commands/create.rs`)
   - Action: Before resolving `sdk_path`, add auto-detection logic:
     ```
     If --local-sdk is NOT explicitly set:
       Call detect_sdk_repo()
       If Some(sdk_path):
         Print notice: "Detected SDK development mode — using local path dependencies"
         Use sdk_path as local_dev
     If --local-sdk IS explicitly set:
       Existing behavior (find_local_sdk_path())
     ```
   - Important: The auto-detection is a **fallback** — explicit `--local-sdk` takes precedence
   - Why: This is where the template variables are constructed; local_dev presence controls dependency generation
   - Dependencies: Steps 1, 3
   - Risk: Medium — must not accidentally trigger for installed CLI binaries

5. **Update `CreateCommand` struct** (File: `cli/src/commands/create.rs`)
   - Action: No structural changes needed. The `local_sdk: bool` field remains; auto-detection happens inside `execute()`.
   - Dependencies: Step 4
   - Risk: Low

### Phase 3: Inform the User

6. **Add informative output for auto-detected SDK mode** (File: `cli/src/commands/create.rs`)
   - Action: When auto-detection triggers, print:
     ```
     ℹ Detected SDK development mode (running from source checkout)
       → Using local path dependencies instead of git tags
       → To force git tag mode, install via: cargo install wavecraft
     ```
   - Why: Transparency — the user should know why behavior differs from documented workflow
   - Dependencies: Step 4
   - Risk: Low

### Phase 4: Integration Tests

7. **Add integration test for auto-detection scenario** (File: `cli/src/sdk_detect.rs` or a new test file)
   - Action: Test that `detect_sdk_repo()` returns `Some` when the current exe is in a path matching the monorepo structure
   - Also test that detection does NOT trigger for paths like `~/.cargo/bin/wavecraft`
   - Why: Ensures the feature works end-to-end
   - Dependencies: Steps 1-6
   - Risk: Low

8. **Verify existing `--local-sdk` tests still pass**
   - Action: Run `cargo test --manifest-path cli/Cargo.toml` and confirm all existing template/override tests pass
   - Why: Regression prevention
   - Dependencies: Steps 1-6
   - Risk: Low

### Phase 5: Documentation Update

9. **Update SDK Getting Started guide** (File: `docs/guides/sdk-getting-started.md`)
   - Action: Add a note about the auto-detection behavior for SDK developers
   - Why: Developers working on the SDK should know this feature exists
   - Dependencies: Steps 1-8
   - Risk: Low

## Detailed Design: `sdk_detect.rs`

```rust
//! Auto-detection of SDK repository for development mode.
//!
//! When the CLI is run via `cargo run` from within the wavecraft monorepo,
//! this module detects the SDK source tree so generated projects can use
//! local path dependencies instead of git tags (which may not exist yet).

use std::path::{Path, PathBuf};

/// Marker file that identifies the wavecraft SDK repository.
const SDK_MARKER: &str = "engine/crates/wavecraft-nih_plug/Cargo.toml";

/// Attempts to detect whether the CLI is running from a source checkout
/// of the wavecraft SDK (i.e., via `cargo run`).
///
/// Returns `Some(path)` pointing to `engine/crates/` if detected,
/// or `None` if the CLI appears to be installed normally.
pub fn detect_sdk_repo() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    
    // Only proceed if the binary is in a cargo target/ directory
    // (i.e., running via `cargo run`, not `cargo install`)
    if !is_cargo_run_binary(&exe_path) {
        return None;
    }
    
    // Walk up from the binary to find the monorepo root
    find_monorepo_root(&exe_path)
        .map(|root| root.join("engine/crates"))
}

/// Checks if the binary appears to be running from a cargo build directory.
fn is_cargo_run_binary(exe_path: &Path) -> bool {
    exe_path.components().any(|c| {
        c.as_os_str() == "target"
    })
}

/// Walks up from the given path looking for the wavecraft monorepo root.
fn find_monorepo_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.parent();
    while let Some(dir) = current {
        if dir.join(SDK_MARKER).is_file() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}
```

## Edge Cases Considered

| Scenario | Expected Behavior | Why |
|----------|-------------------|-----|
| `cargo run -p wavecraft -- create foo` from repo root | Auto-detect → local path deps | Binary in `target/debug/` + SDK marker found |
| `cargo install wavecraft && wavecraft create foo` | Normal → git tag deps | Binary in `~/.cargo/bin/`, no `target/` segment |
| `cargo install --path cli && wavecraft create foo` | Normal → git tag deps | Binary in `~/.cargo/bin/`, no `target/` segment |
| `cargo run -p wavecraft -- create foo --local-sdk` | Explicit `--local-sdk` wins | Explicit flag takes precedence over auto-detect |
| SDK repo deleted after `cargo run` build | Normal → git tag deps | Marker file no longer exists at runtime |
| Cross-compiled binary | Normal → git tag deps | Target dir layout different, likely no marker |

## Testing Strategy

- **Unit tests** (`cli/src/sdk_detect.rs`):
  - `is_cargo_run_binary()` with known paths
  - `find_monorepo_root()` with temp dir structures
- **Existing tests** (`cli/src/template/mod.rs`):
  - `test_apply_local_dev_overrides` — must still pass unchanged
  - `test_extract_template` — must still pass unchanged
- **Manual E2E**:
  - `cargo run -p wavecraft -- create TestPlugin --output target/tmp/test-plugin` → should auto-detect
  - `cd target/tmp/test-plugin && wavecraft start` → should build successfully with path deps
  - Verify installed CLI (`cargo install --path cli`) does NOT auto-detect

## Risks & Mitigations

- **Risk**: False positive on systems where some other project has a `target/` directory in the path
  - Mitigation: We check for BOTH `target/` in the binary path AND the specific SDK marker file. The combination is highly specific to wavecraft.

- **Risk**: Binary path resolution via `current_exe()` fails on some platforms
  - Mitigation: We return `None` (normal mode) on any failure — safe fallback.

- **Risk**: Symlinks or unusual binary locations confuse the detection
  - Mitigation: `current_exe()` resolves symlinks on most platforms. If it doesn't, we fall back to normal mode.

## Success Criteria

- [ ] `cargo run -p wavecraft -- create TestPlugin --output target/tmp/test-plugin` generates path deps (not git tag deps) in `engine/Cargo.toml`
- [ ] `cd target/tmp/test-plugin && wavecraft start` builds and starts successfully
- [ ] The auto-detection notice is printed to the console
- [ ] Existing `--local-sdk` flag still works as before
- [ ] All existing CLI tests pass (`cargo test --manifest-path cli/Cargo.toml`)
- [ ] Installed CLI binary (`cargo install --path cli`) does NOT trigger auto-detection
