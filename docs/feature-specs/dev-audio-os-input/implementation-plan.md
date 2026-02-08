# Implementation Plan: OS Audio Input for Dev Mode

**Feature:** Enable real-time OS audio testing via `wavecraft start --with-audio`  
**Target Version:** 0.8.0  
**Created:** 2026-02-08  
**Status:** Ready for Implementation

---

## Overview

Enable developers to test plugins with real microphone input by spawning a user-compiled audio binary that processes OS audio using the same DSP code as the production plugin. The CLI remains a coordinator, spawning processes and bridging WebSocket communication.

**Core Architecture:**
- Audio binary compiled by **user project** (template provides `src/bin/dev-audio.rs`)
- CLI **spawns** binary via `cargo run --bin dev-audio`
- **Opt-in** via `--with-audio` flag (default unchanged)
- **Protocol** communication (WebSocket/JSON-RPC, same as browser)
- **Zero DSP duplication** (same `Processor` implementation)

---

## Prerequisites

Before starting implementation, verify:
- [ ] Existing `wavecraft start` command works (spawns WebSocket + Vite)
- [ ] WebSocket protocol supports JSON-RPC (request/response + events)
- [ ] Template system supports binary targets and dev-dependencies
- [ ] `cargo xtask ci-check` passes on current main branch

---

## Phase 1: Protocol Extensions

**Goal:** Extend WebSocket protocol to support audio binary communication without breaking existing browser clients.

### 1.1 Add Audio Client Registration Message

**File:** `engine/crates/wavecraft-protocol/src/messages.rs`

- **Action:** Add new message types for audio client lifecycle
- **Why:** Audio binary needs to identify itself to the WebSocket server
- **Dependencies:** None
- **Risk:** Low

```rust
// New message types to add
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    // Existing browser messages
    RegisterBrowser { ... },
    
    // New audio client messages
    RegisterAudio {
        client_id: String,
        sample_rate: f32,
        buffer_size: u32,
    },
    
    AudioReady,
    AudioError { error: String },
}
```

**Expected outcome:** Protocol types compile with new variants.

---

### 1.2 Add Meter Update Messages

**File:** `engine/crates/wavecraft-protocol/src/messages.rs`

- **Action:** Add bidirectional meter data messages
- **Why:** Audio binary sends meter frames to UI via CLI bridge
- **Dependencies:** Step 1.1
- **Risk:** Low

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MeterUpdateMessage {
    pub timestamp_us: u64,
    pub left_peak: f32,
    pub left_rms: f32,
    pub right_peak: f32,
    pub right_rms: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    // Existing messages
    ParameterUpdate { ... },
    
    // New message
    MeterUpdate(MeterUpdateMessage),
}
```

**Expected outcome:** Meter data can flow from audio binary → CLI → browser.

---

### 1.3 Update WebSocket Server to Route Audio Messages

**File:** `engine/crates/wavecraft-dev-server/src/ws_server.rs`

- **Action:** Track audio client connection separately from browser clients
- **Why:** CLI needs to route messages between audio binary and browser
- **Dependencies:** Steps 1.1, 1.2
- **Risk:** Medium (WebSocket state management)

```rust
pub struct WsServer {
    browser_clients: Vec<WebSocketStream>,
    audio_client: Option<WebSocketStream>,  // New field
    // ...
}

// New routing logic:
// - Parameter updates from browser → forward to audio client
// - Meter updates from audio client → broadcast to all browsers
```

**Expected outcome:** WebSocket server can handle two client types simultaneously.

---

## Phase 2: Audio Binary Template

**Goal:** Create the `dev-audio.rs` template that users compile with their DSP code.

### 2.1 Create Audio Server Library Module

**File:** `engine/crates/wavecraft-dev-server/src/audio_server.rs` (new file)

- **Action:** Create generic `AudioServer<P: Processor>` struct
- **Why:** Provides reusable audio I/O infrastructure for user binaries
- **Dependencies:** Protocol extensions (Phase 1)
- **Risk:** Medium (real-time audio stability)

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use wavecraft_dsp::Processor;

pub struct AudioServer<P: Processor> {
    processor: P,
    config: AudioConfig,
    ws_client: WebSocketClient,
    meter_producer: MeterProducer,
}

pub struct AudioConfig {
    pub websocket_url: String,
    pub sample_rate: f32,
    pub buffer_size: u32,
}

impl<P: Processor> AudioServer<P> {
    pub fn new(processor: P, config: AudioConfig) -> Result<Self>;
    pub fn run(self) -> Result<()>;
}
```

