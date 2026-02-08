# Implementation Progress: CLI Self-Update

**Feature:** CLI Self-Update (`wavecraft update`)  
**Target Version:** `0.9.1`  
**Branch:** `feature/cli-self-update`

---

## Progress Tracker

### Phase 1: Restructure `update.rs` — Core Logic

- [x] **Step 1:** Add enums (`SelfUpdateResult`, `ProjectUpdateResult`) and `CURRENT_VERSION` constant
- [x] **Step 2:** Add `is_already_up_to_date()` helper
- [x] **Step 3:** Add `get_installed_version()` helper
- [x] **Step 4:** Add `update_cli()` function
- [x] **Step 5:** Refactor existing `run()` into `update_project_deps()`
- [x] **Step 6:** Add `print_summary()` function
- [x] **Step 7:** Write new `run()` orchestrator

### Phase 2: Update Help Text

- [x] **Step 8:** Update `Commands::Update` help text in `main.rs`

### Phase 3: Version Bump

- [x] **Step 9:** Bump CLI version to `0.9.1` in `cli/Cargo.toml`

### Phase 4: Unit Tests

- [x] **Step 10:** Add unit tests for `is_already_up_to_date()`
- [x] **Step 11:** Verify existing project detection tests still pass

### Phase 5: Integration Tests

- [x] **Step 12:** Update existing integration tests for new behavior
- [x] **Step 13:** Add integration test for update help long text

### Phase 6: Validation

- [x] **Step 14:** Run `cargo xtask ci-check` — all checks pass
- [x] **Step 15:** Manual verification (version flag, help text, outside/inside project)
