## Summary

Enhance `wavecraft update` to self-update the CLI binary before updating project dependencies. The command now works from any directory — if outside a plugin project, it only updates the CLI; inside a project, it does both.

This was the most critical missing piece from the original update command (M14): developers had to manually `cargo install wavecraft` to get the latest CLI, which most would forget to do.

## Changes

- **CLI Core** (`cli/src/commands/update.rs`): Restructured into two-phase command — Phase 1: CLI self-update via `cargo install wavecraft`, Phase 2: project dependency updates. Added `SelfUpdateResult`/`ProjectUpdateResult` enums, version detection (`parse_version_output`, `get_installed_version`), output parsing (`is_already_up_to_date`), summary logic (`determine_summary` with `SummaryOutcome`). 19 unit tests (+12 from QA findings).
- **CLI Help** (`cli/src/main.rs`): Updated `Commands::Update` doc comment and added `long_about` describing two-phase behavior and any-directory support.
- **CLI Version** (`cli/Cargo.toml`): Bumped `0.9.0` → `0.9.1`.
- **Test Fix** (`cli/src/template/mod.rs`): Fixed pre-existing `test_apply_local_dev_overrides` by adding missing `wavecraft-dev-server` dependency.
- **Integration Tests** (`cli/tests/update_command.rs`): Updated help text assertions, added `test_update_help_shows_any_directory_info`, marked network-dependent tests with `#[ignore]`.
- **Documentation**: Updated `high-level-design.md` (repo structure, distribution table), `sdk-getting-started.md` (rewrote "Updating" section), `roadmap.md` (changelog, M14 tasks, version).

## Commits

- `55ebe27` feat(cli): add self-update to wavecraft update command
- `78bc6d3` fix(cli): address all QA findings for CLI self-update
- `e66230b` docs: update architecture docs for CLI self-update feature
- `3f5e51a` chore(po): archive cli-self-update spec, update roadmap

## Related Documentation

- [User Stories](docs/feature-specs/_archive/cli-self-update/user-stories.md)
- [Low-Level Design](docs/feature-specs/_archive/cli-self-update/low-level-design-cli-self-update.md)
- [Implementation Plan](docs/feature-specs/_archive/cli-self-update/implementation-plan.md)
- [Test Plan](docs/feature-specs/_archive/cli-self-update/test-plan.md)
- [QA Report](docs/feature-specs/_archive/cli-self-update/QA-report.md)

## Testing

- [x] `cargo xtask ci-check` passes (lint + tests)
- [x] 19 CLI unit tests passing (7 original + 12 from QA)
- [x] 2 active integration tests passing (4 network-dependent `#[ignore]`d)
- [x] Manual testing: 12/12 test cases (version, help, outside project, inside project, error handling)
- [x] QA approved: 0 Critical/High/Medium findings

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated (19 unit + integration tests)
- [x] Documentation updated (architecture, getting started, roadmap)
- [x] No linting errors
- [x] Feature spec archived
- [x] Roadmap updated
