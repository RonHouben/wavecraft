# Implementation Plan: Subprocess-Based Parameter Extraction

## Related Documents

- [Subprocess Extraction Design](./subprocess-parameter-extraction-design.md) — Detailed architecture and rationale
- [Hot-Reload Low-Level Design](./low-level-design-rust-hot-reload.md) — Original hot-reload architecture
- [Hot-Reload Bugfix Plan](./hot-reload-param-update-bugfix-plan.md) — Investigation that identified the `dlopen` hang
- [High-Level Design](../../architecture/high-level-design.md) — Overall system architecture
- [Development Workflows](../../architecture/development-workflows.md) — Dev workflow documentation
- [Coding Standards (Rust)](../../architecture/coding-standards-rust.md) — Rust conventions

## Date

February 10, 2026

---

## Overview

This plan implements subprocess-based parameter extraction to resolve the `dlopen` hang that occurs during `wavecraft start` and hot-reload. The solution moves `dlopen` + FFI calls into a short-lived subprocess that can be forcefully terminated on timeout, completely isolating the parent process from macOS static initializer issues in nih-plug dependencies.

**Problem:** `wavecraft start` hangs indefinitely when calling `dlopen()` to load a plugin dylib for parameter extraction. The hang is caused by macOS `dyld` executing static initializers in nih-plug or its transitive dependencies that interact with system services.

**Solution:** Add a hidden `extract-params` subcommand to the `wavecraft` CLI that performs `dlopen` isolation. The parent process spawns `wavecraft extract-params <dylib_path>` as a subprocess, which can be killed on timeout.

---

## Requirements

### Functional Requirements

1. **Subprocess Isolation:** All `dlopen` calls for parameter extraction occur in a subprocess, never in the parent process
2. **Timeout Protection:** Subprocess is killed after 30s if it hangs
3. **Error Propagation:** All failure modes (dlopen failure, timeout, JSON parse errors) are clearly reported
4. **Backward Compatibility:** Sidecar cache format unchanged; existing CLI usage unaffected
5. **Audio-Dev Exception:** Keep in-process loading for `audio-dev` feature (vtable loading requires it)

### Non-Functional Requirements

1. **Performance:** Subprocess spawn overhead <200ms (negligible vs 2-30s Cargo build)
2. **Cross-Platform:** Works on macOS, Linux, Windows via `tokio::process`
3. **Safety:** Complete process isolation; OS reclaims all resources on timeout
4. **Maintainability:** No triple-nested timeout/spawn_blocking/catch_unwind wrappers

---

## Architecture Changes

### Component Overview

```
wavecraft start (parent process)
│
├── Initial Startup ────────────────────────────────┐
│   1. Try sidecar cache                            │
│   2. cargo build --lib --features _param-discovery │
│   3. Spawn subprocess for extraction ─────────────┼──┐
│   4. Parse JSON from stdout                       │  │
│   5. Write sidecar cache                          │  │
│                                                   │  │
├── Hot-Reload ─────────────────────────────────────┘  │
│   1. cargo build --lib --features _param-discovery    │
│   2. Copy dylib to temp path                         │
│   3. Spawn subprocess for extraction ────────────────┤
│   4. Parse JSON from stdout                          │
│   5. Update ParameterHost + notify UI                │
│                                                      │
│                                                      ▼
│                             ┌──────────────────────────────────┐
│                             │  wavecraft extract-params        │
│                             │  (subprocess — same binary)      │
│                             │                                  │
│                             │  1. dlopen(dylib_path)           │
│                             │  2. dlsym(wavecraft_get_params)  │
│                             │  3. Call FFI → get JSON          │
│                             │  4. Print JSON to stdout         │
│                             │  5. dlclose + exit               │
│                             │                                  │
│                             │  Lifecycle: spawn → work → exit  │
│                             │  If hangs: killed after timeout  │
│                             └──────────────────────────────────┘
```

### Exit Code Protocol

| Exit Code | Meaning | Parent Action |
|-----------|---------|--------------|
| 0 | Success — JSON on stdout | Parse stdout |
| 1 | General error | Report stderr to user |
| 2 | `dlopen` failure | Report with dylib path |
| 3 | Symbol not found | Suggest `_param-discovery` feature issue |
| 4 | JSON parse error | Report — likely FFI corruption |
| 5 | Invalid UTF-8 | Report — likely FFI corruption |
| — (killed) | Timed out / hung | Report timeout + dylib path |

---

## Implementation Steps

### Phase 1: Core Infrastructure

Build the subprocess command and spawner infrastructure without integrating into existing flows.

#### Step 1.1: Add ExtractParams Variant to Commands Enum

- **File:** `cli/src/main.rs`
- **Action:** Add hidden subcommand variant to the `Commands` enum
- **Why:** Provides CLI entry point for subprocess invocation
- **Dependencies:** None
- **Risk:** Low — isolated enum addition

**Details:**

Add to the `Commands` enum in `main.rs`:

```rust
#[derive(Parser)]
#[command(name = "wavecraft")]
#[command(about = "Wavecraft CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // ... existing variants (Create, Start, Update) ...

    /// Extract parameters from a plugin dylib (internal use)
    #[command(hide = true)]
    ExtractParams {
        /// Path to the plugin dylib
        dylib_path: PathBuf,
    },
}
```

Add the match arm in the `main()` function:

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // ... existing arms ...

        Commands::ExtractParams { dylib_path } => {
            commands::extract_params::execute(&dylib_path)
        }
    }
}
```

**Validation:**
- Compile check: `cargo check --manifest-path cli/Cargo.toml`
- Verify hidden: `cargo run --manifest-path cli/Cargo.toml -- --help` should NOT show `extract-params`
- Verify accessible: `cargo run --manifest-path cli/Cargo.toml -- extract-params --help` should work

---

#### Step 1.2: Create extract_params.rs Command Handler

- **File:** `cli/src/commands/extract_params.rs` (NEW)
- **Action:** Implement the `execute()` function that performs `dlopen` and outputs JSON
- **Why:** This is the subprocess's main logic — the only place `dlopen` occurs
- **Dependencies:** Step 1.1
- **Risk:** Medium — handles FFI, error cases must be comprehensive

**Details:**

Create `cli/src/commands/extract_params.rs`:

```rust
//! Hidden subcommand for extracting parameters from a plugin dylib.
//!
//! This runs in a separate subprocess to isolate `dlopen` from the parent process.
//! Outputs JSON to stdout on success; errors to stderr.

