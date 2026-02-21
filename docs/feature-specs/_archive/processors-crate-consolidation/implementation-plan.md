# Implementation Plan — Processors Crate Consolidation

## Scope
Persist the implementation plan for the completed refactor that consolidates built-ins into `wavecraft-processors` and rewires prelude/macro surfaces.

## Related Documents
- [Low-Level Design](./low-level-design-processors-crate-consolidation.md)
- [High-Level Design](../../architecture/high-level-design.md)
- [Coding Standards](../../architecture/coding-standards.md)
- [SDK Architecture](../../architecture/sdk-architecture.md)
- [Roadmap](../../roadmap.md)

## Plan (Completed)

### Phase 1 — Consolidate processor ownership
- [x] Move/ensure all built-in processor implementations are in `wavecraft-processors`.
- [x] Expose built-ins from `wavecraft-processors` with a flat namespace.
- [x] Keep processor parameter structs collocated with processor implementations.

### Phase 2 — Remove DSP built-in ownership
- [x] Remove built-in implementation ownership from `wavecraft-dsp`.
- [x] Remove legacy built-in implementation files that no longer belong in `wavecraft-dsp`.
- [x] Keep `wavecraft-dsp` focused on traits/contracts/combinators.

### Phase 3 — Rewire SDK import surfaces
- [x] Update `wavecraft-core` prelude built-in exports to `wavecraft_processors::*`.
- [x] Update `wavecraft-core` `wavecraft_processor!` mappings to processor crate types.
- [x] Update `wavecraft-nih_plug` prelude re-exports to include processor crate types.

### Phase 4 — Validate
- [x] Run workspace checks for lint/type/test coverage on affected crates.
- [x] Verify no regressions in macro/prelude usage paths.
- [x] Confirm CI-style validation passes for full repo checks.

## Affected Areas
- `engine/crates/wavecraft-processors/*`
- `engine/crates/wavecraft-dsp/*`
- `engine/crates/wavecraft-core/src/prelude.rs`
- `engine/crates/wavecraft-core/src/macros.rs`
- `engine/crates/wavecraft-nih_plug/src/prelude.rs`

## Exit Criteria
- Built-ins owned by `wavecraft-processors` only.
- No built-in implementation ownership in `wavecraft-dsp`.
- Core/nih_plug prelude imports compile and expose expected built-ins.
- Validation checks pass.
