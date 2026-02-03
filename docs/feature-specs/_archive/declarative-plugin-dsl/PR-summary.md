## Summary

Implement the **Declarative Plugin DSL** for Wavecraft (Milestone 10), delivering a dramatic **95% reduction in plugin boilerplate** — from 190 lines to just 9 lines.

This feature introduces three key macros:
- **`wavecraft_plugin!`** — Zero-boilerplate plugin declaration
- **`#[derive(ProcessorParams)]`** — Automatic parameter metadata with `#[param(...)]` attributes
- **`wavecraft_processor!`** — Named processor wrappers for signal chains

Plus UI support via `ParameterGroup` component and `useParameterGroups` hook for organized parameter display.

**Version:** 0.6.0 (minor — new public API, significant DX improvement)

## Changes

### Engine/DSP
- **wavecraft-macros crate** — New proc-macro crate for DSL macros
  - `ProcessorParams` derive macro with `#[param(range, default, unit, factor, group)]`
  - `wavecraft_plugin!` proc-macro for complete plugin generation
- **wavecraft-dsp** — Built-in processors (Gain, Passthrough) with ProcessorParams
- **wavecraft-core** — Updated prelude exports, branding fix (VstKit → Wavecraft)
- **wavecraft-protocol** — Added `group` field to parameter IPC messages

### UI (React/TypeScript)
- **ParameterGroup component** — Collapsible groups for organized parameter display
- **useParameterGroups hook** — Groups parameters by their `group` metadata
- **Branding update** — Header/footer changed from VstKit to Wavecraft
- **IPC hooks fix** — Removed browser env checks that prevented WebSocket IPC

### Template
- **wavecraft-plugin-template** — Updated with DSL usage (9 lines vs 190)

### Documentation
- **Architecture docs** — Added DSL section to high-level-design.md
- **Coding standards** — Added macro usage guidelines
- **QA report** — Comprehensive quality review with all issues resolved
- **Test plan** — 18 manual test cases, all passing

## Commits

```
377d337 docs: complete Milestone 10 - Declarative Plugin DSL
838bbd2 docs: Update architectural documentation for Declarative Plugin DSL
c77287f docs: Add comprehensive QA report for declarative plugin DSL
f40acd3 fix: Add missing group field support to ProcessorParams derive macro
49e670a fix: Update branding from VstKit to Wavecraft (Rust + UI)
17c2c05 test: Complete manual testing - all 18 tests PASS
ed75bbf fix: Remove browser env checks from React hooks to enable WebSocket IPC
20aa876 feat: complete Phase 9 and all coder implementation
365979f feat: Phase 9 steps 2-4 - UI parameter grouping (85% total)
60fd45a feat: Phase 9 step 1 - add group metadata to IPC protocol (77.5% total)
8f8d9bb feat: complete Phase 7 - template integration and DAW testing (75% total)
196aa1a feat: Phase 7 steps 1-3 - DSL template integration (72.5% total)
56e334b feat: complete Phase 6 - wavecraft_plugin macro (65% total)
e5768b5 feat: complete Phase 6 core structure (60% total progress)
11221bf wip: begin Phase 6 - wavecraft_plugin! proc-macro (step 6.1)
e764cc7 feat: implement declarative DSL phases 1-5 (50% complete)
787d505 docs: update roadmap for Declarative Plugin DSL milestone
d860e6a refactor: update project references from VstKit to Wavecraft
01a047d docs: add user stories for Declarative Plugin DSL feature
378763b Add implementation progress and low-level design documents
```

## Related Documentation

- [User Stories](./user-stories.md)
- [Low-Level Design](./low-level-design-declarative-plugin-dsl.md)
- [Implementation Plan](./implementation-plan.md)
- [Implementation Progress](./implementation-progress.md)
- [Test Plan](./test-plan.md)
- [QA Report](./QA-report.md)

## Testing

- [x] Build passes: `cargo xtask bundle --release`
- [x] Linting passes: `cargo xtask lint`
- [x] Engine tests pass: 28 tests (cargo test --workspace)
- [x] UI tests pass: 35 tests (npm test)
- [x] Manual DAW verification: Plugin loads in Ableton Live
- [x] Parameter sync verified: UI ↔ Engine round-trip
- [x] Code signing verified: Ad-hoc signing for local testing

## Test Results

```
Engine Tests: 28 passed, 0 failed
UI Tests:     35 passed, 0 failed
Manual Tests: 18/18 passed
Linting:      All checks passed (cargo fmt, clippy, ESLint, Prettier)
```

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated (architecture, coding standards)
- [x] No linting errors
- [x] QA review completed and approved
- [x] Feature spec archived
- [x] Roadmap updated
