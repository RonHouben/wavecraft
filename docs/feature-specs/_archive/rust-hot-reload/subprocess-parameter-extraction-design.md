# Low-Level Design: Subprocess-Based Parameter Extraction

## Related Documents

- [Hot-Reload Low-Level Design](./low-level-design-rust-hot-reload.md) — Original hot-reload architecture
- [Hot-Reload Bugfix Plan](./hot-reload-param-update-bugfix-plan.md) — Investigation that identified the `dlopen` hang
- [High-Level Design](../../architecture/high-level-design.md) — Overall system architecture
- [Development Workflows](../../architecture/development-workflows.md) — Dev workflow documentation
- [Coding Standards (Rust)](../../architecture/coding-standards-rust.md) — Rust conventions

## Date

February 10, 2026

---

## 1. Problem Statement

`wavecraft start` hangs indefinitely when calling `dlopen()` to load a plugin dylib for parameter extraction. The hang is caused by macOS `dyld` executing static initializers in nih-plug or its transitive dependencies that interact with system services (audio subsystem, mach ports, etc.).

The `_param-discovery` feature flag successfully suppresses nih-plug's `nih_export_clap!` / `nih_export_vst3!` macros, but **other static initializers outside the wavecraft macro's control** still trigger the hang.

**Affected paths:**
- **Initial startup** (no sidecar cache): start.rs → `load_parameters()` → `PluginParamLoader::load_params_only()`
- **Hot-reload** (every rebuild): rebuild.rs → `load_parameters_from_dylib()` → `PluginParamLoader::load_params_only()`

**Current mitigation:** The sidecar cache masks the issue on subsequent startups, but:
1. First-ever `wavecraft start` on a new project has no cache → hangs
2. Hot-reload **always** calls `dlopen` on the rebuilt dylib → hangs

---

## 2. Solution Overview

**Move the `dlopen` + FFI call into a short-lived subprocess** that can be forcefully terminated on timeout. The parent process (`wavecraft start`) never calls `dlopen` on user plugin dylibs.

### Architecture Diagram

```
wavecraft start (parent process)
│
├── Initial Startup ─────────────────────────────────┐
│   1. Try sidecar cache                             │
│   2. cargo build --lib --features _param-discovery  │
│   3. Spawn subprocess for extraction ──────────────┼──┐
│   4. Parse JSON from stdout                        │  │
│   5. Write sidecar cache                           │  │
│                                                    │  │
├── Hot-Reload ──────────────────────────────────────┘  │
│   1. cargo build --lib --features _param-discovery     │
│   2. Copy dylib to temp path                          │
│   3. Spawn subprocess for extraction ─────────────────┤
│   4. Parse JSON from stdout                           │
│   5. Update ParameterHost + notify UI                 │
│                                                       │
│                                                       ▼
│                              ┌──────────────────────────────────┐
│                              │  wavecraft extract-params        │
│                              │  (subprocess — same binary)      │
│                              │                                  │
│                              │  1. dlopen(dylib_path)           │
│                              │  2. dlsym(wavecraft_get_params)  │
│                              │  3. Call FFI → get JSON          │
│                              │  4. Print JSON to stdout         │
│                              │  5. dlclose + exit               │
│                              │                                  │
│                              │  Lifecycle: spawn → work → exit  │
│                              │  If hangs: killed after timeout  │
│                              └──────────────────────────────────┘
│
│  On timeout (30s default):
│  ├── Kill subprocess (SIGKILL / TerminateProcess)
│  └── Return error with diagnostic context
```

### Key Design Decision: Hidden Subcommand, Not Separate Binary

The subprocess reuses the **existing `wavecraft` CLI binary** via a hidden `extract-params` subcommand, rather than compiling a separate `wavecraft-param-extract` binary.

**Rationale:**

