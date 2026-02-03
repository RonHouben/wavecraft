# PR Summary: Project Rename (VstKit → Wavecraft)

## Summary

This PR completes **Milestone 9: Project Rename**, rebranding the entire project from "VstKit" to "Wavecraft" to avoid potential "VST" trademark concerns before open-source release. This is a **breaking change** that updates all public-facing names, APIs, and identifiers across the codebase.

**Version:** 0.4.0 → **0.5.0** (minor version bump per SemVer)

**Key Changes:**
- 5 SDK crates renamed: `vstkit-*` → `wavecraft-*`
- Macro renamed: `vstkit_plugin!` → `wavecraft_plugin!`
- npm aliases updated: `@vstkit/*` → `@wavecraft/*`
- IPC global object: `__VSTKIT_IPC__` → `__WAVECRAFT_IPC__`
- Bundle names: `vstkit.vst3` → `wavecraft-core.vst3`
- Template project: `vstkit-plugin-template` → `wavecraft-plugin-template`

**Impact:** 162 files changed, 3339 insertions, 613 deletions

## Changes by Area

### 🦀 Engine (Rust)

**Crate Renames:**
- `engine/crates/vstkit-protocol/` → `engine/crates/wavecraft-protocol/`
- `engine/crates/vstkit-dsp/` → `engine/crates/wavecraft-dsp/`
- `engine/crates/vstkit-bridge/` → `engine/crates/wavecraft-bridge/`
- `engine/crates/vstkit-metering/` → `engine/crates/wavecraft-metering/`
- `engine/crates/vstkit-core/` → `engine/crates/wavecraft-core/`

**Workspace Updates:**
- `engine/Cargo.toml`: Updated authors to "Wavecraft Team", version to 0.5.0
- All `Cargo.toml` files: Updated package names, lib names, dependencies
- All `Cargo.lock` files: Regenerated with new crate names

**Source Code:**
- Import statements: `use vstkit_*::` → `use wavecraft_*::`
- Macro definition: `vstkit_plugin!` → `wavecraft_plugin!`
- IPC global: `__VSTKIT_IPC__` → `__WAVECRAFT_IPC__` (in JS injection code)
- Doc comments: Updated all references to Wavecraft

**xtask Build System:**
- Print headers: "VstKit build system" → "Wavecraft build system"
- Bundle names: `vstkit.vst3` → `wavecraft-core.vst3`

### ⚛️ UI (TypeScript/React)

**npm Package:**
- `package.json`: `@vstkit/ui` → `@wavecraft/ui`

**IPC Directory Rename:**
- `ui/src/lib/vstkit-ipc/` → `ui/src/lib/wavecraft-ipc/`
- All 16 files moved to new directory

**TypeScript Configuration:**
- `tsconfig.json`: Path alias `@vstkit/ipc` → `@wavecraft/ipc`
- `vite.config.ts`: Resolve alias updated
- `vitest.config.ts`: Module name mapper updated

**Source Code:**
- Import statements: `from '@vstkit/ipc'` → `from '@wavecraft/ipc'`
- Environment detection: `__VSTKIT_IPC__` → `__WAVECRAFT_IPC__`
- Type definitions: Updated all IPC type references

### 🎯 Template Project

**Directory Rename:**
- `vstkit-plugin-template/` → `wavecraft-plugin-template/`

**Template Updates:**
- `README.md`: Updated branding, renamed directory references
- `engine/Cargo.toml`: Updated dependencies to `wavecraft-*` crates
- `engine/src/lib.rs`: Updated macro usage `wavecraft_plugin!`, imports
- `ui/`: Full IPC directory rename and configuration updates
- All template files now demonstrate Wavecraft SDK usage

**Key Fixes (from testing):**
- VST3_CLASS_ID: Fixed length from 19 bytes to 16 bytes (`WavecraftPlugin0`)
- Removed direct `nih_plug` imports, using prelude re-exports
- Updated TypeScript path aliases in configs

### 📦 Packaging

**AU Wrapper (macOS):**
- `packaging/macos/au-wrapper/CMakeLists.txt`:
  - Project name: `VstKit-AUWrapper` → `Wavecraft-AUWrapper`
  - All variables: `VSTKIT_*` → `WAVECRAFT_*`
  - Bundle identifier: `dev.vstkit.vstkit` → `dev.wavecraft.wavecraft`
  - Manufacturer: "VstKit Team" → "Wavecraft Team"
  - CLAP path: `vstkit.clap` → `wavecraft-core.clap`
  - Version: 0.1.0 → 0.5.0

### 🔧 CI/CD

**GitHub Actions:**
- `.github/workflows/ci.yml`: Artifact names `vstkit-*` → `wavecraft-*`
- `.github/workflows/release.yml`: Release artifact names updated

### 📚 Documentation

**Architecture:**
- `docs/architecture/high-level-design.md`: SDK distribution diagram, all references
- `docs/architecture/coding-standards.md`: Examples updated

**Guides:**
- `docs/guides/sdk-getting-started.md`: Template name, crate names
- `docs/guides/visual-testing.md`: Package references
- `docs/guides/ci-pipeline.md`: Artifact names
- `docs/guides/macos-signing.md`: Bundle names

