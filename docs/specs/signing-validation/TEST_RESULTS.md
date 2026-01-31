# Test Results: Signing Infrastructure Validation

> **Test Date:** 2026-01-31  
> **Tester:** AI-assisted testing  
> **Environment:** macOS

---

## Test Environment

- **OS:** macOS (determined at runtime)
- **Xcode CLI:** Installed (codesign available)
- **Rust:** Latest stable
- **Node.js:** v20
- **Test Plan:** [TEST_PLAN.md](TEST_PLAN.md)

---

## Test Execution Results

### ✅ TS-01: Build Plugin Bundles

**Status:** PASS

**Execution:**
```bash
cd engine
cargo xtask bundle --features webview_editor
```

**Results:**
- ✅ Build completed successfully
- ✅ VST3 bundle created at `target/bundled/vstkit.vst3`
- ✅ CLAP bundle created at `target/bundled/vstkit.clap`
- ✅ React UI built and embedded (154.27 kB JS, 5.06 kB CSS)
- ⚠️ 10 Rust warnings (unused code, snake_case naming) - non-blocking

**Verdict:** PASS

---

### ⏳ TS-02: Ad-Hoc Signing

**Status:** READY TO EXECUTE

**Command:**
```bash
cargo xtask sign --adhoc
```

**Expected Output:**
```
Ad-hoc signing vstkit.vst3...
Ad-hoc signing vstkit.clap...
✓ Ad-hoc signing complete
```

**Notes:**
- Requires macOS with codesign
- Should complete in < 5 seconds
- Manual execution required (terminal disabled)

---

### ⏳ TS-03: Signature Verification (Basic)

**Status:** READY TO EXECUTE

**Command:**
```bash
cargo xtask sign --verify
```

**Expected Output:**
```
Verifying bundle signatures...
✓ VST3 signature valid
✓ CLAP signature valid
✓ All 2 signatures verified successfully
```

**Notes:**
- Depends on TS-02 completion
- Should validate hardened runtime
- Should check for JIT entitlements

---

### ⏳ TS-04: Signature Verification (Verbose)

**Status:** READY TO EXECUTE

**Command:**
```bash
cargo xtask sign --verify --verbose
```

**Expected Output:**
```
Verifying bundle signatures...

[Detailed codesign output for vstkit.vst3]
✓ Hardened runtime enabled
✓ JIT entitlement present
✓ Unsigned executable memory allowed
✓ Library validation disabled
✓ VST3 signature valid

[Detailed codesign output for vstkit.clap]
✓ Hardened runtime enabled
✓ JIT entitlement present
✓ CLAP signature valid

✓ All 2 signatures verified successfully
```

---

### ⏳ TS-05: Verification of Unsigned Bundles (Negative Test)

**Status:** READY TO EXECUTE

**Command:**
```bash
cargo xtask bundle --features webview_editor  # Rebuild without signing
cargo xtask sign --verify
```

**Expected Behavior:**
- Should FAIL with error message
- Should indicate which bundle failed verification
- Exit code should be non-zero

---

### ⏳ TS-06: Re-signing Already Signed Bundles

**Status:** READY TO EXECUTE

**Commands:**
```bash
cargo xtask sign --adhoc
cargo xtask sign --adhoc  # Re-sign
cargo xtask sign --verify  # Verify
```

**Expected Behavior:**
- Both sign commands should succeed
- Using `--force` flag should replace existing signature
- Final verification should pass

---

### ⏳ TS-07: Missing Entitlements Detection

**Status:** READY TO EXECUTE

**Setup:**
```bash
# Manually sign without entitlements
codesign --deep --force --sign - target/bundled/vstkit.vst3
cargo xtask sign --verify --verbose
```

**Expected Behavior:**
- Should detect ad-hoc signature without entitlements
- Should warn about missing JIT entitlement
- Should explain WebView may not work

---

### ⏳ TS-08: CI Workflow Validation (Simulated)

**Status:** READY TO EXECUTE

**Full CI Sequence:**
```bash
cd ui && npm ci && npm run build
cd ../engine
cargo xtask bundle --features webview_editor
cargo xtask sign --adhoc
cargo xtask sign --verify --verbose
```

**Expected Behavior:**
- All steps complete without errors
- Each step produces expected artifacts
- Final artifacts are signed and verified
- Suitable for GitHub Actions execution

---

### ⏳ TS-09: Bundle Structure After Signing

**Status:** READY TO EXECUTE

**Commands:**
```bash
cargo xtask sign --adhoc
ls -la target/bundled/vstkit.vst3/Contents/
ls -la target/bundled/vstkit.vst3/Contents/_CodeSignature/
```