| Approach | Pros | Cons |
|----------|------|------|
| **Hidden subcommand** (chosen) | Zero additional compile time; no extra binary to distribute; always in sync with parent; simplest build system integration | Subprocess loads entire CLI binary (but exits fast) |
| Separate binary | Minimal binary size; clear separation | Extra build target; must be co-located with CLI; distribution complexity; version sync risk |
| `cargo run` script | No binary changes | Requires Cargo/Rust toolchain on user's PATH; slow (compiles); unreliable |

The `wavecraft` CLI is already ~5MB. Loading it as a subprocess adds negligible overhead compared to the 2-10s Cargo build that precedes it. The subprocess exits within milliseconds on success — binary size is irrelevant.

---

## 3. Component Breakdown

### 3.1 Hidden Subcommand: `extract-params`

**Location:** New variant in the `Commands` enum in main.rs

```
wavecraft extract-params <dylib_path> [--timeout <seconds>]
```

**Behavior:**
1. Validate `dylib_path` exists and has expected extension (`.dylib` / `.so` / `.dll`)
2. Call `PluginParamLoader::load_params_only(dylib_path)` — this is the **only** place `dlopen` executes
3. Serialize `Vec<ParameterInfo>` to JSON
4. Write JSON to **stdout** (one line, compact format)
5. Exit with code 0

**Error behavior:**
- Any error → write structured error JSON to **stderr**, exit with code 1
- `dlopen` failure → exit code 2
- Symbol not found → exit code 3
- JSON parse failure → exit code 4

**Hidden from help:** Use clap's `#[command(hide = true)]` attribute so `wavecraft --help` doesn't show it. This is an internal implementation detail, not a user-facing command.

**No logging to stdout.** The subprocess must not print anything to stdout except the final JSON payload. All diagnostics go to stderr. This is critical — the parent parses stdout as JSON.

### 3.2 Subprocess Spawner: `extract_params_subprocess()`

**Location:** New function in `cli/src/project/` (new file `param_extract.rs`)

This replaces direct calls to `PluginParamLoader::load_params_only()` in both `start.rs` and `rebuild.rs`.

**Signature:**
```rust
pub async fn extract_params_subprocess(
    dylib_path: &Path,
    timeout: Duration,
) -> Result<Vec<ParameterInfo>>
```

**Algorithm:**
1. Resolve the path to the current `wavecraft` binary via `std::env::current_exe()`
2. Spawn: `<self_exe> extract-params <dylib_path>`
3. Capture stdout + stderr via piped handles
4. Apply `tokio::time::timeout(timeout, child.wait_with_output())`
5. On success (exit code 0): parse stdout as `Vec<ParameterInfo>`
6. On timeout: kill the subprocess, return error with stderr contents
7. On non-zero exit: return error with exit code + stderr contents

**Process cleanup (cross-platform):**

| Platform | Timeout Kill | Rationale |
|----------|-------------|-----------|
| macOS / Linux | `child.kill()` → sends `SIGKILL` | Cannot be caught; guaranteed termination. `SIGTERM` is insufficient because `dlopen` hangs in kernel/dyld — the process cannot handle signals while blocked in a system call. |
| Windows | `child.kill()` → calls `TerminateProcess` | Equivalent to `SIGKILL`; immediate, non-catchable. |

`tokio::process::Child::kill()` already does the right thing on both platforms. No platform-specific code needed for the kill path.

**Why not `SIGTERM` first?** The entire purpose of this subprocess is to handle unrecoverable `dlopen` hangs. If the process is hung in `dlopen`, it's blocked in a system call and cannot handle `SIGTERM`. Going straight to `SIGKILL` via `child.kill()` is correct.

### 3.3 Integration Points

#### 3.3.1 Initial Startup (`start.rs::load_parameters()`)

**Current flow:**
```
1. Try sidecar cache
2. Build with _param-discovery
3. PluginParamLoader::load_params_only(&dylib_path)  ← HANGS HERE
4. Write sidecar cache
```

