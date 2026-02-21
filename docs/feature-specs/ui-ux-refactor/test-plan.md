# Test Plan: UI/UX Refactor

## Related Documents

- [User Stories](./user-stories.md) — Feature requirements and acceptance criteria
- [Low-Level Design](./low-level-design-ui-ux-refactor-final.md) — Technical design decisions
- [Implementation Plan](./implementation-plan-final.md) — Step-by-step execution plan
- [Implementation Progress](./implementation-progress.md) — Phase completion status
- [Baseline Notes Phase 0.1](./baseline-notes-phase-0.1.md) — Visual baseline capture metadata
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions

---

## 1. Test Scope and Objectives

### 1.1 Scope

This test plan covers all six implementation phases of the `ui-ux-refactor` feature:

| Phase   | Description                                             |
| ------- | ------------------------------------------------------- |
| Phase 0 | Visual baseline captured + guardrails established       |
| Phase 1 | Focus ring and interaction-state foundations            |
| Phase 2 | Token audit + high-priority ad-hoc token cleanup        |
| Phase 3 | Canonical IPC constants + call-site migration           |
| Phase 4 | Smart/presentational split and hook ownership migration |
| Phase 5 | Resize ownership clarified in smart layer               |

**In scope:**

- `ui/packages/core` — IPC constants, hooks, utility exports
- `ui/packages/components` — Presentational components (Meter, ParameterSlider, ToggleButton, ParameterGroup, VersionBadge)
- `sdk-template/ui/` — Smart containers (App.tsx, processor smart wrappers)

**Out of scope:** Rust engine code, DSP behavior, transport architecture, plugin format support (VST3/CLAP/AU).

### 1.2 Objectives

1. Confirm no behavioral regressions were introduced in parameter state management or IPC communication.
2. Verify smart/presentational component boundary refactor does not break rendering or data flow.
3. Validate IPC string constants migration produces identical runtime behavior.
4. Confirm focus rings and interaction states render correctly in browser-dev mode.
5. Verify accessibility semantics remain correct after restructuring (keyboard navigation, ARIA roles).
6. Establish a documented visual baseline for future regression comparison.

---

## 2. Test Environment

| Field                 | Value                         |
| --------------------- | ----------------------------- |
| OS                    | macOS 26.2 (Build 25C56)      |
| Kernel                | Darwin 25.2.0 (arm64)         |
| Test date             | 2026-02-21                    |
| Dev server URL        | `http://localhost:5173`       |
| Transport mode        | WebSocket (browser-dev mode)  |
| App title observed    | My Plugin                     |
| Connection status     | Connected (websocket)         |
| Audio status          | running (full duplex) (ready) |
| Version badge         | vdev                          |
| Automated test runner | Vitest                        |
| Visual testing tool   | Playwright MCP                |

---

## 3. Automated Test Matrix and Results

### 3.1 Full Test Suite Run

**Command:** `cargo xtask ci-check`

| Phase                        | Result  | Details                                      |
| ---------------------------- | ------- | -------------------------------------------- |
| UI lint + typecheck          | ✅ PASS | ESLint, Prettier, `tsc --noEmit` — no errors |
| Rust fmt + clippy            | ✅ PASS | No warnings in production paths              |
| Automated tests (full suite) | ✅ PASS | 27 files, 108 tests passed                   |
| UI dist build                | ✅ PASS | Two-stage build: packages + app              |

### 3.2 Targeted Blocker Retest Run

Run against the specific test files covering the refactored areas after QA blockers were resolved.

| Test File Scope                          | Result      | Tests                 |
| ---------------------------------------- | ----------- | --------------------- |
| IPC constants / call-site migration      | ✅ PASS     | Included in 23        |
| Hook ownership and parameter data flow   | ✅ PASS     | Included in 23        |
| Smart/presentational component contracts | ✅ PASS     | Included in 23        |
| Interaction state and focus foundations  | ✅ PASS     | Included in 23        |
| **Total (targeted blockers)**            | **✅ PASS** | **4 files, 23 tests** |

### 3.3 Test Coverage Notes

- Parameter hook behavior (`useParameter`, `useAllParameters`, `useMeterFrame`) validated at unit level.
- IPC string constant migration verified: raw string literals removed from component call sites; all IPC method/event names now reference `IpcMethods`/`IpcEvents` constants from `@wavecraft/core`.
- No regressions in existing `useConnectionStatus`, `useHasProcessor`, or `useAvailableProcessors` hooks.

---

## 4. Manual Visual and Accessibility Checks

### 4.1 Visual Baseline Artifacts

Initial baseline captured on 2026-02-21 at 18:45:02 UTC as part of Phase 0.1. These serve as the pre-refactor ground truth for all subsequent visual comparison.