**Expected outcome:** Generic audio server compiles and exposes clean public API.

---

### 2.2 Implement Real-Time Audio Callback

**File:** `engine/crates/wavecraft-dev-server/src/audio_server.rs`

- **Action:** Implement cpal stream callback with real-time safety
- **Why:** Core audio processing loop that calls user's `Processor::process()`
- **Dependencies:** Step 2.1
- **Risk:** High (real-time thread safety, xrun potential)

```rust
// Audio callback pattern:
let stream = device.build_input_stream(
    &config,
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        // 1. Convert interleaved → planar buffers
        // 2. Call processor.process(&mut buffers, &transport, &params)
        // 3. Compute RMS/peak for meters (lock-free)
        // 4. Write meter data to ring buffer
    },
    |err| eprintln!("Audio error: {}", err),
)?;
```

**Testing strategy:**
- Unit test: Mock `Processor` with counters to verify calls
- Integration test: Run with test tone processor, verify output levels
- Manual test: Check for xruns with oscilloscope monitoring

**Expected outcome:** Audio flows from mic → processor → speaker with <10ms latency.

---

### 2.3 Implement WebSocket Client for Audio Binary

**File:** `engine/crates/wavecraft-dev-server/src/ws_client.rs` (new file)

- **Action:** Create WebSocket client for audio binary to connect to CLI
- **Why:** Audio binary needs bidirectional communication with CLI
- **Dependencies:** Protocol extensions (Phase 1)
- **Risk:** Medium (connection handling, reconnection)

```rust
pub struct WebSocketClient {
    url: String,
    socket: WebSocket,
}

impl WebSocketClient {
    pub async fn connect(url: &str) -> Result<Self>;
    pub async fn register_audio(&self, sample_rate: f32, buffer_size: u32) -> Result<()>;
    pub async fn send_meter_update(&self, frame: MeterFrame) -> Result<()>;
    pub async fn recv_parameter_update(&mut self) -> Result<ParameterUpdate>;
}
```

**Expected outcome:** Audio binary can send/receive messages via WebSocket.

---

### 2.4 Add Meter Data Collection

**File:** `engine/crates/wavecraft-dev-server/src/audio_server.rs`

- **Action:** Compute RMS and peak levels in audio callback, send via WebSocket
- **Why:** UI needs real-time meter feedback
- **Dependencies:** Steps 2.2, 2.3
- **Risk:** Low (proven pattern from existing metering crate)

```rust
// In audio callback:
let (left_peak, left_rms) = compute_channel_meters(&left_buffer);
let (right_peak, right_rms) = compute_channel_meters(&right_buffer);

// Send at ~60 Hz (every ~735 samples at 44.1kHz)
if frame_counter % 735 == 0 {
    ws_client.send_meter_update(MeterUpdateMessage {
        timestamp_us: now_micros(),
        left_peak: linear_to_db(left_peak),
        left_rms: linear_to_db(left_rms),
        right_peak: linear_to_db(right_peak),
        right_rms: linear_to_db(right_rms),
    }).await?;
}
```

**Expected outcome:** Meter data flows to UI at 60 Hz with accurate levels.

---

### 2.5 Create `dev-audio.rs` Template

**File:** `cli/sdk-templates/new-project/react/engine/src/bin/dev-audio.rs.template` (new file)

- **Action:** Create template binary that users compile
- **Why:** User projects need a starting point for audio binary
- **Dependencies:** Steps 2.1-2.4 (AudioServer API finalized)
- **Risk:** Low (straightforward template)

