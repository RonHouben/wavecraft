# Coding Standards — Rust

## Related Documents

- [Coding Standards Overview](./coding-standards.md) — Quick reference and navigation hub
- [Declarative Plugin DSL](./declarative-plugin-dsl.md) — DSL architecture and macro system
- [SDK Architecture](./sdk-architecture.md) — Crate structure and distribution
- [Testing Standards](./coding-standards-testing.md) — Testing, logging, error handling

---

## Rust

### Module Organization

Follow the existing crate structure:

**Engine workspace** (`engine/crates/`):

- `wavecraft-nih_plug` — nih-plug integration, WebView editor, plugin exports (`publish = false`, git-only)
- `wavecraft-core` — Core SDK types and declarative macros (publishable, no nih_plug dependency)
- `wavecraft-macros` — Procedural macros: `ProcessorParams` derive, `wavecraft_plugin!`
- `wavecraft-protocol` — Shared contracts and types
- `wavecraft-dsp` — Pure DSP code, `Processor` trait, built-in processors
- `wavecraft-bridge` — IPC handling
- `wavecraft-metering` — SPSC ring buffer for audio → UI metering

**Standalone crate** (`dev-server/` at repo root):

- `wavecraft-dev-server` — Unified dev server with WebSocket, hot-reload, audio I/O (feature-gated), FFI processor. Features: `default = ["audio"]`. CLI uses with `default-features = false`. Not published (`publish = false`).

### Declarative Plugin DSL

**Rule:** Use the declarative DSL macros for new plugin definitions. Manual `Plugin` implementations should be avoided unless necessary for advanced use cases.

**Processor Wrapper Macro (built-in processors only):**

```rust
// ✅ Use wavecraft_processor! for named wrappers around built-in processors
wavecraft_processor!(InputGain => Gain);
wavecraft_processor!(OutputStage => Passthrough);

// ❌ Do NOT use wavecraft_processor! for custom processors
// wavecraft_processor!(MyOsc => Oscillator); // Wrong — Oscillator is custom
```

> **Note:** `wavecraft_processor!` only supports built-in processor types (`Gain`, `Passthrough`).
> Custom processors implementing the `Processor` trait go directly in `SignalChain![]`.

**Custom Processors in Signal Chain:**

```rust
use wavecraft::prelude::*;
use wavecraft::Oscillator;

// Built-in processors need wrappers (provides parameter-ID prefix)
wavecraft_processor!(InputGain => Gain);
wavecraft_processor!(OutputGain => Gain);

// Custom processors are used directly — they already have their own Params type
wavecraft_plugin! {
    name: "My Plugin",
    signal: SignalChain![InputGain, Oscillator, OutputGain],
}
```

**Plugin Definition Macro:**

```rust
// ✅ Use wavecraft_plugin! for complete plugin generation
wavecraft_plugin! {
    name: "My Plugin",
    signal: SignalChain![InputGain],
}
```

> **Note:** `vendor` and `url` are derived from `Cargo.toml` metadata. `email` is not exposed as a macro property and defaults internally to an empty string. The `signal` field requires `SignalChain![]` wrapper.

**Parameter Definition:**

```rust
use wavecraft::prelude::*;
// The prelude provides the ProcessorParams *trait*;
// this explicit import brings the *derive macro* (same name, different namespace).
use wavecraft::ProcessorParams;

// ✅ Use #[derive(ProcessorParams)] for parameter structs
#[derive(ProcessorParams, Default)]
struct GainParams {
    #[param(range = "-60.0..=24.0", default = 0.0, unit = "dB")]
    gain: f32,

    #[param(range = "0.0..=1.0", default = 1.0, unit = "%", group = "Output")]
    mix: f32,
}
```

**Param Attribute Options:**
| Attribute | Required | Description | Example |
|-----------|----------|-------------|---------|
| `range` | Yes | Value range as `"MIN..=MAX"` | `range = "-60.0..=24.0"` |
| `default` | No | Default value (defaults to midpoint) | `default = 0.0` |
| `unit` | No | Unit string for display | `unit = "dB"` |
| `factor` | No | Skew factor (>1 = log, <1 = exp) | `factor = 2.5` |
| `group` | No | UI grouping name | `group = "Input"` |

