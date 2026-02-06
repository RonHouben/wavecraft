# Implementation Progress: crates.io Publishing

## Overview

Tracking progress for migrating to cargo-workspaces for crates.io publishing.

**Status:** Not Started  
**Started:** -  
**Completed:** -

---

## Phase 1: Cargo.toml Metadata Updates

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 1.1 | Update workspace root (remove version, add repository) | ⬜ Not Started | |
| 1.2 | Update wavecraft-protocol | ⬜ Not Started | |
| 1.3 | Update wavecraft-macros (add description, authors) | ⬜ Not Started | |
| 1.4 | Update wavecraft-metering (add description) | ⬜ Not Started | |
| 1.5 | Update wavecraft-dsp | ⬜ Not Started | |
| 1.6 | Update wavecraft-bridge | ⬜ Not Started | |
| 1.7 | Update wavecraft-core | ⬜ Not Started | |
| 1.8 | Exclude standalone from publishing | ⬜ Not Started | |
| 1.9 | Verify workspace compiles | ⬜ Not Started | |

---

## Phase 2: Engine Workflow Update

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 2.1 | Rewrite publish-engine job with cargo-workspaces | ⬜ Not Started | |

---

## Phase 3: npm Workflow Update

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 3.1 | Add git tag to publish-cli job | ⬜ Not Started | |
| 3.2 | Add git tag to publish-npm-core job | ⬜ Not Started | |
| 3.3 | Add git tag to publish-npm-components job | ⬜ Not Started | |

---

## Phase 4: Verification

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 4.1 | Install cargo-workspaces locally | ⬜ Not Started | |
| 4.2 | Verify crate metadata locally | ⬜ Not Started | |
| 4.3 | Dry-run publish locally | ⬜ Not Started | |
| 4.4 | Check crate name availability | ⬜ Not Started | |
| 4.5 | Test workflow on feature branch | ⬜ Not Started | |

---

## Blockers

None currently.

---

## Notes

- All crates will start at version `0.7.1` but will diverge as independent updates are made
- cargo-workspaces handles dependency cascade automatically
- Git tags format: `wavecraft-{crate}-v{version}` for Rust, `@wavecraft/{pkg}-v{version}` for npm

