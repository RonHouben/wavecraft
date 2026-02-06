# Implementation Plan: Embedded Dev Server with Plugin Parameter Discovery

**Feature:** Embedded WebSocket dev server in `wavecraft` CLI  
**Version:** 0.8.0  
**Created:** 2026-02-06  
**Based on:** [Low-Level Design](./low-level-design-embedded-dev-server.md)

---

## Overview

This plan implements an embedded WebSocket server in the `wavecraft` CLI that:
1. Builds the user's plugin (`cargo build --lib`)
2. Loads the compiled dylib and extracts parameters via FFI
3. Runs an embedded WebSocket server using `wavecraft-bridge`
4. Spawns Vite dev server for React UI

This replaces the broken `cargo run -p standalone` approach that fails in user projects.

**Estimated effort:** 9-14 hours

---

## Requirements

- [Low-Level Design](./low-level-design-embedded-dev-server.md)
- [CLI Start Command User Stories](../cli-start-command/user-stories.md)

---

## Architecture Changes

| File | Change Type | Description |
|------|-------------|-------------|
| **SDK Changes** | | |
| `wavecraft-macros/src/plugin.rs` | **Modify** | Generate FFI exports in `wavecraft_plugin!` |
| `wavecraft-nih_plug/src/lib.rs` | **Modify** | Add `__internal` module with re-exports |
| **CLI Changes** | | |
| `cli/Cargo.toml` | **Modify** | Add tokio, libloading, wavecraft-bridge deps |
| `cli/src/main.rs` | **Modify** | Add `mod dev_server;` declaration |
| `cli/src/dev_server/mod.rs` | **New** | Module root, exports |
| `cli/src/dev_server/plugin_loader.rs` | **New** | dylib loading via libloading |
| `cli/src/dev_server/host.rs` | **New** | `DevServerHost` implementing `ParameterHost` |
| `cli/src/dev_server/ws_server.rs` | **New** | WebSocket server using tokio-tungstenite |
| `cli/src/dev_server/meter.rs` | **New** | Synthetic meter generator |
| `cli/src/commands/start.rs` | **Modify** | Replace `cargo run -p standalone` with embedded server |

---

## Implementation Steps

### Phase 1: SDK FFI Exports
**Goal:** Enable plugins to export parameter specifications via C ABI

#### Step 1.1: Add FFI function generation to wavecraft_plugin! macro
**File:** `engine/crates/wavecraft-macros/src/plugin.rs`
- Action: Add FFI exports to the `wavecraft_plugin_impl` function output
- Why: Plugins need to export `wavecraft_get_params_json()` and `wavecraft_free_string()` 
- Dependencies: None
- Risk: Medium (proc-macro changes require careful testing)

**Implementation details:**
```rust
// Add to the expanded token stream after existing code:

// FFI exports for parameter discovery (used by `wavecraft start`)
#[no_mangle]
pub extern "C" fn wavecraft_get_params_json() -> *mut ::std::ffi::c_char {
    let specs = <<__ProcessorType as #krate::Processor>::Params as #krate::ProcessorParams>::param_specs();
    
    let params: ::std::vec::Vec<#krate::__internal::ParameterInfo> = specs
        .iter()
        .map(|spec| {
            // Convert ParamSpec to ParameterInfo (JSON-serializable)
            #krate::__internal::param_spec_to_info(spec)
        })
        .collect();
    
    let json = #krate::__internal::serde_json::to_string(&params)
        .unwrap_or_else(|_| "[]".to_string());
    
    ::std::ffi::CString::new(json)
        .map(|s| s.into_raw())
        .unwrap_or(::std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn wavecraft_free_string(ptr: *mut ::std::ffi::c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = ::std::ffi::CString::from_raw(ptr);
        }
    }
}
```

#### Step 1.2: Create __internal module in wavecraft-nih_plug
**File:** `engine/crates/wavecraft-nih_plug/src/lib.rs`
- Action: Add `__internal` module with re-exports needed by generated code
- Why: Generated code needs access to serde_json and ParameterInfo without polluting public API
- Dependencies: Step 1.1
- Risk: Low

