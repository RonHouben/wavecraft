# Implementation Plan: CI-Prefixed xtask Subcommands

## Overview
Rename **only** the `cargo xtask` subcommands that mirror GitHub Actions workflows to start with the `ci-` prefix (e.g., `ci-check`, `ci-validate-template`) while keeping backward-compatible aliases for the old names. All non-CI xtasks (e.g., `dev`, `bundle`, `install`, `sign`) remain unchanged. Update workflows, docs, and templates to use the new `ci-*` names, and ensure the CLI provides deprecation warnings when old aliases are used.

## Requirements
- Only xtask subcommands that **mirror GitHub Actions workflows** are renamed to `ci-*` equivalents.
- Old subcommand names continue to work via aliases with clear deprecation messaging.
- Non-CI xtasks remain unchanged.
- GitHub Actions workflows and scripts are updated to use new `ci-*` subcommands.
- Documentation and skill guides reflect the new command names.
- Template project xtask (if applicable) is aligned with the new naming to avoid drift.

## Architecture Changes
- **`engine/xtask/src/main.rs`**: Update clap subcommand names for workflow-mirroring commands only, add aliases, and add deprecation warnings.
- **`engine/xtask/src/commands/mod.rs`** (and other command modules): Update help text/strings referencing renamed command names.
- **Docs & workflows**: Replace references to old `cargo xtask <command>` names for CI-mirroring commands only.
- **CLI template xtask**: Update template `engine/xtask/src/main.rs` in `cli/sdk-templates/...` if it is intended to mirror the main xtask command naming.

## Implementation Steps

### Phase 1: Discovery & Mapping
1. **Inventory workflow-mirroring xtask subcommands** (File: `engine/xtask/src/main.rs` + `.github/workflows/*.yml`)
   - Action: Identify only those subcommands that directly mirror workflow steps and define the `ci-*` mapping (e.g., `check → ci-check`, `validate-template → ci-validate-template`).
   - Why: Ensure only CI-mirroring commands are renamed; keep non-CI commands unchanged.
   - Dependencies: None.
   - Risk: Low.

2. **Find all references to `cargo xtask` in repo** (Files: `.github/workflows/*.yml`, `README.md`, `CONTRIBUTING.md`, `docs/**/*.md`, `scripts/*.sh`, `.github/skills/**/*.md`, `cli/sdk-templates/**`)
   - Action: Compile a list of files needing updates.
   - Why: Prevent stale docs/workflows and keep instructions consistent.
   - Dependencies: Step 1.
   - Risk: Low.

### Phase 2: CLI Renaming + Aliases
3. **Rename only CI-mirroring clap subcommands to `ci-*`** (File: `engine/xtask/src/main.rs`)
   - Action: Change subcommand names to `ci-*` using clap `#[command(name = "ci-...")]` or equivalent; ensure `cargo xtask ci-check`, `cargo xtask ci-validate-template`, etc.
   - Why: Enforce the new naming convention for workflow-mirroring commands only.
   - Dependencies: Step 1.
   - Risk: Medium (missed CI-mirroring commands or inconsistent naming).

4. **Add backward-compatible aliases with warnings** (File: `engine/xtask/src/main.rs`)
   - Action: Add clap aliases for the old names, and detect alias usage to log a deprecation warning (e.g., “`bundle` is deprecated; use `ci-bundle`”).
   - Why: Preserve compatibility while guiding users to the new commands.
   - Dependencies: Step 3.
   - Risk: Medium (alias detection can be tricky in clap if not handled cleanly).

5. **Update internal help strings and dry-run messages** (File: `engine/xtask/src/commands/mod.rs` + any command modules)
   - Action: Replace hard-coded command names in output strings to `ci-*` versions for renamed commands only.
   - Why: Ensure user-facing output matches the new command names.
   - Dependencies: Step 3.
   - Risk: Low.

### Phase 3: Templates, Workflows, Docs
6. **Update GitHub Actions workflows** (Files: `.github/workflows/*.yml`)
   - Action: Replace `cargo xtask <old>` with `cargo xtask ci-<old>` in CI steps (e.g., release, template-validation, CI workflows).
   - Why: CI must reflect new command names.
   - Dependencies: Step 3.
   - Risk: Medium (missed workflow files or conditional steps).

7. **Update docs and guides** (Files: `README.md`, `CONTRIBUTING.md`, `docs/architecture/coding-standards.md`, `docs/guides/*.md`)
   - Action: Replace examples and references to old commands with `ci-*` for CI-mirroring commands only.
   - Why: Keep developer guidance accurate.
   - Dependencies: Step 3.
   - Risk: Low.

8. **Update skill guides and scripts** (Files: `.github/skills/**/SKILL.md`, `scripts/*.sh`)
   - Action: Align skill references and any scripted xtask calls with `ci-*` for CI-mirroring commands only.
   - Why: Skills and scripts must remain correct for local automation.
   - Dependencies: Step 3.
   - Risk: Low.

9. **Update CLI template xtask (if intended to match naming)** (File: `cli/sdk-templates/new-project/react/engine/xtask/src/main.rs`)
   - Action: Rename only CI-mirroring template subcommands to `ci-*` with alias warnings, or document if template is intentionally different.
   - Why: Avoid template drift and confusion for generated projects.
   - Dependencies: Step 3.
   - Risk: Medium (template compatibility expectations).

### Phase 4: Verification
10. **Run local checks with new commands**
    - Action: Validate `cargo xtask ci-check`, `cargo xtask ci-validate-template`, and any other key `ci-*` commands.
    - Why: Confirm wiring and CLI behavior.
    - Dependencies: Steps 3–9.
    - Risk: Medium (command mapping issues).

11. **Verify alias warnings**
    - Action: Run legacy commands (e.g., `cargo xtask check`) and confirm deprecation warnings are printed while behavior matches.
    - Why: Ensure backward compatibility and user guidance.
    - Dependencies: Step 4.
    - Risk: Low.

## Testing Strategy
- **Unit tests**: If clap parsing or command mapping is testable, add/adjust tests in `engine/xtask` to ensure aliases resolve correctly.
- **Integration checks**: Run key `ci-*` commands (e.g., `ci-check`, `ci-validate-template`) locally.
- **Workflow verification**: Optionally validate `.github/workflows` changes via `act` if needed for YAML correctness.

## Risks & Mitigations
- **Risk**: Missing references in docs/workflows/scripts.
   - Mitigation: Use a repo-wide search for `cargo xtask` and verify all hits are updated, but only adjust CI-mirroring commands.
- **Risk**: Clap alias handling doesn’t emit warnings reliably.
  - Mitigation: Implement explicit detection of legacy subcommand strings before clap parse (e.g., inspect `std::env::args()`), then continue parsing normally.
- **Risk**: Template xtask diverges from main xtask naming.
  - Mitigation: Decide explicitly whether the template should follow `ci-*`; document if it stays as-is.

## Success Criteria
- [ ] All xtask subcommands are accessible via `ci-*` names.
- [ ] Old subcommand names still work and emit a deprecation warning.
- [ ] GitHub Actions workflows use `ci-*` commands.
- [ ] Documentation and skill guides reference only `ci-*` names.
- [ ] Template xtask behavior is aligned or explicitly documented.
