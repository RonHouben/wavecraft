# Low-Level Design — Dev Audio FFI Abstraction

## Summary

Move the `dev-audio.rs` binary out of the user's project template entirely. The CLI runs audio capture + processing in-process, loading the user's DSP code from their compiled cdylib via a C-ABI FFI contract. The `wavecraft_plugin!` macro auto-generates the FFI exports. Users never see, write, or maintain any audio development binary code.

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md)
- [Coding Standards](../../architecture/coding-standards.md)
- [Audio Input via WASM (archived)](../../../docs/feature-specs/_archive/dev-audio-os-input/)

---

## Problem Statement

The current SDK template includes:

1. A 40-line `engine/src/bin/dev-audio.rs` binary — pure SDK boilerplate
2. Six optional dependencies in `Cargo.toml` (`wavecraft-dsp`, `wavecraft-dev-server`, `cpal`, `anyhow`, `env_logger`, `tokio`)
3. A `[features] audio-dev` section and `[[bin]]` section

The **only** user-specific part is one line: `let processor = GainDsp::default()`. Everything else is SDK infrastructure that leaks into user space.

This creates:
- Cognitive overhead for new SDK users seeing unfamiliar audio internals
- Template maintenance burden when audio server APIs change
- A `Cargo.toml` bloated with optional deps the user doesn't understand

---

