# Low-Level Design: Project Rename (VstKit → Wavecraft)

## Overview

This document provides the technical specification for renaming the project from "VstKit" to "Wavecraft". The rename affects multiple layers: Rust crates, TypeScript/npm aliases, documentation, CI/CD, and the GitHub repository.

**Related Documents:**
- [User Stories](./user-stories.md)
- [High-Level Design](../../architecture/high-level-design.md)
- [Coding Standards](../../architecture/coding-standards.md)

---

## 1. Scope of Changes

### 1.1 Name Variations

| Context | Old Name | New Name |
|---------|----------|----------|
| **Project** | VstKit | Wavecraft |
| **Rust crate names** | `vstkit-*` | `wavecraft-*` |
| **Rust module names** | `vstkit_*` | `wavecraft_*` |
| **npm aliases** | `@vstkit/*` | `@wavecraft/*` |
| **IPC global** | `__VSTKIT_IPC__` | `__WAVECRAFT_IPC__` |
| **Plugin bundles** | `vstkit.vst3` | `wavecraft.vst3` |
| **Config directory** | `~/.vstkit/` | `~/.wavecraft/` |
| **GitHub repo** | `RonHouben/vstkit` | `RonHouben/wavecraft` |

### 1.2 Files NOT to Change

The following should preserve historical "VstKit" references:

- `docs/roadmap.md` — Changelog entries are historical records
- `docs/feature-specs/_archive/**` — Archived specs are frozen
- Git commit history — Cannot be changed

---

## 2. Rust Crate Rename

### 2.1 Directory Renames

```
engine/crates/
├── vstkit-protocol/   →  wavecraft-protocol/
├── vstkit-dsp/        →  wavecraft-dsp/
├── vstkit-bridge/     →  wavecraft-bridge/
├── vstkit-metering/   →  wavecraft-metering/
├── vstkit-core/       →  wavecraft-core/
└── standalone/        →  standalone/  (no change)
```

**Command:**
```bash
cd engine/crates
mv vstkit-protocol wavecraft-protocol
mv vstkit-dsp wavecraft-dsp
mv vstkit-bridge wavecraft-bridge
mv vstkit-metering wavecraft-metering
mv vstkit-core wavecraft-core
```

### 2.2 Workspace Cargo.toml Changes

**File:** `engine/Cargo.toml`

| Section | Old | New |
|---------|-----|-----|
| `authors` | `["VstKit Team"]` | `["Wavecraft Team"]` |
| `workspace.dependencies` | `vstkit-protocol = { path = "crates/vstkit-protocol" }` | `wavecraft-protocol = { path = "crates/wavecraft-protocol" }` |
| `workspace.dependencies` | `vstkit-dsp = { path = "crates/vstkit-dsp" }` | `wavecraft-dsp = { path = "crates/wavecraft-dsp" }` |
| `workspace.dependencies` | `vstkit-bridge = { path = "crates/vstkit-bridge" }` | `wavecraft-bridge = { path = "crates/wavecraft-bridge" }` |
| `workspace.dependencies` | `vstkit-metering = { path = "crates/vstkit-metering" }` | `wavecraft-metering = { path = "crates/wavecraft-metering" }` |

### 2.3 Individual Crate Cargo.toml Changes

**For each crate, update:**

| Field | Pattern |
|-------|---------|
| `[package] name` | `vstkit-*` → `wavecraft-*` |
| `[lib] name` | `vstkit_*` → `wavecraft_*` |
| `description` | Replace "VstKit" with "Wavecraft" |
| `[dependencies]` | `vstkit-*.workspace = true` → `wavecraft-*.workspace = true` |

**Affected files:**
- `engine/crates/wavecraft-protocol/Cargo.toml`
- `engine/crates/wavecraft-dsp/Cargo.toml`
- `engine/crates/wavecraft-bridge/Cargo.toml`
- `engine/crates/wavecraft-metering/Cargo.toml`
- `engine/crates/wavecraft-core/Cargo.toml`

### 2.4 Rust Source Code Changes

#### 2.4.1 Import Statements

**Pattern:** `use vstkit_*::` → `use wavecraft_*::`

**Affected files:**
- `engine/crates/wavecraft-core/src/lib.rs`
- `engine/crates/wavecraft-core/src/params.rs`
- `engine/crates/wavecraft-core/src/editor/mod.rs`
- `engine/crates/wavecraft-bridge/src/lib.rs`
- `engine/crates/standalone/src/main.rs`

