# Low-Level Design: Embedded Dev Server with Plugin Parameter Discovery

**Feature:** Embedded WebSocket dev server in `wavecraft` CLI  
**Version:** 0.8.0  
**Created:** 2026-02-06  
**Author:** Architect Agent

---

## Problem Statement

The `wavecraft start` command currently tries to run `cargo run -p standalone`, which:
1. Only exists in the SDK monorepo, not in user projects created by `wavecraft create`
2. Results in "package standalone not found" error for all user projects
3. Leaves the UI unable to connect (no WebSocket server running)

This is a **critical architectural gap** that breaks the core developer experience.

---

## Goals

| Goal | Priority |
|------|----------|
| `wavecraft start` works in user projects | Critical |
| Real plugin parameters displayed in dev UI | High |
| Consistent IPC behavior (dev = production) | High |
| Fast iteration (debug builds by default) | Medium |
| Minimal binary size impact | Low |

## Non-Goals

- Real audio processing in dev mode (no DSP execution)
- Hot-reload of parameter definitions (future enhancement)
- Windows support in initial implementation (macOS first)

---

## Reuse Analysis

Before designing new components, we analyzed existing crates in the SDK monorepo for reuse opportunities.

### Existing Components in `engine/crates/`

| Crate | Component | Reusable? | Notes |
|-------|-----------|-----------|-------|
| `standalone` | `WsServer<H: ParameterHost>` | ✅ **Yes** | Generic WebSocket server, fully tested |
| `standalone` | `AppState` (ParameterHost impl) | ❌ No | Hardcoded 3 params; CLI needs dynamic params |
| `wavecraft-bridge` | `IpcHandler<H>` | ✅ **Yes** | JSON-RPC handler (already in WsServer) |
| `wavecraft-bridge` | `ParameterHost` trait | ✅ **Yes** | Define our `DevServerHost` impl |
| `wavecraft-protocol` | `ParameterInfo`, `MeterFrame` | ✅ **Yes** | Protocol types |

### What We Reuse vs. Build New

| Component | Source | Rationale |
|-----------|--------|-----------|
| `WsServer<H>` | Reuse from `standalone` | Production-tested, has logging, shutdown, verbose mode |
| `IpcHandler<H>` | Reuse from `wavecraft-bridge` | Already integrated in WsServer |
| `ParameterHost` trait | Reuse from `wavecraft-bridge` | Standard interface |
| `DevServerHost` | **New in CLI** | Needs dynamic params from FFI + synthetic metering |
| `PluginLoader` | **New in CLI** | FFI dylib loading (libloading) |
| `MeterGenerator` | **New in CLI** | Synthetic meter data for dev UX |

This reduces implementation effort by ~40% (no need to write/test WebSocket handling).

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    EMBEDDED DEV SERVER ARCHITECTURE                     │
└─────────────────────────────────────────────────────────────────────────┘

  User runs: wavecraft start

  ┌─────────────────────────────────────────────────────────────────────┐
  │                      wavecraft CLI binary                           │
  │                                                                     │
  │  ┌─────────────────────────────────────────────────────────────┐    │
  │  │  StartCommand::execute()                                    │    │
  │  │                                                             │    │
  │  │  1. Detect project (existing logic)                         │    │
  │  │  2. Check/install npm deps (existing logic)                 │    │
  │  │  3. Build user's plugin: cargo build --lib                  │    │
  │  │  4. Load dylib, extract parameters via FFI                  │    │
  │  │  5. Start WsServer<DevServerHost> (reuse from standalone)   │    │
  │  │  6. Start Vite dev server (spawn npm run dev)               │    │
  │  │  7. Wait for Ctrl+C, graceful shutdown                      │    │
  │  └─────────────────────────────────────────────────────────────┘    │
  │                                                                     │
  │  ┌─────────────────┐    ┌─────────────────┐    ┌────────────────┐   │
  │  │ PluginLoader    │───►│ DevServerHost   │───►│ IpcHandler     │   │
  │  │ (NEW: libloading)    │ (NEW: CLI impl) │    │ (reuse:        │   │
  │  │                 │    │                 │    │  wavecraft-    │   │
  │  └─────────────────┘    └─────────────────┘    │  bridge)       │   │
  │          │                      │              └───────┬────────┘   │
  │          │ FFI call             │ params               │            │
  │          ▼                      ▼                      ▼            │
  │  ┌─────────────────┐    ┌─────────────────┐    ┌────────────────┐   │
  │  │ Plugin dylib    │    │ In-memory state │    │ WsServer<H>    │   │
  │  │ (user's plugin) │    │ + MeterGenerator│    │ (reuse:        │   │
  │  └─────────────────┘    └─────────────────┘    │  standalone)   │   │
  │                                                └───────┬────────┘   │
  └─────────────────────────────────────────────────────────│───────────┘
                                                            │
                          ws://127.0.0.1:9000               │
                                                            ▼
                                                   ┌────────────────┐
                                                   │ Browser        │
                                                   │ React UI       │
                                                   │ (Vite HMR)     │
                                                   └────────────────┘
