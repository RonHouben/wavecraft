# Milestone 4: macOS Hardening & Packaging — User Stories

> **Scope:** macOS + Ableton Live is the primary target. Windows/Linux support is deprioritized.

---

## Story 1: macOS Code Signing

**As a** plugin developer distributing VstKit plugins  
**I want** my plugins to be code-signed with a valid Apple Developer certificate  
**So that** users can install and run them without macOS security warnings blocking the plugin

### Acceptance Criteria

- [ ] Plugin bundles (`.vst3`, `.clap`, `.component`) are signed with a valid Developer ID certificate
- [ ] `codesign --verify --deep --strict` passes on all bundles
- [ ] Plugin loads in Ableton Live without "unidentified developer" warning
- [ ] Signing is integrated into `cargo xtask build` (or a dedicated `xtask sign` command)
- [ ] Documentation: How to set up signing for developers without Apple Developer accounts (ad-hoc signing for local dev)

### Notes

- Requires Apple Developer Program membership ($99/year)
- May need to handle entitlements for hardened runtime
- Consider environment variable for certificate identity (CI/CD friendly)

---

## Story 2: macOS Notarization

**As a** plugin developer distributing VstKit plugins  
**I want** my plugins to be notarized by Apple  
**So that** users on macOS Catalina+ can install them without Gatekeeper blocking the plugin entirely

### Acceptance Criteria

- [ ] Plugin bundles are submitted to Apple's notarization service
- [ ] Notarization ticket is stapled to the bundles
- [ ] `spctl --assess --type install` passes on all bundles
- [ ] Plugin loads on a fresh macOS install (no prior user approval)
- [ ] Notarization is integrated into `cargo xtask notarize` (or similar)
- [ ] CI/CD documentation for automated notarization

### Notes

- Requires code signing first (dependency on Story 1)
- Notarization can take 5-15 minutes — consider async workflow
- Need to handle notarization failures (common: unsigned nested code, hardened runtime issues)

---

## Story 3: Ableton Live Compatibility (macOS)

**As a** music producer using Ableton Live on macOS  
**I want** VstKit plugins to work flawlessly  
**So that** I can trust them in my production workflow without crashes, glitches, or unexpected behavior

### Acceptance Criteria

- [ ] Plugin loads and unloads cleanly (no crashes, no error dialogs)
- [ ] Parameter changes from UI reflect in Ableton's automation lanes
- [ ] Parameter automation from Ableton updates the plugin UI in real-time
- [ ] Project save/load correctly restores plugin state (parameters + UI state)
- [ ] Multiple plugin instances work correctly (no shared state issues)
- [ ] CPU usage is reasonable at 64-sample buffer size
- [ ] Plugin window resizing works correctly (if supported by Ableton's plugin window)
- [ ] Bypass/enable toggles work as expected

### Notes

- Test with Ableton Live 11 and 12 if possible
- Document any Ableton-specific quirks or workarounds
- Consider creating a test project that exercises all scenarios

---

## Story 4: AU Custom UI Investigation (Nice-to-Have)

**As a** Logic Pro / GarageBand user  
**I want** the VstKit plugin's custom React UI to appear instead of a generic parameter view  
**So that** I get the same polished experience as VST3/CLAP users

### Acceptance Criteria

- [ ] Root cause documented: Why does clap-wrapper show generic view?
- [ ] Feasibility assessment: Can clap-wrapper forward CLAP GUI extension to AU?
- [ ] If fixable: Custom UI appears in Logic Pro
- [ ] If not fixable: Document limitation and potential workarounds (standalone mode, etc.)

### Notes

- This is **lower priority** — generic view is functional, just not ideal
- May require upstream contribution to clap-wrapper
- Don't block Milestone 4 on this

---

## Story 5: State Persistence (AU)

**As a** Logic Pro user  
**I want** my plugin settings to save and restore correctly in `.aupreset` files  
**So that** I can recall my sounds and share presets

### Acceptance Criteria

- [ ] Saving an `.aupreset` captures all parameter values
- [ ] Loading an `.aupreset` restores the plugin to the saved state
- [ ] UI updates to reflect restored state
- [ ] No data loss or corruption on save/load cycles

### Notes

- Lower priority than VST3/CLAP state (which likely already works via nih-plug)
- Test with Logic Pro's "Save As..." and "Load..." preset functions

---

## Recommended Sprint Plan

| Week | Focus | Stories |
|------|-------|---------|
| **Week 1** | Signing infrastructure | Story 1 (Code Signing) |
| **Week 2** | Notarization + CI/CD | Story 2 (Notarization) |
| **Week 3** | Ableton deep dive | Story 3 (Ableton Compatibility) |
| **Week 4** | Polish + stretch goals | Story 4 & 5 if time permits |

---

## Priority Summary

| Priority | Story | Blocking? |
|----------|-------|-----------|
| 🔴 High | Story 1: Code Signing | Yes — users can't run unsigned plugins |
| 🔴 High | Story 2: Notarization | Yes — Gatekeeper blocks unnotarized code |
| 🔴 High | Story 3: Ableton Compatibility | Yes — primary target DAW |
| 🟡 Medium | Story 4: AU Custom UI | No — generic view is functional |
| 🟢 Low | Story 5: AU State Persistence | No — secondary DAW support |
