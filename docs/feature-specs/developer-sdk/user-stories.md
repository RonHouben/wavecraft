# User Stories: Developer SDK (Milestone 8)

## Overview

VstKit has achieved its core framework goals (Milestones 1–7): a working Rust + React audio plugin with VST3/CLAP export, macOS signing, real-time metering, and a complete development workflow. **The framework works.**

The next step is to make VstKit **usable by other developers**. Currently, building a new plugin requires deep knowledge of the codebase, manual setup, and copying/modifying internal code. This milestone transforms VstKit from an internal project into a proper **Developer SDK** that others can use to build their own audio plugins.

## Version

**Target Version:** `0.4.0` (minor bump from `0.3.1`)

**Rationale:** This is a significant milestone that changes VstKit's positioning from an internal project to an external SDK. A minor version bump communicates this strategic shift.

---

## Problem Statement

### Current Pain Points

1. **No scaffolding** — Developers must clone the entire repo and manually strip out example code
2. **No clear boundaries** — It's unclear what's "framework" code vs "example" code
3. **No documentation** — No getting-started guide for external developers
4. **No CLI tooling** — No `cargo vstkit new my-plugin` equivalent
5. **No versioning strategy** — How do SDK updates affect user projects?

### Strategic Context

Similar frameworks solve these problems differently:
- **nih-plug**: Template repos + `cargo xtask bundle` command
- **JUCE**: Projucer GUI + CMake integration
- **iPlug2**: Template project + documentation

VstKit needs to find its own approach that plays to Rust's strengths (Cargo, workspaces, crates.io).

---

## Phase 1: Investigation & Architecture

This phase is **research-focused**. The goal is to answer fundamental questions about SDK design before committing to implementation. No code changes expected.

### User Story 1: SDK Packaging Research

**As a** VstKit maintainer  
**I want** to understand the viable SDK packaging options  
**So that** I can make an informed decision about the distribution model

#### Acceptance Criteria
- [ ] Research at least 3 SDK packaging approaches:
  - Cargo workspace template (like nih-plug)
  - CLI scaffolding tool (like `create-react-app`)
  - Binary + source hybrid (like JUCE Projucer)
- [ ] Document pros/cons of each approach
- [ ] Recommend an approach with clear rationale
- [ ] Consider future crates.io distribution

#### Questions to Answer
- Should VstKit be a single crate or multiple crates?
- Should user projects depend on VstKit via git or crates.io?
- How do we handle the UI layer (React + Vite)?

---

### User Story 2: SDK Boundary Definition

**As a** VstKit architect  
**I want** clear boundaries between framework code and user code  
**So that** developers know what to customize vs what to leave alone

#### Acceptance Criteria
- [ ] Audit current crate structure and classify each crate:
  - **Framework** — SDK provides, user doesn't touch
  - **Template** — User copies and modifies
  - **Example** — Reference implementation, optional
- [ ] Define where user DSP code should live
- [ ] Define UI customization points (components, styling, layout)
- [ ] Document the boundary in the low-level design

#### Crates to Classify
| Crate | Current Role |
|-------|--------------|
| `protocol` | JSON-RPC types and IPC contracts |
| `bridge` | IPC handler implementation |
| `dsp` | Audio processing primitives |
| `metering` | SPSC ring buffer for meters |
| `plugin` | nih-plug integration |
| `standalone` | Desktop app for testing |
| `xtask` | Build tooling |

---

### User Story 3: Developer Persona Definition

**As a** VstKit product owner  
**I want** a clear target developer persona  
**So that** we design the SDK for the right audience

#### Acceptance Criteria
- [ ] Define minimum Rust experience level (beginner/intermediate/advanced)
- [ ] Define expected audio/DSP knowledge
- [ ] Define expected web development knowledge (React/TypeScript)
- [ ] Document assumptions about development environment (macOS required?)
- [ ] Identify adjacent ecosystems (nih-plug users, JUCE refugees, etc.)