**Expected Structure:**
```
vstkit.vst3/
  Contents/
    MacOS/
      vstkit
    Resources/
      [embedded assets]
    _CodeSignature/
      CodeResources
    Info.plist
```

---

### ⏳ TS-10: Error Handling - Missing Bundles

**Status:** READY TO EXECUTE

**Commands:**
```bash
rm -rf target/bundled/*
cargo xtask sign --adhoc  # Should skip gracefully
cargo xtask sign --verify  # Should fail with clear message
```

**Expected Output for verify:**
```
ERROR: No plugin bundles found to verify
```

---

## Issues Found

### Issue #1: Ad-hoc Signatures May Not Include Entitlements

**Severity:** Medium  
**Component:** Signature Verification  
**Description:**

Ad-hoc signatures (using `--sign -`) may not preserve entitlements from the entitlements.plist file. This means:
- The `--verify` command may show a warning about missing entitlements
- WebView JavaScript JIT may not work in ad-hoc signed builds
- This is a limitation of ad-hoc signing, not a bug

**Impact:**
- Local development testing may show warnings
- Actual functionality may be limited without proper entitlements
- Developer ID signing (with entitlements) will resolve this

**Recommendation:**
- Document this limitation in the signing guide
- Consider adding a note in `run_adhoc()` output
- Phase 3 (Developer ID) testing will validate proper entitlements

**Status:** Documented, not blocking

---

### Issue #2: Rust Warnings During Build

**Severity:** Low  
**Component:** Plugin Code  
**Description:**

Build produces 10 warnings:
- Unused imports (egui editor code)
- Dead code (unused variants, methods)
- Non-snake_case method names (Objective-C bridge methods)

**Impact:**
- No functional impact
- Clutters build output
- May confuse users

**Recommendation:**
- Run `cargo fix --lib -p vstkit` to auto-fix some warnings
- Add `#[allow(dead_code)]` for intentionally unused future code
- Add `#[allow(non_snake_case)]` for Objective-C bridge methods

**Status:** Low priority, cosmetic

---

## Manual Verification Checklist

Due to terminal limitations, the following tests require manual execution:

- [ ] TS-02: Run ad-hoc signing
- [ ] TS-03: Run basic verification
- [ ] TS-04: Run verbose verification
- [ ] TS-05: Test unsigned bundle detection
- [ ] TS-06: Test re-signing
- [ ] TS-07: Test entitlements detection
- [ ] TS-08: Run full CI sequence
- [ ] TS-09: Inspect bundle structure
- [ ] TS-10: Test error handling

**Commands to run:**
```bash
cd /Users/ronhouben/code/private/vstkit/engine

# Test ad-hoc signing and verification
cargo xtask sign --adhoc
cargo xtask sign --verify --verbose

# Test re-signing
cargo xtask sign --adhoc
cargo xtask sign --adhoc

# Test error handling
rm -rf target/bundled/*
cargo xtask sign --verify

# Rebuild for next tests
cargo xtask bundle --features webview_editor
```

---

## Test Summary

| Category | Pass | Fail | Blocked | Pending |
|----------|------|------|---------|---------|
| Build | 1 | 0 | 0 | 0 |
| Signing | 0 | 0 | 0 | 3 |
| Verification | 0 | 0 | 0 | 4 |
| Error Handling | 0 | 0 | 0 | 2 |
| CI/CD | 0 | 0 | 0 | 1 |
| **Total** | **1** | **0** | **0** | **9** |

**Overall Status:** 🟡 PARTIALLY COMPLETE (automated portions done, manual testing required)

---

## Recommendations

### For Immediate Action
1. ✅ **Completed:** Test plan created with 10 comprehensive scenarios
2. ✅ **Completed:** CI workflow includes signing + verification
3. ⏳ **Pending:** Execute manual test scenarios (TS-02 through TS-10)
4. ⏳ **Pending:** Document actual results vs. expected results

### For Follow-up
1. Add documentation about ad-hoc signing limitations
2. Consider adding warning message in `run_adhoc()` about entitlements
3. Clean up Rust warnings (low priority)
4. Add GitHub Actions test run as proof of CI workflow

### For Future Phases
1. Phase 2: Ableton Live compatibility testing (requires DAW)
2. Phase 3: Developer ID signing validation (requires Apple account)
3. Phase 4: Notarization testing (requires Apple account)
4. Phase 5b: Signed release CI/CD (requires GitHub secrets)

---

## Sign-off

- **Test Plan Created:** ✅ Complete
- **Automated Tests:** ✅ Complete (build)
- **Manual Tests:** ⏳ Ready for execution
- **Issues Found:** 2 (Medium, Low severity - documented)
- **Blocking Issues:** None
- **Ready for Manual Testing:** ✅ Yes
