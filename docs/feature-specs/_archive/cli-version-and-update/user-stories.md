# User Stories: CLI Version and Update Command

**Feature:** CLI Enhancements (Milestone 14)  
**Target Version:** `0.8.1` (patch — CLI improvements, no breaking changes)  
**Created:** 2026-02-08

---

## Overview

This feature adds two quality-of-life improvements to the Wavecraft CLI:
1. **Version flags** (`-v` / `--version`) — Standard way to check CLI version
2. **Update command** (`wavecraft update`) — One command to update all dependencies

These features improve developer experience before user testing (Milestone 15).

---

## User Stories

### Story 1: Check CLI Version Quickly

**As a** plugin developer  
**I want to** check my installed Wavecraft CLI version with a simple flag  
**So that** I can verify I'm using the correct version for bug reports and documentation

**Acceptance Criteria:**
- `wavecraft --version` displays the CLI version
- `wavecraft -v` displays the CLI version (short form)
- Output format is: `wavecraft X.Y.Z` (no extra text)
- Both flags work from any directory
- Exit code is 0 (success)

**Priority:** High  
**Estimated Effort:** 1-2 hours

**Example:**
```bash
$ wavecraft --version
wavecraft 0.8.1

$ wavecraft -v
wavecraft 0.8.1
```

---

### Story 2: Update All Dependencies with One Command

**As a** plugin developer  
**I want to** update both Rust crates and npm packages with a single command  
**So that** I don't have to manually run `cargo update` and `npm update` separately

**Acceptance Criteria:**
- `wavecraft update` command exists
- Detects and updates Rust dependencies if `engine/Cargo.toml` exists
- Detects and updates npm dependencies if `ui/package.json` exists
- Shows progress for each component ("Updating Rust dependencies...")
- Shows success/failure for each component ("✅ Rust dependencies updated")
- Reports combined results at the end
- Exit code 0 if all succeed, 1 if any fail

**Priority:** High  
**Estimated Effort:** 6-8 hours

**Example (success):**
```bash
$ wavecraft update
📦 Updating Rust dependencies...
✅ Rust dependencies updated
📦 Updating npm dependencies...
✅ npm dependencies updated

✨ All dependencies updated successfully
```

**Example (partial failure):**
```bash
$ wavecraft update
📦 Updating Rust dependencies...
❌ Rust update failed: Failed to run 'cargo update'
📦 Updating npm dependencies...
✅ npm dependencies updated

Error: Failed to update some dependencies:
  Rust: Failed to run 'cargo update'
```

---

### Story 3: Clear Error When Not in Plugin Project

**As a** plugin developer  
**I want to** see a helpful error message if I run `wavecraft update` outside a plugin project  
**So that** I understand what went wrong and how to fix it

**Acceptance Criteria:**
- Running `wavecraft update` in a non-plugin directory shows clear error
- Error message explains what's expected ("Expected to find 'engine/Cargo.toml' or 'ui/package.json'")
- Error message explains where to run the command ("Run this command from the root of a Wavecraft plugin project")
- Exit code is 1 (failure)

**Priority:** High  
**Estimated Effort:** Included in Story 2

**Example:**
```bash
$ cd /tmp
$ wavecraft update
Not a Wavecraft plugin project.
Expected to find 'engine/Cargo.toml' or 'ui/package.json'.
Run this command from the root of a Wavecraft plugin project.
```

---

### Story 4: Update Works with Engine Only

**As a** plugin developer building a headless plugin (no UI)  
**I want to** update Rust dependencies even if there's no `ui/` directory  
**So that** I can maintain my project without being forced to have a UI component

**Acceptance Criteria:**
- `wavecraft update` works in projects with only `engine/` directory
- Only attempts to update Rust dependencies
- Doesn't show npm-related output
- Exit code is 0 if Rust update succeeds

**Priority:** Medium  
**Estimated Effort:** Included in Story 2

**Example:**
```bash
$ wavecraft update
📦 Updating Rust dependencies...
✅ Rust dependencies updated

✨ All dependencies updated successfully
```

---

### Story 5: Update Works with UI Only

**As a** developer working on a pure UI library  
**I want to** update npm dependencies even if there's no `engine/` directory  
**So that** I can maintain my UI components independently

**Acceptance Criteria:**
- `wavecraft update` works in projects with only `ui/` directory
- Only attempts to update npm dependencies
- Doesn't show Rust-related output
- Exit code is 0 if npm update succeeds

**Priority:** Medium  
**Estimated Effort:** Included in Story 2

**Example:**
```bash
$ wavecraft update
📦 Updating npm dependencies...
✅ npm dependencies updated

✨ All dependencies updated successfully
```

---

### Story 6: See Update Command in Help Text

**As a** plugin developer  
**I want to** see the update command listed in `wavecraft --help`  
**So that** I can discover the feature without reading external documentation

**Acceptance Criteria:**
- `wavecraft --help` lists the "update" command
- `wavecraft update --help` shows specific help for the update command
- Help text clearly explains what the command does