## Design Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      CURRENT (leaky)                                    │
│                                                                         │
│  User's Project/                                                        │
│    engine/src/bin/dev-audio.rs    ← 40 lines of SDK boilerplate         │
│    engine/Cargo.toml              ← 6 optional deps, feature, [[bin]]   │
│                                                                         │
│  CLI (`wavecraft start`)                                                │
│    → spawns separate `cargo run --bin dev-audio` process                │
│    → checks has_audio_binary() heuristic                                │
│    → prints "add this to your Cargo.toml" instructions on missing       │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                      PROPOSED (clean)                                   │
│                                                                         │
│  User's Project/                                                        │
│    engine/src/lib.rs              ← unchanged, no audio binary          │
│    engine/Cargo.toml              ← no audio-dev deps or feature flag   │
│                                                                         │
│  CLI (`wavecraft start`)                                                │
│    1. cargo build --lib (user's cdylib)                                 │
│    2. dlopen(cdylib) — already done for params                          │
│    3. look up wavecraft_dev_create_processor FFI symbol                 │
│    4. run audio capture + processing in-process                         │
│    5. send meter data to WebSocket server (existing)                    │
└─────────────────────────────────────────────────────────────────────────┘
```

### Key Architectural Decision: In-Process Audio

The audio server runs **inside the CLI process** rather than as a separate spawned binary.

**Rationale:**
- The CLI already loads the user's cdylib (for parameter discovery via `PluginParamLoader`)
- The CLI already runs a tokio runtime for the WebSocket server
- cpal runs its audio callback on a dedicated OS audio thread regardless of host process
- Eliminates process management complexity (spawn, monitor, kill, signal forwarding)
- One fewer process to debug, log, and coordinate

**Trade-off:** The CLI binary gains cpal as a dependency (~1–2 MB binary size increase). This is acceptable for an audio development tool. An optional feature flag keeps it out for minimal installs.

---

## FFI Contract

### Location: `wavecraft-protocol` crate

The FFI contract lives in `wavecraft-protocol` because it defines a shared interface between the macro-generated code (in the user's cdylib) and the consumer (the CLI). This follows the existing pattern: `wavecraft-protocol` already defines `ParameterInfo`, `MeterFrame`, and other shared contracts.

### Definition

```rust
// engine/crates/wavecraft-protocol/src/dev_audio_ffi.rs

use std::ffi::c_void;

/// C-ABI stable vtable for dev-mode audio processing.
///
/// This struct is returned by the `wavecraft_dev_create_processor` FFI symbol
/// exported from user plugins. It provides function pointers for creating,
/// using, and destroying a processor instance across the dylib boundary.
///
/// # ABI Stability
///
/// This struct uses `#[repr(C)]` and only `extern "C"` function pointers,
/// making it safe across separately compiled Rust binaries. All data passes
/// through primitive types (f32, u32, *mut c_void, *mut *mut f32).
///
/// # Versioning
///
/// A `version` field allows the CLI to detect incompatible vtable changes
/// and provide clear upgrade guidance instead of undefined behavior.
#[repr(C)]
pub struct DevProcessorVTable {
    /// VTable version. Must equal `DEV_PROCESSOR_VTABLE_VERSION`.
    pub version: u32,

    /// Create a new processor instance.
    ///
    /// Returns an opaque pointer to a heap-allocated processor.
    /// The caller must eventually pass this pointer to `drop` to free it.
    pub create: extern "C" fn() -> *mut c_void,

    /// Process audio in deinterleaved (per-channel) format.
    ///
    /// # Arguments
    /// - `instance`: Opaque processor pointer from `create`
    /// - `channels`: Pointer to an array of `num_channels` mutable f32 pointers
    /// - `num_channels`: Number of audio channels (typically 2)
    /// - `num_samples`: Number of samples per channel
    ///
    /// # Safety
    /// - `instance` must be a valid pointer from `create`
    /// - `channels[0..num_channels]` must each point to `num_samples` valid f32s
    /// - Must be called from a single thread (not thread-safe)
    pub process: extern "C" fn(
        instance: *mut c_void,
        channels: *mut *mut f32,
        num_channels: u32,
        num_samples: u32,
    ),

    /// Update the processor's sample rate.
    pub set_sample_rate: extern "C" fn(instance: *mut c_void, sample_rate: f32),

    /// Reset processor state (clear delay lines, filters, etc.).
    pub reset: extern "C" fn(instance: *mut c_void),

    /// Destroy the processor instance and free its memory.
    ///
    /// # Safety
    /// - `instance` must be a valid pointer from `create`
    /// - Must not be called more than once for the same pointer
    /// - No other vtable function may be called after `drop`
    pub drop: extern "C" fn(instance: *mut c_void),
}

/// Current vtable version. Increment on breaking changes.
pub const DEV_PROCESSOR_VTABLE_VERSION: u32 = 1;

/// FFI symbol name exported by wavecraft_plugin! macro.
pub const DEV_PROCESSOR_SYMBOL: &[u8] = b"wavecraft_dev_create_processor\0";
```

### Why This Shape

| Decision | Rationale |
|---|---|
| `*mut c_void` instance | Type erasure across dylib boundary. Concrete type only known inside the dylib. |
| Per-channel `*mut *mut f32` | Matches `Processor::process(&mut [&mut [f32]])` layout. Avoids interleave/deinterleave overhead. |
| No `Transport` parameter | Dev mode doesn't have DAW transport. Processor receives a default `Transport` internally. |
| No `Params` parameter | Same limitation as current macro: DSL-generated code uses default params. The vtable processes audio with current default values. Future parameter sync enhancement happens inside the dylib, not across FFI. |
| `version` field | Allows CLI to detect vtable mismatches after SDK upgrades and fail with a clear message instead of UB. |
| No `Result`/`Option` returns | C-ABI primitives only. Errors logged inside the dylib via `tracing`. |

---

## Macro Code Generation

The `wavecraft_plugin!` proc-macro adds one new FFI export alongside the existing `wavecraft_get_params_json` and `wavecraft_free_string`:

```rust
// Generated by wavecraft_plugin! macro (in wavecraft-macros/src/plugin.rs)

#[unsafe(no_mangle)]
pub extern "C" fn wavecraft_dev_create_processor()
    -> wavecraft_protocol::DevProcessorVTable
{
    use std::ffi::c_void;
    use wavecraft_protocol::{DevProcessorVTable, DEV_PROCESSOR_VTABLE_VERSION};

    // The concrete processor type from the macro's `signal:` field
    type P = __ProcessorType;
    type Params = <P as wavecraft_dsp::Processor>::Params;

    extern "C" fn create() -> *mut c_void {
        let processor = Box::new(P::default());
        Box::into_raw(processor) as *mut c_void
    }

    extern "C" fn process(
        instance: *mut c_void,
        channels: *mut *mut f32,
        num_channels: u32,
        num_samples: u32,
    ) {
        let processor = unsafe { &mut *(instance as *mut P) };
        let num_ch = num_channels as usize;
        let num_samp = num_samples as usize;

        // Build &mut [&mut [f32]] from raw pointers
        // SAFETY: Caller guarantees valid pointers and bounds (documented in vtable)
        let mut channel_slices: Vec<&mut [f32]> = (0..num_ch)
            .map(|ch| unsafe {
                let ptr = *channels.add(ch);
                std::slice::from_raw_parts_mut(ptr, num_samp)
            })
            .collect();

        let transport = wavecraft_dsp::Transport::default();
        let params = Params::default();

        processor.process(&mut channel_slices, &transport, &params);
    }

    extern "C" fn set_sample_rate(instance: *mut c_void, sample_rate: f32) {
        let processor = unsafe { &mut *(instance as *mut P) };
        wavecraft_dsp::Processor::set_sample_rate(processor, sample_rate);
    }

    extern "C" fn reset(instance: *mut c_void) {
        let processor = unsafe { &mut *(instance as *mut P) };
        wavecraft_dsp::Processor::reset(processor);
    }

    extern "C" fn drop_fn(instance: *mut c_void) {
        if !instance.is_null() {
            let _ = unsafe { Box::from_raw(instance as *mut P) };
        }
    }

    DevProcessorVTable {
        version: DEV_PROCESSOR_VTABLE_VERSION,
        create,
        process,
        set_sample_rate,
        reset,
        drop: drop_fn,
    }
}
```

### Note on `Vec` Allocation in `process()`

The `channel_slices` `Vec` allocates on the heap. In production plugin code, this would violate real-time safety. However, this code runs in the **dev audio server** context (not in a DAW audio thread), where the audio callback latency requirements are much softer. The allocation per callback is acceptable.

If performance becomes a concern, this can be optimized to use a stack-allocated `SmallVec` or a fixed-size array (stereo is by far the common case).

---

## CLI-Side Plugin Loader Extension

### Extend `PluginParamLoader` → `PluginLoader`

The existing `PluginParamLoader` in `wavecraft-bridge` already handles dlopen and parameter symbol loading. Extend it to optionally load the audio processor vtable:

```rust
// engine/crates/wavecraft-bridge/src/plugin_loader.rs

type DevProcessorVTableFn = unsafe extern "C" fn() -> DevProcessorVTable;

pub struct PluginLoader {
    _library: Library,
    parameters: Vec<ParameterInfo>,
    /// Audio processor vtable (None if symbol not found — older plugins)
    dev_processor_vtable: Option<DevProcessorVTable>,
}

impl PluginLoader {
    pub fn load<P: AsRef<Path>>(dylib_path: P) -> Result<Self, PluginLoaderError> {
        let library = unsafe { Library::new(dylib_path.as_ref()) }
            .map_err(PluginLoaderError::LibraryLoad)?;

        // Load parameters (existing logic, unchanged)
        let parameters = Self::load_parameters(&library)?;

        // Try to load audio processor vtable (optional — graceful fallback)
        let dev_processor_vtable = Self::try_load_processor_vtable(&library);

        Ok(Self {
            _library: library,
            parameters,
            dev_processor_vtable,
        })
    }

    /// Returns the dev processor vtable if the plugin exports it.
    pub fn dev_processor_vtable(&self) -> Option<&DevProcessorVTable> {
        self.dev_processor_vtable.as_ref()
    }

    fn try_load_processor_vtable(library: &Library) -> Option<DevProcessorVTable> {
        let symbol: Symbol<DevProcessorVTableFn> = unsafe {
            library.get(b"wavecraft_dev_create_processor\0").ok()?
        };

        let vtable = unsafe { symbol() };

        // Version check
        if vtable.version != DEV_PROCESSOR_VTABLE_VERSION {
            tracing::warn!(
                "Plugin dev-audio vtable version {} != expected {}. \
                 Audio processing disabled. Update your SDK dependency.",
                vtable.version,
                DEV_PROCESSOR_VTABLE_VERSION
            );
            return None;
        }

        Some(vtable)
    }
}
```

### Backward Compatibility

The vtable symbol is loaded via `try_load_processor_vtable` — if the symbol isn't found (user hasn't updated their SDK dependency), the CLI gracefully falls back to metering-only mode (no DSP processing). This makes the transition non-breaking.

---

## Audio Server Refactoring

### Current State

`AudioServer<P: Processor>` in `wavecraft-dev-server` is generic over a `Processor` type. This generic approach can't work with FFI-loaded processors because the concrete type is erased.

### New Design

Replace the generic `AudioServer<P: Processor>` with a callback-based design that works with both direct Rust processors (for testing) and FFI-loaded processors:

```rust
// engine/crates/wavecraft-dev-server/src/audio_server.rs

/// Trait for audio processors in dev mode.
///
/// Simplified interface without associated types — compatible
/// with both direct Rust usage and FFI-loaded processors.
pub trait DevAudioProcessor: Send + 'static {
    /// Process deinterleaved audio in-place.
    fn process(&mut self, channels: &mut [&mut [f32]]);

    /// Update sample rate.
    fn set_sample_rate(&mut self, sample_rate: f32);

    /// Reset state.
    fn reset(&mut self);
}