| #   | Description                               | Artifact                                                                                |
| --- | ----------------------------------------- | --------------------------------------------------------------------------------------- |
| 1   | Full app page                             | `ui/test/visual-baseline/ui-ux-refactor/01-full-app-page.png`                           |
| 2   | Processor cards                           | `ui/test/visual-baseline/ui-ux-refactor/02-processor-cards.png`                         |
| 3   | Meter area                                | `ui/test/visual-baseline/ui-ux-refactor/03-meter-area.png`                              |
| 4   | Status badges                             | `ui/test/visual-baseline/ui-ux-refactor/04-status-badges.png`                           |
| 5   | Parameter controls (slider/select/toggle) | `ui/test/visual-baseline/ui-ux-refactor/05-parameter-controls-slider-select-toggle.png` |
| 6   | Focus visible — slider                    | `ui/test/visual-baseline/ui-ux-refactor/06-focus-visible-slider.png`                    |
| 7   | Focus visible — toggle button             | `ui/test/visual-baseline/ui-ux-refactor/07-focus-visible-toggle-button.png`             |

### 4.2 QA Blocker Retest Visual Artifacts (2026-02-21)

A focused visual retest was conducted after QA blockers were resolved.

| Artifact                                        | Description                                | Result                                                     |
| ----------------------------------------------- | ------------------------------------------ | ---------------------------------------------------------- |
| `qa-retest-2026-02-21-full.png`                 | Full plugin UI — post-refactor state       | ✅ PASS — no visual regression                             |
| `qa-retest-2026-02-21-resize-focus.png`         | Resize + focus scenario                    | ✅ PASS — within expected browser-mode limitation (see §5) |
| `qa-retest-2026-02-21-snapshot-main.md`         | Accessibility tree snapshot — main view    | ✅ PASS — structure matches expected                       |
| `qa-retest-2026-02-21-snapshot-resize-focus.md` | Accessibility tree snapshot — resize/focus | ✅ PASS — structure matches expected                       |

### 4.3 Manual Visual Check Results

| TC   | Check                                                                                                   | Expected                     | Actual                      | Status  |
| ---- | ------------------------------------------------------------------------------------------------------- | ---------------------------- | --------------------------- | ------- |
| V-01 | Plugin title "My Plugin" renders correctly                                                              | H1, visible at top           | ✅ Confirmed                | ✅ PASS |
| V-02 | Version badge displays `vdev`                                                                           | Badge visible in header      | ✅ Confirmed                | ✅ PASS |
| V-03 | Connection status badge shows "Connected (websocket)"                                                   | Status visible in header     | ✅ Confirmed                | ✅ PASS |
| V-04 | Audio status displays "running (full duplex) (ready)"                                                   | Status visible               | ✅ Confirmed                | ✅ PASS |
| V-05 | Processor cards render with correct headings (Oscillator, Input Trim, Tone Filter, Soft Clip)           | H3 per card                  | ✅ Confirmed                | ✅ PASS |
| V-06 | Parameter sliders render with label + value display (e.g. "Frequency — 440.0 Hz")                       | Label + value above slider   | ✅ Confirmed                | ✅ PASS |
| V-07 | Toggle buttons render with correct label (e.g. "Enabled", "Oscillator Bypass")                          | Accessible button with label | ✅ Confirmed                | ✅ PASS |
| V-08 | Comboboxes render with options (Waveform: Sine/Square/Saw/Triangle; Mode: Low-pass/High-pass/Band-pass) | Select element with options  | ✅ Confirmed                | ✅ PASS |
| V-09 | Focus ring appears on keyboard Tab navigation to slider                                                 | Visible focus ring           | ✅ Confirmed (browser-mode) | ✅ PASS |
| V-10 | Focus ring appears on keyboard Tab navigation to toggle button                                          | Visible focus ring           | ✅ Confirmed (browser-mode) | ✅ PASS |

### 4.4 Accessibility Check Results

Accessibility semantics verified via Playwright accessibility tree snapshots.

| TC   | Check                                                             | Expected                                                    | Actual                                                                                        | Status  |
| ---- | ----------------------------------------------------------------- | ----------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ------- |
| A-01 | Plugin title exposed as `heading [level=1]`                       | `heading "My Plugin" [level=1]`                             | ✅ Match                                                                                      | ✅ PASS |
| A-02 | Processor group headings exposed as `heading [level=3]`           | One H3 per processor card                                   | ✅ Match (Oscillator, Input Trim, Tone Filter, Soft Clip, Gain, Output Monitor all confirmed) | ✅ PASS |
| A-03 | Parameter sliders exposed as `slider` role with accessible name   | `slider "Frequency"`, `slider "Level"`, etc.                | ✅ Match                                                                                      | ✅ PASS |
| A-04 | Toggle buttons exposed as `button` role with accessible name      | `button "Enabled"`, `button "Oscillator Bypass"`, etc.      | ✅ Match — `[cursor=pointer]` present                                                         | ✅ PASS |
| A-05 | Comboboxes exposed as `combobox` with accessible name and options | `combobox "Waveform"` with options Sine/Square/Saw/Triangle | ✅ Match                                                                                      | ✅ PASS |
| A-06 | Status indicators present in accessibility tree                   | Connection and audio status nodes                           | ✅ "Connected (websocket)" and "Audio: running (full duplex)" present                         | ✅ PASS |
| A-07 | No orphaned or unnamed interactive elements                       | All interactive elements have accessible names              | ✅ No unnamed elements observed                                                               | ✅ PASS |

