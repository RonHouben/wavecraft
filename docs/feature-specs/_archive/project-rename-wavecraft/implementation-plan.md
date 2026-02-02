# Implementation Plan: Project Rename (VstKit → Wavecraft)

## Overview

This plan provides step-by-step instructions for renaming the project from VstKit to Wavecraft. Each step includes specific file paths, exact changes, and verification commands.

**Related Documents:**
- [User Stories](./user-stories.md)
- [Low-Level Design](./low-level-design-project-rename.md)

**Target Version:** `0.5.0`

---

## Phase 1: Rust Crate Rename

> **Goal:** Rename all 5 SDK crates and update the workspace.

### Step 1.1: Rename Crate Directories

**Action:** Rename crate directories from `vstkit-*` to `wavecraft-*`

```bash
cd engine/crates
mv vstkit-protocol wavecraft-protocol
mv vstkit-dsp wavecraft-dsp
mv vstkit-bridge wavecraft-bridge
mv vstkit-metering wavecraft-metering
mv vstkit-core wavecraft-core
```

**Verify:** `ls engine/crates/` shows `wavecraft-*` directories

---

### Step 1.2: Update Workspace Cargo.toml

**File:** `engine/Cargo.toml`

**Changes:**

| Line | Old | New |
|------|-----|-----|
| 9 | `authors = ["VstKit Team"]` | `authors = ["Wavecraft Team"]` |
| 17 | `vstkit-protocol = { path = "crates/vstkit-protocol" }` | `wavecraft-protocol = { path = "crates/wavecraft-protocol" }` |
| 18 | `vstkit-dsp = { path = "crates/vstkit-dsp" }` | `wavecraft-dsp = { path = "crates/wavecraft-dsp" }` |
| 19 | `vstkit-bridge = { path = "crates/vstkit-bridge" }` | `wavecraft-bridge = { path = "crates/wavecraft-bridge" }` |
| 21 | `vstkit-metering = { path = "crates/vstkit-metering" }` | `wavecraft-metering = { path = "crates/wavecraft-metering" }` |

---

### Step 1.3: Update wavecraft-protocol Cargo.toml

**File:** `engine/crates/wavecraft-protocol/Cargo.toml`

**Changes:**
- `name = "vstkit-protocol"` → `name = "wavecraft-protocol"`

---

### Step 1.4: Update wavecraft-dsp Cargo.toml

**File:** `engine/crates/wavecraft-dsp/Cargo.toml`

**Changes:**
- `name = "vstkit-dsp"` → `name = "wavecraft-dsp"`

---

### Step 1.5: Update wavecraft-bridge Cargo.toml

**File:** `engine/crates/wavecraft-bridge/Cargo.toml`

**Changes:**
- `name = "vstkit-bridge"` → `name = "wavecraft-bridge"`
- `vstkit-protocol = { path = "../vstkit-protocol" }` → `wavecraft-protocol = { path = "../wavecraft-protocol" }`

---

### Step 1.6: Update wavecraft-metering Cargo.toml

**File:** `engine/crates/wavecraft-metering/Cargo.toml`

**Changes:**
- `name = "vstkit-metering"` → `name = "wavecraft-metering"`

---

### Step 1.7: Update wavecraft-core Cargo.toml

**File:** `engine/crates/wavecraft-core/Cargo.toml`

**Changes:**
- `name = "vstkit-core"` → `name = "wavecraft-core"`
- `name = "vstkit_core"` (lib section) → `name = "wavecraft_core"`
- `description = "VstKit audio plugin..."` → `description = "Wavecraft audio plugin..."`
- `vstkit-protocol.workspace = true` → `wavecraft-protocol.workspace = true`
- `vstkit-dsp.workspace = true` → `wavecraft-dsp.workspace = true`
- `vstkit-metering.workspace = true` → `wavecraft-metering.workspace = true`
- `vstkit-bridge.workspace = true` → `wavecraft-bridge.workspace = true`

