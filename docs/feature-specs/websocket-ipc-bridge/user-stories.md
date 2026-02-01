# User Stories: WebSocket IPC Bridge

## Overview

Enable real IPC communication between the React UI running in a browser and the Rust engine, eliminating the need for mock data during development.

**The Problem:** When developing the UI with `npm run dev`, the UI falls back to static mock data because it can only communicate with the Rust engine inside WKWebView. This creates:
- Double implementation that can drift
- Limited dev experience (no real meters, parameters)
- Testing gaps for automated browser testing

**The Solution:** Add a WebSocket server to the desktop app that exposes the same IPC protocol over `ws://localhost:9000`. The UI auto-detects the environment and connects via WebSocket when not in WKWebView.

---

## Version

**Target Version:** `0.3.0` (minor bump from `0.2.1`)

**Rationale:** This is a significant architectural feature that adds a new communication transport and fundamentally improves the development workflow. Minor bump appropriate per coding standards.

---

## User Story 1: WebSocket Server in Desktop App

**As a** plugin developer  
**I want** the desktop app to run a WebSocket server  
**So that** the browser-based UI can connect to the real Rust engine

### Acceptance Criteria
- [ ] Desktop app starts WebSocket server on `ws://localhost:9000` by default
- [ ] WebSocket server uses the same `IpcHandler` as the native bridge
- [ ] Server accepts JSON-RPC 2.0 messages (same protocol as native IPC)
- [ ] Server gracefully handles client connect/disconnect
- [ ] Port is configurable via CLI flag (e.g., `--port 9001`)

### Notes
- Consider `tokio-tungstenite` for async WebSocket support
- Must not block the audio thread or native IPC

---

## User Story 2: Headless Development Server Mode

**As a** UI developer  
**I want** to run the Rust engine without opening a UI window  
**So that** I can develop the UI in my browser with hot reload

### Acceptance Criteria
- [ ] `--dev-server` CLI flag starts engine in headless mode
- [ ] WebSocket server runs without spawning WKWebView window
- [ ] Engine processes audio/parameters normally
- [ ] Clean shutdown on Ctrl+C

### Notes
- Typical workflow: `cargo run -p desktop -- --dev-server` + `npm run dev`
- Consider logging connection events for debugging

---

## User Story 3: Transport Abstraction in UI

**As a** UI developer  
**I want** the IPC client to support multiple transports  
**So that** the same UI code works in both WKWebView and browser

### Acceptance Criteria
- [ ] `IpcBridge` abstracted to support pluggable transports
- [ ] `NativeTransport` wraps existing `window.vstkit` bridge
- [ ] `WebSocketTransport` connects via `ws://localhost:9000`
- [ ] Both transports implement the same interface
- [ ] Factory selects transport based on environment

### Notes
- Existing `isBrowserEnvironment()` detection can be extended
- Transport selection should happen once at initialization

---

## User Story 4: Automatic Environment Detection

**As a** UI developer  
**I want** the UI to automatically choose the right transport  
**So that** I don't need to manually configure anything

### Acceptance Criteria
- [ ] UI detects WKWebView environment → uses native transport
- [ ] UI detects browser environment → uses WebSocket transport
- [ ] Detection happens at module initialization (not per-render)
- [ ] Clear console logging indicates which transport is active

### Notes
- Build on existing `isBrowserEnvironment()` function
- Consider fallback behavior if WebSocket connection fails

---

## User Story 5: Real-Time Meter Streaming

**As a** UI developer  
**I want** to see real meter data in the browser  
**So that** I can develop and test meter visualizations with actual audio levels

### Acceptance Criteria
- [ ] Meter data streams from engine to browser via WebSocket
- [ ] Update rate matches native performance (~60fps)
- [ ] Meter frame format is identical to native IPC
- [ ] UI renders meters without visible lag

### Notes
- May need push-based updates rather than polling
- Consider binary WebSocket frames for efficiency (optional optimization)

---

## User Story 6: Connection Resilience

**As a** UI developer  
**I want** the UI to handle connection interruptions gracefully  
**So that** I can restart the engine without refreshing the browser

### Acceptance Criteria
- [ ] UI attempts reconnection on disconnect (exponential backoff)
- [ ] UI shows connection status indicator
- [ ] Parameters/meters resume after reconnection
- [ ] No console errors spam on expected disconnects

### Notes
- Max reconnection attempts configurable (default: 5)
- Consider "Engine disconnected" overlay vs silent reconnection

---

## User Story 7: Remove Mock Data

**As a** maintainer  
**I want** to remove the static mock data from the UI  
**So that** we have a single source of truth and less code to maintain

### Acceptance Criteria
- [ ] Mock parameter data removed from `IpcBridge`
- [ ] Mock meter data removed from `IpcBridge`
- [ ] Browser mode requires running engine for any data
- [ ] Tests continue to use mock module (`test/mocks/ipc.ts`)

### Notes
- This is the cleanup phase after WebSocket transport works
- Test mocks are separate and should remain

---

## Out of Scope

- Remote debugging (same machine only for M6)
- Authentication/security (localhost only)
- Multiple simultaneous browser connections
- Mobile companion app support
- Binary protocol optimization (JSON-RPC is sufficient)

---

## Dependencies

- None (builds on existing infrastructure)

---

## Risks

| Risk | Mitigation |
|------|------------|
| WebSocket performance insufficient for meters | Start with JSON; optimize to binary if needed |
| Port conflicts on developer machines | Configurable port with clear error messages |
| Complexity of transport abstraction | Keep interface minimal; match existing IPC exactly |

---

## Success Metrics

- `npm run dev` shows real parameter values from engine
- Meters animate with actual audio input
- Hot reload works without reconnecting to engine
- Developer experience feedback is positive

