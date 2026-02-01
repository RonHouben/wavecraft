## Summary

Clean up dead code suppressions in the VstKit editor modules by applying proper platform-gating patterns. This ensures code that only runs on macOS/Windows doesn't trigger dead code warnings on Linux CI.

**Key Achievement:** Reduced `#[allow(dead_code)]` suppressions from 14 to 3 (79% reduction). The remaining 3 are valid cases where trait methods are called by platform-specific implementations.

## Changes

- **Engine/DSP**:
  - Platform-gate editor-related imports, structs, and functions with `#[cfg(any(target_os = "macos", target_os = "windows"))]`
  - Clean up `MeterConsumer` import gating in `lib.rs`
  - Refine `WebViewHandle` trait method suppressions
  - Files: `plugin/src/lib.rs`, `plugin/src/editor/*.rs`, `desktop/src/assets.rs`

- **Documentation**:
  - Add "Platform-Specific Code" section to `coding-standards.md`
  - Update QA agent handoff rules for PASS case
  - Update roadmap: mark dead code cleanup complete, add changelog entry
  - Full feature spec: user stories, low-level design, implementation plan, test plan, QA report

## Commits

```
17733d5 docs: mark M5 dead code cleanup complete, archive spec
3f8fab2 docs(arch): add Platform-Specific Code section to coding standards
d358b11 docs(m5): add QA report for dead code cleanup implementation
4ae15fc fix(m5): remove unnecessary dead code suppressions and refine platform-specific gating
d0a9f60 test(m5): update test plan - CI passed with final fix
0c41ca8 fix(m5): allow dead code for platform-specific items
b5b4fdc style(m5): fix import ordering per rustfmt
ded4600 fix(m5): gate MeterConsumer import to macOS/Windows
5a64002 fix(m5): keep Any import unconditional for WebViewHandle trait
a239628 fix(m5): gate all editor-related imports to macOS/Windows
bd34296 fix(m5): gate editor implementation to macOS/Windows only
ffc2837 fix(m5): add test cfg to platform-gated code
c8142ce docs(m5): update test plan with fix verification results
6390236 fix(m5): add platform cfg to resolve Linux CI dead code warnings
eb5b18d feat(m5): complete dead code cleanup (14→3 suppressions)
e5f16b8 docs(plan): add implementation plan and progress tracker
ee70e32 docs(arch): add low-level design for dead code cleanup
f095899 feat(docs): add user stories for M5 dead code cleanup
```

## Related Documentation

- [User Stories](./user-stories.md)
- [Low-Level Design](./low-level-design-dead-code-cleanup.md)
- [Implementation Plan](./implementation-plan.md)
- [Implementation Progress](./implementation-progress.md)
- [Test Plan](./test-plan.md)
- [QA Report](./QA-report.md)

## Testing

- [x] Build passes: `cargo xtask build`
- [x] Linting passes: `cargo xtask lint`
- [x] Tests pass: `cargo xtask test` (99 engine + 35 UI tests)
- [x] CI pipeline passes (all 6 jobs)
- [x] Manual testing completed per test plan
- [x] QA review approved

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated (coding-standards.md, roadmap.md)
- [x] No linting errors (`cargo xtask lint`)
- [x] Feature spec archived to `_archive/`
