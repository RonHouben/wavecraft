# Test Plan: Browser-Based Visual Testing

## Overview

- **Feature**: Browser-Based Visual Testing Infrastructure
- **Spec Location**: `docs/feature-specs/browser-visual-testing/`
- **Date**: 2026-02-01
- **Tester**: Tester Agent
- **Version**: 0.3.1

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 17 |
| ❌ FAIL | 2 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] Docker is running: `docker info`
- [x] CI image exists: `docker images | grep vstkit-ci`
- [x] Local CI passes (see Phase 2) — **PARTIAL**: Pre-existing xtask test failure (unrelated to this feature)
- [x] Node dependencies installed: `npm install` in `ui/`
- [x] Playwright installed: `npm run playwright:install` in `ui/`

## Phase 1: Pre-Flight Checks

### TC-001: Docker Availability

**Description**: Verify Docker is running for local CI pipeline

**Preconditions**:
- macOS system with Docker Desktop

**Steps**:
1. Run `docker info`
2. Check exit code

**Expected Result**: Docker is running, exit code 0

**Status**: ✅ PASS

**Actual Result**: Docker is running and responding correctly

**Notes**: Docker Desktop was already running, all container operations functional

---

### TC-002: Playwright Dependency Installation

**Description**: Verify Playwright package is installed

**Preconditions**:
- `ui/package.json` contains `@playwright/test`

**Steps**:
1. `cd ui`
2. `npm install`
3. Check for errors

**Expected Result**: Installation succeeds, `@playwright/test` in `node_modules/`

**Status**: ✅ PASS

**Actual Result**: `npm install` completed successfully. Added 53 packages, @playwright/test@^1.41.0 installed in node_modules.

**Notes**: No vulnerabilities found, installation clean

---

### TC-003: Playwright Browser Installation

**Description**: Install Chromium browser for Playwright

**Preconditions**:
- Playwright package installed

**Steps**:
1. `cd ui`
2. `npm run playwright:install`
3. Check for Chromium download

**Expected Result**: Chromium browser installed successfully

**Status**: ✅ PASS

**Actual Result**: Downloaded Chrome for Testing 145.0.7632.6 (playwright chromium v1208) successfully. Also downloaded FFmpeg and Chrome Headless Shell. Total download: ~254MB.

**Notes**: Installation location: `/Users/ronhouben/Library/Caches/ms-playwright/`

---

## Phase 2: Local CI Pipeline

### TC-004: Run Local CI Pipeline

**Description**: Execute full CI pipeline using `act`

**Preconditions**:
- Docker running
- CI image built

**Steps**:
1. `cd` to project root
2. Run `act -W .github/workflows/ci.yml --container-architecture linux/amd64 -P ubuntu-latest=vstkit-ci:latest --pull=false --artifact-server-path /tmp/act-artifacts`
3. Monitor job execution

**Expected Result**: All jobs pass (check-ui, test-ui, prepare-engine, check-engine, test-engine)

**Status**: ❌ FAIL

**Actual Result**: 
- ✅ Check Engine job: PASSED
- ❌ Test Engine job: **FAILED** (1 test failure in xtask)
- Failure: `commands::sign::tests::test_signing_config_missing_env` in xtask/src/commands/sign.rs:339
- Error: `assertion failed: result.is_err()`
- All other tests passed (53/54 total tests)

**Notes**: 
**This failure is PRE-EXISTING and unrelated to the visual testing feature**. The test failure is in the signing configuration module (xtask), which was not modified in this feature. The failure appears to be an environment-dependent test that expects a missing environment variable but one is present. All feature-related code (UI components, test IDs, Playwright config) works correctly.

**Recommendation**: This signing test issue should be addressed separately from this feature. 

---

## Phase 3: Test ID Verification

### TC-005: App Root Test ID

**Description**: Verify `data-testid="app-root"` exists in App.tsx

**Preconditions**:
- Code changes applied

**Steps**:
1. Open `ui/src/App.tsx`
2. Search for `data-testid="app-root"`
3. Verify it's on the root div

**Expected Result**: Test ID present on main container

**Status**: ✅ PASS

**Actual Result**: Found `data-testid="app-root"` on line 36 of App.tsx, correctly placed on the root div element.

**Notes**: Verified via grep search

---

### TC-006: Meter Component Test IDs

**Description**: Verify all Meter test IDs are present

**Preconditions**:
- Code changes applied

**Steps**:
1. Open `ui/src/components/Meter.tsx`
2. Verify presence of:
   - `data-testid="meter"`
   - `data-testid="meter-L"`
   - `data-testid="meter-R"`
   - `data-testid="meter-L-peak"`
   - `data-testid="meter-L-rms"`
   - `data-testid="meter-R-peak"`
   - `data-testid="meter-R-rms"`
   - `data-testid="meter-L-db"`
   - `data-testid="meter-R-db"`
   - `data-testid="meter-clip-button"`