**Implementation details:**
```rust
/// Internal types used by generated code (not part of public API)
#[doc(hidden)]
pub mod __internal {
    pub use serde_json;
    pub use wavecraft_protocol::ParameterInfo;
    
    use wavecraft_dsp::ParamSpec;
    
    /// Convert ParamSpec to ParameterInfo for JSON serialization
    pub fn param_spec_to_info(spec: &ParamSpec) -> ParameterInfo {
        ParameterInfo {
            id: spec.id_suffix.to_string(),
            name: spec.name.to_string(),
            value: spec.default as f32,
            min: spec.range.min() as f32,
            max: spec.range.max() as f32,
            default: spec.default as f32,
            unit: spec.unit.map(|s| s.to_string()),
            group: spec.group.map(|s| s.to_string()),
        }
    }
}
```

#### Step 1.3: Add serde_json dependency to wavecraft-nih_plug
**File:** `engine/crates/wavecraft-nih_plug/Cargo.toml`
- Action: Add `serde_json = "1.0"` to dependencies
- Why: Required for JSON serialization of parameters
- Dependencies: None
- Risk: Low

#### Step 1.4: Test FFI exports work correctly
**File:** `engine/crates/wavecraft-nih_plug/src/lib.rs` (test module)
- Action: Add integration test that loads a test plugin and calls FFI
- Why: Ensure FFI calling convention is correct before CLI integration
- Dependencies: Steps 1.1-1.3
- Risk: Low

---

### Phase 2: CLI Plugin Loader
**Goal:** Enable CLI to load and query compiled plugins via FFI

#### Step 2.1: Add libloading dependency
**File:** `cli/Cargo.toml`
- Action: Add `libloading = "0.8"` dependency
- Why: Cross-platform dynamic library loading
- Dependencies: None
- Risk: Low

#### Step 2.2: Create dev_server module structure
**File:** `cli/src/dev_server/mod.rs`
- Action: Create module root with exports
- Why: Organizes embedded server code
- Dependencies: None
- Risk: Low

```rust
//! Embedded development server for `wavecraft start`.

mod host;
mod meter;
mod plugin_loader;
mod ws_server;

pub use host::DevServerHost;
pub use meter::MeterGenerator;
pub use plugin_loader::PluginLoader;
pub use ws_server::WsServer;
```

#### Step 2.3: Implement PluginLoader
**File:** `cli/src/dev_server/plugin_loader.rs`
- Action: Implement dylib loading and FFI calls
- Why: Core functionality for parameter discovery
- Dependencies: Steps 2.1, 2.2
- Risk: Medium (unsafe code, platform-specific paths)

**Key considerations:**
- Handle `.dylib` (macOS), `.so` (Linux), `.dll` (Windows)
- Proper error handling for missing symbols
- Memory management (call `wavecraft_free_string` after use)

#### Step 2.4: Implement find_plugin_dylib helper
**File:** `cli/src/dev_server/plugin_loader.rs`
- Action: Add function to locate compiled dylib in target/debug/
- Why: Need to find the correct file among build artifacts
- Dependencies: Step 2.3
- Risk: Low

**Implementation details:**
- Scan `target/debug/` for files with platform-specific extension
- Skip files in `deps/` subdirectory
- Return first matching library (user projects have single crate)

#### Step 2.5: Add unit tests for PluginLoader
**File:** `cli/src/dev_server/plugin_loader.rs`
- Action: Test error handling (missing file, invalid dylib, missing symbols)
- Why: Ensure robust error messages for users
- Dependencies: Step 2.3
- Risk: Low

---

### Phase 3: Embedded WebSocket Server
**Goal:** Implement WebSocket server using existing wavecraft-bridge

#### Step 3.1: Add async runtime dependencies
**File:** `cli/Cargo.toml`
- Action: Add tokio, tokio-tungstenite, futures-util dependencies
- Why: Required for async WebSocket server
- Dependencies: None
- Risk: Low

```toml
tokio = { version = "1", features = ["rt-multi-thread", "net", "sync", "macros", "signal", "time"] }
tokio-tungstenite = "0.24"
futures-util = "0.3"
```

#### Step 3.2: Add wavecraft-bridge dependency
**File:** `cli/Cargo.toml`
- Action: Add path dependency to wavecraft-bridge and wavecraft-protocol
- Why: Reuse existing IPC handler for protocol consistency
- Dependencies: None
- Risk: Low

```toml
wavecraft-bridge = { path = "../engine/crates/wavecraft-bridge" }
wavecraft-protocol = { path = "../engine/crates/wavecraft-protocol" }
```

