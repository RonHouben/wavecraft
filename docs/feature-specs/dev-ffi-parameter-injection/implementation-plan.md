# Implementation Plan: Dev FFI Parameter Injection v2 and DSP Unification

## Overview

This plan delivers FFI v2 parameter injection for browser dev audio and removes parameter-semantic DSP duplication from `dev-server`, making `wavecraft-processors` the single DSP source of truth. Work is split into six incremental phases with explicit validation gates, a temporary opt-in v1 compatibility fallback during migration, and a hard parity gate before final cutover.

---

## Scope

### In Scope

- FFI v2 ABI: `apply_plain_values` entrypoint on `DevProcessorVTable`
- Macro-generated FFI wrapper with runtime param cache so live params reach DSP
- Dev-server block-boundary parameter apply path (RT-safe, lock-free)
- Removal (or strict compat-gating) of parameter-semantic DSP in `output_modifiers.rs`
- Loader version enforcement and actionable mismatch diagnostics
- Documentation updates for the development workflow FFI contract

### Out of Scope

- New UI features
- DAW/plugin runtime changes beyond required macro/protocol/bridge integration
- Long-term permanent backwards compatibility for v1 FFI

---

## Prerequisites

- Design doc reviewed and signed off: [low-level-design-dev-ffi-parameter-injection.md](./low-level-design-dev-ffi-parameter-injection.md)
- `wavecraft-protocol::DevProcessorVTable` v1 structure understood
- `wavecraft_plugin!` macro codegen path understood
- Dev-server audio callback pipeline understood (`ffi_processor.rs`, `input_pipeline.rs`, `output_modifiers.rs`)

---

## Phase-by-Phase Implementation

### Phase 1 — Protocol: Introduce FFI v2 ABI

**Files:**

- `engine/crates/wavecraft-protocol/src/dev_audio_ffi.rs`
- `engine/crates/wavecraft-protocol/src/lib.rs`

**Changes:**

- Add v2 function pointer to `DevProcessorVTable`:
  ```rust
  apply_plain_values: unsafe extern "C" fn(
      instance: *mut c_void,
      values_ptr: *const f32,
      len: usize,
  ),
  ```
- Bump `DEV_PROCESSOR_VTABLE_VERSION` to `2`
- Update ABI doc comments to describe v2 layout and versioning rules

**Validation gate:** `cargo test -p wavecraft-protocol`

**Rollback:** Revert vtable bump; downstream phases are not yet wired.

---

### Phase 2 — Macros: Generate v2-Compatible Processor Wrapper

**Files:**

