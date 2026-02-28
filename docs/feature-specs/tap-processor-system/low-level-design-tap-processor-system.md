# Low-Level Design: Tap Processor System

## Scope and Intent

This design keeps taps first-class and separate from reorderable audio processors, while making oscilloscope observation position runtime-dynamic and user-controlled.

- `OscilloscopeTap` is declared under `taps` (not in `processors`)
- Oscilloscope observation position is independently configurable at runtime (`0..=N`)
- Processor order remains managed only by `setProcessorOrder` / `getProcessorOrder` for the **N audio processors**
- No backwards compatibility is provided (clean break)

---

## What This Enables

1. **Draggable oscilloscope in Signal Chain UI**
   — The oscilloscope can be placed at any slot among N processors (N+1 valid positions).

2. **Runtime-dynamic observation point**
   — Position can change live via drag-and-drop without modifying processor order.

3. **Deterministic, explicit semantics**
   — `position = 0`: observe raw plugin input (before any processor).
   — `position = k (1..=N)`: observe output of processor at index `k-1` in current runtime order.

4. **Independent control surfaces**
   — Processor order and tap position are separate state machines and separate IPC contracts.

---

## Core Invariant (Explicit)

**Tap position is independent of audio processor order.**

- Changing `setProcessorOrder` does **not** modify tap position.
- Changing `setAudioSignalTapPosition` does **not** modify processor order.
- Both states coexist; the observed signal is computed from the current `tapPosition` and the current `processorOrder`.

---

## 1. DSL

### Syntax

```rust
wavecraft_plugin! {
    name: "My First Plugin",
    processors: [
        TestTone,
        InputTrim,
        Passthrough,
        ToneFilter,
        SoftClip,
        OutputGain,
    ],
    taps: [
        OscilloscopeTap,
    ],
}
```

### Parse Model

- `taps` is optional; default = empty.
- Entries are plain `TypePath` only.
- No `@ post_chain` annotation.
- No `TapAnchor` enum.
- Extend `PluginDef`:
  - `processors: Vec<Type>` (required, non-empty — unchanged)
  - `taps: Vec<Type>` (optional, new)

### Compile-Time Constraints

1. `processors` must remain non-empty.
2. `taps` may be empty or contain multiple entries.
3. Duplicate tap types are rejected at macro-expansion time.
4. Tap entries must implement `TapProcessor`.
5. Tap types are excluded from host-exposed processor catalog and order payloads.
6. **Guard:** if a `TapProcessor` implementor appears in `processors`, the macro emits `compile_error!`.

---

## 2. TapProcessor Trait

`TapProcessor` is separate from `Processor` and is used for observation-only components.

```rust
pub trait TapProcessor: Default + Send + 'static {
    fn set_sample_rate(&mut self, sample_rate: f32);
    fn reset(&mut self);
    fn observe_stereo(&mut self, left: &[f32], right: &[f32]);
}
```

`OscilloscopeTap` implements `TapProcessor`, not `Processor`. A type cannot be both.

---

## 3. IPC Contract (New, Separate from Processor Order)

Processor-order IPC is unchanged and continues to manage only N audio processors.

### 3.1 Request: `getAudioSignalTapPosition`

- Method: `getAudioSignalTapPosition`
- Params: none
- Result:

```json
{ "position": 2 }
```

Where `position` is an integer in `0..=procCount`.

### 3.2 Request: `setAudioSignalTapPosition`

- Method: `setAudioSignalTapPosition`
- Params:

```json
{ "position": 2 }
```

- Validation: reject values outside `0..=procCount`
- Result: success (empty/ack payload per existing JSON-RPC success style), or error (invalid range, malformed params)

### 3.3 Notification: `audioSignalTapPositionChanged`

- Method: `audioSignalTapPositionChanged`
- Params:

```json
{ "position": 2 }
```

Emitted whenever the effective tap position changes.

### 3.4 Position Semantics

Given `procCount = N` and current runtime order array `order[0..N-1]`:

- `position = 0` → observe raw plugin input block (before any processor runs).
- `position = k` (`1..=N`) → observe output after chain stage `k`, i.e. output of the processor at runtime index `k-1`.

**Example:** processors `[A, B, C]`, runtime order `[B, C, A]`:

- `position=0` → plugin input
- `position=1` → output of `B`
- `position=2` → output of `C` (after B)
- `position=3` → output of `A` (final chain output)

