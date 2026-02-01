# Implementation Plan: WebSocket IPC Bridge

## Overview

This plan breaks down the WebSocket IPC Bridge feature into actionable implementation steps, organized into 5 phases aligned with the low-level design.

**Target Version:** `0.3.0`  
**Estimated Effort:** 5-7 days

---

## Prerequisites

Before starting implementation:
- [ ] Ensure `main` branch is up-to-date
- [ ] Feature branch `feature/websocket-ipc-bridge` created
- [ ] Low-level design reviewed and approved

---

## Phase 0: Crate Rename (Preparation)

**Goal:** Rename `desktop` crate to `standalone` to better reflect its purpose as an industry-standard standalone plugin host for development.

### Step 0.1: Rename Crate Directory

- **Action:** Rename `engine/crates/desktop` → `engine/crates/standalone`
- **Why:** "Standalone" is the industry-standard term for plugin hosts that run outside a DAW
- **Dependencies:** None
- **Risk:** Low

```bash
cd engine/crates
mv desktop standalone
```

### Step 0.2: Update Cargo.toml Package Name

**File:** `engine/crates/standalone/Cargo.toml`

- **Action:** Update `[package] name` from `desktop` to `standalone`
- **Why:** Cargo package name must match crate directory
- **Dependencies:** Step 0.1
- **Risk:** Low

### Step 0.3: Update Workspace Members

**File:** `engine/Cargo.toml`

- **Action:** Update workspace dependencies reference from `desktop` to `standalone`
- **Why:** Workspace must reference the new crate name
- **Dependencies:** Step 0.1
- **Risk:** Low

### Step 0.4: Update Internal References

**Files:** Any files that reference `desktop` crate

- **Action:** Search and replace `desktop` → `standalone` in:
  - `use desktop::` imports
  - `cargo run -p desktop` commands in docs
  - Any other crate references
- **Why:** Ensure all references are consistent
- **Dependencies:** Steps 0.1-0.3
- **Risk:** Low
- **Verification:** `cargo build --workspace` passes

---

## Phase 1: WebSocket Server (Rust)

**Goal:** Add a WebSocket server to the standalone crate that can handle JSON-RPC messages.

### Step 1.1: Add Dependencies

**File:** `engine/crates/standalone/Cargo.toml`

- **Action:** Add tokio, tokio-tungstenite, futures-util, and clap dependencies
- **Why:** Required for async WebSocket server and CLI argument parsing
- **Dependencies:** None
- **Risk:** Low
- **Verification:** `cargo check -p standalone` passes

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "net", "sync", "macros", "time"] }
tokio-tungstenite = "0.24"
futures-util = "0.3"
clap = { version = "4", features = ["derive"] }
```

### Step 1.2: Create WebSocket Server Module

**File:** `engine/crates/standalone/src/ws_server.rs` (NEW)

- **Action:** Create new module with `WsServer` struct and basic connection handling
- **Why:** Core infrastructure for WebSocket transport
- **Dependencies:** Step 1.1
- **Risk:** Medium (tokio runtime integration)

**Implementation details:**
1. Define `WsServer` struct with `port`, `handler` (Arc), `clients` list
2. Implement `new()` constructor
3. Implement `start()` that binds to `127.0.0.1:{port}` and spawns accept loop
4. Implement `handle_connection()` that routes messages through `IpcHandler::handle_json()`
5. Add graceful shutdown via broadcast channel

### Step 1.3: Add CLI Arguments

**File:** `engine/crates/standalone/src/main.rs`

- **Action:** Add clap-based CLI argument parsing
- **Why:** Enable `--dev-server` and `--port` flags
- **Dependencies:** Step 1.1
- **Risk:** Low

**Implementation details:**
1. Add `Args` struct with clap derive macros
2. Add `--dev-server` bool flag
3. Add `--port` u16 flag with default 9000
4. Update `--help` output

### Step 1.4: Implement Dev Server Mode

**File:** `engine/crates/standalone/src/main.rs`

- **Action:** Add `run_dev_server()` function for headless WebSocket-only mode
- **Why:** Enable browser development without opening a window
- **Dependencies:** Steps 1.2, 1.3
- **Risk:** Medium

**Implementation details:**
1. Create tokio runtime
2. Start WebSocket server
3. Block on Ctrl+C signal for clean shutdown
4. Log connection events

### Step 1.5: Verify WebSocket Server

- **Action:** Manual test with `websocat` or similar tool
- **Why:** Ensure server accepts connections and handles JSON-RPC
- **Dependencies:** Step 1.4
- **Risk:** Low

**Verification commands:**
```bash
# Terminal 1
cargo run -p standalone -- --dev-server