use anyhow::{Context, Result};
use std::path::Path;
use wavecraft_protocol::ParameterInfo;

/// Execute the extract-params subcommand.
///
/// Validates the dylib path, loads parameters via FFI, and prints JSON to stdout.
/// This is the ONLY place in the CLI that calls `dlopen` on user plugin dylibs.
pub fn execute(dylib_path: &Path) -> Result<()> {
    // Validate path exists
    if !dylib_path.exists() {
        anyhow::bail!(
            "Plugin dylib does not exist: {}",
            dylib_path.display()
        );
    }

    // Validate file extension
    let extension = dylib_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    if !matches!(extension, "dylib" | "so" | "dll") {
        anyhow::bail!(
            "Invalid dylib extension '{}'. Expected .dylib, .so, or .dll",
            extension
        );
    }

    // Load parameters — this is where dlopen happens
    let params = crate::project::dylib::PluginParamLoader::load_params_only(dylib_path)
        .with_context(|| {
            format!(
                "Failed to load plugin for parameter extraction: {}",
                dylib_path.display()
            )
        })?;

    // Serialize to compact JSON (no pretty-print — minimize stdout size)
    let json = serde_json::to_string(&params)
        .context("Failed to serialize parameters to JSON")?;

    // Output to stdout (parent will parse this)
    println!("{}", json);

    Ok(())
}
```

**Error handling:**
- `dlopen` errors exit with code 1 (general error, stderr contains details)
- Symbol not found exits with code 1 (propagated from `PluginParamLoader`)
- JSON serialization failure exits with code 1

Use `anyhow::bail!` for all errors — the CLI's `main()` already handles printing `anyhow::Error` to stderr and exiting non-zero.

**Validation:**
- Compile check: `cargo check --manifest-path cli/Cargo.toml`
- Manual test with nonexistent path:
  ```bash
  cargo run --manifest-path cli/Cargo.toml -- extract-params /nonexistent.dylib
  # Should exit 1 with error message
  ```
- Manual test with real dylib (after Step 1.5):
  ```bash
  # Generate test plugin first
  cargo run --manifest-path cli/Cargo.toml -- create TestPlugin --output target/tmp/test-extract
  cd target/tmp/test-extract/engine
  cargo build --lib --features _param-discovery
  
  # Test extraction
  cd ../../../../cli
  cargo run -- extract-params ../target/tmp/test-extract/engine/target/debug/libtest_plugin.dylib
  # Should print JSON array to stdout
  ```

---

#### Step 1.3: Export extract_params Module

- **File:** `cli/src/commands/mod.rs`
- **Action:** Add `pub mod extract_params;` to the module exports
- **Why:** Makes the new module accessible to `main.rs`
- **Dependencies:** Step 1.2
- **Risk:** Low — simple module export

**Details:**

Add to `cli/src/commands/mod.rs`:

```rust
pub mod create;
pub mod extract_params;  // NEW
pub mod start;
pub mod update;
```

**Validation:**
- Compile check: `cargo check --manifest-path cli/Cargo.toml`
- Verify accessible: `cargo run --manifest-path cli/Cargo.toml -- extract-params --help`

---

#### Step 1.4: Create param_extract.rs with Subprocess Spawner

- **File:** `cli/src/project/param_extract.rs` (NEW)
- **Action:** Implement `extract_params_subprocess()` async function
- **Why:** Provides the safe subprocess spawning + timeout logic
- **Dependencies:** Steps 1.1-1.3 (needs working `extract-params` subcommand)
- **Risk:** Medium — subprocess lifecycle, timeout handling, JSON parsing

**Details:**

Create `cli/src/project/param_extract.rs`:

```rust
//! Subprocess-based parameter extraction.
//!
//! Spawns `wavecraft extract-params <dylib_path>` to isolate `dlopen`
//! from the parent process, preventing hangs from macOS static initializers.

use anyhow::{Context, Result};
use std::path::Path;
use std::time::Duration;
use wavecraft_protocol::ParameterInfo;

/// Default timeout for subprocess parameter extraction (30 seconds).
pub const DEFAULT_EXTRACT_TIMEOUT: Duration = Duration::from_secs(30);

/// Extract parameters from a plugin dylib via a subprocess.
///
/// Spawns `wavecraft extract-params <dylib_path>` and parses JSON from stdout.
/// The subprocess is killed if it exceeds the timeout.
///
/// This isolates `dlopen` from the parent process, preventing hangs
/// caused by macOS static initializers in nih-plug dependencies.
///
/// # Arguments
/// * `dylib_path` - Path to the plugin dylib (must have _param-discovery feature)
/// * `timeout` - Maximum time to wait for extraction
///
/// # Returns
/// * `Ok(Vec<ParameterInfo>)` - Successfully extracted parameters
/// * `Err` - Timeout, process failure, or JSON parse error
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
        .with_context(|| {
            format!(
                "Failed to spawn parameter extraction subprocess: {}",
                self_exe.display()
            )
        })?;

    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)
                    .context("Subprocess stdout was not valid UTF-8")?;
                
                let params: Vec<ParameterInfo> = serde_json::from_str(stdout.trim())
                    .with_context(|| {
                        format!(
                            "Failed to parse parameter JSON from subprocess.\n\
                             stdout: {}\n\
                             stderr: {}",
                            stdout,
                            String::from_utf8_lossy(&output.stderr)
                        )
                    })?;
                
                Ok(params)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let code = output.status.code().unwrap_or(-1);
                anyhow::bail!(
                    "Parameter extraction failed (exit code {}):\n{}",
                    code,
                    stderr
                );
            }
        }
        Ok(Err(e)) => {
            anyhow::bail!("Failed to wait for extraction subprocess: {}", e);
        }
        Err(_elapsed) => {
            // Timeout — kill the subprocess
            let _ = child.kill().await;
            
            anyhow::bail!(
                "⚠ Parameter extraction timed out after {}s.\n\
                 \n\
                 This indicates a dlopen hang caused by static initializers in the plugin dylib.\n\
                 The plugin was built with --features _param-discovery but dlopen still hung.\n\
                 \n\
                 Suggested actions:\n\
                   • Check for new transitive dependencies that interact with macOS system services\n\
                   • Try: nm -gU {} | grep _init  (look for unexpected initializers)\n\
                   • File a bug with the offending dependency\n\
                 \n\
                 Dylib: {}",
                timeout.as_secs(),
                dylib_path.display(),
                dylib_path.display()
            );
        }
    }
}
```

**Key implementation notes:**

1. **`std::env::current_exe()`:** Resolves to the actual `wavecraft` binary, handling symlinks correctly
2. **`child.kill()`:** Sends `SIGKILL` (macOS/Linux) or calls `TerminateProcess` (Windows) — cannot be caught by a hung process
3. **JSON parsing:** Uses `stdout.trim()` to handle trailing newlines
4. **Error context:** Includes both stdout and stderr in parse errors for debugging

**Validation:**
- Compile check: `cargo check --manifest-path cli/Cargo.toml`
- Unit test (add to Step 3.2): Test with working dylib, nonexistent dylib, timeout simulation

---

#### Step 1.5: Export param_extract Module

- **File:** `cli/src/project/mod.rs`
- **Action:** Add `pub mod param_extract;` and re-export `extract_params_subprocess`
- **Why:** Makes the subprocess spawner accessible to `start.rs` and `rebuild.rs`
- **Dependencies:** Step 1.4
- **Risk:** Low — simple module export

**Details:**

Add to `cli/src/project/mod.rs`:

```rust
pub mod detection;
pub mod dylib;
pub mod param_extract;  // NEW

