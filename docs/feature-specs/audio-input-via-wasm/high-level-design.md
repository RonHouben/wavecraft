# Audio Input via WASM — High-Level Design

**Status:** Draft  
**Created:** 2026-02-04  
**Author:** Architect Agent

---

## Problem Statement

When developing Wavecraft plugin UIs, developers need to:
1. Test UI components with real audio input (meters, waveforms, visualizations)
2. Maintain fast Hot Module Reloading (HMR) for UI iteration
3. Optionally test actual DSP algorithms in the browser

Currently, the only way to test with real audio is through the dev server (`cargo xtask dev`), which:
- Requires Rust compilation for DSP changes
- Doesn't support browser audio input (microphone, audio files)
- Tightly couples UI development to the Rust toolchain

---

## Goals

| Goal | Priority |
|------|----------|
| Browser audio input (mic, files, test tones) for UI development | High |
| Preserve full HMR for React/CSS changes | High |
| Same UI code works against all backends (Mock, WASM, WebSocket) | High |
| Rust remains source of truth for parameter definitions | High |
| Optional WASM DSP for integration testing | Medium |
| "Try in browser" demo capability for end users | Low |

---

## Non-Goals

- Real-time remote audio processing via WebSocket (latency makes this impractical)
- Replacing the dev server for IPC testing
- TypeScript-defined parameter schemas

---

## Architecture Overview

### Tiered Audio Source System

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    TIERED DSP BACKEND ARCHITECTURE                      │
└─────────────────────────────────────────────────────────────────────────┘

                         ┌─────────────────────┐
                         │   Audio Source      │
                         │   Selector (UI)     │
                         └──────────┬──────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              ▼                     ▼                     ▼
     ┌────────────────┐   ┌────────────────┐   ┌────────────────┐
     │  Test Tone     │   │  Browser Mic   │   │  Audio File    │
     │  (Oscillator)  │   │  (getUserMedia)│   │  (Drag & Drop) │
     └────────┬───────┘   └────────┬───────┘   └────────┬───────┘
              │                    │                    │
              └─────────────────►──┴──◄─────────────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    ▼                           ▼
           ┌──────────────┐            ┌──────────────┐
           │  Mock DSP    │            │  WASM DSP    │
           │  (JS-only)   │            │  (Rust→WASM) │
           └──────┬───────┘            └──────┬───────┘
                  │                           │
                  └─────────────►◄────────────┘
                                │
                         ┌──────┴───────┐
                         │  React UI    │
                         │  (Meters,    │
                         │   Sliders)   │
                         └──────────────┘
```

### Three Development Modes

| Mode | Audio Source | DSP Backend | Use Case | HMR? |
|------|--------------|-------------|----------|------|
| **Mock** (default) | Browser (mic/file/tone) | JS AudioWorklet | UI development | ✅ Full |
| **WASM** | Browser (mic/file/tone) | Rust→WASM | DSP integration testing | ✅ UI only |
| **Dev Server** | Rust audio engine | Rust plugin via WebSocket | Full IPC testing | ✅ UI only |

---

## Parameter Ownership Model

**Critical constraint:** Rust is the source of truth for parameter definitions.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    PARAMETER OWNERSHIP FLOW                             │
└─────────────────────────────────────────────────────────────────────────┘

     RUST (Source of Truth)                 TYPESCRIPT (View Layer)
     ──────────────────────                 ───────────────────────
     
     #[derive(ProcessorParams)]             
     struct GainParams {                    useParameter("gain")
         #[param(range = "-60..=24")]             │
         gain: f32,                               │ queries
     }                                            ▼
            │                               ┌─────────────┐
            │ defines schema                │ DspBackend  │
            ▼                               │ interface   │
     ┌──────────────┐                       └──────┬──────┘
     │ WASM exports │ ◄─────────────────────────── │
     │ or WebSocket │   getParameterInfos()        │
     │ or Mock JSON │   getParameter(id)           │
     └──────────────┘   setParameter(id, val)      │
            │                                      │
            │ runtime values                       │
            └──────────────────────────────────────┘
```

TypeScript **never defines** parameters. It:
1. **Queries** parameter definitions from the backend
2. **Displays** current values
3. **Forwards** user input to the backend
4. **Subscribes** to value changes

