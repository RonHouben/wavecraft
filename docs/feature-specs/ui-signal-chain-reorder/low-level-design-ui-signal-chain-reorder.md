# Low-Level Design: Runtime-Reorderable SignalChain

## Problem Statement

Wavecraft currently models processor order as a compile-time type-level composition:

- `signal: SignalChain![A, B, C]`
- expands to nested generic types (`Chain<Bypassed<A>, Chain<Bypassed<B>, ...>>`)
- execution order is fixed at compile time

We need runtime reorder driven by a drag-and-drop UI SignalChain component, where:

1. SDK developers and end-users can reorder processors at runtime
2. UI is the source of truth for order intent
3. Order persists per-preset across DAW project save/load
4. No real-time safety violations in audio `process()`
5. Existing processor parameter contracts continue to work independent of order

---

## Constraints

1. **Real-time safety (hard constraint)**
   - Audio thread must not allocate, lock, block, or perform syscalls.
2. **Processor identity stability**
   - Processor instances carry state (filter memories, envelopes, etc.) and must survive reorder.
3. **Parameter contract stability**
   - Existing prefixed IDs (e.g., `tone_filter_cutoff_hz`, `tone_filter_bypass`) must remain valid and order-independent.
4. **Persistence required**
   - Order must survive DAW session roundtrip, persisted per preset.
5. **No static runtime override**
   - Runtime order is authoritative once loaded/received. No need for backwards compatibility or migration (pre-v1).
6. **Cross-layer consistency**
   - Protocol, bridge, macro codegen, and UI core must use the same canonical `ProcessorId` set.
7. **All processors are reorderable**
   - Including analysis/tap processors (e.g., `OscilloscopeTap`).

---

## Solution Options

### Option A — Generated static dispatch + runtime order table (Recommended)

#### Approach

Keep processors as concrete typed instances, but decouple execution order from type nesting.

- Macro generates:
  - stable processor instance storage (`p0`, `p1`, ... typed fields)
  - stable processor ID table (`["test_tone", "input_trim", ...]`)
  - runtime order table (`[slot_idx; N]`) where each entry references a processor instance
- Audio processing executes processors by iterating current order table.
- Per-processor processing call is generated as `match slot_idx { ... }` static dispatch (no trait object).

#### Audio thread design

- Audio thread holds local active order snapshot (`[u8; N]`).
- Control/UI thread validates new order and publishes to a lock-free/atomic handoff (versioned fixed-size array).
- Audio thread checks version at block boundary and swaps snapshot.
- Short crossfade at block boundary on reorder to eliminate click/zipper artifacts (preallocated scratch buffers).

#### Persistence

Persist order in plugin state serialization (non-automatable state chunk), per preset.

- On load: if valid permutation → apply.
- If absent (first run): default = registration order.

#### Macro API

Replace `signal: SignalChain![...]` with unordered registration:

```rust
wavecraft_plugin! {
    name: "MyPlugin",
    processors: [TestTone, InputTrim, ToneFilter, SoftClip, OutputGain, OscilloscopeTap],
}
```

No backwards compatibility layer needed (pre-v1). The `signal:` field is removed.

Generated code:

- registers available processors
- builds stable instance IDs
- initialises default order (registration order), then overwrites from restored state/UI updates

#### Pros

- Best real-time profile (no virtual dispatch)
- Processor params and IDs remain stable
- No dynamic allocation in process path
- Strong compile-time checks for processor types

#### Cons

- Largest macro/codegen rewrite
- Generated code complexity increases
- Reorder transition needs careful DSP polish (crossfade)

---

### Option B — Type-erased processor nodes + runtime scheduler

#### Approach

Build runtime vector of boxed processor nodes (`Vec<Box<dyn ErasedProcessorNode>>`) plus runtime order indices. Each node wraps a typed processor + typed params mapping internally.

#### Audio thread design

- Audio thread iterates `order_indices` and invokes virtual `process_node(...)`.
- Reorder handoff same as Option A (atomic snapshot at block boundary + crossfade).
- Nodes are allocated once at init; no runtime alloc on audio thread.

#### Persistence

