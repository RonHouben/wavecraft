# Implementation Progress: OS Audio Input

**Feature:** OS Audio Input for Dev Mode  
**Target Version:** 0.8.0  
**Status:** Ready to Start  
**Updated:** 2026-02-08

---

## Progress Overview

- **Phase 1:** Protocol Extensions — ⬜ Not Started
- **Phase 2:** Audio Binary Template — ⬜ Not Started
- **Phase 3:** CLI Integration — ⬜ Not Started
- **Phase 4:** Template Updates — ⬜ Not Started
- **Phase 5:** Testing & Validation — ⬜ Not Started
- **Phase 6:** Documentation — ⬜ Not Started

**Legend:** ⬜ Not Started | 🔄 In Progress | ✅ Complete | ⚠️ Blocked

---

## Phase 1: Protocol Extensions

### 1.1 Add Audio Client Registration Message ⬜
- [ ] Add `ClientMessage::RegisterAudio` variant to protocol
- [ ] Add `ClientMessage::AudioReady` variant
- [ ] Add `ClientMessage::AudioError` variant
- [ ] Update protocol version (if needed)
- [ ] Verify protocol types compile

**File:** `engine/crates/wavecraft-protocol/src/messages.rs`

---

### 1.2 Add Meter Update Messages ⬜
- [ ] Create `MeterUpdateMessage` struct
- [ ] Add `ServerMessage::MeterUpdate` variant
- [ ] Add serialization tests for new message types
- [ ] Update protocol documentation

**File:** `engine/crates/wavecraft-protocol/src/messages.rs`

---

### 1.3 Update WebSocket Server to Route Audio Messages ⬜
- [ ] Add `audio_client` field to `WsServer` state
- [ ] Implement audio client registration handler
- [ ] Implement parameter update routing (browser → audio)
- [ ] Implement meter update routing (audio → browsers)
- [ ] Add connection lifecycle logging
- [ ] Test message routing with mock clients

**File:** `engine/crates/wavecraft-dev-server/src/ws_server.rs`

---

## Phase 2: Audio Binary Template

### 2.1 Create Audio Server Library Module ⬜
- [ ] Create `audio_server.rs` module
- [ ] Define `AudioServer<P: Processor>` struct
- [ ] Define `AudioConfig` struct
- [ ] Implement `AudioServer::new()` constructor
- [ ] Implement `AudioConfig::from_env()` helper
- [ ] Add module export to `lib.rs`

**File:** `engine/crates/wavecraft-dev-server/src/audio_server.rs` (new)

---

### 2.2 Implement Real-Time Audio Callback ⬜
- [ ] Set up cpal audio stream with error handling
- [ ] Implement interleaved → planar buffer conversion
- [ ] Call `processor.process()` in audio callback
- [ ] Add real-time safety audit (no allocations, locks)
- [ ] Compute RMS/peak for metering
- [ ] Write meter data to lock-free ring buffer
- [ ] Handle audio device errors gracefully
- [ ] Add unit test with mock processor

**File:** `engine/crates/wavecraft-dev-server/src/audio_server.rs`

---

### 2.3 Implement WebSocket Client for Audio Binary ⬜
- [ ] Create `ws_client.rs` module
- [ ] Implement `WebSocketClient::connect()`
- [ ] Implement `register_audio()` message
- [ ] Implement `send_meter_update()` method
- [ ] Implement `recv_parameter_update()` method
- [ ] Add reconnection logic with exponential backoff
- [ ] Add connection status logging
- [ ] Test WebSocket client against dev server

**File:** `engine/crates/wavecraft-dev-server/src/ws_client.rs` (new)

---

### 2.4 Add Meter Data Collection ⬜
- [ ] Implement `compute_channel_meters()` function
- [ ] Add meter throttling (send at ~60 Hz)
- [ ] Convert linear levels to dB
- [ ] Add timestamp to meter frames
- [ ] Verify meter accuracy with test signals
- [ ] Profile CPU usage of metering code

**File:** `engine/crates/wavecraft-dev-server/src/audio_server.rs`

---

