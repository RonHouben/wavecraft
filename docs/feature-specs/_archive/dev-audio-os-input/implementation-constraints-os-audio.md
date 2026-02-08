# OS Audio Implementation: Architectural Constraints Checklist

**Date:** 2026-02-08  
**Status:** Implementation Requirements  
**Context:** [Architect Response](./architect-response-os-audio.md) | [Full Analysis](./architectural-reevaluation-os-audio-reuse.md)

---

## Mandatory Architectural Constraints

These constraints **must be enforced** during implementation. Deviations require architectural review.

---

### ✅ 1. Audio Processing Lives in User Code, Not CLI

**Constraint:** The CLI **must not** contain audio processing logic.

**Enforcement:**

```rust
// ❌ FORBIDDEN in CLI crates:
pub struct AudioDevServer {
    audio_stream: cpal::Stream,  // NO
    processor: Box<dyn Processor>, // NO
}

// ✅ CORRECT: CLI spawns external process
spawn_process("cargo", &["run", "--bin", "dev-audio"])?;
```

**Rationale:** Separation of concerns. CLI is tooling infrastructure, not an audio engine.

---

### ✅ 2. Development Server Must Be a Library, Not Executable

**Constraint:** The `wavecraft-dev-server` crate **must** export a generic library interface.

**Enforcement:**

```rust
// ✅ CORRECT: Generic library (in wavecraft-dev-server crate)
pub struct AudioDevServer<P: Processor> {
    processor: P,
    config: Config,
    // ...
}

impl<P: Processor> AudioDevServer<P> {
    pub fn new(processor: P, config: Config) -> Result<Self> {
        // Generic setup, no knowledge of concrete Processor type
    }
    
    pub fn run(self) -> Result<()> {
        // Generic audio I/O loop
    }
}
```

**Rationale:** User projects instantiate their concrete `Processor` types. The SDK provides infrastructure, not DSP.

---

### ✅ 3. User's Project Compiles the Audio Binary

**Constraint:** Audio processing binary **must be compiled as part of user's project**.

**Enforcement:**

Template structure:
```
my-plugin/
├── engine/
│   ├── Cargo.toml
│   │   [dev-dependencies]
│   │   wavecraft-dev-server = { ... }  # ← Optional dependency
│   │
│   └── src/
│       ├── lib.rs              (plugin exports)
│       ├── dsp.rs              (Processor impl)
│       └── bin/
│           └── dev-audio.rs     # ← User compiles this
```

**`dev-audio.rs` template:**

```rust
use my_plugin::MyGainProcessor;  // User's concrete type
use wavecraft_dev_server::{AudioDevServer, Config};

fn main() -> anyhow::Result<()> {
    let processor = MyGainProcessor::default();
    let config = Config::from_env()?;
    
    let server = AudioDevServer::new(processor, config)?;
    server.run()
}
```

**Rationale:** Only the user's project knows the concrete `Processor` type. CLI cannot compile with user DSP.

---

### ✅ 4. CLI Spawns Binary, Doesn't Embed It

**Constraint:** CLI **must spawn** `cargo run --bin dev-audio` as external process.

**Enforcement:**

```rust
// In wavecraft CLI: src/commands/start.rs

if args.with_audio {
    // Check if user has dev-audio binary
    let cargo_toml = std::fs::read_to_string("engine/Cargo.toml")?;
    let has_dev_audio = cargo_toml.contains("bin/dev-audio");
    
    if has_dev_audio {
        // ✅ CORRECT: Spawn external process
        let mut child = Command::new("cargo")
            .args(["run", "--bin", "dev-audio"])
            .current_dir("engine")
            .spawn()?;
            
        // Store child handle for cleanup
        dev_processes.push(child);
    } else {
        // ✅ CORRECT: Graceful fallback
        eprintln!("⚠️  No dev-audio binary found.");
        eprintln!("   Add engine/src/bin/dev-audio.rs to enable audio.");
        eprintln!("   Running without audio integration.");
    }
}
```

**Rationale:** Process isolation. CLI coordinates, user binary runs audio.

---

### ✅ 5. Communication via Protocol (WebSocket)

**Constraint:** CLI and dev-audio binary **must communicate via WebSocket/JSON-RPC**, not shared memory or other mechanisms.