**New flow:**
```
1. Try sidecar cache
2. Build with _param-discovery
3. extract_params_subprocess(&dylib_path, 30s)  ← SAFE
4. Write sidecar cache
```

The `#[cfg(feature = "audio-dev")]` path that calls `PluginParamLoader::load()` (full load with vtable) also needs subprocess isolation. See §3.4.

#### 3.3.2 Hot-Reload (`rebuild.rs::load_parameters_from_dylib()`)

**Current flow:**
```
1. find_plugin_dylib()
2. create_temp_dylib_copy()
3. PluginParamLoader::load_params_only(&temp_path)  ← HANGS HERE
4. Remove temp file
```

**New flow:**
```
1. find_plugin_dylib()
2. create_temp_dylib_copy()
3. extract_params_subprocess(&temp_path, 30s)  ← SAFE
4. Remove temp file
```

The existing `tokio::time::timeout` + `spawn_blocking` + `catch_unwind` wrapper in `do_build()` is **removed** — the subprocess spawner handles all of that internally. This significantly simplifies `do_build()`.

### 3.4 Audio-Dev Mode (`#[cfg(feature = "audio-dev")]`)

The `audio-dev` feature uses `PluginParamLoader::load()` which loads **both** parameters and the processor vtable. The vtable contains function pointers that must remain valid (the `Library` handle must stay alive).

**Subprocess isolation is NOT suitable for vtable loading.** The vtable's function pointers are only valid within the subprocess's address space. They cannot be transmitted to the parent process.

**Design decision:** Keep in-process `PluginParamLoader::load()` for `audio-dev` mode. This is acceptable because:
1. `audio-dev` is an opt-in development feature, not the default path
2. The vtable fundamentally requires in-process `dlopen`
3. The risk is documented and the existing timeout protection remains

For `audio-dev` mode, the initial parameter extraction during startup still uses in-process loading (needed for the vtable), but hot-reload parameter extraction (params-only) uses the subprocess.

---

## 4. API Design

### 4.1 Subprocess Command (CLI layer)

```rust
// In Commands enum (main.rs):
#[command(hide = true)]
ExtractParams {
    /// Path to the plugin dylib
    dylib_path: PathBuf,
}
```

```rust
// Handler (new file: cli/src/commands/extract_params.rs):
pub fn execute(dylib_path: &Path) -> Result<()> {
    // Validate path
    if !dylib_path.exists() {
        // Write error to stderr, exit 1
    }

    // This is the ONLY place dlopen happens
    let params = PluginParamLoader::load_params_only(dylib_path)?;

    // Compact JSON to stdout (no pretty-print — minimize pipe overhead)
    let json = serde_json::to_string(&params)?;
    println!("{}", json);

    Ok(())
}
```

### 4.2 Subprocess Spawner (internal API)

```rust
// New file: cli/src/project/param_extract.rs

use std::path::Path;
use std::time::Duration;
use anyhow::{Context, Result};
use wavecraft_protocol::ParameterInfo;

/// Default timeout for subprocess parameter extraction.
pub const DEFAULT_EXTRACT_TIMEOUT: Duration = Duration::from_secs(30);

/// Extract parameters from a plugin dylib via a subprocess.
///
/// Spawns `wavecraft extract-params <dylib_path>` and parses JSON from stdout.
/// The subprocess is killed if it exceeds the timeout.
///
/// This isolates `dlopen` from the parent process, preventing hangs
/// caused by macOS static initializers in nih-plug dependencies.
pub async fn extract_params_subprocess(
    dylib_path: &Path,
    timeout: Duration,
) -> Result<Vec<ParameterInfo>> {
    let self_exe = std::env::current_exe()
        .context("Failed to determine wavecraft binary path")?;

    let mut child = tokio::process::Command::new(&self_exe)
        .arg("extract-params")
        .arg(dylib_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| format!(
            "Failed to spawn parameter extraction subprocess: {}",
            self_exe.display()
        ))?;

    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)
                    .context("Subprocess stdout was not valid UTF-8")?;
                let params: Vec<ParameterInfo> = serde_json::from_str(stdout.trim())
                    .context("Failed to parse parameter JSON from subprocess")?;
                Ok(params)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let code = output.status.code().unwrap_or(-1);
                anyhow::bail!(
                    "Parameter extraction failed (exit code {}):\n{}",
                    code, stderr
                );
            }
        }
        Ok(Err(e)) => {
            anyhow::bail!("Failed to wait for extraction subprocess: {}", e);
        }
        Err(_) => {
            // Timeout — kill the subprocess
            let _ = child.kill().await;
            anyhow::bail!(
                "Parameter extraction timed out after {}s. \
                 This is likely caused by macOS static initializers in the plugin dylib. \
                 The plugin was built with --features _param-discovery but dlopen \
                 still hung. Check for transitive dependencies that register with \
                 macOS system services during library load.\n\
                 Dylib: {}",
                timeout.as_secs(),
                dylib_path.display()
            );
        }
    }
}
```

