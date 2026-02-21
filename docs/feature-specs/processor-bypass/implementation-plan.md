# Implementation Plan: Processor Bypass

## Overview

This plan adds framework-level per-processor bypass to Wavecraft's compile-time `SignalChain!` pipeline by injecting a standard bypass parameter (`{processor_id}_bypass`) for every processor instance and implementing dry passthrough when bypassed. The implementation keeps the existing IPC surface unchanged (`getParameter`/`setParameter`/`getAllParameters`) and preserves host-driven automation/persistence behavior.

The plan is phased to minimize risk in realtime DSP paths, keep parameter ordering backward-compatible where possible, and provide explicit validation from unit tests up to manual Ableton DAW verification.

## Inputs & Constraints

- User stories: `docs/feature-specs/processor-bypass/user-stories.md`
- Low-level design: `docs/feature-specs/processor-bypass/low-level-design-processor-bypass.md`
- UX design: `docs/feature-specs/processor-bypass/ux-design-processor-bypass.md`

Hard constraints:

- Realtime safety: no alloc/locks/syscalls in audio `process()`.
- No new JSON-RPC methods.
- Per-instance bypass semantics.
- Backward compatibility for existing projects/sessions as far as host behavior allows.

---

## Requirements Coverage (Acceptance-Criteria Mapping)

1. **Auto bypass parameter per processor**
   - Injected at macro generation time for every `SignalChain!` processor instance.
2. **Dry passthrough & DSP skip**
   - Bypassed stage skips child `process()` in steady state.
3. **Persistence + automation + undo/redo**
   - Bypass is a standard exposed host parameter.
4. **UI control via existing APIs**
   - Works via existing parameter APIs/hooks; optional helper hook/utilities added.
5. **Per-instance independence**
   - IDs are instance-based (`input_trim_bypass`, `output_gain_bypass`, etc.).

---

## Architecture Change Set (File-by-File)

## Engine / DSP / Macro / Host

1. `engine/crates/wavecraft-dsp/src/combinators/chain.rs`
   - Add bypass-capable wrapper/combinator (e.g. `Bypassed<P>`), plus transition-safe bypass behavior.
2. `engine/crates/wavecraft-dsp/src/combinators/mod.rs`
   - Export new bypass combinator/wrapper.
3. `engine/crates/wavecraft-dsp/src/lib.rs`
   - Re-export bypass combinator for macro-generated code.
4. `engine/crates/wavecraft-macros/src/plugin/runtime_params.rs`
   - Inject one bypass runtime param per processor instance (`_bypass`, `Stepped 0..1`, default `0`).
5. `engine/crates/wavecraft-macros/src/plugin/metadata.rs`
   - Include bypass metadata in `wavecraft_get_params_json` output.