If order changes to `[A, B, C]` with `position=2` unchanged, observed source becomes output of `B`. This is expected and consistent with index-based stage semantics.

---

## 4. Codegen Changes (`wavecraft-macros/src/plugin/codegen.rs`)

### Generated Struct Layout

- Keep reorderable processor fields: `__proc_0`, `__proc_1`, ...
- Generate per-tap fields: `__tap_0`, `__tap_1`, ...
- For each tap, generate runtime position storage:
  - `__tap_0_position: AtomicUsize` (lock-free, audio-thread safe)
- Generate preallocated scratch buffers for block snapshot at tap point:
  - `__tap_0_scratch_l: Vec<f32>`, `__tap_0_scratch_r: Vec<f32>` (preallocated, reused)
- Remove special-case type-name detection for `OscilloscopeTap` in the processor list.
- Remove previous `__osc_scratch_l`/`__osc_scratch_r` interim fields entirely.

### Counts and Order Arrays

- `__PROC_COUNT = processors.len()` only (taps excluded).
- `__current_order`, pending order state, and permutation validation remain processor-only.
- Taps are excluded from: `__PROC_COUNT`, `__current_order`, processor order IPC payloads.

### Process Flow (Block-Level)

For a plugin with a tap declared:

1. Read `pos = self.__tap_0_position.load(Ordering::Acquire)`.
2. Capture into scratch based on `pos`:
   - `pos = 0`: copy raw plugin input block into scratch before any processor runs.
   - `pos = k (1..=N)`: capture chain output at stage `k` in current runtime order.
3. Continue/complete normal processor execution path.
4. After all processors have run for the block, call once:
   ```rust
   self.__tap_0.observe_stereo(&self.__tap_0_scratch_l[..num_samples], &self.__tap_0_scratch_r[..num_samples]);
   ```

### Capture Requirements

- Capture at `position=0` must use the incoming process buffer before any processor application.
- Capture for `k>0` must reflect the ordered chain stage output for the current block.
- No allocations on the audio thread; scratch buffers are reused.

---

## 5. Engine Host Contract (Non-Macro)

Add tap-position capability alongside parameter/order host APIs.

### 5.1 Trait Surface

Either extend `ParameterHost` or introduce a `TapHost` trait with:

```rust
fn get_audio_signal_tap_position(&self) -> usize;
fn set_audio_signal_tap_position(&self, position: usize) -> Result<(), BridgeError>;
```

### 5.2 Implementors to Update

- `InMemoryParameterHost`
- `DevServerHost`
- `PluginEditorBridge` in `wavecraft-nih_plug`

Each implementation must:

1. Store tap position atomically / lock-free.
2. Validate range against current processor count.
3. Publish `audioSignalTapPositionChanged` notification on updates.
4. Keep tap position state independent from processor order state.

---

## 6. UI Architecture

### 6.1 Client Layer

Add a dedicated `AudioSignalTapPositionClient` to `@wavecraft/core` implementing:

- `getAudioSignalTapPosition()`
- `setAudioSignalTapPosition(position: number)`
- Subscription for `audioSignalTapPositionChanged`

### 6.2 Hook Layer

Add a dedicated hook:

```ts
useAudioSignalTapPosition();
```

Responsibilities:

- Fetch initial position on mount
- Track live updates from `audioSignalTapPositionChanged` notification
- Expose setter with optimistic/pessimistic update strategy consistent with existing hooks

### 6.3 Signal Chain UI

`SignalChain` renders **N+1 slots**:

- N processor slots (reorderable via `setProcessorOrder`)
- 1 tap slot (draggable placement marker among processor boundaries)

Behavior:

- Dragging the oscilloscope slot updates only tap position via `setAudioSignalTapPosition`
- Processor drag-and-drop updates only processor order via `setProcessorOrder`
- Both interactions coexist without mutating each other's state

### 6.4 Package Exports

`@wavecraft/core` exports:

- `AudioSignalTapPositionClient`
- `useAudioSignalTapPosition`

---

## 7. Processor Catalog / FFI / Codegen Impact

No change:

- `wavecraft_get_processors_json()` remains processor-only.
- `ui/src/generated/processors.ts` remains processor-only.
- `useHasProcessor` / `useAvailableProcessors` remain processor-only.
- Tap presence and tap position are not encoded as processors.

---

## 8. Decisions Summary

