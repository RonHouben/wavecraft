## Summary

Add a `processors/` module to the CLI template (`wavecraft create`) with a sine-wave oscillator example, demonstrating how to implement custom DSP processors with the Wavecraft SDK. Bump version to 0.11.0.

This gives new users a working, educational example of custom audio processing from day one — including parameter definitions via `#[derive(ProcessorParams)]`, real-time safe DSP, and signal chain composition.

## Changes

- **CLI Template** (`cli/sdk-templates/`):
  - New `processors/oscillator.rs` — sine-wave generator with frequency/level params, stereo-safe phase handling, division-by-zero guard
  - New `processors/mod.rs` — module exports with 4-step instructional comments for adding new processors
  - Updated `lib.rs` — shows both gain-only (default) and oscillator-enabled signal chains
  - Updated `README.md` — corrected code examples, Processor trait signature, signal chain patterns

- **Engine/SDK** (`engine/crates/`):
  - `wavecraft-macros`: ProcessorParams derive macro now generates `::wavecraft::` paths (works in user projects with Cargo rename)
  - `wavecraft-nih_plug`: Added `ParamSpec` and `ProcessorParams` derive macro re-exports
  - All crate versions bumped to 0.11.0

- **Documentation** (`docs/`):
  - `sdk-getting-started.md` — updated with oscillator example, correct imports, DSP patterns
  - `coding-standards.md` — clarified `wavecraft_processor!` is built-in only, added custom processor guidance
  - `high-level-design.md` — updated macro docs and user project structure

## Commits

- `68c205b` feat: update all dependencies to version 0.11.0 and add custom oscillator processor example
- `799e5b8` docs: add implementation progress for template-processors-module
- `67aebb9` fix: correct documentation examples for custom processors
- `2b08e28` fix: remove wavecraft_processor wrapping in getting-started guide step 4
- `3682315` fix: update test plan for template processors module to reflect final re-test results
- `b67a9d3` fix: address QA findings for template processors module
- `647f866` docs: update QA report with re-review pass status
- `c0dbdfb` docs: update architecture docs for template-processors-module feature

## Related Documentation

- [User Stories](./user-stories.md)
- [Implementation Progress](./implementation-progress.md)
- [Test Plan](./test-plan.md)
- [QA Report](./QA-report.md)

## Testing

- [x] CI passes: `cargo xtask ci-check` (148 engine + 28 UI tests, 11.8s)
- [x] Linting passes: ESLint, Prettier, cargo fmt, clippy
- [x] Generated project compiles (both gain-only and oscillator chains)
- [x] No unreplaced template variables in generated output
- [x] Tester sign-off: 12/12 test cases pass
- [x] QA sign-off: All 5 findings resolved, re-review PASS

## Checklist

- [x] Code follows project coding standards
- [x] Real-time safety verified (no allocations/locks in process())
- [x] Tests added/updated as needed
- [x] Documentation updated (coding standards, HLD, getting started)
- [x] No linting errors
- [x] Version bumped to 0.11.0
