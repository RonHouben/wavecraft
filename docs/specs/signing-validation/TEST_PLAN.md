# Test Plan: Signing Infrastructure Validation

> **Feature:** macOS Code Signing with Verification  
> **Test Date:** 2026-01-31  
> **Tester:** Automated + Manual  
> **Environment:** macOS with Xcode CLI tools

---

## Test Scope

This test plan covers the **in-scope** phases that don't require Apple Developer credentials:
- ✅ Phase 1: Local ad-hoc signing validation
- ✅ Phase 5a: Build-only CI/CD (simulated locally)

**Out of scope:** Phase 2 (Ableton Live), Phase 3-4 (Developer ID/Notarization) - require manual testing with DAW

---

## Prerequisites

- [x] macOS development machine
- [x] Xcode Command Line Tools installed
- [x] Plugin bundles built at `engine/target/bundled/`
- [x] Rust toolchain installed

---

## Test Scenarios

### TS-01: Build Plugin Bundles

**Objective:** Verify plugin bundles can be built with webview editor

**Steps:**
1. Navigate to engine directory
2. Run `cargo xtask bundle --features webview_editor`

**Expected Results:**
- ✅ Build completes without errors
- ✅ VST3 bundle exists at `target/bundled/vstkit.vst3`
- ✅ CLAP bundle exists at `target/bundled/vstkit.clap`
- ✅ React UI is embedded in bundles

**Success Criteria:**
- Both bundles exist and are valid plugin formats
- No build errors or warnings (Rust warnings are acceptable)

---

### TS-02: Ad-Hoc Signing

**Objective:** Verify ad-hoc signing works without Apple Developer credentials

**Steps:**
1. Run `cargo xtask sign --adhoc`

**Expected Results:**
- ✅ Command completes successfully
- ✅ Output shows "Ad-hoc signing vstkit.vst3..."
- ✅ Output shows "Ad-hoc signing vstkit.clap..."
- ✅ Output shows "Ad-hoc signing complete"
- ✅ Exit code 0

**Success Criteria:**
- All bundles are signed
- No errors reported
- Bundles remain valid plugin formats

---

### TS-03: Signature Verification (Basic)

**Objective:** Verify signatures are valid using codesign

**Steps:**
1. Run `cargo xtask sign --verify`

**Expected Results:**
- ✅ Command completes successfully
- ✅ Output shows "Verifying bundle signatures..."
- ✅ Output shows "VST3 signature valid"
- ✅ Output shows "CLAP signature valid"
- ✅ Output shows "All N signatures verified successfully"
- ✅ Exit code 0

**Success Criteria:**
- All signatures pass verification
- No errors or warnings
- Command validates expected properties

---

### TS-04: Signature Verification (Verbose)

**Objective:** Verify signature details and entitlements are correct

**Steps:**
1. Run `cargo xtask sign --verify --verbose`

**Expected Results:**
- ✅ Command completes successfully
- ✅ Shows detailed codesign output
- ✅ Validates hardened runtime flag
- ✅ Validates JIT entitlement present
- ✅ Shows "✓ Hardened runtime enabled"
- ✅ Shows "✓ JIT entitlement present"

**Success Criteria:**
- Hardened runtime is enabled
- JIT entitlement is present (required for WebView)
- All expected entitlements are listed

---

### TS-05: Verification of Unsigned Bundles (Negative Test)

**Objective:** Verify that unsigned bundles fail verification

**Steps:**
1. Rebuild bundles without signing: `cargo xtask bundle --features webview_editor`
2. Run `cargo xtask sign --verify`

**Expected Results:**
- ❌ Command fails with non-zero exit code
- ❌ Error message indicates signature verification failed
- ❌ Clear error about which bundle failed

**Success Criteria:**
- Command properly detects missing signatures
- Error message is clear and actionable
- Exit code indicates failure

---

### TS-06: Re-signing Already Signed Bundles

**Objective:** Verify that signing can be re-run without issues

**Steps:**
1. Sign bundles: `cargo xtask sign --adhoc`
2. Sign again: `cargo xtask sign --adhoc`
3. Verify: `cargo xtask sign --verify`