---

### Step 1.8: Update Rust Source Imports in wavecraft-core

**File:** `engine/crates/wavecraft-core/src/lib.rs`

**Changes:**
- Doc comment: `//! VstKit Core` → `//! Wavecraft Core`
- Doc comment: `//! ...for VstKit` → `//! ...for Wavecraft`
- `use vstkit_core::prelude::*` (in example) → `use wavecraft_core::prelude::*`
- `// The vstkit_plugin! macro...` → `// The wavecraft_plugin! macro...`
- `use vstkit_dsp::GainProcessor` → `use wavecraft_dsp::GainProcessor`
- `use vstkit_metering::...` → `use wavecraft_metering::...`
- `use crate::params::VstKitParams` → `use crate::params::WavecraftParams`
- `pub struct VstKitPlugin` → `pub struct WavecraftPlugin`
- `impl Default for VstKitPlugin` → `impl Default for WavecraftPlugin`
- `impl Plugin for VstKitPlugin` → `impl Plugin for WavecraftPlugin`
- `impl Vst3Plugin for VstKitPlugin` → `impl Vst3Plugin for WavecraftPlugin`
- `impl ClapPlugin for VstKitPlugin` → `impl ClapPlugin for WavecraftPlugin`
- `nih_export_vst3!(VstKitPlugin)` → `nih_export_vst3!(WavecraftPlugin)`
- `nih_export_clap!(VstKitPlugin)` → `nih_export_clap!(WavecraftPlugin)`

---

### Step 1.9: Update Params Module

**File:** `engine/crates/wavecraft-core/src/params.rs`

**Changes:**
- `pub struct VstKitParams` → `pub struct WavecraftParams`
- `impl Default for VstKitParams` → `impl Default for WavecraftParams`

---

### Step 1.10: Update Macro Definition

**File:** `engine/crates/wavecraft-core/src/macros.rs`

**Changes:**
- Doc: `/// \`vstkit_plugin!\`` → `/// \`wavecraft_plugin!\``
- Doc: all references to `vstkit_plugin!` → `wavecraft_plugin!`
- Doc: `vstkit_core::vstkit_plugin!` → `wavecraft_core::wavecraft_plugin!`
- `macro_rules! vstkit_plugin` → `macro_rules! wavecraft_plugin`
- `/// Generated plugin type by \`vstkit_plugin!\` macro` → `/// Generated plugin type by \`wavecraft_plugin!\` macro`
- `"Generated plugin from vstkit_plugin!"` → `"Generated plugin from wavecraft_plugin!"`
- `mod [<__vstkit_exports_ $ident>]` → `mod [<__wavecraft_exports_ $ident>]`
- Test code: `vstkit_plugin!` → `wavecraft_plugin!`
- Test code: `VstKitParams` → `WavecraftParams`

---

### Step 1.11: Update IPC Global in JavaScript

**File:** `engine/crates/wavecraft-core/src/editor/js/ipc-primitives-plugin.js`

**Changes:**
- `[VSTKIT_IPC]` → `[WAVECRAFT_IPC]` (all log prefixes)
- `window.__VSTKIT_IPC__` → `globalThis.__WAVECRAFT_IPC__`
- `console.log('[VSTKIT_IPC]` → `console.log('[WAVECRAFT_IPC]`

---

### Step 1.12: Update IPC Global in Rust Editor Code

**File:** `engine/crates/wavecraft-core/src/editor/mod.rs`

**Changes:**
- `globalThis.__VSTKIT_IPC__` → `globalThis.__WAVECRAFT_IPC__` (all occurrences)

**File:** `engine/crates/wavecraft-core/src/editor/macos.rs`

**Changes:**
- `globalThis.__VSTKIT_IPC__._receive` → `globalThis.__WAVECRAFT_IPC__._receive`

