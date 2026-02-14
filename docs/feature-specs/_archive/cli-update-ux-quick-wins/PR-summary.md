## Summary

Implements the **CLI Update UX Quick Wins** initiative (Pre-M19), delivering all 3 items:

1. **Re-exec flow** — After `wavecraft update` updates the CLI binary, it automatically re-execs itself so the new version handles dependency updates. Eliminates the previous two-step "update CLI, then run again" workflow.
2. **Streaming progress** — Installation progress messages are streamed in real-time during `cargo install`, replacing the previous silent wait. Users see what cargo is compiling/downloading as it happens.
3. **Dev build profile** — Adds `[profile.dev]` and `[profile.dev.package."*"]` sections to the workspace template, compiling dependencies with `opt-level = 2` while keeping plugin code at `opt-level = 0` for faster incremental builds during development.

## Changes

- **CLI** (`cli/`):
  - `cli/src/commands/update.rs` — Rewritten update flow: split progress streaming, re-exec after self-update, improved error handling (+295/-varied lines)
  - `cli/src/main.rs` — Added re-exec support and updated argument handling
  - `cli/Cargo.toml` / `cli/Cargo.lock` — Added `self_update` dependency
  - `cli/src/commands/start.rs` — Minor cleanup
  - `cli/src/template/mod.rs` — Minor cleanup
  - `cli/tests/update_command.rs` — Removed ignored integration tests, cleaned up test code

- **Build/Config**:
  - `sdk-template/Cargo.toml.template` — Added `[profile.dev]` (opt-level=0, incremental=true) and `[profile.dev.package."*"]` (opt-level=2)
  - `.github/workflows/continuous-deploy.yml` — CD pipeline updates for CLI publishing
  - `engine/xtask/src/commands/validate_cli_deps.rs` — Removed obsolete validation command

- **Documentation**:
  - `docs/roadmap.md` — Updated Pre-M19 status to "all 3 items shipped"
  - `docs/backlog.md` — Removed completed items, added changelog entry
  - `docs/guides/sdk-getting-started.md` — Updated to reflect new streaming progress messaging
  - `docs/feature-specs/_archive/cli-update-ux-quick-wins/` — Archived feature spec (LLD, implementation plan, test plan)

## Commits

- `feac055` feat(cli): implement split progress messaging and automatic CLI re-execution after update
- `f99d4a0` feat: enhance `wavecraft update` command with improved UX
- `be73ec0` fix(cli): improve progress messaging during installation phases
- `c42f125` fix(template): replace push_str with push for newline character
- `da6affc` refactor(tests): remove ignored update tests and clean up code
- `24a3588` refactor(tests): remove ignored integration test for registry availability
- `adb3624` feat(tests): add comprehensive test plan for CLI update UX quick wins
- `d05491e` fix(docs): update CLI update guide to reflect new streaming progress messaging
- `cabf0dc` fix(docs): update roadmap to reflect completion of CLI update UX quick wins and dev build profile spike
- `33c27fe` feat(tests): add test plan for CLI update UX quick wins
- `69e70c9` feat(config): add development profile settings to Cargo.toml.template
- `7d11223` fix(docs): update backlog and roadmap to reflect completion of CLI update UX quick wins and dev build profile optimization

## Related Documentation

- [Low-Level Design](../_archive/cli-update-ux-quick-wins/low-level-design-cli-update-ux-quick-wins.md)
- [Implementation Plan](../_archive/cli-update-ux-quick-wins/implementation-plan.md)
- [Test Plan](../_archive/cli-update-ux-quick-wins/test-plan.md)

## Testing

- [x] `cargo xtask ci-check --full` passes (all 6 phases)
- [x] `cargo xtask validate-template` passes (6 steps, 96.9s)
- [x] QA approved (0 Critical/High/Medium findings, 2 Low)
- [x] Manual testing: `wavecraft update` re-exec flow verified
- [x] Manual testing: streaming progress output verified

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No linting errors
- [x] Feature spec archived
- [x] Roadmap updated
- [x] Backlog cleaned up
