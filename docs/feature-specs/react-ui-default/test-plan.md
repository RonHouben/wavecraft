# Test Plan: Make React UI Default

## Overview
- **Feature**: Remove webview_editor feature flag and make React UI the only plugin editor
- **Spec Location**: `docs/feature-specs/react-ui-default/`
- **Date**: 1 February 2026
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 1 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 7 |

## Prerequisites

### Automated Checks (Already Verified)
- [x] Build passes: `cargo build --workspace`
- [x] Tests pass: `cargo test --workspace`
- [x] UI tests pass: `cargo xtask test --ui` (35 tests)
- [x] Engine tests pass: `cargo xtask test --engine` (8 tests)
- [x] Lint passes: `cargo xtask lint`
- [x] Bundle builds: `cargo xtask bundle`
- [x] Release build: `cargo xtask bundle --release`

### Manual Testing Environment
- [ ] macOS system available
- [ ] Ableton Live installed (or equivalent VST3/AU host)
- [ ] Plugin built with `cargo xtask bundle --release`
- [ ] Plugin location: `engine/target/bundled/VstKit.vst3/`

## Test Cases

### TC-001: Build Without Feature Flags (Smoke Test)

**Description**: Verify that the plugin builds with React UI by default without any feature flags.

**Preconditions**:
- Clean build environment
- All code changes committed

**Steps**:
1. Run `cargo clean` to ensure fresh build
2. Run `cargo xtask bundle` (no feature flags)
3. Verify build succeeds
4. Check that plugin bundle exists at `engine/target/bundled/VstKit.vst3/`

**Expected Result**: Build succeeds, plugin bundle created with React UI

**Status**: ✅ PASS

**Actual Result**: 
- Build completed successfully in 13.20s
- React UI assets built (vite build succeeded)
- VST3 bundle created at `/Users/ronhouben/code/private/vstkit/engine/target/bundled/vstkit.vst3`
- CLAP bundle created at `/Users/ronhouben/code/private/vstkit/engine/target/bundled/vstkit.clap`
- No feature flags required
- Code signing applied automatically

**Notes**: Clean build from `cargo clean` verified that no feature flags are needed. React UI is now the default editor. 

---

### TC-002: Plugin Loads in Ableton Live (macOS)

**Description**: Verify the plugin loads correctly in a DAW without security warnings or errors.

**Preconditions**:
- Plugin built with `cargo xtask bundle --release`
- Ableton Live installed
- Plugin copied to `~/Library/Audio/Plug-Ins/VST3/` or scanned from target directory

**Steps**:
1. Open Ableton Live
2. Create new audio track
3. Load VstKit plugin from plugin browser
4. Observe loading behavior

**Expected Result**: 
- Plugin loads without security warnings
- No error dialogs
- Plugin window appears in Ableton's device view

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-003: React UI Renders Correctly

**Description**: Verify the React UI displays all expected components.

**Preconditions**:
- Plugin loaded in DAW (TC-002 passes)
- Plugin window visible

**Steps**:
1. Open VstKit plugin UI by clicking "Show Plugin Window" in Ableton
2. Observe UI elements:
   - Header with "VstKit" title and version badge
   - Two parameter sliders (Threshold, Ratio)
   - Four meter displays (Input L/R, Output L/R)
   - "Test IPC" button
