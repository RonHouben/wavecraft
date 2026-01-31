# Implementation Plan: Signing Infrastructure Validation & DAW Testing

> **Milestone:** 4.5 (Validation Phase)  
> **Version:** 1.0  
> **Created:** 2026-01-31  
> **Status:** ✅ COMPLETE (Archived 2026-01-31)

---

## Overview

Validate the macOS code signing and notarization infrastructure implemented in Milestone 4. This phase focuses on manual testing to ensure signed plugins work correctly in production DAW environments, specifically Ableton Live as the primary target.

## Requirements

### In Scope (No Apple Account Required)
- Ad-hoc signed plugins must load in Ableton Live without security warnings
- Plugin functionality (React UI, parameters, metering) must work after signing
- CI/CD pipeline must successfully build and bundle (no signing)

### Deferred (Requires Apple Developer Account)
- Developer ID signed plugins must pass `codesign --verify --deep --strict`
- Notarized plugins must pass `spctl --assess --type install`
- CI/CD pipeline must successfully sign and notarize

## Architecture Changes

| Component | Change |
|-----------|--------|
| None | This is a validation-only phase; no code changes expected |

---

## Implementation Steps

### Phase 1: Local Ad-Hoc Signing Validation

> **Goal:** Prove the signing infrastructure works without Apple Developer credentials

#### 1.1 Build Plugin Bundles
**Action:** Build all plugin formats with the webview editor

```bash
cd engine
cargo xtask bundle --features webview_editor
```

- **Why:** Need fresh bundles to sign
- **Dependencies:** None
- **Risk:** Low
- **Expected Output:** `target/bundled/vstkit.vst3`, `target/bundled/vstkit.clap`

#### 1.2 Run Ad-Hoc Signing
**Action:** Sign bundles with ad-hoc identity

```bash
cargo xtask sign --adhoc
```

- **Why:** Validates signing workflow without Apple account
- **Dependencies:** Step 1.1
- **Risk:** Low
- **Expected Output:** "VST3 bundle signed", "CLAP bundle signed"

#### 1.3 Verify Ad-Hoc Signatures
**Action:** Run verification commands

```bash
codesign --verify --deep --strict target/bundled/vstkit.vst3
codesign --verify --deep --strict target/bundled/vstkit.clap
```

- **Why:** Confirms signing process completed correctly
- **Dependencies:** Step 1.2
- **Risk:** Low
- **Expected Output:** No output (success) or "valid on disk"

#### 1.4 Inspect Signature Details
**Action:** Display signature information

```bash
codesign -dv --verbose=4 target/bundled/vstkit.vst3
```

- **Why:** Verify entitlements are correctly applied
- **Dependencies:** Step 1.2
- **Risk:** Low
- **Expected Output:** Shows "runtime" flag, entitlements list

---

### Phase 2: Ableton Live Compatibility Testing

> **Goal:** Verify signed plugin works correctly in the primary target DAW

#### 2.1 Install Plugin in Ableton Live
**Action:** Copy signed bundles to plugin directory

```bash
cp -R target/bundled/vstkit.vst3 ~/Library/Audio/Plug-Ins/VST3/
cp -R target/bundled/vstkit.clap ~/Library/Audio/Plug-Ins/CLAP/
```

- **Why:** Make plugins available to Ableton Live
- **Dependencies:** Step 1.2
- **Risk:** Low
- **Notes:** Run Ableton Live's "Rescan Plugins" after copying

#### 2.2 Load Plugin Without Security Warning
**Action:** Open Ableton Live, add VstKit to a track

- **Why:** Confirm no macOS security dialogs appear
- **Dependencies:** Step 2.1
- **Risk:** Medium (ad-hoc may still warn on some macOS versions)
- **Success Criteria:** Plugin loads without "unidentified developer" warning

#### 2.3 Test React UI Functionality
**Action:** Interact with plugin UI in Ableton Live

