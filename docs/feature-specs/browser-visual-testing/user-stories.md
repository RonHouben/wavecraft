# User Stories: Browser-Based Visual Testing

## Overview

This milestone enables **visual testing and validation** of the VstKit UI during development. By integrating Playwright MCP, the development agent can programmatically capture screenshots, compare against baselines, and validate visual correctness of UI components and the full plugin interface.

**Key capabilities:**
- Agent-driven visual testing via Playwright MCP
- Screenshot capture at component and full-page levels
- External baseline storage for comparison
- Coverage of all major UI states

**Dependencies:** Milestone 6 (WebSocket IPC Bridge) — enables real engine data in browser

---

## Version

**Target Version:** `0.3.1` (patch bump from `0.3.0`)

**Rationale:** This is developer tooling infrastructure, not a user-facing feature change. Patch bump is appropriate per coding standards.

---

## User Story 1: Playwright MCP Setup

**As a** developer using the VstKit agent
**I want** Playwright MCP configured and ready to use
**So that** the agent can programmatically control a browser to inspect the UI

### Acceptance Criteria
- [ ] Playwright is installed as a dev dependency in `ui/`
- [ ] Playwright MCP server is configured and documented
- [ ] Agent can launch browser, navigate to UI, and take screenshots
- [ ] Documentation explains how to start the Playwright MCP server

### Notes
- Playwright MCP provides browser automation capabilities to the agent
- No custom test runner needed — agent uses MCP tools directly
- Works with `cargo xtask dev` (WebSocket server + Vite)

---

## User Story 2: Full-Page Visual Testing

**As a** developer validating the plugin UI
**I want** to capture and compare full-page screenshots
**So that** I can verify the overall layout and appearance of the plugin

### Acceptance Criteria
- [ ] Agent can capture full-page screenshot of the plugin UI
- [ ] Screenshots include all visible components (meters, parameters, version badge)
- [ ] Screenshots are captured at a consistent viewport size (e.g., 800x600)
- [ ] Agent can compare current screenshot against a stored baseline
- [ ] Visual differences are reported with actionable feedback

### Test Scenarios (Full-Page)
| Scenario | Description |
|----------|-------------|
| Default state | UI at startup with default parameter values |
| Active metering | Meters showing signal (low, medium, high levels) |
| Clipping state | Clipping indicator active and pulsing |
| Resized window | UI at different sizes (600x400, 800x600, 1024x768) |
| Disconnected state | WebSocket disconnected, status banner visible |

---

## User Story 3: Component-Level Visual Testing

**As a** developer validating individual UI components
**I want** to capture screenshots of specific components in isolation
**So that** I can verify component appearance without full-page context

### Acceptance Criteria
- [ ] Agent can target and screenshot individual components by selector
- [ ] Component screenshots exclude surrounding UI elements
- [ ] Supported components: Meter, ParameterSlider, VersionBadge, ResizeHandle, ConnectionStatus
- [ ] Agent can compare component screenshots against baselines

### Component Test Scenarios
| Component | States to Capture |
|-----------|-------------------|
| **Meter** | Silent (no signal), Low level (-40dB), Medium (-12dB), High (-3dB), Clipping (0dB+) |
| **ParameterSlider** | Minimum (0%), Middle (50%), Maximum (100%), Hover state, Drag state |
| **VersionBadge** | Default display (shows version number) |
| **ResizeHandle** | Default, Hover, Dragging |
| **ConnectionStatus** | Connected (hidden), Disconnected (visible), Reconnecting (visible) |

---

## User Story 4: External Baseline Storage

**As a** developer managing visual test baselines
**I want** baselines stored externally (not in git)
**So that** the repository stays lean and baselines can be managed independently

### Acceptance Criteria
- [ ] Baseline screenshots stored in a configurable external location
- [ ] Default location: `~/.vstkit/visual-baselines/` (local development)
- [ ] Baseline naming convention: `{component|page}_{state}_{viewport}.png`
- [ ] Agent can save new baselines when UI changes are intentional
- [ ] Agent can list existing baselines and their metadata
- [ ] `.gitignore` excludes any local baseline directories

### Baseline Management Commands
| Action | Description |
|--------|-------------|
| Capture baseline | Save current screenshot as new baseline |
| Compare | Compare current state against baseline, report diff |
| Update baseline | Replace baseline with current screenshot (after intentional change) |
| List baselines | Show all available baselines with timestamps |

---

## User Story 5: Visual Diff Reporting

**As a** developer investigating visual differences
**I want** clear reporting when the UI differs from baseline
**So that** I can quickly understand what changed and whether it's intentional

### Acceptance Criteria
- [ ] Diff report includes: baseline image, current image, highlighted differences
- [ ] Pixel difference percentage is calculated and reported
- [ ] Threshold for "acceptable" difference is configurable (default: 0.1%)
- [ ] Agent provides actionable guidance (e.g., "Update baseline?" or "Investigate regression")
- [ ] Diff images saved for review when differences detected

### Notes
- Minor anti-aliasing differences should not trigger failures
- Consider using perceptual diff algorithms (pixelmatch or similar)

---

## User Story 6: Development Workflow Integration

**As a** developer working on UI changes
**I want** a smooth workflow for visual testing during development
**So that** I can validate changes without context switching

### Acceptance Criteria
- [ ] Documented workflow: `cargo xtask dev` → browser opens → agent can test
- [ ] Agent can run visual tests on demand during development
- [ ] Quick feedback loop: capture → compare → report in seconds
- [ ] Agent can suggest which tests to run based on changed files

### Typical Workflow
1. Developer starts `cargo xtask dev` (WebSocket + Vite servers)
2. Browser opens to `http://localhost:5173`
3. Developer makes UI changes
4. Developer asks agent to run visual tests
5. Agent captures screenshots, compares to baselines
6. Agent reports results: "All good" or "Differences found in Meter component"
7. If intentional change: developer asks agent to update baseline

---

## User Story 7: Meter State Testing

**As a** developer validating the metering system
**I want** to test meter visualizations at various signal levels
**So that** I can ensure meters display correctly across the full dynamic range

### Acceptance Criteria
- [ ] Agent can trigger specific meter levels via parameter control (or test harness)
- [ ] Meter screenshots capture: peak bar, RMS bar, dB readout, clipping indicator
- [ ] Tests cover full range: silence → low → medium → high → clipping
- [ ] Peak hold behavior is visually validated
- [ ] Clipping indicator states are captured (inactive, active, held, click-to-reset)

### Technical Notes
- May require test mode in engine to generate specific meter values
- Alternatively, use real audio file playback if available
- Consider adding `setMeterTestLevel(dB)` IPC method for testing

---

## Out of Scope (This Milestone)

| Item | Reason |
|------|--------|
| CI/CD integration | Explicitly deferred per requirements |
| Cross-browser testing | Focus on Chromium; Safari/Firefox later |
| Automated test runner | Agent-driven via MCP, not automated suite |
| Performance benchmarking | Separate concern; visual correctness only |

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Major UI states covered | 15+ scenarios (full-page + component) |
| Baseline capture time | <2 seconds per screenshot |
| Comparison time | <1 second per comparison |
| False positive rate | <5% (threshold tuning) |

---

## Documentation Deliverables

- [ ] `docs/guides/visual-testing.md` — How to use Playwright MCP for visual testing
- [ ] Baseline naming conventions and storage location documented
- [ ] Workflow examples for common scenarios

