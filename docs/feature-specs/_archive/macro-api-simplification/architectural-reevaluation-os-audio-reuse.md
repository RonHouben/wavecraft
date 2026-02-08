# Architectural Re-Evaluation: OS Audio with Code Reuse

**Status:** Architectural Analysis  
**Created:** 2026-02-08  
**Author:** Architect Agent  
**Context:** [User Stories](./user-stories.md) | [Previous Assessment](./architectural-assessment-dev-audio-input.md)

---

## Executive Summary

**The code reuse clarification fundamentally changes the architectural evaluation.**

When the dev server **instantiates the same `Processor` implementation** used in the plugin (rather than duplicating DSP logic), the primary objection from the previous assessment is eliminated. However, significant architectural concerns remain around:

1. **Compilation model** — How does the CLI compile with user DSP code?
2. **Architectural identity** — Is the dev server still tooling, or has it become an audio engine?
3. **Coupling patterns** — What dependencies exist between CLI and user projects?
4. **Separation of concerns** — Where should audio processing responsibility live?

**Revised recommendation:** OS audio with code reuse is architecturally viable **if implemented as a user-compiled binary, not as CLI infrastructure**. This preserves separation while enabling the desired testing workflow.

---

## What Changed: Code Reuse vs. Duplication

### Previous Understanding (Incorrect)

```
Dev Server (in CLI)
├── OS audio input (cpal)
├── ⚠️ DSP logic reimplemented in JS/Rust
└── Divergence risk: dev server DSP ≠ plugin DSP
```

**Problem:** Two implementations of the same audio processing.

### Clarified Proposal (Correct)

```
Dev Server
├── OS audio input (cpal)
├── ✅ User's Processor instance (same code as plugin)
└── No duplication: one implementation, two execution contexts
```

**Key insight:** The `Processor` trait provides an abstract interface. The dev server could instantiate the same concrete type (`MyGainProcessor`, `MyReverbProcessor`, etc.) that the plugin uses.

---

## Architectural Analysis

### 1. Does This Address Code Duplication?

**Yes, completely.**

There is now only one implementation of the audio processing logic:

```rust
// User's DSP (in their project)
struct MyGainProcessor {
    sample_rate: f32,
}

impl Processor for MyGainProcessor {
    type Params = MyGainParams;
    
    fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport, params: &Self::Params) {
        for channel in buffer.iter_mut() {
            for sample in channel.iter_mut() {
                *sample *= params.gain;
            }
        }
    }
}
```

This code runs in **both contexts**:

| Context | How It's Used |
|---------|---------------|
| **Plugin** | `WavecraftEditor<MyGainParams>` wraps `MyGainProcessor` via `wavecraft_plugin!` macro |
| **Dev Server** | Dev server instantiates `MyGainProcessor` directly, feeds it OS audio |

No duplication. Same Rust code, same compiler passes, same real-time safety guarantees.

**Verdict:** ✅ The duplication concern is resolved.

---

### 2. Architectural Implications of Dev Server Instantiating `Processor`

This is where the architecture becomes complex. Let's map out the dependency flow.

#### Compilation Model Options

**Option A: Dev Server as Generic Library**

```rust
// In wavecraft-dev-server crate
pub struct AudioDevServer<P: Processor> {
    processor: P,
    audio_stream: cpal::Stream,
    params: P::Params,
}

impl<P: Processor> AudioDevServer<P> {
    pub fn new(processor: P) -> Self {
        // Set up audio I/O, instantiate processor
    }
}
```

**Usage in user project:**

```rust
// In user's project: src/bin/dev-audio.rs
use wavecraft_dev_server::AudioDevServer;
use my_plugin::MyGainProcessor;

fn main() {
    let processor = MyGainProcessor::new();
    let server = AudioDevServer::new(processor);
    server.run();
}
```

**Implications:**
- ✅ Clean generic interface
- ✅ User project controls instantiation
- ✅ No CLI involvement in DSP
- ⚠️ User must run `cargo run --bin dev-audio` separately from CLI

---

**Option B: CLI Spawns User Binary**

```bash
# User runs this
wavecraft start --with-audio

# CLI internally spawns:
cd $USER_PROJECT && cargo run --bin dev-audio
```

**Implications:**
- ✅ Unified workflow (single command)
- ✅ CLI remains coordinator, not audio engine
- ⚠️ Requires convention: user project must have `dev-audio` binary
- ⚠️ CLI must parse Cargo.toml to find binary

---

**Option C: CLI Compiles User DSP (Dynamic Linking)**

