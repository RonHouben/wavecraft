# User Stories: Codebase Refactor Sweep

## Overview

Systematic refactoring sweep across the entire Wavecraft codebase to improve code quality, reduce file complexity, extract abstractions, and improve maintainability — while preserving all existing behavior. Small quality-of-life improvements (better error messages, clearer naming) are allowed alongside structural changes.

**Delivery:** Single PR containing all tiers of changes.

**Scope:** ~26K lines across 162 source files (77 Rust, 76 TypeScript, 9 template).

---

## User Story 1: Tier 1 — Deep Refactor of Hotspot Files

**As a** Wavecraft SDK maintainer
**I want** the largest, most complex files decomposed into focused modules
**So that** each file has a single responsibility, is easier to navigate, and reduces merge conflict surface

### Target Files (>500 lines)

| File                                                     | Lines | Refactor Focus                                                     |
| -------------------------------------------------------- | ----- | ------------------------------------------------------------------ |
| `cli/src/commands/start.rs`                              | 1,640 | Split into phases: server setup, UI build, codegen, file watching  |
| `cli/src/commands/bundle_command.rs`                     | 1,140 | Extract format-specific bundling, signing, validation              |
| `engine/crates/wavecraft-macros/src/plugin.rs`           | 1,016 | Split into parse/metadata/codegen modules (backlog item)           |
| `cli/src/template/mod.rs`                                | 992   | Extract template rendering, variable substitution, file operations |
| `dev-server/src/audio/server.rs`                         | 923   | Extract audio device management, stream setup, callback logic      |
| `engine/crates/wavecraft-protocol/src/ipc.rs`            | 746   | Separate message types, serialization, contract validation         |
| `cli/src/commands/update.rs`                             | 669   | Extract self-update, dependency update, version resolution         |
| `engine/crates/wavecraft-nih_plug/src/editor/windows.rs` | 598   | Review platform abstraction opportunities with macos.rs            |

### Acceptance Criteria

- [ ] No file in Tier 1 exceeds 400 lines after refactoring (excluding tests)
- [ ] Each extracted module has a clear, documented purpose (doc comment on `mod` or `pub mod`)
- [ ] All existing tests pass without modification (or with minimal test-infra updates)
- [ ] `cargo xtask ci-check` passes cleanly
- [ ] No behavior changes beyond minor quality-of-life improvements (error messages, naming)

### Notes

- The `wavecraft_plugin!` macro refactor (backlog item) is included here as part of `plugin.rs` decomposition
- Platform-specific editor files (`windows.rs`, `macos.rs`) may benefit from shared trait extraction
- `start.rs` is the most-used developer command — quality here directly impacts SDK UX

---

## User Story 2: Tier 2 — Quick Scan of Medium Files

**As a** Wavecraft SDK maintainer
**I want** medium-complexity files reviewed for obvious improvements
**So that** naming is consistent, dead code is removed, and clear extraction opportunities are captured

### Target Files (~200-500 lines)

Files in this range across all crates and packages. Focus areas:

- **Naming consistency** — Do names follow the coding standards?
- **Dead code** — Unused imports, unreachable branches, commented-out code
- **Obvious extractions** — Helper functions, shared constants, repeated patterns
- **Error handling** — Consistent error types, actionable messages

### Acceptance Criteria

- [ ] Dead code and unused imports removed
- [ ] Naming inconsistencies fixed per coding standards
- [ ] Obvious code duplication reduced (DRY)
- [ ] No new files created unless clearly justified by extraction
- [ ] Changes grouped logically by crate/package

### Notes

- This tier is about quick wins, not deep restructuring
- If a medium file reveals deep issues, flag it for a future Tier 1 treatment rather than expanding scope

---

## User Story 3: Tier 3 — Automated Lint Pass

**As a** Wavecraft SDK maintainer
**I want** automated tooling to clean up formatting, lint warnings, and style issues across all remaining files
**So that** the entire codebase meets baseline quality standards

### Acceptance Criteria

- [ ] `cargo fmt` applied to all Rust code
- [ ] `cargo clippy` warnings resolved (or explicitly suppressed with justification)
- [ ] ESLint + Prettier applied to all TypeScript code
- [ ] No new lint suppressions without documented justification
- [ ] `cargo xtask ci-check` passes cleanly

### Notes

- This tier should be the final step, applied after Tier 1 and Tier 2 changes
- Automated fixes only — no manual review of small/clean files

---

## User Story 4: Quality-of-Life Improvements

**As a** Wavecraft SDK user (plugin developer)
**I want** clearer error messages and better developer ergonomics
**So that** when something goes wrong, I can understand and fix it quickly

### Acceptance Criteria

