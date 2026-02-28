# VST UI Research Findings

## Purpose

This document summarizes practical visual UI patterns used in modern VST/audio plugins and translates them into implementation-ready guidance for the Wavecraft UI refactor.

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture context and platform constraints
- [Coding Standards](../../architecture/coding-standards.md) — Naming, conventions, and repo-wide standards
- [TypeScript & React Standards](../../architecture/coding-standards-typescript.md) — UI implementation conventions
- [CSS & Styling Standards](../../architecture/coding-standards-css.md) — Tailwind token and styling rules
- [Testing & Quality Standards](../../architecture/coding-standards-testing.md) — Validation expectations
- [Roadmap](../../roadmap.md) — Milestone context

## Scope and Method

- Focus: visual component patterns in plugin UIs (not DSP behavior).
- Sources: official vendor pages, host/device references, and reputable publication pages.
- Output intent: practical component decisions for React + Tailwind implementation.

---

## Condensed Research Summary

Modern plugin UIs consistently prioritize **dense information layouts**, **single-glance state readability**, and **direct-manipulation controls** (knobs/faders/XY/graphs) optimized for mouse + modifier precision and fast keyboard workflows. Across vendors, recurring choices are:

1. **Dark-biased canvases** for low-fatigue studio use, with bright, role-specific accents.
2. **Immediate state signaling** (armed, mapped, bypassed, clipping, loading) using shape + icon + color, not color only.
3. **Hybrid interaction surfaces**: compact “control strip” plus expandable advanced panel (graph/editor/browser).
4. **Strong meter visibility** and high refresh smoothness for confidence during gain staging.
5. **Hierarchical typography**: tiny labels, mid-size values, selective large numerics for critical readouts.

---

## Grouped Reference Links (links only)

> Note: links point to pages containing product UI screenshots, demos, or official imagery. No images are embedded here.

### 1) Official plugin vendor pages (primary visual references)

| Vendor/Product               | Why useful                                                  | Link                                                                      |
| ---------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------- |
| FabFilter Pro-Q 3            | Best-in-class spectrum/editor + dense control hierarchy     | https://www.fabfilter.com/products/pro-q-3-equalizer-plug-in              |
| FabFilter Pro-C 2            | Dynamics UI with clear state and metering language          | https://www.fabfilter.com/products/pro-c-2-compressor-plug-in             |
| Xfer Serum                   | Wavetable synth layout with macro/mod routing patterns      | https://xferrecords.com/products/serum                                    |
| iZotope Ozone                | Mastering workflow with modular panels and assistant states | https://www.izotope.com/en/products/ozone.html                            |
| Native Instruments Massive X | Modern synth with routing/macro discoverability             | https://www.native-instruments.com/en/products/komplete/synths/massive-x/ |
| Arturia Pigments             | Multi-engine synth, modulation color coding, panel grouping | https://www.arturia.com/products/software-instruments/pigments/overview   |
| u-he Diva                    | Classic synth metaphors translated to modern plugin UI      | https://u-he.com/products/diva/                                           |
| ValhallaDSP (plugins)        | Minimalist control sets + clear labeling in compact UIs     | https://valhalladsp.com/plugins/                                          |

### 2) DAW/host and device UI references

| Host/Device                     | Why useful                                                | Link                                                 |
| ------------------------------- | --------------------------------------------------------- | ---------------------------------------------------- |
| Ableton Live Devices            | Native in-DAW control density and readability conventions | https://www.ableton.com/en/live-manual/              |
| Logic Pro User Guide (Plug-ins) | Host-side plugin container and control expectations       | https://support.apple.com/guide/logicpro/welcome/mac |
| REAPER User Guide               | Dense but efficient parameter and window workflows        | https://www.reaper.fm/userguide.php                  |

### 3) Reputable publication reviews (secondary, comparative)

| Publication                   | Why useful                                            | Link                                         |
| ----------------------------- | ----------------------------------------------------- | -------------------------------------------- |
| Sound On Sound plugin reviews | Comparative screenshots and critical UX commentary    | https://www.soundonsound.com/reviews         |
| MusicTech plugin reviews      | Visual walkthroughs with practical usability notes    | https://www.musictech.com/reviews/           |
| Gearspace plugin discussions  | Real-user workflow pain points and recurring requests | https://gearspace.com/board/music-computers/ |

---

## Component-Type Usage Findings (what appears most and why)

| Component type                 | Typical usage frequency | Why common in plugin UIs                                                                                      |
| ------------------------------ | ----------------------: | ------------------------------------------------------------------------------------------------------------- |
| Rotary knob                    |               Very high | Space-efficient for many parameters, supports fine control via modifier drag, maps well to hardware metaphors |
| Linear fader/slider            |                    High | Excellent for gain/time ranges and quick relative comparison in channel-strip layouts                         |
| Buttons (momentary/latch)      |               Very high | Mode switching, transport-like actions, A/B, reset, learn, bypass                                             |
| Toggles/switches               |                    High | Binary states (on/off, mono/stereo, sync) with low cognitive load                                             |
| Meters (peak/RMS/LUFS)         |               Very high | Immediate confidence for signal safety and balancing decisions                                                |
| Tabs/segmented controls        |                    High | Progressive disclosure of complex pages without window sprawl                                                 |
| Graph editors (EQ/comp/filter) |             Medium-high | Direct visual manipulation improves understanding and speed for spectral/dynamic decisions                    |
| Envelope/LFO editors           |   Medium-high in synths | Motion/modulation programming requires shape + timing visualization                                           |
| Preset browser                 |                    High | Core workflow accelerator for recall, audition, and categorization                                            |
| Macro controls                 |             Medium-high | Reduces complexity by exposing “musical” top-level controls                                                   |
| XY pad                         |                  Medium | Powerful expressive control and performance automation in limited screen area                                 |
| Status/telemetry bar           |             Medium-high | Keeps transport/CPU/voice/mapping state visible without context switching                                     |

---

## Cross-Product Visual Pattern Conclusions

1. **State model depth is a key UX differentiator**
   - Mature plugins visually distinguish not just default/hover/active/disabled, but also routing/mapping/automation/bypass/error states.

2. **Numeric readouts remain critical even with rich visuals**
   - Users trust exact values during recall and matching tasks.

3. **Graphical editors increase speed when paired with explicit handles and constraints**
   - Cursor affordances, focus rings, and modifier hints reduce accidental edits.

4. **Metering is treated as a first-class component, not decoration**
   - Consistent scale, clipping thresholds, and decay behavior improve decision quality.

5. **Theme systems are expected to preserve role contrast under studio lighting**
   - Dark defaults dominate, but high-end products increasingly support lighter neutral variants.

---

## Practical Implications for Wavecraft Implementation

- Build around a reusable visual state contract that includes plugin-specific states (`bypassed`, `armed`, `mapped`).
- Prioritize knob, meter, and graph editor quality first; these are highest-impact trust surfaces.
- Keep tokens semantic (`surface`, `accent`, `danger`, `focus-ring`) and avoid one-off color usage.
- Use compact default density with optional relaxed spacing variant for browser/dev mode.
- Standardize value formatting and label placement for consistency across processors.

## Research-to-Design Handoff Notes

- This research supports the companion spec: `vst-component-spec-sheet.md`.
- For implementation-ready class decisions, use `vst-component-spec-sheet.md` section **9.1 Tailwind Token Mapping (aligned to `ui/tailwind.config.js`)**.
- Implementers should treat this document as rationale; concrete measurements and state definitions live in the spec sheet.