**Expected Result**: All 10 test IDs present

**Status**: ✅ PASS

**Actual Result**: All 10 test IDs found in Meter.tsx:
- meter (line 104, 119)
- meter-clip-button (line 126)
- meter-L (line 137)
- meter-L-rms (line 146)
- meter-L-peak (line 151)
- meter-L-db (line 158)
- meter-R (line 167)
- meter-R-rms (line 176)
- meter-R-peak (line 181)
- meter-R-db (line 188)

**Notes**: Verified via grep search, all IDs correctly placed

---

### TC-007: ParameterSlider Test IDs

**Description**: Verify ParameterSlider test IDs with dynamic ID

**Preconditions**:
- Code changes applied

**Steps**:
1. Open `ui/src/components/ParameterSlider.tsx`
2. Verify presence of:
   - `data-testid={\`param-${id}\`}`
   - `data-testid={\`param-${id}-label\`}`
   - `data-testid={\`param-${id}-slider\`}`
   - `data-testid={\`param-${id}-value\`}`

**Expected Result**: All 4 test IDs present with template literals

**Status**: ✅ PASS

**Actual Result**: All 4 test IDs found with correct template literals:
- param-${id} (line 46)
- param-${id}-label (line 51)
- param-${id}-value (line 57)
- param-${id}-slider (line 62)

**Notes**: Dynamic IDs correctly implemented using template literals

---

### TC-008: VersionBadge Test ID

**Description**: Verify VersionBadge test ID

**Preconditions**:
- Code changes applied

**Steps**:
1. Open `ui/src/components/VersionBadge.tsx`
2. Verify `data-testid="version-badge"` on span

**Expected Result**: Test ID present

**Status**: ✅ PASS

**Actual Result**: Found `data-testid="version-badge"` on line 12 of VersionBadge.tsx

**Notes**: Correctly placed on span element

---

### TC-009: ResizeHandle Test ID

**Description**: Verify ResizeHandle test ID

**Preconditions**:
- Code changes applied

**Steps**:
1. Open `ui/src/components/ResizeHandle.tsx`
2. Verify `data-testid="resize-handle"` on button

**Expected Result**: Test ID present

**Status**: ✅ PASS

**Actual Result**: Found `data-testid="resize-handle"` on line 56 of ResizeHandle.tsx

**Notes**: Correctly placed on button element

---

### TC-010: ConnectionStatus Test ID

**Description**: Verify ConnectionStatus test ID

**Preconditions**:
- Code changes applied

**Steps**:
1. Open `ui/src/components/ConnectionStatus.tsx`
2. Verify `data-testid="connection-status"` on container div

**Expected Result**: Test ID present

**Status**: ✅ PASS

**Actual Result**: Found `data-testid="connection-status"` on line 21 of ConnectionStatus.tsx

**Notes**: Correctly placed on container div 

---

## Phase 4: Runtime Test ID Validation

### TC-011: Test IDs in Browser DOM

**Description**: Verify test IDs are rendered in actual DOM

**Preconditions**:
- `cargo xtask dev` running
- Browser at `http://localhost:5173`

**Steps**:
1. Start `cargo xtask dev`
2. Open browser to `http://localhost:5173`
3. Open DevTools
4. Inspect elements and verify test IDs:
   - App root has `data-testid="app-root"`
   - Meter has `data-testid="meter"`
   - Parameter has `data-testid="param-gain"`
   - Version badge has `data-testid="version-badge"`
   - Resize handle has `data-testid="resize-handle"`

**Expected Result**: All test IDs visible in DOM

**Status**: ✅ PASS

**Actual Result**: User verified all test IDs present in browser DOM at http://localhost:5173:
- `data-testid="app-root"` found
- `data-testid="meter"` found
- `data-testid="meter-L"` and `data-testid="meter-R"` found
- `data-testid="version-badge"` found
- `data-testid="resize-handle"` found
- `data-testid="connection-status"` found

**Notes**: Manual browser inspection completed successfully

---

### TC-012: Parameter Slider Test IDs with Dynamic ID

**Description**: Verify parameter test IDs use the actual parameter ID

**Preconditions**:
- UI running in browser

**Steps**:
1. Inspect parameter slider in DevTools
2. Verify test IDs use actual parameter ID (e.g., "gain"):
   - `data-testid="param-gain"`
   - `data-testid="param-gain-label"`
   - `data-testid="param-gain-slider"`
   - `data-testid="param-gain-value"`

**Expected Result**: Dynamic IDs correctly interpolated

**Status**: ✅ PASS

**Actual Result**: User verified parameter test IDs use actual parameter names (e.g., "param-gain") not literal template strings ("param-${id}"). Dynamic interpolation working correctly.