Same as Option A (plugin state chunk, per preset).

#### Macro API

Same `processors: [...]` field, with simpler codegen than Option A.

#### Pros

- Simpler macro codegen
- Natural abstraction for heterogeneous runtime chains
- Easier future graph features

#### Cons

- Virtual dispatch overhead per node per block
- Weaker compile-time inlining/optimization
- More care needed to avoid accidental heap use in node internals

---

### Option C — Runtime DAG engine with serial-chain compatibility mode

#### Approach

Replace chain abstraction with a small graph runtime. Processors are nodes, edges define signal flow. Current feature uses a serial path, but architecture supports future split/merge routing.

#### Audio thread design

- Precompiled execution plan (topological order) swapped atomically.
- Serial reorder is a plan update.

#### Persistence

Persist graph model (nodes + edges + order metadata) in plugin state.

#### Pros

- Most extensible long-term architecture
- Future-proofs modulation/routing features

#### Cons

- Overkill for current requirement
- Highest implementation/test burden
- Biggest surface for RT regressions

---

## Recommended Approach

Adopt **Option A**.

**Rationale:**

- Best fit for Wavecraft's real-time and deterministic goals
- Preserves current "processor IDs are contract anchors" model
- Avoids dynamic dispatch in the hottest path
- Scales to typical chain lengths with predictable CPU behaviour
- Short crossfade implementation fits naturally at block-boundary swap

---

## IPC Contract Changes

Minimum required additions:

1. **Method**: `setProcessorOrder`
   - params: `{ order: ProcessorId[] }`
   - full permutation of all registered processor IDs
   - validation: all IDs known, no duplicates, contains every registered processor exactly once

2. **Notification**: `processorOrderChanged`
   - params: `{ order: ProcessorId[] }`
   - emitted after successful apply, on state restore, and on initial sync

3. **Method**: `getProcessorOrder`
   - result: `{ order: ProcessorId[] }`
   - for robust UI bootstrap (avoids race if UI mounts after initial notification)

**Error addition:**

- `invalidProcessorOrder` — invalid permutation or unknown IDs

---

## Macro API Change

Replace `signal:` with `processors:`:

```rust
// Before (removed)
signal: SignalChain![Passthrough, ToneFilter, SoftClip],

// After
processors: [Passthrough, ToneFilter, SoftClip],
```

The `processors:` field is unordered registration. Runtime order is owned by the UI.

---

## Persistence Design

Order is stored in plugin state (non-automatable state chunk), per preset.

**State schema (example):**

```json
{
  "version": 1,
  "processorOrder": [
    "test_tone",
    "input_trim",
    "tone_filter",
    "soft_clip",
    "output_gain",
    "oscilloscope_tap"
  ]
}
```

**Behaviour:**

- Load: if valid permutation → apply; otherwise default to registration order.
- Per-preset: each preset snapshot includes its own `processorOrder`.

---

## UI Design

See companion UI design document (to be added by UX Designer).

**Confirmed requirements:**

- SignalChain component with drag-and-drop reordering
- Each processor rendered as an expanded card showing all parameters inline
- Bypass toggle visible on each chain item
- Layout direction: deferred to UX Designer
- All processors (including analysis/tap) are reorderable

---

## Crossfade Design

Reorder transitions use a short crossfade to eliminate click/zipper artifacts without feeling sluggish.

- **Duration**: 256 samples (~5.8ms at 44.1kHz, ~5.3ms at 48kHz) — short enough to feel instantaneous, long enough to avoid an audible click at typical block sizes.
- **Implementation**: hardcoded constant in the engine. Not developer-configurable.
- **Buffer**: preallocated scratch buffer (2× block size) allocated at plugin init, reused on every crossfade. No allocation on the audio thread.

```rust
const REORDER_CROSSFADE_SAMPLES: usize = 256;
```

---

## References

- [Declarative Plugin DSL](../../architecture/declarative-plugin-dsl.md)
- [High-Level Design](../../architecture/high-level-design.md)
- [Coding Standards — Rust](../../architecture/coding-standards-rust.md)
- [SDK Architecture](../../architecture/sdk-architecture.md)
