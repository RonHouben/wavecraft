# Implementation Progress: Project Rename (VstKit → Wavecraft)

**Status:** 🚧 Not Started  
**Target Version:** 0.5.0  
**Branch:** `feature/project-rename-wavecraft`

---

## Progress Summary

| Phase | Status | Steps | Notes |
|-------|--------|-------|-------|
| Phase 1: Rust Crate Rename | ⏳ | 0/14 | |
| Phase 2: xtask Updates | ⏳ | 0/6 | |
| Phase 3: UI/TypeScript | ⏳ | 0/9 | |
| Phase 4: Template | ⏳ | 0/8 | |
| Phase 5: Documentation | ⏳ | 0/5 | |
| Phase 6: CI/CD | ⏳ | 0/2 | |
| Phase 7: Final | ⏳ | 0/2 | |
| Phase 8: GitHub Rename | ⏳ | 0/4 | Post-merge |

---

## Phase 1: Rust Crate Rename

- [ ] 1.1 Rename crate directories (`vstkit-*` → `wavecraft-*`)
- [ ] 1.2 Update workspace `engine/Cargo.toml`
- [ ] 1.3 Update `wavecraft-protocol/Cargo.toml`
- [ ] 1.4 Update `wavecraft-dsp/Cargo.toml`
- [ ] 1.5 Update `wavecraft-bridge/Cargo.toml`
- [ ] 1.6 Update `wavecraft-metering/Cargo.toml`
- [ ] 1.7 Update `wavecraft-core/Cargo.toml`
- [ ] 1.8 Update `wavecraft-core/src/lib.rs` (imports, structs)
- [ ] 1.9 Update `wavecraft-core/src/params.rs`
- [ ] 1.10 Update `wavecraft-core/src/macros.rs` (macro rename)
- [ ] 1.11 Update IPC JavaScript (`__VSTKIT_IPC__` → `__WAVECRAFT_IPC__`)
- [ ] 1.12 Update IPC Rust editor code
- [ ] 1.13 Update standalone crate
- [ ] 1.14 Update bridge crate source
- [ ] ✅ **Verify:** `cargo build --workspace`

---

## Phase 2: xtask Updates

- [ ] 2.1 Update lint command header
- [ ] 2.2 Update release command header
- [ ] 2.3 Update dev command header
- [ ] 2.4 Update mod command header
- [ ] 2.5 Update sign command bundle names
- [ ] 2.6 Update notarize command bundle names
- [ ] ✅ **Verify:** `cargo build --workspace`

---

## Phase 3: UI/TypeScript Updates

- [ ] 3.1 Rename `ui/src/lib/vstkit-ipc` → `wavecraft-ipc`
- [ ] 3.2 Update `ui/tsconfig.json`
- [ ] 3.3 Update `ui/vite.config.ts`
- [ ] 3.4 Update `ui/vitest.config.ts`
- [ ] 3.5 Update IPC library `index.ts`
- [ ] 3.6 Update IPC library `environment.ts`
- [ ] 3.7 Update IPC global references in transports
- [ ] 3.8 Update all component imports
- [ ] 3.9 Update test files
- [ ] ✅ **Verify:** `npm run build && npm run test`

---

## Phase 4: Template Updates

- [ ] 4.1 Rename `vstkit-plugin-template` → `wavecraft-plugin-template`
- [ ] 4.2 Update template `engine/Cargo.toml`
- [ ] 4.3 Update template engine source files
- [ ] 4.4 Rename template UI IPC directory
- [ ] 4.5 Update template `ui/vite.config.ts`
- [ ] 4.6 Update template UI components
- [ ] 4.7 Update template IPC library
- [ ] 4.8 Update template `README.md`
- [ ] ✅ **Verify:** Template builds

---

## Phase 5: Documentation Updates

- [ ] 5.1 Update main `README.md`
- [ ] 5.2 Update `docs/architecture/*.md`
- [ ] 5.3 Update `docs/guides/*.md`
- [ ] 5.4 Update `docs/backlog.md`
- [ ] 5.5 Update `.github/copilot-instructions.md` and skills

---

## Phase 6: CI/CD Updates

- [ ] 6.1 Update `.github/workflows/ci.yml`
- [ ] 6.2 Update `.github/workflows/release.yml`

---

## Phase 7: Final Verification

- [ ] 7.1 Bump version to 0.5.0 in `engine/Cargo.toml`
- [ ] 7.2 Full verification (all tests pass, no stray references)

---

## Phase 8: GitHub Rename (Post-Merge)

- [ ] 8.1 Merge feature branch to main
- [ ] 8.2 Rename repository on GitHub
- [ ] 8.3 Update local git remote
- [ ] 8.4 Verify redirect works

---

## Verification Commands

```bash
# Phase 1
cargo build --workspace
cargo test --workspace

# Phase 2
cargo build --workspace

# Phase 3
cd ui && npm run build && npm run test

# Phase 4
cd wavecraft-plugin-template && cargo build

# Phase 7 - Full check
cargo xtask lint
cargo xtask test

# Check for remaining references
grep -r "vstkit" --include="*.rs" --include="*.ts" . | grep -v "_archive" | grep -v "target/"
```

---

## Notes

_Add implementation notes here as work progresses._