| Test | Action | Expected Result |
|------|--------|-----------------|
| UI Renders | Open plugin | React UI appears with meters and controls |
| Parameter Change | Move slider | Host automation lane updates |
| Automation | Draw automation in Ableton | UI slider follows automation |
| Resize | Click resize button | Window resizes correctly |
| Multiple Instances | Add plugin to 3+ tracks | Each instance independent |

- **Why:** Signing must not break UI functionality
- **Dependencies:** Step 2.2
- **Risk:** Low
- **Notes:** WebView JIT requires entitlements; if UI is blank, check entitlements

#### 2.4 Test Project Save/Load
**Action:** Save Ableton project, close, reopen

- **Why:** State persistence must work with signed plugins
- **Dependencies:** Step 2.3
- **Risk:** Low
- **Success Criteria:** All parameter values restored correctly

#### 2.5 Test CPU Performance
**Action:** Set buffer size to 64 samples, check CPU usage

- **Why:** Signing must not impact performance
- **Dependencies:** Step 2.3
- **Risk:** Low
- **Success Criteria:** CPU usage reasonable (<10% per instance)

---

### Phase 3: Developer ID Signing ⏸️ DEFERRED

> **Status:** Deferred — Requires Apple Developer Program membership ($99/year)  
> **Goal:** Validate production signing workflow

#### 3.1 Configure Apple Developer Credentials
**Action:** Set environment variables

```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
```

- **Why:** Required for Developer ID signing
- **Dependencies:** Apple Developer Program membership
- **Risk:** N/A if no account available
- **Reference:** See `docs/guides/macos-signing.md` section 1

#### 3.2 Sign with Developer ID
**Action:** Run signing command

```bash
cargo xtask sign
```

- **Why:** Production signing for distribution
- **Dependencies:** Step 3.1
- **Risk:** Medium (certificate issues possible)
- **Expected Output:** "VST3 bundle signed" with Developer ID

#### 3.3 Verify Developer ID Signature
**Action:** Run verification with verbose output

```bash
codesign --verify --deep --strict --verbose=2 target/bundled/vstkit.vst3
```

- **Why:** Confirm Developer ID is correctly applied
- **Dependencies:** Step 3.2
- **Risk:** Low
- **Expected Output:** Shows "Developer ID Application: ..." in authority chain

---

### Phase 4: Notarization Validation ⏸️ DEFERRED

> **Status:** Deferred — Requires Apple Developer Program membership ($99/year)  
> **Goal:** Validate Apple notarization workflow

#### 4.1 Configure Notarization Credentials
**Action:** Set environment variables or keychain profile

```bash
export APPLE_ID="your-apple-id@example.com"
export APPLE_TEAM_ID="YOUR_TEAM_ID"
export APPLE_APP_PASSWORD="@keychain:AC_PASSWORD"
```

- **Why:** Required for notarization API access
- **Dependencies:** Apple Developer Program membership
- **Risk:** N/A if no account available
- **Reference:** See `docs/guides/macos-signing.md` section 2

#### 4.2 Submit for Notarization
**Action:** Run notarization submission

```bash
cargo xtask notarize --submit
```

- **Why:** Upload to Apple for notarization
- **Dependencies:** Step 3.2 (must be signed first)
- **Risk:** Medium (Apple service may be slow or reject)
- **Expected Output:** Request ID saved to `.notarization-request`

#### 4.3 Check Notarization Status
**Action:** Poll for completion

```bash
cargo xtask notarize --status
```

- **Why:** Wait for Apple to process
- **Dependencies:** Step 4.2
- **Risk:** Low
- **Expected Output:** "Status: Accepted" (5-30 minutes typical)

#### 4.4 Staple Notarization Ticket
**Action:** Attach ticket to bundles

```bash
cargo xtask notarize --staple
```

- **Why:** Enable offline verification
- **Dependencies:** Step 4.3 (must be accepted)
- **Risk:** Low
- **Expected Output:** "Stapled to vstkit.vst3"

#### 4.5 Verify Gatekeeper Approval
**Action:** Run Gatekeeper assessment