6. `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
   - Wire generated signal type/process path to bypass-capable stage composition.
7. `engine/crates/wavecraft-macros/src/plugin/naming.rs` (if needed)
   - Centralize bypass ID/display naming helper(s) to keep deterministic naming.
8. `engine/crates/wavecraft-nih_plug/src/lib.rs` (`__internal::param_spec_to_info`)
   - Ensure bypass param is consistently emitted as `ParameterType::Bool` with expected defaults/range.
9. `engine/crates/wavecraft-nih_plug/src/editor/bridge.rs`
   - Confirm bool detection from host param map is robust for bypass params and covered by tests.
10. `engine/crates/wavecraft-bridge/src/plugin_loader.rs`
    - Ensure loader tests cover bypass params from FFI JSON.
11. `engine/crates/wavecraft-bridge/src/in_memory_host.rs`
    - Add bool/bypass behavior assertions (set/get/get_all consistency).

## Protocol

12. `engine/crates/wavecraft-protocol/src/ipc/methods.rs`
    - **No schema change expected**; ensure docs/comments/tests confirm bool handling for bypass.
13. `engine/crates/wavecraft-protocol/src/ipc.rs` (tests)
    - Extend serde/roundtrip tests with bypass-like bool parameter samples.

## UI Core SDK

14. `ui/packages/core/src/hooks/useParameter.ts`
    - Confirm bool normalization path works for bypass IDs and no regressions.
15. `ui/packages/core/src/hooks/useParameter.test.ts`
    - Add explicit bypass bool tests (load, setValue, automation update normalization).
16. `ui/packages/core/src/types/parameters.ts`
    - No type-shape change expected; ensure docs include bool semantics for bypass.
17. `ui/packages/core/src/index.ts`
    - Export optional new bypass helper APIs (if added in this feature).
18. `ui/packages/core/src/utils/*` or `ui/packages/core/src/hooks/*` (new, optional but recommended)
    - Add `isBypassParameterId`, `getProcessorBypassParamId`, and/or `useProcessorBypass`.

## CLI / Codegen

19. `cli/src/project/ts_codegen.rs`
    - Ensure generated `parameters.ts` includes bypass IDs as `boolean` typed IDs.

## SDK Template

20. `sdk-template/engine/src/lib.rs`
    - Remove/rename static `BypassStage` sample stage to avoid confusion with dynamic bypass feature.
21. `sdk-template/ui/src/App.tsx`
    - Demonstrate bypass discoverability/usage (generic controls or explicit bypass UI examples).
22. `sdk-template/ui/src/generated/*`
    - Build artifacts only; regenerated during start/build, not source-edited manually.

## Tests (new/updated files likely)

23. `engine/crates/wavecraft-dsp/src/combinators/chain.rs` (tests section)
24. `engine/crates/wavecraft-macros/src/plugin/*.rs` test modules
25. `engine/crates/wavecraft-nih_plug/src/editor/bridge.rs` tests
26. `engine/crates/wavecraft-bridge/src/{plugin_loader.rs,in_memory_host.rs}` tests
27. `ui/packages/core/src/hooks/useParameter.test.ts` (+ optional new bypass hook test file)

---

## Phased Implementation Plan

### Phase 1 — Realtime Bypass Primitive in DSP (foundation)

1. Add `Bypassed<P>` (or equivalent) in `wavecraft-dsp` combinators:
   - Holds child processor + bypass state/transition state.
   - Steady bypass: dry passthrough and no child DSP call.
   - Active: normal child process.
2. Implement bounded transition state machine (`ACTIVE`, `BYPASSED`, `TO_*`) to reduce clicks on toggle edges.
3. Add unit tests:
   - Child called/not-called as expected.
   - Mid-buffer toggles behave correctly.
   - Transition bounded and deterministic.

**Risk control:** keep transition arithmetic branch-light and allocation-free.

---

### Phase 2 — Macro Injection + Generated Runtime Wiring

1. In `runtime_params.rs`, inject bypass param spec for each processor instance:
   - ID suffix: `bypass`
   - Type source: stepped 0..1 (bool classification downstream)
   - Default: 0
   - Name: `{ProcessorDisplayName} Bypass`
2. In `metadata.rs`, include bypass in exported `ParameterInfo` list.
3. In `codegen.rs`, generate chain composition using bypass wrapper around each stage.
4. Add macro-level tests for:
   - ID naming stability (`*_bypass`).
   - Generated param count = old + number_of_processors.
   - Existing processor params unchanged.

**Critical backward-compat rule:** preserve existing non-bypass parameter ordering; append bypass params in a deterministic strategy that minimizes automation index churn.

---

### Phase 3 — Host/Bridge/Protocol Validation (no new IPC surface)

1. Confirm `param_spec_to_info` classifies bypass as `bool`.
2. Validate editor bridge path (`parameter_info_from_ptr`) emits bool + expected min/max/default.
3. Extend bridge/plugin-loader tests to include bypass-shaped params.
4. Confirm protocol serde tests include bool-bypass samples.

**Risk control:** no new methods/constants; only data-path expansion.

---

### Phase 4 — UI SDK + Template Integration

1. Strengthen `useParameter` bool tests for bypass IDs:
   - Initial load normalization
   - `setValue(true/false)` write/readback
   - Automation push update behavior
2. (Optional but recommended) add bypass helper utilities/hook:
   - `getProcessorBypassParamId(processorId)`
   - `useProcessorBypass(processorId)`
3. Update template engine/UI:
   - Remove confusing static `BypassStage` example.
   - Show bypass as discovered processor parameters and/or a small explicit toggle example.

**Risk control:** keep UI API additive and non-breaking.

---

### Phase 5 — End-to-End Verification and Release Readiness

1. Run Rust and TS tests for touched packages.
2. Run full repo validation:
   - `cargo xtask ci-check`
3. Validate generated plugin flow:
   - `cargo run --manifest-path cli/Cargo.toml -- create ... --output target/tmp/...`
   - verify generated project compile + clippy.
4. Manual DAW checks in Ableton Live (macOS primary):
   - automation lanes contain bypass params
   - real-time toggling while audio plays
   - save/reopen persistence
   - undo/redo correctness
   - no clicks/pops under low buffer settings (32/64).

---

## Explicit Test Strategy (Aligned to Acceptance Criteria)

## Unit Tests

- DSP combinator tests (`wavecraft-dsp`):
  - bypass skips child processing
  - active mode processes normally
  - per-instance independence for repeated processor types
  - transition behavior around toggle edges
- Macro tests (`wavecraft-macros`):
  - bypass param injection count, IDs, naming, defaults
  - generated processor metadata includes bypass

## Integration Tests

- Bridge/protocol/loader:
  - `getAllParameters` includes bypass entries with `type=bool`
  - `setParameter/getParameter` roundtrip for bypass values 0/1
  - parameter changed notifications reflected with correct normalized value
- UI core tests:
  - `useParameter` boolean handling for bypass IDs
  - optional `useProcessorBypass` behavior under reconnect/automation

## Manual DAW (Ableton, macOS)

- Discoverability: each processor has its own bypass parameter.
- Playback behavior:
  - bypass/unbypass mid-stream without artifacts
  - bypassed stage effect is audibly removed
- Automation:
  - write/read bypass automation lanes
  - lane names distinguish processor instances
- Persistence:
  - save/reopen session retains bypass states
  - host undo/redo toggles bypass correctly
- Performance:
  - low-buffer smoke (32/64) with rapid bypass automation.

---

## Rollout & Backward-Compatibility Guidance

1. **No IPC API breaking changes**  
   Existing clients continue to function; bypass appears as additional standard parameters.

2. **Parameter ordering stability policy**  
   Preserve existing parameter order as much as possible; introduce bypass IDs in a deterministic additive pattern. Document potential host-specific automation-lane implications when new params are introduced.

3. **Default-safe behavior**  
   All bypass defaults to `false` (active), so legacy sessions/projects keep prior audible behavior unless bypass is explicitly changed.

4. **Template migration clarity**  
   Replace static `BypassStage` sample usage with dynamic bypass explanation to avoid developer confusion.

5. **Generated code compatibility**  
   `parameters.ts` augmentation remains additive; existing type-safe IDs remain valid.

---

## Risks & Mitigations (Top Controls)

1. **Audible click/pop on bypass toggles**
   - Mitigation: bounded transition state machine + dedicated artifact regression tests.

2. **Backward compatibility risk from parameter-list expansion**
   - Mitigation: deterministic/stable ordering policy and explicit release notes for host automation mapping behavior.

3. **Realtime regressions (CPU spikes/allocations) in bypass path**
   - Mitigation: strict allocation-free implementation in `process()`, micro-focused tests, and low-buffer DAW stress checks.

---

## Definition of Done (DoD)

- [ ] Every processor instance in `SignalChain!` gets a bypass param (`*_bypass`) with bool semantics.
- [ ] Steady bypass skips child DSP and outputs dry passthrough.
- [ ] Toggle behavior is artifact-safe in practical playback tests.
- [ ] Bypass appears in `getAllParameters`, works via `setParameter/getParameter`.
- [ ] UI hooks correctly read/write bypass bool values.
- [ ] Generated TS parameter types include bypass IDs as boolean.
- [ ] SDK template demonstrates bypass usage clearly (no static bypass confusion).
- [ ] Unit + integration tests added/updated and passing.
- [ ] `cargo xtask ci-check` passes.
- [ ] Manual Ableton verification completed for automation, persistence, and undo/redo.

---

## Handoff Checklist

## To Coder

- [ ] Implement phases in dependency order (DSP foundation → macro injection → host/bridge → UI/template).
- [ ] Preserve existing param order behavior and document any unavoidable host-visible changes.
- [ ] Add tests at each touched layer before moving to next phase.
- [ ] Run full local validation (`cargo xtask ci-check` + generated-template clippy/build smoke).
- [ ] Record any host-specific caveats found during DAW validation.

## To Tester

- [ ] Execute test matrix: unit, integration, generated-template validation, manual Ableton checks.
- [ ] Verify acceptance criteria mapping explicitly (US1–US6).
- [ ] Validate no regressions in existing parameter workflows and connection/reconnect behavior.
- [ ] Report artifact/perf observations under low buffer and automation stress.

---

## Suggested Execution Order (Concise)

1. DSP bypass primitive + tests
2. Macro param injection + metadata + generated process path
3. Host/bridge/protocol validation tests
4. UI core bool/bypass hook updates + tests
5. Template update + full CI/local + manual DAW pass

---

## Documentation References

- `docs/feature-specs/processor-bypass/user-stories.md`
- `docs/feature-specs/processor-bypass/low-level-design-processor-bypass.md`
- `docs/feature-specs/processor-bypass/ux-design-processor-bypass.md`
- `docs/architecture/high-level-design.md`
- `docs/architecture/coding-standards.md`
- `docs/architecture/development-workflows.md`