```

---

## Component Design

### 1. FFI Contract (Plugin → CLI)

The plugin exports two C ABI functions for parameter discovery:

```rust
/// Exported by user's plugin (generated by wavecraft_plugin! macro)
/// Returns JSON-serialized parameter specifications
#[no_mangle]
pub extern "C" fn wavecraft_get_params_json() -> *mut std::ffi::c_char;

/// Frees the string returned by wavecraft_get_params_json
#[no_mangle]
pub extern "C" fn wavecraft_free_string(ptr: *mut std::ffi::c_char);
```

**JSON Format:**
```json
[
  {
    "id": "input_gain_gain",
    "name": "Gain",
    "value": 0.0,
    "min": -60.0,
    "max": 24.0,
    "default": 0.0,
    "unit": "dB",
    "group": "Input"
  }
]
```

**Why JSON over repr(C) structs?**
- ABI stable across Rust compiler versions
- No struct alignment/padding issues
- Easy to extend without breaking compatibility
- Clear error messages on parse failures
- Already using JSON for IPC anyway

### 2. Plugin Loader (`cli/src/dev_server/plugin_loader.rs`)

```rust
use libloading::{Library, Symbol};
use std::ffi::{c_char, CStr};
use std::path::Path;
use wavecraft_protocol::ParameterInfo;

pub struct PluginLoader {
    library: Library,
}

impl PluginLoader {
    /// Load a plugin dylib from the given path
    pub fn load(dylib_path: &Path) -> Result<Self> {
        let library = unsafe { Library::new(dylib_path) }
            .context("Failed to load plugin library")?;
        Ok(Self { library })
    }

