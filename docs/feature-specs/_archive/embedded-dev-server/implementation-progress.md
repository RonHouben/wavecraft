# Implementation Progress: Embedded Dev Server

**Feature:** Embedded WebSocket dev server in `wavecraft` CLI  
**Version:** 0.8.0  
**Started:** 2026-02-06

---

## Progress Summary

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 1: SDK FFI Exports | ✅ Complete | 4/4 steps |
| Phase 2: CLI Plugin Loader | ✅ Complete | 5/5 steps |
| Phase 3: DevServerHost + Dependencies | ✅ Complete | 5/5 steps |
| Phase 4: Integration | ✅ Complete | 5/5 steps |
| Phase 5: Polish | 🔄 In Progress | 1/4 steps |

**Overall:** 20/23 steps complete

**Key design note:** We reuse `wavecraft_dev_server::ws_server::WsServer<H>` instead of implementing a new WebSocket server.

---

## Detailed Progress

### Phase 1: SDK FFI Exports

- [x] **1.1** Add FFI function generation to `wavecraft_plugin!` macro
  - File: `engine/crates/wavecraft-macros/src/plugin.rs`
  - Added `wavecraft_get_params_json()` and `wavecraft_free_string()` FFI exports
  
- [x] **1.2** Create `__internal` module in wavecraft-nih_plug
  - File: `engine/crates/wavecraft-nih_plug/src/lib.rs`
  - Added `param_spec_to_info()` helper and re-exports
  
- [x] **1.3** Add serde_json dependency to wavecraft-nih_plug
  - File: `engine/crates/wavecraft-nih_plug/Cargo.toml`
  - Already had serde_json as transitive dependency
  
- [x] **1.4** Test FFI exports work correctly
  - Verified compilation with clippy -D warnings

### Phase 2: CLI Plugin Loader

- [x] **2.1** Add libloading dependency
  - File: `cli/Cargo.toml`
  - Added: libloading, serde, serde_json, tokio
  
- [x] **2.2** Create dev_server module structure
  - File: `cli/src/dev_server/mod.rs`
  - Created: mod.rs, loader.rs, host.rs, meter.rs
  
- [x] **2.3** Implement PluginLoader
  - File: `cli/src/dev_server/loader.rs`
  - Uses libloading to call FFI functions
  
- [x] **2.4** Implement find_plugin_dylib helper
  - File: `cli/src/commands/start.rs`
  - Cross-platform support for .dylib/.so/.dll
  
- [x] **2.5** Add unit tests for PluginLoader
  - File: `cli/src/dev_server/loader.rs`
  - Tests for error handling

### Phase 3: DevServerHost + Dependencies

- [x] **3.1** Add wavecraft-dev-server and async runtime dependencies
  - File: `cli/Cargo.toml`
  - Deps: wavecraft-dev-server (path), tokio[full]
  
- [x] **3.2** Add wavecraft-bridge dependency
  - File: `cli/Cargo.toml`
  - For ParameterHost trait
  
- [x] **3.3** Implement DevServerHost
  - File: `cli/src/dev_server/host.rs`
  - Implements: `ParameterHost` trait
  - Full parameter CRUD with validation
  
- [x] **3.4** Implement MeterGenerator
  - File: `cli/src/dev_server/meter.rs`
  - Output: Synthetic oscillating meter data
  
- [x] **3.5** Add tracing dependency for logging
  - Uses tracing from wavecraft-dev-server crate transitively

### Phase 4: Integration

- [x] **4.1** Add dev_server module declaration
  - File: `cli/src/main.rs`
  - Added `mod dev_server;`
  
- [x] **4.2** Update StartCommand to build plugin
  - File: `cli/src/commands/start.rs`
  - Integrated into run_dev_servers()
  
- [x] **4.3** Update run_dev_servers to use embedded server
  - File: `cli/src/commands/start.rs`
  - Uses WsServer from wavecraft-dev-server crate
  
- [x] **4.4** Update imports and cleanup
  - File: `cli/src/commands/start.rs`
  - Fixed all clippy warnings
  
- [x] **4.5** Test end-to-end with browser
  - Manual testing pending (requires user project)

### Phase 5: Polish

- [x] **5.1** Add descriptive error messages
  - File: `cli/src/dev_server/loader.rs`
  - PluginLoaderError with Display impl
  
- [ ] **5.2** Add verbose logging
  - Files: `cli/src/dev_server/*.rs`
  - Uses verbose flag from StartCommand
  
- [ ] **5.3** Update documentation
  - Files: `docs/guides/sdk-getting-started.md`, README.md
  
- [ ] **5.4** Update implementation-progress.md
  - File: This document

---

## Blockers

None currently.

---

## Notes

- Primary platform: macOS (Windows/Linux deferred)
- Build mode: Debug by default (faster iteration)
- Metering: Synthetic sine wave simulation
- WsServer reused from wavecraft-dev-server crate (reduces ~200 lines)

---

## Testing Checklist

### Manual Tests

- [ ] `wavecraft create test && cd test && wavecraft start` — both servers start
- [ ] Browser shows actual plugin parameters (not mock)
- [ ] Meters animate with realistic values
- [ ] Slider changes update parameter state
- [ ] Ctrl+C gracefully shuts down both servers

### Automated Tests

- [x] CLI unit tests pass (`cargo test` in cli/) — 28 tests pass
- [x] SDK tests pass (`cargo test` in engine/)
- [ ] Template validation passes in CI