**File:** `engine/crates/wavecraft-core/src/editor/windows.rs`

**Changes:**
- `globalThis.__VSTKIT_IPC__._receive` → `globalThis.__WAVECRAFT_IPC__._receive`

---

### Step 1.13: Update Standalone Crate

**File:** `engine/crates/standalone/src/main.rs`

**Changes:**
- Any `use vstkit_*` imports → `use wavecraft_*`
- Any `VstKit` references in strings → `Wavecraft`

---

### Step 1.14: Update Bridge Crate Source

**File:** `engine/crates/wavecraft-bridge/src/lib.rs` (and other files)

**Changes:**
- `use vstkit_protocol` → `use wavecraft_protocol`

**Verify Phase 1:**
```bash
cd engine
cargo check --workspace
cargo build --workspace
cargo test --workspace
```

---

## Phase 2: xtask Command Updates

> **Goal:** Update build tool output messages and bundle names.

### Step 2.1: Update Lint Command

**File:** `engine/xtask/src/commands/lint.rs`

**Changes:**
- `print_header("VstKit Linting")` → `print_header("Wavecraft Linting")`

---

### Step 2.2: Update Release Command

**File:** `engine/xtask/src/commands/release.rs`

**Changes:**
- `print_header("VstKit Release Build")` → `print_header("Wavecraft Release Build")`

---

### Step 2.3: Update Dev Command

**File:** `engine/xtask/src/commands/dev.rs`

**Changes:**
- `print_header("VstKit Development Servers")` → `print_header("Wavecraft Development Servers")`

---

### Step 2.4: Update Mod Command

**File:** `engine/xtask/src/commands/mod.rs`

**Changes:**
- `print_header("VstKit Full Build Pipeline")` → `print_header("Wavecraft Full Build Pipeline")`

---

### Step 2.5: Update Sign Command

**File:** `engine/xtask/src/commands/sign.rs`

**Changes:**
- `"vstkit.vst3"` → `"wavecraft.vst3"` (all occurrences)
- `"vstkit.clap"` → `"wavecraft.clap"` (all occurrences)
- `"vstkit.component"` → `"wavecraft.component"` (all occurrences)

---

### Step 2.6: Update Notarize Command

**File:** `engine/xtask/src/commands/notarize.rs`

**Changes:**
- `"vstkit-notarize.zip"` → `"wavecraft-notarize.zip"`
- `"vstkit.vst3"` → `"wavecraft.vst3"` (all occurrences)
- `"vstkit.clap"` → `"wavecraft.clap"` (all occurrences)
- `"vstkit.component"` → `"wavecraft.component"` (all occurrences)

**Verify Phase 2:**
```bash
cargo build --workspace
cargo xtask --help  # Verify commands still work
```

---

## Phase 3: UI / TypeScript Updates

> **Goal:** Rename the IPC library and update all imports.

### Step 3.1: Rename IPC Library Directory

**Action:**
```bash
cd ui/src/lib
mv vstkit-ipc wavecraft-ipc
```

---

### Step 3.2: Update tsconfig.json

**File:** `ui/tsconfig.json`

**Changes:**
- `"@vstkit/ipc": ["./src/lib/vstkit-ipc"]` → `"@wavecraft/ipc": ["./src/lib/wavecraft-ipc"]`
- `"@vstkit/ipc/meters": ["./src/lib/vstkit-ipc/meters"]` → `"@wavecraft/ipc/meters": ["./src/lib/wavecraft-ipc/meters"]`

---

### Step 3.3: Update vite.config.ts

**File:** `ui/vite.config.ts`

**Changes:**
- `'@vstkit/ipc': path.resolve(__dirname, './src/lib/vstkit-ipc')` → `'@wavecraft/ipc': path.resolve(__dirname, './src/lib/wavecraft-ipc')`
- `'@vstkit/ipc/meters': path.resolve(__dirname, './src/lib/vstkit-ipc/meters')` → `'@wavecraft/ipc/meters': path.resolve(__dirname, './src/lib/wavecraft-ipc/meters')`

