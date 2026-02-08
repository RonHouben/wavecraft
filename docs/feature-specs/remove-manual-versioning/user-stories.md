# User Stories: Remove Manual Versioning from Development Flow

## Overview

Manual per-feature version bumping adds unnecessary ceremony to the development flow. The CD pipeline already handles version bumping automatically for all distribution packages (CLI, npm, Rust crates). This feature removes manual version bumping from the documentation, agent roles, and user story templates, simplifying the development process.

The workspace version (`engine/Cargo.toml`) should align with the CLI version since the CLI (`cargo install wavecraft`) is the user's primary entry point to the SDK. Milestone version bumps (minor versions) are the only exception — the PO bumps those during the archive phase when major capabilities are complete.

---

## User Story 1: Remove Per-Feature Version Bumping from Development Flow

**As a** developer (Coder agent)
**I want** version bumping removed from per-feature responsibilities
**So that** I can focus on implementation without versioning ceremony

### Acceptance Criteria
- [ ] Coding standards no longer instruct Coder to bump versions per feature
- [ ] Agent development flow no longer lists version bumping as a Coder responsibility
- [ ] Tester role no longer includes "verifies version display" for every feature
- [ ] PO user story template no longer includes a mandatory "Version" section

---

## User Story 2: Consolidate Versioning Documentation to CI-Only Model

**As a** contributor reading the documentation
**I want** clear guidance that all versioning is handled by CI
**So that** I understand I should never manually bump versions (except at milestones)

### Acceptance Criteria
- [ ] Coding standards "Versioning" section describes the automated CI model
- [ ] The two-tier version domain table is replaced with a single automated model
- [ ] Milestone bumps are documented as the only manual version action (PO only, during archive phase)
- [ ] Milestone criteria are defined: API breaking changes, major feature completions, documentation completeness

---

## User Story 3: Align Workspace Version with CLI Version

**As a** plugin developer using Wavecraft
**I want** the workspace version to match the CLI version
**So that** the version I see (`cargo install wavecraft`) matches the SDK version in my project

### Acceptance Criteria
- [ ] Documentation states that workspace version and CLI version should be aligned
- [ ] CLI is documented as the user's entry point and version source of truth
- [ ] High-level design versioning section reflects the unified version model

---

## Notes

- **Milestones are feature-based** — triggered when major capabilities are complete
- **Milestone criteria** — API breaking changes, major feature completions, documentation completeness
- **Only the PO** bumps the workspace version, and only at milestones (during archive phase)
- All other version bumping is fully automated by the CD pipeline
