# Implementation Progress: wavecraft-core Crate Split

**Status:** Not Started  
**Last Updated:** 2025-01-XX  
**LLD:** [low-level-design-core-split.md](./low-level-design-core-split.md)  
**Plan:** [implementation-plan-core-split.md](./implementation-plan-core-split.md)

---

## Phase 1: Create wavecraft-nih_plug Crate

| Step | Task | Status | Notes |
|------|------|--------|-------|
| 1.1 | Create directory structure | ⬜ Not Started | |
| 1.2 | Create Cargo.toml | ⬜ Not Started | |
| 1.3 | Move editor/ directory | ⬜ Not Started | |
| 1.4 | Move params.rs | ⬜ Not Started | |
| 1.5 | Move util.rs | ⬜ Not Started | |
| 1.6 | Extract wavecraft_plugin! declarative macro | ⬜ Not Started | Use `$crate::__nih::` |
| 1.7 | Create `__nih` re-export module | ⬜ Not Started | |
| 1.8 | Create lib.rs (WavecraftPlugin struct) | ⬜ Not Started | |
| 1.9 | Create prelude.rs | ⬜ Not Started | |
| 1.10 | Update workspace Cargo.toml | ⬜ Not Started | |
| 1.11 | Verify build | ⬜ Not Started | `cargo build -p wavecraft-nih_plug` |

---

## Phase 2: Strip wavecraft-core

| Step | Task | Status | Notes |
|------|------|--------|-------|
| 2.1 | Remove nih_plug from dependencies | ⬜ Not Started | |
| 2.2 | Change crate-type to rlib only | ⬜ Not Started | |
| 2.3 | Remove platform-specific dependencies | ⬜ Not Started | wry, objc2, windows |
| 2.4 | Delete moved modules | ⬜ Not Started | editor/, params.rs, util.rs |
| 2.5 | Update macros.rs (keep only wavecraft_processor!) | ⬜ Not Started | |
| 2.6 | Update lib.rs | ⬜ Not Started | |
| 2.7 | Update prelude.rs | ⬜ Not Started | Remove nih_plug re-exports |

---

## Phase 3: Update Proc-Macro

| Step | Task | Status | Notes |
|------|------|--------|-------|
| 3.1 | Add `crate` field to parser | ⬜ Not Started | Optional, defaults to `::wavecraft_nih_plug` |
| 3.2 | Update generated code paths | ⬜ Not Started | `#krate::__nih::` |
| 3.3 | Update editor path | ⬜ Not Started | `#krate::editor::create_webview_editor` |
| 3.4 | Update proc-macro tests | ⬜ Not Started | |

---

## Phase 4: Update Template and Workspace

| Step | Task | Status | Notes |
|------|------|--------|-------|
| 4.1 | Add version specifiers to workspace deps | ⬜ Not Started | |
| 4.2 | Update plugin-template Cargo.toml | ⬜ Not Started | Single `wavecraft` dep |
| 4.3 | Update plugin-template lib.rs | ⬜ Not Started | `crate: wavecraft` |
| 4.4 | Update CLI template variables | ⬜ Not Started | If needed |
| 4.5 | Update documentation references | ⬜ Not Started | |

---

## Phase 5: Verification

| Step | Task | Status | Notes |
|------|------|--------|-------|
| 5.1 | Build full workspace | ⬜ Not Started | `cargo build --workspace` |
| 5.2 | Run all tests | ⬜ Not Started | `cargo test --workspace` |
| 5.3 | Dry-run publish wavecraft-core | ⬜ Not Started | **Primary goal** |
| 5.4 | Dry-run publish all publishable crates | ⬜ Not Started | |
| 5.5 | Test plugin in DAW | ⬜ Not Started | Manual: Ableton Live |

---

## Summary

| Phase | Total Steps | Completed | Remaining |
|-------|-------------|-----------|-----------|
| Phase 1: Create wavecraft-nih_plug | 11 | 0 | 11 |
| Phase 2: Strip wavecraft-core | 7 | 0 | 7 |
| Phase 3: Update Proc-Macro | 4 | 0 | 4 |
| Phase 4: Update Template/Workspace | 5 | 0 | 5 |
| Phase 5: Verification | 5 | 0 | 5 |
| **Total** | **32** | **0** | **32** |

---

## Blockers & Issues

_None yet_

---

## Change Log

| Date | Change |
|------|--------|
| 2025-01-XX | Created progress tracking document |