# Terminal 2
websocat ws://127.0.0.1:9000
# Send: {"jsonrpc":"2.0","id":1,"method":"getAllParameters"}
# Expect: JSON response with parameters
```

---

## Phase 2: Transport Abstraction (TypeScript)

**Goal:** Create a transport interface and implementations for both native and WebSocket communication.

### Step 2.1: Create Transport Interface

**File:** `ui/src/lib/vstkit-ipc/transports/Transport.ts` (NEW)

- **Action:** Define `Transport` interface
- **Why:** Abstraction layer for swappable transports
- **Dependencies:** None
- **Risk:** Low

**Interface methods:**
- `send(request: string): Promise<string>`
- `onNotification(callback): () => void`
- `isConnected(): boolean`
- `dispose(): void`

### Step 2.2: Implement Native Transport

**File:** `ui/src/lib/vstkit-ipc/transports/NativeTransport.ts` (NEW)

- **Action:** Extract native IPC logic from `IpcBridge` into `NativeTransport` class
- **Why:** Encapsulate WKWebView-specific communication
- **Dependencies:** Step 2.1
- **Risk:** Low

**Implementation details:**
1. Wrap `globalThis.__VSTKIT_IPC__` primitives
2. Implement request/response matching via pending requests map
3. Implement notification callback routing
4. Always return `true` for `isConnected()` (native is always connected)

### Step 2.3: Implement WebSocket Transport

**File:** `ui/src/lib/vstkit-ipc/transports/WebSocketTransport.ts` (NEW)

- **Action:** Create WebSocket transport with reconnection logic
- **Why:** Enable browser-to-engine communication
- **Dependencies:** Step 2.1
- **Risk:** Medium

**Implementation details:**
1. Manage WebSocket connection lifecycle
2. Implement request/response matching by ID
3. Implement notification routing (messages without ID)
4. Add exponential backoff reconnection (1s, 2s, 4s, 8s, 16s)
5. Track connection status
6. Add status change callback

### Step 2.4: Create Transport Factory

**File:** `ui/src/lib/vstkit-ipc/transports/index.ts` (NEW)

- **Action:** Create factory function that selects transport based on environment
- **Why:** Automatic transport selection without manual configuration
- **Dependencies:** Steps 2.2, 2.3
- **Risk:** Low

**Implementation details:**
1. Module-level `IS_WEBVIEW` constant (evaluated once)
2. `getTransport()` async function (singleton pattern)
3. `hasTransport()` check function
4. Export all transport types

### Step 2.5: Add Transport Tests

**File:** `ui/src/lib/vstkit-ipc/transports/WebSocketTransport.test.ts` (NEW)

- **Action:** Unit tests for WebSocket transport
- **Why:** Ensure reconnection logic and message handling work correctly
- **Dependencies:** Step 2.3
- **Risk:** Low

**Test cases:**
1. Connection establishment
2. Request/response matching
3. Notification handling
4. Reconnection on disconnect
5. Max reconnection attempts

---

## Phase 3: Refactor IpcBridge

**Goal:** Update IpcBridge to use the transport abstraction and remove mock data.

### Step 3.1: Update IpcBridge to Use Transports

**File:** `ui/src/lib/vstkit-ipc/IpcBridge.ts`

- **Action:** Replace direct IPC primitive usage with transport abstraction
- **Why:** Enable both native and WebSocket communication through single interface
- **Dependencies:** Phase 2
- **Risk:** Medium (breaking change to internal architecture)

**Implementation details:**
1. Add `transport: Transport | null` property
2. Add `initPromise` for lazy async initialization
3. Update `initialize()` to call `getTransport()`
4. Update `invoke()` to use `transport.send()`
5. Subscribe to transport notifications in `doInitialize()`
6. Add `isConnected()` public method

### Step 3.2: Remove Mock Data

**File:** `ui/src/lib/vstkit-ipc/IpcBridge.ts`

- **Action:** Remove `getMockResponse()` method and mock data
- **Why:** No longer needed with WebSocket transport
- **Dependencies:** Step 3.1
- **Risk:** Low

**Changes:**
1. Delete `getMockResponse()` method entirely
2. Update `invoke()` to throw if no transport (instead of returning mock)
3. Update hooks to handle `null`/`undefined` gracefully

### Step 3.3: Add Connection Status Hook

**File:** `ui/src/lib/vstkit-ipc/useConnectionStatus.ts` (NEW)

- **Action:** Create hook for connection status monitoring
- **Why:** Enable UI to show connection state
- **Dependencies:** Step 3.1
- **Risk:** Low

**Implementation details:**
1. Return `{ connected: boolean, transport: 'native' | 'websocket' | 'none' }`
2. Poll connection status every 1 second
3. Clean up interval on unmount

### Step 3.4: Update Exports

**File:** `ui/src/lib/vstkit-ipc/index.ts`

- **Action:** Export new transports and hooks
- **Why:** Make new functionality available to consumers
- **Dependencies:** Steps 3.1, 3.3
- **Risk:** Low

**New exports:**
- `useConnectionStatus`
- `Transport` (type)
- `WebSocketTransport` (for advanced use)

### Step 3.5: Update Existing Tests

**File:** `ui/src/lib/vstkit-ipc/IpcBridge.test.ts`

- **Action:** Update tests to work with transport abstraction
- **Why:** Ensure existing functionality still works
- **Dependencies:** Step 3.1
- **Risk:** Low

**Changes:**
1. Create mock transport for testing
2. Update test setup to inject mock transport
3. Verify request/response flow through transport

---

## Phase 4: Meter Streaming

**Goal:** Add push-based meter updates over WebSocket.

### Step 4.1: Add Meter Broadcaster (Rust)

**File:** `engine/crates/standalone/src/ws_server.rs`

- **Action:** Add meter broadcasting at 60fps
- **Why:** Push-based meters are more efficient than polling
- **Dependencies:** Phase 1
- **Risk:** Medium

**Implementation details:**
1. Create `MeterBroadcaster` struct
2. Spawn tokio task with 16ms interval timer
3. Query `host.get_meter_frame()` each tick
4. Format as JSON-RPC notification (`method: "meterFrame"`)
5. Broadcast to all connected clients
6. Skip broadcast if no clients connected

### Step 4.2: Handle Meter Notifications (TypeScript)

**File:** `ui/src/lib/vstkit-ipc/hooks.ts`

- **Action:** Update `useMeterFrame` to use push notifications
- **Why:** Receive meters via WebSocket notifications instead of polling
- **Dependencies:** Phase 3
- **Risk:** Low

**Implementation details:**
1. Subscribe to `meterFrame` notifications via `IpcBridge.on()`
2. Update state on each notification
3. Fall back to polling for native transport (existing behavior)
4. Clean up subscription on unmount

### Step 4.3: Test Meter Streaming

- **Action:** Manual test of meter visualization in browser
- **Why:** Verify end-to-end meter flow
- **Dependencies:** Steps 4.1, 4.2
- **Risk:** Low

**Verification:**
1. Start dev server: `cargo run -p standalone -- --dev-server`
2. Start UI: `npm run dev`
3. Open browser, verify meters animate
4. Verify ~60fps update rate (no visible jitter)

---

## Phase 5: Polish & Documentation

**Goal:** Add connection UI, improve error handling, and document the feature.

### Step 5.1: Connection Status Indicator Component

**File:** `ui/src/components/ConnectionStatus.tsx` (NEW)

- **Action:** Create visual indicator for connection status
- **Why:** User feedback on engine connection
- **Dependencies:** Step 3.3
- **Risk:** Low

**Implementation details:**
1. Use `useConnectionStatus()` hook
2. Show green dot when connected
3. Show red dot + "Disconnected" when not connected
4. Position in corner of UI (subtle)

### Step 5.2: Update App to Show Connection Status

**File:** `ui/src/App.tsx`

- **Action:** Add ConnectionStatus component to App
- **Why:** Make connection state visible
- **Dependencies:** Step 5.1
- **Risk:** Low

### Step 5.3: Improve Error Handling

**Files:** Multiple

- **Action:** Add graceful handling for connection failures
- **Why:** Better UX when engine not running
- **Dependencies:** Phase 3
- **Risk:** Low

**Changes:**
1. `IpcBridge`: Throw descriptive errors on transport failure
2. `hooks.ts`: Return `null` states instead of crashing
3. Components: Show "Connecting..." or "Disconnected" states

### Step 5.4: Update Developer Documentation

**File:** `README.md` (or new `docs/guides/browser-development.md`)

- **Action:** Document browser development workflow
- **Why:** Enable developers to use the feature
- **Dependencies:** All phases
- **Risk:** Low

**Content:**
1. Two-terminal development setup
2. Available CLI flags
3. Troubleshooting connection issues
4. Architecture overview

### Step 5.5: Bump Version

**File:** `engine/Cargo.toml`

- **Action:** Bump version to `0.3.0`
- **Why:** Significant feature addition per coding standards
- **Dependencies:** All phases complete and tested
- **Risk:** Low

### Step 5.6: Run Full Test Suite

- **Action:** Run all linters and tests
- **Why:** Ensure no regressions
- **Dependencies:** All implementation complete
- **Risk:** Low

**Commands:**
```bash
cargo xtask lint
cargo xtask test
```

---

## Testing Strategy

### Unit Tests (Automated)

| Component | Test File | Coverage |
|-----------|-----------|----------|
| WebSocketTransport | `WebSocketTransport.test.ts` | Connection, messages, reconnection |
| Transport Factory | `transports/index.test.ts` | Environment detection, singleton |
| IpcBridge | `IpcBridge.test.ts` | Updated for transport abstraction |
| useConnectionStatus | `useConnectionStatus.test.ts` | Status reporting |

### Integration Tests (Manual)

| Scenario | Steps | Expected Result |
|----------|-------|-----------------|
| Basic connection | Start dev-server, open browser | UI connects, shows "Connected" |
| Parameter sync | Change slider in browser | Engine receives value |
| Meter streaming | Play audio through engine | Meters animate at 60fps |
| Reconnection | Restart engine while browser open | Auto-reconnects within 5s |
| Native mode | Build plugin, open in DAW | Works as before (no regression) |

---

## Risks & Mitigations

| Risk | Mitigation | Phase |
|------|------------|-------|
| Tokio conflicts with wry event loop | Run tokio in separate thread | Phase 1 |
| Breaking native IPC | NativeTransport wraps existing code unchanged | Phase 2 |
| WebSocket performance | Start with JSON; binary optimization is Phase 2 if needed | Phase 4 |
| Test flakiness with async | Use proper async test utilities, mock timers | Phase 2-3 |

---

## Definition of Done

- [ ] All implementation steps completed
- [ ] All unit tests passing (`cargo xtask test`)
- [ ] All lint checks passing (`cargo xtask lint`)
- [ ] Manual integration tests verified
- [ ] Native mode regression tested (plugin in DAW)
- [ ] Version bumped to 0.3.0
- [ ] Documentation updated
- [ ] PR created and ready for review

---

## Estimated Timeline

| Phase | Estimated Time | Dependencies |
|-------|---------------|--------------|
| Phase 0: Crate Rename | 0.5 day | None |
| Phase 1: WebSocket Server | 1-2 days | Phase 0 |
| Phase 2: Transport Abstraction | 1 day | Phase 1 |
| Phase 3: Refactor IpcBridge | 1 day | Phase 2 |
| Phase 4: Meter Streaming | 0.5 day | Phases 1, 3 |
| Phase 5: Polish | 0.5-1 day | All |
| **Total** | **4.5-6 days** | |