- `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
- `engine/crates/wavecraft-macros/src/plugin.rs`
- `engine/crates/wavecraft-macros/tests/processor_params.rs`

**Changes:**

- Generated dev FFI wrapper stores a mutable `__Params` cache per instance
- `create()` initializes the param cache once from defaults
- `process()` uses the cached params (no per-callback `from_param_defaults()`)
- Generate `apply_plain_values(instance, values_ptr, len)` implementation that calls `ProcessorParams::apply_plain_values()`
- Register new pointer in generated vtable init block
- Add/update macro snapshot tests for the v2 wrapper

**Validation gate:** `cargo test -p wavecraft-macros`

**Rollback:** Retain internal v1 behavior branch behind a flag until parity gate passes.

---

### Phase 3 — Bridge: Strict v2 Contract

**Files:**

- `engine/crates/wavecraft-bridge/src/plugin_loader.rs`
- `engine/crates/wavecraft-bridge/src/lib.rs` (if needed)

**Changes:**

- Default loader requires vtable version `2`; version mismatch fails fast
- On mismatch, diagnostics report: found version, expected version, and remediation step (rebuild with current SDK)
- Enforce v2-only contract unconditionally

**Validation gate:** `cargo test -p wavecraft-bridge`

**Rollback:** If the compat path is unstable, default to strict-only and remove the env var.

---

### Phase 4 — Dev-Server: Block-Boundary Parameter Injection (v2-only)

**Files:**

- `dev-server/src/audio/ffi_processor.rs`
- `dev-server/src/audio/atomic_params.rs`
- `dev-server/src/audio/server/input_pipeline.rs`
- `dev-server/src/audio/server/output_modifiers.rs`
- `dev-server/src/audio/server.rs`
- `dev-server/src/host.rs`

**Changes:**

- Extend the safe Rust processor wrapper in `ffi_processor.rs` with a call-through for `apply_plain_values`
- Evolve `atomic_params.rs` to support a dense indexed snapshot bridge (SPSC ring or atomic generation) if not already compatible
- In `input_pipeline.rs` audio callback: drain parameter updates at block boundary, call `apply_plain_values` before `process`
- No allocations, locks, HashMap lookups, or string work in the callback hot path
- Remove old `output_modifiers.rs` parameter-semantic DSP compatibility path from the runtime callback flow
- `DevServerHost` forwards parameter updates from IPC/WebSocket into the lock-free bridge on parameter set

**Validation gate:** `cargo test -p wavecraft-dev-server`

**Rollback:** Rebuild plugin/runtime against the current SDK contracts.

---

### Phase 5 — Remove DSP Duplication by Default

> **Prerequisite:** Parity suite must pass before this phase is merged (see [Parity Gate](#parity-gate)).

**Files:**

- `dev-server/src/audio/server/output_modifiers.rs`
- `dev-server/src/audio/server/input_pipeline.rs`
- `dev-server/src/audio/server.rs`

**Changes:**

- Remove parameter-semantic DSP (gain, saturation, oscillator, clip logic) from `output_modifiers.rs`
- Retain only non-semantic audio transport utilities in `output_modifiers.rs`, or remove the file entirely
- Remove all remaining v1 compatibility branches from runtime startup and injection flow

**Validation gate:**

1. `cargo test -p wavecraft-dev-server`
2. `cargo xtask ci-check --skip-docs`

**Rollback:** Re-enable compat branch only; no permanent v1 DSP path remains after this phase.

---

### Phase 6 — Docs and Migration Closure

**Files:**

- `docs/architecture/development-workflows.md`
- `docs/feature-specs/dev-ffi-parameter-injection/implementation-progress.md` (Coder creates/updates)

**Changes:**

- Update `development-workflows.md` to describe FFI v2 parameter injection flow and data path
- Remove v1 compatibility mode guidance and document v2-only startup/injection behavior
- Note block-boundary injection semantics and RT-safety constraints

**Validation gate:** `cargo xtask ci-check`

---

## Parity Gate

Phase 5 is gated on parity suite pass. The suite must:

- Run the same processor, input buffer, and parameter automation schedule through both the dev FFI path and the reference processor path
- Compare outputs sample-by-sample (strict epsilon)
- Cover oscillator, filter, saturator, and gain processors as a minimum

Passing the parity suite is a required signal before removing compatibility-era runtime code.

---

## Validation Matrix

| Phase | Command                                                                   |
| ----- | ------------------------------------------------------------------------- |
| P1    | `cargo test -p wavecraft-protocol`                                        |
| P2    | `cargo test -p wavecraft-macros`                                          |
| P3    | `cargo test -p wavecraft-bridge`                                          |
| P4    | `cargo test -p wavecraft-dev-server`                                      |
| P5    | `cargo test -p wavecraft-dev-server` + `cargo xtask ci-check --skip-docs` |
| P6    | `cargo xtask ci-check`                                                    |

---

## Compatibility and Rollback Policy

| Default behavior            | Strict v2 FFI required                                              |
| --------------------------- | ------------------------------------------------------------------- |
| Migration escape hatch      | None (v2-only runtime)                                              |
| Compat window               | N/A                                                                 |
| Compat removal trigger      | Completed as part of v2-only cleanup                                |
| Permanent behavior (post-5) | Strict v2 only; compat flag removed; ABI mismatch always fails fast |

---

## Acceptance Criteria Mapping

| AC  | Description                                                          | Addressed by |
| --- | -------------------------------------------------------------------- | ------------ |
| AC1 | Browser dev params affect DSP through the FFI path                   | Phases 2 + 4 |
| AC2 | `wavecraft-processors` is the single DSP source of truth             | Phase 5      |
| AC3 | ABI version mismatch fails fast with actionable diagnostics          | Phases 1 + 3 |
| AC4 | Audio callback path is allocation-free and lock-free in steady state | Phase 4      |
| AC5 | Deterministic parity tests pass before cutover                       | Phases 4 + 5 |
| AC6 | `output_modifiers.rs` no longer implements parameter-semantic DSP    | Phase 5      |
| AC7 | Development workflow documentation updated                           | Phase 6      |

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview and component relationships
- [Coding Standards](../../architecture/coding-standards.md) — Conventions for Rust and TypeScript code
- [Roadmap](../../roadmap.md) — Milestone tracking and progress
- [SDK Architecture](../../architecture/sdk-architecture.md) — SDK crate and package boundaries
- [Development Workflows](../../architecture/development-workflows.md) — Dev/build/test and FFI parameter flow
- [Low-Level Design: Dev FFI Parameter Injection](./low-level-design-dev-ffi-parameter-injection.md) — Technical design backing this plan