---

### Step 3.4: Update vitest.config.ts

**File:** `ui/vitest.config.ts`

**Changes:**
- `'@vstkit/ipc': path.resolve(__dirname, './src/lib/vstkit-ipc')` → `'@wavecraft/ipc': path.resolve(__dirname, './src/lib/wavecraft-ipc')`
- `'@vstkit/ipc/meters': path.resolve(__dirname, './src/lib/vstkit-ipc/meters')` → `'@wavecraft/ipc/meters': path.resolve(__dirname, './src/lib/wavecraft-ipc/meters')`

---

### Step 3.5: Update IPC Library Index

**File:** `ui/src/lib/wavecraft-ipc/index.ts`

**Changes:**
- `@vstkit/ipc - IPC library for VstKit` → `@wavecraft/ipc - IPC library for Wavecraft`

---

### Step 3.6: Update IPC Library Environment

**File:** `ui/src/lib/wavecraft-ipc/environment.ts`

**Changes:**
- `globalThis.vstkit` → `globalThis.wavecraft` (if present)
- Update any doc comments referencing VstKit

---

### Step 3.7: Update IPC Global References in Transport

**Files in `ui/src/lib/wavecraft-ipc/`:**

Search for and replace:
- `__VSTKIT_IPC__` → `__WAVECRAFT_IPC__`
- `globalThis.__VSTKIT_IPC__` → `globalThis.__WAVECRAFT_IPC__`

---

### Step 3.8: Update Component Imports

**File:** `ui/src/components/ParameterSlider.tsx`
- `from '@vstkit/ipc'` → `from '@wavecraft/ipc'`

**File:** `ui/src/components/ParameterSlider.test.tsx`
- `vi.mock('@vstkit/ipc'` → `vi.mock('@wavecraft/ipc'`
- Other imports from `@vstkit/ipc` → `@wavecraft/ipc`

**File:** `ui/src/components/ParameterToggle.tsx`
- `from '@vstkit/ipc'` → `from '@wavecraft/ipc'`

**File:** `ui/src/components/LatencyMonitor.tsx`
- `from '@vstkit/ipc'` → `from '@wavecraft/ipc'`

**File:** `ui/src/components/ConnectionStatus.tsx`
- `from '@vstkit/ipc'` → `from '@wavecraft/ipc'`

---

### Step 3.9: Update Test Files

**File:** `ui/src/lib/audio-math.test.ts`
- `from '@vstkit/ipc/meters'` → `from '@wavecraft/ipc/meters'`

**File:** `ui/src/test/mocks/ipc.ts`
- `from '@vstkit/ipc'` → `from '@wavecraft/ipc'`

**Verify Phase 3:**
```bash
cd ui
npm run typecheck
npm run build
npm run test
npm run lint
```

---

## Phase 4: Template Project Updates

> **Goal:** Update the plugin template to use Wavecraft naming.

### Step 4.1: Rename Template Directory

**Action:**
```bash
mv vstkit-plugin-template wavecraft-plugin-template
```

---

### Step 4.2: Update Template Engine Cargo.toml

**File:** `wavecraft-plugin-template/engine/Cargo.toml`

**Changes:**
- `# VstKit SDK dependencies` → `# Wavecraft SDK dependencies`
- `vstkit-core = { path = "../../engine/crates/vstkit-core" }` → `wavecraft-core = { path = "../../engine/crates/wavecraft-core" }`
- `vstkit-protocol = { path = "../../engine/crates/vstkit-protocol" }` → `wavecraft-protocol = { path = "../../engine/crates/wavecraft-protocol" }`
- `vstkit-dsp = { path = "../../engine/crates/vstkit-dsp" }` → `wavecraft-dsp = { path = "../../engine/crates/wavecraft-dsp" }`
- `vstkit-bridge = { path = "../../engine/crates/vstkit-bridge" }` → `wavecraft-bridge = { path = "../../engine/crates/wavecraft-bridge" }`
- `vstkit-metering = { path = "../../engine/crates/vstkit-metering" }` → `wavecraft-metering = { path = "../../engine/crates/wavecraft-metering" }`

