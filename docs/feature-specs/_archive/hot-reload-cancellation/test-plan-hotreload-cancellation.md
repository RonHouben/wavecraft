# Test Plan: Hot-Reload Cancellation on Superseded Changes

## Overview

This test plan verifies that when parameter extraction is in progress and a new file change occurs, the current extraction is cancelled immediately rather than waiting for timeout.

## Prerequisites

- Working Wavecraft SDK project with `wavecraft start` capability
- Terminal with `wavecraft` CLI installed

## Test Cases

### Test 1: Normal Parameter Extraction (Baseline)

**Objective:** Verify normal hot-reload works without cancellation.

**Steps:**

1. Start dev server: `wavecraft start`
2. Wait for initial load to complete
3. Make a small code change (e.g., add a comment in a processor)
4. Save the file
5. Observe hot-reload logs

**Expected Result:**

```
  → Rebuilding plugin...
  ✓ Build succeeded in X.Xs
  → Finding plugin dylib...
  → Found: target/debug/...
  → Copying to temp location...
  → Temp: /tmp/wavecraft_hotreload_...
  → Loading parameters via subprocess...
  → Loaded N parameters via subprocess
  → Updating parameter host...
  → Updated N parameters
  → Notifying UI clients...
  → UI notified
  ✓ Hot-reload complete — N parameters
```

**Pass Criteria:**

- Hot-reload completes successfully
- No errors or warnings
- Time to complete is reasonable (<5 seconds for small project)

---

### Test 2: Rapid Consecutive Changes (Superseded Extraction)

**Objective:** Verify that rapid consecutive file changes cancel pending parameter extraction.

**Steps:**

1. Start dev server: `wavecraft start`
2. Wait for initial load
3. Make a code change and save (e.g., add a comment)
4. **Immediately** (within 1-2 seconds) make another change and save
5. Observe terminal logs

**Expected Result:**

```
  → Rebuilding plugin...
  ✓ Build succeeded in X.Xs
  → Loading parameters via subprocess...
  → Build already in progress, queuing rebuild...
  ⚠ Parameter extraction cancelled — superseded by newer change
  → Rebuilding plugin...
  ✓ Build succeeded in X.Xs
  → Loading parameters via subprocess...
  → Loaded N parameters via subprocess
  ✓ Hot-reload complete — N parameters
```

**Pass Criteria:**

- The first parameter extraction is cancelled (see ⚠ warning)
- The second rebuild starts immediately
- Total time for both rebuilds is reasonable (<10 seconds)
- No 30-second timeout occurs

---

### Test 3: Multiple Rapid Changes (Coalescing)

**Objective:** Verify that multiple rapid changes coalesce into one rebuild.

**Steps:**

1. Start dev server: `wavecraft start`
2. Wait for initial load
3. Quickly make 3-4 consecutive changes (save each time)
4. Observe logs

**Expected Result:**

- Only see "Build already in progress, queuing rebuild..." messages
- Only 1-2 actual rebuilds occur (not 3-4)
- Parameter extraction may be cancelled if new changes arrive during extraction
- No hangs or timeouts

**Pass Criteria:**

- Multiple file saves don't trigger multiple complete rebuilds
- System coalesces changes efficiently
- No long waits (30+ seconds)

---

### Test 4: Slow Parameter Extraction (Simulated)

**Objective:** Verify cancellation works even with slow extraction (harder to test manually).

**Setup:**

- This test requires either:
  1. A very large plugin with many parameters (slow extraction)
  2. Or modifying `DEFAULT_EXTRACT_TIMEOUT` temporarily to 60s to simulate slowness

**Steps:**

1. (Optional) Modify `cli/src/project/param_extract.rs` to increase timeout to 60s
2. Start dev server: `wavecraft start`
3. Make a code change
4. Wait 2-3 seconds (while "Loading parameters via subprocess..." is showing)
5. Make another code change

**Expected Result:**

- First extraction shows "⚠ Parameter extraction cancelled — superseded by newer change"
- Second rebuild starts immediately
- No 60-second wait

**Pass Criteria:**

- Cancellation message appears
- Second build starts within seconds, not minutes
- No timeout errors

---

## Regression Tests

### Regression 1: Ensure Normal Build Still Works

**Objective:** Verify the cancellation mechanism doesn't break normal builds.

**Steps:**

1. Start dev server
2. Make single changes with 5+ seconds between each
3. Verify each rebuild completes normally

**Pass Criteria:**

- All rebuilds succeed
- No false cancellations
- Parameters load correctly each time

---

### Regression 2: Ensure Cargo Build Cancellation Still Works

**Objective:** Verify that cancelling during Cargo build phase still works (existing feature).

**Steps:**

1. Start dev server
2. Make a change that triggers a long build (e.g., add a dependency)
3. While "Rebuilding plugin..." is shown, make another change

**Pass Criteria:**

- Cargo build is cancelled or queued appropriately
- No build hangs
- Second build starts after first completes or is cancelled

---

## Success Criteria Summary

- ✅ Normal hot-reload works (Test 1)
- ✅ Rapid changes cancel parameter extraction (Test 2)
- ✅ Multiple rapid changes coalesce (Test 3)
- ✅ Slow extraction can be cancelled (Test 4)
- ✅ Normal builds still work (Regression 1)
- ✅ Cargo build cancellation unaffected (Regression 2)

## Known Limitations

1. **Temp file cleanup:** When parameter extraction is cancelled, the temporary dylib copy in `/tmp/wavecraft_hotreload_*` may not be cleaned up immediately. These files will be cleaned up by the OS eventually.

2. **Cancellation timing:** If parameter extraction completes very quickly (< 100ms), it may complete before the cancellation signal arrives. This is acceptable behavior.

3. **Error messages:** If a build fails after cancelling parameter extraction, you may see two error messages (one for cancellation, one for build failure). This is expected.

## Automated Test

The automated test in `dev-server/tests/reload_cancellation.rs` verifies the cancellation mechanism works at the unit level. To run:

```bash
cargo test --manifest-path dev-server/Cargo.toml reload_cancellation
```

This test verifies:

- Cancellation channel works correctly
- `tokio::select!` races param loading against cancellation
- Pipeline handles concurrent `handle_change()` calls

## Implementation Files Modified

1. `dev-server/src/reload/rebuild.rs`
   - Added `cancel_param_load_tx/rx` watch channels
   - Modified `handle_change()` to signal cancellation when marking pending
   - Modified `do_build()` to race param loading against cancellation

2. `dev-server/Cargo.toml`
   - Added `tempfile` dev-dependency for tests

3. `dev-server/tests/reload_cancellation.rs` (NEW)
   - Unit tests for cancellation mechanism

## Rollback Plan

If this change causes issues:

1. Revert `dev-server/src/reload/rebuild.rs` changes
2. Remove cancellation channels from `RebuildPipeline`
3. Remove `tokio::select!` from `do_build()`
4. Keep the test file for future attempts

The previous behavior was: wait for parameter extraction to complete or timeout (30s) even if new changes arrive.
