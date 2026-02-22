---
name: simple-browser-ui-testing
description: Execute required visual UI validation using VS Code integrated browser (Simple Browser) with screenshot evidence recorded in test-plan.md.
---

# Skill: Visual UI Testing via VS Code Simple Browser

## Required Policy

For any UI/visual change:

1. Check whether a dev server is already running; reuse it if available
2. Open `http://localhost:5173` in **VS Code Simple Browser**
3. Validate changed UI states in-app
4. Capture screenshots of changed/verified states
5. Record screenshot evidence paths/references in `test-plan.md`
6. Stop dev servers via `pkill -f "cargo xtask dev"` only if you started them for this session

Testing is incomplete until screenshot evidence is documented.

## Dev Server Pre-check (macOS/Linux)

Run this check before starting anything:

```bash
pgrep -f "cargo xtask dev" >/dev/null || lsof -ti tcp:5173 >/dev/null
```

- Exit code `0`: server already running → reuse it
- Exit code non-zero: no server detected → start `cargo xtask dev`

## Quick Workflow

1. Check if server is already running:

   ```bash
   pgrep -f "cargo xtask dev" >/dev/null || lsof -ti tcp:5173 >/dev/null
   ```

2. If check passed, reuse the existing server. If not, start dev server:

   ```bash
   cargo xtask dev
   ```

3. In VS Code, open Command Palette and run **Simple Browser: Show**
4. Enter URL: `http://localhost:5173`
5. Validate changed visual states and interactions
6. Capture screenshots and store evidence paths
7. Document results and evidence in `docs/feature-specs/{feature}/test-plan.md`
8. If you started a new server in step 2, shut it down when done:

   ```bash
   pkill -f "cargo xtask dev"
   ```

## Evidence Expectations

- Include screenshots for default and changed states
- Include screenshot references/paths in test-plan results
- Mark each visual case PASS/FAIL with supporting evidence

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Port 5173 already in use | Re-run pre-check; if a server exists, reuse it instead of starting another |
| Page does not load | Confirm `http://localhost:5173` and wait for Vite startup if you just started the server |
| Accidentally stopped shared server | Restart with `cargo xtask dev` and rerun validation steps |

## Reference

- `docs/guides/visual-testing.md`
