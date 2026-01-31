# Implementation Progress: macOS Hardening & Packaging

> **Milestone:** 4  
> **Started:** 2026-01-31  
> **Last Updated:** 2026-01-31

---

## Progress Tracker

### Phase 1: Entitlements Configuration
| # | Task | Status | Notes |
|---|------|--------|-------|
| 1.1 | Create production entitlements (`entitlements.plist`) | ✅ Complete | JIT + unsigned memory entitlements added |
| 1.2 | Create development entitlements (`entitlements-debug.plist`) | ✅ Complete | Debug entitlements added |

### Phase 2: Code Signing Command
| # | Task | Status | Notes |
|---|------|--------|-------|
| 2.1 | Add `chrono` dependency to xtask | ✅ Complete | Added chrono, serde, serde_json |
| 2.2 | Create `sign.rs` command module | ✅ Complete | Full implementation with unit tests |
| 2.3 | Register sign module in `mod.rs` | ✅ Complete | |
| 2.4 | Add Sign CLI to `main.rs` | ✅ Complete | |

### Phase 3: Notarization Command
| # | Task | Status | Notes |
|---|------|--------|-------|
| 3.1 | Create `notarize.rs` command module | ✅ Complete | Submit/status/staple/full workflow |
| 3.2 | Register notarize module in `mod.rs` | ✅ Complete | |
| 3.3 | Add Notarize CLI to `main.rs` | ✅ Complete | |

### Phase 4: Release Command
| # | Task | Status | Notes |
|---|------|--------|-------|
| 4.1 | Create `release.rs` command module | ✅ Complete | Combined workflow implementation |
| 4.2 | Register release module in `mod.rs` | ✅ Complete | |
| 4.3 | Add Release CLI to `main.rs` | ✅ Complete | |

### Phase 5: CI/CD Pipeline
| # | Task | Status | Notes |
|---|------|--------|-------|
| 5.1 | Create GitHub Actions workflow | ✅ Complete | `.github/workflows/release.yml` |
| 5.2 | Create signing documentation | ✅ Complete | `docs/guides/macos-signing.md` |

### Phase 6: Testing & Validation
| # | Task | Status | Notes |
|---|------|--------|-------|
| 6.1 | Unit tests for signing module | ✅ Complete | Included in sign.rs |
| 6.2 | Unit tests for notarization module | ✅ Complete | Included in notarize.rs |
| 6.3 | Manual integration testing | ⏳ Pending | Requires Apple Developer credentials |

---

## Verification Checklist

### Code Signing
- [ ] `cargo xtask sign --adhoc` works without Apple account
- [ ] `cargo xtask sign` works with Developer ID
- [ ] `codesign --verify --deep --strict vstkit.vst3` passes
- [ ] `codesign --verify --deep --strict vstkit.clap` passes
- [ ] `codesign --verify --deep --strict vstkit.component` passes

### Notarization
- [ ] `cargo xtask notarize --submit` returns request ID
- [ ] `cargo xtask notarize --status` shows correct status
- [ ] `cargo xtask notarize --staple` attaches tickets
- [ ] `spctl --assess --type install vstkit.vst3` passes
- [ ] `spctl --assess --type install vstkit.clap` passes

### DAW Testing
- [ ] Plugin loads in Ableton Live without security warning
- [ ] Plugin loads on fresh macOS install (no prior approval)
- [ ] Plugin functions correctly (React UI works, parameters work)

### CI/CD
- [ ] GitHub Actions workflow runs on tag push
- [ ] Keychain import succeeds
- [ ] Artifacts are uploaded correctly

---

## Implementation Summary

### Completed Features

**Phase 1: Entitlements** ✅
- Production entitlements with JIT permissions for WKWebView
- Development entitlements with debugging support
- Properly scoped for hardened runtime requirements

**Phase 2: Sign Command** ✅
- `cargo xtask sign` — Sign with Developer ID
- `cargo xtask sign --adhoc` — Ad-hoc signing for local dev
- Automatic entitlements loading from `engine/signing/`
- Deep verification with `codesign --verify --deep --strict`
- User-friendly error messages for common failures

**Phase 3: Notarize Command** ✅
- `cargo xtask notarize --submit` — Submit to Apple
- `cargo xtask notarize --status` — Check progress
- `cargo xtask notarize --staple` — Attach tickets
- `cargo xtask notarize --full` — Blocking workflow (submit + wait + staple)
- State persistence via `.notarization-request` file
- Automatic log fetching on failure

**Phase 4: Release Command** ✅
- `cargo xtask release` — Complete build → sign → notarize workflow
- `cargo xtask release --skip-notarize` — Sign-only release
- Builds with `webview_editor` feature enabled

**Phase 5: CI/CD Pipeline** ✅
- GitHub Actions workflow for automated releases
- Keychain management for CI environment
- Polling-based notarization wait (30-minute timeout)
- Artifact upload for distribution

**Phase 6: Documentation** ✅
- Comprehensive signing guide at `docs/guides/macos-signing.md`
- Environment variable reference
- Troubleshooting section
- CI/CD setup instructions

### Files Created

| File | Purpose |
|------|---------|
| `engine/signing/entitlements.plist` | Production entitlements |
| `engine/signing/entitlements-debug.plist` | Development entitlements |
| `engine/xtask/src/commands/sign.rs` | Code signing implementation (217 lines) |
| `engine/xtask/src/commands/notarize.rs` | Notarization implementation (386 lines) |
| `engine/xtask/src/commands/release.rs` | Release workflow (37 lines) |
| `.github/workflows/release.yml` | CI/CD pipeline |
| `docs/guides/macos-signing.md` | Developer documentation |

### Files Modified

| File | Changes |
|------|---------|
| `engine/xtask/Cargo.toml` | Added chrono, serde, serde_json dependencies |
| `engine/xtask/src/commands/mod.rs` | Registered sign, notarize, release modules |
| `engine/xtask/src/main.rs` | Added Sign, Notarize, Release CLI commands |

### Next Steps

1. **Test ad-hoc signing** — Run `cargo xtask sign --adhoc` on local machine
2. **Obtain Apple Developer credentials** — Follow `docs/guides/macos-signing.md`
3. **Test full signing** — Run `cargo xtask sign` with Developer ID
4. **Test notarization** — Run `cargo xtask notarize --full`
5. **Configure CI/CD secrets** — Add required secrets to GitHub repository
6. **Trigger release build** — Push a tag to test the full pipeline
7. **Test in Ableton Live** — Verify signed plugin loads without warnings

### Architectural Documentation Updated

The following docs were updated to reflect the new build system:

| Document | Updates |
|----------|---------|
| `docs/architecture/high-level-design.md` | Added "Build System & Tooling" section; updated packaging notes, risks, roadmap |
| `docs/roadmap.md` | Milestone 4 marked as implementation complete |
| `docs/guides/macos-signing.md` | Created comprehensive signing guide |

---

## Issues & Blockers

| Issue | Status | Resolution |
|-------|--------|------------|
| *None currently* | | |
