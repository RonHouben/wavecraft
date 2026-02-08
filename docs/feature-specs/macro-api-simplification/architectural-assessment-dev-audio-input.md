# Architectural Assessment: Development Mode Audio Input Strategy

**Status:** Assessment  
**Created:** 2026-02-08  
**Author:** Architect Agent  
**Context:** [User Stories](./user-stories.md) | [Audio Input via WASM](../audio-input-via-wasm/high-level-design.md)

---

## Executive Summary

**Recommendation: Browser-based WASM audio input, not OS audio in dev server.**

The proposal to add OS audio input directly to the dev server is **technically feasible but architecturally incorrect**. It violates core design principles by:

1. Blurring the separation between development tooling and production plugin code
2. Duplicating audio processing infrastructure outside the plugin boundary
3. Introducing real-time constraints into what should be a pure IPC/parameter testing tool
4. Creating cross-platform audio dependency complexity in the wrong layer

The existing WASM proposal maintains clean architectural boundaries and addresses the same need without compromising the system's conceptual integrity.

---

## Current Architecture Analysis

### Dev Server Purpose (As Designed)

The `wavecraft start` dev server has a **single, focused responsibility**:

```
┌─────────────────────────────────────────────────────────────────┐
│                   CURRENT DEV SERVER SCOPE                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌──────────────┐                                              │
│   │  User Plugin │  FFI parameter discovery                     │
│   │  (dylib)     │  ────────────►                              │
│   └──────────────┘               │                             │
│                                  ▼                             │
│                       ┌────────────────────┐                    │
│                       │  DevServerHost     │                    │
│                       │  • Parameter CRUD  │                    │
│                       │  • Synthetic meters│ (MeterGenerator)   │
│                       │  • State storage   │                    │
│                       └─────────┬──────────┘                    │
│                                 │                              │
│                                 │ JSON-RPC                     │
│                                 ▼                              │
│                       ┌────────────────────┐                    │
│                       │  WebSocket Server  │                    │
│                       │  (IPC transport)   │                    │
│                       └─────────┬──────────┘                    │
│                                 │                              │
│                                 ▼                              │
│                            Browser UI                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key characteristic:** No audio processing. Pure IPC layer.

The `MeterGenerator` produces **synthetic sine wave data** to test UI rendering, not to reflect real audio:

```rust
// wavecraft-metering/src/dev.rs
pub fn frame(&self) -> MeterFrame {
    let sine = self.phase.sin();  // Animated test data
    let cosine = (self.phase * 1.3).cos();
    
    let left_db = self.base_level_db + sine * self.modulation_db;
    let right_db = self.base_level_db + cosine * self.modulation_db;
    // ...
}
```

This design is intentional: fast iteration on UI/IPC without audio complexity.

---

## Proposal 1: OS Audio Input in Dev Server

### What It Would Require

To add OS audio input to the dev server, you would need:

```
┌─────────────────────────────────────────────────────────────────┐
│               OS AUDIO INPUT ARCHITECTURE (PROPOSED)            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌────────────────┐                                            │
│   │  OS Audio API  │  (CoreAudio/WASAPI/ALSA)                   │
│   │  • Mic input   │                                            │
│   │  • Callbacks   │                                            │
│   │  • Buffer mgmt │                                            │
│   └────────┬───────┘                                            │
│            │ Real-time thread                                   │
│            ▼                                                    │
│   ┌────────────────────────┐                                    │
│   │  Audio Processing      │  ← DSP duplication required!       │
│   │  • Load plugin DSP?    │                                    │
│   │  • Reimplement?        │                                    │
│   │  • Meter extraction    │                                    │
│   └────────┬───────────────┘                                    │
│            │ Lock-free queue (rtrb)                             │
│            ▼                                                    │
│   ┌────────────────┐                                            │
│   │  DevServerHost │  ← Now bridging two worlds                 │
│   └────────┬───────┘                                            │
│            │                                                    │
│            ▼                                                    │
│       WebSocket → Browser                                       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Required crates:**
- `cpal` or platform-specific audio I/O (CoreAudio, WASAPI, etc.)
- `rtrb` or similar lock-free queue for audio → parameter thread
- Real-time safe DSP processing or plugin loading infrastructure