| Decision                                      | Choice                                                    |
| --------------------------------------------- | --------------------------------------------------------- |
| DSL field name                                | `taps`                                                    |
| Tap syntax                                    | `taps: [Type]` only                                       |
| Tap annotation support                        | None (`@ post_chain` does not exist)                      |
| Anchor enum support                           | None (`TapAnchor` does not exist)                         |
| Oscilloscope placement                        | Runtime dynamic (`0..=N`)                                 |
| Processor order IPC (`set/getProcessorOrder`) | Unchanged, processor-only                                 |
| New IPC pair                                  | `getAudioSignalTapPosition` / `setAudioSignalTapPosition` |
| New notification                              | `audioSignalTapPositionChanged`                           |
| Tap fields                                    | `__tap_0`, `__tap_1`, ...                                 |
| Tap position storage                          | `__tap_0_position: AtomicUsize`                           |
| Scratch buffers                               | `__tap_0_scratch_l`, `__tap_0_scratch_r`                  |
| Taps in `__PROC_COUNT` / order arrays         | Excluded                                                  |
| Tap in processors list                        | Compile-time `compile_error!`                             |
| Backwards compatibility                       | Not provided (clean break)                                |

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — architecture overview and IPC principles
- [Coding Standards](../../architecture/coding-standards.md) — conventions and quality constraints
- [Declarative Plugin DSL](../../architecture/declarative-plugin-dsl.md) — macro system and DSL context
- [SDK Architecture](../../architecture/sdk-architecture.md) — crate/package boundaries and API surface
- [Development Workflows](../../architecture/development-workflows.md) — build/test flows
- [Plugin Formats](../../architecture/plugin-formats.md) — host/plugin integration constraints
- [Roadmap](../../roadmap.md) — milestone tracking# Low-Level Design: Tap Processor System

## Scope and Intent

This design makes taps first-class, **separate from reorderable signal-chain processors**:

- `OscilloscopeTap` is declared under `taps`
- it always observes **post-chain output** (the audible output after runtime order + bypass + crossfade)
- it is excluded from `setProcessorOrder`/`getProcessorOrder`, processor catalog, and UI drag-and-drop

---

## 1. DSL Change

### Syntax

```rust
wavecraft_plugin! {
    name: "My First Plugin",
    processors: [
        TestTone,
        InputTrim,
        Passthrough,
        ToneFilter,
        SoftClip,
        OutputGain,
    ],
    taps: [
        OscilloscopeTap,
    ],
}
```

### Parse Model

- `taps` is optional; default = empty.
- Each entry is a plain `TypePath` only — no anchor annotation. All taps implicitly observe post-chain output.
- Extend `PluginDef`:
  - `processors: Vec<Type>` (required, non-empty — unchanged)
  - `taps: Vec<Type>` (optional, new)

### Compile-Time Constraints

1. `processors` must remain non-empty (unchanged).
2. `taps` may be empty or contain multiple entries.
3. Duplicate tap types are rejected at macro-expansion time.
4. A tap must implement the `TapProcessor` trait (compile-time bound, see below).
5. Taps do **not** participate in host-exposed parameter discovery (no parameters).

### TapProcessor Trait

Introduce `TapProcessor` in `wavecraft-core`:

```rust
pub trait TapProcessor: Default + Send + 'static {
    fn set_sample_rate(&mut self, sample_rate: f32);
    fn reset(&mut self);
    fn observe_stereo(&mut self, left: &[f32], right: &[f32]);
}
```

`OscilloscopeTap` implements `TapProcessor`. `TapProcessor` is intentionally separate from the `Processor` reorder contract — a type cannot be both.

---

## 2. Codegen Changes (`wavecraft-macros/src/plugin/codegen.rs`)

### Generated Struct Layout

- Keep generated per-signal-processor fields (`__proc_0`, …) exactly as reorderable chain members.
- Generate per-tap fields (`__tap_0`, `__tap_1`, …) with concrete tap types.
- Remove special-case type-name detection for `OscilloscopeTap` in the processor list.
- Remove `__osc_scratch_l`/`__osc_scratch_r` slot-dependent scratch buffers entirely.

### Counts and Order Arrays

- `__PROC_COUNT` = `processors.len()` only (taps excluded).
- `__current_order`, `__pending_slots`, `__order_state`, and permutation validation remain tied to signal processors only.
- Taps are outside reorder state and never included in `__current_order`.

