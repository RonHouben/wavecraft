# Visual Testing Guide

## Overview

This guide explains how to use Playwright MCP for visual testing of the VstKit UI during development. Visual testing enables screenshot capture, baseline comparison, and validation of UI components at various states.

**Requirements:**
- `cargo xtask dev` running (WebSocket server + Vite)
- Playwright MCP server configured
- Chromium browser installed via `npm run playwright:install`

---

## Quick Start

1. **Start the development servers:**
   ```bash
   cargo xtask dev
   ```
   This starts both the WebSocket server (port 9000) and Vite dev server (port 5173).

2. **Install Playwright (first time only):**
   ```bash
   cd ui
   npm run playwright:install
   ```

3. **Ask the agent to take a screenshot:**
   ```
   "Take a screenshot of the full plugin UI"
   "Capture the meter component"
   "Compare the current UI to the baseline"
   ```

The agent uses Playwright MCP to control the browser programmatically.

---

## Test ID Registry

All testable components have `data-testid` attributes for reliable selection.

### Component Test IDs

| Component | Test ID | Description |
|-----------|---------|-------------|
| **App Root** | `app-root` | Main application container |
| **Meter (container)** | `meter` | Meter component wrapper |
| **Meter L Channel** | `meter-L` | Left channel row |
| **Meter R Channel** | `meter-R` | Right channel row |
| **Meter L Peak** | `meter-L-peak` | Left peak bar |
| **Meter L RMS** | `meter-L-rms` | Left RMS bar |
| **Meter R Peak** | `meter-R-peak` | Right peak bar |
| **Meter R RMS** | `meter-R-rms` | Right RMS bar |
| **Meter L dB Display** | `meter-L-db` | Left dB readout |
| **Meter R dB Display** | `meter-R-db` | Right dB readout |
| **Meter Clip Button** | `meter-clip-button` | Clipping indicator button |
| **Parameter Container** | `param-{id}` | Parameter wrapper (e.g., `param-gain`) |
| **Parameter Label** | `param-{id}-label` | Parameter name label |
| **Parameter Slider** | `param-{id}-slider` | Range input element |
| **Parameter Value** | `param-{id}-value` | Displayed value |
| **Version Badge** | `version-badge` | Version display |
| **Resize Handle** | `resize-handle` | Window resize button |
| **Connection Status** | `connection-status` | WebSocket status indicator |

### Playwright Selectors

```typescript
// Full page
page.locator('[data-testid="app-root"]')

// Individual component
page.locator('[data-testid="meter"]')
page.locator('[data-testid="meter-L"]')
page.locator('[data-testid="param-gain"]')

// Nested elements
page.locator('[data-testid="meter-L-peak"]')
page.locator('[data-testid="param-gain-slider"]')
```

---

## Baseline Storage

Baselines are stored externally in your user directory, not in the git repository.

### Directory Structure

```
~/.vstkit/
└── visual-baselines/
    ├── manifest.json              # Baseline metadata
    ├── full-page/
    │   ├── default_800x600.png
    │   ├── metering-active_800x600.png
    │   ├── clipping_800x600.png
    │   ├── resized_600x400.png
    │   ├── resized_1024x768.png
    │   └── disconnected_800x600.png
    └── components/
        ├── meter/
        │   ├── silent.png
        │   ├── low.png
        │   ├── medium.png
        │   ├── high.png
        │   └── clipping.png
        ├── parameter-slider/
        │   ├── minimum.png
        │   ├── middle.png
        │   ├── maximum.png
        │   ├── hover.png
        │   └── dragging.png
        ├── version-badge/
        │   └── default.png
        ├── resize-handle/
        │   ├── default.png
        │   ├── hover.png
        │   └── dragging.png
        └── connection-status/
            ├── disconnected.png
            └── reconnecting.png
```

### Naming Convention

**Full-page screenshots:**
```
{scenario}_{viewport}.png
```
- Example: `default_800x600.png`, `metering-active_800x600.png`

**Component screenshots:**
```
{state}.png
```
- Example: `silent.png`, `hover.png`, `clipping.png`
- Stored in `components/{component-name}/` subdirectory

---

## Visual Test Scenarios

### Full-Page Scenarios

| ID | Scenario | Description | Setup |
|----|----------|-------------|-------|
| FP-01 | Default state | Fresh load, default parameters | Navigate to app, wait for connection |
| FP-02 | Metering active | Meters showing signal | Play audio or inject test signal |
| FP-03 | Clipping state | Clipping indicator visible | Trigger clipping (signal > 0dB) |
| FP-04 | Resized small | 600×400 viewport | Resize window to 600×400 |
| FP-05 | Resized large | 1024×768 viewport | Resize window to 1024×768 |
| FP-06 | Disconnected | WebSocket disconnected | Stop WebSocket server |

### Component Scenarios

#### Meter Component

| State | Description | Setup |
|-------|-------------|-------|
| Silent | No signal | Bars at minimum |
| Low | -40dB signal | ~10% bar height |
| Medium | -12dB signal | ~60% bar height |
| High | -3dB signal | ~90% bar height |
| Clipping | 0dB+ signal | Full bar + clip indicator |

#### Parameter Slider

| State | Description | Setup |
|-------|-------------|-------|
| Minimum | Value at 0% | Set parameter to 0.0 |
| Middle | Value at 50% | Set parameter to 0.5 |
| Maximum | Value at 100% | Set parameter to 1.0 |
| Hover | Mouse over thumb | Hover on slider thumb |
| Dragging | Dragging thumb | Mouse down on thumb |

