---
name: playwright-mcp-ui-testing
description: Visual UI testing using Playwright MCP to interact with the Wavecraft web UI. Use this skill when testing UI components, verifying visual appearance, taking screenshots, or validating UI behavior that requires browser interaction. Requires the dev server running (cargo xtask dev).
---

# Skill: Visual UI Testing with Playwright MCP

Use Playwright MCP tools to visually test the Wavecraft UI during manual testing sessions.

## Prerequisites

1. **Dev server pre-check is required first** (reuse existing server when available)
2. **Playwright installed**: `cd ui && npm run playwright:install` (first time only)

## Starting Dev Server (Automated, with Pre-check)

Before starting any new process, the agent checks whether the server already exists:

```bash
# macOS/Linux pre-check (process OR port):
pgrep -f "cargo xtask dev" >/dev/null || lsof -ti tcp:5173 >/dev/null
```

- Exit code `0`: server is already running → reuse it, do not start another instance
- Exit code non-zero: server not running → start it as shown below

If no server is found, the agent starts the dev server automatically without user intervention:

```bash
# Agent runs this in background:
run_in_terminal(
  command="cd /Users/ronhouben/code/private/wavecraft && cargo run --manifest-path engine/xtask/Cargo.toml --release -- dev",
  isBackground=true
)
# Returns terminal_id for later status checks
```

**Wait for server startup** (only when a new server was started):
```bash
# Dev server needs ~5 seconds to compile and start Vite
sleep 5
# Playwright will handle connection verification
```

**Stopping the server** (when done testing):
```bash
# Only stop it if this session started a new instance:
pkill -f "cargo xtask dev"
```

## Quick Workflow

```
1. Pre-check server:     pgrep -f "cargo xtask dev" || lsof -ti tcp:5173
2. If running:           Reuse existing server
3. If not running:       run_in_terminal(..., isBackground=true)
4. Wait for startup:     sleep 5 (only if step 3 started a server)
5. Navigate to UI:       mcp_playwright_browser_navigate → http://localhost:5173
                         (Playwright will fail with timeout if server isn't ready)
6. Wait for load:        mcp_playwright_browser_wait_for → "Wavecraft" text
7. Get page state:       mcp_playwright_browser_snapshot
8. Take screenshot:      mcp_playwright_browser_take_screenshot
9. Interact:             mcp_playwright_browser_click, _type, etc.
10. Close browser:       mcp_playwright_browser_close
11. Stop server:         pkill -f "cargo xtask dev" (only if started in step 3)
```

## MCP Tool Reference

### Navigation & State

| Tool | Purpose | Example |
|------|---------|---------|
| `browser_navigate` | Open URL | `url: "http://localhost:5173"` |
| `browser_snapshot` | Get accessibility tree (preferred for interactions) | — |
| `browser_take_screenshot` | Capture PNG | `type: "png"`, `filename: "meter.png"` |
| `browser_wait_for` | Wait for text/time | `text: "Wavecraft"` or `time: 2` |

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

All Wavecraft components have `data-testid` attributes. Use with snapshot refs:

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
2. browser_wait_for → text: "Wavecraft"
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
| Page blank | Run the pre-check and confirm server is running on process/port before retrying |
| Port 5173 already in use | Reuse the existing server; do not start a second `cargo xtask dev` |
| Connection error | Check WebSocket server on port 9000 |
| Element not found | Use `browser_snapshot` to see current refs |
| Browser not installed | Run `mcp_playwright_browser_install` |

## Reference

Full test ID registry and baseline management: `docs/guides/visual-testing.md`
