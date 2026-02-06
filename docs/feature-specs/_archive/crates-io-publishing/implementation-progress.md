# Implementation Progress: crates.io Publishing

## Overview

Tracking progress for migrating to cargo-workspaces for crates.io publishing.

**Status:** In Progress (Phases 1-3 Complete)  
**Started:** 2026-02-06  
**Completed:** -

---

## Phase 1: Cargo.toml Metadata Updates

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 1.1 | Update workspace root (remove version, add repository) | ✅ Done | Also updated xtask + vite.config.ts |
| 1.2 | Update wavecraft-protocol | ✅ Done | |
| 1.3 | Update wavecraft-macros (add description, authors) | ✅ Done | |
| 1.4 | Update wavecraft-metering (add description) | ✅ Done | |
| 1.5 | Update wavecraft-dsp | ✅ Done | |
| 1.6 | Update wavecraft-bridge | ✅ Done | |
| 1.7 | Update wavecraft-core | ✅ Done | |
| 1.8 | Exclude standalone from publishing | ✅ Done | |
| 1.9 | Verify workspace compiles | ✅ Done | cargo check + clippy pass |

---

## Phase 2: Engine Workflow Update

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 2.1 | Rewrite publish-engine job with cargo-workspaces | ✅ Done | Includes dry-run step |

---

## Phase 3: npm Workflow Update

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 3.1 | Add git tag to publish-cli job | ✅ Done | |
| 3.2 | Add git tag to publish-npm-core job | ✅ Done | |
| 3.3 | Add git tag to publish-npm-components job | ✅ Done | |

---

## Phase 4: Verification

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 4.1 | Install cargo-workspaces locally | ⬜ Not Started | |
| 4.2 | Verify crate metadata locally | ⬜ Not Started | |
| 4.3 | Dry-run publish locally | ⬜ Not Started | Requires crates.io token |
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