#### Step 3.3: Implement DevServerHost
**File:** `cli/src/dev_server/host.rs`
- Action: Implement `ParameterHost` trait for in-memory parameter state
- Why: Bridge between loaded parameters and IpcHandler
- Dependencies: Step 3.2
- Risk: Low

**Key methods:**
- `new(params: Vec<ParameterInfo>)` - Initialize with params from plugin
- `get_parameter(&self, id: &str)` - Return param with current value
- `set_parameter(&self, id: &str, value: f32)` - Update in-memory state
- `get_all_parameters(&self)` - Return all params with current values
- `get_meter_frame(&self)` - Delegate to MeterGenerator (step 3.4)
- `request_resize(&self, width, height)` - Return false (no-op in dev mode)

#### Step 3.4: Implement MeterGenerator
**File:** `cli/src/dev_server/meter.rs`
- Action: Implement synthetic meter data generator
- Why: Provide realistic-looking meters for UI testing
- Dependencies: None
- Risk: Low

**Algorithm:**
- Base level: -12 dB
- Slow sine modulation (0.5 Hz, ±6 dB)
- Fast noise jitter (±2 dB)
- Peak = RMS + 3-5 dB offset
- Slight stereo difference

#### Step 3.5: Implement WsServer
**File:** `cli/src/dev_server/ws_server.rs`
- Action: Implement WebSocket server using tokio-tungstenite
- Why: Accept connections from browser UI
- Dependencies: Steps 3.1, 3.3
- Risk: Medium (async code, connection management)

**Key features:**
- Accept connections on configurable port
- Route messages through IpcHandler
- Broadcast meter frames at ~60 Hz
- Handle disconnection gracefully

#### Step 3.6: Add tracing dependency for logging
**File:** `cli/Cargo.toml`
- Action: Add tracing and tracing-subscriber
- Why: Consistent logging in async context
- Dependencies: None
- Risk: Low

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

---

### Phase 4: Integration
**Goal:** Wire up all components in start command

#### Step 4.1: Add dev_server module declaration
**File:** `cli/src/main.rs`
- Action: Add `mod dev_server;` declaration
- Why: Make dev_server module available
- Dependencies: Phase 3
- Risk: Low

#### Step 4.2: Update StartCommand to build plugin
**File:** `cli/src/commands/start.rs`
- Action: Add `cargo build --lib` step before loading dylib
- Why: Ensure plugin is compiled with latest code
- Dependencies: None
- Risk: Low

**Implementation:**
```rust
fn build_plugin(project: &ProjectMarkers, verbose: bool) -> Result<()> {
    println!("{} Building plugin...", style("→").cyan());
    
    let status = Command::new("cargo")
        .args(["build", "--lib"])
        .current_dir(&project.engine_dir)
        .stdout(if verbose { Stdio::inherit() } else { Stdio::null() })
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run cargo build")?;
    
    if !status.success() {
        anyhow::bail!(
            "Plugin build failed. Check the error messages above.\n\
             \n\
             Common causes:\n\
             - Syntax errors in your Rust code\n\
             - Missing dependencies in Cargo.toml\n\
             - Incompatible wavecraft SDK version"
        );
    }
    
    println!("{} Plugin built", style("✓").green());
    Ok(())
}
```

#### Step 4.3: Update run_dev_servers to use embedded server
**File:** `cli/src/commands/start.rs`
- Action: Replace `cargo run -p standalone` with embedded WebSocket server
- Why: Core change that fixes the broken behavior
- Dependencies: Steps 4.1, 4.2
- Risk: Medium (major control flow change)

**New flow:**
1. Build plugin
2. Load dylib and extract parameters
3. Create DevServerHost with parameters
4. Start WebSocket server (tokio runtime)
5. Start Vite dev server (subprocess)
6. Wait for Ctrl+C

#### Step 4.4: Update imports and cleanup
**File:** `cli/src/commands/start.rs`
- Action: Add imports for dev_server module, remove unused code
- Why: Clean up after refactoring
- Dependencies: Step 4.3
- Risk: Low

#### Step 4.5: Test end-to-end with browser
**Manual test:**
1. `cd myTestProject && wavecraft start`
2. Open http://localhost:5173
3. Verify parameters appear in UI
4. Verify meters are animating
5. Move slider, verify value updates

