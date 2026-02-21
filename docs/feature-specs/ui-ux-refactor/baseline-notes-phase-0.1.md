# Phase 0.1 Visual Baseline Notes — ui-ux-refactor

## 1. Capture Metadata

| Field                     | Value                              |
| ------------------------- | ---------------------------------- |
| Feature                   | `ui-ux-refactor`                   |
| Phase                     | `0.1 visual baseline capture`      |
| Capture date (UTC)        | `2026-02-21 18:45:02 UTC`          |
| Capture date (local)      | `2026-02-21 19:45:02 CET`          |
| Environment OS            | `macOS 26.2 (Build 25C56)`         |
| Kernel                    | `Darwin 25.2.0 (arm64)`            |
| URL                       | `http://localhost:5173`            |
| App title                 | `My Plugin`                        |
| Transport status observed | `Connected (websocket)`            |
| Audio status observed     | `running (full duplex) (ready)`    |
| Version badge observed    | `vdev`                             |

---

## 2. Baseline Screenshot Artifacts

| # | Description                              | Path |
|---|------------------------------------------|------|
| 1 | Full app page                            | `ui/test/visual-baseline/ui-ux-refactor/01-full-app-page.png` |
| 2 | Processor cards                          | `ui/test/visual-baseline/ui-ux-refactor/02-processor-cards.png` |
| 3 | Meter area                               | `ui/test/visual-baseline/ui-ux-refactor/03-meter-area.png` |
| 4 | Status badges                            | `ui/test/visual-baseline/ui-ux-refactor/04-status-badges.png` |
| 5 | Parameter controls (slider/select/toggle)| `ui/test/visual-baseline/ui-ux-refactor/05-parameter-controls-slider-select-toggle.png` |
| 6 | Focus visible — slider                   | `ui/test/visual-baseline/ui-ux-refactor/06-focus-visible-slider.png` |
| 7 | Focus visible — toggle button            | `ui/test/visual-baseline/ui-ux-refactor/07-focus-visible-toggle-button.png` |

---

## 3. Known Caveats / Observations

- Dynamic UI values in meter/latency regions may vary between captures due to live audio data.
- Processor cards screenshot is taller and noisier due to container sizing at time of capture.
- Loading placeholders briefly appear on initial load before full population of processor data.
- Focus captures were keyboard Tab-driven; focus ring appearance depends on keyboard navigation mode being active.

---

## 4. Operational Note

An existing running instance on port `:5173` was reused after a port-in-use condition was encountered during dev server startup. Dev server processes were terminated after all baseline screenshots were captured.