---

### Step 4.3: Update Template Engine Source

**Files in `wavecraft-plugin-template/engine/src/`:**

- `use vstkit_core` → `use wavecraft_core`
- `vstkit_plugin!` → `wavecraft_plugin!` (if used)

---

### Step 4.4: Rename Template UI IPC Directory

**Action:**
```bash
cd wavecraft-plugin-template/ui/src/lib
mv vstkit-ipc wavecraft-ipc
```

---

### Step 4.5: Update Template UI vite.config.ts

**File:** `wavecraft-plugin-template/ui/vite.config.ts`

**Changes:**
- `'@vstkit/ipc': path.resolve(__dirname, './src/lib/vstkit-ipc')` → `'@wavecraft/ipc': path.resolve(__dirname, './src/lib/wavecraft-ipc')`
- `'@vstkit/ipc/meters': path.resolve(__dirname, './src/lib/vstkit-ipc/meters')` → `'@wavecraft/ipc/meters': path.resolve(__dirname, './src/lib/wavecraft-ipc/meters')`

---

### Step 4.6: Update Template UI Components

**File:** `wavecraft-plugin-template/ui/src/components/ParameterSlider.tsx`
- `from '@vstkit/ipc'` → `from '@wavecraft/ipc'`

**File:** `wavecraft-plugin-template/ui/src/components/LatencyMonitor.tsx`
- `from '@vstkit/ipc'` → `from '@wavecraft/ipc'`

---

### Step 4.7: Update Template UI IPC Library

**File:** `wavecraft-plugin-template/ui/src/lib/wavecraft-ipc/index.ts`
- `@vstkit/ipc - IPC library for VstKit` → `@wavecraft/ipc - IPC library for Wavecraft`

---

### Step 4.8: Update Template README

**File:** `wavecraft-plugin-template/README.md`

- Replace all `VstKit` → `Wavecraft`
- Replace all `vstkit` → `wavecraft`

**Verify Phase 4:**
```bash
cd wavecraft-plugin-template
cargo build
cd ui && npm install && npm run build
```

---

## Phase 5: Documentation Updates

> **Goal:** Update all documentation to use Wavecraft naming.

### Step 5.1: Update Main README

**File:** `README.md`

- Replace all `VstKit` → `Wavecraft`
- Replace all `vstkit` → `wavecraft`
- Update clone URL: `git clone https://github.com/RonHouben/wavecraft.git`

---

### Step 5.2: Update Architecture Docs

**File:** `docs/architecture/high-level-design.md`
- Replace all `VstKit` → `Wavecraft`
- Replace all `vstkit` → `wavecraft`
- Replace all `@vstkit/` → `@wavecraft/`
- Replace all `~/.vstkit/` → `~/.wavecraft/`

**File:** `docs/architecture/coding-standards.md`
- Replace all `VstKit` → `Wavecraft`
- Replace all `vstkit` → `wavecraft`
- Replace all `@vstkit/` → `@wavecraft/`

**File:** `docs/architecture/agent-development-flow.md`
- Replace all `VstKit` → `Wavecraft` (if present)

---

### Step 5.3: Update Guides

**File:** `docs/guides/sdk-getting-started.md`
- Replace all `VstKit` → `Wavecraft`
- Replace all `vstkit` → `wavecraft`
- Replace all `vstkit_plugin!` → `wavecraft_plugin!`
- Replace all `vstkit-*` crate names → `wavecraft-*`