### xtask Commands

The `xtask` crate provides build system commands. Each command is a module under `commands/`:

```
engine/xtask/src/
├── lib.rs           # Shared utilities (paths, platform, output)
├── main.rs          # CLI definition (clap)
└── commands/
    ├── mod.rs              # Command exports and run_all()
    ├── build_ui.rs         # Two-stage UI dist build
    ├── bundle.rs           # Build VST3/CLAP bundles
    ├── cd_dry_run.rs       # CD dry-run change detection (used by ci-check --full)
    ├── check.rs            # ci-check: 6-phase pre-push validation
    ├── clean.rs            # Clean build artifacts
    ├── desktop.rs          # Desktop POC
    ├── dev.rs              # Dev servers (WebSocket + Vite)
    ├── install.rs          # Install plugins to system directories
    ├── lint.rs             # Unified linting (UI + Engine, incl. tsc --noEmit)
    ├── notarize.rs         # Apple notarization
    ├── release.rs          # Complete release workflow
    ├── sign.rs             # macOS code signing
    ├── test.rs             # Run tests (engine + UI)
    ├── validate_cli_deps.rs # Validate CLI dependency versions
    └── validate_template.rs # Validate CLI template generation
```

**Command conventions:**

- Each command module exposes a `run()` function as entry point
- Use `anyhow::Result` for error propagation
- Use `xtask::output::*` helpers for colored terminal output
- Platform checks: `if Platform::current() != Platform::MacOS { bail!(...) }`
- Configuration from environment: `Config::from_env()` pattern
- Unit tests in `#[cfg(test)] mod tests { }` at bottom of file

**Adding a new command:**

1. Create `commands/mycommand.rs` with `pub fn run(...) -> Result<()>`
2. Register in `commands/mod.rs`: `pub mod mycommand;`
3. Add CLI variant in `main.rs`: `enum Commands { MyCommand { ... } }`
4. Wire up in `main()` match: `Some(Commands::MyCommand { .. }) => commands::mycommand::run(...)`

### Naming Conventions

| Type      | Convention       | Example                           |
| --------- | ---------------- | --------------------------------- |
| Structs   | PascalCase       | `IpcHandler`, `AppState`          |
| Traits    | PascalCase       | `ParameterHost`                   |
| Functions | snake_case       | `handle_request`, `get_parameter` |
| Methods   | snake_case       | `fn set_sample_rate(&mut self)`   |
| Constants | UPPER_SNAKE_CASE | `const WINDOW_WIDTH: u32`         |
| Modules   | snake_case       | `mod params`, `mod handler`       |

### Platform-Specific Code

**Rule:** Use `#[cfg(target_os = "...")]` attributes for platform-specific code. Do not use `#[allow(dead_code)]` to suppress warnings for platform-gated items.

Wavecraft is primarily developed for macOS, with the editor/WebView components being platform-specific. Code that only runs on certain platforms should be properly gated.

**Patterns:**

```rust
// ✅ Platform-gate the entire item (imports, functions, statics)
#[cfg(any(target_os = "macos", target_os = "windows"))]
use include_dir::{Dir, include_dir};

#[cfg(any(target_os = "macos", target_os = "windows"))]
static UI_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../../ui/dist");

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn get_asset(path: &str) -> Option<(&'static [u8], &'static str)> {
    // ...
}

// ✅ Platform-gate tests that use platform-specific functions
#[cfg(test)]
mod tests {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    use super::*;

    #[test]
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn test_platform_specific_function() {
        // Test only runs on macOS/Windows
    }
}

// ✅ Use #[allow(dead_code)] ONLY for trait methods called by platform implementations
// (Rust's analysis can't see calls from platform-specific code)
pub trait WebViewHandle: Any + Send {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn evaluate_script(&self, script: &str) -> Result<(), String>;

    /// Note: Called by platform implementations, not from trait consumers.
    #[allow(dead_code)]
    fn resize(&self, width: u32, height: u32);
}
```

