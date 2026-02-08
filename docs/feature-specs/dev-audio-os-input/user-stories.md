# User Stories: OS Audio Input for Dev Mode

## Overview

Enable developers to test their audio plugins with real microphone input during development without loading the plugin in a DAW. This feature adds an optional `--with-audio` flag to `wavecraft start` that spawns a user-compiled binary that processes OS audio using the same DSP code as the production plugin.

## Version

**Target Version:** `0.8.0` (minor bump from `0.7.1`)

**Rationale:** This is a significant new feature that enhances the developer experience but does not break existing APIs. It represents a measurable improvement to the SDK's capabilities (real audio testing in dev mode) without requiring changes to user code.

---

## User Story 1: Test DSP with Real Audio Input

**As a** plugin developer working on a gain/EQ/dynamics processor  
**I want** to test my DSP code with real microphone input  
**So that** I can validate audio processing behavior without loading the plugin in a DAW

### Acceptance Criteria
- [ ] Running `wavecraft start --with-audio` starts the dev server with OS audio input
- [ ] Plugin's `Processor::process()` is called with real audio buffers from the system microphone
- [ ] Meters in the UI show levels reflecting the processed audio
- [ ] Parameter changes in the UI affect the audio processing in real-time
- [ ] UI hot-reloading still works (Vite HMR) while audio is running
- [ ] Audio dropout/xruns are minimal on typical development hardware

### Notes
- This reuses the exact same `Processor` implementation code as the plugin
- No DSP code duplication — single source of truth
- Audio binary is compiled by the user's project, not embedded in CLI

---

## User Story 2: Opt-in Audio Without Breaking Existing Workflow

**As a** UI-focused developer  
**I want** the default `wavecraft start` to work without audio setup  
**So that** I can iterate on UI components without Rust compilation overhead

### Acceptance Criteria
- [ ] `wavecraft start` (without flag) works exactly as before with synthetic meter data
- [ ] `wavecraft start --with-audio` is an explicit opt-in
- [ ] If audio binary is not compiled, the command fails with a clear error message directing to template setup
- [ ] Documentation explains when to use each mode

### Notes
- Default behavior is unchanged (mock data)
- Audio mode is additive, not a replacement
- Graceful degradation: works without audio binary present

---

## User Story 3: Clear Separation Between CLI and User Code

**As a** plugin developer  
**I want** the audio processing to live in my project, not the CLI  
**So that** the CLI remains a lightweight coordinator and my DSP is fully customizable

### Acceptance Criteria
- [ ] CLI spawns `cargo run --bin dev-audio` in the user's project
- [ ] Audio binary compiles with user's `Cargo.toml` dependencies
- [ ] CLI communicates with audio binary via WebSocket (same protocol as browser)
- [ ] Audio binary can be customized/extended by users
- [ ] Template includes `src/bin/dev-audio.rs` as reference implementation

### Notes
- CLI is a **coordinator**, not an audio engine
- Audio binary is **user code**, like integration tests or benchmarks
- Protocol-based communication ensures loose coupling

---

## User Story 4: Integration Testing Without DAW

**As a** plugin developer  
**I want** to validate the full signal chain (input → DSP → metering) in development mode  
**So that** I can catch integration bugs early without DAW loading overhead

### Acceptance Criteria
- [ ] Audio flows from OS input → Processor → Meter calculations → WebSocket → UI
- [ ] Full round-trip latency is measured and logged
- [ ] Parameter updates from UI are reflected in audio processing within 50ms
- [ ] Metering shows both input and output levels accurately
- [ ] Audio routing matches plugin behavior (stereo, mono, sidechain if applicable)

### Notes
- This tests the complete integration, not just DSP in isolation
- Helps catch parameter sync issues, metering bugs, IPC latency
- Closer to production environment than unit tests

---

## Technical Constraints (from Architect)

These are **non-negotiable** boundaries to maintain clean architecture:

1. **Audio binary compiled by user project** — Template includes `src/bin/dev-audio.rs`, user's `Cargo.toml` controls dependencies
2. **CLI spawns binary, doesn't embed** — `cargo run --bin dev-audio` initiated by CLI
3. **Opt-in feature** — `--with-audio` flag required, default behavior unchanged
4. **Protocol communication** — WebSocket/JSON-RPC, same as browser transport
5. **Graceful fallback** — Works without audio binary present (shows helpful error)

---

## Out of Scope

- **WASM audio input** — Future consideration, complementary to this feature
- **Multiple audio devices** — Phase 1 uses system default microphone only
- **MIDI input** — Separate feature
- **Recording/playback** — Dev mode is for live testing only
- **Windows/Linux support** — Phase 1 is macOS-only (consistent with project focus)

---

## Success Metrics

- Developers can test audio processing within 5 seconds of starting dev server
- UI iteration speed is unaffected (HMR still works)
- Zero DSP code duplication (architectural audit confirms single implementation)
- Audio latency < 50ms on typical MacBook hardware
- Template-generated projects include working audio binary

---

## Dependencies

- Rust audio library: `cpal` (cross-platform audio I/O)
- WebSocket protocol: Already implemented in `wavecraft-dev-server`
- Parameter host trait: Already implemented in `wavecraft-bridge`
- Template system: Already supports binary targets

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Audio setup complexity confuses users | Medium | Medium | Clear docs, opt-in flag, helpful error messages |
| Performance issues (xruns) | Low | High | Use proven audio library (cpal), document hardware requirements |
| Protocol complexity | Low | Medium | Reuse existing WebSocket/JSON-RPC, same as browser |
| Cross-platform maintenance | Medium | Medium | Phase 1 macOS-only, Windows/Linux later |

---

## Related Documents

- [Architect Q&A](./architect-response-os-audio.md) — Direct answers to architectural questions
- [Full Architectural Analysis](./architectural-reevaluation-os-audio-reuse.md) — Complete technical evaluation
- [Implementation Constraints](./implementation-constraints-os-audio.md) — Non-negotiable boundaries
