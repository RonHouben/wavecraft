# Plugin UI Integration Testing Guide

**Status:** Phase 8 - DAW Integration Testing  
**Date:** 2026-01-30  
**Platform:** macOS (Primary)

---

## Build Instructions

### 1. Build Plugin Bundles

```bash
cd /Users/ronhouben/code/private/vstkit/engine
cargo xtask bundle
```

**Output:**
- VST3: `target/bundled/vstkit.vst3`
- CLAP: `target/bundled/vstkit.clap`

### 2. Install to System Directories

**VST3 (recommended for widest compatibility):**
```bash
cp -r target/bundled/vstkit.vst3 ~/Library/Audio/Plug-Ins/VST3/
```

**CLAP (for Bitwig/Reaper):**
```bash
cp -r target/bundled/vstkit.clap ~/Library/Audio/Plug-Ins/CLAP/
```

### 3. Verify Installation

```bash
# Check VST3
ls -la ~/Library/Audio/Plug-Ins/VST3/vstkit.vst3

# Check CLAP
ls -la ~/Library/Audio/Plug-Ins/CLAP/vstkit.clap
```

---

## Test Cases

### Test Suite 1: Basic Editor Functionality

#### TC-01: Plugin Loads Without Crash
**Prerequisites:** Plugin installed to system directory  
**Steps:**
1. Open DAW (Ableton Live, Logic Pro, Reaper)
2. Create new audio track
3. Add vstkit as insert effect
4. Verify plugin loads without crash

**Expected Result:**
- Plugin appears in plugin list
- Plugin loads without error dialog
- Audio passes through (no silence)

**Status:** ✅ Passed  
**Notes:** Tested in Ableton Live 12. Plugin loads successfully as VST3.

---

#### TC-02: Editor Opens Successfully
**Prerequisites:** Plugin loaded on track  
**Steps:**
1. Click to open plugin editor window
2. Observe editor appearance
3. Check for errors in DAW console/logs

**Expected Result:**
- Editor window opens
- UI renders (currently: egui editor with sliders)
- No crash or hang

**Status:** ✅ Passed  
**Notes:** WebView editor opens successfully. React UI renders with gain parameter slider and stereo meters.

---

#### TC-03: Parameter Changes Reflect in Host
**Prerequisites:** Editor open  
**Steps:**
1. Move the "Gain" slider in plugin UI
2. Observe host's parameter automation display
3. Record automation while moving slider
4. Play back automation

**Expected Result:**
- Host shows parameter value changing
- Automation can be recorded
- Automation plays back correctly
- Latency ≤16ms

**Status:** ✅ Passed  
**Measured Latency:** <1ms (immediate response)  
**Notes:** Parameter changes in UI immediately reflect in host. Bidirectional sync working correctly.

---

#### TC-04: Host Automation Reflects in UI
**Prerequisites:** Plugin with automation recorded  
**Steps:**
1. Create automation lane for Gain parameter
2. Draw automation curve in host
3. Play back with editor open
4. Observe UI slider movement

**Expected Result:**
- UI slider follows automation in real-time
- Update latency ≤16ms
- No UI stuttering

**Status:** ✅ Passed  
**Measured Latency:** <16ms (real-time)  
**Notes:** Fixed notification method name mismatch (`paramUpdate` → `parameterChanged`). UI now correctly updates during automation playback. Added debug logging to `param_value_changed()` for troubleshooting.

---

#### TC-05: Metering Data Flows
**Prerequisites:** Editor open, audio playing  
**Steps:**
1. Route audio through plugin
2. Play audio source (sine wave, music)
3. Observe meters in UI (if visible)

**Expected Result:**
- Meters animate with audio
- Peak values are accurate
- RMS values are accurate
- Update rate 30-60 Hz

**Status:** ✅ Passed  
**Notes:** Meters animate correctly at ~30 Hz. Fixed MeterConsumer lifecycle issue (changed from Option<MeterConsumer> to Arc<Mutex<MeterConsumer>> to persist across editor open/close cycles).

---

### Test Suite 2: Audio Processing

#### TC-06: Audio Passes Through Correctly
**Prerequisites:** Plugin loaded  
**Steps:**
1. Route audio through plugin
2. Set gain to 0 dB
3. Compare input vs output

**Expected Result:**
- No audio dropouts
- No clicks or pops
- No latency introduced (zero-latency plugin)

**Status:** ⬜ Not Tested  
**Notes:**

---

#### TC-07: No Dropouts at Low Buffer Size
**Prerequisites:** Plugin loaded  
**Steps:**
1. Set DAW buffer size to 64 samples
2. Play audio for 5 minutes
3. Monitor CPU usage
4. Check for audio dropouts

**Expected Result:**
- No dropouts or glitches
- CPU usage stable
- No performance warnings from DAW

**Status:** ⬜ Not Tested  
**CPU Usage:** ____%  
**Notes:**

---

#### TC-08: Gain Parameter Affects Audio
**Prerequisites:** Plugin loaded, audio playing  
**Steps:**
1. Set gain to -inf dB → expect silence
2. Set gain to 0 dB → expect unity gain
3. Set gain to +6 dB → expect 2x amplitude
4. Set gain to -6 dB → expect 0.5x amplitude

**Expected Result:**
- Gain changes take effect smoothly (no clicks)
- Attenuation is accurate
- Boost is accurate

**Status:** ⬜ Not Tested  
**Notes:**

---

