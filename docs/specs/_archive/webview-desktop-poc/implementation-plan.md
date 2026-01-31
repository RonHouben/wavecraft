# Implementation Plan: WebView Desktop POC

## Overview

This plan implements Milestone 2: a standalone desktop application that embeds a React UI via wry WebView, demonstrates bidirectional IPC communication, and validates the architecture for future plugin integration. The POC will establish patterns for parameter synchronization, asset embedding, and cross-platform desktop window management.

## Requirements

**Must Have:**
- Minimal React app (Vite + TypeScript) with modern tooling
- React app embedded in Rust desktop window via `wry`
- Bidirectional IPC bridge (JSON-RPC style) with clear message contracts
- `setParameter` / `getParameter` roundtrip verified
- Message latency measured and documented (target: < 5ms)
- Static assets bundled into Rust binary
- macOS and Windows build verification

**Nice to Have:**
- Parameter change visualization with smooth animations
- Multiple parameter types (float, bool, enum)
- Error handling in IPC layer
- Hot-reload development workflow

**Out of Scope:**
- Plugin window integration (Milestone 3)
- SPSC ring buffers for metering (Milestone 3)
- Real DSP processing
- Code signing and notarization

## Architecture Changes

| Change | Files/Crates | Description |
|--------|--------------|-------------|
| Extend protocol crate | [engine/crates/protocol/src/ipc.rs](../../engine/crates/protocol/src/ipc.rs) | Add IPC message contracts |
| New bridge crate | [engine/crates/bridge/](../../engine/crates/bridge/) | IPC handler and message dispatch |
| New desktop crate | [engine/crates/desktop/](../../engine/crates/desktop/) | wry window, asset embedding, app state |
| New ui directory | [ui/](../../ui/) | React SPA with Vite + TypeScript |
| Workspace updates | [engine/Cargo.toml](../../engine/Cargo.toml) | Add new crates to workspace |

---

## Implementation Steps

### Phase 1: Protocol Layer — IPC Message Contracts

**Goal:** Define type-safe IPC message structures shared between Rust and TypeScript.

---

#### Step 1.1: Add serde dependencies to protocol crate

**File:** [engine/crates/protocol/Cargo.toml](../../engine/crates/protocol/Cargo.toml)

**Action:** Add `serde` and `serde_json` dependencies for JSON serialization.

**Why:** IPC messages need JSON serialization for WebView communication.

**Dependencies:** None

**Risk:** Low

**Changes:**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

#### Step 1.2: Create IPC message contracts

**File:** [engine/crates/protocol/src/ipc.rs](../../engine/crates/protocol/src/ipc.rs) (NEW)

**Action:** Create JSON-RPC 2.0 style message types:
- `IpcRequest` — request with id, method, params
- `IpcResponse` — response with id, result or error
- `IpcNotification` — push notification with event, data
- `IpcError` — error with code, message, data
- Method-specific types: `GetParameterParams`, `GetParameterResult`, `SetParameterParams`, `ParameterInfo`, etc.
- Error code constants

**Why:** Type-safe contracts ensure Rust and TypeScript agree on message format.

**Dependencies:** Step 1.1

**Risk:** Low

---

#### Step 1.3: Re-export IPC module from protocol lib

**File:** [engine/crates/protocol/src/lib.rs](../../engine/crates/protocol/src/lib.rs)

**Action:** Add `pub mod ipc;` and re-export key types.

**Why:** Makes IPC types accessible from other crates.

**Dependencies:** Step 1.2

**Risk:** Low

---

### Phase 2: Bridge Layer — IPC Handler

**Goal:** Create message dispatch layer that routes IPC requests to appropriate handlers.

---

#### Step 2.1: Create bridge crate structure

**File:** [engine/crates/bridge/Cargo.toml](../../engine/crates/bridge/Cargo.toml) (NEW)

**Action:** Create new crate with dependencies on `protocol` and `serde_json`.

**Why:** Separates IPC handling from desktop/plugin specifics.

**Dependencies:** Phase 1 complete

**Risk:** Low

---

#### Step 2.2: Define ParameterHost trait

**File:** [engine/crates/bridge/src/handler.rs](../../engine/crates/bridge/src/handler.rs) (NEW)