---

### Phase 5: Polish
**Goal:** Error messages, logging, documentation

#### Step 5.1: Add descriptive error messages
**File:** `cli/src/dev_server/plugin_loader.rs`
- Action: Add helpful error messages with recovery suggestions
- Why: Better developer experience when things go wrong
- Dependencies: Phase 4
- Risk: Low

**Error cases:**
- Missing dylib: "Plugin not found. Run `cargo build --lib` in engine/"
- Missing FFI symbols: "Plugin built with old SDK. Update wavecraft dependency."
- Invalid JSON: "Plugin returned invalid parameters. This is a SDK bug."

#### Step 5.2: Add verbose logging
**Files:** Multiple in `cli/src/dev_server/`
- Action: Add tracing::debug!() calls for troubleshooting
- Why: Help users diagnose issues with --verbose
- Dependencies: Step 3.6
- Risk: Low

#### Step 5.3: Update documentation
**Files:** `docs/guides/sdk-getting-started.md`, README.md
- Action: Document `wavecraft start` behavior
- Why: Users need to know how the dev server works
- Dependencies: Phase 4
- Risk: Low

#### Step 5.4: Update implementation-progress.md
**File:** `docs/feature-specs/cli-start-command/implementation-progress.md`
- Action: Update to reflect embedded server implementation
- Why: Keep feature docs in sync
- Dependencies: Phase 4
- Risk: Low

---

## Testing Strategy

### Unit Tests

| Test | File | Description |
|------|------|-------------|
| DevServerHost CRUD | `cli/src/dev_server/host.rs` | get/set parameter operations |
| MeterGenerator bounds | `cli/src/dev_server/meter.rs` | Output values in valid range |
| PluginLoader errors | `cli/src/dev_server/plugin_loader.rs` | Missing file, invalid dylib |

### Integration Tests

| Test | Description |
|------|-------------|
| FFI round-trip | Build test plugin, load, query params, verify JSON |
| IPC handler | Create DevServerHost, send JSON-RPC, verify response |

### Manual Tests

| Test | Steps | Expected |
|------|-------|----------|
| Basic flow | `wavecraft create test && cd test && wavecraft start` | Both servers start, UI loads |
| Parameter display | Open browser, check sliders | Plugin's actual params shown |
| Meter animation | Watch meters | Realistic animation ~60 Hz |
| Slider interaction | Move slider | Value updates, no errors |
| Graceful shutdown | Ctrl+C | Both servers stop cleanly |

### CI Validation

The template-validation workflow will verify:
1. Scaffold project with `wavecraft create`
2. Build plugin with `cargo build --lib`
3. Load dylib and query parameters (new test)
4. Start servers (timeout 5s to verify startup)

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| FFI ABI mismatch | Low | High | Use JSON serialization (ABI-stable) |
| dylib not found | Medium | Medium | Clear error message with path |
| tokio version conflicts | Low | Medium | Use same version as wavecraft-bridge |
| Windows paths | Medium | Low | macOS-first, defer Windows testing |

---

## Success Criteria

- [ ] `wavecraft start` works in user projects created by `wavecraft create`
- [ ] Browser UI shows actual plugin parameters (not mock data)
- [ ] Meters animate with synthetic data at ~60 Hz
- [ ] Parameter changes via UI sliders update in-memory state
- [ ] Graceful shutdown on Ctrl+C
- [ ] Error messages include recovery suggestions
- [ ] All existing CLI tests still pass

---

## Estimated Timeline

| Phase | Duration | Cumulative |
|-------|----------|------------|
| Phase 1: SDK FFI Exports | 1-2 hours | 1-2 hours |
| Phase 2: CLI Plugin Loader | 2-3 hours | 3-5 hours |
| Phase 3: WebSocket Server | 3-4 hours | 6-9 hours |
| Phase 4: Integration | 2-3 hours | 8-12 hours |
| Phase 5: Polish | 1-2 hours | 9-14 hours |

---

## Related Documents

- [Low-Level Design](./low-level-design-embedded-dev-server.md) — Architectural design
- [CLI Start Command](../cli-start-command/) — Original feature spec (being superseded)
- [High-Level Design](../../architecture/high-level-design.md) — Overall architecture
- [Coding Standards](../../architecture/coding-standards.md) — Implementation conventions
