# Implementation Plan: Processor Presence Hook (Codegen-First, v1)

## Overview

Implement a **codegen-first processor presence capability** that mirrors the existing parameter discovery flow: export processor metadata from Rust macro-generated FFI, generate `ui/src/generated/processors.ts` during `wavecraft start` (and hot reload), and expose `@wavecraft/core` hooks for runtime checks without adding new runtime IPC methods.

This keeps v1 simple, additive, and compatible with current architecture: **discovery happens at dev/build time**, UI runtime checks are local and synchronous.

## Planning checklist

- [x] Reviewed architect direction and v1 scope constraints
- [x] Mapped existing parameter discovery/codegen flow end-to-end
- [x] Identified file-level impact across macro, bridge, CLI, dev-server, core, template
- [x] Defined phased implementation with explicit dependencies
- [x] Added test strategy and validation commands
- [x] Captured risks, mitigations, and definition of done

## Requirements (v1)

1. Add processor metadata export from macro/FFI (similar to parameter export flow).
2. Generate `ui/src/generated/processors.ts` during `wavecraft start` and on Rust source hot-reload.
3. Add `@wavecraft/core` hook API:
   - `useHasProcessor(processorId: string): boolean`
   - optional `useAvailableProcessors(): string[]`
4. Support strict typing via generated map augmentation strategy (parallel to parameter typing).
5. Keep scope minimal: no new runtime IPC request/response contract for v1.

## Proposed v1 shape

### Runtime API

- `useHasProcessor(processorId: string): boolean`
- `useAvailableProcessors(): readonly string[]` (optional but recommended in same pass)

### Type safety strategy

- Add augmentable `ProcessorIdMap` in `@wavecraft/core`.
- Add `ProcessorId` type derived from generated map marker, mirroring `ParameterId`.
- Generate module augmentation in `ui/src/generated/processors.ts`:
  - marker key for augmented-empty detection
  - one entry per discovered processor id

### Discovery/source-of-truth strategy

- Reuse macro parsing of `SignalChain![...]` processor types.
- Add FFI export for processor metadata JSON (additive; do not remove existing param export).
- Extend CLI/start discovery pipeline to load processors alongside parameters.
- Generate processors TS on initial start + hot reload.

## File-level impact map

| File                                                                     | Change                                                                               | Purpose                                                              |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------ | -------------------------------------------------------------------- |
| `engine/crates/wavecraft-macros/src/plugin.rs`                           | Add processor metadata JSON export (`wavecraft_get_processors_json`)                 | Source processor list from parsed signal chain                       |
| `engine/crates/wavecraft-bridge/src/plugin_loader.rs`                    | Add processor symbol loading + parse helpers (or combined metadata loader)           | Retrieve processor metadata from dylib FFI                           |
| `engine/crates/wavecraft-protocol/src/ipc.rs` (or protocol types module) | Add `ProcessorInfo` and optional discovery metadata struct                           | Shared typed contract for CLI/bridge serialization                   |
| `cli/src/project/ts_codegen.rs`                                          | Add `write_processor_types(...)` generator with deterministic output/tests           | Generate `ui/src/generated/processors.ts`                            |
| `cli/src/project/mod.rs`                                                 | Export new codegen utility if needed                                                 | Wire new generator into project module                               |
| `cli/src/commands/start.rs`                                              | Load processor metadata; generate processors TS during startup and rebuild callbacks | Ensure start/hot-reload regenerate processor typing/runtime registry |
| `dev-server/src/reload/rebuild.rs`                                       | Extend callbacks to support processors TS generation (or unified metadata writer)    | Regenerate processors file on Rust source changes                    |
| `engine/xtask/src/commands/dev.rs`                                       | Preflight invalidation removes stale `ui/src/generated/processors.ts`                | Keep generated artifacts consistent after dependency/code changes    |
| `sdk-template/ui/src/generated/processors.ts`                            | Add generated baseline file (marker + example ids)                                   | Template parity and typed compile bootstrap                          |
| `sdk-template/ui/src/main.tsx`                                           | Import generated processors module for runtime registration side effect              | Ensure hooks have local registry at runtime                          |
| `ui/packages/core/src/types/processors.ts`                               | New augmentable processor ID map + `ProcessorId`                                     | Strict typing support                                                |
| `ui/packages/core/src/hooks/useHasProcessor.ts`                          | New hook                                                                             | Presence check API                                                   |
| `ui/packages/core/src/hooks/useAvailableProcessors.ts`                   | Optional hook                                                                        | Enumerate discovered processors                                      |
| `ui/packages/core/src/index.ts`                                          | Export new types/hooks                                                               | Public API surface                                                   |
| `ui/packages/core/src/hooks/*.test.ts` + `cli`/`bridge`/`macros` tests   | Add/extend tests                                                                     | Regressions + behavior verification                                  |

## Phased implementation with dependencies

### Phase 1 — Rust metadata export (macro + loader contract)

**Dependencies:** none (foundation phase)

1. Add processor metadata model (`ProcessorInfo`) in shared protocol crate.
2. In `wavecraft_plugin!` expansion, derive processor metadata from already-parsed `SignalChain` processor types.
3. Add new FFI export (e.g., `wavecraft_get_processors_json`) plus reuse existing free-string export.
4. Extend plugin loader to resolve new symbol and parse JSON.

