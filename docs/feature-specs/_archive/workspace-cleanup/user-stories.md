# User Stories: Comprehensive Workspace Cleanup

## Overview

The current `cargo xtask clean` command only cleans `engine/target`, leaving build artifacts scattered across the monorepo. This feature extends cleanup to cover all build outputs, providing a single command to reclaim disk space from the complete workspace.

**User Impact:** Plugin developers need to free up disk space periodically, especially when switching between debug/release builds or troubleshooting build issues. A comprehensive cleanup command saves them from manually deleting multiple directories.

## Version

**Target Version:** `0.8.6` (patch bump from `0.8.5`)

**Rationale:** This is a polish/tooling improvement that enhances developer experience for an existing command. It doesn't introduce new features or change plugin behavior — just makes `cargo xtask clean` more thorough. Follows patch version guidelines per [Coding Standards](../../architecture/coding-standards.md#versioning).

---

## User Story 1: Clean All Rust Build Artifacts

**As a** plugin developer  
**I want** `cargo xtask clean` to remove all Rust build directories  
**So that** I can reclaim disk space from both engine and CLI builds with one command

### Acceptance Criteria
- [ ] `cargo xtask clean` removes `engine/target/`
- [ ] `cargo xtask clean` removes `cli/target/`
- [ ] Command reports both directories being cleaned
- [ ] Works even if directories don't exist (no errors)

### Notes
- Currently only cleans `engine/target`
- CLI builds accumulate in `cli/target/` during development

---

## User Story 2: Clean UI Build Artifacts

**As a** plugin developer  
**I want** `cargo xtask clean` to remove UI build outputs  
**So that** I can ensure fresh builds when troubleshooting bundling issues

### Acceptance Criteria
- [ ] `cargo xtask clean` removes `ui/dist/`
- [ ] `cargo xtask clean` removes `ui/coverage/` (test artifacts)
- [ ] Command reports UI directories being cleaned
- [ ] Works even if directories don't exist (no errors)

### Notes
- `ui/dist/` contains Vite build outputs
- `ui/coverage/` contains Vitest coverage reports
- `ui/node_modules/` intentionally **not** included in cleanup (npm manages this)

---

## User Story 3: Clean Temporary Test Artifacts

**As a** plugin developer  
**I want** `cargo xtask clean` to remove temporary test artifacts  
**So that** I reclaim space from CLI test scaffolding

### Acceptance Criteria
- [ ] `cargo xtask clean` removes `target/tmp/` (contains CLI test plugins)
- [ ] Command reports temp directory being cleaned
- [ ] Works even if directory doesn't exist (no error)

### Notes
- `target/tmp/` accumulates test plugins from CLI integration tests and manual testing
- Users may have forgotten test artifacts in this directory

---

## User Story 4: Clear Summary Output

**As a** plugin developer  
**I want** clear feedback about what was cleaned  
**So that** I know the command succeeded and understand what was removed

### Acceptance Criteria
- [ ] Command outputs "Cleaning workspace build artifacts..."
- [ ] Lists each directory cleaned with checkmark/status
- [ ] Reports total disk space reclaimed (if possible)
- [ ] Success message: "Workspace cleaned successfully"

### Example Output
```
Cleaning workspace build artifacts...
✓ engine/target (1.2 GB)
✓ cli/target (450 MB)
✓ ui/dist (2 MB)
✓ ui/coverage (5 MB)
✓ target/tmp (150 MB)

Workspace cleaned successfully (1.8 GB reclaimed)
```

---

## Out of Scope

- **npm dependencies**: `ui/node_modules/` is NOT included. Users should use `npm clean-install` for UI dependency refresh.
- **Plugin build outputs**: `engine/target/bundled/` is part of `engine/target`, already covered.
- **AU wrapper builds**: `packaging/macos/au-wrapper/build/` is rare and manually managed.
- **Git ignored files**: No `git clean -fdx` behavior — only known build directories.

---

## Success Criteria

- [ ] Single command cleans all Rust + UI + temp build artifacts
- [ ] No errors on missing directories (idempotent)
- [ ] Clear output showing what was cleaned
- [ ] Developer experience: "this just works"

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Build System & Tooling
- [Backlog](../../backlog.md) — Original task definition
