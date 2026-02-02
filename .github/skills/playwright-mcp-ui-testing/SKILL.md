---
name: playwright-mcp-ui-testing
description: Visual UI testing using Playwright MCP to interact with the VstKit web UI. Use this skill when testing UI components, verifying visual appearance, taking screenshots, or validating UI behavior that requires browser interaction. Requires the dev server running (cargo xtask dev).
---

# Skill: Visual UI Testing with Playwright MCP

Use Playwright MCP tools to visually test the VstKit UI during manual testing sessions.

## Prerequisites

1. **Dev servers will be auto-started** by the agent using background process
2. **Playwright installed**: `cd ui && npm run playwright:install` (first time only)

## Starting Dev Server (Automated)

The agent can start the dev server automatically without user intervention:

```bash
# Agent runs this in background:
run_in_terminal(
  command="cd /Users/ronhouben/code/private/vstkit && cargo run --manifest-path engine/xtask/Cargo.toml --release -- dev",
  isBackground=true
)
# Returns terminal_id for later status checks
```

**Wait for server startup**:
```bash
# Dev server needs ~5 seconds to compile and start Vite
sleep 5
# Playwright will handle connection verification
```

**Stopping the server** (when done testing):
```bash
# Kill the background process:
pkill -f "cargo xtask dev"
```

## Quick Workflow

```
1. Start dev server:     run_in_terminal(..., isBackground=true)
2. Wait for startup:     sleep 5
3. Navigate to UI:       mcp_playwright_browser_navigate → http://localhost:5173
                         (Playwright will fail with timeout if server isn't ready)
4. Wait for load:        mcp_playwright_browser_wait_for → "VstKit" text
5. Get page state:       mcp_playwright_browser_snapshot
6. Take screenshot:      mcp_playwright_browser_take_screenshot
7. Interact:             mcp_playwright_browser_click, _type, etc.
8. Close browser:        mcp_playwright_browser_close
9. Stop server:          pkill -f "cargo xtask dev"
```

## MCP Tool Reference

### Navigation & State

| Tool | Purpose | Example |
|------|---------|---------|
| `browser_navigate` | Open URL | `url: "http://localhost:5173"` |
| `browser_snapshot` | Get accessibility tree (preferred for interactions) | — |
| `browser_take_screenshot` | Capture PNG | `type: "png"`, `filename: "meter.png"` |
| `browser_wait_for` | Wait for text/time | `text: "VstKit"` or `time: 2` |

### Interactions

| Tool | Purpose | Key Parameters |
|------|---------|----------------|
| `browser_click` | Click element | `ref: "E123"` from snapshot |
| `browser_type` | Type text | `ref: "E123"`, `text: "value"` |
| `browser_hover` | Hover element | `ref: "E123"` |
| `browser_press_key` | Keyboard input | `key: "Enter"` |

### Lifecycle

| Tool | Purpose |
|------|---------|
| `browser_tabs` | List/create/close tabs |
| `browser_close` | Close browser |

## Test ID Selectors

All VstKit components have `data-testid` attributes. Use with snapshot refs:

| Component | Test ID | Usage |
|-----------|---------|-------|
| App root | `app-root` | Full page loaded |
| Meter | `meter` | Meter container |
| Left channel | `meter-L` | Left meter row |
| Clip button | `meter-clip-button` | Clipping indicator |
| Parameter | `param-{id}` | e.g., `param-gain` |
| Slider | `param-{id}-slider` | Range input |
| Version | `version-badge` | Version display |
| Connection | `connection-status` | WebSocket status |

## Common Test Patterns

### Verify Page Load

```
1. browser_navigate → http://localhost:5173
2. browser_wait_for → text: "VstKit"
3. browser_snapshot → verify app-root visible
```

### Screenshot Full Page

```
1. browser_take_screenshot → type: "png", fullPage: true
```

### Screenshot Component

```
1. browser_snapshot → find ref for [data-testid="meter"]
2. browser_take_screenshot → ref: "E123", element: "meter component"
```

### Verify Version Display

```
1. browser_snapshot → find version-badge element
2. Verify text matches expected version
```

### Test Slider Interaction

```
1. browser_snapshot → find param-gain-slider ref
2. browser_click → ref for slider
3. browser_type → new value
4. browser_snapshot → verify value updated
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Page blank | Verify `cargo xtask dev` is running |
| Connection error | Check WebSocket server on port 9000 |
| Element not found | Use `browser_snapshot` to see current refs |
| Browser not installed | Run `mcp_playwright_browser_install` |

## Reference

Full test ID registry and baseline management: `docs/guides/visual-testing.md`
