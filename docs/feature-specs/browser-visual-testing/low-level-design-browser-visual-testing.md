# Low-Level Design: Browser-Based Visual Testing

## Overview

This document describes the technical architecture for browser-based visual testing in VstKit. The system enables agent-driven visual validation using Playwright MCP, with external baseline storage and comprehensive coverage of UI states.

**Related Documents:**
- [User Stories](./user-stories.md)
- [High-Level Design](../../architecture/high-level-design.md)
- [Coding Standards](../../architecture/coding-standards.md)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           VISUAL TESTING ARCHITECTURE                           │
└─────────────────────────────────────────────────────────────────────────────────┘

  Developer / Agent                    Playwright MCP                Browser
  ┌─────────────┐                     ┌─────────────┐            ┌─────────────┐
  │             │  "take screenshot"  │             │  CDP       │             │
  │   Agent     │────────────────────►│  Playwright │───────────►│  Chromium   │
  │  (Copilot)  │                     │  MCP Server │            │             │
  │             │◄────────────────────│             │◄───────────│  VstKit UI  │
  └─────────────┘  screenshot.png     └─────────────┘            └──────┬──────┘
        │                                                               │
        │  "compare to baseline"                                        │ WebSocket
        ▼                                                               ▼
  ┌─────────────┐                                                ┌─────────────┐
  │  Baseline   │                                                │   Rust      │
  │  Storage    │                                                │   Engine    │
  │  (External) │                                                │  (xtask dev)│
  └─────────────┘                                                └─────────────┘
   ~/.vstkit/visual-baselines/
```

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Browser automation | Playwright MCP | Agent-native integration, no custom tooling |
| Baseline storage | External (`~/.vstkit/`) | Keep repo lean, independent versioning |
| Comparison | pixelmatch (via Playwright) | Industry standard, perceptual diff support |
| Test orchestration | Agent-driven | No automated runner; on-demand validation |
| Component isolation | CSS selectors + `data-testid` | Reliable targeting, framework-agnostic |

---

## Component Targeting Strategy

### Test ID Convention

All testable components must have a `data-testid` attribute for reliable selection:

```tsx
// Component implementation
export function Meter({ channel }: MeterProps) {
  return (
    <div data-testid={`meter-${channel}`} className="...">
      <div data-testid={`meter-${channel}-peak`} className="..." />
      <div data-testid={`meter-${channel}-rms`} className="..." />
      <div data-testid={`meter-${channel}-clip`} className="..." />
    </div>
  );
}
```

### Test ID Registry

| Component | Test ID Pattern | Selectable Elements |
|-----------|-----------------|---------------------|
| **Meter** | `meter-{L\|R}` | `meter-L`, `meter-R`, `meter-L-peak`, `meter-L-rms`, `meter-L-clip` |
| **ParameterSlider** | `param-{id}` | `param-gain`, `param-gain-slider`, `param-gain-value` |
| **VersionBadge** | `version-badge` | Single element |
| **ResizeHandle** | `resize-handle` | Single element |
| **ConnectionStatus** | `connection-status` | Single element |
| **App Root** | `app-root` | Full plugin UI container |

### Selector Examples

```typescript
// Full page
page.locator('[data-testid="app-root"]')

// Individual component
page.locator('[data-testid="meter-L"]')

// Nested element
page.locator('[data-testid="meter-L-clip"]')