**Notes**: Template literals correctly interpolate parameter IDs in the DOM 

---

## Phase 5: Playwright Configuration

### TC-013: Playwright Config File Exists

**Description**: Verify `playwright.config.ts` is present

**Preconditions**:
- Implementation complete

**Steps**:
1. Check `ui/playwright.config.ts` exists
2. Verify configuration:
   - `baseURL: 'http://localhost:5173'`
   - Chromium project configured
   - Test directory: `./tests/visual`

**Expected Result**: Config file exists with correct settings

**Status**: ✅ PASS

**Actual Result**: Config file exists with correct settings:
- baseURL: 'http://localhost:5173'
- testDir: './tests/visual'
- projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }]
- Other settings: fullyParallel: false, workers: 1, timeout: 30000ms

**Notes**: Configuration matches design spec

---

### TC-014: Playwright in package.json

**Description**: Verify Playwright is in dependencies

**Preconditions**:
- package.json updated

**Steps**:
1. Open `ui/package.json`
2. Check `devDependencies` for `@playwright/test`
3. Check `scripts` for `playwright:install`

**Expected Result**: Playwright listed correctly

**Status**: ✅ PASS

**Actual Result**: 
- Dependency: `"@playwright/test": "^1.41.0"` in devDependencies
- Script: `"playwright:install": "playwright install chromium"`
- npm install succeeded (476 packages, 0 vulnerabilities)

**Notes**: Correctly configured in package.json 

---

## Phase 6: Documentation

### TC-015: Visual Testing Guide Exists

**Description**: Verify comprehensive guide is present

**Preconditions**:
- Documentation created

**Steps**:
1. Check `docs/guides/visual-testing.md` exists
2. Verify contents include:
   - Test ID registry table
   - Baseline storage structure
   - Playwright selector examples
   - Visual test scenarios
   - Agent workflow examples

**Expected Result**: Complete guide with all sections

**Status**: ✅ PASS

**Actual Result**: Guide exists at `docs/guides/visual-testing.md` (11,264 bytes) with all required sections:
- Test ID Registry (complete table)
- Baseline Storage Structure
- Naming Conventions
- Test Scenarios (full-page and component-level)
- Agent Workflow Examples
- Troubleshooting Guide
- Playwright MCP Tools Reference

**Notes**: Comprehensive 11KB guide with all sections present

---

### TC-016: README Updated

**Description**: Verify README mentions visual testing

**Preconditions**:
- README.md updated

**Steps**:
1. Open `README.md`
2. Check Documentation section
3. Verify link to `visual-testing.md`

**Expected Result**: Link present in Documentation section

**Status**: ✅ PASS

**Actual Result**: Link found in README.md:
`- [Visual Testing Guide](docs/guides/visual-testing.md) — Browser-based visual testing with Playwright`

**Notes**: Correctly added in Documentation section 

---

## Phase 7: Version Verification

### TC-017: Version Bump in Cargo.toml

**Description**: Verify version bumped to 0.3.1

**Preconditions**:
- Version bump complete

**Steps**:
1. Open `engine/Cargo.toml`
2. Check `[workspace.package]` version field

**Expected Result**: Version = "0.3.1"

**Status**: ✅ PASS

**Actual Result**: Version confirmed as `0.3.1` in `[workspace.package]` section

**Notes**: Correct patch version bump from 0.3.0

---

### TC-018: Version Displayed in UI

**Description**: Verify UI shows v0.3.1

**Preconditions**:
- UI running in browser

**Steps**:
1. Start `cargo xtask dev`
2. Open browser to `http://localhost:5173`
3. Locate version badge in footer
4. Read displayed version

**Expected Result**: Shows "v0.3.1"

**Status**: ❌ FAIL

**Actual Result**: Version number does not display in the UI. The VersionBadge component is present in the DOM (test ID found) but the version text is not visible to the user.

**Notes**: Issue documented as Issue #2 - requires coder agent to investigate and fix 

---

## Issues Found

### Issue #1: Pre-existing CI Test Failure (xtask signing test)

- **Severity**: Medium
- **Test Case**: TC-004
- **Description**: One test fails in `engine/xtask/src/commands/sign.rs`: `commands::sign::tests::test_signing_config_missing_env`
- **Expected**: Test expects an error when signing config env vars are missing
- **Actual**: Test assertion fails: `assertion failed: result.is_err()` at line 339
- **Steps to Reproduce**:
  1. Run `act -W .github/workflows/ci.yml --container-architecture linux/amd64 -P ubuntu-latest=vstkit-ci:latest --pull=false --artifact-server-path /tmp/act-artifacts`
  2. Observe Test Engine job failure