**Don't:**

```rust
// ❌ Using #[allow(dead_code)] instead of proper platform-gating
#[allow(dead_code)]
pub fn get_asset(path: &str) -> Option<...> {
    // This compiles everywhere but is only used on macOS
}

// ❌ Using `test` in cfg to make code compile for tests on all platforms
#[cfg(any(target_os = "macos", target_os = "windows", test))]
static UI_ASSETS: Dir = ...;  // Compiles on Linux CI but isn't used
```

**Rationale:**

- Platform-gated code should only compile on platforms where it's used
- This catches real dead code (lint checks work correctly)
- Linux CI doesn't need to compile macOS/Windows GUI code
- `#[allow(dead_code)]` should be reserved for legitimate false positives (e.g., trait methods called by platform impls)

### Real-Time Safety

Code running on the audio thread must:

- Never allocate (`Box::new`, `Vec::push`, `String::from`)
- Never lock (`Mutex`, `RwLock`)
- Never make system calls that can block
- Use atomic types for shared state
- Use SPSC ring buffers for data transfer

### Lock-Free Parameter Bridge Pattern

**Rule:** Use `AtomicF32` with immutable `HashMap` structure for passing parameter values from non-RT threads to the audio thread.

When parameter values need to flow from a WebSocket/UI thread to the audio callback, the bridge must be lock-free on the read side. The `AtomicParameterBridge` pattern achieves this:

1. Build a `HashMap<String, Arc<AtomicF32>>` once at startup (one entry per parameter)
2. Never mutate the `HashMap` after construction — only the atomic values change
3. Non-RT thread writes via `AtomicF32::store(value, Ordering::Relaxed)`
4. Audio thread reads via `AtomicF32::load(Ordering::Relaxed)`

**Do:**

```rust
use atomic_float::AtomicF32;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub struct AtomicParameterBridge {
    params: HashMap<String, Arc<AtomicF32>>,  // Immutable after construction
}

impl AtomicParameterBridge {
    pub fn new(parameters: &[ParameterInfo]) -> Self {
        let params = parameters.iter()
            .map(|p| (p.id.clone(), Arc::new(AtomicF32::new(p.default))))
            .collect();
        Self { params }
    }

    // Called from WebSocket thread (non-RT)
    pub fn write(&self, id: &str, value: f32) {
        if let Some(atomic) = self.params.get(id) {
            atomic.store(value, Ordering::Relaxed);
        }
    }

    // Called from audio thread (RT-safe: single atomic load, no allocation)
    pub fn read(&self, id: &str) -> Option<f32> {
        self.params.get(id).map(|a| a.load(Ordering::Relaxed))
    }
}
```

**Don't:**

```rust
// ❌ RwLock on audio thread (blocks, can deadlock)
let params = Arc::new(RwLock::new(HashMap::new()));
// in audio callback:
let value = params.read().unwrap().get("gain").copied();

// ❌ Mutex on audio thread
let value = params.lock().unwrap().get("gain").copied();
```

**Rationale:**

- `Relaxed` ordering is sufficient: parameter updates are not synchronization points. A one-block delay (~5-12ms) is imperceptible.
- The `HashMap` is immutable after construction, so `get()` on the audio thread is safe (no reallocation possible).
- `Arc<AtomicF32>` is `Send + Sync`, so the bridge is auto-derived as `Send + Sync` by the compiler — no `unsafe impl` needed.

### SPSC Ring Buffer for Inter-Thread Communication

**Rule:** Use `rtrb` SPSC ring buffers for passing data from audio callbacks to async tasks.

When data needs to flow from a real-time audio callback to a non-RT consumer (e.g., meter data → WebSocket broadcast), use an `rtrb::RingBuffer` instead of `tokio` channels. Tokio channels (`mpsc::UnboundedSender`) allocate a linked-list node per `send()`, violating real-time safety.

**Do:**

