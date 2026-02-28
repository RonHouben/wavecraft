## Summary

This PR delivers processor bypass support end-to-end across the Wavecraft stack, centered on SignalChain/DSP integration and UI processor controls. It also includes related processor UX and hook updates, dev-server audio modifier improvements, and supporting CLI/docs updates required by the feature branch.

## Changes

- **Engine/DSP**
  - Added bypass support in SignalChain-related DSP combinators and parameter extraction paths.
  - Updated macro/plugin metadata/runtime parameter plumbing for processor/bypass-aware generation.
  - Adjusted bridge/loader and processor crate integration points.

- **UI**
  - Added/updated processor components and tests (including processor-specific render/control paths).
  - Introduced/updated bypass-related core hooks/utilities and removed deprecated parameter-group hook usage.
  - Updated SDK template UI processor rendering and generated processor metadata.

- **Dev Server / CLI / Build**
  - Extended dev-server output modifier and startup/audio wiring behavior to support new processor paths.
  - Improved CLI bundle/start behavior and project root resolution/error handling.
  - Refreshed embedded UI dist assets and related bundling outputs.

- **Documentation**
  - Added processor-bypass feature-spec artifacts (user stories, low-level design, plan, progress, QA report, test plan, UX design).
  - Added dev-ffi-parameter-injection design/plan updates.
  - Updated architecture workflow/development docs and introduced UX-related skills/docs.

## Commits

- `dfe6e51` fix: Ensure proper handling of unknown processor parameter types in Processor component
- `c35feee` Refactor code structure for improved readability and maintainability
- `28e8a1b` Merge branch 'main' into feature/processor-bypass
- `e520f04` feat: Add implementation plan for Dev FFI Parameter Injection v2 and DSP unification
- `1b0373a` feat: Add low-level design documentation for Dev FFI Parameter Injection v2 and DSP unification
- `3a8f860` feat: Implement soft clip functionality with drive and output trim controls
- `72094be` feat: Add tone filter functionality with state management to output modifiers
- `1025256` feat: Implement input trim bypass functionality and enhance output modifiers
- `3c0e117` feat: Enhance project root resolution with SDK hint and improve error handling
- `60a274b` feat: Update processor components to support input trim and bypass functionality
- `67b7062` feat: Refactor processor components to use ProcessorId and remove unused utility functions
- `d5d9dc2` feat: Add Oscillator and Oscilloscope processors with tests
- `6f0fbdd` feat: update asset references in index.html and enhance bundle command error handling
- `d8db66e` feat: add bypass support to SignalChain macro and related components
- `1e163b3` feat: add implementation plan for processor bypass feature in SignalChain pipeline
- `f605919` Add design token compliance, accessibility review, and UI/UX change workflow skills

## Related Documentation

- [Implementation Plan](./implementation-plan.md)
- [Implementation Progress](./implementation-progress.md)
- [Low-Level Design](./low-level-design-processor-bypass.md)
- [User Stories](./user-stories.md)
- [Test Plan](./test-plan.md)
- [QA Report](./QA-report.md)
- [UX Design](./ux-design-processor-bypass.md)

## Testing

- [ ] Build passes: `cargo xtask build`
- [ ] Linting passes: `cargo xtask lint`
- [ ] Tests pass: `cargo xtask test`
- [ ] Manual UI verification
- [ ] Audio processing verification (including processor bypass behavior)

## Checklist

- [ ] Code follows project coding standards
- [ ] Tests added/updated as needed
- [ ] Documentation updated
- [ ] No linting errors (`cargo xtask lint`)