---

## Backend Abstraction

A unified interface allows swapping backends without changing UI code:

```typescript
interface DspBackend {
  // Discovery (called once on connect)
  getParameterInfos(): Promise<ParameterInfo[]>;
  
  // Read current value
  getParameter(id: string): Promise<number>;
  
  // Write value (user interaction)
  setParameter(id: string, value: number): Promise<void>;
  
  // Subscribe to changes (automation, presets, external updates)
  onParameterChange(cb: (id: string, value: number) => void): () => void;
  
  // Audio metering (high-frequency updates)
  onMeterUpdate(cb: (frame: MeterFrame) => void): () => void;
}
```

### Backend Implementations

| Backend | Transport | Parameter Source | When Used |
|---------|-----------|------------------|-----------|
| `MockBackend` | Direct JS calls | JSON file or hardcoded | `npm run dev` (default) |
| `WasmBackend` | Direct WASM calls | WASM module exports | `VITE_DSP_MODE=wasm npm run dev` |
| `WebSocketBackend` | WebSocket IPC | Rust plugin via bridge | `cargo xtask dev` |

---

## Audio Pipeline (Browser Mode)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    WEB AUDIO PIPELINE                                   │
└─────────────────────────────────────────────────────────────────────────┘

  ┌──────────────┐
  │ AudioContext │
  └──────┬───────┘
         │
         ▼
  ┌──────────────┐     getUserMedia() or
  │ Source Node  │ ◄── createOscillator() or
  │              │     createMediaElementSource()
  └──────┬───────┘
         │
         ▼
  ┌──────────────────────────────────────────┐
  │            AudioWorkletNode              │
  │  ┌────────────────────────────────────┐  │
  │  │  AudioWorkletProcessor             │  │
  │  │  ├── Mock: JS gain/analysis        │  │
  │  │  └── WASM: wasmModule.process()    │  │
  │  └────────────────────────────────────┘  │
  │              │                           │
  │              │ postMessage (meters)      │
  │              ▼                           │
  └──────────────┬───────────────────────────┘
                 │
                 ▼
          ┌──────────────┐
          │ Destination  │ (speakers)
          └──────────────┘
```

### Mock DSP Processor (JavaScript)

For UI development, a simple JS processor provides realistic meter data:

```javascript
class MockDspProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.gain = 1.0;
    this.port.onmessage = (e) => {
      if (e.data.type === 'setParameter') {
        this[e.data.id] = e.data.value;
      }
    };
  }

  process(inputs, outputs) {
    const input = inputs[0]?.[0];
    const output = outputs[0]?.[0];
    
    if (!input || !output) return true;

    // Simple gain processing
    for (let i = 0; i < input.length; i++) {
      output[i] = input[i] * this.gain;
    }

    // Compute RMS for meters
    let sum = 0;
    for (let i = 0; i < output.length; i++) {
      sum += output[i] * output[i];
    }
    const rms = Math.sqrt(sum / output.length);

    // Send meter data to main thread (~60 Hz)
    if (currentFrame % 735 === 0) { // ~60 Hz at 44.1kHz
      this.port.postMessage({ type: 'meter', rms, peak: Math.max(...output.map(Math.abs)) });
    }

    return true;
  }
}
```

### WASM DSP Processor

For integration testing, the actual Rust DSP runs in the worklet:

```javascript
class WasmDspProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.wasmReady = false;
    this.port.onmessage = async (e) => {
      if (e.data.type === 'init') {
        this.wasm = await WebAssembly.instantiate(e.data.module);
        this.wasmReady = true;
      } else if (e.data.type === 'setParameter') {
        this.wasm.exports.set_parameter(e.data.id, e.data.value);
      }
    };
  }

  process(inputs, outputs) {
    if (!this.wasmReady) return true;
    
    // Copy input to WASM memory, process, copy output back
    // (SharedArrayBuffer optimization possible)
    this.wasm.exports.process(inputPtr, outputPtr, inputs[0][0].length);
    
    return true;
  }
}
```

---

## Audio Source Selector Component

A dev-mode UI component for selecting audio input:

```tsx
type AudioSource = 
  | { type: 'none' }
  | { type: 'oscillator'; frequency: number; waveform: OscillatorType }
  | { type: 'microphone'; deviceId: string }
  | { type: 'file'; url: string };