```rust
// In CLI
let plugin = libloading::Library::new("target/debug/libmy_plugin.so")?;
let create_processor: Symbol<fn() -> Box<dyn Processor>> = 
    plugin.get(b"create_processor")?;

let processor = create_processor();
```

**Implications:**
- ✅ CLI remains self-contained binary
- ❌ Complex: requires C ABI, trait objects, dynamic symbol resolution
- ❌ Real-time safety burden: CLI now responsible for audio thread correctness
- ❌ Platform-specific: dylib loading varies wildly

---

#### Recommended Compilation Model

**Use Option A (user-compiled binary) with Option B (CLI spawns it).**

```
┌─────────────────────────────────────────────────────────────────┐
│                   RECOMMENDED ARCHITECTURE                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  wavecraft start --with-audio                                   │
│         │                                                       │
│         ├──► Spawn Vite (UI dev server)                         │
│         ├──► Spawn WebSocket server (IPC bridge)                │
│         └──► Spawn: cargo run --bin dev-audio                   │
│                     │                                           │
│                     ▼                                           │
│            ┌─────────────────────┐                              │
│            │  User's Project     │                              │
│            │  (dev-audio binary) │                              │
│            │                     │                              │
│            │  • MyGainProcessor  │                              │
│            │  • cpal audio I/O   │                              │
│            │  • WebSocket client │                              │
│            │  • Meter extraction │                              │
│            └──────────┬──────────┘                              │
│                       │                                         │
│                       │ WebSocket                               │
│                       │ (meters + params)                       │
│                       ▼                                         │
│              CLI WebSocket Server ──► Browser UI                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key architectural properties:**

1. **CLI is pure coordinator**: spawns processes, manages WebSocket bridge
2. **User project compiles audio binary**: links against `wavecraft-dev-server` helper crate
3. **Separation of concerns**: audio processing lives in user code, not tooling
4. **Communication via protocol**: WebSocket/JSON-RPC, same as browser mode

---

### 3. Comparison to WASM Approach

With code reuse, both approaches are now viable. Let's compare:

| Aspect | OS Audio (User Binary) | WASM (Browser AudioWorklet) |
|--------|------------------------|------------------------------|
| **Code duplication** | ✅ None (same Processor) | ✅ None (same Processor) |
| **Compilation target** | Native executable | `wasm32-unknown-unknown` |
| **Audio source** | OS mic (cpal) | Browser APIs (getUserMedia/file) |
| **Real-time handling** | User binary (Rust) | Browser (JS runtime) |
| **Execution context** | Dev server sidecar | AudioWorkletProcessor |
| **Setup complexity** | Medium (cpal deps) | High (WASM build + COOP/COEP headers) |
| **Platform deps** | Yes (CoreAudio/WASAPI) | No (browser only) |
| **Latency** | Lower (~5-10ms) | Higher (~20-50ms, browser buffering) |
| **Use case** | Integration testing | UI-focused development |

**Key difference:**

- **OS Audio**: Better for testing actual audio behavior (latency, CPU, glitches)
- **WASM**: Better for rapid UI iteration without audio setup

**Architectural verdict:** They serve different purposes and could **coexist**.

---

### 4. Separation of Concerns: Is Dev Server Still Tooling?

This is the critical architectural question.

#### Current Role: Pure Testing Harness

```
Dev Server (current):
├── Loads plugin .dylib (FFI parameter discovery only)
├── Stores parameter state (InMemoryParameterHost)
├── Generates synthetic meters (MeterGenerator)
└── Bridges WebSocket ↔ UI

Identity: IPC/parameter testing tool
```

#### With OS Audio: Hybrid Identity

```
Dev Server (with OS audio):
├── Coordinates processes (CLI)
├── WebSocket IPC bridge (CLI)
└── Spawns: dev-audio binary (user project)
    ├── OS audio I/O (cpal)
    ├── Real-time audio thread
    ├── User's Processor instance
    └── Meter extraction

Identity: ??? (coordinator + audio system)
```

**Has the line been crossed?**

**My assessment: Yes, but appropriately so.**

The dev server's identity has evolved from:
- **"Parameter testing tool"**

To:
- **"Development environment for plugin UIs"**

This is a **legitimate expansion of scope** if:

1. The audio processing **remains in user code** (not CLI internals)
2. The CLI's role is **coordination**, not audio engineering
3. The feature is **opt-in** (`--with-audio` flag)
4. It doesn't compromise the **core use case** (IPC/UI testing without audio)

---

### 5. Coupling Analysis

Let's map the dependencies:

#### Current Architecture (No Audio)

```
CLI (wavecraft)
├── wavecraft-dev-server (generic, no user code)
└── Scaffolds → User Project
                └── Uses CLI via: wavecraft start
