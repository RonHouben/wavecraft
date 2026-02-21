# Low-Level Design — Processors Crate Consolidation

## Summary
Consolidate built-in processor implementations under `wavecraft-processors` and remove built-in ownership from `wavecraft-dsp`. This clarifies crate responsibilities and keeps `wavecraft-dsp` focused on DSP contracts/combinators.

## Related Documents
- [High-Level Design](../../architecture/high-level-design.md)
- [Coding Standards](../../architecture/coding-standards.md)
- [SDK Architecture](../../architecture/sdk-architecture.md)
- [Development Workflows](../../architecture/development-workflows.md)
- [Roadmap](../../roadmap.md)

## Problem
Built-in processors were historically mixed into `wavecraft-dsp` concerns, creating unclear boundaries between:
- reusable processor implementations, and
- DSP interfaces/combinator primitives.

This increased coupling and made ownership ambiguous for prelude/macro exports.

## Goals
1. All built-in processors live in `wavecraft-processors`.
2. `wavecraft-processors` uses a flat namespace (no `builtins` submodule).
3. `wavecraft-dsp` no longer owns built-in implementations.
4. Core/prelude surfaces continue to provide ergonomic imports.
5. Legacy implementation stubs are removed from `wavecraft-dsp`.

## Non-Goals
- No behavior changes to processor algorithms themselves.
- No UI/IPC protocol changes.
- No roadmap updates in this implementation.

## Design Decisions

### 1) Ownership boundary
- `wavecraft-dsp`: contracts and composition only (`Processor`, `ProcessorParams`, `Transport`, `Chain`, `SignalChain`).
- `wavecraft-processors`: concrete processors and processor params.

### 2) Flat processor export surface
`wavecraft-processors/src/lib.rs` exposes top-level modules/exports for built-ins and related processors (e.g. gain, passthrough, saturator, unified_filter, oscillator, oscilloscope).

### 3) Re-export rewiring
- `wavecraft-core` prelude and macro internals point built-in mappings to `wavecraft_processors::*`.
- `wavecraft-nih_plug` prelude re-exports processors from `wavecraft-processors`.

### 4) DSP crate cleanup
`wavecraft-dsp` drops built-in implementation ownership and retains only minimal legacy module stubs/documentation where needed for migration clarity.

## Before/After (Ownership)
- **Before:** Built-in processor implementations were treated as part of `wavecraft-dsp` ownership.
- **After:** Built-in implementations are owned by `wavecraft-processors`; `wavecraft-dsp` owns only DSP traits/contracts/combinators.

## Compatibility Notes
- Public convenience surfaces remain available through core/nih_plug preludes.
- `wavecraft_processor!` built-in aliases still work, but map to `wavecraft_processors` types.

## Validation Strategy
- Compile and lint affected crates through workspace checks.
- Verify prelude and macro exports compile in downstream usage paths.
- Ensure removal of old implementation ownership in `wavecraft-dsp` does not regress tests.