#### Persona Candidates
1. **Rust DSP Developer** — Knows Rust and DSP, new to plugins
2. **Plugin Developer** — Knows plugin development (JUCE/etc), new to Rust
3. **Full-Stack Developer** — Knows React, wants to build audio tools
4. **Hobbyist** — Musician who wants to build custom effects

*Which persona(s) should we optimize for?*

---

### User Story 4: Minimum Viable Workflow Design

**As a** potential VstKit user  
**I want** to understand the end-to-end workflow for building a plugin  
**So that** I can evaluate if VstKit is right for my project

#### Acceptance Criteria
- [ ] Document the ideal workflow from zero to working plugin:
  1. Install prerequisites
  2. Create new project
  3. Implement DSP logic
  4. Customize UI
  5. Build and test
  6. Package for distribution
- [ ] Identify which steps exist today vs need building
- [ ] Estimate effort for missing pieces
- [ ] Define "minimum viable SDK" scope

#### Workflow Questions
- What's the first command a user runs?
- How long until they see a working plugin in a DAW?
- What's the smallest useful plugin they can build?

---

### User Story 5: Versioning Strategy

**As a** VstKit user  
**I want** to understand how SDK updates affect my project  
**So that** I can plan for maintenance and upgrades

#### Acceptance Criteria
- [ ] Define semantic versioning policy for the SDK
- [ ] Define upgrade path (how users update to new SDK versions)
- [ ] Define breaking change policy
- [ ] Consider lock file / reproducible builds
- [ ] Document in low-level design

#### Scenarios to Address
- SDK bug fix (patch) — User wants it automatically
- New feature (minor) — User can opt-in
- Breaking change (major) — User needs migration guide

---

### User Story 6: Documentation Requirements

**As a** VstKit adopter  
**I want** comprehensive documentation  
**So that** I can learn the SDK without reading source code

#### Acceptance Criteria
- [ ] Identify required documentation artifacts:
  - Getting Started guide
  - Architecture overview
  - API reference
  - Tutorials (build your first plugin)
  - Examples (gain, EQ, synth)
- [ ] Define documentation home (GitHub Wiki, mdBook, Docusaurus, etc.)
- [ ] Estimate documentation effort
- [ ] Prioritize documentation for Phase 2

---

## Phase 2: Implementation (Scope TBD)

*Phase 2 scope will be defined after Phase 1 investigation with Architect.*

Potential deliverables (to be validated):
- [ ] CLI tool for project scaffolding
- [ ] Template project structure
- [ ] Getting Started documentation
- [ ] Example plugins
- [ ] crates.io publishing (if applicable)

---

## Success Metrics

### Phase 1 Success
- [ ] Low-level design document completed
- [ ] SDK packaging approach decided
- [ ] SDK boundaries clearly defined
- [ ] Developer persona documented
- [ ] Phase 2 scope defined and estimated

### Phase 2 Success (TBD)
- [ ] New developer can create a plugin in < 30 minutes
- [ ] Documentation enables self-service onboarding
- [ ] At least one example plugin demonstrating SDK usage

---

## Out of Scope

The following are **not** part of Milestone 8:

- Windows/Linux support (remains macOS-focused)
- crates.io publishing (may be Phase 3)
- Commercial licensing / marketplace
- Visual plugin builder / GUI designer
- Synth/instrument support (effects only)

---

## Dependencies

- **Milestone 6** (WebSocket IPC) ✅ — Enables browser-based UI development
- **Milestone 7** (Visual Testing) ✅ — Provides testing infrastructure

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Scope creep | High | Phase 1 research-only; Phase 2 scoped after |
| Packaging complexity | Medium | Start simple (template repo), iterate |
| Documentation debt | High | Budget documentation time explicitly |
| User expectations | Medium | Clear "alpha/beta SDK" messaging |

---

## Handoff

**Next Step:** Hand off to **Architect agent** to create low-level design.

*"Create low level design for VstKit Developer SDK, covering SDK packaging strategy, crate boundaries, and developer workflow. Reference user stories in `/docs/feature-specs/developer-sdk/user-stories.md`."*