3. Verify styling:
   - Dark theme (#1a1a1a background)
   - Accent color (#4a9eff)
   - Clean borders and spacing

**Expected Result**: 
- Full React UI renders
- All meters, sliders, and buttons visible
- UI matches design (dark theme, accent color)
- No layout issues or missing elements

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-004: Parameter Manipulation (Manual Control)

**Description**: Verify parameter sliders work correctly and sync with plugin state.

**Preconditions**:
- Plugin UI open (TC-003 passes)
- Audio not playing

**Steps**:
1. Move "Threshold" slider to -10 dB
2. Observe value display updates
3. Move "Ratio" slider to 4.0
4. Observe value display updates
5. Move sliders back to default positions
6. Verify smooth slider movement (no lag)

**Expected Result**: 
- Sliders respond smoothly to mouse input
- Value displays update in real-time
- No visual glitches or lag
- Parameter values match slider positions

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-005: DAW Automation Integration

**Description**: Verify that DAW automation controls sync with the React UI.

**Preconditions**:
- Plugin loaded in Ableton Live
- Plugin UI visible

**Steps**:
1. In Ableton, show plugin automation parameters
2. Enable automation recording
3. Play audio track
4. Automate "Threshold" parameter from DAW (not UI)
5. Observe UI slider movement during playback
6. Stop playback
7. Move UI slider manually
8. Verify automation lane updates in DAW

**Expected Result**: 
- DAW automation causes UI sliders to move
- UI slider movements create automation lanes
- Bidirectional sync works correctly
- No delays or desync issues

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-006: Audio Metering

**Description**: Verify that meter displays respond to audio signal.

**Preconditions**:
- Plugin loaded on audio track
- Audio file loaded on track (or live input available)
- Plugin UI visible

**Steps**:
1. Play audio through the plugin
2. Observe Input L/R meters:
   - Meters show green bars during audio
   - Peak levels reach appropriate dB values
3. Observe Output L/R meters:
   - Meters show processed signal
   - Levels reflect compression (if threshold exceeded)
4. Stop audio playback
5. Verify meters return to silence (-∞ dB)

**Expected Result**: 
- Input meters show incoming signal levels
- Output meters show processed signal
- Meters update in real-time (no lag)
- Peak holds work correctly
- Meters return to -∞ dB when silent
- Color coding: green (safe), yellow (warning), red (clip)

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-007: State Persistence (Save/Load Project)

**Description**: Verify plugin state persists when saving and reloading DAW projects.

**Preconditions**:
- Plugin loaded with modified parameters

**Steps**:
1. Set "Threshold" to -12 dB
2. Set "Ratio" to 6.0
3. Save Ableton project as "vstkit_state_test.als"
4. Close Ableton Live completely
5. Reopen Ableton Live
6. Load "vstkit_state_test.als"
7. Open VstKit plugin UI
8. Verify parameter values:
   - Threshold = -12 dB
   - Ratio = 6.0

**Expected Result**: 
- Plugin state saves with project
- All parameter values restored correctly
- UI reflects saved state on reload

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-008: Multiple Plugin Instances

**Description**: Verify multiple instances of the plugin work independently.

**Preconditions**:
- Ableton Live open

**Steps**:
1. Create Track 1, load VstKit instance
2. Set Track 1 Threshold to -15 dB
3. Create Track 2, load VstKit instance
4. Set Track 2 Threshold to -6 dB
5. Open both plugin UIs side-by-side
6. Verify each UI shows different parameter values
7. Play audio through both tracks simultaneously
8. Verify meters update independently
9. Change parameters in one instance
10. Verify other instance is unaffected

**Expected Result**: 
- Multiple instances load without conflicts
- Each instance has independent state
- Parameters don't cross-contaminate
- Metering works independently
- No crashes or UI glitches

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

## Issues Found

_(No issues found yet — will be populated during testing)_

---

## Testing Notes

### Environment Details
- **macOS Version**: {To be filled by user}
- **DAW**: Ableton Live {version - to be filled by user}
- **Plugin Format**: VST3
- **Build Type**: Release
- **Git Commit**: `17efa96cd6c8660e8b658064928fa405b0596d89`

### Testing Strategy
This is a **regression test** to ensure the removal of the `webview_editor` feature flag did not break existing functionality. The React UI was already validated in production — this test plan verifies:
1. Clean build without feature flags
2. No regressions in DAW loading
3. No regressions in UI rendering
4. No regressions in parameter/metering behavior

### Manual Testing Requirements
The user will need to execute test cases **TC-002 through TC-008**, as they require:
- GUI interaction (DAW, plugin UI)
- Visual verification (metering, styling)
- Audio playback (Ableton Live)

The tester agent will execute **TC-001** (build verification) via terminal.

---

## Sign-off

- [ ] All critical tests pass (TC-001, TC-002, TC-003)
- [ ] All functional tests pass (TC-004, TC-005, TC-006)
- [ ] State persistence works (TC-007)
- [ ] Multi-instance stability confirmed (TC-008)
- [ ] No critical issues found
- [ ] Ready for release: **PENDING TESTING**