// Re-export commonly used items
pub use param_extract::{extract_params_subprocess, DEFAULT_EXTRACT_TIMEOUT};  // NEW
```

**Validation:**
- Compile check: `cargo check --manifest-path cli/Cargo.toml`
- Verify accessible from other modules: `cargo check --manifest-path cli/Cargo.toml`

---

### Phase 2: Integration

Replace in-process `dlopen` calls with subprocess spawning in startup and hot-reload flows.

#### Step 2.1: Convert load_parameters() to Async in start.rs

- **File:** `cli/src/commands/start.rs`
- **Action:** Make `load_parameters()` async and replace `PluginParamLoader::load_params_only()` with `extract_params_subprocess()`
- **Why:** Required for calling the async subprocess spawner
- **Dependencies:** Steps 1.1-1.5 (needs working subprocess infrastructure)
- **Risk:** Medium — changes async boundary, affects error propagation

**Details:**

Locate the `load_parameters()` function in `start.rs`. Current signature:

```rust
fn load_parameters(
    engine_dir: &Path,
    target_dir: &Path,
) -> Result<Vec<ParameterInfo>> {
    // ...
}
```

Change to async and replace the `PluginParamLoader::load_params_only()` call:

```rust
async fn load_parameters(
    engine_dir: &Path,
    target_dir: &Path,
) -> Result<Vec<ParameterInfo>> {
    use crate::project::{extract_params_subprocess, DEFAULT_EXTRACT_TIMEOUT};
    
    // Try sidecar cache first
    let sidecar_path = engine_dir
        .join("target")
        .join("debug")
        .join("wavecraft-params.json");
    
    if sidecar_path.exists() {
        if let Ok(cached) = std::fs::read_to_string(&sidecar_path) {
            if let Ok(params) = serde_json::from_str::<Vec<ParameterInfo>>(&cached) {
                println!("✓ Loaded {} parameters from cache", params.len());
                return Ok(params);
            }
        }
    }
    
    // Build with _param-discovery feature
    println!("Building plugin for parameter discovery...");
    let status = std::process::Command::new("cargo")
        .arg("build")
        .arg("--lib")
        .arg("--features")
        .arg("_param-discovery")
        .current_dir(engine_dir)
        .status()
        .context("Failed to run cargo build")?;
    
    if !status.success() {
        anyhow::bail!("cargo build failed");
    }
    
    // Find dylib
    let dylib_path = crate::project::dylib::find_plugin_dylib(target_dir)
        .context("Failed to find plugin dylib after build")?;
    
    // Extract via subprocess (replaces PluginParamLoader::load_params_only)
    let params = extract_params_subprocess(&dylib_path, DEFAULT_EXTRACT_TIMEOUT)
        .await
        .with_context(|| {
            format!(
                "Failed to extract parameters from plugin: {}",
                dylib_path.display()
            )
        })?;
    
    // Write sidecar cache
    if let Ok(json) = serde_json::to_string_pretty(&params) {
        let _ = std::fs::write(&sidecar_path, json);
    }
    
    println!("✓ Loaded {} parameters", params.len());
    Ok(params)
}
```

**Caller adjustment:** The `execute()` function in `start.rs` is already `async` (uses `#[tokio::main]`), so it can directly `.await` the now-async `load_parameters()`:

```rust
pub async fn execute(/* ... */) -> Result<()> {
    // ...
    let params = load_parameters(&engine_dir, &target_dir).await?;  // Add .await
    // ...
}
```

**Validation:**
- Compile check: `cargo check --manifest-path cli/Cargo.toml`
- Test fresh startup (no cache):
  ```bash
  cargo run --manifest-path cli/Cargo.toml -- create TestStartup --output target/tmp/test-startup
  cd target/tmp/test-startup
  # Delete cache to force subprocess path
  rm -f engine/target/debug/wavecraft-params.json
  cargo run --manifest-path /path/to/wavecraft/cli/Cargo.toml -- start
  # Should succeed, print "✓ Loaded N parameters"
  ```

---

#### Step 2.2: Update load_parameters_from_dylib() to Async in rebuild.rs

- **File:** `cli/src/dev_server/rebuild.rs`
- **Action:** Convert `load_parameters_from_dylib()` to async and replace in-process `PluginParamLoader` with subprocess
- **Why:** Hot-reload path must also use subprocess isolation
- **Dependencies:** Steps 1.1-1.5, Step 2.1 (infrastructure + pattern established)
- **Risk:** Medium — affects hot-reload critical path

**Details:**

Locate `load_parameters_from_dylib()` in `rebuild.rs`. Current signature:

```rust
fn load_parameters_from_dylib(engine_dir: PathBuf) -> Result<Vec<ParameterInfo>> {
    let lib_path = find_plugin_dylib(&engine_dir)
        .context("Failed to find plugin dylib after rebuild")?;

    let temp_path = create_temp_dylib_copy(&lib_path)?;

    let params = PluginParamLoader::load_params_only(&temp_path)
        .with_context(|| format!("Failed to extract params: {}", temp_path.display()))?;

    let _ = std::fs::remove_file(&temp_path);

    Ok(params)
}
```

