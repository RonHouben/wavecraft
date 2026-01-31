# Implementation Progress: Signing Infrastructure Validation

> **Feature:** Signing Infrastructure Validation & DAW Testing  
> **Started:** 2026-01-31  
> **Last Updated:** 2026-01-31

---

## Scope

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 1: Ad-Hoc Signing | ✅ In Scope | No Apple account required |
| Phase 2: Ableton Live Testing | ✅ In Scope | No Apple account required |
| Phase 3: Developer ID Signing | ⏸️ Deferred | Requires Apple Developer Program |
| Phase 4: Notarization | ⏸️ Deferred | Requires Apple Developer Program |
| Phase 5a: Build-Only CI/CD | ✅ In Scope | No Apple account required |
| Phase 5b: Signed Release CI/CD | ⏸️ Deferred | Requires Apple credentials |

---

## Progress Tracker

### Phase 1: Local Ad-Hoc Signing Validation ✅ IN SCOPE
| # | Task | Status | Notes |
|---|------|--------|-------|
| 1.1 | Build plugin bundles with webview_editor | ⏳ Not Started | |
| 1.2 | Run ad-hoc signing | ⏳ Not Started | |
| 1.3 | Verify ad-hoc signatures | ⏳ Not Started | |
| 1.4 | Inspect signature details | ⏳ Not Started | |

### Phase 2: Ableton Live Compatibility Testing ✅ IN SCOPE
| # | Task | Status | Notes |
|---|------|--------|-------|
| 2.1 | Install plugin in Ableton Live | ⏳ Not Started | |
| 2.2 | Load plugin without security warning | ⏳ Not Started | |
| 2.3 | Test React UI functionality | ⏳ Not Started | |
| 2.4 | Test project save/load | ⏳ Not Started | |
| 2.5 | Test CPU performance | ⏳ Not Started | |

### Phase 3: Developer ID Signing ⏸️ DEFERRED
| # | Task | Status | Notes |
|---|------|--------|-------|
| 3.1 | Configure Apple Developer credentials | ⏸️ Deferred | Requires Apple Developer Program |
| 3.2 | Sign with Developer ID | ⏸️ Deferred | |
| 3.3 | Verify Developer ID signature | ⏸️ Deferred | |

### Phase 4: Notarization Validation ⏸️ DEFERRED
| # | Task | Status | Notes |
|---|------|--------|-------|
| 4.1 | Configure notarization credentials | ⏸️ Deferred | Requires Apple Developer Program |
| 4.2 | Submit for notarization | ⏸️ Deferred | |
| 4.3 | Check notarization status | ⏸️ Deferred | |
| 4.4 | Staple notarization ticket | ⏸️ Deferred | |
| 4.5 | Verify Gatekeeper approval | ⏸️ Deferred | |
| 4.6 | Test on fresh macOS install | ⏸️ Deferred | |

### Phase 5a: Build-Only CI/CD ✅ IN SCOPE
| # | Task | Status | Notes |
|---|------|--------|-------|
| 5a.1 | Create build-only workflow | ⏳ Not Started | `.github/workflows/ci.yml` |
| 5a.2 | Trigger CI build | ⏳ Not Started | Push to main or PR |
| 5a.3 | Download and verify artifacts | ⏳ Not Started | |
| 5a.4 | Local ad-hoc sign CI artifacts | ⏳ Not Started | Proves CI artifacts can be signed |

### Phase 5b: Signed Release CI/CD ⏸️ DEFERRED
| # | Task | Status | Notes |
|---|------|--------|-------|
| 5b.1 | Configure GitHub repository secrets | ⏸️ Deferred | Requires Apple credentials |
| 5b.2 | Trigger signed release build | ⏸️ Deferred | |
| 5b.3 | Download and verify signed artifacts | ⏸️ Deferred | |

---

## Verification Checklist

### Phase 1: Ad-Hoc Signing ✅ IN SCOPE
- [ ] `cargo xtask bundle --features webview_editor` succeeds
- [ ] `cargo xtask sign --adhoc` succeeds
- [ ] `codesign --verify --deep --strict vstkit.vst3` passes
- [ ] `codesign --verify --deep --strict vstkit.clap` passes
- [ ] Entitlements include `com.apple.security.cs.allow-jit`

### Phase 2: Ableton Live ✅ IN SCOPE
- [ ] Plugin appears in Ableton's plugin browser
- [ ] Plugin loads without security warning
- [ ] React UI renders correctly
- [ ] Parameter sliders work
- [ ] Automation lanes sync
- [ ] Metering displays audio levels
- [ ] Clipping indicator triggers
- [ ] Window resizing works
- [ ] Project save/load preserves state
- [ ] Multiple instances work independently
- [ ] CPU usage < 10% per instance at 64 samples

### Phase 5a: Build-Only CI/CD ✅ IN SCOPE
- [ ] CI workflow file exists (`.github/workflows/ci.yml`)
- [ ] Push/PR triggers workflow
- [ ] Workflow completes successfully
- [ ] Unsigned artifacts are downloadable
- [ ] Downloaded artifacts can be signed locally

### Phase 3: Developer ID Signing ⏸️ DEFERRED
- [ ] `APPLE_SIGNING_IDENTITY` env var configured
- [ ] `cargo xtask sign` succeeds with Developer ID
- [ ] Signature shows "Developer ID Application" authority
- [ ] Signed plugin loads in Ableton without warning

### Phase 4: Notarization ⏸️ DEFERRED
- [ ] `cargo xtask notarize --submit` returns request ID
- [ ] `cargo xtask notarize --status` shows "Accepted"
- [ ] `cargo xtask notarize --staple` succeeds
- [ ] `spctl --assess --type install` shows "accepted"
- [ ] Plugin loads on fresh Mac without security prompts

### Phase 5b: Signed Release CI/CD ⏸️ DEFERRED
- [ ] All 6 GitHub secrets configured
- [ ] Tag push triggers release workflow
- [ ] Workflow completes with signing + notarization
- [ ] Artifacts are downloadable
- [ ] Downloaded artifacts pass `codesign --verify` and `spctl --assess`

---

## Issues Found

| Issue | Phase | Severity | Status | Notes |
|-------|-------|----------|--------|-------|
| *None yet* | | | | |

---

## Test Results

### Phase 2: Ableton Live Tests

**Test Environment:**
- macOS version: TBD
- Ableton Live version: TBD
- Plugin version: TBD

| Test Case | Result | Notes |
|-----------|--------|-------|
| Plugin loads | ⏳ | |
| UI renders | ⏳ | |
| Gain slider | ⏳ | |
| Bypass toggle | ⏳ | |
| Automation sync | ⏳ | |
| Peak meter | ⏳ | |
| RMS meter | ⏳ | |
| Clipping indicator | ⏳ | |
| Resize 600x400 | ⏳ | |
| Resize 1280x960 | ⏳ | |
| Save/load | ⏳ | |
| 3 instances | ⏳ | |
| 64-sample buffer | ⏳ | |

---

## Notes

- **Phase 1, 2, 5a** can be completed without any Apple account ✅
- **Phase 3, 4, 5b** require Apple Developer Program membership ($99/year) — deferred until credentials available
