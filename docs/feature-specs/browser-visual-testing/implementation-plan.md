# Implementation Plan: Browser-Based Visual Testing

## Overview

This plan implements visual testing infrastructure for VstKit using Playwright MCP. The implementation enables agent-driven screenshot capture, comparison against baselines, and validation of UI components.

**Target Version:** `0.3.1`

**Related Documents:**
- [User Stories](./user-stories.md)
- [Low-Level Design](./low-level-design-browser-visual-testing.md)

---

## Implementation Phases

```
Phase 1          Phase 2           Phase 3           Phase 4
Infrastructure → Test IDs       → Baseline Mgmt  → Documentation
(Playwright)     (Components)     (Storage/Diff)    (Guides)
    │                │                 │                │
    ▼                ▼                 ▼                ▼
  2 tasks          7 tasks          3 tasks          2 tasks
```

**Total:** 14 tasks across 4 phases

---

## Phase 1: Playwright Infrastructure

### Task 1.1: Install Playwright Dependencies

**File:** `ui/package.json`

**Action:** Add Playwright as a dev dependency

**Changes:**
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

**Dependencies:** None
**Risk:** Low
**Estimated effort:** 5 minutes

---

### Task 1.2: Create Playwright Configuration

**File:** `ui/playwright.config.ts` (new file)

**Action:** Create Playwright configuration for visual testing

**Content:**
```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/visual',
  fullyParallel: false,
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

  webServer: undefined,
});
```

**Dependencies:** Task 1.1
**Risk:** Low
**Estimated effort:** 5 minutes

---

### Task 1.3: Update .gitignore for Playwright

**File:** `ui/.gitignore`

**Action:** Add Playwright artifacts to gitignore

**Additions:**
```
# Playwright
playwright-report/
test-results/
```

**Dependencies:** None
**Risk:** Low
**Estimated effort:** 2 minutes

---

## Phase 2: Component Test IDs

Add `data-testid` attributes to all testable components for reliable Playwright selection.

### Task 2.1: Add Test ID to App Root

**File:** `ui/src/App.tsx`

**Action:** Add `data-testid="app-root"` to root div

**Before:**
```tsx
return (
  <div className="flex min-h-full flex-col bg-plugin-dark">
```

**After:**
```tsx
return (
  <div data-testid="app-root" className="flex min-h-full flex-col bg-plugin-dark">
```

**Dependencies:** None
**Risk:** Low
**Estimated effort:** 2 minutes

---

### Task 2.2: Add Test IDs to Meter Component

**File:** `ui/src/components/Meter.tsx`

**Action:** Add test IDs to meter container and all internal elements

**Elements to tag:**
- Meter container: `data-testid="meter"`
- Left channel row: `data-testid="meter-L"`
- Right channel row: `data-testid="meter-R"`
- Left peak bar: `data-testid="meter-L-peak"`
- Left RMS bar: `data-testid="meter-L-rms"`
- Right peak bar: `data-testid="meter-R-peak"`
- Right RMS bar: `data-testid="meter-R-rms"`
- Clip button: `data-testid="meter-clip-button"`
- Left dB display: `data-testid="meter-L-db"`
- Right dB display: `data-testid="meter-R-db"`

**Dependencies:** None
**Risk:** Low
**Estimated effort:** 15 minutes

---

### Task 2.3: Add Test IDs to ParameterSlider Component

**File:** `ui/src/components/ParameterSlider.tsx`

**Action:** Add test IDs using the parameter ID as suffix

**Elements to tag:**
- Container: `data-testid="param-{id}"`
- Label: `data-testid="param-{id}-label"`
- Slider input: `data-testid="param-{id}-slider"`
- Value display: `data-testid="param-{id}-value"`

**Example for `id="gain"`:**
```tsx
<div data-testid={`param-${id}`} className="...">
  <label data-testid={`param-${id}-label`} ...>
  <span data-testid={`param-${id}-value`} ...>
  <input data-testid={`param-${id}-slider`} ...>
</div>
```

**Dependencies:** None
**Risk:** Low
**Estimated effort:** 10 minutes

---

### Task 2.4: Add Test ID to VersionBadge Component