**Action:** Create `ParameterHost` trait with methods:
- `fn get_parameter(&self, id: &str) -> Option<ParameterInfo>`
- `fn set_parameter(&self, id: &str, value: f64, normalized: bool) -> Result<(), String>`
- `fn get_all_parameters(&self) -> Vec<ParameterInfo>`

**Why:** Abstracts parameter storage; reusable for both desktop POC and future plugin integration.

**Dependencies:** Step 2.1

**Risk:** Low

---

#### Step 2.3: Implement IpcHandler

**File:** [engine/crates/bridge/src/handler.rs](../../engine/crates/bridge/src/handler.rs)

**Action:** Create `IpcHandler<H: ParameterHost>` with:
- `pub fn handle_request(&self, request: IpcRequest) -> IpcResponse`
- Method dispatch: `getParameter`, `setParameter`, `getAllParameters`, `ping`
- Error handling for unknown methods and invalid params

**Why:** Central dispatcher for all IPC requests.

**Dependencies:** Step 2.2

**Risk:** Low

---

#### Step 2.4: Create bridge error types

**File:** [engine/crates/bridge/src/error.rs](../../engine/crates/bridge/src/error.rs) (NEW)

**Action:** Define bridge-specific error types using `thiserror`.

**Why:** Clean error propagation across crate boundaries.

**Dependencies:** Step 2.1

**Risk:** Low

---

#### Step 2.5: Create bridge lib.rs

**File:** [engine/crates/bridge/src/lib.rs](../../engine/crates/bridge/src/lib.rs) (NEW)

**Action:** Re-export handler, error, and protocol types.

**Why:** Clean public API surface.

**Dependencies:** Steps 2.3, 2.4

**Risk:** Low

---

#### Step 2.6: Write bridge unit tests

**File:** [engine/crates/bridge/src/handler.rs](../../engine/crates/bridge/src/handler.rs)

**Action:** Add `#[cfg(test)]` module with tests for:
- Get parameter success
- Get parameter not found
- Set parameter success
- Set parameter out of range
- Unknown method error
- Ping response

**Why:** Validates handler logic before UI integration.

**Dependencies:** Step 2.3

**Risk:** Low

---

### Phase 3: Desktop Application — Rust Side

**Goal:** Create standalone desktop app with wry WebView and embedded assets.

---

#### Step 3.1: Create desktop crate structure

**File:** [engine/crates/desktop/Cargo.toml](../../engine/crates/desktop/Cargo.toml) (NEW)

**Action:** Create new crate with dependencies:
- `bridge` (path)
- `protocol` (path)
- `wry = "0.47"`
- `tao = "0.30"`
- `include_dir = "0.7"`
- `serde_json = "1.0"`

**Why:** Desktop binary needs windowing, webview, and asset embedding.

**Dependencies:** Phase 2 complete

**Risk:** Low

---

#### Step 3.2: Update workspace Cargo.toml

**File:** [engine/Cargo.toml](../../engine/Cargo.toml)

**Action:** Add `bridge` and `desktop` to workspace dependencies section.

**Why:** Enables `cargo build -p desktop` and internal path dependencies.

**Dependencies:** Step 3.1

**Risk:** Low

---

#### Step 3.3: Implement AppState

**File:** [engine/crates/desktop/src/app.rs](../../engine/crates/desktop/src/app.rs) (NEW)

**Action:** Create `AppState` struct with:
- Atomic storage for parameters (gain, bypass, mix)
- `impl ParameterHost for AppState`
- Fixed-point conversion utilities for atomics

**Why:** Simulates plugin parameter state with thread-safe atomics.

**Dependencies:** Step 3.1

**Risk:** Low

---

#### Step 3.4: Create asset embedding module

**File:** [engine/crates/desktop/src/assets.rs](../../engine/crates/desktop/src/assets.rs) (NEW)

**Action:** Use `include_dir!` macro to embed `ui/dist/`:
- `pub fn get_asset(path: &str) -> Option<(&'static [u8], &'static str)>`
- MIME type inference from file extension

**Why:** Bundles React app into single binary; no external file dependencies.

**Dependencies:** Step 3.1, Phase 4 (for assets to embed)

**Risk:** Low

---

#### Step 3.5: Create IPC primitives JavaScript