**Project Docs:**
- `README.md`: Title, description, all branding references
- `docs/roadmap.md`: Updated to 100% complete, M9 marked complete, archived
- `docs/backlog.md`: Updated references

**Agent/Skill Docs:**
- `.github/agents/*.agent.md`: Updated crate/package references
- `.github/skills/*.md`: Updated examples

### ✅ Quality Assurance

**Testing:**
- 24/24 manual test cases executed and passing
- All automated tests passing (35 UI tests, 111+ engine tests)
- Template compilation verified (engine + UI)
- Bundle generation verified (VST3 + CLAP)

**QA Review:**
- All linting checks passed (Clippy, ESLint, Prettier, TypeScript)
- 5 issues found and resolved during testing
- Architectural review completed
- Domain separation verified (DSP layer clean)

## Commits

```
a734402 docs: mark Milestone 9 complete, archive feature spec
6da29a3 docs: fix remaining VSTKIT reference in SDK distribution diagram
b19cfaf docs: update QA report - all findings resolved
4d027b7 fix: update AU wrapper to use Wavecraft naming (Finding #1)
7cb2df9 docs: add QA report - approved with one minor finding
dfd6887 docs: update test plan - all issues resolved, ready for QA
50cc66e fix: resolve all remaining rename issues (Issues #2-5)
5291800 style: format import statements for better readability
fb100ae fix: correct VST3_CLASS_ID length in template (16 bytes)
2dc2fd5 fix: remove direct nih_plug usage from template, use prelude re-exports
fa39532 docs: mark implementation progress as complete
0b5dbe0 feat: rename project from VstKit to Wavecraft (v0.5.0)
4b57b69 docs: create low-level design for project rename
81fd5a0 docs: create user stories for project rename
```

## Related Documentation

All feature documentation is archived in `docs/feature-specs/_archive/project-rename-wavecraft/`:

- **[User Stories](user-stories.md)** — 9 user stories covering all rename aspects
- **[Low-Level Design](low-level-design-project-rename.md)** — 13-section technical spec
- **[Implementation Plan](implementation-plan.md)** — 8 phases, 50 steps
- **[Implementation Progress](implementation-progress.md)** — Phase-by-phase tracking
- **[Test Plan](test-plan.md)** — 24 manual test cases with results
- **[QA Report](QA-report.md)** — Quality review, all findings resolved

## Testing Performed

### Automated Tests
- ✅ **Engine tests**: All passing (`cargo test --workspace`)
- ✅ **UI tests**: 35/35 passing (Vitest)
- ✅ **Linting**: All checks passed (Clippy, ESLint, Prettier, TypeScript)

### Manual Tests (24/24 Passing)
- ✅ **TC-001-TC-005**: Compilation (engine, UI, template, full build, linting)
- ✅ **TC-006-TC-008**: Testing (unit tests, imports, no old names)
- ✅ **TC-009-TC-011**: Documentation (README, guides, arch docs)
- ✅ **TC-012-TC-014**: Bundles (names, metadata, paths)
- ✅ **TC-015-TC-017**: Template (compilation, IPC, VST3_CLASS_ID)
- ✅ **TC-018-TC-021**: CI/CD (workflows, artifacts)
- ✅ **TC-022-TC-024**: Integration (DAW load, UI, parameters)

### QA Findings (All Resolved)
1. ✅ **Issue #1**: Template VST3_CLASS_ID length (fixed to 16 bytes)
2. ✅ **Issue #2**: Main README branding (fixed)
3. ✅ **Issue #3**: Template IPC directory naming (fixed)
4. ✅ **Issue #4**: Template IPC global object (fixed)
5. ✅ **Issue #5**: Template TypeScript configs (fixed)
6. ✅ **Finding #1**: AU wrapper naming (fixed)

## Breaking Changes

⚠️ **This is a breaking change for any code using the VstKit SDK.**

**Migration Required:**
1. Update `Cargo.toml` dependencies: `vstkit-*` → `wavecraft-*`
2. Update import statements: `use vstkit_*::` → `use wavecraft_*::`
3. Update macro calls: `vstkit_plugin!` → `wavecraft_plugin!`
4. Update TypeScript imports: `@vstkit/*` → `@wavecraft/*`
5. Update IPC directory: `vstkit-ipc` → `wavecraft-ipc`
6. Update TypeScript path aliases in configs

**Note:** The template (`wavecraft-plugin-template`) has been fully updated to demonstrate the new API.

## Post-Merge Tasks

- [ ] Rename GitHub repository from `vstkit` to `wavecraft` (optional, creates redirect)
- [ ] Update any external references or links
- [ ] Consider registering `wavecraft.dev` domain (currently available)

## Checklist

- [x] All commits follow conventional commit format
- [x] All automated tests passing
- [x] Manual testing completed (24/24 test cases)
- [x] Documentation updated (README, guides, architecture docs)
- [x] Breaking changes documented
- [x] Version bumped appropriately (0.4.0 → 0.5.0)
- [x] QA review completed and approved
- [x] Architect review completed
- [x] Feature spec archived
- [x] Roadmap updated

---

**Ready to merge:** This PR completes all 9 project milestones. Wavecraft v0.5.0 is ready for open-source release! 🎉