```

- **No coupling**: CLI and user project are independent
- **Communication**: WebSocket IPC (synthetic data)

#### With OS Audio (Reuse Approach)

```
CLI (wavecraft)
├── wavecraft-dev-server (generic audio server library)
└── Scaffolds → User Project
                ├── Depends on: wavecraft-dev-server (Cargo.toml)
                ├── Binary: src/bin/dev-audio.rs
                │   └── Instantiates: MyGainProcessor
                └── Uses CLI via: wavecraft start --with-audio
                    └── CLI spawns: cargo run --bin dev-audio
```

**New coupling:**

| Type | Direction | Strength |
|------|-----------|----------|
| **Build-time** | User project → `wavecraft-dev-server` crate | Weak (optional dev-dependency) |
| **Runtime** | CLI → User binary (via `cargo run`) | Medium (convention-based) |
| **Protocol** | User binary ↔ CLI WebSocket | Weak (versioned JSON-RPC) |

**Coupling verdict:** Acceptable if:
- The `wavecraft-dev-server` crate is **stable and optional**
- The `dev-audio` binary is **convention, not requirement** (template includes it, users can delete)
- The CLI **detects presence** of `dev-audio` rather than requiring it

---

## Revised Risk Assessment

### Risks Eliminated by Code Reuse

| Risk | Previous | Now |
|------|----------|-----|
| **DSP duplication** | High | ✅ Eliminated |
| **Divergence** | High | ✅ Eliminated |
| **Maintenance burden** | High | ✅ Eliminated |

### Remaining Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Audio driver complexity** | Medium | Isolate in user binary, not CLI |
| **Platform dependencies** | Medium | Optional feature, document requirements |
| **Real-time safety testing** | Low | Same Processor code as plugin |
| **Compilation model** | Medium | Clear convention: `src/bin/dev-audio.rs` |
| **CLI scope creep** | Low | CLI remains coordinator, not audio engine |
| **Feature discoverability** | Low | Template includes `dev-audio` by default |

---

## Architectural Recommendation

### Recommended: Hybrid Approach with Clear Boundaries

**Implementation pattern:**

1. **SDK provides library**: `wavecraft-dev-server` crate with generic `AudioDevServer<P: Processor>`

2. **Template includes binary**: `src/bin/dev-audio.rs` that instantiates user's `Processor`

3. **CLI spawns binary**: `wavecraft start --with-audio` detects and runs the binary

4. **Communication via protocol**: User binary → CLI WebSocket → Browser UI

**Template structure:**

```
my-plugin/
├── engine/
│   └── src/
│       ├── lib.rs              (plugin exports)
│       ├── dsp.rs              (Processor impl)
│       └── bin/
│           └── dev-audio.rs    (← NEW: optional dev server)
├── ui/
│   └── src/App.tsx
└── README.md
```

**`dev-audio.rs` example:**

```rust
use my_plugin::MyGainProcessor;
use wavecraft_dev_server::{AudioDevServer, Config};

