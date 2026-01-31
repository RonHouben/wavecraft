# Implementation Progress: macOS Hardening & Packaging

> **Milestone:** 4  
> **Started:** TBD  
> **Last Updated:** 2026-01-31

---

## Progress Tracker

### Phase 1: Entitlements Configuration
| # | Task | Status | Notes |
|---|------|--------|-------|
| 1.1 | Create production entitlements (`entitlements.plist`) | ⏳ Not Started | |
| 1.2 | Create development entitlements (`entitlements-debug.plist`) | ⏳ Not Started | |

### Phase 2: Code Signing Command
| # | Task | Status | Notes |
|---|------|--------|-------|
| 2.1 | Add `chrono` dependency to xtask | ⏳ Not Started | |
| 2.2 | Create `sign.rs` command module | ⏳ Not Started | |
| 2.3 | Register sign module in `mod.rs` | ⏳ Not Started | |
| 2.4 | Add Sign CLI to `main.rs` | ⏳ Not Started | |

### Phase 3: Notarization Command
| # | Task | Status | Notes |
|---|------|--------|-------|
| 3.1 | Create `notarize.rs` command module | ⏳ Not Started | |
| 3.2 | Register notarize module in `mod.rs` | ⏳ Not Started | |
| 3.3 | Add Notarize CLI to `main.rs` | ⏳ Not Started | |

### Phase 4: Release Command
| # | Task | Status | Notes |
|---|------|--------|-------|
| 4.1 | Create `release.rs` command module | ⏳ Not Started | |
| 4.2 | Register release module in `mod.rs` | ⏳ Not Started | |
| 4.3 | Add Release CLI to `main.rs` | ⏳ Not Started | |

### Phase 5: CI/CD Pipeline
| # | Task | Status | Notes |
|---|------|--------|-------|
| 5.1 | Create GitHub Actions workflow | ⏳ Not Started | |
| 5.2 | Create signing documentation | ⏳ Not Started | |

### Phase 6: Testing & Validation
| # | Task | Status | Notes |
|---|------|--------|-------|
| 6.1 | Unit tests for signing module | ⏳ Not Started | |
| 6.2 | Unit tests for notarization module | ⏳ Not Started | |
| 6.3 | Manual integration testing | ⏳ Not Started | |

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

## Issues & Blockers

| Issue | Status | Resolution |
|-------|--------|------------|
| *None yet* | | |

---

## Notes

*Implementation notes will be added here as work progresses.*
