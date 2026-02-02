---
name: playwright-mcp-ui-testing
description: Visual UI testing using Playwright MCP to interact with the VstKit web UI. Use this skill when testing UI components, verifying visual appearance, taking screenshots, or validating UI behavior that requires browser interaction. Requires the dev server running (cargo xtask dev).
---

# Skill: Visual UI Testing with Playwright MCP

Use Playwright MCP tools to visually test the VstKit UI during manual testing sessions.

## Prerequisites

1. **Start dev servers**: `cargo xtask dev` (WebSocket server + Vite)
2. **Playwright installed**: `cd ui && npm run playwright:install` (first time only)

## Quick Workflow

```
1. Start dev server:     cargo xtask dev
2. Navigate to UI:       mcp_playwright_browser_navigate → http://localhost:5173
3. Wait for load:        mcp_playwright_browser_wait_for → "VstKit" text
4. Get page state:       mcp_playwright_browser_snapshot
5. Take screenshot:      mcp_playwright_browser_take_screenshot
6. Interact:             mcp_playwright_browser_click, _type, etc.
7. Close when done:      mcp_playwright_browser_close
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
