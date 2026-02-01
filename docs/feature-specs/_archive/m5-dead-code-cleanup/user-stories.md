# User Stories: M5 Dead Code Cleanup

## Overview

During the resize-handle feature implementation, several `#[allow(dead_code)]` suppressions were added as a workaround to silence compiler warnings. Now that React UI is the default (and only) editor, these suppressions and associated code should be reviewed:

- **Remove** code that is genuinely dead and no longer needed
- **Keep** code that serves a legitimate purpose (platform traits, future Windows/Linux support)
- **Update** stale comments that reference "when editor is re-enabled" (it IS enabled now)

This cleanup prepares the codebase for Milestone 6 (WebSocket IPC Bridge) with a clean foundation.

---

## Version

**Target Version:** `0.2.2` (patch bump from `0.2.1`)

**Rationale:** Housekeeping cleanup with no user-visible changes. Patch version appropriate per coding standards.

---

## Scope

**14 `#[allow(dead_code)]` suppressions across 5 files:**

| File | Count | Category |
|------|-------|----------|
| `plugin/src/editor/webview.rs` | 5 | Platform traits, config structs |
| `plugin/src/editor/assets.rs` | 3 | Asset serving API |
| `plugin/src/editor/mod.rs` | 2 | IPC message variants |
| `plugin/src/editor/bridge.rs` | 2 | Bridge setup |
| `plugin/src/editor/windows.rs` | 1 | Window handle storage |
| `desktop/src/assets.rs` | 1 | Desktop asset handling |

---

## User Story 1: Review and Categorize Dead Code

**As a** maintainer of the VstKit codebase  
**I want** each `#[allow(dead_code)]` suppression reviewed and categorized  
**So that** we make informed decisions about what to keep vs. remove

### Acceptance Criteria

- [ ] Each of the 14 suppressions is categorized as:
  - **REMOVE** — Code is genuinely dead, no longer needed
  - **KEEP** — Code serves a purpose (platform completeness, future use with clear rationale)
  - **REFACTOR** — Code structure should change to eliminate the need for suppression
- [ ] Findings documented in implementation progress

### Notes

Categories of suppressions found:
1. Platform trait completeness (enums with variants for Windows/Linux)
2. Structs/fields used on some platforms but not others
3. "Will be used when editor is re-enabled" — stale, editor IS enabled
4. "Future IPC use" — assess if genuinely planned or speculative

---

## User Story 2: Remove Dead Code

**As a** developer working on VstKit  
**I want** genuinely dead code removed from the codebase  
**So that** the code is easier to understand and maintain

### Acceptance Criteria

- [ ] All code categorized as REMOVE is deleted
- [ ] Associated `#[allow(dead_code)]` suppressions are removed
- [ ] `cargo clippy --workspace -- -D warnings` passes without new warnings
- [ ] `cargo test --workspace` passes
- [ ] No functional changes to plugin behavior

### Notes

- If removing code reveals other dead code (unused imports, etc.), remove those too
- Prefer surgical removal over large refactors

---

## User Story 3: Update Stale Comments

**As a** developer reading the codebase  
**I want** comments to accurately describe the code's purpose  
**So that** I'm not misled by outdated information

### Acceptance Criteria

- [ ] Comments saying "when editor is re-enabled" are removed or updated (React UI IS the editor)
- [ ] Comments on KEEP items explain WHY they're kept (e.g., "Required for Windows support")
- [ ] No misleading "TODO" or "will be used" comments for code that won't be used

---

## User Story 4: Clean Compile Without Suppressions

**As a** maintainer  
**I want** the codebase to compile cleanly without `#[allow(dead_code)]` workarounds  
**So that** future dead code is caught immediately by the compiler

### Acceptance Criteria

- [ ] Remaining `#[allow(dead_code)]` suppressions have clear, documented justification
- [ ] Target: Reduce from 14 suppressions to ≤5 (legitimate platform-specific code)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] All existing tests pass

---

## Out of Scope

- CI cache optimization (separate task, lower priority)
- Refactoring architecture of the editor modules
- Windows/Linux support work
- New features or functionality

---

## Success Metrics

| Metric | Before | Target |
|--------|--------|--------|
| `#[allow(dead_code)]` count | 14 | ≤5 |
| Stale "re-enable" comments | ~4 | 0 |
| Build warnings | 0 | 0 |
| Test failures | 0 | 0 |

---

## Handoff

→ **Architect**: Create low-level design analyzing each suppression and recommending action