#### Connection Status

| State | Description | Setup |
|-------|-------------|-------|
| Connected | Banner hidden | Normal operation |
| Disconnected | Red banner | Stop WebSocket server |
| Reconnecting | Reconnecting message | Stop then restart server |

---

## Agent Workflow Examples

### Capture Full-Page Screenshot

```
User: "Take a screenshot of the full plugin UI"

Agent:
1. Uses Playwright MCP to navigate to http://localhost:5173
2. Waits for `[data-testid="app-root"]` to be visible
3. Captures full-page screenshot
4. Saves to temp location or compares to baseline
```

### Capture Component Screenshot

```
User: "Capture the meter component"

Agent:
1. Navigates to app
2. Locates `[data-testid="meter"]`
3. Captures screenshot of element only
4. Compares to baseline in ~/.vstkit/visual-baselines/components/meter/
```

### Compare to Baseline

```
User: "Compare the current meter to baseline"

Agent:
1. Captures current meter screenshot
2. Loads baseline from ~/.vstkit/visual-baselines/components/meter/default.png
3. Compares using pixelmatch (0.1% threshold)
4. Reports: PASS, FAIL (with diff %), or NO_BASELINE
```

### Save New Baseline

```
User: "Save this as the new baseline for clipping state"

Agent:
1. Captures current screenshot
2. Saves to ~/.vstkit/visual-baselines/components/meter/clipping.png
3. Updates manifest.json with metadata
```

---

## Baseline Management

### Creating Baselines

When a baseline doesn't exist, the agent will prompt to save the current state as a new baseline.

```
Status: NO_BASELINE
Message: No baseline found for 'components/meter/clipping'
Action: Save current capture as baseline? (Y/n)
```

### Updating Baselines

When UI changes are intentional, update the baseline:

```
User: "Update the baseline for the default state"

Agent:
1. Captures current state
2. Replaces existing baseline
3. Updates manifest.json timestamp
```

### Listing Baselines

```
User: "Show me all available baselines"

Agent:
Lists all baselines from manifest.json with:
- ID (e.g., "full-page/default_800x600")
- Type (full-page or component)
- State
- Capture date
- App version
```

---

## Visual Diff Reporting

When differences are detected:

```
Status: FAIL
Message: Visual difference detected in 'components/meter/default'
Diff: 2.3% pixels differ (threshold: 0.1%)

Files:
  Baseline: ~/.vstkit/visual-baselines/components/meter/default.png
  Current:  ~/.vstkit/visual-baselines/.diff/meter_default_current.png
  Diff:     ~/.vstkit/visual-baselines/.diff/meter_default_diff.png

Action: Review diff and either:
  1. Fix regression in code
  2. Update baseline if change is intentional
```

**Diff Image Legend:**
- **Magenta pixels** — Significant differences
- **Yellow pixels** — Anti-aliasing differences (usually acceptable)

---

## Testing Meter States

To test meter visualizations at specific levels:

### Option A: Real Audio

Play audio through the plugin and observe meter response.

### Option B: CSS Override (Visual Testing Only)

For purely visual validation without engine:

```typescript
// Agent injects CSS to set meter bar heights
await page.addStyleTag({
  content: `
    [data-testid="meter-L-peak"] { height: 60% !important; }
    [data-testid="meter-L-rms"] { height: 40% !important; }
  `
});
```

### Option C: Debug IPC (Future)

Once implemented, the agent can set exact meter levels:

```typescript
// Set left channel to -12dB
await page.evaluate(() => {
  window.ipc.invoke('debug.setMeterLevel', { channel: 0, level: -12 });
});
```

---

## Troubleshooting

### Browser Won't Connect

**Problem:** Agent cannot navigate to `http://localhost:5173`

**Solution:**
- Verify `cargo xtask dev` is running
- Check Vite dev server logs
- Try manually opening `http://localhost:5173` in browser

### WebSocket Connection Failed

**Problem:** UI shows "Connecting..." indefinitely

**Solution:**
- Verify WebSocket server is running on port 9000
- Check for port conflicts: `lsof -i :9000`
- Restart `cargo xtask dev`

### Baseline Not Found

**Problem:** Agent reports "NO_BASELINE"

**Solution:**
- First time testing? Save current state as baseline
- Check `~/.vstkit/visual-baselines/` exists
- Verify `manifest.json` is valid JSON

### Screenshot Differences Too Sensitive

**Problem:** Minor pixel differences trigger failures

**Solution:**
- Increase threshold in comparison (default 0.1%)
- Ignore anti-aliasing differences (`includeAA: false`)
- Ensure consistent viewport size

---

## Playwright MCP Tools Reference

The agent uses these Playwright MCP tools:

| Tool | Purpose |
|------|---------|
| `browser_navigate` | Navigate to URL |
| `browser_screenshot` | Capture full-page or element screenshot |
| `browser_click` | Click element |
| `browser_type` | Type text into input |
| `browser_hover` | Hover over element |
| `browser_wait_for` | Wait for selector/condition |
| `browser_evaluate` | Execute JavaScript in page context |

---

## Related Documentation

- [Implementation Plan](../feature-specs/browser-visual-testing/implementation-plan.md)
- [Low-Level Design](../feature-specs/browser-visual-testing/low-level-design-browser-visual-testing.md)
- [User Stories](../feature-specs/browser-visual-testing/user-stories.md)

