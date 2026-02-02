# User Stories: Project Rename (VstKit → Wavecraft)

## Overview

Rename the project from "VstKit" to "Wavecraft" to avoid potential "VST" trademark concerns before public/open-source release. "VST" is a Steinberg trademark, and while "VstKit" may be defensible, rebranding to "Wavecraft" eliminates any trademark risk and establishes a unique, memorable identity.

---

## Version

**Target Version:** `0.5.0` (minor bump from `0.4.0`)

**Rationale:** This is a significant change affecting all crate names, npm packages, documentation, and branding. A minor version bump communicates the scope of change to users and signals a fresh identity for the project.

---

## Name Availability (Verified 2026-02-02)

| Platform | Name | Status |
|----------|------|--------|
| **GitHub** | `wavecraft` | ⚠️ User exists (inactive since 2020, 1 repo about electronics) |
| **crates.io** | `wavecraft`, `wavecraft-*` | ✅ Available |
| **npm** | `@wavecraft/*` | ✅ Available |
| **Domain** | `wavecraft.dev` | ✅ Available (€10.89/yr) |

**GitHub Strategy:** Keep repository on personal account (`RonHouben/wavecraft`) for now. Request inactive `WaveCraft` username via GitHub's Name Squatting Policy later (see User Story 7).

---

## User Story 1: Rename Rust Crates

**As a** plugin developer using Wavecraft  
**I want** the Rust crates to be named `wavecraft-*`  
**So that** I can import them with the new name and avoid VST trademark concerns

### Acceptance Criteria

- [ ] `vstkit-protocol` → `wavecraft-protocol`
- [ ] `vstkit-dsp` → `wavecraft-dsp`
- [ ] `vstkit-bridge` → `wavecraft-bridge`
- [ ] `vstkit-metering` → `wavecraft-metering`
- [ ] `vstkit-core` → `wavecraft-core`
- [ ] All `Cargo.toml` files updated (workspace + crates)
- [ ] Internal dependencies updated to new names
- [ ] All `use vstkit_*` imports updated to `use wavecraft_*`
- [ ] Code compiles with `cargo build --workspace`

### Notes

- Crate folder names should also change (`engine/crates/vstkit-*` → `engine/crates/wavecraft-*`)
- Update workspace members in root `Cargo.toml`

---

## User Story 2: Update Macro and Prelude

**As a** plugin developer  
**I want** the `vstkit_plugin!` macro renamed to `wavecraft_plugin!`  
**So that** my plugin declaration uses the new brand name

### Acceptance Criteria

- [ ] `vstkit_plugin!` macro → `wavecraft_plugin!`
- [ ] `vstkit_core::prelude::*` → `wavecraft_core::prelude::*`
- [ ] Template project updated to use new macro name
- [ ] SDK Getting Started guide updated with new macro name

### Notes

- The macro is defined in `vstkit-core/src/lib.rs`

---

## User Story 3: Update npm Package Names

**As a** UI developer  
**I want** the npm package aliases to use `@wavecraft/*`  
**So that** imports are consistent with the new brand

### Acceptance Criteria

- [ ] Update `tsconfig.json` path aliases: `@vstkit/*` → `@wavecraft/*`
- [ ] Update `vite.config.ts` aliases
- [ ] Update `vitest.config.ts` aliases
- [ ] Update all import statements in UI code
- [ ] All UI tests pass (`npm run test`)
- [ ] All lint checks pass (`npm run lint`)

### Notes

- This is an internal alias change for now (not publishing to npm registry)
- When publishing, reserve `@wavecraft` organization on npm

---

## User Story 4: Update Documentation

**As a** new developer evaluating Wavecraft  
**I want** all documentation to consistently use "Wavecraft"  
**So that** I understand this is the official name of the project

### Acceptance Criteria

- [ ] `README.md` — Update project name, description, all references
- [ ] `docs/roadmap.md` — Update milestone 9 title, all references
- [ ] `docs/backlog.md` — Update project rename item
- [ ] `docs/architecture/high-level-design.md` — Update all references
- [ ] `docs/architecture/coding-standards.md` — Update all references
- [ ] `docs/guides/sdk-getting-started.md` — Update all references
- [ ] `docs/guides/macos-signing.md` — Update all references
- [ ] `docs/guides/visual-testing.md` — Update all references
- [ ] `docs/guides/ci-pipeline.md` — Update all references
- [ ] Agent instructions (`.github/copilot-instructions.md`) — Update references
- [ ] No remaining "VstKit" or "vstkit" references in docs (except historical changelog)

### Notes