**New dependencies:**
```toml
[dependencies]
cpal = "0.15"              # Cross-platform audio I/O
rtrb = "0.2"               # Real-time safe ring buffer
# Platform-specific:
coreaudio-sys = "0.2"      # macOS direct access
windows-wasapi = "..."     # Windows direct access
alsa = "0.7"               # Linux direct access
```

---

## Technical Feasibility

### Can It Be Done?

**Yes, but at significant cost.**

| Requirement | Feasibility | Complexity | Risk |
|------------|-------------|------------|------|
| OS audio input (mic) | ✅ Proven (cpal) | Medium | Platform bugs |
| Real-time audio callbacks | ✅ Standard pattern | Medium | Xruns, latency |
| Meter extraction | ✅ Simple RMS/peak | Low | None |
| Parameter application | ⚠️ Complex | **High** | Real-time safety |
| DSP processing | ⚠️ Duplication needed | **High** | Code fragmentation |
| Cross-platform | ⚠️ Partial (cpal) | **High** | Linux flaky |

### The DSP Duplication Problem

**Critical issue:** How do you process the audio?

#### Option A: Load the Plugin Binary

```rust
// In dev server
let plugin = load_plugin_dylib("path/to/plugin.so")?;
let mut processor = plugin.create_processor();

audio_callback(|input, output| {
    processor.process(input, output);  // Real-time constraints!
    
    // Extract meters
    let meters = calculate_meters(output);
    meter_queue.push(meters);  // Send to WebSocket thread
});
```

**Issues:**
- Requires dynamic loading of plugin code (brittle)
- Plugin may expect host context (VST3 `IPlugFrame`, etc.)
- Real-time safety of user's DSP code becomes dev server's problem
- Separate compilation step before `wavecraft start` works

#### Option B: Duplicate DSP Logic

```rust
// Reimplement processing in dev server
audio_callback(|input, output| {
    // Hardcoded gain/processing
    for (i, o) in input.iter().zip(output.iter_mut()) {
        *o = *i * gain.load(Ordering::Relaxed);
    }
    
    let meters = calculate_meters(output);
    meter_queue.push(meters);
});
```

**Issues:**
- DSP logic lives in two places (plugin + dev server)
- Divergence risk: dev server != production plugin
- Defeats the purpose of testing real DSP behavior

#### Option C: Pure Metering (No Processing)

```rust
// Just pass-through and meter
audio_callback(|input, output| {
    output.copy_from_slice(input);
    
    let meters = calculate_meters(input);
    meter_queue.push(meters);
});
```

**Issues:**
- Can't test parameter changes affecting sound
- Limited value: just metering display
- Still requires full real-time infrastructure for minimal gain

---

## Architecture Impact Assessment

### Violation of Core Principles

The OS audio approach violates multiple architectural principles from [High-Level Design](../../architecture/high-level-design.md):

#### 1. **Clear Separation of Domains**

> "You must enforce strict boundaries between: DSP Core, Plugin Host Layer, UI Layer"

**Violated:** Dev server becomes a mini-plugin-host with real-time DSP concerns.

```
Current (Clean):
  Dev Server = Pure IPC + Parameter State
  
Proposed (Blurred):
  Dev Server = IPC + Parameter State + Audio I/O + DSP + Real-time Safety
```

#### 2. **Real-Time Audio Is Sacred**

> "No allocations, locks, syscalls, logging, or I/O on the audio thread."

**Consequence:** The dev server (a **development tool**) now has real-time safety obligations. Bugs in dev tooling could cause xruns.

This inverts priorities: production code should be real-time safe; dev tooling should be debuggable and flexible.