**File:** [engine/crates/desktop/src/js/ipc-primitives.js](../../engine/crates/desktop/src/js/ipc-primitives.js) (NEW)

**Action:** Minimal JS injected by Rust:
- `window.__VSTKIT_IPC__` object with `postMessage`, `setReceiveCallback`, `_receive`
- Frozen and non-writable for security

**Why:** Provides minimal bridge between wry's native IPC and our TypeScript layer.

**Dependencies:** Step 3.1

**Risk:** Low

---

#### Step 3.6: Implement WebView setup

**File:** [engine/crates/desktop/src/webview.rs](../../engine/crates/desktop/src/webview.rs) (NEW)

**Action:** Create `run_app(state: Arc<AppState>)` that:
- Creates tao EventLoop and Window
- Sets up custom protocol handler (`vstkit://`) for embedded assets
- Configures IPC handler with `with_ipc_handler`
- Injects IPC primitives with `with_initialization_script`
- Runs event loop

**Why:** Core integration point between Rust backend and React frontend.

**Dependencies:** Steps 3.3, 3.4, 3.5

**Risk:** Medium — wry API specifics may require adjustment

---

#### Step 3.7: Create main entry point

**File:** [engine/crates/desktop/src/main.rs](../../engine/crates/desktop/src/main.rs) (NEW)

**Action:** Main function that:
- Creates `Arc<AppState>` with defaults
- Calls `run_app(state)`
- Handles CLI args (e.g., `--dev-url` for development mode)

**Why:** Binary entry point for `cargo run -p desktop`.

**Dependencies:** Step 3.6

**Risk:** Low

---

#### Step 3.8: Create desktop lib.rs

**File:** [engine/crates/desktop/src/lib.rs](../../engine/crates/desktop/src/lib.rs) (NEW)

**Action:** Re-export modules for testing.

**Why:** Enables `cargo test -p desktop`.

**Dependencies:** Steps 3.3-3.7

**Risk:** Low

---

### Phase 4: React UI — TypeScript Side

**Goal:** Create React SPA with typed IPC library and parameter controls.

---

#### Step 4.1: Initialize React project

**Files:** [ui/package.json](../../ui/package.json), [ui/tsconfig.json](../../ui/tsconfig.json), [ui/vite.config.ts](../../ui/vite.config.ts) (NEW)

**Action:** Create Vite + React + TypeScript project:
- React 18, TypeScript 5
- Vite with `base: './'` for relative paths
- Path alias `@vstkit/ipc`

**Why:** Modern toolchain with fast builds and HMR.

**Dependencies:** None

**Risk:** Low

---

#### Step 4.2: Create IPC types

**File:** [ui/src/lib/vstkit-ipc/types.ts](../../ui/src/lib/vstkit-ipc/types.ts) (NEW)

**Action:** TypeScript interfaces matching Rust protocol:
- `ParameterInfo`, `IpcError`, `ParameterChangedEvent`
- `VstKitIpcPrimitives` for injected global
- `declare global` for `window.__VSTKIT_IPC__`

**Why:** Type safety across the IPC boundary.

**Dependencies:** Step 4.1

**Risk:** Low

---

#### Step 4.3: Implement IpcBridge class

**File:** [ui/src/lib/vstkit-ipc/IpcBridge.ts](../../ui/src/lib/vstkit-ipc/IpcBridge.ts) (NEW)

**Action:** Singleton class wrapping injected primitives:
- Request/response correlation with unique IDs
- Promise-based `invoke<T>(method, params)`
- Event subscription `on<T>(event, callback)`
- Timeout handling (5s default)

**Why:** Layer 1 of IPC architecture; handles low-level communication.

**Dependencies:** Step 4.2

**Risk:** Low

---

#### Step 4.4: Implement ParameterClient class

**File:** [ui/src/lib/vstkit-ipc/ParameterClient.ts](../../ui/src/lib/vstkit-ipc/ParameterClient.ts) (NEW)

**Action:** Singleton class with typed methods:
- `getParameter(id: string): Promise<ParameterInfo>`
- `setParameter(id: string, value: number, normalized?: boolean)`
- `getAllParameters(): Promise<ParameterInfo[]>`
- `ping(): Promise<number>` (returns roundtrip ms)
- `onParameterChanged(callback)`

**Why:** Layer 2; provides typed API for application code.

