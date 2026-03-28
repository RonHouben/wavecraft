# Low-Level Design: Tap Processor System

## 1. Scope and Intent

This design defines a **unified signal-chain ordering model** for processors and taps.
It replaces split runtime control surfaces (processor order vs tap position) with a single contract: `SignalChainOrder`.

### Intent

- Keep taps first-class via DSL compile-time declaration (`taps: [...]`)
- Represent runtime signal flow with one ordered slot list
- Preserve real-time safety in generated engine code
- Simplify UI behavior to one drag/drop model and one mutation path

### Explicit scope changes

- `getProcessorOrder` / `setProcessorOrder` / `processorOrderChanged` are replaced by:
  - `getSignalChainOrder`
  - `setSignalChainOrder`
  - `signalChainOrderChanged`
- No backward compatibility path is provided (clean break).

---

## 2. Core Invariants

1. **Single runtime ordering authority**
   - Runtime order is represented only by `SignalChainOrder[]`.

2. **Typed slot identity**
   - Each slot carries an explicit `type` (`'processor'` or `'tap'`) and an `id` (`ProcessorId` or `AudioSignalTapId`).

3. **DSL remains compile-time, ordering remains runtime**
   - `taps: [...]` declares tap types at compile time.
   - Runtime placement is determined by `SignalChainOrder[]`.

4. **Tap processors are observers, not chain processors**
   - Tap types do not participate in processor catalog or processor-only FFI surfaces.

5. **Real-time safety**
   - No allocations on audio thread.
   - Scratch buffers are preallocated and reused.
   - Audio-thread read of resolved ordering/capture indexes is lock-free.

6. **No migration compatibility layer**
   - Old processor-order contract is removed, not bridged.

---

## 3. DSL

### Syntax

