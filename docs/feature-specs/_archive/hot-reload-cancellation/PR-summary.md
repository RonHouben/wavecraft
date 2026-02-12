# PR Summary: Fix Hot-Reload Hang and Add Cancellation Support

## Summary

This PR fixes a critical hot-reload hang issue that occurred when adding parameters to `SignalChain`. The hang manifested during parameter extraction, timing out after 30 seconds. The root cause was `OnceLock` initialization blocking on macOS during `dlopen()` in the subprocess. Additionally, this PR implements proper cancellation support for parameter extraction to prevent hanging when new file changes occur during rebuild.

**Key improvements:**
- Replaced `OnceLock` with direct `Box::leak()` allocation in `ChainParams::param_specs()` to eliminate macOS hang
- Added cancellation mechanism (`CancellationToken`) to parameter extraction process
- Implemented integration tests to verify cancellation behavior
- Enhanced error handling in rebuild workflow

## Changes

### Engine/DSP
- **engine/crates/wavecraft-dsp/src/combinators/chain.rs**: Refactored `ChainParams::param_specs()` to use `Box::leak()` instead of `OnceLock` to prevent deadlock on macOS during hot-reload subprocess parameter extraction
- **engine/crates/wavecraft-dsp/tests/chain_param_extraction.rs**: Added integration test to verify parameter extraction works correctly with the new implementation

### Dev Server
- **dev-server/src/reload/rebuild.rs**: Implemented cancellation support in rebuild workflow, added `CancellationToken` to prevent hanging when new changes arrive during parameter extraction
- **dev-server/tests/reload_cancellation.rs**: Added comprehensive integration tests for cancellation scenarios (148 lines)
- **dev-server/Cargo.toml** & **Cargo.lock**: Added `tokio-util` dependency for cancellation token support

### Build/Config
- **engine/xtask/src/commands/validate_cli_deps.rs**: Minor updates (8 lines)

### Documentation
- **docs/feature-specs/hot-reload-cancellation/hotreload-hang-fix-summary.md**: Detailed technical analysis of the hang issue, root cause, and solution trade-offs (123 lines)
- **docs/feature-specs/hot-reload-cancellation/test-plan-hotreload-cancellation.md**: Comprehensive test plan for cancellation and subprocess-based extraction (237 lines)
- **docs/backlog.md**: Updated backlog with refactorings and future work items (120 lines modified)

## Commits

```
3ddfa2b feat(hot-reload): implement parameter extraction cancellation and add tests
9123e16 docs(hot-reload): add test plans for cancellation and subprocess-based extraction
```

## Detailed Commit Messages

**3ddfa2b** - feat(hot-reload): implement parameter extraction cancellation and add tests
- Introduced cancellation mechanism for parameter extraction during hot-reload to prevent hanging
- Updated `Chain` combinator to avoid using `OnceLock` to prevent deadlocks on macOS
- Added integration tests to verify cancellation behavior and ensure parameter extraction completes correctly
- Enhanced `Cargo.toml` and `Cargo.lock` with new dependencies for testing

**9123e16** - docs(hot-reload): add test plans for cancellation and subprocess-based extraction

## Related Documentation

- [Hot-Reload Hang Fix Summary](./hotreload-hang-fix-summary.md) — Technical analysis of the OnceLock hang issue
- [Test Plan: Hot-Reload Cancellation](./test-plan-hotreload-cancellation.md) — Comprehensive test scenarios

## Testing

### Automated Tests
- [x] Build passes: `cargo build` (dev-server crate)
- [x] New integration tests pass:
  - `test_param_extraction_cancelled_on_new_change` — Verifies cancellation works when new file changes occur
  - `chain_param_extraction` — Verifies parameter extraction from Chain combinator
- [x] Existing tests pass (no regressions)

### Manual Testing
- [x] Hot-reload no longer hangs at "Loading parameters" step
- [x] Adding `AnotherGain` to `SignalChain` completes successfully
- [x] Parameter extraction completes without 30s timeout
- [x] Cancellation triggers correctly when file changes during rebuild

### Test Coverage
| Area | Test Type | Status |
|------|-----------|--------|
| Parameter extraction (subprocess) | Integration | ✅ Added |
| Cancellation on file change | Integration | ✅ Added |
| `OnceLock` → `Box::leak()` fix | Integration | ✅ Added |
| Rebuild workflow | Existing | ✅ Passing |

## Known Trade-offs

**Memory Leak (Intentional):**
- `param_specs()` now uses `Box::leak()` instead of `OnceLock`
- ✅ **Acceptable**: Called at most once per plugin load (not per-sample/frame)
- ✅ **Small**: Leak is ~hundreds of bytes (parameter metadata)
- ✅ **Plugin lifetime**: Plugin runs for entire DAW session
- ⚠️ **Future**: Investigate root cause of OnceLock hang on macOS dlopen (tracked in backlog)

## Checklist

- [x] Code follows project coding standards (Rust idioms, error handling)
- [x] Tests added for cancellation behavior
- [x] Documentation updated (hang fix summary, test plan)
- [x] No linting errors
- [x] Manual verification completed (hot-reload works)
- [x] Trade-offs documented (intentional leak rationale)

## Files Changed

**11 files changed, 766 insertions(+), 105 deletions(-)**

**Core changes:**
- `dev-server/src/reload/rebuild.rs` — Cancellation mechanism (44+ lines)
- `engine/crates/wavecraft-dsp/src/combinators/chain.rs` — OnceLock → Box::leak (84 lines)
- `dev-server/tests/reload_cancellation.rs` — Integration tests (148 new lines)

**Documentation:**
- `docs/feature-specs/hot-reload-cancellation/hotreload-hang-fix-summary.md` (123 lines)
- `docs/feature-specs/hot-reload-cancellation/test-plan-hotreload-cancellation.md` (237 lines)