#### 2.4.2 Struct/Type Names

| Old | New | Location |
|-----|-----|----------|
| `VstKitPlugin` | `WavecraftPlugin` | `wavecraft-core/src/lib.rs` |
| `VstKitParams` | `WavecraftParams` | `wavecraft-core/src/params.rs` |

**Note:** These are internal types. The macro-generated plugin uses user-defined names.

#### 2.4.3 Macro Definition

**File:** `engine/crates/wavecraft-core/src/macros.rs`

| Old | New |
|-----|-----|
| `vstkit_plugin!` | `wavecraft_plugin!` |
| `[<__vstkit_exports_ $ident>]` | `[<__wavecraft_exports_ $ident>]` |
| Doc comments mentioning "vstkit_plugin!" | Update to "wavecraft_plugin!" |
| `"Generated plugin from vstkit_plugin!"` | `"Generated plugin from wavecraft_plugin!"` |

#### 2.4.4 IPC Global Object

**Files:**
- `engine/crates/wavecraft-core/src/editor/js/ipc-primitives-plugin.js`
- `engine/crates/wavecraft-core/src/editor/mod.rs`
- `engine/crates/wavecraft-core/src/editor/macos.rs`
- `engine/crates/wavecraft-core/src/editor/windows.rs`

| Old | New |
|-----|-----|
| `__VSTKIT_IPC__` | `__WAVECRAFT_IPC__` |
| `[VSTKIT_IPC]` (log prefix) | `[WAVECRAFT_IPC]` |

#### 2.4.5 Doc Comments

**Pattern:** Search and replace in doc comments:
- `//! VstKit` → `//! Wavecraft`
- `/// VstKit` → `/// Wavecraft`
- `# VstKit` (in doc examples) → `# Wavecraft`

---

## 3. xtask Command Updates

### 3.1 Print Headers

**Files and changes:**

| File | Old | New |
|------|-----|-----|
| `xtask/src/commands/lint.rs` | `print_header("VstKit Linting")` | `print_header("Wavecraft Linting")` |
| `xtask/src/commands/release.rs` | `print_header("VstKit Release Build")` | `print_header("Wavecraft Release Build")` |
| `xtask/src/commands/dev.rs` | `print_header("VstKit Development Servers")` | `print_header("Wavecraft Development Servers")` |
| `xtask/src/commands/mod.rs` | `print_header("VstKit Full Build Pipeline")` | `print_header("Wavecraft Full Build Pipeline")` |

### 3.2 Bundle Names

**Files:** `xtask/src/commands/sign.rs`, `xtask/src/commands/notarize.rs`

| Old | New |
|-----|-----|
| `"vstkit.vst3"` | `"wavecraft.vst3"` |
| `"vstkit.clap"` | `"wavecraft.clap"` |
| `"vstkit.component"` | `"wavecraft.component"` |
| `"vstkit-notarize.zip"` | `"wavecraft-notarize.zip"` |

---

## 4. UI / TypeScript Changes

### 4.1 Path Aliases

**Files:**
- `ui/tsconfig.json`
- `ui/vite.config.ts`
- `ui/vitest.config.ts`

| Old | New |
|-----|-----|
| `@vstkit/ipc` | `@wavecraft/ipc` |
| `@vstkit/ipc/meters` | `@wavecraft/ipc/meters` |
| `./src/lib/vstkit-ipc` | `./src/lib/wavecraft-ipc` |

### 4.2 Directory Rename

```
ui/src/lib/
├── vstkit-ipc/   →  wavecraft-ipc/
```

**Command:**
```bash
cd ui/src/lib
mv vstkit-ipc wavecraft-ipc
```

### 4.3 Import Statement Updates

**Pattern:** `from '@vstkit/ipc'` → `from '@wavecraft/ipc'`

**Affected files:**
- `ui/src/components/ParameterSlider.tsx`
- `ui/src/components/ParameterSlider.test.tsx`
- `ui/src/components/ParameterToggle.tsx`
- `ui/src/components/LatencyMonitor.tsx`
- `ui/src/components/ConnectionStatus.tsx`
- `ui/src/lib/audio-math.test.ts`
- `ui/src/test/mocks/ipc.ts`

### 4.4 IPC Library Internal Updates

**File:** `ui/src/lib/wavecraft-ipc/index.ts`

| Old | New |
|-----|-----|
| `@vstkit/ipc - IPC library for VstKit` | `@wavecraft/ipc - IPC library for Wavecraft` |