**Priority:** Medium  
**Estimated Effort:** Automatic (clap generates help text)

**Example:**
```bash
$ wavecraft --help
Wavecraft audio plugin development toolkit

Usage: wavecraft [OPTIONS] [COMMAND]

Commands:
  create  Create a new Wavecraft plugin project
  start   Start development servers for browser-based UI development
  update  Update all project dependencies (Rust crates + npm packages)
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

---

### Story 7: Clear Error When Tools Are Missing

**As a** plugin developer who might not have cargo/npm installed  
**I want to** see a clear error message if the required tool isn't found  
**So that** I know what to install

**Acceptance Criteria:**
- If `cargo` is not in PATH, error message mentions this
- If `npm` is not in PATH, error message mentions this
- Error includes context about what was being attempted
- Exit code is 1 (failure)

**Priority:** Medium  
**Estimated Effort:** Included in Story 2

**Example:**
```bash
$ wavecraft update
📦 Updating Rust dependencies...
❌ Rust update failed: Failed to run 'cargo update'. Is cargo installed?

Error: Failed to update some dependencies:
  Rust: Failed to run 'cargo update'. Is cargo installed?
```

---

### Story 8: Understand What Gets Updated

**As a** plugin developer  
**I want to** understand exactly what the update command modifies  
**So that** I can confidently run it without fear of breaking my project

**Acceptance Criteria:**
- Documentation clearly explains what gets updated (`Cargo.lock`, `package-lock.json`)
- Documentation clarifies that `Cargo.toml` and `package.json` are not modified (in most cases)
- Documentation mentions respecting semver constraints
- Documentation recommends committing before updating

**Priority:** Medium  
**Estimated Effort:** 1 hour (documentation only)

**Documentation Example:**
```markdown
### Updating Dependencies

To update all dependencies in your plugin project:

```bash
cd my-awesome-plugin
wavecraft update
```

This updates:
- **Rust crates** in `engine/Cargo.lock`
- **npm packages** in `ui/package-lock.json`

The command respects version constraints in `Cargo.toml` and `package.json`.

**Best practice:** Commit your changes before running updates. This makes it
easy to review lock file changes via `git diff`.
```

---

## Non-Functional Requirements

### Performance
- Detection (file existence checks): <10ms
- Overall command execution: 15-90s (dominated by network I/O)
- No noticeable performance impact on CLI startup

### Security
- No command injection vulnerabilities
- No path traversal vulnerabilities
- Uses `Command::new()` with explicit arguments (not shell execution)

### Usability
- Output uses emoji for quick visual scanning (📦, ✅, ❌, ✨)
- Progressive output (user sees updates happening in real-time)
- Error messages are actionable (explain what went wrong and how to fix)
- Exit codes follow Unix conventions (0 = success, 1 = failure)

### Compatibility
- Works on macOS (primary target)
- Theoretically works on Windows/Linux (not tested)
- Requires `cargo` and/or `npm` in PATH
- No special prerequisites beyond tool installation

---

## Out of Scope

### Not Included in v0.8.1

- **Self-updating the CLI** — Use `cargo install wavecraft` instead
- **Selective updates** — Always updates all detected components
- **Interactive prompts** — Consistent with M13's "zero prompts" philosophy  
- **Version pinning** — Delegate to Cargo/npm version specifiers
- **Lock file diff display** — Use `git diff` for this
- **Dry-run mode** — Use underlying tools' `--dry-run` if needed
- **Update schedules** — Manual operation only
- **Dependency conflict resolution** — Handled by cargo/npm

---

## Dependencies

| Dependency | Type | Version |
|------------|------|---------|
| clap | Rust crate | (existing) |
| anyhow | Rust crate | (existing) |
| cargo | External tool | Any recent version |
| npm | External tool | Any recent version |

---

## Success Metrics

**Quantitative:**
- All 8 user stories implemented
- 24/24 implementation tasks complete
- 6 automated tests passing
- 19/19 manual tests passing
- Zero breaking changes

**Qualitative:**
- Users can check version quickly
- Users save time updating dependencies
- Error messages are helpful
- Documentation is clear
- No confusion about command behavior

---

## Timeline

| Milestone | Duration | Completion |
|-----------|----------|------------|
| User Stories | Complete | 2026-02-08 |
| Low-Level Design | Complete | 2026-02-08 |
| Implementation Plan | Complete | 2026-02-08 |
| Implementation | 11-16 hours | TBD |
| Testing & QA | Included | TBD |
| Documentation | Included | TBD |
| **Total** | **1.5-2 days** | **TBD** |

---

## Related Documents

- [Low-Level Design](low-level-design-cli-version-and-update.md) — Architecture and technical decisions
- [Implementation Plan](implementation-plan.md) — Step-by-step implementation guide
- [Implementation Progress](implementation-progress.md) — Task tracking
- [Roadmap](../../roadmap.md) — Milestone 14: CLI Enhancements
- [Backlog](../../backlog.md) — Original feature requests
