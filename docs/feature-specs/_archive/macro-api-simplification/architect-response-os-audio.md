# OS Audio Approach: Architect's Response

**Date:** 2026-02-08  
**To:** Product Owner  
**From:** Architect Agent  
**Re:** OS Audio Input with Code Reuse

---

## Direct Answers

### 1. Does this address the "code duplication" concern?

**✅ Yes, completely.**

With code reuse, there is only one `Processor` implementation. The dev server instantiates the same concrete type (`MyGainProcessor`, `MyReverbProcessor`, etc.) that the plugin uses. No divergence risk.

---

### 2. What are the architectural implications of having the dev server instantiate `Processor`?

**Key implications:**

**Compilation Model:**
- User's project must compile a `dev-audio` binary that links their `Processor` with audio I/O
- CLI cannot directly instantiate user DSP (would require dynamic loading with trait objects)

**Recommended Pattern:**
```
CLI (wavecraft start --with-audio)
├── Spawns: Vite (UI server)
├── Spawns: WebSocket server (IPC bridge)  
└── Spawns: cargo run --bin dev-audio
            └── User's binary:
                ├── MyGainProcessor (user code)
                ├── OS audio I/O (cpal)
                └── WebSocket client → CLI
```

**Dependency Flow:**
- User project depends on new `wavecraft-dev-server` crate (library, not executable)
- Template includes `src/bin/dev-audio.rs` by default
- CLI detects and spawns the binary (graceful fallback if missing)

**Clear Boundary:**
- CLI remains **coordinator** (spawns processes, bridges WebSocket)
- User binary is the **audio engine** (runs Processor, handles real-time)

---

### 3. How does this compare to the WASM approach now that duplication is eliminated?

**Both are architecturally sound. They serve different purposes:**

| Aspect | OS Audio | WASM |
|--------|----------|------|
| **Use case** | Integration testing with real audio | UI-focused development |
| **Audio source** | OS mic (cpal) | Browser APIs (mic/file/tone) |
| **Latency** | Lower (~5-10ms) | Higher (~20-50ms) |
| **Setup** | Requires audio hardware | Works everywhere (browser only) |
| **Performance testing** | ✅ Actual CPU/buffer sizes | ❌ Browser abstracts this |
| **Shareable demos** | ❌ Can't distribute binaries | ✅ Just a URL |

**Recommendation: Implement both.**

They address different developer workflows:
- OS Audio: "I want to test actual audio behavior with my Processor"
- WASM: "I want to iterate quickly on UI without audio setup"

```bash
wavecraft start                  # Browser (synthetic meters)
wavecraft start --with-audio     # OS audio integration testing
wavecraft start --wasm           # Browser WASM audio (future)
```

---

### 4. Is the dev server still a testing harness, or has it become an audio engine?

**It has evolved, and that's acceptable.**

**Previous identity:**
- "IPC/parameter testing tool"

**New identity:**
- "Development environment for plugin UIs"

**This is architecturally sound because:**

1. **Audio processing lives in user code**, not CLI internals
2. **CLI remains coordinator**: spawns processes, bridges WebSocket
3. **Core use case preserved**: IPC testing still works without audio
4. **Opt-in feature**: `--with-audio` flag required

The dev server is now a **hybrid tool**: simple IPC testing OR full audio integration, depending on flags.

---

### 5. Would this create coupling between dev tooling and user DSP code?

**Yes, but it's manageable and follows Rust ecosystem patterns.**

**Coupling created:**

| Type | Details | Precedent |
|------|---------|-----------|
| **Build-time** | User project depends on `wavecraft-dev-server` crate | Like `criterion` for benchmarks |
| **Runtime** | CLI spawns `cargo run --bin dev-audio` | Like `cargo test` compiling user tests |
| **Protocol** | WebSocket/JSON-RPC communication | Versioned, backward-compatible |

**Mitigation strategies:**
- `wavecraft-dev-server` is **optional dev-dependency** in template
- `dev-audio` binary is **convention, not requirement** (users can delete it)
- CLI **detects presence** rather than requiring it (graceful fallback)

**Examples from Rust ecosystem:**

```toml
# Similar patterns in mature projects:
[dev-dependencies]
criterion = "0.5"     # Benchmarking (compiles with user code)
proptest = "1.0"      # Property testing (same)
```

The coupling is **intentional and opt-in**, not architectural debt.

---

## Architectural Approval: ✅ Approved with Constraints

### Required Implementation Pattern

**1. Audio Server as Library (Not Executable)**

```
wavecraft/engine/crates/wavecraft-dev-server/
├── src/
│   ├── lib.rs                  (pub struct AudioDevServer<P: Processor>)
│   ├── audio.rs                (cpal setup, real-time thread)
│   └── websocket_client.rs     (connects to CLI's WebSocket)
└── Cargo.toml
```

**2. User's Project Compiles Audio Binary**

```
my-plugin/engine/src/bin/dev-audio.rs:

use my_plugin::MyGainProcessor;
use wavecraft_dev_server::{AudioDevServer, Config};

fn main() -> anyhow::Result<()> {
    let processor = MyGainProcessor::default();
    let server = AudioDevServer::new(processor, Config::default())?;
    server.run()
}
```