### 4.3 Caller Changes

**`start.rs::load_parameters()`** — Replace `PluginParamLoader::load_params_only()` calls:

```rust
// Before:
let params = PluginLoader::load_params_only(&dylib_path)
    .context("Failed to load plugin for parameter discovery")?;

// After:
let params = extract_params_subprocess(&dylib_path, DEFAULT_EXTRACT_TIMEOUT)
    .await
    .context("Failed to extract parameters from plugin")?;
```

Note: `load_parameters()` is currently synchronous. It must become `async` or use `tokio::runtime::Handle::current().block_on()` depending on whether the caller is already in an async context. The `start` command uses `tokio::main` — converting to async is preferred.

**`rebuild.rs::load_parameters_from_dylib()`** — Replace the entire function:

```rust
// Before: synchronous, called inside spawn_blocking + catch_unwind + timeout
fn load_parameters_from_dylib(engine_dir: PathBuf) -> Result<Vec<ParameterInfo>> { ... }

// After: async, called directly (no spawn_blocking / catch_unwind needed)
async fn load_parameters_from_dylib(engine_dir: PathBuf) -> Result<Vec<ParameterInfo>> {
    let lib_path = find_plugin_dylib(&engine_dir)
        .context("Failed to find plugin dylib after rebuild")?;

    let temp_path = create_temp_dylib_copy(&lib_path)?;

    let params = extract_params_subprocess(&temp_path, DEFAULT_EXTRACT_TIMEOUT)
        .await
        .with_context(|| format!("Failed to extract params: {}", temp_path.display()))?;

    let _ = std::fs::remove_file(&temp_path);

    Ok(params)
}
```

This eliminates the existing triple-nested `tokio::time::timeout` + `spawn_blocking` + `catch_unwind` in `do_build()`, replacing it with a single `await`.

---

## 5. Error Handling Strategy

### 5.1 Exit Code Protocol

| Exit Code | Meaning | Parent Action |
|-----------|---------|--------------|
| 0 | Success — JSON on stdout | Parse stdout |
| 1 | General error | Report stderr to user |
| 2 | `dlopen` failure (`LibraryLoad`) | Report with dylib path |
| 3 | Symbol not found (`SymbolNotFound`) | Suggest `_param-discovery` feature may not be enabled |
| 4 | JSON parse error (`JsonParse`) | Report — likely FFI corruption |
| 5 | Invalid UTF-8 (`InvalidUtf8`) | Report — likely FFI corruption |
| — (killed) | Timed out / hung | Report timeout + dylib path |

### 5.2 Stderr Format

The subprocess writes human-readable diagnostics to stderr. Not structured JSON — this is for developer consumption.

```
Error: Failed to load plugin library: dlopen(libmy_plugin.dylib): ...
  dylib: /tmp/wavecraft_hotreload_1234567890.dylib
  hint: This may be caused by static initializers in nih-plug dependencies.
```

