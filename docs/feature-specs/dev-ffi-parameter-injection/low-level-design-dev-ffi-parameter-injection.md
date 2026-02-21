# Low-Level Design: Dev FFI Parameter Injection v2 and DSP Unification

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Project conventions
- [Roadmap](../../roadmap.md) — Milestones and progress
- [SDK Architecture](../../architecture/sdk-architecture.md) — SDK/package boundaries
- [Development Workflows](../../architecture/development-workflows.md) — Dev/build/test and FFI parameter flow
- [Declarative Plugin DSL](../../architecture/declarative-plugin-dsl.md) — Macro system and parameter discovery

---

## Overview

Browser dev mode currently duplicates processor DSP logic in `dev-server/src/audio/server/output_modifiers.rs` because the `wavecraft_plugin!`-generated FFI `process()` constructs processor parameters from defaults on every callback (`from_param_defaults()`), making live UI/WebSocket parameter updates invisible to the DSP path.

This design removes that duplication by:

1. Introducing **FFI v2** — a versioned ABI extension that adds a `apply_plain_values` entrypoint to `DevProcessorVTable`.
2. Wiring the dev-server parameter injection path to forward updates to the audio callback via a lock-free bridge.
3. Retiring parameter-semantic DSP from `output_modifiers.rs`.

After this change, `wavecraft-processors` is the single DSP source of truth for both browser dev mode and plugin/DAW runtime.

---

## Assumptions

1. The `wavecraft_plugin!`-generated FFI `process()` currently constructs params from defaults every callback; live UI/WS parameter updates do not reach DSP in dev FFI mode.
2. `dev-server/src/audio/server/output_modifiers.rs` is a temporary DSP duplication layer used to recover audible parameter behavior in browser dev mode.
3. Pre-1.0 policy mandates strict contract enforcement by default; temporary compatibility paths are explicit opt-in, never the default.

---

## Constraints & Invariants

1. **Real-time safety is non-negotiable** — no allocations, locks, string work, or HashMap lookups in the audio callback hot path.
2. **Parameter injection at block boundary only** — updates are drained once per block, not mid-block.
3. **Lock-free bounded control bridge** — SPSC ring or atomic generation snapshot; drop counters exposed for observability.
4. **No DSP duplication in dev-server** — dev-server routes audio/meter/oscilloscope transport concerns only.
5. **Fail-fast on ABI mismatch** — incompatible FFI versions produce actionable diagnostics, never silent degradation.

---

## Target Architecture

### Components and Ownership

| Component           | Owner                                    | Responsibility                                 |
| ------------------- | ---------------------------------------- | ---------------------------------------------- |
| DSP source of truth | `wavecraft-processors`                   | `Processor` / `ProcessorParams` implementation |
| FFI contract        | `wavecraft-protocol::DevProcessorVTable` | Versioned ABI definition                       |
| Generated wrapper   | `wavecraft-macros`                       | Emits FFI wrapper storing runtime param cache  |
| Dev host state      | `dev-server::DevServerHost`              | Canonical parameter state for IPC/UI           |
| Parameter bridge    | `dev-server::audio::atomic_params`       | RT-safe update delivery to audio thread        |
| ABI loader          | `wavecraft-bridge::PluginParamLoader`    | Enforces version compatibility at load time    |

### Data Flow (Target)

```
UI → setParameter(id, value) [JSON-RPC]
  │
  ▼
DevServerHost
  ├── updates canonical host state (for IPC reads)
  └── writes to lock-free parameter update bridge
                │
                ▼ (block boundary drain)
         Audio callback
                │
                ▼
         FFI v2: apply_plain_values(instance, values_ptr, len)
                │
                ▼
         Generated wrapper: __Params cache updated via
         ProcessorParams::apply_plain_values()
                │
                ▼
         Processor::process(..., &params)
                │
                ▼
         Meter / oscilloscope transport
```

`output_modifiers.rs` transitions from a DSP behavior layer to either:

- removed entirely, or
- limited to non-semantic plumbing (audio routing/transport only), never parameter-driven tone/gain/clip/oscillator DSP.

---

## Migration Strategy

Migration is incremental with explicit rollback gates at each phase.

### Phase A — Instrumentation and Parity Baseline

- Add non-audio-thread metrics/logging for: parameter update rates, applied update generations, dropped updates under queue pressure.
- Document current parity mismatch as a baseline for Phase D acceptance gate.

**Rollback:** None required (observability only).

### Phase B — Introduce FFI v2 in Parallel

- Add v2 ABI with `apply_plain_values` entrypoint (see [FFI v2 Contract](#ffi-v2-contract)).
- Keep v1 path fully intact.
- Macro generates v2 exports by default for current SDK builds.

**Rollback:** `--dev-ffi-v1-compat` env flag forces v1 path.

### Phase C — Wire Dev-Server Parameter Injection to v2

- Replace callback-side DSP duplication with v2 parameter apply + process call.
- Gate old `output_modifiers` DSP path behind explicit compatibility switch only.

**Rollback:** Flip compatibility switch to restore previous behavior immediately.

### Phase D — Parity Gate and Burn-In

- Run deterministic parity suite: dev FFI output vs direct processor/plugin reference path (same input, same automation schedule).
- Passing the suite is required before defaulting the compatibility switch to off.

**Rollback:** Keep compatibility switch for one release window.

### Phase E — Remove Duplication

- Remove parameter-driven DSP from `output_modifiers.rs`.
- Remove compatibility switch after burn-in period.
- Strict fail-fast on incompatible ABI remains permanently.

**Rollback:** None after formal deprecation window.

---

## FFI v2 Contract

### `DevProcessorVTable` Evolution

**v1** (current) provides: `create`, `process`, `set_sample_rate`, `reset`, `drop`.

**v2** adds:

```rust
apply_plain_values: unsafe extern "C" fn(
    instance: *mut c_void,
    values_ptr: *const f32,
    len: usize,
),
```

Batch `apply_plain_values` is preferred over per-parameter `set_parameter_by_index` for lower call overhead and deterministic per-block apply semantics.

### Generated Wrapper State (Macro)

Per instance the generated wrapper stores:

- Processor instance `__P`
- Mutable params cache `__Params`
- Optional generation counter for idempotent apply behavior

`process()` uses the cached params. `apply_plain_values()` mutates the cache via `ProcessorParams::apply_plain_values()`.

### Versioning

- `DEV_PROCESSOR_VTABLE_VERSION` bumped to `2`.
- Loader defaults to strict v2 requirement (pre-1.0 fail-fast policy).
- Optional explicit flag (`--dev-ffi-v1-compat`) may allow v1 during migration period only.
- On version mismatch, diagnostics must report: found version, expected version, and remediation (rebuild plugin with current SDK or disable compat flag).

---

## Real-Time Safety Constraints

1. No allocations, locks, or string work in the audio callback.
2. No HashMap lookups by parameter ID strings in the callback hot path; use dense indexed arrays derived from stable `ParameterInfo` generation order.
3. Parameter injection occurs at block boundary only; updates are drained once per block.
4. Lock-free bounded structures only for control updates (SPSC ring or atomic generation snapshots).
5. No logging on the steady-state audio callback path.
6. Preserve existing `catch_unwind` panic containment at FFI boundary; avoid panic-prone logic in callback.
7. Preserve existing preallocated deinterleave/process/interleave/meter pipeline invariants.

---

## Testing Strategy

### Unit Tests

| Crate                | Tests                                                                                                      |
| -------------------- | ---------------------------------------------------------------------------------------------------------- |
| `wavecraft-protocol` | Vtable v2 layout and version constant                                                                      |
| `wavecraft-macros`   | Generated v2 wrapper: defaults on init, `apply_plain_values` updates params, `process` uses updated params |
| `dev-server`         | Parameter bridge update ordering and coalescing correctness                                                |

### Integration Tests

| Area                        | Tests                                                                                                                     |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `wavecraft-bridge` loader   | v2 load success; version mismatch fails fast; explicit compat-mode behavior                                               |
| `dev-server` audio pipeline | Parameter update from WS thread reaches process callback within bounded block count; no `output_modifiers` DSP dependence |

### End-to-End Parity Tests

**Offline deterministic harness:**

- Same processor, same input buffer, same parameter automation schedule.
- Compare dev FFI output vs reference processor path sample-by-sample (strict epsilon).

**Browser dev E2E:**

- Scripted `setParameter` sweeps for oscillator, filter, saturator, and gain processors.
- Assert expected meter/frame behavior changes and absence of stale-default behavior.

Parity suite pass is a hard gate before Phase D completion (defaulting off the compatibility switch).

---

## Risks and Mitigations

| Risk                                                                          | Mitigation                                                                                                  |
| ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| ABI churn (v1 → v2)                                                           | Strict version diagnostics + short explicit compatibility flag window with documented sunset date           |
| Parameter ordering mismatch (dense plain-value arrays depend on stable order) | Derive ordering from `ParameterInfo` generation contract; validate length/order checksum in debug and tests |
| Behavior mismatch with plugin host automation semantics                       | Parity harness uses same `ProcessorParams::apply_plain_values` path and shared processor code               |
| Queue pressure / dropped updates                                              | Coalesce by latest value per parameter per block; expose drop counters for observability                    |
| Accidental residual DSP duplication                                           | Architectural guardrail: dev-server must not contain processor-semantic DSP transforms                      |

---

## File-Level Impact Map

### `engine/crates/wavecraft-protocol`

- `src/dev_audio_ffi.rs` — vtable v2 additions, `DEV_PROCESSOR_VTABLE_VERSION` bump
- `src/lib.rs` — export updates and doc comments

### `engine/crates/wavecraft-macros`

- `src/plugin/codegen.rs` — generated processor wrapper: `__Params` cache state, new vtable function stubs

### `engine/crates/wavecraft-bridge`

- `src/plugin_loader.rs` — v2 validation, compatibility switch behavior and sunset policy

### `dev-server`

- `src/audio/ffi_processor.rs` — safe Rust wrapper for new vtable entrypoint(s)
- `src/audio/server/input_pipeline.rs` — invoke `apply_plain_values` before `process` at block boundary
- `src/audio/atomic_params.rs` — evolve toward dense/indexed snapshot bridge if needed
- `src/audio/server/output_modifiers.rs` — deprecate and remove parameter-driven DSP
- `src/host.rs` — parameter forwarding path; bridge mapping changes as needed

### Documentation

- `docs/architecture/development-workflows.md` — update FFI contract and parameter injection flow description

---

## Acceptance Criteria

1. Browser dev mode parameter changes audibly affect DSP through the FFI processor path without relying on `output_modifiers` DSP logic.
2. `wavecraft-processors` is the single DSP semantics source for both dev FFI and plugin/DAW runtime.
3. FFI ABI version mismatch fails fast with actionable diagnostics (found version, expected version, remediation).
4. The audio callback path is allocation-free and lock-free in steady state.
5. Deterministic parity tests pass for the agreed processor set and automation scenarios.
6. `output_modifiers.rs` no longer implements parameter-semantic DSP (or is removed entirely).
7. This feature-spec document is present at the canonical path.

---

## Implementation-Ready Checklist

- [x] Define v2 FFI parameter-apply ABI (`DevProcessorVTable`) and versioning rules
- [x] Define macro-generated runtime param cache model (`__Params` lifecycle)
- [x] Define dev-server block-boundary parameter injection stage placement
- [x] Define v1 compatibility switch and sunset policy
- [x] Define RT-safety guardrails and no-duplication architectural rule
- [x] Define parity test matrix (unit / integration / E2E) and pass gates
- [x] Define file-level impact map and sequencing dependencies
- [x] Define documentation artifact path and title