**3. CLI Spawns User Binary (If Present)**

```rust
// In wavecraft start command
if args.with_audio {
    if project_has_binary("dev-audio")? {
        spawn_process("cargo", &["run", "--bin", "dev-audio"])?;
    } else {
        eprintln!("⚠️  No dev-audio binary found (add engine/src/bin/dev-audio.rs)");
        eprintln!("   Running without audio integration.");
    }
}
```

---

## Comparison Matrix

|  | Current (No Audio) | OS Audio (Reuse) | WASM (Browser) |
|--|-------------------|------------------|----------------|
| **Code duplication** | N/A | ✅ None | ✅ None |
| **Audio source** | Synthetic | OS mic | Browser APIs |
| **Setup complexity** | Low | Medium | High (WASM build) |
| **Integration testing** | ❌ No audio | ✅ Real audio | ⚠️ Browser environment |
| **UI iteration speed** | ✅ Instant HMR | ✅ Instant HMR | ✅ Instant HMR |
| **Shareable demos** | N/A | ❌ No | ✅ Yes |
| **Platform deps** | None | Yes (cpal) | None (browser) |

---

## Recommendation

**✅ Approve OS audio approach** with the following constraints:

1. **Implement as user-compiled binary** (`src/bin/dev-audio.rs` in template)
2. **CLI spawns binary, doesn't embed audio logic**
3. **Opt-in via `--with-audio` flag** (defaults to synthetic meters)
4. **Clear error handling** (graceful fallback if binary doesn't exist)
5. **Protocol communication** (user binary → CLI WebSocket → browser)

**Next steps:**

1. **Phase 1:** OS audio (immediate value for integration testing)
2. **Phase 2:** WASM audio (future, for UI-focused workflows)

Both approaches are complementary and can coexist. OS audio provides better integration testing; WASM provides better accessibility.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│              APPROVED ARCHITECTURE (OS AUDIO)                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  $ wavecraft start --with-audio                                 │
│         │                                                       │
│         ├──► Vite (UI dev server)                               │
│         │                                                       │
│         ├──► WebSocket Server (port 9000)                       │
│         │    (IPC bridge: meters + params)                      │
│         │                                                       │
│         └──► cargo run --bin dev-audio                          │
│                     │                                           │
│                     ▼                                           │
│            ┌─────────────────────┐                              │
│            │  User's Project     │                              │
│            │  (engine/src/bin/   │                              │
│            │   dev-audio.rs)     │                              │
│            │                     │                              │
│            │  ┌───────────────┐  │                              │
│            │  │ MyProcessor   │  │  (same code as plugin)       │
│            │  │ (user's DSP)  │  │                              │
│            │  └───────┬───────┘  │                              │
│            │          │          │                              │
│            │          ▼          │                              │
│            │  ┌───────────────┐  │                              │
│            │  │ cpal Audio I/O│  │  (OS mic input)              │
│            │  │ Real-time     │  │                              │
│            │  └───────┬───────┘  │                              │
│            │          │          │                              │
│            │          ▼          │                              │
│            │  ┌───────────────┐  │                              │
│            │  │ Meter Extract │  │                              │
│            │  └───────┬───────┘  │                              │
│            │          │          │                              │
│            │          ▼          │                              │
│            │  ┌───────────────┐  │                              │
│            │  │ WebSocket     │  │  (client to CLI port 9000)   │
│            │  │ Client        │  │                              │
│            │  └───────────────┘  │                              │
│            └──────────┬──────────┘                              │
│                       │                                         │
│                       │ JSON-RPC                                │
│                       │ (meters + param ACKs)                   │
│                       ▼                                         │
│              CLI WebSocket Server                               │
│                       │                                         │
│                       ▼                                         │
│                  Browser UI                                     │
│                  (useParameter,                                 │
│                   useMeterFrame)                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key architectural properties:**
- ✅ **No duplication**: Single Processor implementation
- ✅ **Clear separation**: CLI = coordinator, user binary = audio engine
- ✅ **Opt-in complexity**: Audio only when needed (`--with-audio`)
- ✅ **Protocol communication**: Versioned WebSocket interface
- ✅ **Real-time safety**: Same guarantees as plugin (same code)

---

## Risk Assessment (Revised)

| Risk | Previous | Now | Mitigation |
|------|----------|-----|------------|
| **Code duplication** | 🔴 High | ✅ Eliminated | Same Processor in both contexts |
| **Divergence** | 🔴 High | ✅ Eliminated | Single implementation |
| **Audio driver complexity** | 🟡 Medium | 🟡 Medium | Isolated in user binary |
| **Compilation model** | N/A | 🟡 Medium | Clear convention in template |
| **CLI scope creep** | 🔴 High | ✅ Low | CLI remains coordinator only |

**Status:** Architecturally approved for implementation.

---

**Full technical analysis:** [architectural-reevaluation-os-audio-reuse.md](./architectural-reevaluation-os-audio-reuse.md)