Replace with async subprocess call:

```rust
async fn load_parameters_from_dylib(engine_dir: PathBuf) -> Result<Vec<ParameterInfo>> {
    use crate::project::{extract_params_subprocess, DEFAULT_EXTRACT_TIMEOUT};
    
    let lib_path = find_plugin_dylib(&engine_dir)
        .context("Failed to find plugin dylib after rebuild")?;

    let temp_path = create_temp_dylib_copy(&lib_path)?;

    // Extract via subprocess (replaces PluginParamLoader::load_params_only)
    let params = extract_params_subprocess(&temp_path, DEFAULT_EXTRACT_TIMEOUT)
        .await
        .with_context(|| {
            format!(
                "Failed to extract parameters from rebuilt plugin: {}",
                temp_path.display()
            )
        })?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    // Update sidecar cache (so next `wavecraft start` uses latest params)
    let sidecar_path = engine_dir
        .join("target")
        .join("debug")
        .join("wavecraft-params.json");
    if let Ok(json) = serde_json::to_string_pretty(&params) {
        let _ = std::fs::write(&sidecar_path, json);
    }

    Ok(params)
}
```

**Note:** This function is called from `do_build()` which currently uses `spawn_blocking`. That will be removed in Step 2.3.

**Validation:**
- Compile check: `cargo check --manifest-path cli/Cargo.toml`
- Test hot-reload (Step 2.5)

---

#### Step 2.3: Simplify do_build() by Removing Nested Wrappers

- **File:** `cli/src/dev_server/rebuild.rs`
- **Action:** Remove `spawn_blocking` + `catch_unwind` + triple-nested `timeout` wrappers from `do_build()`
- **Why:** The subprocess spawner handles all isolation, timeout, and safety — these wrappers are redundant
- **Dependencies:** Step 2.2 (async `load_parameters_from_dylib` must exist)
- **Risk:** Low — simplification reduces complexity

**Details:**

Locate the `do_build()` function in `rebuild.rs`. Current structure:

```rust
async fn do_build(/* ... */) -> Result<()> {
    // ... cargo build ...
    
    // Triple-nested: timeout → spawn_blocking → catch_unwind → load_parameters_from_dylib
    let params = tokio::time::timeout(
        Duration::from_secs(30),
        tokio::task::spawn_blocking(move || {
            std::panic::catch_unwind(|| {
                load_parameters_from_dylib(engine_dir_clone)
            })
            .map_err(|_| anyhow::anyhow!("Parameter extraction panicked"))
            .and_then(|res| res)
        })
    )
    .await
    .context("Parameter extraction timed out")??;
    
    // ... update ParameterHost ...
}
```

Replace with direct async call:

```rust
async fn do_build(/* ... */) -> Result<()> {
    // ... cargo build ...
    
    // Direct async call — subprocess handles timeout and isolation
    let params = load_parameters_from_dylib(engine_dir.clone())
        .await
        .context("Failed to extract parameters from rebuilt plugin")?;
    
    // ... update ParameterHost ...
}
```

**Rationale:** The `extract_params_subprocess()` function already:
- Applies timeout via `tokio::time::timeout`
- Isolates in a separate process (stronger than `spawn_blocking`)
- Handles panics via process exit codes (stronger than `catch_unwind`)

**Validation:**
- Compile check: `cargo check --manifest-path cli/Cargo.toml`
- Test hot-reload (Step 2.5)

---

#### Step 2.4: Preserve Audio-Dev In-Process Loading

- **File:** `cli/src/commands/start.rs` (or wherever `audio-dev` vtable loading occurs)
- **Action:** Verify `#[cfg(feature = "audio-dev")]` path still uses in-process `PluginParamLoader::load()` (not `load_params_only()`)
- **Why:** Vtable loading requires in-process `dlopen` — function pointers are only valid in the same address space
- **Dependencies:** Step 2.1 (must not break `audio-dev` path)
- **Risk:** Low — verification only, no code changes

**Details:**

Search for `#[cfg(feature = "audio-dev")]` blocks in `start.rs` or related files. Verify they use:

```rust
#[cfg(feature = "audio-dev")]
{
    // In-process load with vtable (correct)
    let plugin = PluginParamLoader::load(&dylib_path)?;
    // ... use plugin.processor ...
}

#[cfg(not(feature = "audio-dev"))]
{
    // Subprocess load (params only)
    let params = extract_params_subprocess(&dylib_path, DEFAULT_EXTRACT_TIMEOUT).await?;
}
```

If the `audio-dev` path was incorrectly changed to use `extract_params_subprocess()`, revert it to in-process loading.

**Document the exception:**

Add a comment near the `audio-dev` code block:

```rust
// IMPORTANT: audio-dev mode requires in-process dlopen to obtain a valid
// processor vtable. The vtable's function pointers are only valid within
// the same address space, so subprocess extraction is not suitable here.
// The existing timeout protection remains in place.
#[cfg(feature = "audio-dev")]
{
    // ...
}
```

**Validation:**
- Compile check with `audio-dev` feature: `cargo check --manifest-path cli/Cargo.toml --features audio-dev`
- If `audio-dev` tests exist, run them: `cargo test --manifest-path cli/Cargo.toml --features audio-dev`

---

#### Step 2.5: Test Non-Audio-Dev Path

- **File:** N/A (testing step)
- **Action:** End-to-end test of `wavecraft start` and hot-reload without `audio-dev` feature
- **Why:** Verify subprocess integration works in production flow
- **Dependencies:** Steps 2.1-2.4 (all integration changes complete)
- **Risk:** Low — verification only

**Details:**

**Test 1: Fresh startup (no cache)**

```bash
# Generate test plugin
cargo run --manifest-path cli/Cargo.toml -- create TestFresh --output target/tmp/test-fresh

# Delete sidecar cache to force subprocess path
rm -f target/tmp/test-fresh/engine/target/debug/wavecraft-params.json

# Start (should use subprocess)
cd target/tmp/test-fresh
cargo run --manifest-path /path/to/wavecraft/cli/Cargo.toml -- start

# Expected output:
# Building plugin for parameter discovery...
# ✓ Loaded N parameters
# Starting dev server on http://localhost:5173
```

