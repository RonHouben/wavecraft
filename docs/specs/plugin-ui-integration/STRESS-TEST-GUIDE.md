# Stress Testing Guide

**Purpose:** Verify plugin stability under real-time constraints (TC-07)  
**Date:** 2026-01-30  
**Platform:** macOS

---

## Prerequisites

1. Plugin installed: `~/Library/Audio/Plug-Ins/VST3/vstkit.vst3`
2. DAW: Ableton Live (or other VST3 host)
3. Audio interface configured
4. Test audio track ready

---

## Test Procedure: TC-07 - 5-Minute Stress Test @ 64 Samples

### 1. Configure DAW Settings

**Ableton Live:**
```
Preferences → Audio
- Buffer Size: 64 samples
- Sample Rate: 44100 Hz (or your interface's native rate)
```

**Expected latency:** ~1.5ms input + 1.5ms output

### 2. Setup Test Track

1. Create new audio track
2. Load `vstkit.vst3` on the track
3. Open the plugin editor (React UI should appear)
4. Load audio file or enable input monitoring

### 3. Start Monitoring

Before starting the test, note:
- CPU meter reading: _____%
- Memory usage: _____MB
- DAW buffer size: confirmed at 64 samples ✅

### 4. Run Test

**Duration:** 5 minutes continuous playback

**Monitor for:**
- ❌ Audio dropouts/glitches
- ❌ CPU overload warnings
- ❌ UI freezes
- ❌ Parameter lag
- ✅ Smooth meter animation
- ✅ Responsive parameter changes
- ✅ Stable CPU usage

**Actions during test:**
1. Adjust gain parameter several times
2. Watch meters respond to audio
3. Open/close plugin editor 2-3 times
4. Automate gain parameter (optional)

### 5. Record Results

**After 5 minutes:**

| Metric | Result | Notes |
|--------|--------|-------|
| Dropouts detected? | Yes/No | |
| CPU usage (avg) | ____% | |
| CPU usage (peak) | ____% | |
| Memory stable? | Yes/No | |
| UI responsive? | Yes/No | |
| Meters smooth? | Yes/No | |
| DAW warnings? | None/Details | |

---

## Success Criteria

✅ **PASS if:**
- No audio dropouts for full 5 minutes
- CPU usage stable (no spikes > 10%)
- No DAW performance warnings
- UI remains responsive throughout
- Meters update smoothly at 30Hz

❌ **FAIL if:**
- Any audio dropouts occur
- CPU spikes or overload warnings
- UI freezes or becomes unresponsive
- Meters stutter or stop updating

---

## Alternative Test: Stress with Multiple Instances

For additional verification:

1. Create 4-8 audio tracks
2. Load `vstkit.vst3` on each track
3. Play polyphonic audio
4. Monitor total CPU usage
5. Expected: Linear scaling (2x instances ≈ 2x CPU)

**Target:** < 50% CPU with 8 instances @ 64 samples

---

## Debugging Tips

**If dropouts occur:**

1. Check system audio settings (sample rate mismatch?)
2. Verify no background apps consuming CPU
3. Try 128-sample buffer (should work flawlessly)
4. Check Console.app for nih-plug log messages
5. Look for `[IPC]` or `[Asset Handler]` errors

**If CPU is high:**

1. Profile with Instruments (Time Profiler)
2. Check if UI rendering is on audio thread (shouldn't be!)
3. Verify ring buffer not blocking
4. Check for excessive logging

---

## Next Steps After Testing

### If PASS:
- ✅ Mark TC-07 complete in progress tracker
- Document CPU usage in testing guide
- Update status to "Ready for Production"
- Consider AU build for Logic Pro

### If FAIL:
- Document failure mode in detail
- Open issue in progress tracker
- Profile with Instruments to find bottleneck
- Review real-time safety in audio thread code

---

## Manual Profiling (Optional)

For detailed performance analysis:

```bash
# Open Instruments
open -a Instruments

# Select "Time Profiler" template
# Attach to DAW process
# Record during 5-minute test
# Analyze hotspots in vstkit code
```

Look for:
- Any `vstkit` functions in top 10 time consumers?
- Audio thread calling non-RT-safe functions?
- Lock contention in ring buffer?

---

## Reference

- **Implementation Progress:** `docs/specs/plugin-ui-integration/implementation-progress.md`
- **Testing Guide (Full):** `docs/specs/plugin-ui-integration/testing-guide.md`
- **Architecture:** `docs/architecture/high-level-design.md`

---

**Note:** This test is the final validation before considering the plugin-ui-integration feature complete for macOS VST3.