**Test strategy (Phase 1):**

- Macro/unit tests: signal chain processors serialize expected ids/order.
- Bridge loader tests: missing symbol/null pointer/json parse failures for processors mirror parameter error behavior.
- Backward-compat checks: existing parameter loader tests still pass unchanged.

---

### Phase 2 — CLI discovery + processors TS codegen

**Dependencies:** Phase 1 complete

1. Add `write_processor_types(ui_dir, processors)` in `cli/src/project/ts_codegen.rs`.
   - deterministic sort
   - dedupe
   - reserved marker collision handling
   - no-op write when content unchanged
2. Update `start` flow to load processor metadata and call new writer during initial startup.
3. Update hot-reload callbacks so Rust source edits regenerate processor file.
4. Extend cache/preflight invalidation paths to include `generated/processors.ts`.
5. Keep v1 simple and compatible:
   - additive sidecar strategy (optional combined metadata sidecar with legacy read fallback), or
   - regenerate processors from discovery path whenever sidecar refresh occurs.
   - Must not break existing `wavecraft-params.json` consumers.

**Test strategy (Phase 2):**

- `ts_codegen` unit tests for processors output shape, sorting, dedupe, escaping, empty augmented mode.
- Start command tests for generation call path and stale artifact invalidation.
- Hot-reload test ensuring processors file regeneration is triggered on Rust source changes.

---

### Phase 3 — `@wavecraft/core` typed hooks + runtime registry

**Dependencies:** Phase 2 complete

1. Add processor typing primitives in `@wavecraft/core`:
   - `ProcessorIdMap`
   - marker key
   - `ProcessorId` union fallback behavior mirroring `ParameterId`
2. Implement a tiny runtime registry in core (in-memory set) and hooks:
   - `useHasProcessor(processorId: string): boolean`
   - `useAvailableProcessors(): readonly string[]` (optional but recommended)
3. In generated `ui/src/generated/processors.ts`:
   - emit module augmentation for typing
   - emit registration side effect to populate runtime registry
4. Ensure template runtime imports generated processors module (e.g., in `sdk-template/ui/src/main.tsx`).

**Test strategy (Phase 3):**

- Hook tests:
  - returns true for known id
  - false for unknown id
  - stable behavior across rerenders
- Type-level checks (tsc): known processor ids autocomplete and narrow to `ProcessorId`.
- Regression: existing parameter hook behavior unaffected.

---

### Phase 4 — Integration validation + polish

**Dependencies:** Phases 1–3 complete

1. End-to-end developer flow validation:
   - `wavecraft start` generates `parameters.ts` and `processors.ts`.
   - editing signal chain updates generated processors on rebuild.
2. Optional docs/API notes updates in `ui/packages/core/README.md`.
3. Ensure no new runtime IPC methods were introduced for v1.

**Test strategy (Phase 4):**

- Manual integration run in sdk-template project with sample chain:
  `SignalChain![Oscillator, OscilloscopeTap, ExampleProcessor, InputGain, OutputGain]`
- Confirm generated processor ids include all chain entries.
- Verify hooks return expected booleans in template UI component smoke usage.

## Validation commands

```bash
cargo test -p wavecraft-macros
cargo test -p wavecraft-bridge
cargo test --manifest-path /Users/ronhouben/code/private/wavecraft/cli/Cargo.toml
npm --prefix /Users/ronhouben/code/private/wavecraft/ui test
npm --prefix /Users/ronhouben/code/private/wavecraft/ui run typecheck
cargo xtask ci-check
```

## Risks & mitigations

- **Risk:** FFI symbol compatibility regressions.
  - **Mitigation:** Add new processor export symbol additively; do not remove/rename existing parameter symbols in v1.

- **Risk:** Cache/schema drift causes missing/stale `processors.ts`.
  - **Mitigation:** Extend invalidation logic and add fallback regeneration path + tests for stale cache scenarios.

- **Risk:** Type-only generation without runtime registration leads to hook always-false behavior.
  - **Mitigation:** Ensure generated file includes runtime registration side effect and template imports it explicitly.

- **Risk:** Processor ID naming instability (e.g., wrappers/custom type paths).
  - **Mitigation:** Document and enforce canonical id derivation (e.g., terminal type segment convention) with deterministic tests.

- **Risk:** Scope creep into runtime IPC contract.
  - **Mitigation:** Explicitly keep v1 local/codegen-only; defer IPC-based dynamic chain introspection to future phase.

## Definition of done

- [ ] `wavecraft start` generates `ui/src/generated/processors.ts` alongside parameter generation.
- [ ] Hot-reload regenerates processors file when engine Rust source changes.
- [ ] `@wavecraft/core` exposes `useHasProcessor(processorId: string): boolean`.
- [ ] Optional `useAvailableProcessors()` implemented and exported (if included in scope).
- [ ] Strict typing works via generated augmentation (`ProcessorIdMap` / `ProcessorId`) similar to parameter strategy.
- [ ] No new runtime IPC method required for processor presence in v1.
- [ ] Unit/integration tests added for macro export, loader parsing, codegen output, and hooks.
- [ ] `cargo xtask ci-check` passes.

## Out-of-scope for v1

- Runtime IPC query like `getAvailableProcessors`.
- Dynamic processor graph mutation at runtime.
- DAW/runtime-time processor presence changes beyond codegen refresh cycle.