/// Wrapper that adapts a DevProcessorVTable to DevAudioProcessor.
///
/// Owns the opaque processor instance and calls through the vtable.
pub struct FfiProcessor {
    instance: *mut c_void,
    vtable: DevProcessorVTable,
}

// SAFETY: The processor instance is only accessed from the audio thread.
// The vtable functions are thread-safe by contract (single-threaded access).
unsafe impl Send for FfiProcessor {}

impl FfiProcessor {
    /// Create from a loaded vtable.
    pub fn new(vtable: &DevProcessorVTable) -> Self {
        let instance = (vtable.create)();
        Self {
            instance,
            vtable: *vtable,
        }
    }
}

impl DevAudioProcessor for FfiProcessor {
    fn process(&mut self, channels: &mut [&mut [f32]]) {
        let num_channels = channels.len() as u32;
        if num_channels == 0 || channels[0].is_empty() {
            return;
        }
        let num_samples = channels[0].len() as u32;

        // Build array of channel pointers
        let mut ptrs: Vec<*mut f32> = channels.iter_mut()
            .map(|ch| ch.as_mut_ptr())
            .collect();

        (self.vtable.process)(
            self.instance,
            ptrs.as_mut_ptr(),
            num_channels,
            num_samples,
        );
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        (self.vtable.set_sample_rate)(self.instance, sample_rate);
    }

