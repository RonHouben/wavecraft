# Test Plan — Processors Crate Consolidation

## Overview
- Feature: `processors-crate-consolidation`
- Date: 2026-02-21
- Scope: verify crate ownership shift, flat exports, prelude/macro rewiring, and no regressions in affected crates.

## Related Documents
- [Low-Level Design](./low-level-design-processors-crate-consolidation.md)
- [Implementation Plan](./implementation-plan.md)
- [Implementation Progress](./implementation-progress.md)
- [Coding Standards](../../architecture/coding-standards.md)
- [Testing & Quality Standards](../../architecture/coding-standards-testing.md)
- [Development Workflows](../../architecture/development-workflows.md)

## Test Cases

1. **Processor ownership**
   - Verify built-in processor implementations are exported from `wavecraft-processors` root.
   - Expected: built-ins available from `wavecraft_processors::*` without `builtins::*` namespace.

2. **DSP boundary cleanup**
   - Verify `wavecraft-dsp` public surface is contracts/combinators oriented.
   - Expected: built-in implementation ownership removed from `wavecraft-dsp`.

3. **Macro/prelude correctness**
   - Verify `wavecraft-core` prelude and `wavecraft_processor!` mappings point to `wavecraft_processors` built-ins.
   - Verify `wavecraft-nih_plug` prelude re-exports processor crate built-ins.

4. **Workspace validation**
   - Run full CI-style checks.
   - Expected: all checks pass.

## Executed Validation
- ✅ `cargo xtask ci-check --full` passed (exit code 0).

## Result
- PASS: 4
- FAIL: 0
- BLOCKED: 0

## Files Verified
- `engine/crates/wavecraft-processors/src/lib.rs`
- `engine/crates/wavecraft-dsp/src/lib.rs`
- `engine/crates/wavecraft-dsp/src/builtins/mod.rs`
- `engine/crates/wavecraft-core/src/prelude.rs`
- `engine/crates/wavecraft-core/src/macros.rs`
- `engine/crates/wavecraft-nih_plug/src/prelude.rs`

## Sign-off
Ready for QA/archival flow once remaining non-documentation process requirements are satisfied.