    /// Extract parameter specifications from the plugin
    pub fn get_parameters(&self) -> Result<Vec<ParameterInfo>> {
        unsafe {
            // Get function pointers
            let get_params: Symbol<unsafe extern "C" fn() -> *mut c_char> =
                self.library.get(b"wavecraft_get_params_json")?;
            let free_string: Symbol<unsafe extern "C" fn(*mut c_char)> =
                self.library.get(b"wavecraft_free_string")?;

            // Call FFI function
            let json_ptr = get_params();
            if json_ptr.is_null() {
                anyhow::bail!("Plugin returned null parameter data");
            }

            // Convert to Rust string
            let json_cstr = CStr::from_ptr(json_ptr);
            let json_str = json_cstr.to_str()
                .context("Plugin returned invalid UTF-8")?;

            // Parse JSON
            let params: Vec<ParameterInfo> = serde_json::from_str(json_str)
                .context("Failed to parse plugin parameters")?;

            // Free the C string
            free_string(json_ptr);

            Ok(params)
        }
    }
}
```

### 3. Dev Server Host (`cli/src/dev_server/host.rs`)

Implements `ParameterHost` trait from `wavecraft-bridge`. Integrates `MeterGenerator` for synthetic metering.

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use wavecraft_bridge::{BridgeError, ParameterHost};
use wavecraft_protocol::{MeterFrame, ParameterInfo, ParameterType};

use super::meter::MeterGenerator;

/// In-memory parameter host for dev server
///
/// Unlike `standalone::AppState` which has hardcoded parameters, this host
/// accepts dynamic parameters loaded from the user's plugin via FFI.
pub struct DevServerHost {
    /// Parameter specifications (from plugin FFI)
    params: Vec<ParameterInfo>,
    /// Current parameter values (mutable state)
    values: RwLock<HashMap<String, f32>>,
    /// Synthetic meter generator for dev UX
    meter_gen: MeterGenerator,
}

impl DevServerHost {
    pub fn new(params: Vec<ParameterInfo>) -> Self {
        // Initialize values with defaults
        let values: HashMap<String, f32> = params
            .iter()
            .map(|p| (p.id.clone(), p.default))
            .collect();

        Self {
            params,
            values: RwLock::new(values),
            meter_gen: MeterGenerator::new(),
        }
    }
}

impl ParameterHost for DevServerHost {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        let values = self.values.read().unwrap();
        self.params.iter().find(|p| p.id == id).map(|p| {
            let mut param = p.clone();
            if let Some(&val) = values.get(&p.id) {
                param.value = val;
            }
            param
        })
    }

    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
        if self.params.iter().any(|p| p.id == id) {
            let mut values = self.values.write().unwrap();
            values.insert(id.to_string(), value);
            Ok(())
        } else {
            Err(BridgeError::ParameterNotFound(id.to_string()))
        }
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        let values = self.values.read().unwrap();
        self.params
            .iter()
            .map(|p| {
                let mut param = p.clone();
                if let Some(&val) = values.get(&p.id) {
                    param.value = val;
                }
                param
            })
            .collect()
    }

    fn get_meter_frame(&self) -> Option<MeterFrame> {
        // Return synthetic meter data for dev UX
        Some(self.meter_gen.next_frame())
    }

    fn request_resize(&self, _width: u32, _height: u32) -> bool {
        // Dev server doesn't have a window to resize
        false
    }
}
```

### 4. WebSocket Server (Reuse from `standalone`)

**No new code needed.** We reuse `standalone::ws_server::WsServer<H>`:

```rust
// In cli/src/commands/start.rs
use standalone::ws_server::WsServer;
use wavecraft_bridge::IpcHandler;
use std::sync::Arc;

// Create parameter host with plugin data
let host = DevServerHost::new(params);

// Wrap in IpcHandler (same as standalone does)
let handler = Arc::new(IpcHandler::new(host));

// Reuse the existing WsServer
let server = WsServer::new(port, handler, verbose);
server.start().await?;
```

**Why this works:**
- `WsServer<H: ParameterHost>` is generic over any `ParameterHost` implementation
- `DevServerHost` implements `ParameterHost`
- All async runtime deps (tokio, tokio-tungstenite) are already in `standalone`

### 5. Synthetic Meter Generator (`cli/src/dev_server/meter.rs`)

Generates realistic-looking meter data for UI testing:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use wavecraft_protocol::MeterFrame;

/// Generates synthetic meter data for dev mode
pub struct MeterGenerator {
    start_time: Instant,
    frame_count: AtomicU64,
}

impl MeterGenerator {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            frame_count: AtomicU64::new(0),
        }
    }

    /// Generate next meter frame (synthetic sine wave simulation)
    pub fn next_frame(&self) -> MeterFrame {
        let frame = self.frame_count.fetch_add(1, Ordering::Relaxed);
        let elapsed = self.start_time.elapsed().as_secs_f64();

        // Simulate audio with slowly varying levels
        // Creates a pulsing effect that looks realistic
        let base_level = -12.0; // dB
        let modulation = 6.0 * (elapsed * 0.5).sin(); // Slow pulse
        let noise = 2.0 * ((elapsed * 7.3).sin() + (elapsed * 11.7).sin()) / 2.0; // Jitter

        let level_db = base_level + modulation + noise;
        
        // Slight stereo difference
        let left_db = level_db;
        let right_db = level_db + 0.5 * (elapsed * 0.3).sin();

        // Peak slightly higher than RMS
        let peak_offset = 3.0 + 2.0 * (elapsed * 2.1).sin().abs();

        MeterFrame {
            left_rms: db_to_linear(left_db),
            right_rms: db_to_linear(right_db),
            left_peak: db_to_linear(left_db + peak_offset),
            right_peak: db_to_linear(right_db + peak_offset),
            left_db,
            right_db,
            timestamp: frame,
        }
    }
}