    fn reset(&mut self) {
        (self.vtable.reset)(self.instance);
    }
}

impl Drop for FfiProcessor {
    fn drop(&mut self) {
        if !self.instance.is_null() {
            (self.vtable.drop)(self.instance);
            self.instance = std::ptr::null_mut();
        }
    }
}
```

`AudioServer` becomes `AudioServer<P: DevAudioProcessor>` — still generic, but over the simplified trait.

---

## CLI `start.rs` Changes

### Before (current)

```
wavecraft start:
  1. Build user's cdylib
  2. Load params via FFI (PluginParamLoader)
  3. has_audio_binary()? → spawn separate cargo run --bin dev-audio process
  4. Start WebSocket server
  5. Start Vite UI server
  6. Monitor all 3 processes for shutdown
```

### After (proposed)

```
wavecraft start:
  1. Build user's cdylib (unchanged)
  2. Load params + processor vtable via FFI (extended PluginLoader)
  3. If vtable available:
       → create FfiProcessor
       → start AudioServer in-process on background tokio task
     Else:
       → log "Audio processing not available (update SDK)"
       → continue with metering-only mode (existing)
  4. Start WebSocket server (unchanged)
  5. Start Vite UI server (unchanged)
  6. Monitor 2 processes (UI server + CLI itself) — no audio child
```

### Code Removed from CLI

| Item | Status |
|---|---|
| `try_start_audio_server()` function (~60 lines) | **Removed** — no separate process |
| `has_audio_binary()` function (~10 lines) | **Removed** — no template detection |
| Audio child process monitoring in `wait_for_shutdown()` | **Simplified** — no `Option<Child>` for audio |
| "Add [[bin]] to your Cargo.toml" instructions | **Removed** — always available |

---

## Template Cleanup

### Files Removed

| File | Reason |
|---|---|
| `cli/sdk-templates/new-project/react/engine/src/bin/dev-audio.rs` | SDK owns audio binary now |
| `cli/sdk-templates/new-project/react/engine/src/bin/` (entire dir) | No bin targets in template |

### Cargo.toml Template Changes

Removed from `engine/Cargo.toml.template`:

```toml
# REMOVE: These all move into the CLI crate
wavecraft-dsp = { ... optional = true }
wavecraft-dev-server = { ... optional = true }
cpal = { ... optional = true }
anyhow = { ... optional = true }
env_logger = { ... optional = true }
tokio = { ... optional = true }

[features]
audio-dev = [...]   # REMOVE