- Preserve historical changelog entries that mention "VstKit" — they're accurate records
- Update any ASCII art or diagrams that include the name

---

## User Story 5: Update UI Branding

**As an** end user of a Wavecraft-built plugin  
**I want** the UI to display "Wavecraft" branding (if any)  
**So that** the experience is consistent with the project name

### Acceptance Criteria

- [ ] Update any user-visible "VstKit" text in the UI
- [ ] Update `VersionBadge` component if it references VstKit
- [ ] Update page title in `index.html` if applicable
- [ ] Update any debug/dev mode indicators

### Notes

- The current UI may not have much explicit branding — verify what exists

---

## User Story 6: Update Template Project

**As a** developer starting a new plugin  
**I want** the template project to use Wavecraft naming  
**So that** my new project follows the correct conventions

### Acceptance Criteria

- [ ] Rename `vstkit-plugin-template/` → `wavecraft-plugin-template/`
- [ ] Update template `Cargo.toml` to reference `wavecraft-*` crates
- [ ] Update template `README.md`
- [ ] Update any template code that references vstkit
- [ ] Template builds successfully with new names

### Notes

- The template is a separate project that demonstrates SDK usage

---

## User Story 7: Plan GitHub Username Request (Deferred)

**As the** project maintainer  
**I want** to document the process for requesting the inactive `WaveCraft` GitHub username  
**So that** we can potentially migrate to `github.com/wavecraft` in the future

### Acceptance Criteria

- [ ] Document GitHub's Name Squatting Policy process in backlog
- [ ] Note that `WaveCraft` user has been inactive since 2020
- [ ] Add task to submit request after project is stable and public
- [ ] Record current repository location: `RonHouben/wavecraft`

### Notes

- GitHub's policy: https://docs.github.com/en/site-policy/other-site-policies/github-username-policy
- Requires demonstrating legitimate interest and that name is being squatted
- Not blocking for rename — repository works fine under personal account
- Consider this for post-1.0 or when going fully open-source

---

## User Story 8: Update CI/CD and Build System

**As a** contributor  
**I want** the CI/CD pipelines to work with the new names  
**So that** automated builds and tests continue to function

### Acceptance Criteria

- [ ] Update `.github/workflows/*.yml` if they reference vstkit paths
- [ ] Update `xtask` commands if they have hardcoded vstkit references
- [ ] Update any bundle/signing scripts that reference vstkit
- [ ] CI pipeline passes on feature branch
- [ ] All build artifacts use new naming

### Notes

- Check `cargo xtask bundle`, `cargo xtask sign`, etc.
- Plugin bundle names may include the project name

---

## User Story 9: Rename GitHub Repository (Final Step)

**As the** project maintainer  
**I want** to rename the GitHub repository from `vstkit` to `wavecraft`  
**So that** the repository URL matches the project name

### Acceptance Criteria

- [ ] Rename repository: `RonHouben/vstkit` → `RonHouben/wavecraft`
- [ ] Verify GitHub redirect works from old URL
- [ ] Update local git remote: `git remote set-url origin ...`
- [ ] Update any external links (if documented elsewhere)
- [ ] Update clone instructions in README

### Notes

- Do this as the LAST step after all code changes are merged
- GitHub automatically creates redirects from old repo name
- Contributors will need to update their remotes

---

## Out of Scope

The following are explicitly **not** part of this milestone:

- Publishing to crates.io (future milestone)
- Publishing to npm registry (future milestone)
- Registering `wavecraft.dev` domain (optional, not required)
- Requesting GitHub username (deferred task, see User Story 7)
- Creating a logo or visual brand identity

---

## Implementation Order

Recommended sequence to minimize conflicts:

1. **Rust crates** (User Story 1) — Foundation for everything else
2. **Macro/Prelude** (User Story 2) — Depends on crate rename
3. **npm aliases** (User Story 3) — UI code changes
4. **Template project** (User Story 6) — Uses SDK crates
5. **Documentation** (User Story 4) — Can be done in parallel
6. **UI branding** (User Story 5) — Minor, can be done anytime
7. **CI/CD** (User Story 8) — Verify everything works
8. **GitHub repository rename** (User Story 9) — Very last step, after merge

---

## Definition of Done

- [ ] All code compiles (`cargo build --workspace`)
- [ ] All Rust tests pass (`cargo test --workspace`)
- [ ] All UI tests pass (`npm run test`)
- [ ] All lint checks pass (`cargo xtask lint`)
- [ ] Template project builds successfully
- [ ] Documentation has no "vstkit" references (except changelog)
- [ ] CI pipeline passes
- [ ] Version bumped to 0.5.0
- [ ] Repository renamed on GitHub