**File:** `ui/src/lib/wavecraft-ipc/environment.ts`

| Old | New |
|-----|-----|
| `globalThis.vstkit` (if referenced) | `globalThis.wavecraft` |

### 4.5 Global Object Reference

The TypeScript IPC bridge checks for the global `__VSTKIT_IPC__` object. Update:

**Files:**
- `ui/src/lib/wavecraft-ipc/IpcBridge.ts` (or similar)
- `ui/src/lib/wavecraft-ipc/transports/NativeTransport.ts` (if exists)

| Old | New |
|-----|-----|
| `globalThis.__VSTKIT_IPC__` | `globalThis.__WAVECRAFT_IPC__` |
| `window.__VSTKIT_IPC__` | `globalThis.__WAVECRAFT_IPC__` |

---

## 5. Template Project Updates

### 5.1 Directory Rename

```
vstkit-plugin-template/  →  wavecraft-plugin-template/
```

**Command:**
```bash
mv vstkit-plugin-template wavecraft-plugin-template
```

### 5.2 Template Cargo.toml Updates

**File:** `wavecraft-plugin-template/engine/Cargo.toml`

| Old | New |
|-----|-----|
| `vstkit-core = { path = "../../engine/crates/vstkit-core" }` | `wavecraft-core = { path = "../../engine/crates/wavecraft-core" }` |
| Similar for all vstkit-* dependencies | wavecraft-* |
| Comment: `# VstKit SDK dependencies` | `# Wavecraft SDK dependencies` |
| Comment: `# TODO: Switch to git dependencies when published` | Update URL reference |

### 5.3 Template UI Updates

**Files in `wavecraft-plugin-template/ui/`:**

- `vite.config.ts` — Update `@vstkit/*` aliases to `@wavecraft/*`
- `src/lib/vstkit-ipc/` → `src/lib/wavecraft-ipc/`
- `src/components/*.tsx` — Update imports

### 5.4 Template README

**File:** `wavecraft-plugin-template/README.md`

Update all "VstKit" references to "Wavecraft".

---

## 6. Documentation Updates

### 6.1 Search Patterns

Apply these find/replace patterns across all docs:

| Pattern | Replacement | Notes |
|---------|-------------|-------|
| `VstKit` | `Wavecraft` | Case-sensitive |
| `vstkit` | `wavecraft` | Lowercase (paths, URLs) |
| `VSTKIT` | `WAVECRAFT` | Constants |
| `vstkit_` | `wavecraft_` | Rust modules |
| `vstkit-` | `wavecraft-` | Crate names |
| `@vstkit/` | `@wavecraft/` | npm aliases |
| `~/.vstkit/` | `~/.wavecraft/` | Config paths |

### 6.2 Files to Update

**Core documentation:**
- `README.md`
- `docs/architecture/high-level-design.md`
- `docs/architecture/coding-standards.md`
- `docs/architecture/agent-development-flow.md`

**Guides:**
- `docs/guides/sdk-getting-started.md`
- `docs/guides/macos-signing.md`
- `docs/guides/visual-testing.md`
- `docs/guides/ci-pipeline.md`

**Roadmap (selective):**
- `docs/roadmap.md` — Update milestone 9 title and scope description
- DO NOT change changelog entries (historical accuracy)

**Backlog:**
- `docs/backlog.md` — Update remaining references

**Agent instructions:**
- `.github/copilot-instructions.md`
- `.github/skills/**/*.md`

### 6.3 README.md Specific Changes

Update:
- Project title and description
- Badge URLs (if any)
- Clone URL: `git clone https://github.com/RonHouben/wavecraft.git`
- Documentation links

---

## 7. CI/CD Updates

### 7.1 GitHub Workflows

**File:** `.github/workflows/ci.yml`

| Old | New |
|-----|-----|
| `vstkit-vst3-adhoc-signed` | `wavecraft-vst3-adhoc-signed` |
| `vstkit-clap-adhoc-signed` | `wavecraft-clap-adhoc-signed` |
| `engine/target/bundled/vstkit.vst3` | `engine/target/bundled/wavecraft.vst3` |
| `engine/target/bundled/vstkit.clap` | `engine/target/bundled/wavecraft.clap` |

**File:** `.github/workflows/release.yml`

| Old | New |
|-----|-----|
| `vstkit-macos` | `wavecraft-macos` |
| `vstkit.vst3` | `wavecraft.vst3` |
| `vstkit.clap` | `wavecraft.clap` |
| `vstkit.component` | `wavecraft.component` |

