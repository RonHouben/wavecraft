# Visual Testing Guide

## Overview

This guide defines the required visual UI testing workflow for Wavecraft using the **VS Code integrated browser (Simple Browser)**.

For any UI/visual change, **screenshot-based in-app checks are required before testing sign-off**.

**Required policy:**

- Run the app in dev mode via `cargo xtask dev`
- Open `http://localhost:5173` in VS Code Simple Browser
- Capture screenshots of changed UI states
- Record screenshot evidence paths/references in `test-plan.md`

---

## Prerequisites

- VS Code workspace opened at repository root
- Dev servers available via `cargo xtask dev`
- A feature `test-plan.md` ready to store evidence

---

## Required Workflow (UI/Visual Changes)

1. **Start dev servers:**

   ```bash
   cargo xtask dev
   ```

2. **Open the app in VS Code integrated browser (Simple Browser):**
   - Open Command Palette
   - Run **Simple Browser: Show**
   - Enter: `http://localhost:5173`

3. **Validate changed UI in-app:**
   - Verify updated states, interactions, and layout in the integrated browser
   - Check at least the states directly affected by the change

4. **Capture screenshot evidence (required):**
   - Capture screenshots for each changed/verified state
   - Save screenshots to a reproducible local path (for example `artifacts/visual-testing/...`)
   - Add evidence references to `docs/feature-specs/{feature}/test-plan.md`

5. **Stop dev servers after testing:**

   ```bash
   pkill -f "cargo xtask dev"
   ```

---

## Minimum Evidence Set

For each UI/visual change, include:

- At least one screenshot for the **default/steady state**
- Screenshots for each **changed visual state** (e.g. hover/focus/active/disabled/error)
- For responsive/layout work, screenshots at relevant sizes
- A short pass/fail note for each captured state in `test-plan.md`

---

## Suggested Evidence Format in `test-plan.md`

Use a compact table per visual test case:

| Case   | State                 | Result  | Screenshot Evidence                            |
| ------ | --------------------- | ------- | ---------------------------------------------- |
| VT-001 | Default               | ✅ PASS | `artifacts/visual-testing/VT-001-default.png`  |
| VT-002 | Focus ring visible    | ✅ PASS | `artifacts/visual-testing/VT-002-focus.png`    |
| VT-003 | Disabled button style | ❌ FAIL | `artifacts/visual-testing/VT-003-disabled.png` |

---

## Test IDs and Stable Selection

Wavecraft UI components expose `data-testid` attributes for stable targeting and verification.

Examples:

- `app-root`
- `meter`, `meter-L`, `meter-R`
- `param-{id}`, `param-{id}-slider`
- `version-badge`
- `connection-status`

Use these IDs to guide what you validate and capture in screenshots.

---

## Troubleshooting

### Simple Browser cannot load `http://localhost:5173`

- Verify `cargo xtask dev` is running
- Check for port conflicts on `5173` and `9000`
- Restart dev servers

### UI stuck in connecting state

- Confirm WebSocket server started by `cargo xtask dev`
- Check terminal logs for startup errors

### Missing screenshot evidence

- Testing is **not complete** until screenshot references are recorded in `test-plan.md`

---

## Related Documentation

- [Development Workflows](../architecture/development-workflows.md)
- [Coding Standards — Testing](../architecture/coding-standards-testing.md)
- [Agent Development Flow](../architecture/agent-development-flow.md)