#### 3. **Parameters Are the Only Contract**

> "All UI → audio communication must go through host-managed parameters, atomics, or lock-free queues."

**Issue:** The dev server would need a full parameter atomics system to apply changes in real-time. This duplicates the plugin's parameter handling.

#### 4. **Rust-Specific Discipline**

> "Minimal `unsafe`, always justified. No `Arc<Mutex<T>>` in real-time paths."

**Problem:** Audio I/O inherently requires platform-specific `unsafe` code. Dev tooling should minimize `unsafe` for debuggability.

---

### Conceptual Model Confusion

The dev server's purpose becomes ambiguous:

| Question | Current Answer | With OS Audio |
|----------|----------------|---------------|
| What is the dev server? | IPC testing harness | Mini audio engine? |
| When do I use it? | UI development | UI + DSP testing? |
| What does it test? | Parameter plumbing | Audio correctness? |
| Where's the plugin? | Loaded for FFI metadata | Also processing audio? |
| Is DSP real? | No (synthetic) | Yes? Maybe? Partial? |

**Result:** Cognitive overhead for developers. Two audio processing paths to maintain.

---

## Proposal 2: Browser-Based WASM Audio Input (Existing Plan)

### Architecture (From Feature Spec)

```
┌─────────────────────────────────────────────────────────────────┐
│              BROWSER AUDIO ARCHITECTURE (WASM)                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────────┐                                           │
│   │  Browser WebAPI │                                           │
│   │  • getUserMedia │  (Microphone)                             │
│   │  • File input   │  (Drag & drop)                            │
│   │  • Oscillator   │  (Test tones)                             │
│   └────────┬────────┘                                           │
│            │                                                    │
│            ▼                                                    │
│   ┌─────────────────────────┐                                   │
│   │  AudioWorkletProcessor  │                                   │
│   │  ┌───────────────────┐  │                                   │
│   │  │ Mock: JS gain     │  │  (Phase 1)                        │
│   │  └───────────────────┘  │                                   │
│   │  ┌───────────────────┐  │                                   │
│   │  │ WASM: Real DSP    │  │  (Phase 2, optional)              │
│   │  │ (plugin.wasm)     │  │                                   │
│   │  └───────────────────┘  │                                   │
│   └─────────┬───────────────┘                                   │
│             │                                                   │
│             │ postMessage (meters)                              │
│             ▼                                                   │
│        React UI                                                 │
│                                                                 │
│   ┌─────────────────────────┐                                   │
│   │  Dev Server (WebSocket) │  ← Unchanged: pure IPC            │
│   │  • Parameter sync       │                                   │
│   │  • State management     │                                   │
│   └─────────────────────────┘                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key insight:** Audio processing stays in the browser. Dev server stays pure IPC.

---

## Comparative Analysis

| Criterion | OS Audio in Dev Server | WASM in Browser | Winner |
|-----------|------------------------|-----------------|--------|
| **Architectural Clarity** | ❌ Blurs boundaries | ✅ Clean separation | **WASM** |
| **Code Duplication** | ❌ DSP in two places | ✅ Single DSP codebase | **WASM** |
| **Real-time Complexity** | ❌ Dev tool + RT safety | ✅ Browser handles RT | **WASM** |
| **Cross-platform** | ⚠️ cpal + platform quirks | ✅ Web standard | **WASM** |
| **Audio Sources** | 🟡 Mic only | ✅ Mic + files + tones | **WASM** |
| **HMR Preservation** | ✅ Yes (UI only) | ✅ Yes (UI only) | **Tie** |
| **DSP Testing** | ⚠️ Requires plugin loading | ✅ Compile to WASM | **WASM** |
| **Dev Server Simplicity** | ❌ 500+ LOC added | ✅ No changes | **WASM** |
| **Debuggability** | ❌ RT constraints limit | ✅ Browser DevTools | **WASM** |
| **Latency** | ✅ Low (native) | 🟡 Acceptable (Web Audio) | OS Audio |
| **Production Parity** | ⚠️ Different code path | ⚠️ Different runtime | **Tie** |

**Score: WASM wins 8/11 criteria.**

---

## Implementation Complexity

### OS Audio Approach

**Estimated LOC:** ~800-1200 lines

```rust
// cli/src/dev_server/audio.rs (~400 LOC)
pub struct AudioEngine {
    stream: cpal::Stream,
    meter_sender: rtrb::Producer<MeterFrame>,
    param_recv: rtrb::Consumer<ParamChange>,
    // Platform-specific state
}

