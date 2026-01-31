# QA Report: macOS Hardening Feature Branch

**Date**: 2026-01-31  
**Reviewer**: QA Agent  
**Branch**: `feat/macos-hardening`  
**Status**: ⚠️ **CONDITIONAL PASS**

---

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High | 1 |
| Medium | 5 |
| Low | 3 |

**Overall**: ⚠️ **CONDITIONAL PASS** - 1 High severity issue in existing code requires attention. The new signing infrastructure (xtask) passes all checks.

---

## Automated Check Results

### cargo fmt --check
✅ **Passed** - All code is properly formatted.

### cargo clippy -p xtask -- -D warnings
✅ **Passed** - No warnings in the signing infrastructure code.

### cargo test -p xtask
✅ **Passed** - 46 tests passed (42 + 4)
```
test result: ok. 42 passed; 0 failed; 0 ignored
test result: ok. 4 passed; 0 failed; 0 ignored
```

### cargo clippy --workspace -- -D warnings
❌ **Failed** - 33 errors in existing code (vstkit, desktop crates)

**Breakdown:**
- `vstkit` (plugin): 30 errors (dead code, non_snake_case, collapsible_if)
- `desktop`: 2 errors (double_ended_iterator_last, single_match)

---

## Findings

| ID | Severity | Category | Description | Location | Recommendation |
|----|----------|----------|-------------|----------|----------------|
| 1 | High | Real-time Safety | `Arc<std::sync::Mutex>` used for `meter_consumer` in plugin. While the mutex is only accessed from the UI thread, the lock contention with editor open/close could block. | [lib.rs#L30](../../../../engine/crates/plugin/src/lib.rs#L30) | Consider using `parking_lot::Mutex` which has better performance characteristics, or restructure to avoid shared state. |
| 2 | Medium | Code Quality | 30 clippy errors in vstkit crate - dead code, unused enums, non_snake_case methods | `crates/plugin/src/*.rs` | Run `cargo fix --lib -p vstkit` and add appropriate `#[allow()]` for Objective-C methods |
| 3 | Medium | Code Quality | Collapsible if statement in mod.rs | [mod.rs#L154](../../../../engine/crates/plugin/src/editor/mod.rs#L154) | Combine nested `if let` statements as suggested by clippy |
| 4 | Medium | Code Quality | `Iterator::last()` on `DoubleEndedIterator` | [assets.rs#L41](../../../../engine/crates/desktop/src/assets.rs#L41) | Use `next_back()` for O(1) instead of O(n) |
| 5 | Medium | Code Quality | `match` for single pattern in webview.rs | [webview.rs#L94](../../../../engine/crates/desktop/src/webview.rs#L94) | Convert to `if let` as suggested |
| 6 | Medium | Documentation | Entitlements in sign.rs validation assumes JIT entitlement is always required | [sign.rs#L272](../../../../engine/xtask/src/commands/sign.rs#L272) | JIT is only required when WebView feature is enabled. Consider making this conditional. |
| 7 | Low | Code Style | Non-snake_case method names in macOS WebView bridge | [macos.rs#L299,383,458](../../../../engine/crates/plugin/src/editor/macos.rs#L299) | Add `#[allow(non_snake_case)]` attribute - these match Objective-C selectors |
| 8 | Low | Code Quality | Unused `meter_consumer` field warning | [lib.rs#L30](../../../../engine/crates/plugin/src/lib.rs#L30) | Field is used via `meter_consumer.clone()` in editor creation - this is a false positive from conditional compilation |
| 9 | Low | Test Coverage | Sign and notarize commands have minimal unit tests | `sign.rs`, `notarize.rs` | Consider adding more unit tests for error conditions and edge cases |

---

## Feature-Specific Analysis

### ✅ Signing Infrastructure (New Code)

The new signing infrastructure in `xtask/src/commands/sign.rs` is well-structured:

**Positive Findings:**
- ✅ Clear separation of concerns (sign, verify, adhoc)
- ✅ Proper error handling with `anyhow::Context`
- ✅ Hardened runtime properly enabled with `--options runtime`
- ✅ Entitlements correctly included for WebView JIT
- ✅ User-friendly error messages in `diagnose_signing_error()`
- ✅ No real-time safety concerns (build tooling, not audio code)
- ✅ Unit tests present for configuration loading

**Verification:**
- Ad-hoc signing tested: ✅ Works correctly
- Signature verification tested: ✅ Detects invalid signatures
- Hardened runtime: ✅ `flags=0x10002(adhoc,runtime)` confirmed

### ✅ Notarization Infrastructure (New Code)

The notarization code in `xtask/src/commands/notarize.rs`:

**Positive Findings:**
- ✅ Async-friendly design with submit/status/staple/full workflow
- ✅ State persistence for long-running notarization
- ✅ Proper use of `xcrun notarytool`
- ✅ Configuration loaded from environment variables

### ✅ Entitlements (New Files)

`engine/signing/entitlements.plist`:
- ✅ JIT entitlement for WebView JavaScript
- ✅ Unsigned executable memory for WebKit
- ✅ Library validation disabled for AU wrapper
- ✅ Comments explain purpose of each entitlement

### ✅ Domain Separation

- ✅ `dsp/` - Pure audio math, no framework dependencies
- ✅ `protocol/` - Parameter contracts only
- ✅ `metering/` - Uses lock-free SPSC (`rtrb`) for real-time safety
- ✅ `xtask/` - Build tooling, properly isolated

---

## Architectural Concerns

> ⚠️ **The following items require architect review before merging.**

### 1. Mutex in Plugin Struct (Medium Concern)

The `meter_consumer: Arc<std::sync::Mutex<MeterConsumer>>` pattern could potentially cause issues:

```rust
pub struct VstKitPlugin {
    // ...
    meter_consumer: Arc<std::sync::Mutex<MeterConsumer>>,
}
```

While the mutex is only accessed from the UI thread (not audio thread), there's a subtle issue:
- Editor creation/destruction happens on the main thread
- If the host rapidly opens/closes the editor while audio is processing, the Arc refcount operations could cause cache contention

**Recommendation:** This is acceptable for now since the mutex is never accessed from the audio thread, but document this constraint clearly.

### 2. Conditional Entitlement Validation

The signature verification requires JIT entitlement unconditionally:

```rust
if !entitlements_output.contains("com.apple.security.cs.allow-jit") {
    anyhow::bail!("Bundle {} is missing required JIT entitlement...");
}
```

This should ideally check whether the webview_editor feature was enabled during build.

---

## Handoff Decision

**Target Agent**: `coder`  
**Priority**: Medium  
**Reasoning**: The clippy errors are straightforward fixes. The High severity item is a documentation/constraint issue rather than a critical bug.

### Immediate Fixes Required:

1. **Address clippy errors in vstkit crate:**
   - Add `#[allow(dead_code)]` for intentionally unused editor code
   - Add `#[allow(non_snake_case)]` for Objective-C bridge methods
   - Fix collapsible if statement

2. **Address clippy errors in desktop crate:**
   - Change `.last()` to `.next_back()`
   - Change match to if-let

### Optional Improvements:

3. Add documentation comment to `meter_consumer` explaining thread-safety constraints
4. Consider making JIT entitlement validation conditional on feature flag

---

## Test Results Reference

The signing infrastructure was tested extensively:
- See [MANUAL_TEST_RESULTS.md](../signing-validation/MANUAL_TEST_RESULTS.md)
- 11/11 test scenarios passed
- Issue #1 (missing hardened runtime) has been fixed

---

## Sign-off

- **QA Reviewer:** QA Agent
- **Date:** 2026-01-31
- **Scope:** feat/macos-hardening branch (55+ modified files)
- **Result:** ⚠️ **CONDITIONAL PASS**

**Conclusion:**

The new signing infrastructure code is high quality and passes all checks. The feature is ready for production use. However, there are existing clippy errors in the codebase that should be addressed before merging. These are straightforward fixes and don't affect the signing functionality.

**Recommendation:** 
1. Fix the clippy errors (estimated 15-30 minutes)
2. Merge after clippy passes workspace-wide