---

## 8. Plugin Metadata

The plugin exports define metadata visible in DAWs. These may need updating in `wavecraft-core/src/lib.rs`:

| Field | Old | New |
|-------|-----|-----|
| `NAME` | May contain "VstKit" | Update to "Wavecraft" or keep as plugin name |
| `VENDOR` | May be "VstKit" | Update to "Wavecraft" |
| `URL` | May reference vstkit | Update URL |

**Note:** Check if the default plugin implementation uses hardcoded names or if these are parameterized via the macro.

---

## 9. Implementation Order

Execute in this order to maintain build integrity:

### Phase 1: Rust Core (Must compile after each step)

1. **Rename crate directories** — `mv` commands
2. **Update workspace `Cargo.toml`** — Fix dependency paths
3. **Update individual `Cargo.toml` files** — Package names, lib names
4. **Verify:** `cargo check --workspace` (will fail on imports)
5. **Update Rust imports** — `use vstkit_*` → `use wavecraft_*`
6. **Update struct names** — `VstKitPlugin` → `WavecraftPlugin`, etc.
7. **Update macro** — `vstkit_plugin!` → `wavecraft_plugin!`
8. **Update IPC globals** — `__VSTKIT_IPC__` → `__WAVECRAFT_IPC__`
9. **Update xtask** — Print headers, bundle names
10. **Verify:** `cargo build --workspace` ✅

### Phase 2: UI

1. **Rename directory** — `ui/src/lib/vstkit-ipc` → `wavecraft-ipc`
2. **Update config files** — tsconfig, vite, vitest
3. **Update imports** — All `@vstkit/ipc` → `@wavecraft/ipc`
4. **Update IPC global references**
5. **Verify:** `npm run build && npm run test` ✅

### Phase 3: Template

1. **Rename directory** — `vstkit-plugin-template` → `wavecraft-plugin-template`
2. **Update Cargo.toml dependencies**
3. **Update UI config and imports**
4. **Verify:** Template compiles with `cargo build`

### Phase 4: Documentation & CI

1. **Apply find/replace across docs** (except changelog)
2. **Update CI workflow files**
3. **Verify:** CI pipeline passes

### Phase 5: Version Bump

1. **Update `engine/Cargo.toml`** — `version = "0.5.0"`
2. **Add changelog entry for rename**

### Phase 6: GitHub Repository Rename (Post-Merge)

1. **Merge feature branch** into main
2. **Rename repository** via GitHub settings
3. **Update local remote** — `git remote set-url origin ...`

---

## 10. Verification Checklist

```bash
# After Phase 1
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings

# After Phase 2
cd ui && npm run build && npm run test && npm run lint

# After Phase 3
cd wavecraft-plugin-template && cargo build

# Full pipeline
cargo xtask lint
cargo xtask test

# Search for remaining occurrences (should only be in changelog/archive)
grep -r "vstkit" --include="*.rs" --include="*.ts" --include="*.md" . | grep -v "_archive" | grep -v "CHANGELOG"
```

---

## 11. Risk Assessment

| Risk | Mitigation |
|------|------------|
| **Broken imports** | Run `cargo check` after each step; fix incrementally |
| **TypeScript type errors** | Run `npm run typecheck` after alias changes |
| **CI failures** | Feature branch CI will catch issues before merge |
| **Template breaks** | Template has its own `cargo build`; verify separately |
| **Missed references** | Final grep scan catches stragglers |
| **Git history loss** | Git tracks content, not filenames; history preserved |

---

## 12. Rollback Strategy

If issues arise mid-implementation:

1. **Git stash** current changes
2. **Reset** to last known good commit on feature branch
3. **Re-apply** changes incrementally with verification

The feature branch isolates all changes from main until merge.

---

## 13. Post-Rename Maintenance

### 13.1 External References

If any external documentation/links reference `vstkit`:
- GitHub creates automatic redirects for repository renames
- Update any bookmarks or external documentation manually

### 13.2 Future crates.io Publication

When publishing to crates.io:
1. Publish crates in dependency order: `protocol` → `dsp` → `bridge` → `metering` → `core`
2. Reserve `wavecraft` namespace early
3. Update template to use crates.io dependencies instead of paths

### 13.3 npm Publication

When publishing to npm:
1. Create `@wavecraft` organization on npmjs.com
2. Publish `@wavecraft/ipc` package
3. Update documentation with npm install instructions