**Dependencies:** Step 4.3

**Risk:** Low

---

#### Step 4.5: Implement React hooks

**File:** [ui/src/lib/vstkit-ipc/hooks.ts](../../ui/src/lib/vstkit-ipc/hooks.ts) (NEW)

**Action:** Functional hooks:
- `useParameter(id)` — returns `{ param, setValue, isLoading, error }`
- `useAllParameters()` — returns `{ params, isLoading }`
- `useLatencyMonitor(intervalMs)` — returns `{ latency, avg, max }`

**Why:** Layer 3; React integration with automatic state management.

**Dependencies:** Step 4.4

**Risk:** Low

---

#### Step 4.6: Create public exports

**File:** [ui/src/lib/vstkit-ipc/index.ts](../../ui/src/lib/vstkit-ipc/index.ts) (NEW)

**Action:** Export types, classes, and hooks.

**Why:** Clean `import { useParameter } from '@vstkit/ipc'` syntax.

**Dependencies:** Steps 4.2-4.5

**Risk:** Low

---

#### Step 4.7: Create ParameterSlider component

**File:** [ui/src/components/ParameterSlider.tsx](../../ui/src/components/ParameterSlider.tsx) (NEW)

**Action:** Range input component using `useParameter` hook:
- Displays name, value with unit
- Drag to change value
- Loading state

**Why:** Primary control for continuous parameters.

**Dependencies:** Step 4.5

**Risk:** Low

---

#### Step 4.8: Create ParameterToggle component

**File:** [ui/src/components/ParameterToggle.tsx](../../ui/src/components/ParameterToggle.tsx) (NEW)

**Action:** Button/checkbox for boolean parameters.

**Why:** Control for bypass-style parameters.

**Dependencies:** Step 4.5

**Risk:** Low

---

#### Step 4.9: Create LatencyMonitor component

**File:** [ui/src/components/LatencyMonitor.tsx](../../ui/src/components/LatencyMonitor.tsx) (NEW)

**Action:** Displays current, average, and max latency with color coding.

**Why:** Development tool for validating < 5ms target.

**Dependencies:** Step 4.5

**Risk:** Low

---

#### Step 4.10: Create App component

**File:** [ui/src/App.tsx](../../ui/src/App.tsx) (NEW)

**Action:** Main app component:
- Uses `useAllParameters` and `useLatencyMonitor`
- Renders header with latency monitor
- Renders parameter controls (gain, bypass, mix)
- Footer with milestone info

**Why:** Main UI layout.

**Dependencies:** Steps 4.7-4.9

**Risk:** Low

---

#### Step 4.11: Create entry points and styles

**Files:** [ui/src/main.tsx](../../ui/src/main.tsx), [ui/index.html](../../ui/index.html), [ui/src/styles/main.css](../../ui/src/styles/main.css) (NEW)

**Action:** 
- Standard React 18 `createRoot` setup
- HTML with viewport meta
- Basic CSS styling

**Why:** Required Vite entry points.

**Dependencies:** Step 4.10

**Risk:** Low

---

### Phase 5: Integration and Testing

**Goal:** Wire everything together and verify end-to-end functionality.

---

#### Step 5.1: Build React UI

**Action:** Run `cd ui && npm install && npm run build`

**Why:** Creates `ui/dist/` for embedding.

**Dependencies:** Phase 4 complete

**Risk:** Low

---

#### Step 5.2: Build desktop with embedded UI

**Action:** Run `cd engine && cargo build -p desktop --release`

**Why:** Embeds built React app into binary.

**Dependencies:** Step 5.1, Phase 3 complete

**Risk:** Medium — first integration point

---

#### Step 5.3: Manual testing on macOS

**Action:** Run `./target/release/vstkit-desktop` and verify:
- Window opens with UI
- Parameters display initial values
- Sliders change values
- Toggle works
- Latency < 5ms

**Why:** Primary development platform validation.

**Dependencies:** Step 5.2

**Risk:** Low

---

#### Step 5.4: Windows cross-compilation setup (if applicable)

**Action:** Test on Windows VM or cross-compile:
- Install WebView2 runtime if needed
- Verify same functionality

**Why:** Cross-platform requirement.

**Dependencies:** Step 5.3

**Risk:** Medium — Windows-specific issues possible