```rust
// Pre-allocate ring buffer before stream creation
let (mut meter_producer, meter_consumer) =
    rtrb::RingBuffer::<MeterUpdateNotification>::new(64);

// Audio callback (RT-safe: no allocation)
let _ = meter_producer.push(notification);  // Drops on overflow — acceptable for metering

// Async task (non-RT)
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(16));
    loop {
        interval.tick().await;
        let mut latest = None;
        while let Ok(frame) = meter_consumer.pop() {
            latest = Some(frame);  // Keep only the latest
        }
        if let Some(frame) = latest {
            ws_handle.broadcast(&frame).await;
        }
    }
});
```

**Don't:**

```rust
// ❌ tokio unbounded channel in audio callback (allocates per send)
let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
// in audio callback:
let _ = tx.send(meter_data);  // Allocates a linked-list node!
```

### nih-plug Buffer Write Pattern

**Rule:** When converting nih-plug buffers to wavecraft DSP format, use the unsafe pointer write pattern with comprehensive safety documentation.

nih-plug's `Buffer` API only provides immutable slices via `as_slice()`, but plugin processing requires in-place modification. The cast from `*const f32` to `*mut f32` is safe because the DAW host guarantees exclusive buffer access during the `process()` callback.

**Why unsafe is necessary:**

1. nih-plug's `Buffer` API only provides `as_slice() -> &[&[f32]]` (immutable references)
2. The wavecraft DSP API requires mutable slices: `process(&mut self, buffer: &mut [&mut [f32]])`
3. Plugin contract allows (and expects) in-place modification during `process()` callback

**Do:**

```rust
// Bounds check before unsafe block
if let Some(channel) = buffer.as_slice().get(ch) {
    if sample_idx < channel.len() {
        // SAFETY JUSTIFICATION:
        //
        // 1. Exclusive Access: nih-plug's process() callback guarantees exclusive
        //    buffer access (no concurrent reads/writes from other threads).
        //
        // 2. Bounds Check: The `if` guards above ensure:
        //    - `ch` is a valid channel index (within buffer.channels())
        //    - `sample_idx < channel.len()` (within channel sample count)
        //
        // 3. Pointer Validity:
        //    - `channel.as_ptr()` is from nih-plug's Buffer allocation (valid)
        //    - `.add(sample_idx)` offset is within bounds (checked above)
        //    - Pointer is properly aligned (f32 alignment guaranteed by host)
        //
        // 4. Write Safety:
        //    - f32 is Copy (atomic write, no drop required)
        //    - No aliasing: Buffer<'a> lifetime ensures no other refs exist
        //    - Host expects in-place modification (plugin contract)
        //
        // 5. Why unsafe is necessary:
        //    nih-plug's Buffer API only provides immutable refs (as_slice()).
        //    However, the plugin contract allows (and expects) in-place writes.
        //    Casting *const → *mut is sound because we have exclusive access
        //    during process() callback (guaranteed by DAW host).
        unsafe {
            let channel_ptr = channel.as_ptr() as *mut f32;
            *channel_ptr.add(sample_idx) = value;
        }
    }
}
```

**Don't:**

```rust
// ❌ Minimal safety comment (insufficient justification)
unsafe {
    let channel_ptr = channel.as_ptr() as *mut f32;
    *channel_ptr.add(sample_idx) = value; // we're within bounds
}

// ❌ Copying buffer to avoid unsafe (allocates on audio thread)
let mut temp_buffer = vec![0.0; buffer.samples()]; // VIOLATES REAL-TIME SAFETY
// ... copy input, process, copy back (defeats zero-copy design)
```

**Rationale:**

- nih-plug's immutable refs are a convenience API, not an ownership guarantee
- The DAW host provides exclusive access during `process()` (single-threaded audio callback)
- Alternative approaches (copying buffers) violate real-time safety (allocations on audio thread)
- This pattern is used in the macro-generated code (`wavecraft-macros/src/plugin.rs`)

### FFI Safety Patterns