impl AudioEngine {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_input_device()?;
        let config = device.default_input_config()?;
        
        let (meter_tx, meter_rx) = rtrb::RingBuffer::new(64);
        let (param_tx, param_rx) = rtrb::RingBuffer::new(256);
        
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Real-time callback (no allocations!)
                process_audio(data, &param_rx, &meter_tx);
            },
            |err| eprintln!("Audio error: {}", err),
            None,
        )?;
        
        Ok(Self { stream, meter_sender: meter_tx, param_recv: param_rx })
    }
}

// cli/src/dev_server/processor.rs (~300 LOC)
// Duplicate DSP logic or plugin loading

// cli/src/dev_server/host.rs (~200 LOC modifications)
// Wire audio engine into DevServerHost

// Platform-specific fixes (~100 LOC)
// Handle macOS/Windows/Linux quirks
```

**New dependencies:** 3-5 crates  
**Platform testing:** macOS, Windows, Linux (ALSA/PulseAudio/Jack)  
**Maintenance burden:** High (OS audio APIs change)

### WASM Approach

**Estimated LOC:** ~400-600 lines (Phase 1 only)

```typescript
// ui/src/audio/AudioSourceSelector.tsx (~150 LOC)
// UI for mic/file/tone selection

// ui/src/audio/MockDspProcessor.ts (~100 LOC)
// Simple JS gain + metering

// ui/src/audio/AudioContext.ts (~150 LOC)
// Web Audio API plumbing