**Expected Results:**
- ✅ Both sign commands succeed
- ✅ Second sign replaces first signature (using --force)
- ✅ Verification passes after re-signing
- ✅ No errors about existing signatures

**Success Criteria:**
- Re-signing works without manual intervention
- Final signature is valid
- Bundles remain functional

---

### TS-07: Missing Entitlements Detection

**Objective:** Verify that missing required entitlements are detected

**Steps:**
1. Manually sign without entitlements: 
   ```bash
   codesign --deep --force --sign - target/bundled/vstkit.vst3
   ```
2. Run `cargo xtask sign --verify --verbose`

**Expected Results:**
- ⚠️ Warning about missing entitlements (ad-hoc signatures may not include entitlements)
- ✅ OR error if JIT entitlement is critical and missing
- ✅ Clear message about what's missing

**Success Criteria:**
- Missing entitlements are detected
- User is informed about potential issues
- Message explains impact (WebView won't work)

---

### TS-08: CI Workflow Validation (Simulated)

**Objective:** Verify CI workflow steps work locally

**Steps:**
1. Run full CI sequence locally:
   ```bash
   cd ui && npm ci && npm run build
   cd ../engine
   cargo xtask bundle --features webview_editor
   cargo xtask sign --adhoc
   cargo xtask sign --verify --verbose
   ```

**Expected Results:**
- ✅ UI build succeeds
- ✅ Plugin build succeeds
- ✅ Signing succeeds
- ✅ Verification succeeds with assertions
- ✅ All steps complete in sequence

**Success Criteria:**
- Entire workflow completes without manual intervention
- Each step produces expected artifacts
- Final artifacts are signed and verified

---

### TS-09: Bundle Structure After Signing

**Objective:** Verify that signing doesn't corrupt bundle structure

**Steps:**
1. Sign bundles: `cargo xtask sign --adhoc`
2. Inspect bundle structure:
   ```bash
   ls -la target/bundled/vstkit.vst3/Contents/
   ls -la target/bundled/vstkit.clap/Contents/
   ```
3. Check for CodeResources:
   ```bash
   ls -la target/bundled/vstkit.vst3/Contents/_CodeSignature/
   ```

**Expected Results:**
- ✅ Bundle structure intact (Contents/, MacOS/, Resources/)
- ✅ `_CodeSignature/` directory exists
- ✅ `CodeResources` file exists in `_CodeSignature/`
- ✅ Binary files are not corrupted

**Success Criteria:**
- All expected directories and files present
- CodeSignature directory created by codesign
- Bundle remains valid plugin format

---

### TS-10: Error Handling - Missing Bundles

**Objective:** Verify graceful handling when no bundles exist

**Steps:**
1. Remove bundles: `rm -rf target/bundled/*`
2. Run `cargo xtask sign --adhoc`
3. Run `cargo xtask sign --verify`

**Expected Results:**
- ✅ Sign command completes (skips missing bundles)
- ✅ Verify command fails with clear message
- ✅ Error: "No plugin bundles found to verify"

**Success Criteria:**
- Clear error messages
- No confusing stack traces
- User knows what to do (build first)

---

## Test Execution Summary

| Test | Status | Pass/Fail | Notes |
|------|--------|-----------|-------|
| TS-01: Build Bundles | ⏳ Pending | - | |
| TS-02: Ad-Hoc Signing | ⏳ Pending | - | |
| TS-03: Verification Basic | ⏳ Pending | - | |
| TS-04: Verification Verbose | ⏳ Pending | - | |
| TS-05: Unsigned Detection | ⏳ Pending | - | |
| TS-06: Re-signing | ⏳ Pending | - | |
| TS-07: Missing Entitlements | ⏳ Pending | - | |
| TS-08: CI Workflow | ⏳ Pending | - | |
| TS-09: Bundle Structure | ⏳ Pending | - | |
| TS-10: Error Handling | ⏳ Pending | - | |

---

## Test Environment

- **OS:** macOS (version TBD)
- **Xcode CLI:** (version TBD)
- **Rust:** (version TBD)
- **Node.js:** (version TBD)

---

## Issues Found

*(Issues will be documented here during test execution)*

---

## Sign-off

- **Tester:** _________________
- **Date:** _________________
- **Overall Result:** ⏳ Pending / ✅ Pass / ❌ Fail