[[bin]]
name = "dev-audio"   # REMOVE
path = "src/bin/dev-audio.rs"
required-features = ["audio-dev"]
```

**Net effect on user's Cargo.toml:** 15 fewer lines, 0 optional dependencies, no feature flags, no binary targets.

---

## CLI Cargo.toml Changes

The CLI crate gains audio dependencies behind an optional feature:

```toml
# cli/Cargo.toml

[dependencies]
# Existing:
wavecraft-dev-server = { path = "../engine/crates/wavecraft-dev-server", version = "0.9.0" }
wavecraft-bridge = { path = "../engine/crates/wavecraft-bridge", version = "0.9.0" }

# New (for audio capture):
cpal = { version = "0.15", optional = true }

[features]
default = ["audio-dev"]
audio-dev = ["cpal", "wavecraft-dev-server/audio"]
```

The `audio-dev` feature is enabled by default so `cargo install wavecraft` includes audio support. Users who want a minimal CLI can install with `--no-default-features`.

---

## Data Flow: Audio Capture with FFI Processor

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  CLI Process (`wavecraft start`)                                             │
│                                                                              │
│  ┌─────────────────────┐                                                     │
│  │  dlopen(cdylib)      │                                                     │
│  │  → params (existing) │                                                     │
│  │  → vtable (new)      │                                                     │
│  └──────────┬──────────┘                                                     │
│             │                                                                │
│             ▼                                                                │
│  ┌─────────────────────┐      ┌──────────────────────────┐                   │
│  │  FfiProcessor        │      │  cpal audio input stream  │                   │
│  │  (wraps vtable)      │◄─────│  (OS microphone)          │                   │
│  └──────────┬──────────┘      └──────────────────────────┘                   │
│             │                                                                │
│             │  process(channels) → audio data modified in-place              │
│             │  compute meters from processed output                          │
│             │                                                                │
│             ▼                                                                │
│  ┌─────────────────────┐      ┌──────────────────┐    ┌──────────────────┐   │
│  │  Meter computation   │─────►│  WebSocket server │───►│  Browser UI      │   │
│  └─────────────────────┘      │  (ws://9000)      │    │  (localhost:5173) │   │
│                               └──────────────────┘    └──────────────────┘   │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Lifecycle and Safety

### Memory Ownership

```
create()  → Box::into_raw(Box::new(Processor))     [dylib allocates]
process() → &mut *(ptr as *mut Processor)           [dylib borrows]
drop()    → Box::from_raw(ptr as *mut Processor)    [dylib deallocates]
```

All allocations and deallocations happen **inside the dylib**, using the dylib's allocator. No cross-allocator free — this is a hard requirement. The CLI never allocates or frees the processor memory; it only passes the opaque pointer back into vtable functions.

### Thread Safety Contract

- `create()`: Called once from the CLI's main thread
- `set_sample_rate()`: Called once from the CLI's main thread before audio starts
- `process()`: Called repeatedly from cpal's audio callback thread — **single-threaded access only**
- `reset()`: Called from CLI's main thread (only when audio is stopped)
- `drop()`: Called once from CLI's main thread during shutdown

The `FfiProcessor` is `Send` (transferable to the audio thread) but NOT `Sync` (no concurrent access). cpal's callback model guarantees single-threaded access in the audio callback.

### Library Lifetime

The `Library` (from `libloading`) must outlive the `FfiProcessor`. The current `PluginLoader` struct holds the `Library` by value, ensuring it's dropped last. This ordering must be preserved:

```
PluginLoader {
    _library: Library,        // dropped last (LIFO)
    dev_processor_vtable,     // metadata only, no pointers into library
}
```

The `FfiProcessor` holds a copy of the vtable (function pointers), not a reference to the library. The function pointers remain valid as long as the library is loaded. The CLI must ensure:
1. `FfiProcessor` is dropped before `PluginLoader`
2. The audio stream is stopped before `FfiProcessor` is dropped

---

## Error Handling

| Scenario | Behavior |
|---|---|
| Symbol `wavecraft_dev_create_processor` not found | Log info, continue with metering-only mode (synthetic meters) |
| VTable version mismatch | Log warning with upgrade instructions, continue without audio processing |
| `create()` returns null | Log error, continue with metering-only mode |
| `process()` panics inside dylib | Panic is confined to the audio thread; cpal reports stream error; CLI logs and continues |
| cpal: No input device available | Log warning, continue without audio (UI still works) |

All fallbacks are **graceful**: the UI development experience continues to work, just without real audio input.

---

## Phased Implementation

### Phase 1: Core FFI (this feature)

1. Add `DevProcessorVTable` + constants to `wavecraft-protocol`
2. Generate `wavecraft_dev_create_processor` export in `wavecraft_plugin!` macro
3. Extend `PluginParamLoader` to load the vtable
4. Add `FfiProcessor` wrapper to `wavecraft-dev-server`
5. Refactor `AudioServer` to use `DevAudioProcessor` trait
6. Update CLI `start.rs` to run audio in-process via FFI
7. Remove `dev-audio.rs`, optional deps, feature flag, `[[bin]]` from template

### Phase 2: Parameter Forwarding (future, separate feature)

Once full bidirectional parameter sync is implemented in the macro (tracked in roadmap), extend the vtable with:

```rust
pub struct DevProcessorVTable {
    // ... existing fields ...

