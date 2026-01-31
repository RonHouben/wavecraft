# Implementation Plan: macOS Hardening & Packaging

> **Milestone:** 4  
> **Version:** 1.0  
> **Created:** 2026-01-31  
> **Status:** Ready for Implementation

---

## Overview

Implement macOS code signing and notarization for VstKit plugins, enabling distribution without Gatekeeper warnings. The implementation adds three new xtask commands (`sign`, `notarize`, `release`) and the required entitlements configuration.

## Requirements

- Plugin bundles (`.vst3`, `.clap`, `.component`) must be code-signed
- Plugins must pass Apple notarization
- WKWebView JavaScript JIT must work (requires entitlements)
- CI/CD pipeline must automate the release process
- Local development must work without Apple Developer account (ad-hoc signing)

## Architecture Changes

| Component | Change |
|-----------|--------|
| `engine/signing/` | New directory for entitlements files |
| `engine/xtask/src/commands/sign.rs` | New signing command |
| `engine/xtask/src/commands/notarize.rs` | New notarization command |
| `engine/xtask/src/commands/release.rs` | New combined release workflow |
| `engine/xtask/Cargo.toml` | Add `chrono` dependency |
| `.github/workflows/release.yml` | New CI/CD pipeline |

---

## Implementation Steps

### Phase 1: Entitlements Configuration

#### 1.1 Create Production Entitlements
**File:** `engine/signing/entitlements.plist`

- **Action:** Create entitlements file with JIT permissions for WKWebView
- **Why:** Hardened runtime blocks JIT by default; WebKit needs it for JavaScript
- **Dependencies:** None
- **Risk:** Low

**Content:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
</dict>
</plist>
```

#### 1.2 Create Development Entitlements
**File:** `engine/signing/entitlements-debug.plist`

- **Action:** Create debug entitlements with additional debugging permissions
- **Why:** Allows LLDB attachment and Instruments profiling during development
- **Dependencies:** None
- **Risk:** Low

**Content:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
    <key>com.apple.security.cs.debugger</key>
    <true/>
    <key>com.apple.security.get-task-allow</key>
    <true/>
</dict>
</plist>
```

---

### Phase 2: Code Signing Command

#### 2.1 Add Dependencies to xtask
**File:** `engine/xtask/Cargo.toml`

- **Action:** Add `chrono` crate for timestamp handling in notarization
- **Why:** Need to record submission timestamps
- **Dependencies:** None
- **Risk:** Low

#### 2.2 Create Sign Command Module
**File:** `engine/xtask/src/commands/sign.rs`

- **Action:** Implement `SigningConfig` struct and `run()` function
- **Why:** Orchestrates `codesign` CLI for all bundle types
- **Dependencies:** Step 1.1
- **Risk:** Low

**Key functions:**
- `SigningConfig::from_env()` — Load identity from environment
- `run(config)` — Sign all bundles in `target/bundled/`
- `sign_bundle(path, config)` — Sign a single bundle with entitlements
- `verify_signature(path)` — Run `codesign --verify`
- `run_adhoc()` — Ad-hoc signing for local development

#### 2.3 Register Sign Command in mod.rs
**File:** `engine/xtask/src/commands/mod.rs`

- **Action:** Add `pub mod sign;`
- **Why:** Make module available to main.rs
- **Dependencies:** Step 2.2
- **Risk:** Low

#### 2.4 Add Sign CLI to main.rs
**File:** `engine/xtask/src/main.rs`

- **Action:** Add `Sign` variant to `Commands` enum and match arm
- **Why:** Expose `cargo xtask sign` command
- **Dependencies:** Step 2.3
- **Risk:** Low

**CLI interface:**
```
cargo xtask sign [--identity <ID>] [--entitlements <PATH>] [--adhoc] [--verbose]
```

---

### Phase 3: Notarization Command

#### 3.1 Create Notarize Command Module
**File:** `engine/xtask/src/commands/notarize.rs`

- **Action:** Implement notarization workflow with submit/status/staple subcommands
- **Why:** Automates Apple notarization via `notarytool`
- **Dependencies:** Step 2.2 (signing must work first)
- **Risk:** Medium (depends on Apple service)

**Key functions:**
- `NotarizeConfig::from_env()` — Load Apple ID credentials
- `run(action, config)` — Dispatch to appropriate subcommand
- `submit(config)` — Create ZIP, submit to Apple, save request ID
- `status(config)` — Poll notarization status
- `staple(config)` — Attach ticket to bundles
- `full(config)` — Blocking workflow: submit → poll → staple

**State file:** `.notarization-request` (JSON with request ID)

#### 3.2 Register Notarize Command
**File:** `engine/xtask/src/commands/mod.rs`

- **Action:** Add `pub mod notarize;`
- **Why:** Make module available
- **Dependencies:** Step 3.1
- **Risk:** Low

#### 3.3 Add Notarize CLI to main.rs
**File:** `engine/xtask/src/main.rs`

- **Action:** Add `Notarize` variant to `Commands` enum
- **Why:** Expose `cargo xtask notarize` command
- **Dependencies:** Step 3.2
- **Risk:** Low

**CLI interface:**
```
cargo xtask notarize [--submit | --status | --staple | --full] [--verbose]
```

---

### Phase 4: Release Command

#### 4.1 Create Release Command Module
**File:** `engine/xtask/src/commands/release.rs`

