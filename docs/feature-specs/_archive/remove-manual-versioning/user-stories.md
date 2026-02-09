# User Stories: Remove Manual Versioning from Development Flow

## Overview

Manual per-feature version bumping adds unnecessary ceremony to the development flow. The CD pipeline already handles version bumping automatically for all distribution packages (CLI, npm, Rust crates). This feature removes manual version bumping from the documentation, agent roles, and user story templates, simplifying the development process.

The workspace version (`engine/Cargo.toml`) should align with the CLI version since the CLI (`cargo install wavecraft`) is the user's primary entry point to the SDK. All version bumping is fully automated by the CD pipeline — no manual bumps are needed.

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
- [ ] Documentation confirms all versioning is fully CI-automated with no manual exceptions

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

- All version bumping is fully automated by the CD pipeline
- No manual version bumps are needed — not per feature, not at milestones
- If a specific version is needed (e.g., minor bump for a breaking change), bump it in the PR — CI will respect the manual bump