**File:** `docs/guides/macos-signing.md`
- Replace `vstkit.vst3` → `wavecraft.vst3`
- Replace `vstkit.clap` → `wavecraft.clap`
- Replace `vstkit.component` → `wavecraft.component`

**File:** `docs/guides/visual-testing.md`
- Replace all `~/.vstkit/` → `~/.wavecraft/`
- Replace all `VstKit` → `Wavecraft`

**File:** `docs/guides/ci-pipeline.md`
- Replace all `VstKit` → `Wavecraft`
- Replace all `vstkit` → `wavecraft`

---

### Step 5.4: Update Backlog

**File:** `docs/backlog.md`
- Update any remaining `VstKit` references (already partially done)

---

### Step 5.5: Update Agent Instructions

**File:** `.github/copilot-instructions.md`
- Replace all `VstKit` → `Wavecraft`
- Replace all `vstkit` → `wavecraft`
- Replace all `@vstkit/` → `@wavecraft/`

**Files in `.github/skills/`:**
- Search and replace `VstKit` → `Wavecraft` in all `.md` files

---

## Phase 6: CI/CD Updates

> **Goal:** Update GitHub Actions workflows.

### Step 6.1: Update CI Workflow

**File:** `.github/workflows/ci.yml`

**Changes:**
- `vstkit-vst3-adhoc-signed` → `wavecraft-vst3-adhoc-signed`
- `vstkit-clap-adhoc-signed` → `wavecraft-clap-adhoc-signed`
- `engine/target/bundled/vstkit.vst3` → `engine/target/bundled/wavecraft.vst3`
- `engine/target/bundled/vstkit.clap` → `engine/target/bundled/wavecraft.clap`

---

### Step 6.2: Update Release Workflow

**File:** `.github/workflows/release.yml`

**Changes:**
- `vstkit-macos` → `wavecraft-macos`
- `vstkit.vst3` → `wavecraft.vst3`
- `vstkit.clap` → `wavecraft.clap`
- `vstkit.component` → `wavecraft.component`

**Verify Phase 6:**
```bash
# Syntax check workflows
cat .github/workflows/ci.yml | head -50
cat .github/workflows/release.yml | head -50
```

---

## Phase 7: Version Bump & Final Verification

### Step 7.1: Bump Version

**File:** `engine/Cargo.toml`

**Change:**
- `version = "0.4.0"` → `version = "0.5.0"`

---

### Step 7.2: Full Verification

```bash
# Rust compilation and tests
cd engine
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings

# UI build and tests
cd ../ui
npm run typecheck
npm run build
npm run test
npm run lint

# Template build
cd ../wavecraft-plugin-template
cargo build

# Full pipeline
cd ..
cargo xtask lint
cargo xtask test

# Check for remaining occurrences (expect only changelog/archive)
grep -r "vstkit" --include="*.rs" --include="*.ts" --include="*.tsx" . 2>/dev/null | grep -v "_archive" | grep -v "target/" | grep -v "node_modules/"
grep -r "VstKit" --include="*.md" . 2>/dev/null | grep -v "_archive" | grep -v "roadmap.md"
```

---

## Phase 8: GitHub Repository Rename (Post-Merge)

> **Important:** Execute this phase ONLY after the feature branch is merged to main.

### Step 8.1: Merge Feature Branch

```bash
git checkout main
git pull origin main
git merge feature/project-rename-wavecraft
git push origin main
```

### Step 8.2: Rename Repository on GitHub

1. Go to https://github.com/RonHouben/vstkit/settings
2. Scroll to "Repository name"
3. Change `vstkit` to `wavecraft`
4. Click "Rename"

### Step 8.3: Update Local Remote

```bash
git remote set-url origin https://github.com/RonHouben/wavecraft.git
git fetch origin
```

### Step 8.4: Verify Redirect

- Visit https://github.com/RonHouben/vstkit
- Should redirect to https://github.com/RonHouben/wavecraft

---

## Implementation Checklist