- **Action:** Implement combined build → sign → notarize workflow
- **Why:** Single command for full release builds
- **Dependencies:** Steps 2.4, 3.3
- **Risk:** Low

**Key function:**
- `run(skip_notarize, verbose)` — Execute full release pipeline

#### 4.2 Register Release Command
**File:** `engine/xtask/src/commands/mod.rs`

- **Action:** Add `pub mod release;`
- **Why:** Make module available
- **Dependencies:** Step 4.1
- **Risk:** Low

#### 4.3 Add Release CLI to main.rs
**File:** `engine/xtask/src/main.rs`

- **Action:** Add `Release` variant to `Commands` enum
- **Why:** Expose `cargo xtask release` command
- **Dependencies:** Step 4.2
- **Risk:** Low

**CLI interface:**
```
cargo xtask release [--skip-notarize] [--verbose]
```

---

### Phase 5: CI/CD Pipeline

#### 5.1 Create GitHub Actions Workflow
**File:** `.github/workflows/release.yml`

- **Action:** Create workflow for automated release builds
- **Why:** Automate signing and notarization on tag push
- **Dependencies:** All previous phases
- **Risk:** Medium (requires secrets configuration)

**Workflow steps:**
1. Checkout code
2. Install Rust + Node.js
3. Import signing certificate to keychain
4. Build with `cargo xtask bundle`
5. Sign with `cargo xtask sign`
6. Submit for notarization
7. Poll for completion (30-second intervals, 30-minute timeout)
8. Staple tickets
9. Upload artifacts

#### 5.2 Document Required Secrets
**File:** `docs/guides/macos-signing.md`

- **Action:** Create developer documentation for signing setup
- **Why:** Developers need to know how to configure signing
- **Dependencies:** Step 5.1
- **Risk:** Low

**Secrets to document:**
- `APPLE_CERTIFICATE_P12`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_TEAM_ID`
- `APPLE_APP_PASSWORD`

---

### Phase 6: Testing & Validation

#### 6.1 Unit Tests for Signing Module
**File:** `engine/xtask/src/commands/sign.rs`

- **Action:** Add tests for `SigningConfig::from_env()`
- **Why:** Ensure environment parsing works correctly
- **Dependencies:** Step 2.2
- **Risk:** Low

#### 6.2 Unit Tests for Notarization Module
**File:** `engine/xtask/src/commands/notarize.rs`

- **Action:** Add tests for request serialization/deserialization
- **Why:** Ensure state file handling works
- **Dependencies:** Step 3.1
- **Risk:** Low

#### 6.3 Manual Integration Testing
- **Action:** Test full workflow on macOS with real certificate
- **Why:** Verify end-to-end functionality
- **Dependencies:** All phases
- **Risk:** Low

**Test checklist:**
- [ ] Ad-hoc signing works without Apple account
- [ ] Full signing works with Developer ID
- [ ] `codesign --verify --deep --strict` passes
- [ ] Notarization submission succeeds
- [ ] Notarization status polling works
- [ ] Stapling succeeds
- [ ] `spctl --assess` passes
- [ ] Plugin loads in Ableton Live without warning

---

## Testing Strategy

### Unit Tests
- `SigningConfig::from_env()` parsing
- `NotarizationRequest` JSON serialization
- Error message formatting

### Integration Tests
- Ad-hoc signing of test bundle
- Signature verification
- ZIP creation for notarization

### Manual Testing
- Full signing with Developer ID certificate
- Notarization with real Apple credentials
- Plugin loading in Ableton Live
- Plugin loading on fresh macOS (Gatekeeper test)

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Entitlements break plugin | Medium | High | Test on fresh macOS install |
| Notarization rejected | Medium | Medium | Check logs, fix issues, resubmit |
| CI keychain issues | Medium | Medium | Follow Apple's CI best practices |
| Apple service downtime | Low | Medium | Retry logic, manual fallback |

---

## Success Criteria

- [ ] `cargo xtask sign --adhoc` works without Apple account
- [ ] `cargo xtask sign` works with Developer ID
- [ ] `cargo xtask notarize --full` completes successfully
- [ ] Signed plugin loads in Ableton Live without security warning
- [ ] Signed plugin loads on fresh macOS install (Gatekeeper passes)
- [ ] `codesign --verify --deep --strict` passes on all bundles
- [ ] `spctl --assess --type install` passes on all bundles
- [ ] CI/CD pipeline runs successfully on tag push

---

## Estimated Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: Entitlements | 0.5 days | None |
| Phase 2: Sign Command | 1 day | Phase 1 |
| Phase 3: Notarize Command | 1.5 days | Phase 2 |
| Phase 4: Release Command | 0.5 days | Phase 3 |
| Phase 5: CI/CD Pipeline | 1 day | Phase 4 |
| Phase 6: Testing | 1.5 days | All phases |

**Total:** ~6 days

---

## Appendix: Environment Variables Reference

| Variable | Required For | Example |
|----------|--------------|---------|
| `APPLE_SIGNING_IDENTITY` | `sign` | `"Developer ID Application: Name (TEAM)"` |
| `APPLE_ENTITLEMENTS` | `sign` (optional) | `/path/to/entitlements.plist` |
| `APPLE_ID` | `notarize` | `developer@example.com` |
| `APPLE_TEAM_ID` | `notarize` | `ABC123XYZ` |
| `APPLE_APP_PASSWORD` | `notarize` | `xxxx-xxxx-xxxx-xxxx` or `@keychain:AC_PASSWORD` |