**Enforcement:**

```rust
// In dev-audio binary
let ws_client = WebSocketClient::connect("ws://127.0.0.1:9000")?;

// On meter updates (from audio thread via ring buffer)
ws_client.send_meter_frame(meter_frame)?;

// On parameter changes from UI
ws_client.on_set_parameter(|id, value| {
    // Update processor params atomically
});
```

**Rationale:** 
- Clean separation via versioned protocol
- Same interface as browser WebSocket mode
- Easy to debug (inspect JSON-RPC messages)

---

### ✅ 6. Feature Must Be Opt-In

**Constraint:** Audio integration **must require explicit flag**, not default behavior.

**Enforcement:**

```bash
# ❌ WRONG: Audio by default
wavecraft start  # Should NOT start audio automatically

# ✅ CORRECT: Explicit opt-in
wavecraft start --with-audio
```

**Rationale:** 
- Audio setup is complex (drivers, permissions)
- Core use case (IPC testing) should remain simple
- Users without audio hardware should not be blocked

---

### ✅ 7. No Real-Time Safety Burden on CLI

**Constraint:** CLI **must not** be responsible for real-time safety of audio thread.

**Enforcement:**

```rust
// ❌ FORBIDDEN: CLI managing audio thread
pub struct CliDevServer {
    audio_thread: JoinHandle<()>,  // NO
    // ...real-time concerns in CLI
}

// ✅ CORRECT: CLI coordinates processes only
pub struct DevServerCoordinator {
    vite_process: Child,
    websocket_server: WebSocketServer,
    dev_audio_process: Option<Child>,  // Just a process handle
}
```

**Rationale:** Real-time audio requires deep expertise. User code handles it (same code as plugin). CLI should never panic on audio underruns.

---

### ✅ 8. Template Includes Working Example

**Constraint:** SDK template **must include** a functional `dev-audio.rs` by default.

**Enforcement:**

```
cli/sdk-templates/new-project/react/engine/src/bin/dev-audio.rs.template

Should contain:
- Complete working example
- Instantiation of template's Processor
- Config loading
- Error handling
- Helpful comments
```

**Rationale:** Discoverability. New users should find a working example, not start from scratch.

---

## Testing Requirements

### Pre-Merge Checklist

- [ ] CLI can spawn `dev-audio` binary successfully
- [ ] CLI detects missing `dev-audio` binary gracefully (no panic)
- [ ] WebSocket communication works (meters flow UI → CLI → dev-audio → CLI → UI)
- [ ] Parameter changes propagate (UI → dev-audio → Processor)
- [ ] Audio doesn't glitch when changing parameters
- [ ] CLI cleans up `dev-audio` process on exit (Ctrl+C handling)
- [ ] Works on macOS (CoreAudio)
- [ ] Windows support documented (WASAPI) but not required for Phase 1
- [ ] Template's `dev-audio.rs` compiles without modification

---

## Non-Negotiable Boundaries

| Boundary | Enforced By |
|----------|-------------|
| **CLI does not process audio** | Code review: no cpal/audio deps in CLI crates |
| **User project compiles audio binary** | Template structure, documentation |
| **Protocol-based communication** | IPC handler enforces JSON-RPC |
| **Opt-in feature** | CLI arg parsing requires `--with-audio` |
| **Graceful degradation** | CI test: CLI runs without audio binary present |

---

## Architectural Validation

Before merging OS audio implementation, verify:

1. **Unit test:** Can spawn external binary and receive WebSocket messages
2. **Integration test:** Template project's `dev-audio` compiles and runs
3. **Code review:** No audio logic in CLI crates (grep for `cpal`, `Processor`, etc.)
4. **Documentation:** Clear explanation of when to use `--with-audio` vs browser mode

---

## Future Compatibility

This architecture **must not prevent** adding WASM support later:

```bash
wavecraft start                  # Current: synthetic meters
wavecraft start --with-audio     # Phase 1: OS audio via dev-audio binary
wavecraft start --wasm           # Phase 2: Browser WASM audio (future)
```

All three modes should coexist without conflict.

---

**See also:**
- [Architect Response](./architect-response-os-audio.md) — Answers to PO questions
- [Full Re-evaluation](./architectural-reevaluation-os-audio-reuse.md) — Complete analysis
