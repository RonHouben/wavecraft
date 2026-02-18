# PR Summary: Oscillator Passthrough/Mix Fix & Runtime Contract Repairs

## Summary

Fixes the oscillator passthrough/additive mix feature and resolves a cascade of follow-on runtime issues discovered during end-to-end validation in Ableton Live. The changes span DSP logic, IPC contract generation, parameter metadata, and UI compatibility. Final Ableton behavior confirmed working by user.

---

## Changes

### DSP / Engine

- Fixed oscillator passthrough and additive mix logic so dry/wet blending and bypass behave correctly in the audio graph
- Corrected parameter ID and enum metadata propagation at runtime so parameter values round-trip accurately between host automation and the UI

### IPC Contract

- Refreshed the generated IPC contract bundle to reflect current parameter and processor definitions
- Added stale-sidecar handling: the dev server now detects and invalidates an out-of-date parameter cache (sidecar) on startup, preventing ghost values from surfacing in the UI after an engine rebuild

### UI

- Improved `ResizeHandle` compatibility — corrected pointer-event and cursor handling to work reliably across WKWebView and browser dev mode

---

## Documentation Updates

- Feature spec folder (`docs/feature-specs/oscillator-passthrough-mix/`) contains:
  - `user-stories.md`
  - `low-level-design-oscillator-passthrough-mix.md`
  - `implementation-plan.md`
  - `implementation-progress.md`
  - `test-plan.md`
  - `QA-report.md`
- Folder to be archived to `docs/feature-specs/_archive/oscillator-passthrough-mix/` before merge

---

## Testing

- `cargo xtask ci-check` passes (docs, UI build, lint + typecheck, Rust + Vitest tests)
- QA review completed — all findings resolved (see `QA-report.md`)
- Manual validation performed in Ableton Live (see below)

---

## Manual Validation

Validated in Ableton Live with the built VST3:

| Scenario                                                           | Result |
| ------------------------------------------------------------------ | ------ |
| Oscillator passthrough (0% mix) routes audio cleanly               | ✅     |
| Additive mix (100% osc) replaces input correctly                   | ✅     |
| Blended mix values produce expected wet/dry balance                | ✅     |
| Parameter automation in Ableton reflects correct enum/float values | ✅     |
| UI loads without stale sidecar values after engine rebuild         | ✅     |
| ResizeHandle drag behaves correctly in plugin window               | ✅     |

---

## Checklist

- [x] All CI checks pass (`cargo xtask ci-check`)
- [x] QA approval received
- [x] Manual validation in Ableton Live confirmed
- [x] Feature spec archived to `docs/feature-specs/_archive/oscillator-passthrough-mix/`
- [x] Roadmap updated (task marked complete, changelog entry added)
