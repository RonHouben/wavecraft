# Implementation Progress: Rename `standalone` → `wavecraft-dev-server`

**Status:** Not Started  
**Last Updated:** 2026-02-06

---

## Progress Tracker

### Phase 1: Crate Rename (Atomic)

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 1.1 | Rename folder via `git mv` | ⬜ | |
| 1.2 | Update crate `Cargo.toml` (name, description, binary) | ⬜ | |
| 1.3 | Update workspace `Cargo.toml` | ⬜ | |
| 1.4 | Update `integration_test.rs` imports | ⬜ | |
| 1.5 | Update `latency_bench.rs` imports | ⬜ | |
| 1.6 | Verify Phase 1 build | ⬜ | |

### Phase 2: xtask & CLI Updates

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 2.1 | Update `xtask/commands/dev.rs` | ⬜ | |
| 2.2 | Update `cli/commands/start.rs` | ⬜ | |
| 2.3 | Verify Phase 2 functionality | ⬜ | |

### Phase 3: Source Comments & CLI Metadata

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 3.1 | Update `main.rs` doc comment | ⬜ | |
| 3.2 | Update CLI command metadata | ⬜ | |
| 3.3 | Update dev server log message | ⬜ | |
| 3.4 | Update GUI app log message | ⬜ | |
| 3.5 | Update `lib.rs` doc comment | ⬜ | |
| 3.6 | Update crate README | ⬜ | |

### Phase 4: Documentation Updates

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 4.1 | Update root `README.md` project structure | ⬜ | |
| 4.2 | Update `coding-standards.md` crate list | ⬜ | |
| 4.3 | Update `high-level-design.md` diagram | ⬜ | |
| 4.4 | Update `high-level-design.md` command example | ⬜ | |
| 4.5 | Update `roadmap.md` references | ⬜ | |
| 4.6 | Update `cli-start-command/user-stories.md` | ⬜ | |
| 4.7 | Update `cli-start-command/implementation-plan.md` | ⬜ | |
| 4.8 | Update `internal-testing/user-stories.md` | ⬜ | |
| 4.9 | Update `WebSocketTransport.ts` comments | ⬜ | |

### Phase 5: Final Verification

| Step | Description | Status | Notes |
|------|-------------|--------|-------|
| 5.1 | Full workspace build | ⬜ | |
| 5.2 | Full test suite (`cargo xtask check`) | ⬜ | |
| 5.3 | Manual dev server test | ⬜ | |
| 5.4 | Check help output | ⬜ | |

---

## Summary

| Phase | Steps | Completed | Progress |
|-------|-------|-----------|----------|
| Phase 1 | 6 | 0 | 0% |
| Phase 2 | 3 | 0 | 0% |
| Phase 3 | 6 | 0 | 0% |
| Phase 4 | 9 | 0 | 0% |
| Phase 5 | 4 | 0 | 0% |
| **Total** | **28** | **0** | **0%** |

---

## Notes

- Archived feature specs (`docs/feature-specs/_archive/*`) are NOT modified per project guidelines
- Phase 1 steps must be completed atomically to avoid build breakage