    /// Set a parameter value by ID.
    /// id: null-terminated UTF-8 string
    /// value: normalized float
    pub set_parameter: extern "C" fn(
        instance: *mut c_void,
        id: *const c_char,
        value: f32,
    ),
}
```

This allows the UI → WebSocket → CLI → FFI → Processor parameter flow for real-time parameter editing during development.

### Phase 3: Audio Output (future, separate feature)

Add audio output support so the processed audio is played back through speakers/headphones, enabling a full monitor-your-plugin workflow.

---

## Crate Dependency Changes

### Before

```
cli/
  └── wavecraft-dev-server (no audio feature)
  └── wavecraft-bridge (PluginParamLoader)
  └── wavecraft-protocol

user's plugin/
  └── wavecraft-nih_plug
  └── wavecraft-dsp (optional, for dev-audio binary)
  └── wavecraft-dev-server (optional, for dev-audio binary)
  └── cpal (optional)
  └── tokio (optional)
  └── anyhow (optional)
  └── env_logger (optional)
```

### After

```
cli/
  └── wavecraft-dev-server (with audio feature)
  └── wavecraft-bridge (extended PluginLoader)
  └── wavecraft-protocol (DevProcessorVTable)
  └── cpal (optional, default-on)

user's plugin/
  └── wavecraft-nih_plug (generates vtable FFI export)
  └── (no audio-related deps at all)
```

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| **ABI breakage between SDK versions** | CLI crashes or UB when loading old plugin | `version` field in vtable; CLI checks before use |
| **Allocator mismatch** | Memory corruption if CLI frees dylib memory | All alloc/dealloc happens inside dylib; CLI never frees |
| **Panic across FFI** | UB per Rust spec if panic unwinds through `extern "C"` | Generated vtable functions use `catch_unwind` at the boundary |
| **cpal adds binary size to CLI** | ~1–2 MB increase | Behind `audio-dev` feature flag; opt-out via `--no-default-features` |
| **macOS: dylib code signing** | Debug dylib may fail to load if signing is strict | Development builds use ad-hoc signing; `wavecraft start` builds with `--lib` (debug), not hardened runtime |
| **Library unload order** | Use-after-free if library dropped before processor | `PluginLoader` struct enforces drop order; documented invariant |

### `catch_unwind` at FFI Boundary

Every generated vtable function must wrap the Rust body in `std::panic::catch_unwind` to prevent unwinding through `extern "C"`:

```rust
extern "C" fn process(instance: *mut c_void, channels: *mut *mut f32, num_channels: u32, num_samples: u32) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let processor = unsafe { &mut *(instance as *mut P) };
        // ... actual processing ...
    }));
    // If panic occurred, audio buffer is left unmodified (silence or pass-through)
}
```

---

## Test Plan Considerations

| Test | Type | Description |
|---|---|---|
| VTable version constant | Unit | Ensure `DEV_PROCESSOR_VTABLE_VERSION` is consistent across crates |
| FfiProcessor lifecycle | Unit | Create → process → drop without leaks or crashes |
| Vtable with mock functions | Unit | Verify FfiProcessor correctly dispatches through function pointers |
| `catch_unwind` in vtable | Unit | Verify panic in process() doesn't crash the host |
| CLI graceful fallback | Integration | Load dylib without vtable symbol → continues without audio |
| CLI version mismatch | Integration | Load dylib with wrong vtable version → warning + fallback |
| Template has no audio deps | Template validation | Scaffolded project compiles without audio-dev feature |
| End-to-end `wavecraft start` | Manual | Audio input captured, meters visible in browser UI |