**Test 2: Cached startup**

```bash
# Restart (should use cache, no subprocess)
cargo run --manifest-path /path/to/wavecraft/cli/Cargo.toml -- start

# Expected output:
# ✓ Loaded N parameters from cache  (note: "from cache")
# Starting dev server on http://localhost:5173
```

**Test 3: Hot-reload**

```bash
# With wavecraft start running, edit engine/src/lib.rs
# Add a new parameter:

#[derive(Params)]
struct MyPluginParams {
    // ... existing params ...

    #[id = "new_param"]
    pub new_param: FloatParam,
}

// In impl Default:
new_param: FloatParam::new("New Param", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),

# Save the file — should trigger rebuild

# Expected console output:
# [Hot-reload] Starting build...
# [Hot-reload] Build complete
# [Hot-reload] Loaded N parameters (N increased by 1)
# [Hot-reload] UI notified

# Verify UI updates (new parameter appears)
```

**Test 4: Timeout simulation (optional manual test)**

Create a test plugin with a slow initializer:

```rust
use ctor::ctor;
use std::time::Duration;

#[ctor]
fn slow_init() {
    std::thread::sleep(Duration::from_secs(60));
}
```

Build and extract:

```bash
cargo run --manifest-path cli/Cargo.toml -- extract-params <dylib_path>
# Should timeout after 30s with clear error message
```

**Validation:**
- All tests pass without hangs
- Error messages are clear
- Sidecar cache is written/used correctly
- UI updates on hot-reload

---

### Phase 3: Testing

Add comprehensive tests for the subprocess infrastructure.

#### Step 3.1: Unit Tests for extract_params Command

- **File:** `cli/tests/extract_params_command.rs` (NEW)
- **Action:** Add unit tests for the `extract-params` subcommand handler
- **Why:** Verify command validation, error codes, JSON output
- **Dependencies:** Steps 1.1-1.3 (command must exist)
- **Risk:** Low — testing only

**Details:**

Create `cli/tests/extract_params_command.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_extract_params_nonexistent_dylib() {
    let mut cmd = Command::cargo_bin("wavecraft").unwrap();
    cmd.arg("extract-params")
        .arg("/nonexistent/path.dylib")
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_extract_params_invalid_extension() {
    // Create temp file with wrong extension
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    
    let mut cmd = Command::cargo_bin("wavecraft").unwrap();
    cmd.arg("extract-params")
        .arg(temp_file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid dylib extension"));
}

#[test]
fn test_extract_params_help_is_hidden() {
    let mut cmd = Command::cargo_bin("wavecraft").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("extract-params").not());
}

#[test]
fn test_extract_params_works_with_help_flag() {
    let mut cmd = Command::cargo_bin("wavecraft").unwrap();
    cmd.arg("extract-params")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Path to the plugin dylib"));
}

// TODO: Add test with real dylib (requires test fixture)
// #[test]
// fn test_extract_params_success() {
//     // Build test plugin, run extract-params, verify JSON output
// }
```

**Dependencies:**
- `assert_cmd` crate for CLI testing
- `predicates` crate for assertions
- `tempfile` crate for temp files

Add to `cli/Cargo.toml` under `[dev-dependencies]`:

```toml
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.8"
```

**Validation:**
- Run tests: `cargo test --manifest-path cli/Cargo.toml --test extract_params_command`
- All tests pass

---

#### Step 3.2: Unit Tests for extract_params_subprocess()

- **File:** `cli/tests/param_extract_subprocess.rs` (NEW)
- **Action:** Add async tests for the subprocess spawner
- **Why:** Verify subprocess lifecycle, timeout, error propagation
- **Dependencies:** Steps 1.4-1.5 (spawner must exist)
- **Risk:** Medium — async testing, requires test fixtures

**Details:**

Create `cli/tests/param_extract_subprocess.rs`:

```rust
use anyhow::Result;
use std::time::Duration;
use wavecraft_cli::project::{extract_params_subprocess, DEFAULT_EXTRACT_TIMEOUT};

// Note: These tests require a real test plugin dylib fixture.
// Generate one in CI using: cargo run -- create TestFixture --output target/tmp/test-fixture

#[tokio::test]
async fn test_extract_params_nonexistent_dylib() {
    let result = extract_params_subprocess(
        std::path::Path::new("/nonexistent/path.dylib"),
        Duration::from_secs(5),
    )
    .await;
    
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("does not exist") || err_msg.contains("failed"));
}

#[tokio::test]
async fn test_extract_params_timeout() {
    // TODO: Create test dylib with slow initializer for timeout simulation
    // This requires a test fixture with ctor that sleeps for >5s
    // For now, skip this test
}

// #[tokio::test]
// async fn test_extract_params_success() {
//     // Generate test plugin
//     let test_plugin_dylib = Path::new("target/tmp/test-fixture/engine/target/debug/libtest_fixture.dylib");
//     
//     let params = extract_params_subprocess(test_plugin_dylib, DEFAULT_EXTRACT_TIMEOUT)
//         .await
//         .expect("Should extract parameters");
//     
//     assert!(!params.is_empty());
//     // Verify parameter structure
// }
```

**Dependencies:**
- `tokio-test` or `#[tokio::test]` macro
- Test fixture dylib

Add to `cli/Cargo.toml` under `[dev-dependencies]`:

```toml
tokio-test = "0.4"
```

**Validation:**
- Run tests: `cargo test --manifest-path cli/Cargo.toml --test param_extract_subprocess`
- Tests pass (or are marked `#[ignore]` if fixtures unavailable)

---

#### Step 3.3: Integration Test for Initial Startup

- **File:** `cli/tests/integration_startup.rs` (NEW)
- **Action:** End-to-end test of `wavecraft start` subprocess extraction
- **Why:** Verify first-time startup flow (no cache) uses subprocess
- **Dependencies:** Steps 2.1-2.5 (integration complete)
- **Risk:** Medium — requires full stack

**Details:**