---

## 5. Known Limitations

### 5.1 Browser-Mode Resize Limitation

**Limitation:** In browser-dev mode (WebSocket transport, `localhost:5173`), the plugin window does not behave as an embedded DAW window. Window resize behavior is governed by the browser viewport, not by the plugin host.

**Impact:** The resize-ownership refactor (Phase 5) improved the smart layer's handling of resize state and eliminated split ownership between declarative and legacy paths in the plugin binary. However, browser-mode visual screenshots cannot fully validate resize behavior in a real DAW context (Ableton Live / WKWebView).

**Mitigation:** Resize unit tests and accessibility tree snapshots confirm correct component structure after resize events in browser mode. Full resize validation in a live DAW requires a plugin bundle loaded in Ableton.

**Risk level:** Low — the resize concern was architectural (ownership clarity), not visual. No layout regressions were observed in browser-mode at any tested viewport width.

### 5.2 Live Audio Meter Variability

Meter and latency values in screenshots vary per capture due to live audio data. Visual comparisons of the meter area are treated as informational, not pixel-exact regression gates.

### 5.3 Focus Ring Browser-Mode Only

Focus ring visual tests were captured in browser-mode using keyboard Tab navigation. Focus ring appearance in WKWebView (plugin host) may differ slightly due to platform system-level focus styling. This is expected and consistent with pre-refactor behavior.

---

## 6. Test Results Summary

| Category                             | Total   | Passed  | Failed |
| ------------------------------------ | ------- | ------- | ------ |
| Automated — full suite (ci-check)    | 108     | 108     | 0      |
| Automated — targeted blocker retest  | 23      | 23      | 0      |
| Manual visual checks                 | 10      | 10      | 0      |
| Accessibility checks (tree snapshot) | 7       | 7       | 0      |
| **All checks**                       | **148** | **148** | **0**  |

---

## 7. Final Test Verdict and Release Recommendation

**Verdict: ✅ PASS — ready for release**

All automated and manual checks passed with no failures. The QA blocker retest (2026-02-21) confirmed resolution of all previously identified issues. Accessibility tree structure is correct across main and resize-focus scenarios. Visual artifacts show no regressions relative to the Phase 0.1 baseline.

### Release Recommendation

| Criterion                                      | Status |
| ---------------------------------------------- | ------ |
| All automated tests pass (108/108)             | ✅     |
| Targeted blocker tests pass (23/23)            | ✅     |
| Visual regression — no regressions vs baseline | ✅     |
| Accessibility semantics correct                | ✅     |
| Known limitations documented and risk-assessed | ✅     |
| No core audio or transport behavior changes    | ✅     |
| All six implementation phases complete         | ✅     |

**Recommendation:** Proceed with PR merge. The `ui-ux-refactor` feature is complete, tested, and safe to ship. The known resize limitation in browser mode is documented and does not block release — it is an environmental constraint of the test setup, not a regression.

---

## Follow-up: final-plan execution batch (S1–S7, 2026-02-21)

- This batch executed the final minimization slices from `implementation-plan-final.md` and `low-level-design-ui-ux-refactor-final.md`.
- Regression focus for this batch:
  - class/token deduplication and motion-safe transitions
  - shared parameter rendering extraction
  - meter channel consolidation with preserved test IDs
  - wrapper deletion + compat shims
  - polling/subscription helper consolidation
  - `WavecraftProvider` private-module decomposition
  - template wrapper collapse to direct `SmartProcessor`
- Verification outcomes for this batch are reported in the implementation run summary (targeted tests + full CI checks).

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
- [Roadmap](../../roadmap.md) — Milestone tracking

---

## Follow-up: final tester verification batch (wrapper shims + core decomposition + ci-check hang fix) — 2026-02-21

### Commands Run

```bash
cd ui && npm test -- packages/components/src/OscillatorProcessor.test.tsx packages/components/src/Processor.test.tsx packages/components/src/TemplateApp.test.tsx packages/core/src/context/WavecraftProvider.test.tsx packages/core/src/hooks/useParameter.test.ts packages/core/src/hooks/useAllParameters.test.ts packages/core/src/hooks/useAllParameterFor.test.ts
```

```bash
cargo xtask ci-check
```

```bash
cargo xtask ci-check --full
```

### Outcomes

| Check                                                   | Result  |
| ------------------------------------------------------- | ------- |
| Targeted tests (7 files, 23 tests)                      | ✅ PASS |
| `cargo xtask ci-check`                                  | ✅ PASS |
| `cargo xtask ci-check --full`                           | ✅ PASS |
| Template validation                                     | ✅ PASS |
| Git-source startup smoke path in `validate_template.rs` | ✅ PASS |
| CD dry-run                                              | ✅ PASS |

### Final Verdict

✅ **PASS**