\`\`\`rust
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
\`\`\`

### Parse Model

- \`processors\` remains required and non-empty.
- \`taps\` remains optional and defaults to empty.
- \`taps\` entries are plain \`TypePath\` values.
- No annotations, no anchors in DSL.

### Compile-Time Constraints

1. \`processors\` must be non-empty.
2. \`taps\` may be empty or contain multiple entries.
3. Duplicate tap types are rejected at macro expansion.
4. Tap entries must implement \`TapProcessor\`.
5. If a \`TapProcessor\` implementor appears in \`processors\`, macro emits \`compile_error!\`.
6. Tap types are excluded from processor catalog surfaces (\`wavecraft_get_processors_json\` and generated processor lists).

---

## 4. TapProcessor Trait

\`TapProcessor\` is the observation contract and is intentionally distinct from \`Processor\`:

\`\`\`rust
pub trait TapProcessor: Default + Send + 'static {
    fn set_sample_rate(&mut self, sample_rate: f32);
    fn reset(&mut self);
    fn observe_stereo(&mut self, left: &[f32], right: &[f32]);
}
\`\`\`

\`OscilloscopeTap\` implements \`TapProcessor\`, not \`Processor\`. A type cannot be both.

---

## 5. SignalChainOrder IPC Contract

Replaces \`setProcessorOrder\` / \`getProcessorOrder\` / \`processorOrderChanged\`.

### 5.1 Slot Type

\`SignalChainOrder\` is the type of a **single slot** in the chain. The full order is \`SignalChainOrder[]\` at call sites.

TypeScript:

\`\`\`typescript
type ProcessorId = string;
type AudioSignalTapId = string;

type SignalChainOrder = {
  id: ProcessorId | AudioSignalTapId;
  type: 'processor' | 'tap';
};
\`\`\`

Rust domain type:

\`\`\`rust
pub struct SignalChainSlot {
    pub id: String,
    pub slot_type: SlotType,
}

pub enum SlotType {
    Processor,
    Tap,
}
\`\`\`

Wire format (JSON) per slot:

\`\`\`json
{ "id": "TestTone", "type": "processor" }
{ "id": "OscilloscopeTap", "type": "tap" }
\`\`\`

### 5.2 \`getSignalChainOrder\`

- Method: \`getSignalChainOrder\`
- Params: none
- Result:

\`\`\`json
{
  "slots": [
    { "id": "OscilloscopeTap", "type": "tap" },
    { "id": "TestTone", "type": "processor" },
    { "id": "SoftClip", "type": "processor" }
  ]
}
\`\`\`

Returns full runtime slot order (processors + taps) reflecting current effective chain.

### 5.3 \`setSignalChainOrder\`

- Method: \`setSignalChainOrder\`
- Params:

\`\`\`json
{
  "slots": [
    { "id": "TestTone", "type": "processor" },
    { "id": "OscilloscopeTap", "type": "tap" },
    { "id": "SoftClip", "type": "processor" }
  ]
}
\`\`\`

Validation rules (all must pass):

1. Payload must be an array of slot objects with \`id\` (string) and \`type\` (\`"processor"\` | \`"tap"\`).
2. Every \`id\` must be a known processor or tap type name for this plugin instance.
3. \`type\` must match the declared kind for that \`id\`.
4. No duplicates.
5. No omissions — slot set must be an exact permutation of all declared runtime slots.
6. Length must equal \`processor_count + tap_count\`.

On failure: return JSON-RPC error (invalid params).
On success: atomically update runtime order and emit \`signalChainOrderChanged\`.

### 5.4 \`signalChainOrderChanged\` Notification

- Notification: \`signalChainOrderChanged\`
- Params:

\`\`\`json
{
  "slots": [
    { "id": "TestTone", "type": "processor" },
    { "id": "OscilloscopeTap", "type": "tap" },
    { "id": "SoftClip", "type": "processor" }
  ]
}
\`\`\`

Emitted whenever the effective runtime order changes.

### 5.5 Position Semantics

Tap observation point is derived from tap slot index in the resolved \`SignalChainOrder[]\`.

For a tap at slot index \`k\`:

- \`k = 0\` → observe raw plugin input (before any processor runs)
- \`k > 0\` → observe output of the last processor slot before index \`k\`
- tap at end → observe final chain output

**Example — tap in the middle:**

\`\`\`json
[
  { "id": "A", "type": "processor" },
  { "id": "OscilloscopeTap", "type": "tap" },
  { "id": "B", "type": "processor" },
  { "id": "C", "type": "processor" }
]
\`\`\`

Resolved processor execution order: \`[A, B, C]\`
Tap is at index 1 → observes output of \`A\`.

**Example — tap at end:**

\`\`\`json
[
  { "id": "A", "type": "processor" },
  { "id": "B", "type": "processor" },
  { "id": "C", "type": "processor" },
  { "id": "OscilloscopeTap", "type": "tap" }
]
\`\`\`

Tap observes final chain output (\`C\` output).

---

## 6. OscilloscopeFrame Data Contract

Separate from chain ordering — this is the waveform payload delivered via SPSC ring buffer.

### 6.1 \`getOscilloscopeFrame\` IPC Method

- Method: \`getOscilloscopeFrame\`
- Params: none
- Result:

\`\`\`json
{
  "frame": {
    "left": [0.0, 0.1, -0.05],
    "right": [0.0, 0.08, -0.04],
    "timestamp": 1700000000
  }
}
\`\`\`

or \`{ "frame": null }\` when no frame is available.

### 6.2 Naming (parallel to \`MeterFrame\`)

| Layer | Meter | Oscilloscope |
|---|---|---|
| Rust type | \`MeterFrame\` | \`OscilloscopeFrame\` |
| Rust producer | \`MeterProducer\` | \`OscilloscopeFrameProducer\` |
| Rust consumer | \`MeterConsumer\` | \`OscilloscopeFrameConsumer\` |
| IPC method | \`getMeterFrame\` | \`getOscilloscopeFrame\` |
| UI hook | \`useMeterFrame\` | \`useOscilloscopeFrame\` |

No shared base type between \`MeterFrame\` and \`OscilloscopeFrame\` — parallel patterns, distinct domain types.

### 6.3 \`useOscilloscopeFrame\` Hook

- Polls \`getOscilloscopeFrame\` at interval (same transport pattern as \`useMeterFrame\`)
- Returns \`OscilloscopeFrame | null\`
- Lifecycle and connection semantics match existing frame hooks

---

## 7. Codegen Changes (\`wavecraft-macros/src/plugin/codegen.rs\`)

### 7.1 Generated Struct Layout

- Processor fields: \`__proc_0\`, \`__proc_1\`, ... (unchanged)
- Tap fields: \`__tap_0\`, \`__tap_1\`, ...
- Per-tap scratch buffers: \`__tap_0_scratch_l\`, \`__tap_0_scratch_r\`, ... (preallocated, reused)
- Remove special-case type-name detection for \`OscilloscopeTap\`.
- Remove previous \`__osc_scratch_l\` / \`__osc_scratch_r\` interim fields entirely.

### 7.2 Runtime Resolution from \`SignalChainOrder[]\`

At control boundary (non-audio thread, atomic handoff):

1. Parse \`SignalChainOrder[]\` into:
   - Processor execution sequence (processor-only slots, in order)
   - Per-tap insertion index (count of processor slots before the tap in the full list)
2. Store resolved state lock-free for audio thread consumption.
3. Audio thread reads resolved processor order and tap insertion index(es) — no JSON parsing on RT path.

### 7.3 Process Flow (Block-Level)

1. Read resolved processor order and tap insertion index(es) (lock-free atomic load).
2. If tap insertion index is \`0\`, copy input block into \`__tap_N_scratch_*\` before any processing.
3. Execute per-sample/per-processor loop in resolved processor order.
4. After each processor stage, check if any tap insertion index targets that boundary; if so, copy output to corresponding scratch buffer.
5. After all processor stages complete, call each tap's \`observe_stereo\` with its scratch buffers:
   \`\`\`rust
   self.__tap_0.observe_stereo(
       &self.__tap_0_scratch_l[..num_samples],
       &self.__tap_0_scratch_r[..num_samples],
   );
   \`\`\`
6. Continue existing oscilloscope frame publication flow.

No allocations on audio thread. All scratch memory is reused.

---

## 8. Engine Host Contract

### 8.1 Trait Surface

Replace processor-order methods with unified order methods:

\`\`\`rust
fn get_signal_chain_order(&self) -> Vec<SignalChainSlot>;
fn set_signal_chain_order(&self, order: Vec<SignalChainSlot>) -> Result<(), BridgeError>;
\`\`\`

### 8.2 Implementors to Update

- \`InMemoryParameterHost\`
- \`DevServerHost\`
- \`PluginEditorBridge\` (\`wavecraft-nih_plug\`)
- Bridge/protocol handlers currently serving processor-order endpoints

Each must:

1. Validate exact slot permutation rules.
2. Persist/update order atomically.
3. Emit \`signalChainOrderChanged\` on effective change.
4. Remove legacy processor-order endpoint handling.

---

## 9. UI Architecture

### 9.1 \`SignalChainOrderClient\`

Replaces \`ProcessorOrderClient\`. Responsibilities:

- \`getSignalChainOrder(): Promise<SignalChainOrder[]>\`
- \`setSignalChainOrder(slots: SignalChainOrder[]): Promise<void>\`
- Subscribe to \`signalChainOrderChanged\`

### 9.2 \`useSignalChainOrder\` Hook

Replaces \`useProcessorOrder\`. Responsibilities:

- Fetch initial unified order on mount
- Subscribe to \`signalChainOrderChanged\` notifications
- Expose optimistic/pessimistic setter consistent with existing hook patterns

### 9.3 \`useOscilloscopeFrame\` Hook

Separate from order state:

- Polls \`getOscilloscopeFrame\`
- Returns \`OscilloscopeFrame | null\`
- Mirrors \`useMeterFrame\` transport lifecycle

### 9.4 \`SignalChain\` Component

Renders a **single unified ordered list** of \`N + T\` slots (N processors + T taps).

Behavior:

- One drag-and-drop model for all slot types
- One mutation path: \`setSignalChainOrder(slots: SignalChainOrder[])\`
- No merging of separate processor-order and tap-position state

### 9.5 Package Exports

\`@wavecraft/core\` exports:

- \`SignalChainOrderClient\`
- \`SignalChainOrder\` type
- \`useSignalChainOrder\`
- \`OscilloscopeFrame\` type
- \`useOscilloscopeFrame\`

Legacy \`ProcessorOrderClient\` and \`useProcessorOrder\` are removed from public surface.

---

## 10. Processor Catalog / FFI Impact

No change to processor catalog:

- \`wavecraft_get_processors_json()\` remains processor-only.
- \`ui/src/generated/processors.ts\` remains processor-only.
- \`useHasProcessor\` / \`useAvailableProcessors\` remain processor-only.
- Taps appear in \`SignalChainOrder[]\` but not in processor catalog contracts.

---

## 11. Decisions Summary

| Decision | Choice |
|---|---|
| Runtime ordering model | Unified \`SignalChainOrder[]\` |
| \`SignalChainOrder\` type | Single slot: \`{ id: ProcessorId \| AudioSignalTapId, type: 'processor' \| 'tap' }\` |
| Wire slot format | \`{ "id": "TypeName", "type": "processor" \| "tap" }\` |
| IPC methods | \`getSignalChainOrder\`, \`setSignalChainOrder\` |
| IPC notification | \`signalChainOrderChanged\` |
| Legacy processor-order contract | Replaced; no backward compatibility |
| DSL tap declaration | \`taps: [Type]\` |
| TapProcessor trait | \`set_sample_rate\`, \`reset\`, \`observe_stereo\` |
| Generated tap fields | \`__tap_0\`, \`__tap_1\`, ... |
| Scratch buffers | \`__tap_0_scratch_l\`, \`__tap_0_scratch_r\`, ... |
| Compile-time guard | \`compile_error!\` if TapProcessor type appears in \`processors\` |
| Engine capture semantics | Derived from tap index in unified \`SignalChainOrder[]\` |
| UI order hook | \`useSignalChainOrder\` replaces \`useProcessorOrder\` |
| UI client | \`SignalChainOrderClient\` replaces \`ProcessorOrderClient\` |
| Oscilloscope payload naming | \`OscilloscopeFrame\`, \`getOscilloscopeFrame\`, \`useOscilloscopeFrame\` |
| Meter/Oscilloscope type relation | Parallel patterns, no shared base type |
| Backwards compatibility | Not provided (clean break) |
| Migration guide | None |

---

## 12. Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — architecture overview and IPC principles
- [Coding Standards](../../architecture/coding-standards.md) — project-wide conventions
- [Declarative Plugin DSL](../../architecture/declarative-plugin-dsl.md) — macro and DSL context
- [SDK Architecture](../../architecture/sdk-architecture.md) — crate/package boundaries
- [Development Workflows](../../architecture/development-workflows.md) — build/test workflows
- [Plugin Formats](../../architecture/plugin-formats.md) — host/plugin integration constraints
- [Roadmap](../../roadmap.md) — milestone tracking