Create `cli/tests/integration_startup.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_fresh_startup_uses_subprocess() {
    // Create test plugin
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("TestStartup");
    
    Command::cargo_bin("wavecraft")
        .unwrap()
        .arg("create")
        .arg("TestStartup")
        .arg("--output")
        .arg(&project_path)
        .assert()
        .success();
    
    // Delete sidecar cache to force subprocess path
    let sidecar = project_path.join("engine/target/debug/wavecraft-params.json");
    let _ = std::fs::remove_file(&sidecar);
    
    // Run wavecraft start (will timeout if hangs)
    // Note: Use timeout command or expect quick startup
    let mut cmd = Command::cargo_bin("wavecraft").unwrap();
    cmd.arg("start")
        .current_dir(&project_path)
        .timeout(std::time::Duration::from_secs(60))  // Full build + extraction
        .assert()
        .success();  // Should not hang
    
    // TODO: Verify output contains "✓ Loaded N parameters"
}

#[test]
fn test_cached_startup_skips_subprocess() {
    // Similar test but run twice — second run should be faster (cache hit)
}
```

**Validation:**
- Run test: `cargo test --manifest-path cli/Cargo.toml --test integration_startup`
- Test completes within 60s (no hang)

---

#### Step 3.4: Integration Test for Hot-Reload

- **File:** `cli/tests/integration_hotreload.rs` (NEW)
- **Action:** Test hot-reload parameter extraction via subprocess
- **Why:** Verify rebuild flow uses subprocess correctly
- **Dependencies:** Steps 2.1-2.5 (integration complete)
- **Risk:** High — requires running dev server, file watching

**Details:**

This test is complex because it requires:
1. Starting `wavecraft start` in the background
2. Editing a source file
3. Waiting for rebuild
4. Verifying parameters updated

**Simplified approach:** Test only the `load_parameters_from_dylib()` function directly:

```rust
use anyhow::Result;
use std::path::PathBuf;

#[tokio::test]
async fn test_load_parameters_from_dylib() {
    // Generate test plugin
    let project_dir = PathBuf::from("target/tmp/test-hotreload");
    // ... create project, build it ...
    
    // Call load_parameters_from_dylib directly
    let engine_dir = project_dir.join("engine");
    let params = wavecraft_cli::dev_server::rebuild::load_parameters_from_dylib(engine_dir)
        .await
        .expect("Should load parameters");
    
    assert!(!params.is_empty());
}
```

**Alternative:** Mark as manual test and document in test plan (see Step 3.7).

**Validation:**
- Run test: `cargo test --manifest-path cli/Cargo.toml --test integration_hotreload`
- OR mark as `#[ignore]` and test manually per Step 2.5

---

#### Step 3.5: Timeout Simulation Test

- **File:** `cli/tests/timeout_simulation.rs` (NEW)
- **Action:** Test subprocess timeout with slow initializer dylib
- **Why:** Verify timeout protection works correctly
- **Dependencies:** Steps 1.4-1.5 (spawner must exist)
- **Risk:** Medium — requires test fixture with slow init

**Details:**

This requires creating a test plugin with a `#[ctor]` that sleeps:

**Test fixture:** Create `test-fixtures/slow-init-plugin/src/lib.rs`:

```rust
use ctor::ctor;
use std::time::Duration;

#[ctor]
fn slow_init() {
    // Sleep longer than test timeout
    std::thread::sleep(Duration::from_secs(60));
}

// Minimal plugin implementation...
```

**Test:**

```rust
use wavecraft_cli::project::extract_params_subprocess;
use std::path::PathBuf;
use std::time::Duration;

#[tokio::test]
async fn test_subprocess_timeout() {
    // Build slow-init-plugin fixture
    // ...
    
    let slow_dylib = PathBuf::from("test-fixtures/slow-init-plugin/target/debug/libslow_init.dylib");
    
    let result = extract_params_subprocess(&slow_dylib, Duration::from_secs(5))
        .await;
    
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("timed out"));
    assert!(err.contains("30s") || err.contains("5s"));
}
```

**Alternative:** Mark as manual test (see Step 3.7) due to fixture complexity.

**Validation:**
- Test times out after 5s, not 60s
- Error message is clear

---

#### Step 3.6: CI Integration Checks

- **File:** `.github/workflows/template-validation.yml` (or equivalent CI config)
- **Action:** Add CI checks for subprocess functionality
- **Why:** Ensure PRs don't break subprocess extraction
- **Dependencies:** Steps 2.1-2.5, 3.1-3.5 (all code and tests exist)
- **Risk:** Low — CI configuration

**Details:**

Add to CI workflow:

```yaml
- name: Test subprocess extraction
  run: |
    # Verify extract-params is hidden
    cargo run --manifest-path cli/Cargo.toml -- --help > help.txt
    if grep -q "extract-params" help.txt; then
      echo "ERROR: extract-params should be hidden"
      exit 1
    fi
    
    # Verify extract-params --help works
    cargo run --manifest-path cli/Cargo.toml -- extract-params --help
    
    # Test fresh startup (no cache)
    cargo run --manifest-path cli/Cargo.toml -- create CITest --output target/tmp/ci-test
    cd target/tmp/ci-test
    rm -f engine/target/debug/wavecraft-params.json  # Force subprocess path
    
    # Run start command (should succeed without hang)
    timeout 120s cargo run --manifest-path /path/to/wavecraft/cli/Cargo.toml -- start &
    START_PID=$!
    sleep 10  # Let it start
    kill $START_PID || true
    
    echo "✓ Subprocess extraction works in CI"
```

**Validation:**
- CI pipeline passes on test branch
- `wavecraft start` completes within timeout

---

#### Step 3.7: Update Documentation

- **File:** `docs/feature-specs/rust-hot-reload/subprocess-parameter-extraction-design.md`
- **Action:** Add "Implementation Status" section
- **Why:** Track completion, note any deviations, document manual test procedures
- **Dependencies:** All previous steps
- **Risk:** Low — documentation only

**Details:**

Add to the design document:

```markdown
---

## 13. Implementation Status

**Completed:** [date]

### Changes Made

- Added hidden `extract-params` subcommand to CLI
- Implemented `extract_params_subprocess()` async spawner
- Replaced in-process `dlopen` with subprocess in:
  - `start.rs::load_parameters()` (initial startup)
  - `rebuild.rs::load_parameters_from_dylib()` (hot-reload)
- Removed triple-nested timeout/spawn_blocking/catch_unwind wrappers
- Preserved in-process loading for `audio-dev` feature (vtable requirement)

### Test Results

| Test | Status | Notes |
|------|--------|-------|
| Fresh startup (no cache) | ✅ Pass | Subprocess extracts params in ~200ms |
| Cached startup | ✅ Pass | Uses sidecar cache, no subprocess |
| Hot-reload | ✅ Pass | Subprocess extracts on every rebuild |
| Timeout simulation | ⚠ Manual | Tested with slow initializer, timeout works |
| `audio-dev` mode | ✅ Pass | In-process loading still works |

### Known Limitations

- `audio-dev` feature still uses in-process `dlopen` (required for vtable)
- Subprocess spawn overhead ~10-20ms (negligible vs ~2-30s build time)
- Timeout set to 30s (configurable via `DEFAULT_EXTRACT_TIMEOUT`)

### Migration Notes

No breaking changes. Existing `wavecraft start` invocations work unchanged.
```

**Also update:**

- `docs/architecture/development-workflows.md` — Add section on subprocess parameter extraction
- `cli/README.md` (if exists) — Mention hidden subcommand for maintainers

**Validation:**
- Run link checker: `./scripts/check-links.sh`
- Verify all document cross-references resolve

---

## Testing Strategy

### Unit Testing

1. **Command handler** (`extract_params.rs`):
   - Validate path validation (nonexistent file, wrong extension)
   - Test JSON output format with mock dylib

2. **Subprocess spawner** (`param_extract.rs`):
   - Test timeout mechanism with slow dylib
   - Test error propagation (exit codes, stderr capture)
   - Test JSON parsing

### Integration Testing

1. **Initial startup flow**:
   - Fresh project (no cache) → subprocess extraction → sidecar written
   - Cached project → cache used, no subprocess

2. **Hot-reload flow**:
   - Edit parameter → rebuild → subprocess extraction → UI update

3. **Error scenarios**:
   - Missing dylib → clear error message
   - Corrupted dylib → subprocess exits non-zero
   - Timeout → subprocess killed, timeout error reported

### Performance Testing

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Subprocess spawn | <20ms | `time` command around `extract-params` |
| Parameter extraction (success) | <200ms | End-to-end subprocess run |
| Timeout enforcement | ~30s | Test with slow dylib |

### Manual Testing Checklist

- [ ] First `wavecraft start` on new project succeeds (no hang)
- [ ] Subsequent `wavecraft start` uses cache (no subprocess)
- [ ] Hot-reload: add parameter → UI updates
- [ ] Hot-reload: remove parameter → UI updates
- [ ] Hot-reload: modify parameter range → UI updates
- [ ] Deliberately slow dylib → timeout error after 30s
- [ ] Verify `extract-params` is hidden from `wavecraft --help`
- [ ] Verify `extract-params --help` works for maintainers
- [ ] Test on macOS (primary platform)
- [ ] Test `audio-dev` mode still works (in-process)

---

## Risks & Mitigations

### Risk 1: Subprocess Spawn Overhead

**Risk:** Subprocess spawn + JSON parsing adds latency to every hot-reload.

**Likelihood:** Low  
**Impact:** Low (200ms is negligible vs 2-30s build)

**Mitigation:**
- Sidecar cache eliminates subprocess on subsequent startups
- subprocess overhead is measured and acceptable
- No optimization needed unless hot-reload rebuild time drops to <500ms

### Risk 2: Audio-Dev Mode Regression

**Risk:** Vtable loading breaks if accidentally switched to subprocess.

**Likelihood:** Medium (easy to overlook in refactor)  
**Impact:** High (audio-dev mode unusable)

**Mitigation:**
- Explicit `#[cfg(feature = "audio-dev")]` guards preserved
- Compile-time check: `cargo check --features audio-dev`
- Document exception in code comments
- Add test (if feasible) for audio-dev path

### Risk 3: Cross-Platform Subprocess Differences

**Risk:** Process spawning/killing behaves differently on Windows/Linux.

**Likelihood:** Low (tokio::process abstracts)  
**Impact:** Medium (some platforms unusable)

**Mitigation:**
- Use `tokio::process::Command` and `child.kill()` (cross-platform)
- No platform-specific code in spawner
- CI tests on Linux (macOS primary; Windows deprioritized but works)

### Risk 4: Timeout Too Short/Long

**Risk:** 30s is too short for slow builds or too long for fast failure detection.

**Likelihood:** Low  
**Impact:** Low (annoyance only)

**Mitigation:**
- `DEFAULT_EXTRACT_TIMEOUT` is a constant; easy to adjust
- Document current value (30s) in error messages
- Consider making configurable via env var in future (out of scope)

### Risk 5: JSON Parsing Errors

**Risk:** FFI returns corrupt JSON; subprocess succeeds but parent can't parse.

**Likelihood:** Low (FFI tested)  
**Impact:** Medium (parameter loading fails)

**Mitigation:**
- Subprocess exits non-zero if FFI fails
- Parent captures stderr for diagnostics
- Exit code 4 for JSON parse errors (distinct from other failures)

### Risk 6: Sidecar Cache Stale

**Risk:** Cache not updated after hot-reload → next startup uses old params.

**Likelihood:** Medium  
**Impact:** Medium (confusing UX — params revert on restart)

**Mitigation:**
- Step 2.2 includes cache update after hot-reload
- Test: hot-reload → stop → start → verify params match

---

## Success Criteria

### Must Have (MVP)

1. ✅ `wavecraft start` succeeds on first run (no cache, no hang)
2. ✅ Hot-reload succeeds on every rebuild (no hang)
3. ✅ Timeout protection works (subprocess killed after 30s)
4. ✅ Error messages are clear and actionable
5. ✅ Sidecar cache updated after hot-reload
6. ✅ `audio-dev` mode preserves in-process loading
7. ✅ No breaking changes to existing CLI usage
8. ✅ Subprocess overhead <200ms

### Should Have (Quality)

9. ✅ CI tests validate subprocess path
10. ✅ Documentation updated with implementation status
11. ✅ Manual test checklist completed

### Nice to Have (Future)

12. 🚧 Configurable timeout via env var (out of scope)
13. 🚧 Test fixture for timeout simulation (deferred to future)

---

## Performance Requirements

