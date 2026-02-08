# User Stories: CLI Self-Update in `wavecraft update`

**Feature:** CLI Self-Update (Milestone 14 Enhancement)  
**Target Version:** `0.9.1` (patch — enhancement to existing command, no breaking changes)  
**Created:** 2026-02-08

---

## Overview

The `wavecraft update` command currently only updates project dependencies (Rust crates + npm packages) and requires being inside a plugin project directory. It does **not** update the CLI itself — which is the most important thing to keep current.

An outdated CLI means outdated templates, outdated dependency resolution, and potentially incompatible tooling. The CLI should self-update **first**, then proceed to update project dependencies if applicable.

### Problem

Today, a developer must manually run `cargo install wavecraft` to update the CLI, and then separately run `wavecraft update` for project deps. Most developers won't remember to update the CLI regularly, leading to:

- Using outdated templates when creating new projects
- Missing new commands or flags
- Potential incompatibilities between CLI version and SDK crate versions

### Solution

Make `wavecraft update` a single command that handles **everything**:

1. Self-update the CLI binary (`cargo install wavecraft`)
2. Update project dependencies (if in a plugin project)
3. Work from **any** directory (not just inside a project)

---

## Version

**Target Version:** `0.9.1` (patch from `0.9.0`)

**Rationale:** This enhances an existing command with additional functionality. No breaking changes — existing behavior (project dependency updates) is preserved. The self-update step is additive.

---

## User Story 1: Self-Update the CLI First

**As a** plugin developer  
**I want** `wavecraft update` to update the CLI tool itself before updating project dependencies  
**So that** I'm always running the latest version and don't have to remember a separate update step

### Acceptance Criteria

- [ ] `wavecraft update` runs `cargo install wavecraft` as the first step
- [ ] Shows clear progress: "🔄 Checking for CLI updates..."
- [ ] If a newer version was installed, shows: "✅ CLI updated to X.Y.Z (was A.B.C)"
- [ ] If already up-to-date, shows: "✅ CLI is up to date (X.Y.Z)"
- [ ] If CLI update fails, shows error but continues with project dependency updates
- [ ] CLI self-update happens **before** any project dependency updates

### Notes

- Uses `cargo install wavecraft` which is idempotent (skips if already latest)
- The current binary continues executing after the update (no re-exec)
- If the CLI was updated, a note is shown: "💡 Note: Project dependencies were updated using the previous CLI version. Re-run `wavecraft update` to use the new CLI."

---

## User Story 2: Work from Any Directory

**As a** plugin developer  
**I want** to run `wavecraft update` from any directory (not just inside a project)  
**So that** I can update my CLI without navigating to a specific project first

### Acceptance Criteria

- [ ] `wavecraft update` works from any directory (no longer errors outside projects)
- [ ] If outside a plugin project, only updates the CLI itself
- [ ] If inside a plugin project, updates CLI **and** project dependencies
- [ ] Shows clear messaging about what was updated based on context
- [ ] Exit code is 0 if all applicable updates succeed

### Example (outside project)

```bash
$ cd ~
$ wavecraft update
🔄 Checking for CLI updates...
✅ CLI updated to 0.9.1 (was 0.9.0)

ℹ️  Not in a Wavecraft plugin project — skipping dependency updates.
   Run this command from a project root to also update Rust and npm dependencies.
```

### Example (inside project)

```bash
$ cd ~/my-plugin
$ wavecraft update
🔄 Checking for CLI updates...
✅ CLI is up to date (0.9.1)
📦 Updating Rust dependencies...
✅ Rust dependencies updated
📦 Updating npm dependencies...
✅ npm dependencies updated

✨ All updates complete
```

---

## User Story 3: Graceful Handling When `cargo` Is Unavailable

**As a** plugin developer who might have an unusual setup  
**I want** clear feedback if the CLI self-update fails  
**So that** I understand what happened and can fix it manually

### Acceptance Criteria

- [ ] If `cargo install` fails, shows a clear error with the reason
- [ ] Failure to self-update does **not** prevent project dependency updates
- [ ] Suggests manual alternative: "Run `cargo install wavecraft` manually to update the CLI"
- [ ] Exit code reflects partial success (0 if project deps succeeded, 1 if everything failed)

### Example (cargo not available)

```bash
$ wavecraft update
🔄 Checking for CLI updates...
⚠️  CLI self-update failed: Failed to run 'cargo install'. Is cargo installed?
   Run 'cargo install wavecraft' manually to update the CLI.

📦 Updating Rust dependencies...
✅ Rust dependencies updated
📦 Updating npm dependencies...
✅ npm dependencies updated

✨ Project dependencies updated (CLI self-update skipped)
```

---

## User Story 4: Version Change Notification

**As a** plugin developer  
**I want** to know when the CLI was actually updated to a new version  
**So that** I'm aware that project dependency updates were performed by the old binary

### Acceptance Criteria

- [ ] When a new CLI version is installed, the output clearly shows old → new version
- [ ] A note recommends re-running if the CLI was updated: "💡 Re-run `wavecraft update` to use the new CLI for dependency updates"
- [ ] When the CLI is already at the latest version, no re-run suggestion is shown
- [ ] The version comparison uses the CLI's own version (`env!("CARGO_PKG_VERSION")`) vs the newly installed version

### Notes

- This is important because `cargo install` might update the binary on disk, but the currently running process is still the old version
- The re-run suggestion is informational, not a hard requirement — the old binary's dependency updates are still valid in most cases

---

## User Story 5: Help Text Reflects New Behavior

**As a** plugin developer  
**I want** the help text for `wavecraft update` to reflect that it also updates the CLI  
**So that** I understand the full scope of what the command does

### Acceptance Criteria

- [ ] `wavecraft --help` shows updated description for `update` command
- [ ] `wavecraft update --help` explains the two-phase update (CLI + dependencies)
- [ ] Help text mentions that the command works from any directory

### Example

```bash
$ wavecraft --help
...
Commands:
  create  Create a new Wavecraft plugin project
  start   Start development servers for browser-based UI development
  update  Update the CLI and project dependencies (Rust crates + npm packages)
  help    Print this message or the help of the given subcommand(s)
```

```bash
$ wavecraft update --help
Update the CLI and project dependencies

Updates the Wavecraft CLI to the latest version, then updates Rust crates
and npm packages if run from a plugin project directory.

Usage: wavecraft update

Options:
  -h, --help  Print help
```

---

## Out of Scope

- **Automatic update checks on every command** — Only `wavecraft update` triggers updates; no background checks on `create`, `start`, etc.
- **Rollback mechanism** — If the update breaks something, users can `cargo install wavecraft@0.9.0` to pin a version
- **Offline mode** — The command requires internet access; no offline caching of versions
- **npm global update** — The CLI is Rust-only; no npm global package to update

---

## Priority & Effort

| Story | Priority | Effort |
|-------|----------|--------|
| Story 1: Self-update CLI first | **Critical** | 4-6 hours |
| Story 2: Work from any directory | **High** | 1-2 hours |
| Story 3: Graceful error handling | **High** | 1-2 hours |
| Story 4: Version change notification | **Medium** | 2-3 hours |
| Story 5: Updated help text | **Medium** | 30 min |

**Total estimated effort:** 8-13 hours

---

## Success Metrics

1. A developer can run `wavecraft update` from their home directory and get the latest CLI
2. A developer can run `wavecraft update` from a project and get both CLI + dependency updates
3. Partial failures don't block the rest of the update process
4. The command is self-documenting via clear progress output