// Optional Phase 2:
// engine/crates/wavecraft-dsp/src/wasm.rs (~200 LOC)
// WASM bindings for real DSP
```

**New dependencies:** 0 (Web APIs)  
**Platform testing:** Any browser  
**Maintenance burden:** Low (Web Audio is stable)

---

## Recommended Approach

### Primary Recommendation: Browser WASM (Existing Proposal)

**Rationale:**

1. **Preserves architectural integrity**  
   - Dev server remains a pure IPC/parameter testing tool
   - Audio processing stays within plugin/browser boundary
   - Clear mental model: "Browser for UI, dev server for IPC"

2. **Reduces complexity**  
   - No OS audio dependencies in dev tooling
   - No real-time safety concerns in development infrastructure
   - No DSP duplication or plugin loading complexity

3. **Better developer experience**  
   - Multiple audio sources (mic, files, test tones)
   - Browser DevTools for debugging
   - Works in CI/CD (headless Chrome with audio fixtures)

4. **Future-proof**  
   - Web Audio API is stable and improving
   - WASM maturity enables real DSP testing
   - No platform-specific audio driver issues

### Implementation Phases

**Phase 1 (High Value, Low Risk):** Mock DSP in Browser  
- Time: 1-2 weeks  
- Value: Real audio input for UI testing  
- Risk: Low (pure JS, no WASM complexity)

**Phase 2 (Optional):** WASM DSP for Integration Testing  
- Time: 2-3 weeks  
- Value: Test actual DSP algorithms in browser  
- Risk: Medium (WASM build tooling, SharedArrayBuffer requirements)

**Phase 0 (Current):** Synthetic Meters  
- Keep for pure UI layout testing
- Fastest iteration, no audio setup required

---

## If OS Audio Must Be Pursued

If there's a compelling business case I'm missing, here's how to minimize damage:

### Architectural Constraints

1. **Create a separate binary** (`wavecraft-audio-dev`), not part of `wavecraft start`  
   - Reason: Keeps dev server focused
   - Tradeoff: Extra CLI command

2. **Make it opt-in**  
   - Default: synthetic meters (current)
   - Flag: `wavecraft start --real-audio`
   - Reason: Fail-safe for users without working audio setup

3. **Use plugin loading, not DSP duplication**  
   ```rust
   // Load compiled plugin for processing
   let plugin = PluginInstance::load("target/debug/libmyplugin.dylib")?;
   ```
   - Reason: Single source of truth for DSP
   - Tradeoff: Requires compilation before `wavecraft start --real-audio`

4. **Document real-time constraints**  
   - Warn users that xruns in dev mode indicate plugin bugs
   - Provide `--audio-buffer-size` flag for testing
   - Reason: Educate about real-time expectations

5. **Fail gracefully**  
   - If audio setup fails, fall back to synthetic meters
   - Never block `wavecraft start` on audio availability
   - Reason: Dev tooling must always work

### Cost-Benefit Truth

Even with these constraints:
- **Cost:** 800+ LOC, real-time complexity, platform testing
- **Benefit:** Slightly lower latency than WASM (~5-10ms difference)
- **Alternative:** WASM achieves 90% of the same goal with 10% of the cost

---

## Risks & Mitigations

### WASM Approach Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Browser audio permission UX | Medium | Low | Clear prompts, docs |
| SharedArrayBuffer COOP/COEP | High | Medium | Phase 1 works without; document headers |
| WASM-JS bridge overhead | Low | Low | Acceptable for dev mode |
| Parameter definition drift | Medium | Medium | Generate schema from Rust (Phase 3) |

### OS Audio Approach Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Platform audio driver bugs | High | High | Extensive testing per platform |
| Dev server becomes plugin-hosting | High | High | Clear architectural boundaries |
| DSP duplication | High | Medium | Force plugin loading |
| Real-time safety in dev tool | Medium | High | Isolate in separate binary |
| Maintenance burden | High | Medium | Dedicated platform testing |

---

## Decision Criteria

Choose **OS Audio** if:
- [ ] Latency <5ms is critical for development (it's not)
- [ ] Browser audio setup is blocked by policy (e.g., corporate network)
- [ ] Team has real-time audio engineering expertise to maintain it
- [ ] Willing to accept architectural complexity for marginal benefit

Choose **WASM** if:
- [x] Clean architecture is a priority
- [x] Cross-platform consistency matters
- [x] Dev tooling should be simple and reliable
- [x] Multiple audio sources (mic, files, tones) are valuable
- [x] Browser-based "try it now" demos are desirable

**Recommended:** WASM (all criteria met)

---

## Conclusion

The OS audio input proposal is **technically feasible but architecturally wrong**.

It solves a real problem (testing UI with real audio) by introducing complexity in the wrong layer. The existing WASM proposal achieves the same goal while preserving the system's conceptual integrity.

**Architects optimize for maintainability, not features.**

Adding OS audio input to the dev server would create a maintenance burden that outweighs its marginal benefits over the WASM approach. The 5-10ms latency difference is imperceptible in development mode, and the browser-based approach offers superior flexibility (multiple audio sources, cross-platform consistency, debugging tools).

### Final Recommendation

**Proceed with browser-based WASM audio input as documented in [audio-input-via-wasm/high-level-design.md](../audio-input-via-wasm/high-level-design.md).**

If real-time DSP testing is needed beyond what WASM provides, the correct approach is:
1. Load the plugin in a real DAW (production environment)
2. Use the existing `cargo xtask bundle` + install workflow

Development tooling should make common tasks easy, not replicate production environments.

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — System architecture principles
- [Audio Input via WASM](../audio-input-via-wasm/high-level-design.md) — Approved approach
- [Coding Standards](../../architecture/coding-standards.md) — Real-time safety guidelines