// Parameter by ID
page.locator('[data-testid="param-gain"]')
```

---

## Baseline Storage Design

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

### Manifest Schema

```json
{
  "version": "1.0",
  "created": "2026-02-01T12:00:00Z",
  "updated": "2026-02-01T14:30:00Z",
  "appVersion": "0.3.1",
  "baselines": [
    {
      "id": "full-page/default_800x600",
      "path": "full-page/default_800x600.png",
      "viewport": { "width": 800, "height": 600 },
      "type": "full-page",
      "state": "default",
      "capturedAt": "2026-02-01T12:00:00Z",
      "hash": "sha256:abc123..."
    },
    {
      "id": "components/meter/clipping",
      "path": "components/meter/clipping.png",
      "selector": "[data-testid=\"meter-L\"]",
      "type": "component",
      "component": "meter",
      "state": "clipping",
      "capturedAt": "2026-02-01T12:05:00Z",
      "hash": "sha256:def456..."
    }
  ]
}
```

### Naming Convention

```
{type}_{state}_{viewport}.png       # Full-page
{state}.png                         # Component (in component subdirectory)
```

| Field | Description | Examples |
|-------|-------------|----------|
| `type` | `full-page` or component name | `full-page`, `meter`, `parameter-slider` |
| `state` | Visual state being captured | `default`, `clipping`, `hover`, `minimum` |
| `viewport` | Dimensions (full-page only) | `800x600`, `1024x768` |

---

## Visual Comparison Algorithm

### Comparison Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Capture    │────►│   Load      │────►│  Compare    │────►│   Report    │
│  Current    │     │  Baseline   │     │  (pixelmatch)│     │  Results    │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
      │                   │                   │                   │
      ▼                   ▼                   ▼                   ▼
  current.png        baseline.png      diff pixels         PASS/FAIL
                                       diff percentage     diff.png
```

### Comparison Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `threshold` | `0.1` | Color difference threshold (0-1) |
| `includeAA` | `false` | Include anti-aliasing differences |
| `alpha` | `0.5` | Diff overlay opacity |
| `diffColor` | `[255, 0, 255]` | Highlight color (magenta) |
| `aaColor` | `[255, 255, 0]` | Anti-aliasing diff color (yellow) |

### Acceptance Threshold

```typescript
const DIFF_THRESHOLD_PERCENT = 0.1; // 0.1% pixel difference allowed

function isAcceptable(diffPixels: number, totalPixels: number): boolean {
  const diffPercent = (diffPixels / totalPixels) * 100;
  return diffPercent <= DIFF_THRESHOLD_PERCENT;
}
```

### Diff Output

When differences are detected, three images are saved:

```
~/.vstkit/visual-baselines/.diff/
├── {baseline-id}_current.png      # Current capture
├── {baseline-id}_baseline.png     # Expected baseline
└── {baseline-id}_diff.png         # Visual diff overlay
```

---

## Test Scenarios

### Full-Page Scenarios

| ID | Scenario | Setup | Viewport |
|----|----------|-------|----------|
| `FP-01` | Default state | Fresh load, no interaction | 800×600 |
| `FP-02` | Metering active | Play audio or inject test signal | 800×600 |
| `FP-03` | Clipping state | Trigger clipping indicator | 800×600 |
| `FP-04` | Resized small | Resize to 600×400 | 600×400 |
| `FP-05` | Resized large | Resize to 1024×768 | 1024×768 |
| `FP-06` | Disconnected | Stop WebSocket server | 800×600 |

### Component Scenarios

#### Meter Component

| ID | State | Setup | Notes |
|----|-------|-------|-------|
| `M-01` | Silent | No signal | Bars at minimum |
| `M-02` | Low | -40dB signal | ~10% bar height |
| `M-03` | Medium | -12dB signal | ~60% bar height |
| `M-04` | High | -3dB signal | ~90% bar height |
| `M-05` | Clipping | 0dB+ signal | Full bar + clip indicator |

#### Parameter Slider

| ID | State | Setup | Notes |
|----|-------|-------|-------|
| `PS-01` | Minimum | Set value to 0% | Thumb at left |
| `PS-02` | Middle | Set value to 50% | Thumb centered |
| `PS-03` | Maximum | Set value to 100% | Thumb at right |
| `PS-04` | Hover | Mouse hover on thumb | Hover styling visible |
| `PS-05` | Dragging | Mouse down on thumb | Drag styling visible |

#### Connection Status

| ID | State | Setup | Notes |
|----|-------|-------|-------|
| `CS-01` | Connected | Normal operation | Banner hidden |
| `CS-02` | Disconnected | Stop WS server | Red banner visible |
| `CS-03` | Reconnecting | Stop then restart WS | Reconnecting message |

---

## React Component Changes

### Required Test ID Additions

Components need `data-testid` attributes for reliable selection:

#### App.tsx

```tsx
export function App() {
  return (
    <div data-testid="app-root" className="...">
      {/* existing content */}
    </div>
  );
}
```

#### Meter.tsx