```rust
use {{plugin_name_snake}}::{{processor_type}};  // User's DSP
use wavecraft_dev_server::audio_server::{AudioConfig, AudioServer};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let config = AudioConfig {
        websocket_url: std::env::var("WAVECRAFT_WS_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:9000".to_string()),
        sample_rate: 44100.0,
        buffer_size: 512,
    };
    
    let processor = {{processor_type}}::default();
    let server = AudioServer::new(processor, config)?;
    
    println!("Audio server started. Press Ctrl+C to stop.");
    server.run()
}
```

**Expected outcome:** Template compiles and runs when user executes `cargo run --bin dev-audio`.

---

## Phase 3: CLI Integration

**Goal:** Extend CLI start command to spawn audio binary when `--with-audio` flag is used.

### 3.1 Add `--with-audio` Flag to Start Command

**File:** `cli/src/commands/start.rs`

- **Action:** Add new boolean flag to `StartCommand` struct
- **Why:** Opt-in audio mode
- **Dependencies:** None
- **Risk:** Low

```rust
#[derive(Debug)]
pub struct StartCommand {
    pub port: u16,
    pub ui_port: u16,
    pub install: bool,
    pub no_install: bool,
    pub verbose: bool,
    pub with_audio: bool,  // New field
}
```

**File:** `cli/src/main.rs`

```rust
#[command(name = "start")]
struct StartArgs {
    // ... existing args ...
    
    /// Enable OS audio input (requires dev-audio binary)
    #[arg(long)]
    with_audio: bool,
}
```

**Expected outcome:** `wavecraft start --with-audio` accepted by CLI parser.

---

### 3.2 Detect Audio Binary Presence

**File:** `cli/src/commands/start.rs`

- **Action:** Check if user project has `dev-audio` binary target
- **Why:** Graceful fallback if audio binary not compiled
- **Dependencies:** Step 3.1
- **Risk:** Low

```rust
fn has_dev_audio_binary(project: &ProjectMarkers) -> Result<bool> {
    let cargo_toml_path = project.engine_dir.join("Cargo.toml");
    let cargo_toml = std::fs::read_to_string(&cargo_toml_path)?;
    
    // Check for [[bin]] section with name = "dev-audio"
    Ok(cargo_toml.contains("name = \"dev-audio\""))
}
```

**Expected outcome:** CLI can detect if audio binary is available.

---

### 3.3 Compile Audio Binary

**File:** `cli/src/commands/start.rs`

- **Action:** Run `cargo build --bin dev-audio` before starting servers
- **Why:** Ensure audio binary is up-to-date
- **Dependencies:** Step 3.2
- **Risk:** Medium (compilation errors need clear messaging)

```rust
fn compile_audio_binary(project: &ProjectMarkers, verbose: bool) -> Result<()> {
    println!("{} Compiling audio binary...", style("→").cyan());
    
    let status = Command::new("cargo")
        .args(["build", "--bin", "dev-audio"])
        .current_dir(&project.engine_dir)
        .stdout(if verbose { Stdio::inherit() } else { Stdio::null() })
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to build audio binary")?;
    
    if !status.success() {
        anyhow::bail!("Audio binary compilation failed. Check errors above.");
    }
    
    println!("{} Audio binary compiled", style("✓").green());
    Ok(())
}
```

**Expected outcome:** Audio binary compiles successfully or shows clear error.

---

### 3.4 Spawn Audio Binary Process

**File:** `cli/src/commands/start.rs`

- **Action:** Spawn `cargo run --bin dev-audio` as child process
- **Why:** Audio processing runs in separate process from CLI
- **Dependencies:** Step 3.3
- **Risk:** Medium (process lifecycle management)