### 5.3 Timeout Diagnostics

When the parent kills the subprocess on timeout, it reports:

```
⚠ Parameter extraction timed out after 30s.
  This indicates a dlopen hang caused by static initializers in the plugin dylib.
  The plugin was built with --features _param-discovery but dlopen still hung.

  Suggested actions:
    • Check for new transitive dependencies that interact with macOS system services
    • Try: nm -gU <dylib> | grep _init  (look for unexpected initializers)
    • File a bug with the offending dependency

  Dylib: /tmp/wavecraft_hotreload_1707552000000.dylib
```

### 5.4 Panic Safety

The subprocess may panic during FFI. Since it's a separate process, panics are contained — they produce a non-zero exit code. The parent handles this like any other failure. No `catch_unwind` needed.

---

## 6. Performance Analysis

### 6.1 Subprocess Spawn Overhead

| Operation | macOS (M1) | Linux | Windows |
|-----------|-----------|-------|---------|
| `fork()` + `exec()` | ~2-5ms | ~1-3ms | ~10-20ms (`CreateProcess`) |
| Load `wavecraft` binary | ~5-10ms | ~3-8ms | ~10-15ms |
| `dlopen` (success) | ~50-200ms | ~30-100ms | ~50-150ms |
| FFI call + JSON serialization | ~1-5ms | ~1-5ms | ~1-5ms |
| **Total (success)** | **~60-220ms** | **~35-115ms** | **~70-190ms** |

This is well under the 1s target. For context, the Cargo build that precedes extraction takes 2-30s.

### 6.2 Compared to Current In-Process Approach

| Metric | In-process (`load_params_only`) | Subprocess |
|--------|-------------------------------|-----------|
| Overhead | ~0ms (no spawn) | ~10-20ms (fork/exec) |
| Hang recovery | Timeout kills async task, but library remains loaded in address space (potential corruption) | Timeout kills subprocess cleanly (OS reclaims all resources) |
| Safety | `dlopen` contaminates parent process | Complete isolation |

The ~10-20ms overhead is negligible. The safety benefit is decisive.

### 6.3 Caching Strategy

The existing sidecar cache (`wavecraft-params.json`) already avoids subprocess spawn on subsequent `wavecraft start` invocations. No additional caching is needed.

**Hot-reload** cannot use the sidecar cache (the whole point is to extract **new** parameters from the rebuilt dylib). The subprocess overhead (~200ms) is invisible next to the Cargo build time (~2-30s).

**Update sidecar cache after hot-reload:** After successful subprocess extraction, `rebuild.rs` should write the updated params to the sidecar cache. This ensures the next `wavecraft start` uses the latest parameters without any extraction. This is already partially implemented at rebuild.rs via the import of `write_sidecar_cache`.

---

## 7. Cross-Platform Considerations

### 7.1 Process Lifecycle

| Concern | macOS / Linux | Windows |
|---------|--------------|---------|
| Spawn | `fork()` + `exec()` | `CreateProcess()` |
| Kill on timeout | `SIGKILL` via `child.kill()` | `TerminateProcess` via `child.kill()` |
| Dylib extension | `.dylib` / `.so` | `.dll` |
| `current_exe()` | `/path/to/wavecraft` | `C:\path\to\wavecraft.exe` |

`tokio::process` abstracts all platform differences. No `#[cfg]` blocks needed in the spawner.

### 7.2 Binary Resolution

`std::env::current_exe()` returns the path to the running `wavecraft` binary. This works correctly when:
- Installed via `cargo install wavecraft`
- Run via `cargo run --manifest-path cli/Cargo.toml`
- Run from a development build (`target/debug/wavecraft`)

**Edge case:** If `wavecraft` is invoked via a symlink, `current_exe()` resolves to the actual binary, not the symlink. This is correct behavior — the subprocess needs the actual binary.

### 7.3 Environment Inheritance

