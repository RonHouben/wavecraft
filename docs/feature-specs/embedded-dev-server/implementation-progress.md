# Implementation Progress: Embedded Dev Server

**Feature:** Embedded WebSocket dev server in `wavecraft` CLI  
**Version:** 0.8.0  
**Started:** 2026-02-06

---

## Progress Summary

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 1: SDK FFI Exports | ⬜ Not Started | 0/4 steps |
| Phase 2: CLI Plugin Loader | ⬜ Not Started | 0/5 steps |
| Phase 3: DevServerHost + Dependencies | ⬜ Not Started | 0/5 steps |
| Phase 4: Integration | ⬜ Not Started | 0/5 steps |
| Phase 5: Polish | ⬜ Not Started | 0/4 steps |

**Overall:** 0/23 steps complete

**Key design note:** We reuse `standalone::ws_server::WsServer<H>` instead of implementing a new WebSocket server.

---

## Detailed Progress

### Phase 1: SDK FFI Exports

- [ ] **1.1** Add FFI function generation to `wavecraft_plugin!` macro
  - File: `engine/crates/wavecraft-macros/src/plugin.rs`
  
- [ ] **1.2** Create `__internal` module in wavecraft-nih_plug
  - File: `engine/crates/wavecraft-nih_plug/src/lib.rs`
  
- [ ] **1.3** Add serde_json dependency to wavecraft-nih_plug
  - File: `engine/crates/wavecraft-nih_plug/Cargo.toml`
  
- [ ] **1.4** Test FFI exports work correctly
  - Compile test plugin, verify exports exist

### Phase 2: CLI Plugin Loader

- [ ] **2.1** Add libloading dependency
  - File: `cli/Cargo.toml`
  
- [ ] **2.2** Create dev_server module structure
  - File: `cli/src/dev_server/mod.rs`
  
- [ ] **2.3** Implement PluginLoader
  - File: `cli/src/dev_server/plugin_loader.rs`
  
- [ ] **2.4** Implement find_plugin_dylib helper
  - File: `cli/src/dev_server/plugin_loader.rs`
  
- [ ] **2.5** Add unit tests for PluginLoader
  - File: `cli/src/dev_server/plugin_loader.rs`

### Phase 3: DevServerHost + Dependencies

- [ ] **3.1** Add standalone and async runtime dependencies
  - File: `cli/Cargo.toml`
  - Deps: standalone (brings tokio/tungstenite transitively)
  
- [ ] **3.2** Add wavecraft-bridge dependency
  - File: `cli/Cargo.toml`
  
- [ ] **3.3** Implement DevServerHost
  - File: `cli/src/dev_server/host.rs`
  - Implements: `ParameterHost` trait
  - Note: No WsServer impl needed — reuse from standalone
  
- [ ] **3.4** Implement MeterGenerator
  - File: `cli/src/dev_server/meter.rs`
  - Output: Synthetic ~60 Hz meter data
  
- [ ] **3.5** Add tracing dependency for logging
  - File: `cli/Cargo.toml`

### Phase 4: Integration

- [ ] **4.1** Add dev_server module declaration
  - File: `cli/src/main.rs`
  
- [ ] **4.2** Update StartCommand to build plugin
  - File: `cli/src/commands/start.rs`
  - New function: `build_plugin()`
  
- [ ] **4.3** Update run_dev_servers to use embedded server
  - File: `cli/src/commands/start.rs`
  - Replace: `cargo run -p standalone` with embedded server
  
- [ ] **4.4** Update imports and cleanup
  - File: `cli/src/commands/start.rs`
  
- [ ] **4.5** Test end-to-end with browser
  - Manual test in user project

### Phase 5: Polish

- [ ] **5.1** Add descriptive error messages
  - File: `cli/src/dev_server/plugin_loader.rs`
  
- [ ] **5.2** Add verbose logging
  - Files: `cli/src/dev_server/*.rs`
  
- [ ] **5.3** Update documentation
  - Files: `docs/guides/sdk-getting-started.md`, README.md
  
- [ ] **5.4** Update implementation-progress.md
  - File: `docs/feature-specs/cli-start-command/implementation-progress.md`

---

## Blockers

None currently.

---

## Notes

- Primary platform: macOS (Windows/Linux deferred)
- Build mode: Debug by default (faster iteration)
- Metering: Synthetic sine wave simulation

---

## Testing Checklist

### Manual Tests

- [ ] `wavecraft create test && cd test && wavecraft start` — both servers start
- [ ] Browser shows actual plugin parameters (not mock)
- [ ] Meters animate with realistic values
- [ ] Slider changes update parameter state
- [ ] Ctrl+C gracefully shuts down both servers

### Automated Tests

- [ ] CLI unit tests pass (`cargo test` in cli/)
- [ ] SDK tests pass (`cargo test` in engine/)
- [ ] Template validation passes in CI