```rust
fn spawn_audio_server(
    project: &ProjectMarkers,
    ws_port: u16,
    verbose: bool,
) -> Result<Child> {
    println!("{} Starting audio server...", style("→").cyan());
    
    let child = Command::new("cargo")
        .args(["run", "--bin", "dev-audio"])
        .current_dir(&project.engine_dir)
        .env("WAVECRAFT_WS_URL", format!("ws://127.0.0.1:{}", ws_port))
        .env("RUST_LOG", if verbose { "debug" } else { "info" })
        .stdout(if verbose { Stdio::inherit() } else { Stdio::null() })
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start audio server")?;
    
    println!("{} Audio server started (PID: {})", style("✓").green(), child.id());
    Ok(child)
}
```

**Expected outcome:** Audio binary runs as independent process, CLI tracks PID for cleanup.

---

### 3.5 Add Process Cleanup on Exit

**File:** `cli/src/commands/start.rs`

- **Action:** Handle Ctrl+C and cleanup spawned processes
- **Why:** Graceful shutdown, no orphaned processes
- **Dependencies:** Step 3.4
- **Risk:** Low

```rust
fn run_dev_servers_with_audio(
    project: &ProjectMarkers,
    ws_port: u16,
    ui_port: u16,
    verbose: bool,
) -> Result<()> {
    // ... spawn WebSocket server, Vite, audio binary ...
    
    let mut children = vec![ws_server_child, vite_child, audio_child];
    
    // Register Ctrl+C handler
    ctrlc::set_handler(move || {
        println!("\n{} Shutting down...", style("→").yellow());
        for child in &mut children {
            let _ = child.kill();
        }
        std::process::exit(0);
    })?;
    
    // Wait for any child to exit
    wait_for_any_exit(&mut children)?;
    Ok(())
}
```

**Expected outcome:** Ctrl+C cleanly stops all spawned processes.

---

### 3.6 Add Helpful Error Messages

**File:** `cli/src/commands/start.rs`

- **Action:** Provide actionable error messages for common issues
- **Why:** Developer experience
- **Dependencies:** Steps 3.2-3.4
- **Risk:** Low

```rust
if self.with_audio {
    if !has_dev_audio_binary(&project)? {
        eprintln!("{} Audio binary not found", style("✗").red().bold());
        eprintln!();
        eprintln!("To enable audio mode, add this binary to your engine/Cargo.toml:");
        eprintln!();
        eprintln!("  [[bin]]");
        eprintln!("  name = \"dev-audio\"");
        eprintln!("  path = \"src/bin/dev-audio.rs\"");
        eprintln!();
        eprintln!("Then create engine/src/bin/dev-audio.rs (see SDK template).");
        eprintln!();
        anyhow::bail!("Run `wavecraft start` without --with-audio to use mock data.");
    }
    
    // ... proceed with audio mode ...
}
```

**Expected outcome:** Clear guidance when audio binary is missing.

---

## Phase 4: Template Updates

**Goal:** Provide users with working audio binary template and dependencies.

### 4.1 Update Template Cargo.toml

**File:** `cli/sdk-templates/new-project/react/engine/Cargo.toml.template`

- **Action:** Add cpal and wavecraft-dev-server dependencies
- **Why:** Audio binary needs OS audio I/O library
- **Dependencies:** None
- **Risk:** Low

```toml
# Add to [dev-dependencies] section:
[dev-dependencies]
wavecraft-dev-server = { package = "wavecraft-dev-server", git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
cpal = "0.15"
anyhow = "1.0"
env_logger = "0.11"

# Add binary target:
[[bin]]
name = "dev-audio"
path = "src/bin/dev-audio.rs"
```

**Expected outcome:** Template projects can compile audio binary.

---

### 4.2 Copy Template to New Projects

**File:** `cli/src/commands/create.rs`

- **Action:** Copy `dev-audio.rs.template` when scaffolding projects
- **Why:** Users get working reference implementation
- **Dependencies:** Step 2.5 (template created)
- **Risk:** Low

```rust
// In create.rs template copying logic:
let template_files = vec![
    // ... existing files ...
    ("engine/src/bin/dev-audio.rs.template", "engine/src/bin/dev-audio.rs"),
];
```

**Expected outcome:** New projects include `src/bin/dev-audio.rs` by default.