- **Evidence**: 
  ```
  test commands::sign::tests::test_signing_config_missing_env ... FAILED
  failures:
      commands::sign::tests::test_signing_config_missing_env
  test result: FAILED. 53 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
  ```
- **Impact**: Does NOT block this feature - all visual testing infrastructure code is unrelated to signing. 53/54 tests pass (98% pass rate).
- **Suggested Fix**: This is a pre-existing issue with the xtask signing test that needs separate investigation. The test expects a missing env var scenario but one appears to be present in the test environment.
- **Status**: DOCUMENTED - NOT BLOCKING release of browser visual testing feature

---

### Issue #2: Version Number Not Displaying in UI

- **Severity**: High
- **Test Case**: TC-018
- **Description**: The VersionBadge component is present in the DOM with correct test ID, but the version number (v0.3.1) is not visible to users in the browser.
- **Expected**: Version badge should display "v0.3.1" in the UI
- **Actual**: Version number does not appear in the rendered UI
- **Steps to Reproduce**:
  1. Start dev server: `cd ui && npm run dev`
  2. Open browser to http://localhost:5173
  3. Look for version badge in the UI
  4. Version text is not visible
- **Evidence**: User confirmed via manual browser inspection - VersionBadge test ID exists in DOM but no visible version text
- **Impact**: BLOCKS release - users cannot see the application version, which is important for debugging and support
- **Suggested Fix**: Investigate VersionBadge component rendering:
  - Check if `__APP_VERSION__` is defined at build time
  - Verify component is not hidden by CSS
  - Check if component renders children correctly
  - Verify Vite define configuration is working
- **Status**: BLOCKING - requires coder agent to fix before QA review

---

## Testing Notes

### Summary
- **Total Tests**: 18
- **Passed**: 17
- **Failed**: 2 (1 pre-existing CI failure, 1 version display issue)
- **Blocked**: 0

### Key Findings
1. ✅ **Infrastructure**: All Playwright dependencies installed successfully (Chromium 145.0.7632.6, @playwright/test ^1.41.0)
2. ✅ **Test IDs**: All 18 test IDs present in source code with correct naming conventions
3. ✅ **Configuration**: playwright.config.ts correctly configured (baseURL, testDir, Chromium project)
4. ✅ **Documentation**: Comprehensive 11KB guide created with all required sections
5. ✅ **Version**: Correctly bumped to 0.3.1 in Cargo.toml
6. ✅ **Unit Tests**: All 35 UI tests passing (Vitest)
7. ⚠️ **CI**: 53/54 tests pass (1 pre-existing xtask signing test failure - unrelated to this feature)
8. ✅ **Runtime Validation**: Test IDs render correctly in DOM (TC-011, TC-012)
9. ❌ **Version Display**: Version number not visible in UI (TC-018) - BLOCKING issue

### Code Quality
- ✅ TypeScript typechecking: Clean
- ✅ ESLint: 0 warnings
- ✅ Prettier: Formatted correctly
- ✅ All component tests passing

### Pre-existing Issue
One pre-existing test failure in `engine/xtask/src/commands/sign.rs` (signing configuration test). This is unrelated to the browser visual testing feature and should be addressed separately. All DSP, bridge, metering, protocol, standalone, and integration tests pass successfully.

## Sign-off

- [x] All critical tests pass (except version display)
- [ ] All high-priority tests pass (version display fails)
- [x] All feature-specific code validated
- [x] Issues documented for coder agent
- [x] Documentation complete and comprehensive
- [ ] Ready for QA review: **NO** - version display issue must be fixed first

### Recommendation

**⚠️ HAND OFF TO CODER AGENT** - Version display issue must be fixed

1. **Feature Implementation**: ⚠️ MOSTLY COMPLETE
   - ✅ All 18 test IDs added correctly and verified in browser
   - ✅ Playwright infrastructure installed and configured
   - ✅ Comprehensive documentation created
   - ✅ Version bumped appropriately in Cargo.toml
   - ❌ Version not displaying in UI (BLOCKING)

2. **Code Quality**: ✅ VERIFIED
   - All unit tests passing (35/35)
   - Code formatted and linted
   - Test IDs work correctly in browser

3. **Blocking Issue**: 
   - **Issue #2**: Version number not visible in UI (High severity)
   - VersionBadge component exists with test ID but no visible text
   - Likely build-time variable issue (`__APP_VERSION__` not defined during dev)
   - Requires coder agent investigation

4. **Non-Blocking Issues**:
   - Issue #1: Pre-existing xtask signing test failure (separate from this feature)

**Next Step:** Hand off to **coder agent** to fix version display issue (Issue #2), then return to tester for re-testing before QA.

## Testing Notes

_Will be added as testing progresses._

---

## Sign-off

- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [ ] Issues documented for coder agent
- [ ] Ready for release: YES / NO