fn main() -> anyhow::Result<()> {
    let config = Config {
        websocket_port: 9000,
        audio_buffer_size: 256,
        ..Default::default()
    };
    
    let processor = MyGainProcessor::default();
    let server = AudioDevServer::new(processor, config)?;
    
    println!("🎵 Dev audio server running on :9000");
    server.run()?;
    
    Ok(())
}
```

**CLI integration:**

```rust
// In wavecraft start command
if args.with_audio {
    if project_has_dev_audio_binary()? {
        spawn_process("cargo", &["run", "--bin", "dev-audio"])?;
    } else {
        warn!("No dev-audio binary found. Run without --with-audio or add src/bin/dev-audio.rs");
    }
}
```

---

## Comparison to WASM Approach (Revised)

Both approaches are now architecturally sound. Here's when to use each:

| Scenario | OS Audio | WASM |
|----------|----------|------|
| **UI iteration with real audio** | ✅ Best | ⚠️ High setup cost |
| **Testing CPU/latency** | ✅ Best | ❌ Browser too different |
| **No audio hardware available** | ❌ Requires mic | ✅ Works everywhere |
| **Quick prototyping** | ⚠️ Needs binary | ✅ Just `npm run dev` |
| **Integration testing** | ✅ Same code as plugin | ✅ Same code as plugin |
| **Shareable demos** | ❌ Can't distribute | ✅ Just a URL |

**Architectural verdict:**

Both approaches are complementary:

- **OS Audio**: For developers with audio hardware who want integration testing
- **WASM**: For UI-focused development and demos

**They can coexist** with different modes:

```bash
wavecraft start                    # Browser (no audio)
wavecraft start --with-audio       # OS audio via dev-audio binary
wavecraft start --wasm             # Browser WASM audio (future)
```

---

## Answers to Specific Questions

### 1. Does this address the "code duplication" concern?

**Yes, completely.** There is only one `Processor` implementation. The dev server instantiates the same type the plugin uses. No divergence risk.

---

### 2. What are the architectural implications of having the dev server instantiate `Processor`?

**Implications:**

- **Compilation model**: User project must compile the `dev-audio` binary
- **Dependency graph**: User project depends on `wavecraft-dev-server` crate
- **Execution context**: CLI spawns user binary, which runs audio I/O
- **Separation**: CLI remains coordinator; audio lives in user code

**Acceptable if**: The `dev-audio` binary is part of the **user's project**, not the CLI internals.

---

### 3. How does this compare to the WASM approach now that duplication is eliminated?

**Both are viable.** Key differences:

| WASM | OS Audio |
|------|----------|
| Browser AudioWorklet | Native binary (cpal) |
| Higher setup complexity | Requires audio hardware |
| Good for UI dev | Good for integration testing |
| Shareable demos | Performance validation |

**They serve different use cases and could coexist.**

---

### 4. What about the "separation of concerns" - is the dev server still a testing harness, or has it become an audio engine?

**It has become a hybrid: "Development environment for plugin UIs"**

This is acceptable because:

- The audio engine **remains in user code** (not CLI)
- The CLI's role is **process coordination** (spawning binaries, WebSocket bridge)
- The core use case (IPC testing) **still works without audio**

The identity evolution is:
- Before: "Parameter/IPC testing tool"
- After: "Development environment with optional audio integration"

---

### 5. Would this create coupling between dev tooling and user DSP code?

**Yes, but manageable coupling:**

- **Build-time**: User project optionally depends on `wavecraft-dev-server` crate
- **Runtime**: CLI spawns user's `dev-audio` binary via `cargo run`
- **Protocol**: Communication via versioned WebSocket/JSON-RPC

**Coupling is acceptable because:**
- It's **opt-in** (user can delete `dev-audio.rs`)
- It follows Rust ecosystem patterns (`cargo test`, `cargo bench` also compile user code)
- The `wavecraft-dev-server` crate is **library code**, not deep integration

---

## Final Recommendation

**Approve OS audio approach with the following constraints:**

### ✅ Implementation Requirements

1. **Audio binary lives in user project**: `src/bin/dev-audio.rs` in template
2. **CLI spawns binary, doesn't embed audio**: `cargo run --bin dev-audio` via CLI
3. **Opt-in feature**: `wavecraft start --with-audio` (defaults to synthetic meters)
4. **Clear error handling**: Graceful fallback if `dev-audio` binary doesn't exist
5. **Protocol communication**: User binary → CLI WebSocket → Browser

### ✅ SDK Structure

```
wavecraft/
├── cli/
│   └── src/commands/start.rs    (spawns dev-audio binary)
├── engine/crates/
│   └── wavecraft-dev-server/    (NEW: audio server library)
│       ├── src/
│       │   ├── lib.rs           (AudioDevServer<P: Processor>)
│       │   ├── audio.rs         (cpal setup)
│       │   └── websocket.rs     (client to CLI)
│       └── Cargo.toml
└── cli/sdk-templates/new-project/react/
    └── engine/src/bin/
        └── dev-audio.rs         (user's audio server)
```

### ✅ Future Compatibility

This design allows adding WASM support later without conflict:

```bash
wavecraft start                  # Browser (synthetic meters)
wavecraft start --with-audio     # OS audio (dev-audio binary)
wavecraft start --wasm           # Browser WASM audio (future)
```

---

## Conclusion

**The code reuse clarification resolves the primary architectural objection.**

The OS audio approach is **architecturally sound** when implemented as:
- A **library** (`wavecraft-dev-server`)
- Compiled by the **user's project** (`dev-audio` binary)
- Spawned by the **CLI** (coordinator role)
- Communicating via **protocol** (WebSocket)

This preserves separation of concerns, eliminates duplication, and provides valuable integration testing capability.

**Status: ✅ Architecturally approved** with implementation constraints as specified.