---

### 4.3 Update Template README

**File:** `cli/sdk-templates/new-project/react/README.md`

- **Action:** Document audio testing workflow
- **Why:** Users need to know about `--with-audio` flag
- **Dependencies:** None
- **Risk:** Low

```markdown
## Development

### Browser Testing (Default)
```bash
wavecraft start
```
UI with synthetic meter data. Fast iteration, no audio setup required.

### OS Audio Testing
```bash
wavecraft start --with-audio
```
Test with real microphone input. Requires audio hardware and compiled `dev-audio` binary.
```

**Expected outcome:** Users understand when to use each mode.

---

## Phase 5: Testing & Validation

**Goal:** Verify the feature works end-to-end and document test results.

### 5.1 Unit Tests for Protocol Messages

**File:** `engine/crates/wavecraft-protocol/src/messages.rs`

- **Action:** Add tests for new message serialization
- **Why:** Ensure protocol compatibility
- **Dependencies:** Phase 1 (protocol extensions)
- **Risk:** Low

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_registration_serialization() {
        let msg = ClientMessage::RegisterAudio {
            client_id: "test".to_string(),
            sample_rate: 44100.0,
            buffer_size: 512,
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
        
        assert!(matches!(deserialized, ClientMessage::RegisterAudio { .. }));
    }
}
```

**Expected outcome:** All protocol tests pass.

---

### 5.2 Integration Test: Generate and Compile Test Plugin

**File:** `cli/tests/audio_binary_compilation.rs` (new file)

- **Action:** Test that generated projects can compile audio binary
- **Why:** Catch template issues
- **Dependencies:** Phase 4 (template updates)
- **Risk:** Low

```rust
#[test]
fn test_audio_binary_compiles() {
    let temp_dir = TempDir::new().unwrap();
    
    // Run wavecraft create
    let status = Command::new(env!("CARGO_BIN_EXE_wavecraft"))
        .args(["create", "TestPlugin", "--output", temp_dir.path().to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());
    
    // Compile audio binary
    let engine_dir = temp_dir.path().join("TestPlugin/engine");
    let status = Command::new("cargo")
        .args(["build", "--bin", "dev-audio"])
        .current_dir(&engine_dir)
        .status()
        .unwrap();
    assert!(status.success(), "Audio binary failed to compile");
}
```

**Expected outcome:** Test plugin audio binary compiles cleanly.

---

### 5.3 Manual Test: Real Audio Flow

**File:** `docs/feature-specs/dev-audio-os-input/test-plan.md`

- **Action:** Create test plan for manual validation
- **Why:** Some aspects require human verification (audio quality, latency)
- **Dependencies:** All phases complete
- **Risk:** Low

**Test steps:**
1. Generate test plugin: `wavecraft create TestAudioPlugin`
2. Start with audio: `cd TestAudioPlugin && wavecraft start --with-audio`
3. Verify audio input flows to speakers
4. Adjust gain parameter in UI, verify audio changes
5. Check meter levels match audio monitoring
6. Measure latency with click test (<50ms target)
7. Test Ctrl+C shutdown (no orphaned processes)

**Expected outcome:** All manual tests pass, documented in test-plan.md.

---

### 5.4 Performance Testing

**File:** `docs/feature-specs/dev-audio-os-input/test-plan.md`

- **Action:** Test under load (low buffer sizes, CPU stress)
- **Why:** Validate real-time stability
- **Dependencies:** Step 5.3
- **Risk:** Medium (may reveal performance issues)

**Test scenarios:**
- Buffer size: 256 samples (prefer), 128 samples (acceptable), 64 samples (stretch goal)
- CPU load: Run while compiling (simulate development environment)
- Duration: 5 minutes continuous playback
- Metrics: Count xruns, measure latency jitter

**Expected outcome:** <1% xrun rate at 256 samples, documented performance baseline.

---

### 5.5 Error Handling Tests

**File:** `cli/tests/audio_error_handling.rs` (new file)

- **Action:** Test graceful failure scenarios
- **Why:** Developer experience on errors
- **Dependencies:** Phase 3 (CLI integration)
- **Risk:** Low

**Test cases:**
- Missing audio binary → helpful error message
- Audio device unavailable → graceful fallback
- WebSocket connection fails → retry and report
- Audio binary crashes → CLI detects and reports

**Expected outcome:** All error scenarios show actionable messages.

---

## Phase 6: Documentation

**Goal:** Complete user-facing and developer documentation.

### 6.1 Update SDK Getting Started Guide

**File:** `docs/guides/sdk-getting-started.md`

- **Action:** Add section on audio testing workflow
- **Why:** Users need to discover the feature
- **Dependencies:** All phases complete
- **Risk:** Low

**Content to add:**
```markdown
## Testing with Real Audio

For testing audio processing with microphone input:

```bash
wavecraft start --with-audio
```

This mode:
- Processes real audio from your system microphone
- Uses the same `Processor` code as your plugin
- Updates meters in real-time
- Requires the `dev-audio` binary (included in templates)

**Note:** UI hot-reloading still works while audio is running.
```

**Expected outcome:** Getting Started guide covers audio testing.

---

### 6.2 Create Audio Development Guide

**File:** `docs/guides/audio-development.md` (new file)

- **Action:** Write comprehensive guide for audio testing
- **Why:** Users need detailed explanation of audio mode
- **Dependencies:** All phases complete
- **Risk:** Low

**Topics to cover:**
- When to use audio mode vs browser mode
- Audio binary architecture (user code, not CLI)
- Customizing the audio binary
- Troubleshooting audio issues (xruns, latency)
- Performance tuning (buffer sizes, sample rates)
- macOS audio permissions

**Expected outcome:** Comprehensive audio development guide published.

---

### 6.3 Update High-Level Design Doc

**File:** `docs/architecture/high-level-design.md`

- **Action:** Document OS audio architecture
- **Why:** Maintain architectural reference
- **Dependencies:** All phases complete
- **Risk:** Low

**Sections to add:**
- OS Audio Development Mode diagram
- Audio binary compilation model
- WebSocket protocol extensions
- Comparison: Browser vs OS Audio modes

**Expected outcome:** Architecture docs reflect new capabilities.

---

### 6.4 Update Roadmap

**File:** `docs/roadmap.md`

- **Action:** Mark OS Audio feature as complete
- **Why:** Track project progress
- **Dependencies:** All phases complete, testing passed
- **Risk:** Low

**Changes:**
- Move "OS Audio Input" from In Progress to Completed
- Update version: 0.8.0
- Add changelog entry

**Expected outcome:** Roadmap reflects current state.

---

## Dependencies & Critical Path

```
Phase 1 (Protocol) → Phase 2 (Audio Binary) → Phase 3 (CLI) → Phase 4 (Template)
                                                                      ↓
                                                                 Phase 5 (Testing)
                                                                      ↓
                                                                 Phase 6 (Docs)
```

**Critical path:**
1. Protocol extensions (1.1-1.3)
2. AudioServer API (2.1)
3. Audio callback implementation (2.2)
4. CLI spawning logic (3.4)
5. Integration testing (5.2, 5.3)

**Parallelizable work:**
- Documentation (Phase 6) can start after Phase 3
- Template updates (Phase 4) can overlap with Phase 3
- Unit tests (5.1) can be written during Phase 1

---

## Testing Strategy

### Automated Tests

| Test Type | Location | What It Tests |
|-----------|----------|---------------|
| Unit | `wavecraft-protocol/src/messages.rs` | Protocol serialization |
| Unit | `wavecraft-dev-server/src/audio_server.rs` | Audio server logic |
| Integration | `cli/tests/audio_binary_compilation.rs` | Template compilation |
| Integration | `cli/tests/audio_error_handling.rs` | Error scenarios |

### Manual Tests

| Test | Scenario | Success Criteria |
|------|----------|------------------|
| Audio Flow | Mic → DSP → Speaker | Audio plays, <50ms latency |
| Parameter Update | Slider change in UI | Audio reflects change instantly |
| UI Hot Reload | Edit React component | UI updates, audio continues |
| Graceful Shutdown | Ctrl+C | All processes stop, no orphans |
| Missing Binary | `--with-audio` without binary | Clear error message |

### Performance Benchmarks

- **Latency:** <50ms round-trip (input → speakers)
- **Xruns:** <1% error rate at 256 samples
- **CPU:** <10% on 2020 MacBook Air
- **Memory:** <50MB for audio binary

---

## Risks & Mitigations

### High Risk

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Real-time xruns under load | High | Medium | Use proven cpal library, preallocate buffers, test thoroughly |
| WebSocket latency causes audio glitches | High | Low | Audio callback doesn't block on WebSocket, use lock-free ring buffer for meters |

### Medium Risk

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| macOS audio permissions confuse users | Medium | Medium | Document permission prompts, add troubleshooting section |
| Process lifecycle bugs (orphaned processes) | Medium | Low | Thorough testing of Ctrl+C and error paths |
| Template compilation errors | Medium | Low | CI test for template compilation |

### Low Risk

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Protocol backward compatibility | Low | Low | Version protocol, maintain old message types |
| Documentation incomplete | Low | Medium | Allocate time for comprehensive docs in Phase 6 |

---

## Success Criteria

- [ ] `wavecraft start --with-audio` compiles audio binary and starts all servers
- [ ] Audio flows from microphone → user's `Processor` → speakers
- [ ] Meters in UI reflect processed audio levels in real-time
- [ ] Parameter changes in UI affect audio processing within 50ms
- [ ] UI hot-reloading (Vite HMR) works while audio is running
- [ ] Audio latency <50ms on MacBook hardware
- [ ] <1% xrun rate at 256 sample buffer size
- [ ] Ctrl+C cleanly stops all processes (no orphans)
- [ ] Missing audio binary shows helpful error message
- [ ] Template-generated projects compile audio binary without errors
- [ ] Documentation covers audio workflow and troubleshooting
- [ ] All automated tests pass
- [ ] Manual testing validates end-to-end flow

---

## Implementation Order Recommendation

**Week 1: Core Infrastructure**
- Phase 1: Protocol Extensions (Steps 1.1-1.3)
- Phase 2: Audio Server Library (Steps 2.1-2.4)

**Week 2: CLI Integration**
- Phase 3: CLI Integration (Steps 3.1-3.6)
- Phase 4: Template Updates (Steps 4.1-4.3)

**Week 3: Testing & Polish**
- Phase 5: Testing (Steps 5.1-5.5)
- Phase 6: Documentation (Steps 6.1-6.4)

**Estimated total effort:** 15-20 development days (3 weeks with testing)

---

## Open Questions

1. **Q:** Should we support multiple audio devices (input/output selection)?
   **A:** Not for Phase 1. System default only. Can add device selection later.

2. **Q:** Should audio binary connect to existing WebSocket or spawn its own?
   **A:** Connect to CLI's existing WebSocket server (port passed via env).

3. **Q:** What happens if audio binary crashes?
   **A:** CLI detects exit, logs error, continues serving UI (graceful degradation).

4. **Q:** Should we add MIDI support in this phase?
   **A:** No, MIDI is a separate feature. Focus on audio input only.

5. **Q:** Should we support Windows/Linux in Phase 1?
   **A:** No, macOS only (consistent with project focus). Windows/Linux later.

---

## Related Documents

- [User Stories](./user-stories.md) — Feature requirements and acceptance criteria
- [Architect Q&A](./architect-response-os-audio.md) — Direct architectural answers
- [Full Analysis](./architectural-reevaluation-os-audio-reuse.md) — Complete technical evaluation
- [Constraints](./implementation-constraints-os-audio.md) — Non-negotiable boundaries
- [High-Level Design](../../architecture/high-level-design.md) — Overall project architecture
- [Coding Standards](../../architecture/coding-standards.md) — Implementation conventions