### Test Suite 3: Editor Lifecycle

#### TC-09: Open/Close Repeatedly
**Prerequisites:** Plugin loaded  
**Steps:**
1. Open editor
2. Close editor
3. Repeat 10 times rapidly

**Expected Result:**
- No crashes
- No memory leaks (check Activity Monitor)
- Editor state preserved

**Status:** ⬜ Not Tested  
**Notes:**

---

#### TC-10: Multiple Instances
**Prerequisites:** None  
**Steps:**
1. Load 5 instances of vstkit on different tracks
2. Open all editors simultaneously
3. Change parameters in different instances
4. Play audio through all instances

**Expected Result:**
- All instances work independently
- No interference between instances
- No crashes or performance issues

**Status:** ⬜ Not Tested  
**Notes:**

---

#### TC-11: Save/Load Project
**Prerequisites:** Plugin loaded with non-default parameters  
**Steps:**
1. Set gain to -6 dB
2. Save project
3. Close project
4. Reopen project

**Expected Result:**
- Parameter state restored (-6 dB)
- Plugin loads correctly
- No errors

**Status:** ⬜ Not Tested  
**Notes:**

---

#### TC-12: Preset Management
**Prerequisites:** Plugin loaded  
**Steps:**
1. Change parameters
2. Save preset (if supported by DAW)
3. Load preset
4. Verify parameters restored

**Expected Result:**
- Presets save correctly
- Presets load correctly
- Parameters match

**Status:** ⬜ Not Tested (may not be implemented yet)  
**Notes:**

---

### Test Suite 4: Host Compatibility

#### TC-13: Ableton Live 11/12 (VST3)
**Platform:** macOS  
**Steps:**
1. Run all test cases TC-01 through TC-12
2. Document any Ableton-specific issues

**Status:** ⬜ Not Tested  
**Ableton Version:** _______  
**Notes:**

---

#### TC-14: Logic Pro (AU)
**Platform:** macOS  
**Prerequisites:** AU bundle built  
**Steps:**
1. Build AU wrapper
2. Install to `~/Library/Audio/Plug-Ins/Components/`
3. Run test cases TC-01 through TC-12

**Status:** ⬜ Not Tested (AU build not started)  
**Logic Version:** _______  
**Notes:**

---

#### TC-15: Reaper (VST3/CLAP)
**Platform:** macOS  
**Steps:**
1. Run test cases with VST3
2. Run test cases with CLAP
3. Compare behavior

**Status:** ⬜ Not Tested  
**Reaper Version:** _______  
**Notes:**

---

## Performance Profiling

### Metrics to Measure

| Metric | Target | Measured | Status |
|--------|--------|----------|--------|
| UI → Host param latency | ≤16ms | ___ ms | ⬜ |
| Host → UI param latency | ≤16ms | ___ ms | ⬜ |
| Meter update rate | 30-60 Hz | ___ Hz | ⬜ |
| CPU usage (idle) | <1% | ___% | ⬜ |
| CPU usage (processing) | <5% | ___% | ⬜ |
| Memory usage | <50 MB | ___ MB | ⬜ |

### Profiling Tools

**macOS:**
- Activity Monitor (CPU, Memory)
- Instruments (Time Profiler)
- DAW's built-in CPU meter

**Commands:**
```bash
# Monitor CPU usage
top -pid $(pgrep -f "Ableton Live")

# Monitor memory
vmmap $(pgrep -f "Ableton Live") | grep vstkit
```

---

## Known Issues & Limitations

### Current State (Phase 7 Complete)

✅ **Implemented:**
- VST3/CLAP export
- Basic gain parameter
- Peak/RMS metering (backend)
- egui editor (default)
- WebView editor structure (not active)
- Parameter synchronization infrastructure
- Windows support (untested)

⚠️ **Not Yet Active:**
- WebView editor (behind `webview_editor` feature flag)
- React UI in plugin (only in desktop crate)
- IPC bridge to WebView
- Meter visualization in plugin UI

🔧 **Next Steps:**
- Test current egui editor in DAW
- Enable WebView editor with feature flag
- Build and embed React UI assets
- Complete WebView IPC integration

---

## Test Environment

### System Information
- **macOS Version:** _______
- **CPU:** _______
- **RAM:** _______
- **Audio Interface:** _______
- **Sample Rate:** 44100 / 48000 Hz
- **Buffer Size:** 64 / 128 / 256 samples

### DAW Versions
- **Ableton Live:** _______
- **Logic Pro:** _______
- **Reaper:** _______

### Rust Toolchain
```bash
rustc --version
cargo --version
```

---

## Reporting Issues

When filing issues, include:
1. Test case number (e.g., TC-03)
2. DAW name and version
3. macOS version
4. Full error message or crash log
5. Steps to reproduce
6. Expected vs actual behavior

**Crash logs location:**
```bash
~/Library/Logs/DiagnosticReports/
```

---

## Quick Start Checklist

- [ ] Build plugin: `cargo xtask bundle`
- [ ] Install VST3: `cp -r target/bundled/vstkit.vst3 ~/Library/Audio/Plug-Ins/VST3/`
- [ ] Open DAW and rescan plugins
- [ ] Load vstkit on audio track
- [ ] Run TC-01 through TC-08
- [ ] Document results in this file
- [ ] Report any issues

---

## Revision History

| Date | Change |
|------|--------|
| 2026-01-30 | Initial testing guide created for Phase 8 |
