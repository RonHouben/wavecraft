---
name: simple-browser-ui-testing
description: Execute required visual UI validation using VS Code integrated browser (Simple Browser) with screenshot evidence recorded in test-plan.md.
---

# Skill: Visual UI Testing via VS Code Simple Browser

## Required Policy

For any UI/visual change:

1. Start dev servers with `cargo xtask dev`
2. Open `http://localhost:5173` in **VS Code Simple Browser**
3. Validate changed UI states in-app
4. Capture screenshots of changed/verified states
5. Record screenshot evidence paths/references in `test-plan.md`
6. Stop dev servers via `pkill -f "cargo xtask dev"`

Testing is incomplete until screenshot evidence is documented.

## Quick Workflow

1. Start dev server:

   ```bash
   cargo xtask dev
   ```

2. In VS Code, open Command Palette and run **Simple Browser: Show**
3. Enter URL: `http://localhost:5173`
4. Validate changed visual states and interactions
5. Capture screenshots and store evidence paths
6. Document results and evidence in `docs/feature-specs/{feature}/test-plan.md`
7. Shut down servers:

   ```bash
   pkill -f "cargo xtask dev"
   ```

## Evidence Expectations

- Include screenshots for default and changed states
- Include screenshot references/paths in test-plan results
- Mark each visual case PASS/FAIL with supporting evidence

## Reference

- `docs/guides/visual-testing.md`