**Rule:** All `extern "C"` functions that cross a dylib boundary must use `catch_unwind` to prevent panics from unwinding across the FFI boundary. All `unsafe` blocks interacting with FFI must have `// SAFETY:` annotations.

Wavecraft uses C-ABI FFI vtables to load user DSP processors from compiled cdylibs at runtime (see [Dev Audio via FFI](./development-workflows.md#dev-audio-via-ffi) in the development workflows). This pattern requires strict safety discipline.

**Panic Safety at FFI Boundaries:**

```rust
// ✅ Every extern "C" function wraps its body in catch_unwind
#[unsafe(no_mangle)]
pub extern "C" fn wavecraft_dev_create_processor() -> DevProcessorVTable {
    extern "C" fn process(
        instance: *mut c_void,
        channels: *mut *mut f32,
        num_channels: u32,
        num_samples: u32,
    ) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // ... actual processing ...
        }));
        // If panic occurred, audio buffer is left unmodified (silence/passthrough)
    }
    // ... other vtable functions similarly wrapped ...
}

// ❌ Missing catch_unwind — UB if user DSP code panics
extern "C" fn process(instance: *mut c_void, ...) {
    let processor = unsafe { &mut *(instance as *mut P) };
    processor.process(...); // Panic here = undefined behavior
}
```

**Memory Ownership Across Dylib Boundaries:**

```rust
// ✅ All alloc/dealloc happens inside the dylib — no cross-allocator issues
extern "C" fn create() -> *mut c_void {
    Box::into_raw(Box::new(Processor::default())) as *mut c_void  // dylib allocates
}

extern "C" fn drop_fn(instance: *mut c_void) {
    if !instance.is_null() {
        let _ = unsafe { Box::from_raw(instance as *mut Processor) };  // dylib frees
    }
}

// ❌ CLI frees memory allocated by dylib — allocator mismatch, UB
let processor = unsafe { Box::from_raw(ptr as *mut SomeType) };  // WRONG allocator
```

**VTable Versioning:**

```rust
// ✅ Version field enables fail-fast contract checks on ABI mismatch
#[repr(C)]
pub struct DevProcessorVTable {
    pub version: u32,  // Must equal DEV_PROCESSOR_VTABLE_VERSION
    pub create: extern "C" fn() -> *mut c_void,
    pub process: extern "C" fn(...),
    // ...
}
```

**SAFETY Comments for FFI `unsafe` Blocks:**

```rust
// ✅ Comprehensive SAFETY annotations
// SAFETY: `library` is a valid loaded Library. If the symbol doesn't exist,
// `get()` returns Err; callers must surface this as an explicit contract error
// (or use an explicit opt-in compatibility mode), never as a silent default path.
// The symbol type is trusted to match the macro-generated `extern "C"` function.
let symbol: Symbol<DevProcessorVTableFn> =
    unsafe { library.get(b"wavecraft_dev_create_processor\0").ok()? };

// ❌ Bare unsafe without justification
let symbol = unsafe { library.get(b"wavecraft_dev_create_processor\0").ok()? };
```

**Key Invariants:**

| Invariant                  | Enforcement                                                         |
| -------------------------- | ------------------------------------------------------------------- |
| No panics across FFI       | `catch_unwind` in every `extern "C"` function                       |
| No cross-allocator frees   | All alloc/dealloc inside dylib via vtable functions                 |
| Library outlives processor | Struct field order (`_library` last) + caller variable order        |
| ABI compatibility          | `#[repr(C)]` structs, `extern "C"` fn pointers, version field       |
| Null pointer safety        | Guards in `create()` return, `drop_fn()`, and `FfiProcessor::new()` |

---

## Validation

### Validation Against Language Specifications

**Rule:** When validating identifiers, keywords, or language constructs, use the language's own parser/lexer libraries instead of maintaining custom lists.

**Rationale:**

- **Future-proof**: Automatically stays current with language updates (new keywords, editions)
- **Authoritative**: Uses the language's official rules as source of truth
- **Comprehensive**: Covers all cases including strict keywords, reserved words, and edition-specific additions
- **Maintainable**: No manual lists to keep in sync

**Do (Rust keyword validation):**

```rust
use syn;

/// Validates that a name is not a Rust keyword.
/// Uses syn's parser - the same rules Rust itself uses.
pub fn validate_not_keyword(name: &str) -> Result<()> {
    // Convert hyphens to underscores (crate names allow hyphens)
    let ident_name = name.replace('-', "_");

    // syn::parse_str::<syn::Ident>() fails for keywords
    if syn::parse_str::<syn::Ident>(&ident_name).is_err() {
        bail!("'{}' is a reserved Rust keyword", name);
    }
    Ok(())
}
```

**Don't (hardcoded keyword list):**

```rust
// ❌ Hardcoded list becomes stale as language evolves
const KEYWORDS: &[&str] = &[
    "fn", "let", "if", "else", "match", // incomplete...
    // Missing: async, await, try, dyn, etc.
];

fn validate_not_keyword(name: &str) -> Result<()> {
    if KEYWORDS.contains(&name) {
        bail!("Reserved keyword");
    }
    Ok(())
}
```

**Why syn for Rust:**

- `syn` is the de-facto standard Rust parser, used by proc-macros
- `syn::Ident` parsing uses Rust's official keyword list
- Automatically includes edition-specific keywords (e.g., `async`/`await` in 2018+)
- Zero maintenance burden for keyword list updates

**Similar patterns for other languages:**

- **TypeScript**: Use TypeScript compiler API for identifier validation
- **JavaScript**: Use `acorn` or `esprima` parser libraries

---

## Error Prevention

### Rust `unwrap()` and `expect()` Usage

**Rule:** Avoid `unwrap()` in production code. Use `expect()` with descriptive messages or proper error handling.

**Rationale:**

- `unwrap()` panics without context, making debugging difficult
- `expect()` provides a message explaining why the operation should succeed
- Proper error handling with `?` is preferred when errors are recoverable

**Production Code:**

```rust
// ✅ Use expect() with justification for infallible operations
// Serialization of well-typed Response structs cannot fail because:
// - All fields are simple types (strings, numbers, Options)
// - No custom serializers that could error
// - serde_json always succeeds for #[derive(Serialize)] types
serde_json::to_string(&response).expect("Response serialization is infallible")

// ✅ Use ? operator for fallible operations
let config = SigningConfig::from_env()?;
let data = serde_json::from_str::<Request>(json)?;

// ✅ Use if-let or match for optional handling
if let Some(param) = params.get(id) {
    // use param
}

// ❌ Avoid bare unwrap() in production
let value = some_option.unwrap();  // No context if it fails
let data = serde_json::from_str(json).unwrap();  // Hides parse errors
```

**Test Code:**

```rust
// ✅ Prefer expect() with descriptive messages in tests
let result: GetParameterResult = serde_json::from_value(response.result.clone())
    .expect("response should contain valid GetParameterResult");

// ✅ Use assert! macros for test assertions
assert!(response.result.is_some(), "expected successful response");
assert_eq!(result.value, 0.5, "parameter value should be 0.5");

// ⚠️ unwrap() is acceptable in tests when the intent is obvious
// but expect() is preferred for better failure messages
let error = response.error.unwrap();  // Acceptable but not ideal
```

**When `unwrap()` is Acceptable:**

1. **Infallible operations with documentation**: When an operation mathematically cannot fail and this is documented in a comment
2. **Test setup code**: Where failure indicates a test bug, not a product bug
3. **Compile-time constants**: `NonZeroU32::new(2).unwrap()` in const contexts

**Pattern for IPC Response Serialization:**

The `IpcHandler::handle_json()` method uses `unwrap()` for serializing responses. This is acceptable because:

```rust
// IpcResponse derives Serialize with simple field types:
// - RequestId (enum of u32/String)
// - Option<Value> (serde_json::Value, always serializable)
// - Option<IpcError> (simple struct with String fields)
//
// serde_json::to_string() cannot fail for these types.
serde_json::to_string(&response).expect("IpcResponse serialization is infallible")
```