### Process Flow in Generated `process()`

```
1. Apply pending reorder (unchanged)
2. Process signal chain by iterating __current_order (unchanged)
3. Apply reorder crossfade (unchanged)
4. Write final samples to plugin output buffer
5. After the per-sample loop, for each tap:
       self.__tap_N.observe_stereo(
           &buffer.as_slice()[0][..block_len],
           &buffer.as_slice()[1][..block_len],
       );
6. Metering (unchanged)
```

Step 5 reads final output slices directly — zero allocation, zero per-sample overhead. Signal-chain processing is unchanged.

---

## 3. Tap Position Semantics

All taps observe post-chain output, defined as the signal **after**:

- runtime processor order application
- each processor's bypass behavior
- reorder crossfade gain

In other words: "what is about to leave the plugin buffer this block." This gives deterministic, user-expected behavior and matches the user-facing meaning of "show me what I hear."

---

## 4. Signal-Chain Count vs. IPC Contract

Taps are excluded from all IPC order messages. `N = processors.len()` (signal-chain only).

| Method                  | Semantics                                           |
| ----------------------- | --------------------------------------------------- |
| `getProcessorOrder`     | Returns order of signal-chain processors only       |
| `setProcessorOrder`     | Accepts permutation of signal-chain processors only |
| `processorOrderChanged` | Emits signal-chain-only order                       |

### UI Implications

- `SignalChain` drag-and-drop surface renders only signal-chain processors.
- Tap entries are invisible to the reorder UI and never participate in DnD state.
- No tap-filtering hacks required in the UI.

---

## 5. OscilloscopeTap Behavior

`OscilloscopeTap` moves from accidental in-chain pseudo-processor to explicit post-chain observer:

- Always captures the final signal-chain output block.
- Always reflects the current runtime order automatically (including reorder transitions and crossfade).
- Includes bypass effects because bypass is part of chain execution.
- No fragile type-name string match in codegen.

**Implementation note:** The generated plugin still wires the oscilloscope producer/consumer channel for IPC frame delivery. `OscilloscopeTap` is instantiated as a tap field and receives its `OscilloscopeFrameProducer` at `Default::default()` construction.

---

## 6. Processor Catalog / FFI Codegen Impact

`wavecraft_get_processors_json()` emits **signal-chain processors only**.

| Artifact                                     | Change                                        |
| -------------------------------------------- | --------------------------------------------- |
| `ProcessorInfo` FFI list                     | Excludes taps                                 |
| `ui/src/generated/processors.ts`             | Excludes taps                                 |
| `useHasProcessor` / `useAvailableProcessors` | Remain signal-chain only (correct by default) |
| `__PROC_COUNT`                               | Signal-chain count only                       |
| `setProcessorOrder` payload length           | Aligned with `__PROC_COUNT`                   |

No tap-presence IPC API in v1. If needed later, add a dedicated `wavecraft_get_taps_json()` contract.

---

## 7. Future Extensibility

This design explicitly supports additional observer taps without contaminating signal-chain reorder:

- `SpectrumAnalyzerTap`
- `LevelMeterTap` (if metering unification is desired)
- `PhaseScopeTap`, `CorrelationTap`, etc.

**Guiding rule:**

- Signal-chain processors **transform** audio and are reorderable.
- Taps **observe** the post-chain output and are non-reorderable.

If a future anchor model (e.g. `pre_chain`, `post_slot(id)`) is needed, that is a new design requiring a dedicated LLD.

---

## Decisions Summary

| Decision                                  | Choice                                      |
| ----------------------------------------- | ------------------------------------------- |
| DSL field name                            | `taps`                                      |
| Tap position                              | Always post-chain (implicit, no annotation) |
| Taps in IPC order payloads                | Excluded                                    |
| Taps in `__PROC_COUNT`                    | Excluded                                    |
| Taps in processor catalog                 | Excluded                                    |
| `OscilloscopeTap` string-match in codegen | Removed                                     |
| Oscilloscope capture point                | Post-chain output (deterministic)           |
| Backwards compatibility                   | Not provided — clean break                  |
| Tap parameters                            | None                                        |

---

## Related Documents

- [Declarative Plugin DSL](../../architecture/declarative-plugin-dsl.md)
- [High-Level Design](../../architecture/high-level-design.md)
- [SDK Architecture](../../architecture/sdk-architecture.md)