- [ ] Error messages in CLI commands include actionable guidance (what went wrong + what to do)
- [ ] Panic messages replaced with proper `Result` returns where found
- [ ] Public API doc comments reviewed and improved where unclear
- [ ] No behavior-breaking changes

### Notes

- These improvements are allowed alongside structural refactoring
- Focus on the developer-facing surface: CLI output, error messages, public API docs
- Internal implementation comments are lower priority

---

## User Story 5: Lessons Learned — Coding Guidelines Updates

**As a** Wavecraft SDK maintainer
**I want** common patterns, anti-patterns, and improvement opportunities discovered during the refactor captured as coding guideline updates
**So that** the same issues don't recur in new code and contributors have clear guidance to follow

### Process

During each tier of the refactor, the Coder should track recurring issues and patterns that should be codified. At the end of the sweep, these are distilled into updates to the coding standards.

### What to Capture

- **Recurring anti-patterns** — What mistakes kept appearing? (e.g., "functions over 100 lines", "error messages without context")
- **Effective abstractions** — What extractions worked well? (e.g., "builder pattern for complex setup", "trait extraction for platform code")
- **File organization rules** — Size thresholds, module splitting conventions, when to create a new file vs. extend existing
- **Error handling patterns** — Consistent Result types, error context chains, actionable messages
- **Naming conventions** — Any gaps or ambiguities discovered in current naming standards
- **Testing patterns** — How structural changes affected tests; what test organization works best

### Acceptance Criteria

- [ ] A `lessons-learned.md` document is created in the feature-spec folder capturing all findings
- [ ] At least the top 5 most impactful patterns are identified with before/after examples
- [ ] Concrete PRs or items are proposed for updating `docs/architecture/coding-standards*.md` files
- [ ] Each proposed guideline update includes rationale (why) and enforcement method (how — lint rule, code review checklist, etc.)
- [ ] DocWriter agent is invoked to apply approved updates to the coding standards documents

### Notes

- This is a **passive output** of the refactoring work — the Coder tracks findings during Tiers 1-3, not a separate pass
- Focus on patterns that are **actionable and enforceable**, not vague advice
- Language-specific findings go to the relevant coding standards doc (Rust, TypeScript, CSS, Testing)
- Cross-cutting findings go to the main `coding-standards.md`

---

## Prioritization

| Story                             | Priority   | Estimated Effort | Risk                                      |
| --------------------------------- | ---------- | ---------------- | ----------------------------------------- |
| US1: Tier 1 Deep Refactor         | **High**   | Large            | Medium (structural changes in core files) |
| US2: Tier 2 Quick Scan            | **Medium** | Medium           | Low (focused improvements)                |
| US3: Tier 3 Lint Pass             | **Low**    | Small            | Very Low (automated tooling)              |
| US4: QoL Improvements             | **Medium** | Small            | Low (additive improvements)               |
| US5: Lessons Learned / Guidelines | **Medium** | Small            | Very Low (documentation output)           |

**Execution order:** US3 (lint baseline) → US1 (deep refactor) → US2 (quick scan) → US4 (QoL) → US3 again (final lint pass) → US5 (distill findings into guidelines)

---

## Out of Scope

- New features or capabilities
- Architecture changes (e.g., new crates, new IPC methods)
- Performance optimization (separate backlog item)
- Documentation updates beyond code comments (DocWriter handles post-refactor doc sync)
- Version bumping (automated by CD pipeline)

---

## Risk Assessment

| Risk                                | Likelihood | Impact | Mitigation                                                         |
| ----------------------------------- | ---------- | ------ | ------------------------------------------------------------------ |
| Regressions from structural changes | Medium     | High   | Full CI passes required; test before and after each Tier 1 file    |
| Scope creep during deep refactor    | Medium     | Medium | Strict file-count limits per tier; flag expansions for future work |
| Merge conflicts with other work     | Low        | Medium | No other milestones in flight; single PR minimizes window          |
| Single mega-PR is hard to review    | Medium     | Low    | Organized by tier; clear commit messages; CI as safety net         |

---

## Definition of Done

- [ ] All Tier 1 files decomposed (no file >400 lines excluding tests)
- [ ] All Tier 2 quick-scan improvements applied
- [ ] All Tier 3 lint/format passes clean
- [ ] `cargo xtask ci-check` passes
- [ ] QoL improvements applied where found
- [ ] No behavior changes beyond documented minor improvements
- [ ] `lessons-learned.md` created with top patterns and proposed coding guideline updates
- [ ] Coding standards updates proposed (with before/after examples)

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions to enforce
- [Roadmap](../../roadmap.md) — Milestone tracking
- [Backlog](../../backlog.md) — Macro refactor item (promoted to this milestone)