```bash
spctl --assess --type install --verbose target/bundled/vstkit.vst3
```

- **Why:** Final verification that Gatekeeper will allow
- **Dependencies:** Step 4.4
- **Risk:** Low
- **Expected Output:** "accepted, source=Notarized Developer ID"

#### 4.6 Test on Fresh macOS Install (Optional)
**Action:** Copy plugin to a Mac that has never approved VstKit

- **Why:** Simulates end-user first-time install
- **Dependencies:** Step 4.4
- **Risk:** Low
- **Success Criteria:** Plugin loads without any security prompts

---

### Phase 5a: Build-Only CI/CD Validation ✅ NO APPLE ACCOUNT REQUIRED

> **Goal:** Verify CI/CD can build and bundle plugins (without signing)

#### 5a.1 Create Build-Only Workflow
**Action:** Create or modify GitHub Actions workflow for build-only CI

**File:** `.github/workflows/ci.yml`

```yaml
name: CI Build
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      - name: Build UI
        run: cd ui && npm ci && npm run build
      - name: Build Plugin
        run: cd engine && cargo xtask bundle --features webview_editor
      - name: Upload Artifacts
        uses: actions/upload-artifact@v4
        with:
          name: vstkit-unsigned
          path: engine/target/bundled/
```

- **Why:** Validates build infrastructure without Apple credentials
- **Dependencies:** None
- **Risk:** Low

#### 5a.2 Trigger CI Build
**Action:** Push to main or create PR

- **Why:** Verify workflow triggers correctly
- **Dependencies:** Step 5a.1
- **Risk:** Low
- **Expected Output:** Green build with unsigned artifacts

#### 5a.3 Download and Verify Artifacts
**Action:** Download unsigned bundles from GitHub Actions

- **Why:** Confirm bundles are correctly built
- **Dependencies:** Step 5a.2
- **Risk:** Low
- **Success Criteria:** Downloaded `.vst3` and `.clap` bundles exist and are valid

#### 5a.4 Local Ad-Hoc Sign Downloaded Artifacts
**Action:** Sign the CI-built artifacts locally

```bash
# Download artifacts from GitHub Actions
# Then sign locally:
cd downloaded-artifacts
codesign --deep --force --options runtime --timestamp \
  --entitlements ../engine/signing/entitlements.plist \
  --sign - vstkit.vst3
codesign --verify --deep --strict vstkit.vst3
```

- **Why:** Proves CI artifacts can be signed post-build
- **Dependencies:** Step 5a.3
- **Risk:** Low
- **Success Criteria:** Signature verification passes

---

### Phase 5b: Signed Release CI/CD ⏸️ DEFERRED

> **Status:** Deferred — Requires Apple Developer credentials  
> **Goal:** Automate signing and notarization in CI

#### 5b.1 Configure GitHub Repository Secrets
**Action:** Add required secrets to GitHub (when credentials available)

| Secret | Value |
|--------|-------|
| `APPLE_CERTIFICATE_P12` | Base64-encoded P12 certificate |
| `APPLE_CERTIFICATE_PASSWORD` | P12 export password |
| `APPLE_SIGNING_IDENTITY` | Full identity string |
| `APPLE_ID` | Apple ID email |
| `APPLE_TEAM_ID` | 10-character team ID |
| `APPLE_APP_PASSWORD` | App-specific password |

- **Why:** CI needs credentials to sign
- **Dependencies:** Apple Developer credentials
- **Risk:** Medium (secrets misconfiguration possible)
- **Reference:** See `docs/guides/macos-signing.md` section 4

#### 5b.2 Trigger Signed Release Build
**Action:** Push a release tag

```bash
git tag v0.1.0
git push origin v0.1.0
```

- **Why:** Verify full release pipeline with signing
- **Dependencies:** Step 5b.1
- **Risk:** Medium (CI environment differences)
- **Expected Output:** Green build with signed + notarized artifacts

#### 5b.3 Download and Verify Signed Artifacts
**Action:** Download from GitHub Actions artifacts