The subprocess inherits the parent's environment by default (`tokio::process::Command`). This is important because:
- `DYLD_LIBRARY_PATH` / `LD_LIBRARY_PATH` may be needed for dylib resolution
- `PATH` is needed for system library discovery

No environment manipulation is needed. The default inheritance is correct.

---

## 8. Security Considerations

### 8.1 Dylib Path Validation

The `extract-params` subcommand receives a dylib path as an argument. Validate:

1. **Path exists** — `dylib_path.exists()` before attempting `dlopen`
2. **Extension check** — Must be `.dylib`, `.so`, or `.dll` (prevents loading arbitrary files)
3. **No shell interpretation** — The path is passed as a direct argument to `Command`, not through a shell. No command injection risk.

### 8.2 Subprocess Trust

The subprocess runs the **same binary** as the parent. No external binary is invoked. The `current_exe()` approach ensures we don't accidentally invoke a different `wavecraft` binary from `PATH`.

### 8.3 Temp File Race Conditions

The hot-reload path copies the dylib to `/tmp/wavecraft_hotreload_{timestamp}.{ext}` before extraction. The timestamp provides uniqueness. The temp file is deleted after extraction. No TOCTOU risk — the subprocess opens the file by path, and the file is only deleted after the subprocess exits.

---

## 9. Testing Strategy

### 9.1 Unit Tests

**`extract_params` command handler:**
- Test with a mock dylib that exports `wavecraft_get_params_json` → verify JSON stdout
- Test with nonexistent dylib path → verify exit code 2
- Test with dylib missing `wavecraft_get_params_json` symbol → verify exit code 3

**`extract_params_subprocess()` spawner:**
- Test with a real `wavecraft` binary and a working test dylib → verify `Vec<ParameterInfo>` parsed correctly
- Test timeout: Use a test dylib that sleeps in its initializer → verify timeout error after configured duration
- Test non-zero exit: Use a dylib with invalid FFI output → verify error propagation

### 9.2 Integration Tests

**Full startup flow:**
```bash
# Generate test plugin
cargo run --manifest-path cli/Cargo.toml -- create TestPlugin \
  --output target/tmp/test-subprocess

# Delete sidecar cache (force extraction path)
rm -f target/tmp/test-subprocess/engine/target/debug/wavecraft-params.json

# Run wavecraft start — should succeed via subprocess extraction
cd target/tmp/test-subprocess
cargo run --manifest-path /path/to/wavecraft/cli/Cargo.toml -- start
# Verify: "Loaded N parameters" appears without hang
```

**Hot-reload flow:**
```bash
# With wavecraft start running, edit engine/src/lib.rs
# Verify: rebuild triggers, parameters extracted via subprocess, UI updates
```

**Timeout simulation:**
- Create a test dylib with a `#[ctor]` initializer that sleeps for 60s
- Run `wavecraft extract-params <test_dylib>` with a 5s timeout
- Verify: subprocess is killed, parent reports timeout error

### 9.3 CI Integration

Add to `cargo xtask ci-check`:
- Verify `wavecraft extract-params --help` is hidden (not in `wavecraft --help` output)
- Verify `wavecraft extract-params <nonexistent>` exits with code 2

Template validation (`template-validation.yml`):
- After `wavecraft create`, verify first `wavecraft start` succeeds (exercises the subprocess path with fresh project / no cache)

### 9.4 Manual Testing Checklist

| Scenario | Expected |
|----------|----------|
| First `wavecraft start` (no cache) | Parameters extracted via subprocess, sidecar written |
| Subsequent `wavecraft start` (cache exists) | Parameters loaded from cache (no subprocess) |
| Hot-reload: add parameter | Subprocess extracts, UI updates |
| Hot-reload: remove parameter | Subprocess extracts, UI updates |
| Deliberately slow dylib (sleep in init) | Timeout after 30s, clear error message |
| Corrupted dylib | Subprocess exits non-zero, parent reports error |

