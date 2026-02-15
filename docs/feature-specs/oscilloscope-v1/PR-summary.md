## Summary

This PR delivers **Oscilloscope v1** end-to-end and includes the supporting **processor-presence/type generation flow** needed to gate UI rendering by available processors.

It adds oscilloscope data production/consumption in Rust, IPC contracts, dev-server and editor bridge integration, React hooks/components for rendering, and CLI/template generation updates so generated apps get typed processor metadata and stable development behavior.

## Changes

### Engine / DSP / Protocol

- Added oscilloscope processor implementation and exports in `wavecraft-processors`
- Extended protocol/IPC contracts for oscilloscope frame retrieval and related types
- Updated bridge/host/plugin loader paths for processor metadata and new IPC behavior
- Integrated editor/webview bridge updates in `wavecraft-nih_plug`
- Updated xtask dev flow and preflight behavior for generated parameter/processor typing

### CLI / Template / Dev Server

- Added processor extraction command path and TypeScript codegen improvements
- Updated startup/preflight checks to preserve generated typing in verbose/dev flows
- Updated template signal chain and example processor behavior
- Updated dev-server rebuild/reload behavior and host/audio integration points

### UI Packages

- Added `Oscilloscope` component and tests in `@wavecraft/components`
- Added `useOscilloscopeFrame`, `useHasProcessor`, and `useAvailableProcessors` hooks and tests in `@wavecraft/core`
- Added processor registry/types and IPC helpers for oscilloscope + processor presence
- Updated typecheck and mock infrastructure to support generated processor IDs

### Documentation

- Added/archived oscilloscope and processor-presence feature docs
- Updated architecture docs to reflect codegen/processor metadata and workflow changes

## Commits

- `e18dfad` feat(oscilloscope): add implementation progress and low-level design documentation
- `2c39547` feat: update signal chain to include OutputGain and remove InputGain processor
- `bbdf3f5` feat: update preflight checks to retain generated parameter and processor types during verbose mode
- `c8a9753` refactor: update .gitignore and remove generated processors file; adjust tsconfig includes
- `3b346e6` feat: update processor ID augmentation and type definitions across SDK and UI components
- `c8f3bb3` feat: add development-only type loader for generated processor ID augmentation in core and components
- `1abad47` feat: enhance processor handling and type checks in useHasProcessor hook
- `c3d16e3` refactor(oscilloscope): move signal chain visibility check to the end of the render function
- `1f3d1a3` feat: implement processor presence hook with metadata handling
- `ed6e31b` feat(example_processor): implement fixed gain processing and update documentation
- `fe9bf20` style: format signal chain for better readability in plugin definition
- `5476f93` feat(oscilloscope): implement history management for trigger alignment and update component visibility
- `dfb47ee` fix(oscilloscope): improve trigger alignment logic to prevent frame tail wrapping
- `b26562e` feat: enhance oscilloscope integration and update test plan documentation
- `1c7c8df` feat: add oscilloscope functionality to wavecraft
- `69e7f43` feat(oscilloscope): add implementation and low-level design documents for oscilloscope v1

## Related Documentation

- `docs/feature-specs/_archive/oscilloscope/implementation-plan.md`
- `docs/feature-specs/_archive/oscilloscope/implementation-progress.md`
- `docs/feature-specs/_archive/oscilloscope/low-level-design-oscilloscope.md`
- `docs/feature-specs/_archive/oscilloscope/test-plan.md`
- `docs/feature-specs/_archive/processor-presence-hook/implementation-plan.md`
- `docs/feature-specs/_archive/processor-presence-hook/implementation-progress.md`
- `docs/feature-specs/_archive/processor-presence-hook/test-plan.md`

## Testing

Executed prior to PR creation on this branch (see repository history / prior local runs):

- Rust engine and bridge tests
- UI package tests for oscilloscope and processor-presence hooks
- Typecheck coverage for generated processor typing paths

## Checklist

- [x] Feature implemented across engine, protocol, bridge, CLI, and UI
- [x] Tests added/updated for new oscilloscope + processor presence behavior
- [x] Documentation updated and archived for completed feature tracks
- [x] PR body generated from branch commits and merge-base diff