```tsx
export function Meter({ channel, ...props }: MeterProps) {
  const testId = `meter-${channel}`;
  return (
    <div data-testid={testId} className="...">
      <div data-testid={`${testId}-peak`} ... />
      <div data-testid={`${testId}-rms`} ... />
      <div data-testid={`${testId}-clip`} ... />
      <span data-testid={`${testId}-db`} ... />
    </div>
  );
}
```

#### ParameterSlider.tsx

```tsx
export function ParameterSlider({ id, ...props }: Props) {
  const testId = `param-${id}`;
  return (
    <div data-testid={testId} className="...">
      <label data-testid={`${testId}-label`} ... />
      <input data-testid={`${testId}-slider`} ... />
      <span data-testid={`${testId}-value`} ... />
    </div>
  );
}
```

#### VersionBadge.tsx

```tsx
export function VersionBadge() {
  return (
    <span data-testid="version-badge" className="...">
      v{__APP_VERSION__}
    </span>
  );
}
```

#### ResizeHandle.tsx

```tsx
export function ResizeHandle({ ... }) {
  return (
    <button data-testid="resize-handle" className="...">
      {/* icon */}
    </button>
  );
}
```

#### ConnectionStatus.tsx

```tsx
export function ConnectionStatus({ status }: Props) {
  if (status === 'connected') return null;
  return (
    <div data-testid="connection-status" className="...">
      {/* status message */}
    </div>
  );
}
```

---

## Meter Test Signal Injection

To test meter visualizations at specific levels, we need a way to inject known signal levels.

### Option A: Engine Test Mode (Recommended)

Add a debug IPC method to set meter levels directly:

```rust
// In IpcHandler
fn handle_request(&self, method: &str, params: Value) -> Result<Value, IpcError> {
    match method {
        // ... existing methods ...
        
        #[cfg(debug_assertions)]
        "debug.setMeterLevel" => {
            let level_db: f32 = params["level"].as_f64().unwrap() as f32;
            let channel: usize = params["channel"].as_u64().unwrap() as usize;
            self.meter_producer.set_test_level(channel, level_db);
            Ok(json!({}))
        }
        
        _ => Err(IpcError::MethodNotFound)
    }
}
```

UI usage:

```typescript
// Set left channel to -12dB
await ipc.invoke('debug.setMeterLevel', { channel: 0, level: -12 });
```

### Option B: CSS Override for Visual Testing

For purely visual validation, override meter bar heights via injected CSS:

```typescript
// Inject test styling
await page.addStyleTag({
  content: `
    [data-testid="meter-L-peak"] { height: 60% !important; }
    [data-testid="meter-L-rms"] { height: 40% !important; }
  `
});
```

**Recommendation:** Use Option A (Engine Test Mode) for accurate testing, Option B as fallback for CI environments without engine.

---

## Development Workflow

### Typical Session

```
┌─────────────────────────────────────────────────────────────────────┐
│  1. Start Development Environment                                   │
│     $ cargo xtask dev                                               │
│     → WebSocket server on :9000                                     │
│     → Vite dev server on :5173                                      │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│  2. Agent Launches Browser via Playwright MCP                       │
│     → Navigate to http://localhost:5173                             │
│     → Wait for WebSocket connection                                 │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│  3. Capture / Compare Cycle                                         │
│     → Agent captures screenshot (full-page or component)            │
│     → Agent compares against baseline (if exists)                   │
│     → Agent reports: PASS, FAIL, or NO_BASELINE                     │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│  4. Handle Results                                                  │
│     PASS → Continue development                                     │
│     FAIL → Review diff, fix issue or update baseline               │
│     NO_BASELINE → Agent saves as new baseline                       │
└─────────────────────────────────────────────────────────────────────┘
```

### Agent Commands (Natural Language)

| Request | Agent Action |
|---------|--------------|
| "Take a screenshot of the full UI" | Capture full-page, save to temp |
| "Compare the meter component to baseline" | Capture meter, compare, report diff |
| "Save this as the new baseline for clipping state" | Save current capture to baselines |
| "Show me all available baselines" | List manifest.json entries |
| "Run all visual tests" | Iterate all baseline entries, compare each |
| "Update the baseline for parameter slider hover" | Replace existing baseline with current |

---

## File Structure

### New Files