fn db_to_linear(db: f64) -> f64 {
    10.0_f64.powf(db / 20.0)
}
```

---

## SDK Changes

### 1. Update `wavecraft_plugin!` Macro

The proc-macro must generate FFI exports alongside the existing plugin code.

**Location:** `engine/crates/wavecraft-macros/src/lib.rs`

```rust
// Add to the generated code in wavecraft_plugin! macro:

// FFI exports for parameter discovery (used by wavecraft start)
#[no_mangle]
pub extern "C" fn wavecraft_get_params_json() -> *mut std::ffi::c_char {
    use wavecraft::prelude::ProcessorParams;
    
    // Get parameter specs from the signal processor's params type
    let specs = <#signal_params_type as ProcessorParams>::param_specs();
    
    // Convert to ParameterInfo for JSON serialization
    let params: Vec<wavecraft::__internal::ParameterInfo> = specs
        .iter()
        .enumerate()
        .map(|(i, spec)| wavecraft::__internal::ParameterInfo {
            id: format!("{}_{}", #signal_name_snake, spec.id_suffix),
            name: spec.name.to_string(),
            value: spec.default,
            min: spec.range.min(),
            max: spec.range.max(),
            default: spec.default,
            unit: spec.unit.map(|s| s.to_string()),
            group: spec.group.map(|s| s.to_string()),
        })
        .collect();
    
    let json = serde_json::to_string(&params).unwrap_or_else(|_| "[]".to_string());
    std::ffi::CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn wavecraft_free_string(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(std::ffi::CString::from_raw(ptr));
        }
    }
}
```

### 2. Re-export Internal Types

**Location:** `engine/crates/wavecraft-nih_plug/src/lib.rs`

```rust
/// Internal types used by generated code (not part of public API)
#[doc(hidden)]
pub mod __internal {
    pub use wavecraft_protocol::ParameterInfo;
    pub use serde_json;
}
```

---

## CLI Changes

### 1. New Dependencies

**File:** `cli/Cargo.toml`

```toml
[dependencies]
# Existing deps...
clap = { version = "4.4", features = ["derive"] }
anyhow = "1.0"
console = "0.15"
ctrlc = "3"

# New: Dynamic library loading
libloading = "0.8"

# Reuse standalone crate for WebSocket server
# This brings in tokio, tokio-tungstenite, futures-util transitively
standalone = { path = "../engine/crates/standalone" }

# Reuse SDK crates for types
wavecraft-bridge = { path = "../engine/crates/wavecraft-bridge" }
wavecraft-protocol = { path = "../engine/crates/wavecraft-protocol" }

# Async runtime (needed for CLI orchestration)
tokio = { version = "1", features = ["rt-multi-thread", "macros", "signal"] }

# Logging (optional - standalone already uses tracing)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[target.'cfg(unix)'.dependencies]
nix = { version = "0.29", features = ["signal"] }
```

**Note:** `standalone` pulls in `tokio-tungstenite` and `futures-util` as transitive dependencies. We don't need to declare them directly.

**Binary Size Consideration:** `standalone` also pulls in `wry`/`tao` for the desktop app GUI (~2MB). If this becomes a concern, we can add a feature flag to `standalone` to exclude the GUI dependencies:

```toml
# Future optimization (not needed for v0.8.0)
[features]
default = ["gui"]
gui = ["wry", "tao"]
ws-server = []  # CLI only needs this
```

### 2. New Module Structure

```
cli/src/
├── main.rs
├── commands/
│   ├── mod.rs
│   ├── create.rs
│   └── start.rs          # Modified: use embedded dev server
├── project/
│   ├── mod.rs
│   └── detection.rs
├── dev_server/           # NEW (3 files, not 4)
│   ├── mod.rs
│   ├── host.rs           # DevServerHost impl (ParameterHost)
│   ├── meter.rs          # Synthetic meter generator
│   └── plugin_loader.rs  # dylib loading via libloading
└── template/
    ├── mod.rs
    └── variables.rs