**File:** `ui/src/components/VersionBadge.tsx`

**Action:** Add `data-testid="version-badge"` to span

**Before:**
```tsx
return <span className="text-xs text-gray-500">v{__APP_VERSION__}</span>;
```

**After:**
```tsx
return <span data-testid="version-badge" className="text-xs text-gray-500">v{__APP_VERSION__}</span>;
```

**Dependencies:** None
**Risk:** Low
**Estimated effort:** 2 minutes

---

### Task 2.5: Add Test ID to ResizeHandle Component

**File:** `ui/src/components/ResizeHandle.tsx`

**Action:** Add `data-testid="resize-handle"` to button

**Before:**
```tsx
<button
  className={`group fixed bottom-1 right-5 ...`}
  onMouseDown={handleMouseDown}
```

**After:**
```tsx
<button
  data-testid="resize-handle"
  className={`group fixed bottom-1 right-5 ...`}
  onMouseDown={handleMouseDown}
```

**Dependencies:** None
**Risk:** Low
**Estimated effort:** 2 minutes

---

### Task 2.6: Add Test ID to ConnectionStatus Component

**File:** `ui/src/components/ConnectionStatus.tsx`

**Action:** Add `data-testid="connection-status"` to container div

**Before:**
```tsx
return (
  <div
    className={`flex items-center gap-2 rounded px-3 py-1.5 text-sm ${
```

**After:**
```tsx
return (
  <div
    data-testid="connection-status"
    className={`flex items-center gap-2 rounded px-3 py-1.5 text-sm ${
```

**Dependencies:** None
**Risk:** Low
**Estimated effort:** 2 minutes

---

### Task 2.7: Update Existing Unit Tests

**Files:** 
- `ui/src/components/Meter.test.tsx`
- `ui/src/components/ParameterSlider.test.tsx`
- `ui/src/components/VersionBadge.test.tsx`

**Action:** Update tests to use `data-testid` selectors where applicable

**Note:** Existing tests use `getByText`, `getByRole`, etc. We can optionally add tests that verify test IDs exist, but this is low priority.

**Dependencies:** Tasks 2.2, 2.3, 2.4
**Risk:** Low
**Estimated effort:** 10 minutes

---

## Phase 3: Baseline Storage & Management

### Task 3.1: Create Baseline Directory Structure

**Action:** Document the baseline directory structure for developers

The baseline storage is external (`~/.vstkit/visual-baselines/`), so we don't create files in the repo. Instead, we document the expected structure and the agent will create it on first use.

**Documented structure:**
```
~/.vstkit/
└── visual-baselines/
    ├── manifest.json
    ├── full-page/
    └── components/
        ├── meter/
        ├── parameter-slider/
        ├── version-badge/
        ├── resize-handle/
        └── connection-status/
```

**Dependencies:** None
**Risk:** Low
**Estimated effort:** 5 minutes (documentation)

---

### Task 3.2: Create Test ID Reference Document

**File:** `docs/guides/visual-testing.md` (new file)

**Action:** Document all test IDs and their selectors for agent reference

**Content includes:**
- Test ID registry (all components and their test IDs)
- Selector examples for Playwright MCP
- Baseline naming conventions
- Workflow instructions

**Dependencies:** Tasks 2.1-2.6
**Risk:** Low
**Estimated effort:** 30 minutes

---

### Task 3.3: Add Visual Testing to README

**File:** `README.md`

**Action:** Add brief mention of visual testing with link to guide

**Dependencies:** Task 3.2
**Risk:** Low
**Estimated effort:** 5 minutes

---

## Phase 4: Engine Test Support (Optional)

### Task 4.1: Add Debug Meter Level IPC Method

**File:** `engine/crates/bridge/src/handler.rs`

**Action:** Add `debug.setMeterLevel` method for testing specific meter states

**Note:** This is gated by `#[cfg(debug_assertions)]` so it's only available in debug builds.

```rust
#[cfg(debug_assertions)]
"debug.setMeterLevel" => {
    let level_db: f32 = params["level"].as_f64().unwrap() as f32;
    let channel: usize = params["channel"].as_u64().unwrap() as usize;
    // Set test level in meter producer
    Ok(json!({}))
}
```

**Dependencies:** None
**Risk:** Medium (requires MeterProducer changes)
**Estimated effort:** 30 minutes

**Decision:** Mark as **optional/deferred**. Visual testing can proceed without this by using CSS overrides or real audio. This task adds convenience but isn't blocking.

---

### Task 4.2: Version Bump

**File:** `engine/Cargo.toml`

**Action:** Bump version to `0.3.1`

**Before:**
```toml
[workspace.package]
version = "0.3.0"
```

**After:**
```toml
[workspace.package]
version = "0.3.1"
```

**Dependencies:** All other tasks complete
**Risk:** Low
**Estimated effort:** 2 minutes

---

## Implementation Order

```
┌─────────────────────────────────────────────────────────────────────┐
│  IMPLEMENTATION ORDER (Parallel where possible)                     │
└─────────────────────────────────────────────────────────────────────┘

Step 1: Infrastructure (can run in parallel)
├── Task 1.1: Install Playwright
├── Task 1.2: Create playwright.config.ts
└── Task 1.3: Update .gitignore

Step 2: Test IDs (can run in parallel)
├── Task 2.1: App.tsx
├── Task 2.2: Meter.tsx
├── Task 2.3: ParameterSlider.tsx
├── Task 2.4: VersionBadge.tsx
├── Task 2.5: ResizeHandle.tsx
└── Task 2.6: ConnectionStatus.tsx

Step 3: Validation
└── Task 2.7: Update unit tests (verify test IDs work)

Step 4: Documentation
├── Task 3.1: Document baseline structure
├── Task 3.2: Create visual-testing.md guide
└── Task 3.3: Update README

Step 5: Finalization
└── Task 4.2: Version bump to 0.3.1

Optional (Deferred):
└── Task 4.1: Debug meter level IPC
```

---

## Testing Strategy

### Unit Tests

Existing unit tests should continue to pass after adding `data-testid` attributes. Run:
```bash
cargo xtask test --ui
```

### Manual Verification

After implementation, verify:
1. `npm install` installs Playwright
2. `npm run playwright:install` downloads Chromium
3. All components render correctly with test IDs
4. Agent can use Playwright MCP to take screenshots

### Playwright MCP Verification

Agent should be able to:
1. Launch browser and navigate to `http://localhost:5173`
2. Take full-page screenshot
3. Take component screenshot using `[data-testid="meter"]`
4. Compare screenshots (when baselines exist)

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Test IDs break existing tests | Low | Low | Test IDs are additive, don't affect existing selectors |
| Playwright MCP compatibility | Medium | Medium | Test with agent early in Phase 1 |
| Baseline storage permissions | Low | Low | Use standard user config directory |
| Performance impact of test IDs | Very Low | None | Attributes have zero runtime cost |

---

## Success Criteria

- [ ] Playwright installed and configured
- [ ] All 6 components have `data-testid` attributes
- [ ] Existing unit tests pass (35 tests)
- [ ] Agent can take screenshots via Playwright MCP
- [ ] `visual-testing.md` guide created
- [ ] Version bumped to `0.3.1`

---

## Estimated Total Effort

| Phase | Tasks | Estimated Time |
|-------|-------|----------------|
| Phase 1: Infrastructure | 3 | 15 minutes |
| Phase 2: Test IDs | 7 | 45 minutes |
| Phase 3: Documentation | 3 | 40 minutes |
| Phase 4: Finalization | 1 | 5 minutes |
| **Total** | **14** | **~2 hours** |

---

## Notes for Coder

1. **Test ID Convention:** Use kebab-case for multi-word test IDs (e.g., `connection-status`, not `connectionStatus`)

2. **Dynamic Test IDs:** For parameterized components (like `ParameterSlider`), include the ID in the test ID (e.g., `param-gain`, `param-volume`)

3. **Nested Elements:** Use consistent suffixes: `-label`, `-slider`, `-value`, `-peak`, `-rms`, `-db`

4. **No Breaking Changes:** Adding `data-testid` attributes should not change component behavior or styling

5. **Playwright MCP:** The agent will use the MCP server directly — no custom test files needed in `ui/tests/visual/`