interface AudioSourceSelectorProps {
  onSourceChange: (source: AudioSource) => void;
  currentSource: AudioSource;
}

function AudioSourceSelector({ onSourceChange, currentSource }: AudioSourceSelectorProps) {
  const [devices, setDevices] = useState<MediaDeviceInfo[]>([]);
  
  useEffect(() => {
    navigator.mediaDevices.enumerateDevices()
      .then(d => setDevices(d.filter(d => d.kind === 'audioinput')));
  }, []);

  return (
    <div className="flex items-center gap-2 p-2 bg-plugin-surface rounded border border-plugin-border">
      <span className="text-xs text-gray-400">Audio:</span>
      
      <button 
        onClick={() => onSourceChange({ type: 'none' })}
        className={currentSource.type === 'none' ? 'text-accent' : 'text-gray-400'}
      >
        🔇 None
      </button>
      
      <button 
        onClick={() => onSourceChange({ type: 'oscillator', frequency: 440, waveform: 'sine' })}
        className={currentSource.type === 'oscillator' ? 'text-accent' : 'text-gray-400'}
      >
        🎵 440 Hz
      </button>
      
      <select 
        onChange={(e) => onSourceChange({ type: 'microphone', deviceId: e.target.value })}
        className="bg-plugin-dark text-sm"
      >
        <option value="">🎤 Microphone...</option>
        {devices.map(d => (
          <option key={d.deviceId} value={d.deviceId}>
            {d.label || `Mic ${d.deviceId.slice(0, 8)}`}
          </option>
        ))}
      </select>
      
      <input 
        type="file" 
        accept="audio/*"
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (file) onSourceChange({ type: 'file', url: URL.createObjectURL(file) });
        }}
        className="text-sm"
      />
    </div>
  );
}
```

---

## Implementation Phases

### Phase 1: Mock DSP + Audio Source Selector
**Effort:** Medium  
**Value:** High (enables UI dev with real audio, preserves HMR)

1. Create `MockBackend` implementing `DspBackend` interface
2. Create `MockDspProcessor` AudioWorklet (JS-only, simple gain + metering)
3. Create `AudioSourceSelector` component
4. Wire up audio routing (source → worklet → speakers)
5. Mock parameter definitions loaded from JSON (mirrors Rust schema)

### Phase 2: WASM DSP Integration
**Effort:** High  
**Value:** Medium (for DSP integration testing)

1. Add `wasm32-unknown-unknown` target to `wavecraft-dsp`
2. Create `wasm-bindgen` exports for parameter access and processing
3. Create `WasmBackend` implementing `DspBackend` interface
4. Create `WasmDspProcessor` AudioWorklet wrapper
5. Handle SharedArrayBuffer/COOP/COEP requirements

### Phase 3: Parameter Schema Sync
**Effort:** Medium  
**Value:** High (ensures Mock matches Rust)

1. Generate JSON schema from `ProcessorParams` derive macro
2. Export schema as part of WASM build
3. Mock mode loads same schema (single source of truth)

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| WASM Audio Worklet browser support gaps | Medium | High | Phase 1 uses JS mock; WASM is optional |
| SharedArrayBuffer COOP/COEP complexity | Medium | Medium | Document header requirements; fallback to postMessage |
| Parameter schema drift (Mock vs Rust) | Medium | Medium | Phase 3 generates schema from Rust |
| Audio latency in browser | Low | Low | Acceptable for dev mode; not for production use |

---

## Open Questions

1. **Schema generation:** Should the `ProcessorParams` derive macro generate a JSON schema file, or should WASM export it at runtime?

2. **Dev mode UI:** Should the audio source selector be part of `@wavecraft/ui-components` or a separate dev-only package?

3. **WASM build integration:** Should `cargo xtask dev` automatically build WASM when `VITE_DSP_MODE=wasm`?

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Overall architecture
- [Coding Standards](../../architecture/coding-standards.md) — Implementation conventions
- [Backlog](../../backlog.md) — Prioritization tracking