---

## 10. Migration Path

### 10.1 Backward Compatibility

The change is **fully backward compatible:**
- The `extract-params` subcommand is hidden — existing CLI usage is unaffected
- The sidecar cache format is unchanged
- The `_param-discovery` feature flag is unchanged
- The `PluginParamLoader` API is unchanged (still used by the subprocess internally)

### 10.2 Feature Flag

No feature flag is needed. The subprocess approach is strictly superior to in-process `dlopen` for parameter-only extraction. There is no scenario where the caller should prefer in-process loading for parameters.

The `audio-dev` feature continues to use in-process `PluginParamLoader::load()` for vtable loading, as documented in §3.4.

### 10.3 Rollout

1. Add `extract-params` hidden subcommand
2. Add `extract_params_subprocess()` function
3. Update `start.rs::load_parameters()` to call subprocess (non-`audio-dev` path)
4. Update `rebuild.rs::load_parameters_from_dylib()` to call subprocess
5. Remove the `spawn_blocking` + `catch_unwind` + `timeout` wrapper from `do_build()` (the subprocess spawner handles all of this)
6. Update `load_parameters()` to be async (or use `block_on` bridge)

All changes are in the CLI crate. No engine crate changes. No protocol changes. No UI changes.

---

## 11. Alternatives Considered

### 11.1 In-Process `dlopen` with Timeout (Current Approach)

**What it does:** `tokio::time::timeout` + `spawn_blocking` + `catch_unwind`

**Why insufficient:** The timeout fires, but the blocked thread holding the `dlopen` call cannot be forcefully terminated. The thread remains hung in the kernel. On macOS, the library may be partially loaded, corrupting the process's address space. `catch_unwind` cannot catch a hang — it only catches Rust panics.

### 11.2 Separate Binary (`wavecraft-param-extract`)

**What it does:** Compile a small, dedicated binary that only does parameter extraction.

**Why not chosen:**
- Additional build target increases compile time
- Must be distributed alongside `wavecraft` CLI
- Version synchronization risk (binary version ≠ CLI version)
- Binary discovery problem (where is it installed?)
- Marginal benefit — the `wavecraft` binary is already small (~5MB)

### 11.3 `cargo run` a Script

**What it does:** Use `cargo run` to execute a Rust file that loads the dylib.

**Why not chosen:**
- Requires Cargo toolchain on PATH (may not be true in installed environments)
- Compilation overhead (seconds) on first invocation
- Unreliable in cross-compilation scenarios

### 11.4 Named Pipe / Unix Socket IPC

**What it does:** Use IPC instead of stdout for data transfer.

**Why not chosen:**
- Stdout is simpler, portable, and sufficient for the data volume (~1-10KB of JSON)
- Named pipes add platform-specific code (Windows named pipes vs Unix domain sockets)
- No performance benefit — the data is tiny

### 11.5 Shared Memory / mmap

**Why not chosen:** Massive overengineering for ~1-10KB of JSON. Introduces platform-specific code, synchronization complexity, and cleanup concerns.

---

## 12. File Changes Summary

| File | Change |
|------|--------|
| cli/src/main.rs | Add `ExtractParams` variant to `Commands` enum |
| cli/src/commands/extract_params.rs (new) | `extract-params` handler — `dlopen` + FFI + JSON stdout |
| cli/src/commands/mod.rs | Export new module |
| cli/src/project/param_extract.rs (new) | `extract_params_subprocess()` async spawner |
| cli/src/project/mod.rs | Export new module |
| cli/src/commands/start.rs | Replace `load_params_only()` calls with `extract_params_subprocess()` |
| cli/src/dev_server/rebuild.rs | Replace `load_parameters_from_dylib()` with async subprocess call; remove `spawn_blocking`/`catch_unwind`/`timeout` nesting |

**No changes to:** engine crates, protocol, UI, templates, build system, CI workflows.