---

#### Step 5.5: Run all Rust tests

**Action:** `cd engine && cargo test -p protocol -p bridge -p desktop`

**Why:** Validates Rust components work correctly.

**Dependencies:** Step 5.2

**Risk:** Low

---

#### Step 5.6: Latency benchmarking

**Action:** 
- Run app and observe latency monitor
- Document p50, p95, p99 latencies
- Verify target: p95 < 5ms

**Why:** Key success criterion.

**Dependencies:** Step 5.3

**Risk:** Low — if target missed, investigate

---

### Phase 6: xtask Integration and Documentation

**Goal:** Add convenience commands and update project documentation.

---

#### Step 6.1: Add xtask desktop command

**File:** [engine/xtask/src/commands/desktop.rs](../../engine/xtask/src/commands/desktop.rs) (NEW)

**Action:** Add `cargo xtask desktop` command:
- `--release` flag for release build
- `--dev` flag to point at dev server
- Builds UI first, then desktop crate

**Why:** Simplified developer workflow.

**Dependencies:** Phase 5 complete

**Risk:** Low

---

#### Step 6.2: Update roadmap

**File:** [docs/roadmap.md](../../docs/roadmap.md)

**Action:** Mark Milestone 2 tasks as complete.

**Why:** Project tracking.

**Dependencies:** Phase 5 complete

**Risk:** Low

---

## Testing Strategy

### Unit Tests
- **Rust:** `engine/crates/bridge/src/handler.rs` — handler dispatch logic
- **Rust:** `engine/crates/protocol/src/ipc.rs` — serialization roundtrip
- **TypeScript:** `ui/src/lib/vstkit-ipc/__tests__/` — mock IPC tests

### Integration Tests
- End-to-end parameter get/set via IPC
- Error handling (unknown method, invalid params, parameter not found)
- Timeout behavior

### Manual Tests
| Test | Expected Result |
|------|-----------------|
| App launches | Window opens with UI |
| Parameters display | Gain=0dB, Bypass=off, Mix=100% |
| Slider drag | Value changes smoothly |
| Toggle click | Bypass toggles |
| Latency monitor | Shows < 5ms average |
| Window resize | UI scales properly |
| App close | Clean exit, no orphans |

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **wry IPC latency too high** | Low | High | Measure early; consider alternative if > 10ms |
| **WebView2 not installed (Windows)** | Medium | High | Document requirement; consider bundler |
| **Cross-platform rendering differences** | Medium | Medium | Test on all platforms; use web-safe CSS |
| **Asset embedding increases binary size** | Low | Low | Monitor size; consider compression if > 10MB |
| **Memory leaks at JS↔Rust boundary** | Medium | Medium | Profile with heap tools; ensure cleanup |

## Success Criteria

- [ ] React app builds: `cd ui && npm run build`
- [ ] Desktop app builds with embedded assets: `cargo build -p desktop --release`
- [ ] App launches and displays UI on macOS
- [ ] App launches and displays UI on Windows
- [ ] `getAllParameters` returns three parameters with correct metadata
- [ ] `setParameter` roundtrip completes within 5ms (p95)
- [ ] Parameter changes from UI persist
- [ ] Latency monitor shows metrics in real-time
- [ ] Unit tests pass: `cargo test -p bridge -p protocol`
- [ ] TypeScript compiles: `cd ui && npm run typecheck`
- [ ] Binary size < 20MB (release, stripped)

---

## Estimated Effort

| Phase | Estimated Time |
|-------|----------------|
| Phase 1: Protocol Layer | 2-3 hours |
| Phase 2: Bridge Layer | 3-4 hours |
| Phase 3: Desktop App | 4-6 hours |
| Phase 4: React UI | 4-6 hours |
| Phase 5: Integration | 2-3 hours |
| Phase 6: xtask + Docs | 1-2 hours |
| **Total** | **16-24 hours** |

---

## References

- [Low-Level Design Document](low-level-design-webview-desktop-poc.md)
- [High-Level Design](../../docs/architecture/high-level-design.md)
- [Coding Standards](../../docs/architecture/coding-standards.md)
- [wry Documentation](https://github.com/nicebusiness/nice-wry)
- [tao Documentation](https://github.com/nicebusiness/nice-tao)
