## Summary

This PR adds a `wavecraft bundle --install` workflow and validates template install behavior for newly created projects.

The implementation wires the CLI bundle command into project-context validation and delegation to `xtask`, extends template validation flows, and adds focused tests and docs updates for the new install path.

## Changes

### CLI / Command Surface

- Added `bundle` command implementation in `cli/src/commands/bundle.rs`
- Updated command registration and entry wiring in:
  - `cli/src/commands/mod.rs`
  - `cli/src/main.rs`
  - `cli/src/commands/create.rs`

### Tests

- Added `cli/tests/bundle_command.rs` covering:
  - project context validation
  - `--install` flag requirements and delegation behavior

### Template / Build Validation

- Updated template validation workflow and command paths:
  - `.github/workflows/template-validation.yml`
  - `engine/xtask/src/commands/validate_template.rs`
  - `sdk-template/engine/xtask/src/main.rs`

### Documentation

- Updated architecture and guide docs:
  - `docs/architecture/development-workflows.md`
  - `docs/architecture/plugin-formats.md`
  - `docs/guides/sdk-getting-started.md`

- Included feature lifecycle/closure documentation updates:
  - `docs/roadmap.md`
  - archived feature-spec artifacts under `docs/feature-specs/_archive/new-project-vst3-build-install/`

## Commits

- `803b5f5` feat: complete implementation of new-project VST3 build and install feature
- `1465b12` feat(install): enhance `xtask bundle` command with `--install` support and validation
- `81bd197` feat(bundle): implement `wavecraft bundle --install` command with project context validation and delegation to xtask

Additional commit messages in range:

- docs: add implementation plan and progress documentation for new project VST3 build and install feature
- test: add tests for bundle command, including context validation and install flag requirements

## Diff Overview

- 17 files changed
- 1672 insertions, 84 deletions

## Validation Notes

Changes include command implementation, test coverage, template validation updates, and documentation alignment for the install-enabled bundle workflow.