### 2.5 Create `dev-audio.rs` Template ⬜
- [ ] Create template file with variable placeholders
- [ ] Add `main()` function with config setup
- [ ] Instantiate user's processor from template
- [ ] Add error handling and logging
- [ ] Add graceful shutdown handler (Ctrl+C)
- [ ] Test template with SDK's example plugin
- [ ] Add comments explaining customization points

**File:** `cli/sdk-templates/new-project/react/engine/src/bin/dev-audio.rs.template` (new)

---

## Phase 3: CLI Integration

### 3.1 Add `--with-audio` Flag to Start Command ⬜
- [ ] Add `with_audio: bool` field to `StartCommand`
- [ ] Add `--with-audio` flag to CLI parser
- [ ] Update `StartCommand::execute()` to check flag
- [ ] Add help text for new flag
- [ ] Test CLI accepts flag without errors

**Files:** `cli/src/commands/start.rs`, `cli/src/main.rs`

---

### 3.2 Detect Audio Binary Presence ⬜
- [ ] Create `has_dev_audio_binary()` function
- [ ] Parse `Cargo.toml` for `[[bin]]` section
- [ ] Check for `name = "dev-audio"` target
- [ ] Add error handling for missing Cargo.toml
- [ ] Test detection with/without audio binary

**File:** `cli/src/commands/start.rs`

---

### 3.3 Compile Audio Binary ⬜
- [ ] Create `compile_audio_binary()` function
- [ ] Run `cargo build --bin dev-audio`
- [ ] Handle compilation errors with clear messages
- [ ] Show compilation progress indicator
- [ ] Add `--verbose` flag support for cargo output
- [ ] Test with intentionally broken audio binary

**File:** `cli/src/commands/start.rs`

---

### 3.4 Spawn Audio Binary Process ⬜
- [ ] Create `spawn_audio_server()` function
- [ ] Run `cargo run --bin dev-audio` as child process
- [ ] Pass WebSocket URL via `WAVECRAFT_WS_URL` env var
- [ ] Pass log level via `RUST_LOG` env var
- [ ] Capture and store child process handle
- [ ] Add startup success detection (wait for registration)
- [ ] Test process spawning and PID tracking

**File:** `cli/src/commands/start.rs`

---

### 3.5 Add Process Cleanup on Exit ⬜
- [ ] Update `run_dev_servers()` to track all child processes
- [ ] Register Ctrl+C handler with `ctrlc` crate
- [ ] Kill all child processes on exit
- [ ] Add cleanup on error paths
- [ ] Test graceful shutdown (no orphaned processes)
- [ ] Test cleanup when one process exits prematurely

**File:** `cli/src/commands/start.rs`

---

### 3.6 Add Helpful Error Messages ⬜
- [ ] Add error message for missing audio binary
- [ ] Add instructions for creating `dev-audio.rs`
- [ ] Add error message for audio device unavailable
- [ ] Add error message for WebSocket connection failure
- [ ] Format error messages with colors/styles
- [ ] Test all error scenarios and verify messages

**File:** `cli/src/commands/start.rs`

---

## Phase 4: Template Updates

### 4.1 Update Template Cargo.toml ⬜
- [ ] Add `wavecraft-dev-server` to `[dev-dependencies]`
- [ ] Add `cpal` dependency
- [ ] Add `anyhow` dependency
- [ ] Add `env_logger` dependency
- [ ] Add `[[bin]]` section for `dev-audio`
- [ ] Test template Cargo.toml parses correctly

**File:** `cli/sdk-templates/new-project/react/engine/Cargo.toml.template`

---

### 4.2 Copy Template to New Projects ⬜
- [ ] Add `dev-audio.rs.template` to template file list
- [ ] Update `create` command to copy audio binary template
- [ ] Process template variables in `dev-audio.rs`
- [ ] Create `src/bin/` directory if needed
- [ ] Test `wavecraft create` includes audio binary
- [ ] Verify generated audio binary compiles

**File:** `cli/src/commands/create.rs`

---

### 4.3 Update Template README ⬜
- [ ] Add "Development" section with audio testing
- [ ] Document browser mode vs OS audio mode
- [ ] Add usage examples for both modes
- [ ] Add troubleshooting section
- [ ] Link to audio development guide
- [ ] Review README clarity with fresh eyes

