# Implementation Progress: CLI `--local-dev` Flag

**Feature:** CI Local Development Dependencies  
**Started:** 2026-02-04  
**Status:** Not Started

---

## Progress Tracker

### Phase 1: CLI Argument
| Step | Task | Status | Notes |
|------|------|--------|-------|
| 1.1 | Add `--local-dev` argument to `main.rs` | ⬜ Not Started | |
| 1.2 | Update `NewCommand` struct in `new.rs` | ⬜ Not Started | |
| 1.3 | Pass `local_dev` in main.rs match arm | ⬜ Not Started | |

### Phase 2: Template Variables
| Step | Task | Status | Notes |
|------|------|--------|-------|
| 2.1 | Add `local_dev` to `TemplateVariables` struct | ⬜ Not Started | |
| 2.2 | Update `TemplateVariables::new()` constructor | ⬜ Not Started | |
| 2.3 | Update call site in `new.rs` | ⬜ Not Started | |

### Phase 3: Post-Processing Logic
| Step | Task | Status | Notes |
|------|------|--------|-------|
| 3.1 | Add `apply_local_dev_overrides()` function | ⬜ Not Started | |
| 3.2 | Call function in `extract_dir()` | ⬜ Not Started | |

### Phase 4: Unit Tests
| Step | Task | Status | Notes |
|------|------|--------|-------|
| 4.1 | Add test for `apply_local_dev_overrides()` | ⬜ Not Started | |
| 4.2 | Update existing `TemplateVariables` tests | ⬜ Not Started | |

### Phase 5: CI Workflow
| Step | Task | Status | Notes |
|------|------|--------|-------|
| 5.1 | Update `template-validation.yml` | ⬜ Not Started | |

---

## Summary

| Phase | Tasks | Completed | Progress |
|-------|-------|-----------|----------|
| Phase 1: CLI Argument | 3 | 0 | 0% |
| Phase 2: Template Variables | 3 | 0 | 0% |
| Phase 3: Post-Processing | 2 | 0 | 0% |
| Phase 4: Unit Tests | 2 | 0 | 0% |
| Phase 5: CI Workflow | 1 | 0 | 0% |
| **Total** | **11** | **0** | **0%** |

---

## Blockers

None identified.

---

## Completion Log

| Date | Step | Notes |
|------|------|-------|
| - | - | - |