### Phase 1: Rust Crate Rename
- [ ] 1.1 Rename crate directories
- [ ] 1.2 Update workspace Cargo.toml
- [ ] 1.3 Update wavecraft-protocol Cargo.toml
- [ ] 1.4 Update wavecraft-dsp Cargo.toml
- [ ] 1.5 Update wavecraft-bridge Cargo.toml
- [ ] 1.6 Update wavecraft-metering Cargo.toml
- [ ] 1.7 Update wavecraft-core Cargo.toml
- [ ] 1.8 Update wavecraft-core/src/lib.rs
- [ ] 1.9 Update wavecraft-core/src/params.rs
- [ ] 1.10 Update wavecraft-core/src/macros.rs
- [ ] 1.11 Update IPC JavaScript
- [ ] 1.12 Update IPC Rust editor code
- [ ] 1.13 Update standalone crate
- [ ] 1.14 Update bridge crate source
- [ ] **Verify:** `cargo build --workspace`

### Phase 2: xtask Updates
- [ ] 2.1 Update lint command
- [ ] 2.2 Update release command
- [ ] 2.3 Update dev command
- [ ] 2.4 Update mod command
- [ ] 2.5 Update sign command
- [ ] 2.6 Update notarize command
- [ ] **Verify:** `cargo build --workspace`

### Phase 3: UI/TypeScript Updates
- [ ] 3.1 Rename IPC library directory
- [ ] 3.2 Update tsconfig.json
- [ ] 3.3 Update vite.config.ts
- [ ] 3.4 Update vitest.config.ts
- [ ] 3.5 Update IPC library index
- [ ] 3.6 Update IPC library environment
- [ ] 3.7 Update IPC global references
- [ ] 3.8 Update component imports
- [ ] 3.9 Update test files
- [ ] **Verify:** `npm run build && npm run test`

### Phase 4: Template Updates
- [ ] 4.1 Rename template directory
- [ ] 4.2 Update template Cargo.toml
- [ ] 4.3 Update template engine source
- [ ] 4.4 Rename template UI IPC directory
- [ ] 4.5 Update template vite.config.ts
- [ ] 4.6 Update template components
- [ ] 4.7 Update template IPC library
- [ ] 4.8 Update template README
- [ ] **Verify:** `cargo build` in template

### Phase 5: Documentation Updates
- [ ] 5.1 Update main README
- [ ] 5.2 Update architecture docs
- [ ] 5.3 Update guides
- [ ] 5.4 Update backlog
- [ ] 5.5 Update agent instructions

### Phase 6: CI/CD Updates
- [ ] 6.1 Update CI workflow
- [ ] 6.2 Update release workflow

### Phase 7: Final
- [ ] 7.1 Bump version to 0.5.0
- [ ] 7.2 Full verification
- [ ] Commit and push

### Phase 8: GitHub Rename (Post-Merge)
- [ ] 8.1 Merge to main
- [ ] 8.2 Rename repository
- [ ] 8.3 Update local remote
- [ ] 8.4 Verify redirect

---

## Estimated Effort

| Phase | Estimated Time | Risk |
|-------|---------------|------|
| Phase 1 | 45-60 min | Medium (most changes) |
| Phase 2 | 15 min | Low |
| Phase 3 | 20-30 min | Low |
| Phase 4 | 20 min | Low |
| Phase 5 | 30-45 min | Low |
| Phase 6 | 10 min | Low |
| Phase 7 | 15 min | Low |
| Phase 8 | 5 min | Low |
| **Total** | **~3 hours** | |

---

## Rollback Procedure

If issues arise at any phase:

```bash
# Stash current changes
git stash

# Reset to last commit
git reset --hard HEAD

# Or reset to specific commit
git log --oneline  # Find good commit
git reset --hard <commit-sha>

# Pop stash if needed
git stash pop
```

For post-merge issues, the GitHub redirect ensures old URLs continue working.