**File:** `cli/sdk-templates/new-project/react/README.md`

---

## Phase 5: Testing & Validation

### 5.1 Unit Tests for Protocol Messages ⬜
- [ ] Test `RegisterAudio` serialization/deserialization
- [ ] Test `MeterUpdate` serialization/deserialization
- [ ] Test protocol version compatibility
- [ ] Run `cargo test` in `wavecraft-protocol` crate
- [ ] Verify all tests pass

**File:** `engine/crates/wavecraft-protocol/src/messages.rs`

---

### 5.2 Integration Test: Generate and Compile Test Plugin ⬜
- [ ] Create `audio_binary_compilation.rs` test file
- [ ] Test `wavecraft create TestPlugin`
- [ ] Test `cargo build --bin dev-audio` succeeds
- [ ] Test audio binary runs without errors
- [ ] Run test in CI environment
- [ ] Fix any discovered issues

**File:** `cli/tests/audio_binary_compilation.rs` (new)

---

### 5.3 Manual Test: Real Audio Flow ⬜
- [ ] Generate test plugin
- [ ] Start with `--with-audio` flag
- [ ] Verify audio flows mic → speakers
- [ ] Test parameter changes affect audio
- [ ] Test meter levels are accurate
- [ ] Measure latency (target <50ms)
- [ ] Test Ctrl+C shutdown
- [ ] Document test results in test-plan.md

**File:** `docs/feature-specs/dev-audio-os-input/test-plan.md`

---

### 5.4 Performance Testing ⬜
- [ ] Test with 256 sample buffer size
- [ ] Test with 128 sample buffer size
- [ ] Test under CPU load (compile while running)
- [ ] Run for 5 minutes continuous
- [ ] Count xruns (<1% target)
- [ ] Measure latency jitter
- [ ] Document performance baseline

**File:** `docs/feature-specs/dev-audio-os-input/test-plan.md`

---

### 5.5 Error Handling Tests ⬜
- [ ] Test missing audio binary error
- [ ] Test audio device unavailable
- [ ] Test WebSocket connection failure
- [ ] Test audio binary crash recovery
- [ ] Verify all error messages are helpful
- [ ] Document error scenarios

**File:** `cli/tests/audio_error_handling.rs` (new)

---

## Phase 6: Documentation

### 6.1 Update SDK Getting Started Guide ⬜
- [ ] Add "Testing with Real Audio" section
- [ ] Document `--with-audio` flag usage
- [ ] Explain when to use each mode
- [ ] Add troubleshooting tips
- [ ] Link to audio development guide

**File:** `docs/guides/sdk-getting-started.md`

---

### 6.2 Create Audio Development Guide ⬜
- [ ] Create new guide file
- [ ] Document audio binary architecture
- [ ] Explain customization options
- [ ] Add performance tuning section
- [ ] Add macOS audio permissions section
- [ ] Add troubleshooting section
- [ ] Review guide completeness

**File:** `docs/guides/audio-development.md` (new)

---

### 6.3 Update High-Level Design Doc ⬜
- [ ] Add OS Audio Development Mode diagram
- [ ] Document audio binary compilation model
- [ ] Document WebSocket protocol extensions
- [ ] Add comparison table (Browser vs OS Audio)
- [ ] Update architecture overview
- [ ] Review for consistency with implementation

**File:** `docs/architecture/high-level-design.md`

---

### 6.4 Update Roadmap ⬜
- [ ] Move "OS Audio Input" to Completed
- [ ] Update version to 0.8.0
- [ ] Add changelog entry
- [ ] Archive feature spec folder
- [ ] Commit roadmap updates

**File:** `docs/roadmap.md`

---

## Blocked Items

_No blocked items currently._

---

## Notes & Decisions

### 2026-02-08 — Initial Plan Created
- Target version set to 0.8.0 (minor bump)
- macOS-only for Phase 1
- System default audio device only (no device selection)
- WebSocket protocol reused from browser mode
- Estimated effort: 15-20 development days

---

## Next Steps

1. Start with Phase 1: Protocol Extensions (Steps 1.1-1.3)
2. Implement Phase 2: Audio Binary in parallel where possible
3. Block out focused time for Phase 2.2 (real-time audio callback)