```

**Key difference from prior design:** No `ws_server.rs` — we reuse `standalone::ws_server::WsServer<H>`.

### 3. Updated `start.rs`

Replace the `cargo run -p standalone` approach with embedded server:

```rust
// cli/src/commands/start.rs

use crate::dev_server::{DevServerHost, PluginLoader};
use standalone::ws_server::WsServer;  // Reuse from standalone crate
use wavecraft_bridge::IpcHandler;
use std::sync::Arc;

fn run_dev_servers(
    project: &ProjectMarkers,
    ws_port: u16,
    ui_port: u16,
    verbose: bool,
) -> Result<()> {
    println!("{}", style("Starting Wavecraft Development Servers").cyan().bold());

    // 1. Build user's plugin
    println!("{} Building plugin...", style("→").cyan());
    let build_status = Command::new("cargo")
        .args(["build", "--lib"])
        .current_dir(&project.engine_dir)
        .stdout(if verbose { Stdio::inherit() } else { Stdio::null() })
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run cargo build")?;

    if !build_status.success() {
        anyhow::bail!("Plugin build failed");
    }
    println!("{} Plugin built", style("✓").green());

    // 2. Find and load the dylib
    let dylib_path = find_plugin_dylib(&project.engine_dir)?;
    println!("{} Loading plugin from {:?}", style("→").cyan(), dylib_path);
    
    let loader = PluginLoader::load(&dylib_path)?;
    let params = loader.get_parameters()?;
    println!("{} Discovered {} parameters", style("✓").green(), params.len());

    if verbose {
        for p in &params {
            println!("    {} ({}: {} to {})", p.name, p.id, p.min, p.max);
        }
    }

    // 3. Start embedded WebSocket server (reusing standalone::ws_server)
    println!("{} Starting WebSocket server on port {}...", style("→").cyan(), ws_port);
    let host = DevServerHost::new(params);
    let handler = Arc::new(IpcHandler::new(host));
    let ws_server = WsServer::new(ws_port, handler, verbose);

    // Run WebSocket server in background
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        ws_server.start().await?;
        
        // 4. Start Vite dev server (in parallel with WS server)
        println!("{} Starting UI dev server on port {}...", style("→").cyan(), ui_port);
        let ui_server = Command::new("npm")
            .args(["run", "dev", "--", &format!("--port={}", ui_port)])
            .current_dir(&project.ui_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to start UI dev server")?;

        // Wait for Ctrl+C signal
        tokio::signal::ctrl_c().await?;
        println!("\n{} Shutting down...", style("→").cyan());

        // Kill UI server and clean up
        // ...

        Ok::<_, anyhow::Error>(())
    })?;

    Ok(())
}

/// Find the compiled plugin dylib
fn find_plugin_dylib(engine_dir: &Path) -> Result<PathBuf> {
    let target_dir = engine_dir.join("target/debug");
    
    // Look for .dylib (macOS), .so (Linux), or .dll (Windows)
    let extensions = if cfg!(target_os = "macos") {
        &["dylib"][..]
    } else if cfg!(target_os = "windows") {
        &["dll"][..]
    } else {
        &["so"][..]
    };

    for entry in std::fs::read_dir(&target_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if extensions.contains(&ext.to_str().unwrap_or("")) {
                // Skip deps folder artifacts
                if !path.to_string_lossy().contains("deps") {
                    return Ok(path);
                }
            }
        }
    }

    anyhow::bail!(
        "No plugin library found in {:?}. Supported extensions: {:?}",
        target_dir,
        extensions
    )
}
```

---

## Binary Size Impact

By reusing `standalone`, we get the WebSocket server code "for free" but inherit the full `standalone` dependency graph.

| Component | Estimated Size | Notes |
|-----------|----------------|-------|
| Current CLI | ~4 MB | |
| + libloading | +100 KB | New: FFI dylib loading |
| + standalone | +4-5 MB | Includes tokio, tungstenite, wry, tao |
| + wavecraft-bridge | +200 KB | Types, might already be in standalone |
| + wavecraft-protocol | +100 KB | Types, might already be in standalone |
| **Total** | **~8-10 MB** | |

**Trade-off analysis:**
- **Pros**: No code duplication, tested WsServer, faster implementation
- **Cons**: ~2-3 MB larger than writing a minimal WsServer (wry/tao overhead)

**Acceptable for v0.8.0.** If binary size becomes a concern:
1. Add feature flags to `standalone` to exclude `wry`/`tao` when only WsServer is needed
2. Or extract `WsServer` into a separate `wavecraft-ws-server` crate

---

## Error Handling

### Build Failures

```
error: Plugin build failed