- **Why:** Confirm artifacts are correctly signed and notarized
- **Dependencies:** Step 5b.2
- **Risk:** Low
- **Success Criteria:** `codesign --verify` and `spctl --assess` pass

---

## Testing Strategy

### Unit Tests
- Already complete (included in sign.rs, notarize.rs)

### Integration Tests
- Phase 1-2: Ad-hoc signing + Ableton Live (no account needed) ✅ **IN SCOPE**
- Phase 3-4: Developer ID + Notarization ⏸️ **DEFERRED**
- Phase 5a: Build-only CI/CD ✅ **IN SCOPE**
- Phase 5b: Signed release CI/CD ⏸️ **DEFERRED**

### Manual Tests
| Test | Phase | Status | Blocking? |
|------|-------|--------|----------|
| Ad-hoc signing works | 1 | ✅ In Scope | Yes |
| Plugin loads in Ableton | 2 | ✅ In Scope | Yes |
| React UI functions | 2 | ✅ In Scope | Yes |
| CI build-only works | 5a | ✅ In Scope | No |
| Developer ID signing | 3 | ⏸️ Deferred | No |
| Notarization passes | 4 | ⏸️ Deferred | No |
| CI signed release works | 5b | ⏸️ Deferred | No |

---

## Risks & Mitigations

### Risk: Ad-hoc signing shows security warning
- **Likelihood:** Medium (depends on macOS version, SIP status)
- **Impact:** Low (expected for ad-hoc, Developer ID will fix)
- **Mitigation:** Document workaround (System Settings → Privacy → allow anyway)

### Risk: WebView shows blank due to entitlements
- **Likelihood:** Low (entitlements correctly configured)
- **Impact:** High (plugin unusable)
- **Mitigation:** Verify entitlements with `codesign -d --entitlements :- vstkit.vst3`

### Risk: Notarization fails due to unsigned nested code
- **Likelihood:** Low (nih-plug bundles are simple)
- **Impact:** Medium (blocks distribution)
- **Mitigation:** Check `cargo xtask notarize --status` logs for specific file

### Risk: CI keychain import fails
- **Likelihood:** Medium (CI environment quirks)
- **Impact:** Medium (blocks automated releases)
- **Mitigation:** Test locally first; use existing GitHub Actions marketplace actions

---

## Success Criteria

### In Scope (Must Pass)
- [x] Ad-hoc signed plugin loads in Ableton Live
- [x] React UI renders and is interactive
- [x] Parameter changes sync between UI and host
- [x] Project save/load works correctly
- [x] CI/CD build-only workflow produces downloadable unsigned artifacts
- [x] Downloaded CI artifacts can be signed locally

### Deferred (Future Validation)
- [ ] Developer ID signing produces valid signature
- [ ] Notarization passes Gatekeeper check
- [ ] CI/CD signed release produces notarized artifacts

---

## Timeline

| Day | Focus | Phases | Status |
|-----|-------|--------|--------|
| 1 | Ad-hoc + Ableton | Phase 1, Phase 2 | ✅ Complete |
| 1 | Build-only CI/CD | Phase 5a | ✅ Complete |
| — | Developer ID + Notarization | Phase 3, Phase 4 | ⏸️ Deferred |
| — | Signed Release CI/CD | Phase 5b | ⏸️ Deferred |

---

## Dependencies

### Required (Blocking for In-Scope Phases)
- macOS development machine with Xcode CLI tools
- Ableton Live (any version 10+)
- GitHub repository with Actions enabled

### Deferred (Required for Future Phases)
- Apple Developer Program membership ($99/year)

---

## Next Steps After Validation

1. **If in-scope tests pass:** Mark Milestone 4.5 as complete, document deferred items ✅ DONE
2. **When Apple credentials available:** Execute Phase 3, 4, and 5b
3. **If issues found:** Document bugs, create follow-up stories, prioritize fixes
4. **Move to Milestone 5:** Polish & Optimization
