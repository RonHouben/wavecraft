# QA Report: macOS Hardening & Code Signing Infrastructure

**Date**: 2026-01-31
**Reviewer**: QA Agent
**Status**: PASS

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |

**Overall**: PASS

## Automated Check Results

### cargo fmt --check
✅ **PASSED** - All code in xtask crate is properly formatted.

### cargo clippy -p xtask -- -D warnings
✅ **PASSED** - No clippy warnings or errors.

### cargo test -p xtask
✅ **PASSED** - 46 tests passed (42 unit tests + 4 integration tests).

```
running 42 tests (unit tests) ... ok
running 4 tests (sign + notarize modules) ... ok
```

### Signature Verification
✅ **PASSED** - Ad-hoc signed bundles verify correctly:
- `vstkit.vst3`: Valid signature with hardened runtime
- `vstkit.clap`: Valid signature with hardened runtime

## Feature Review

### Phase 1: Entitlements Configuration ✅

| Check | Status | Notes |
|-------|--------|-------|
| Production entitlements file exists | ✅ | `engine/signing/entitlements.plist` |
| Debug entitlements file exists | ✅ | `engine/signing/entitlements-debug.plist` |
| JIT entitlement present | ✅ | `com.apple.security.cs.allow-jit` |
| Unsigned memory entitlement | ✅ | `com.apple.security.cs.allow-unsigned-executable-memory` |
| Library validation disabled | ✅ | `com.apple.security.cs.disable-library-validation` |
| Debug entitlements include debugger access | ✅ | `com.apple.security.get-task-allow` |

### Phase 2: Sign Command ✅

| Check | Status | Notes |
|-------|--------|-------|
| `cargo xtask sign --help` works | ✅ | Shows all options |
| `cargo xtask sign --adhoc` works | ✅ | Signs without Apple ID |
| `cargo xtask sign --verify` works | ✅ | Validates signatures |
| Hardened runtime flag enabled | ✅ | `--options runtime` used |
| Timestamp included | ✅ | `--timestamp` flag used |
| Entitlements auto-loaded | ✅ | From `signing/entitlements.plist` |
| Error diagnostics implemented | ✅ | User-friendly error messages |
| Unit tests pass | ✅ | 2 tests in sign.rs |

### Phase 3: Notarize Command ✅

| Check | Status | Notes |
|-------|--------|-------|
| `--submit` workflow implemented | ✅ | ZIP creation + xcrun notarytool submit |
| `--status` workflow implemented | ✅ | xcrun notarytool info |
| `--staple` workflow implemented | ✅ | xcrun stapler staple |
| `--full` blocking workflow | ✅ | Submit → poll → staple |
| Request state persisted | ✅ | `.notarization-request` JSON file |
| Gatekeeper verification | ✅ | spctl --assess after staple |
| Notarization log fetched on failure | ✅ | xcrun notarytool log |
| Unit tests pass | ✅ | 2 tests in notarize.rs |

### Phase 4: Release Command ✅

| Check | Status | Notes |
|-------|--------|-------|
| Combines bundle + sign + notarize | ✅ | Single workflow command |
| `--skip-notarize` option | ✅ | For sign-only releases |
| Verbose mode propagated | ✅ | Passed to all sub-commands |

### Phase 5: CI/CD Pipeline ✅

| Check | Status | Notes |
|-------|--------|-------|
| CI workflow exists | ✅ | `.github/workflows/ci.yml` |
| Release workflow exists | ✅ | `.github/workflows/release.yml` |
| Ad-hoc signing in CI | ✅ | Artifacts are signed |
| Certificate import step | ✅ | Keychain setup in release workflow |
| Secrets referenced | ✅ | `APPLE_*` secrets used |
| Artifact upload | ✅ | VST3, CLAP bundles uploaded |

### Phase 6: Documentation ✅

| Check | Status | Notes |
|-------|--------|-------|
| macOS signing guide exists | ✅ | `docs/guides/macos-signing.md` |
| Prerequisites documented | ✅ | Apple Developer Program, Xcode tools |
| Step-by-step instructions | ✅ | Certificate creation, env vars |
| Keychain credential storage | ✅ | `xcrun notarytool store-credentials` |
| Troubleshooting included | ✅ | Common error messages explained |

## Code Quality Analysis

### Real-Time Safety
N/A - xtask is a build tool, not real-time audio code.

### Domain Separation
✅ All signing/notarization code is contained in `xtask` crate which is a CLI build tool. No audio or plugin code affected.

### Security Analysis

| Check | Status | Notes |
|-------|--------|-------|
| No hardcoded credentials | ✅ | All secrets via env vars |
| Password passed to xcrun, not logged | ✅ | Password not echoed |
| Temporary files cleaned up | ✅ | ZIP removed after submit |
| Request ID not exposed in logs | ✅ | Only shown on success |

### Error Handling

| Check | Status | Notes |
|-------|--------|-------|
| Missing env vars handled | ✅ | Clear error messages |
| codesign failures diagnosed | ✅ | `diagnose_signing_error()` |
| Notarization failures logged | ✅ | Fetches Apple's error log |
| No panics in production code | ✅ | `unwrap()` only in tests |

### Code Patterns

| Check | Status | Notes |
|-------|--------|-------|
| Functions under 50 lines | ✅ | All functions appropriately sized |
| Clear naming | ✅ | `run_adhoc`, `verify_signature`, etc. |
| Public APIs documented | ✅ | `///` comments on public functions |
| No dead code | ✅ | All functions used |
| Proper `Result` propagation | ✅ | `?` operator used consistently |

## Tests Fixed During Review

The following test issues were discovered and fixed during QA:

| Issue | Location | Fix |
|-------|----------|-----|
| Unsafe `set_var`/`remove_var` calls | `sign.rs:311-319`, `notarize.rs:379-390` | Wrapped in `unsafe` blocks with SAFETY comments |

These were Rust 2024 edition requirements where environment variable mutation is now considered unsafe due to potential data races.

## Verification Commands

All commands verified to work correctly:

```bash
# Format check
cargo fmt --check                          # ✅ Passed

# Clippy
cargo clippy -p xtask -- -D warnings      # ✅ Passed

# Tests
cargo test -p xtask                        # ✅ 46 tests passed

# CLI
cargo xtask --help                         # ✅ All commands listed
cargo xtask sign --help                    # ✅ Options displayed
cargo xtask sign --verify --verbose        # ✅ Signatures verified

# Signature verification
codesign --verify --deep --strict target/bundled/vstkit.vst3  # ✅ Valid
codesign --verify --deep --strict target/bundled/vstkit.clap  # ✅ Valid
```

## Architectural Concerns

None identified. The implementation follows the architecture defined in:
- `docs/architecture/high-level-design.md` - Build system section
- `docs/specs/macos-hardening/low-level-design-macos-hardening.md`

## Handoff Decision

**Target Agent**: None (QA Complete)
**Reasoning**: All checks pass, no issues found. Feature is ready for manual integration testing (Phase 6.3 in implementation progress).

## Recommendations

1. **Manual Testing**: Test notarization workflow with Apple Developer credentials when available
2. **DAW Testing**: Verify signed plugins load without security warnings in Ableton Live
3. **CI Monitoring**: Monitor first release workflow run to ensure secrets are correctly configured