```
ui/
├── playwright.config.ts           # Playwright configuration
├── package.json                   # Add @playwright/test dependency
└── src/
    └── components/
        ├── App.tsx                # Add data-testid="app-root"
        ├── Meter.tsx              # Add data-testid attributes
        ├── ParameterSlider.tsx    # Add data-testid attributes
        ├── VersionBadge.tsx       # Add data-testid attribute
        ├── ResizeHandle.tsx       # Add data-testid attribute
        └── ConnectionStatus.tsx   # Add data-testid attribute

docs/
└── guides/
    └── visual-testing.md          # Usage documentation

~/.vstkit/                         # External (not in repo)
└── visual-baselines/
    ├── manifest.json
    ├── full-page/
    └── components/
```

### Updated Files

| File | Changes |
|------|---------|
| `ui/package.json` | Add `@playwright/test` dev dependency |
| `ui/.gitignore` | Add `playwright-report/`, `test-results/` |
| `engine/Cargo.toml` | Version bump to `0.3.1` |
| All React components | Add `data-testid` attributes |

---

## Playwright Configuration

### playwright.config.ts

```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/visual',
  fullyParallel: false,  // Sequential for visual consistency
  forbidOnly: true,
  retries: 0,
  workers: 1,
  reporter: 'html',
  
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  // Don't start servers - assume `cargo xtask dev` is running
  webServer: undefined,
});
```

### package.json additions

```json
{
  "devDependencies": {
    "@playwright/test": "^1.41.0"
  },
  "scripts": {
    "playwright:install": "playwright install chromium"
  }
}
```

---

## Error Handling

### Missing Baseline

```
Status: NO_BASELINE
Message: No baseline found for 'components/meter/clipping'
Action: Save current capture as baseline? (Y/n)
```

### Comparison Failure

```
Status: FAIL
Message: Visual difference detected in 'full-page/default_800x600'
Diff: 2.3% pixels differ (threshold: 0.1%)
Files:
  - Baseline: ~/.vstkit/visual-baselines/full-page/default_800x600.png
  - Current:  ~/.vstkit/visual-baselines/.diff/full-page_default_800x600_current.png
  - Diff:     ~/.vstkit/visual-baselines/.diff/full-page_default_800x600_diff.png
Action: Review diff and either fix regression or update baseline
```

### Connection Failure

```
Status: ERROR
Message: Cannot connect to http://localhost:5173
Action: Ensure `cargo xtask dev` is running
```

---

## Security Considerations

| Concern | Mitigation |
|---------|------------|
| Baseline tampering | SHA-256 hash in manifest for integrity verification |
| Path traversal | Validate baseline paths, reject `..` sequences |
| External storage access | Use OS-standard config directory (`~/.vstkit/`) |

---

## Implementation Plan Outline

1. **Phase 1: Infrastructure**
   - Install Playwright, configure Playwright MCP
   - Create baseline directory structure
   - Add `data-testid` attributes to all components

2. **Phase 2: Capture & Storage**
   - Implement baseline capture workflow
   - Create manifest.json management
   - Document baseline naming conventions

3. **Phase 3: Comparison**
   - Implement visual diff with pixelmatch
   - Add threshold configuration
   - Generate diff images on failure

4. **Phase 4: Integration**
   - Add meter test signal injection (debug IPC)
   - Document development workflow
   - Create visual-testing.md guide

---

## Open Questions

| Question | Decision Needed |
|----------|-----------------|
| Should we add a `cargo xtask visual-test` command? | Likely unnecessary if agent-driven |
| Baseline sharing between developers? | Consider cloud storage for team use |
| Headless vs headed browser? | Headed for development, headless for CI (future) |

---

## Appendix: Playwright MCP Tools Reference

The Playwright MCP server exposes these tools for agent use:

| Tool | Description |
|------|-------------|
| `browser_navigate` | Navigate to URL |
| `browser_screenshot` | Capture screenshot (full page or element) |
| `browser_click` | Click element by selector |
| `browser_type` | Type text into input |
| `browser_hover` | Hover over element |
| `browser_wait_for` | Wait for selector/condition |
| `browser_evaluate` | Execute JavaScript in page context |

These tools enable the agent to fully automate visual testing without custom code.

