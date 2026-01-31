# Semantic Versioning - User Stories

## Overview

Implement semantic versioning (SemVer) for VstKit plugins with a single source of truth in `Cargo.toml` that propagates to plugin metadata and UI.

---

## User Stories

### US-1: Version Visible in Plugin UI

**As a** music producer using a VstKit plugin  
**I want** to see the version number in the plugin UI  
**So that** I can verify I'm running the correct version when troubleshooting issues or checking for updates

#### Acceptance Criteria
- [ ] Version number is visible somewhere in the plugin UI
- [ ] Version follows SemVer format (e.g., `1.0.0`)
- [ ] Version updates automatically when a new build is released

#### Notes
- UI placement decision delegated to Architect
- Should be unobtrusive but findable

---

### US-2: Single Source of Truth in Cargo.toml

**As a** plugin developer  
**I want** the version to be defined in one place (`Cargo.toml`)  
**So that** I don't have to update multiple files when releasing a new version

#### Acceptance Criteria
- [ ] Version is defined in `engine/Cargo.toml` (workspace level) or `engine/crates/plugin/Cargo.toml`
- [ ] Plugin metadata (VST3, CLAP, AU) reads version from Cargo.toml
- [ ] UI receives version from the engine (not hardcoded in TypeScript)
- [ ] No manual version synchronization required across files

#### Notes
- nih-plug may already use Cargo.toml version for plugin metadata
- Need to verify how version flows to VST3/CLAP/AU formats

---

### US-3: Version in Plugin Metadata

**As a** DAW user  
**I want** the plugin version to appear in my DAW's plugin manager  
**So that** I can see which version is installed without opening the plugin

#### Acceptance Criteria
- [ ] VST3 plugin reports correct version to host
- [ ] CLAP plugin reports correct version to host
- [ ] AU plugin reports correct version to host (via clap-wrapper)
- [ ] Version matches what's displayed in the UI

#### Notes
- Most DAWs show plugin version in plugin info/manager
- Ableton Live shows version in plugin device header

---

### US-4: Build-Time Version Injection

**As a** plugin developer  
**I want** the version to be injected at build time  
**So that** the UI always shows the correct version without runtime IPC calls

#### Acceptance Criteria
- [ ] Version is available to UI at build time (compile-time constant or embedded asset)
- [ ] No IPC call required to fetch version (reduces startup latency)
- [ ] Version is baked into the bundled UI assets

#### Notes
- Could use Vite define/env variables injected during build
- Or embed version in a JSON file that's bundled with UI assets

---

## Out of Scope

- Pre-release tags (e.g., `-beta.1`, `-rc.1`) — may add later
- Automatic version bumping (manual process for now)
- Changelog generation
- Git tag automation

---

## Success Metrics

- Version visible in plugin UI within 1 second of plugin load
- Version matches across: Cargo.toml, plugin metadata, UI display
- Zero manual synchronization steps when bumping version

