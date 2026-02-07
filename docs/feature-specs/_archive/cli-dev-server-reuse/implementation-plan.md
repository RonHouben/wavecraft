# Implementation Plan: CLI Dev Server Reuse

## Overview
Reduce duplication between `cli/src/dev_server` and engine crates by extracting shared utilities (parameter host storage, plugin param FFI loader, synthetic meter generator) into engine-level crates. This preserves the current CLI behavior while centralizing reusable logic and aligning protocol/metering types. Optional cleanups cover meter frame type unification and wavecraft-dev-server host reuse.

## Requirements
- Preserve existing `wavecraft start` behavior and CLI UX.
- Keep WebSocket server and IPC handler usage unchanged (already shared).
- Move reusable functionality into engine crates with stable APIs.
- Avoid changes to `docs/roadmap.md` and archived feature specs.
- Ensure tests cover newly extracted components.
- Optional: unify meter frame types and re-use shared host in wavecraft-dev-server.

## Architecture Changes
- **`engine/crates/wavecraft-bridge`**: introduce reusable `InMemoryParameterHost` (or similarly named) that implements `ParameterHost` for a `Vec<ParameterInfo>` + value map.
- **`engine/crates/wavecraft-core`** (or `wavecraft-bridge`): add a `PluginParamLoader` utility that wraps the FFI symbols (`wavecraft_get_params_json`, `wavecraft_free_string`).
- **`engine/crates/wavecraft-protocol`**: reuse `db_to_linear` for synthetic meters and optionally house a `MeterGenerator` in a dev-only module.
- **Optional**: unify `MeterFrame` type between `wavecraft-protocol` and `wavecraft-metering` via re-export or type move.

## Implementation Steps

### Phase 1: Shared Host & Loader Extraction (Core)
1. **Create shared in-memory host** (File: `engine/crates/wavecraft-bridge/src/in_memory_host.rs`)
   - Action: Implement `InMemoryParameterHost` with `Vec<ParameterInfo>`, `RwLock<HashMap<String, f32>>`, optional meter provider.
   - Why: Replace CLI’s `DevServerHost` storage logic with reusable component.
   - Dependencies: None.
   - Risk: Low (feature is additive).

2. **Export host in bridge crate** (File: `engine/crates/wavecraft-bridge/src/lib.rs`)
   - Action: Re-export the new host type with clear docs.
   - Why: Make it available to CLI and other tools.
   - Dependencies: Step 1.
   - Risk: Low.

3. **Create plugin param loader** (File: `engine/crates/wavecraft-bridge/src/plugin_loader.rs` or `engine/crates/wavecraft-core/src/plugin_loader.rs`)
   - Action: Move FFI loading logic from CLI into shared utility; keep error types and symbols intact.
   - Why: Centralize host-side FFI logic for parameter discovery.
   - Dependencies: None.
   - Risk: Medium (FFI safety/ABI must remain identical).

4. **Export loader utility** (File: same crate `lib.rs`)
   - Action: Re-export loader and error types.
   - Why: Allow CLI to depend on shared loader.
   - Dependencies: Step 3.
   - Risk: Low.

### Phase 2: CLI Dev Server Refactor
5. **Refactor `DevServerHost` to wrap shared host** (File: `cli/src/dev_server/host.rs`)
   - Action: Replace internal storage with `InMemoryParameterHost` (composition or type alias) and retain any CLI-specific behavior (e.g., meter generator or tick control).
   - Why: Remove duplicated parameter storage logic in CLI.
   - Dependencies: Phase 1.
   - Risk: Medium (behavioral parity with existing tests).

6. **Refactor `PluginLoader` usage** (File: `cli/src/dev_server/loader.rs` and call sites)
   - Action: Replace CLI implementation with a thin wrapper or direct use of shared `PluginParamLoader`.
   - Why: Eliminate duplicate FFI loader.
   - Dependencies: Phase 1.
   - Risk: Low.

7. **Use shared math helper for meter generator** (File: `cli/src/dev_server/meter.rs`)
   - Action: Swap local `db_to_linear` with `wavecraft_protocol::db_to_linear` or new shared helper.
   - Why: Avoid math duplication.
   - Dependencies: None.
   - Risk: Low.

### Phase 3: Tests & Validation
8. **Add/adjust unit tests for shared host** (File: `engine/crates/wavecraft-bridge/tests` or `src/in_memory_host.rs` test module)
   - Action: Port relevant tests from CLI `DevServerHost`.
   - Why: Preserve coverage after refactor.
   - Dependencies: Phase 1.
   - Risk: Low.

9. **Add/adjust tests for loader** (File: shared loader module)
   - Action: Keep error formatting tests; add a safe “symbol missing” test using a fake `Library` (or feature-gated test).
   - Why: Ensure error surfaces remain stable.
   - Dependencies: Phase 1.
   - Risk: Medium (test environment constraints).

10. **Update CLI tests** (File: `cli/src/dev_server/*` tests)
    - Action: Ensure tests still pass or move tests to shared modules.
    - Why: Maintain CI coverage.
    - Dependencies: Phase 2.
    - Risk: Low.

### Phase 4: Optional Cleanups (Nice-to-have)
11. **Unify `MeterFrame` types** (Files: `engine/crates/wavecraft-protocol`, `engine/crates/wavecraft-metering`)
    - Action: Choose canonical type; re-export or move to avoid duplication.
    - Why: Reduce type conversion and future glue code.
    - Dependencies: None, but touches multiple crates.
    - Risk: Medium (public API change).

12. **Move `MeterGenerator` to shared dev module** (File: `engine/crates/wavecraft-metering/src/dev.rs` or `wavecraft-protocol/src/dev.rs`)
    - Action: Export a dev-only synthetic generator; keep it separate from real-time metering.
    - Why: Enables reuse for other tooling or tests.
    - Dependencies: None.
    - Risk: Low.

13. **Align wavecraft-dev-server host with shared host** (File: `engine/crates/wavecraft-dev-server/src/app.rs`)
    - Action: Optionally refactor to use `InMemoryParameterHost` for consistency.
    - Why: Avoid two host implementations with similar responsibilities.
    - Dependencies: Phase 1.
   - Risk: Medium (wavecraft-dev-server tests may need update).

## Testing Strategy
- **Unit tests**:
  - `wavecraft-bridge`: `InMemoryParameterHost` value updates, range validation, `get_all_parameters`.
  - shared loader: error display and null pointer handling.
- **Integration tests**:
  - CLI `start` flow with sample plugin (smoke test).
- **Regression checks**:
  - Ensure existing CLI tests pass without changes to UX.

## Risks & Mitigations
- **FFI ABI mismatch**: loader refactor might change symbol handling.
  - Mitigation: keep symbol names and signatures identical; add tests around error paths.
- **Behavioral drift in dev server host**: parameter defaults or value handling could change.
  - Mitigation: port CLI tests to shared host and keep existing CLI tests.
- **Meter frame type unification affects public API** (optional).
  - Mitigation: treat as optional phase; consider a re-export first.

## Success Criteria
- [ ] CLI `start` behaves identically and remains stable.
- [ ] Shared `InMemoryParameterHost` is used by CLI dev server.
- [ ] Shared plugin param loader is used by CLI.
- [ ] Tests pass (CLI + engine).
- [ ] Optional: `MeterFrame` unification and shared `MeterGenerator` are implemented without breaking API expectations.