The plugin failed to compile. Check the error messages above.
Common causes:
  - Syntax errors in your Rust code
  - Missing dependencies in Cargo.toml
  - Incompatible wavecraft SDK version

Run `cargo build --lib` in your engine/ directory for details.
```

### Missing FFI Exports

```
error: Plugin does not export wavecraft_get_params_json

This plugin was built with an older version of the Wavecraft SDK
that doesn't support parameter discovery.

To fix:
  1. Update your wavecraft dependency to the latest version
  2. Rebuild: cargo build --lib
```

### Invalid Parameter JSON

```
error: Failed to parse plugin parameters

The plugin returned invalid parameter data. This is likely a bug
in the Wavecraft SDK. Please report this issue.

Raw data: [truncated JSON...]
```

---

## Testing Strategy

### Unit Tests

| Test | Location | Coverage |
|------|----------|----------|
| `DevServerHost` crud operations | `cli/src/dev_server/host.rs` | get/set parameters |
| `MeterGenerator` output format | `cli/src/dev_server/meter.rs` | valid ranges, no panics |
| `PluginLoader` error handling | `cli/src/dev_server/plugin_loader.rs` | missing file, invalid dylib |

### Integration Tests

| Test | Description |
|------|-------------|
| Round-trip IPC | Create DevServerHost, send JSON-RPC via IpcHandler, verify response |
| Full stack (manual) | `wavecraft create`, `wavecraft start`, open browser, verify UI |

### CI Validation

The template validation workflow will catch regressions:
1. Scaffold project with `wavecraft create`
2. Build with `cargo build --lib`
3. Run `wavecraft start` (timeout after 5s to verify it starts)

---

## Implementation Phases

### Phase 1: SDK FFI Exports (1-2 hours)
- Update `wavecraft_plugin!` macro to generate FFI functions
- Add `__internal` module with re-exports
- Test with existing Wavecraft SDK plugin

### Phase 2: CLI Plugin Loader (2-3 hours)
- Add `libloading` dependency
- Implement `PluginLoader` with error handling
- Unit tests for FFI calling convention

### Phase 3: DevServerHost + MeterGenerator (1-2 hours)
- Add `standalone` as dependency for `WsServer` reuse
- Implement `DevServerHost` (ParameterHost trait)
- Implement `MeterGenerator` for synthetic meter data
- No WebSocket code needed — reuse `standalone::ws_server::WsServer`

### Phase 4: Integration (2-3 hours)
- Update `start.rs` to use embedded server
- Wire up build → load → serve pipeline
- Test end-to-end with browser

### Phase 5: Polish (1 hour)
- Verbose logging
- Error messages with recovery hints
- Update documentation

**Total estimated effort:** 7-11 hours (reduced ~25% by reusing WsServer)

---

## Open Questions Resolved

| Question | Decision | Rationale |
|----------|----------|-----------|
| Metering data | Synthetic sine wave simulation | Better dev experience than silence |
| Build mode | Debug by default | Faster iteration (~3s vs ~15s) |
| Hot reload params | Manual restart (v1) | Complex; defer to future enhancement |
| WebSocket server | Reuse `standalone::ws_server` | Avoid code duplication, tested implementation |

---

## Future Enhancements

1. **`--release` flag**: Build in release mode for performance testing
2. **Parameter hot reload**: Watch `src/` for changes, auto-rebuild and reload
3. **Audio file playback**: Play audio files through plugin for real metering
4. **Multiple clients**: Broadcast parameter changes to all connected browsers
5. **Feature-flag standalone**: Extract `WsServer` to reduce CLI binary size