| Operation | Target | Actual | Status |
|-----------|--------|--------|--------|
| Subprocess spawn | <20ms | TBD | 🚧 Measure |
| Parameter extraction (success) | <200ms | TBD | 🚧 Measure |
| Timeout enforcement | ~30s | 30s | ✅ |
| First startup (no cache) | <60s | TBD | 🚧 Measure |
| Cached startup | <5s | TBD | 🚧 Measure |
| Hot-reload cycle | <30s | TBD | 🚧 Measure |

*Note: "Actual" values to be measured during Step 2.5 manual testing.*

---

## File Changes Summary

| File | Type | Lines Changed | Description |
|------|------|---------------|-------------|
| `cli/src/main.rs` | Modified | +10 | Add `ExtractParams` variant, match arm |
| `cli/src/commands/extract_params.rs` | New | +80 | Hidden subcommand handler |
| `cli/src/commands/mod.rs` | Modified | +1 | Export module |
| `cli/src/project/param_extract.rs` | New | +120 | Async subprocess spawner |
| `cli/src/project/mod.rs` | Modified | +3 | Export module and function |
| `cli/src/commands/start.rs` | Modified | +20 / -15 | Replace `load_params_only` with subprocess |
| `cli/src/dev_server/rebuild.rs` | Modified | +25 / -40 | Async, remove nested wrappers |
| `cli/tests/extract_params_command.rs` | New | +60 | Command validation tests |
| `cli/tests/param_extract_subprocess.rs` | New | +50 | Subprocess spawner tests |
| `cli/tests/integration_startup.rs` | New | +40 | End-to-end startup test |
| `cli/Cargo.toml` | Modified | +3 | Add test dependencies |
| Total | — | ~410 | 3 new modules, 4 modified |

**No changes required:**
- Engine crates (wavecraft-core, wavecraft-protocol, etc.)
- UI packages
- Build system (xtask, CI workflows)
- Templates

---

## Migration Path

### For Developers

No action required. Changes are fully backward compatible:
- Existing `wavecraft start` commands work unchanged
- Existing projects don't need updates
- Sidecar cache format unchanged

### For Maintainers

1. The `extract-params` subcommand is hidden but accessible for debugging:
   ```bash
   wavecraft extract-params <dylib_path>
   ```

2. Timeout can be adjusted in `cli/src/project/param_extract.rs`:
   ```rust
   pub const DEFAULT_EXTRACT_TIMEOUT: Duration = Duration::from_secs(30);
   ```

3. For troubleshooting hangs, check subprocess stderr:
   ```bash
   wavecraft extract-params <dylib> 2> error.log
   ```

### Rollback Plan

If subprocess approach causes unforeseen issues:

1. Revert commits for Steps 2.1-2.3 (caller changes)
2. Keep Steps 1.1-1.5 (infrastructure) — no harm if unused
3. Re-enable in-process `PluginParamLoader::load_params_only()` calls
4. File issue with diagnostics from subprocess stderr

**Safe to revert:** All changes are in CLI crate; no engine or protocol changes.

---

## Related Work

- [Hot-Reload Bugfix Plan](./hot-reload-param-update-bugfix-plan.md) — Investigation analyzing the `dlopen` hang root cause
- [Development Workflows](../../architecture/development-workflows.md) — Dev workflow documentation (to be updated with subprocess details)
- [Coding Standards (Rust)](../../architecture/coding-standards-rust.md) — Async conventions, error handling patterns

---

## Open Questions

1. **Should timeout be configurable via env var?**  
   → Out of scope for initial implementation. Add `WAVECRAFT_EXTRACT_TIMEOUT` support in future if needed.

2. **Should we cache parameter JSON in subprocess stdout for faster subsequent calls?**  
   → No. Sidecar cache already handles this. Subprocess is only invoked on cache miss or rebuild.

3. **Should `extract-params` subcommand accept `--timeout` flag?**  
   → No. Timeout is enforced by parent process, not subprocess. Subprocess should be unaware of timeout.

4. **Should we support multiple dylib paths in one subprocess call?**  
   → No. One dylib per subprocess keeps it simple. Startup only needs one dylib; hot-reload only needs one dylib.

---

## Appendix: Error Code Reference

| Exit Code | Meaning | When It Happens | Parent Response |
|-----------|---------|-----------------|-----------------|
| 0 | Success | JSON printed to stdout | Parse stdout as `Vec<ParameterInfo>` |
| 1 | General error | Validation failure, FFI error, JSON serialization error | Report stderr to user |
| 2 | Dylib load error | `dlopen()` failed | Report with dylib path, suggest checking dependencies |
| 3 | Symbol not found | `wavecraft_get_params_json` not in dylib | Suggest `_param-discovery` feature not enabled |
| 4 | JSON parse error | FFI returned invalid JSON | Report — likely plugin macro bug |
| 5 | UTF-8 error | FFI returned non-UTF-8 | Report — likely FFI corruption |

**Note:** Panics in subprocess result in non-zero exit (137 on Unix for SIGKILL-equivalent), treated as general error by parent.

---

## Appendix: Subprocess Lifecycle Diagram

```
Parent: wavecraft start
│
├─ load_parameters() called
│  ├─ Check sidecar cache → MISS
│  ├─ cargo build --features _param-discovery
│  └─ extract_params_subprocess(&dylib, 30s) ────┐
│                                                 │
│                                                 ▼
│                                     ┌────────────────────────────┐
│                                     │ Subprocess: wavecraft      │
│                                     │ extract-params <dylib>     │
│                                     │                            │
│                                     │ 1. Validate path           │
│                                     │ 2. dlopen(dylib)           │
│  ┌──────────────────────────────────┤    ↓                       │
│  │ If hangs here → timeout → kill   │    (may hang in dyld)      │
│  │                                   │ 3. dlsym(get_params_json)  │
│  │                                   │ 4. Call FFI                │
│  │                                   │ 5. Print JSON to stdout    │
│  │                                   │ 6. Exit 0                  │
│  │                                   └────────────────────────────┘
│  │                                                 │
│  │◄────────────────────────────────────────────────┘
│  │ Parse JSON from stdout
│  │ Write sidecar cache
│  └─ Return Vec<ParameterInfo>
│
└─ Continue startup with parameters
```

**Key Points:**
- Subprocess is completely separate — if it hangs, parent is unaffected
- `tokio::time::timeout` enforces 30s limit
- `child.kill()` sends `SIGKILL` (cannot be caught by hung process)
- OS reclaims all subprocess resources on kill

---

*End of Implementation Plan*
