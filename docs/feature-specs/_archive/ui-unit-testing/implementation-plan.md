# UI Unit Testing Framework — Implementation Plan

## Overview

This plan breaks down the UI unit testing framework implementation into discrete, testable tasks. Each task should be completable in one coding session and independently verifiable.

---

## Phase 1: Infrastructure Setup (Tasks 1–6)

### Task 1: Install Testing Dependencies

**Description:** Add Vitest, React Testing Library, and related packages to the UI workspace.

**Files to modify:**
- `ui/package.json`

**Steps:**
1. Install dev dependencies:
   ```bash
   cd ui
   npm install -D vitest happy-dom @testing-library/react @testing-library/jest-dom @testing-library/user-event @vitest/coverage-v8
   ```

**Verification:**
- [ ] `npm ls vitest` shows vitest installed
- [ ] `npm ls @testing-library/react` shows RTL installed
- [ ] No peer dependency warnings

---

### Task 2: Create Vitest Configuration

**Description:** Create the Vitest configuration file with happy-dom environment and proper paths.

**Files to create:**
- `ui/vitest.config.ts`

**Content:**
```typescript
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'happy-dom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
      exclude: ['src/test/**', '**/*.test.{ts,tsx}', '**/*.d.ts'],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
```

**Verification:**
- [ ] File exists at `ui/vitest.config.ts`
- [ ] TypeScript compiles without errors

**Dependencies:** Task 1

---

### Task 3: Create Test Setup File

**Description:** Create the global test setup file that runs before each test file.

**Files to create:**
- `ui/src/test/setup.ts`

**Content:**
```typescript
import '@testing-library/jest-dom';

// Global test setup runs before each test file
```

**Verification:**
- [ ] File exists at `ui/src/test/setup.ts`
- [ ] Imports compile without errors

**Dependencies:** Task 1

---

### Task 4: Create IPC Mock Module

**Description:** Create the mock module for IPC hooks that allows tests to control parameter and meter state.

**Files to create:**
- `ui/src/test/mocks/ipc.ts`

**Content:** See low-level design for full implementation. Key exports:
- `setMockParameter(id, value)` — Set a parameter value for tests
- `setMockMeterFrame(frame)` — Set meter data for tests
- `resetMocks()` — Clear all mock state
- Mock implementations of `useParameter`, `useMeter`, etc.

**Verification:**
- [ ] File exists at `ui/src/test/mocks/ipc.ts`
- [ ] TypeScript compiles without errors
- [ ] Exports match the real `ipc.ts` interface

**Dependencies:** Task 1

---

### Task 5: Update TypeScript Configuration

**Description:** Add Vitest types to TypeScript configuration for global test functions.

**Files to modify:**
- `ui/tsconfig.json`

**Changes:**
- Add `"vitest/globals"` to `compilerOptions.types`
- Add `"@testing-library/jest-dom"` to `compilerOptions.types`

**Verification:**
- [ ] `describe`, `it`, `expect` recognized without imports
- [ ] `toBeInTheDocument()` matcher recognized
- [ ] No TypeScript errors in test files

**Dependencies:** Task 1

---

### Task 6: Add npm Test Scripts

**Description:** Add test-related scripts to package.json.

**Files to modify:**
- `ui/package.json`

**Scripts to add:**
```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage",
    "test:ui": "vitest --ui"
  }
}
```

**Verification:**
- [ ] `npm test` runs (may fail if no tests exist yet)
- [ ] `npm run test:watch` starts watch mode
- [ ] `npm run test:coverage` generates coverage report

**Dependencies:** Tasks 1–5

---

## Phase 2: Example Tests (Tasks 7–9)

### Task 7: Create ParameterSlider Test

**Description:** Write unit tests for the ParameterSlider component demonstrating component testing patterns.

**Files to create:**
- `ui/src/components/ParameterSlider.test.tsx`

**Test cases:**
1. Renders with label
2. Displays current parameter value
3. Updates value on slider change
4. Handles min/max bounds correctly

**Verification:**
- [ ] `npm test` passes
- [ ] All 4 test cases pass
- [ ] Tests run in < 2 seconds

**Dependencies:** Tasks 1–6

---

### Task 8: Create Meter Test

**Description:** Write unit tests for the Meter component demonstrating timer-based testing patterns.

**Files to create:**
- `ui/src/components/Meter.test.tsx`

**Test cases:**
1. Renders peak level bars
2. Shows clip indicator when peak >= 1.0
3. Clip indicator clears after timeout
4. Displays dB values correctly

**Verification:**
- [ ] `npm test` passes
- [ ] All test cases pass
- [ ] Fake timers work correctly

**Dependencies:** Tasks 1–6

---

### Task 9: Create Pure Function Tests

**Description:** Write unit tests for utility functions (audio math, helpers).

**Files to create:**
- `ui/src/lib/audio-math.test.ts` (if audio-math.ts exists)
- Or tests for any existing utility functions in `ui/src/lib/`

**Test cases:**
- Test linearToDb/dbToLinear conversions
- Test edge cases (0, negative, infinity)
- Test roundtrip accuracy

**Verification:**
- [ ] `npm test` passes
- [ ] All utility tests pass

**Dependencies:** Tasks 1–6

---

## Phase 3: xtask Integration (Tasks 10–11)

### Task 10: Add Test Subcommand to xtask

**Description:** Add a `test` subcommand to the xtask CLI with `--ui` and `--engine` flags.

**Files to modify:**
- `engine/xtask/src/main.rs`

**Changes:**
1. Add `Test` variant to `Commands` enum
2. Add `TestArgs` struct with `--ui` and `--engine` flags
3. Implement `cmd_test()` function

**Verification:**
- [ ] `cargo xtask test --help` shows usage
- [ ] `cargo xtask test --ui` runs UI tests
- [ ] `cargo xtask test --engine` runs Rust tests
- [ ] `cargo xtask test` runs both

**Dependencies:** Phase 2 complete (so there are tests to run)

---

### Task 11: Update xtask Documentation

**Description:** Update xtask help text and any related documentation.

**Files to modify:**
- `engine/xtask/src/main.rs` (help text)
- `README.md` or relevant docs (if they document xtask commands)

**Verification:**
- [ ] `cargo xtask --help` lists `test` command
- [ ] Help text is clear and accurate

**Dependencies:** Task 10

---

## Phase 4: CI Integration (Task 12)

### Task 12: Add UI Tests to GitHub Actions

**Description:** Add UI test step to the CI workflow so tests run on every PR.

**Files to modify or create:**
- `.github/workflows/test.yml` (create new) OR
- `.github/workflows/ci.yml` (add to existing)

**Workflow steps:**
1. Checkout code
2. Setup Node.js 20
3. Install UI dependencies (`npm ci`)
4. Run UI tests (`npm test`)
5. (Optional) Upload coverage report

**Verification:**
- [ ] Push branch and create PR
- [ ] GitHub Actions runs test workflow
- [ ] Tests pass in CI
- [ ] PR shows test status check

**Dependencies:** Phase 2 complete

---

## Task Summary

| Task | Description | Dependencies | Estimated Time |
|------|-------------|--------------|----------------|
| 1 | Install dependencies | — | 5 min |
| 2 | Vitest config | 1 | 10 min |
| 3 | Test setup file | 1 | 5 min |
| 4 | IPC mock module | 1 | 20 min |
| 5 | TypeScript config | 1 | 5 min |
| 6 | npm scripts | 1–5 | 5 min |
| 7 | ParameterSlider test | 1–6 | 30 min |
| 8 | Meter test | 1–6 | 30 min |
| 9 | Pure function tests | 1–6 | 15 min |
| 10 | xtask test command | Phase 2 | 30 min |
| 11 | xtask documentation | 10 | 10 min |
| 12 | CI workflow | Phase 2 | 20 min |

**Total estimated time:** ~3 hours

---

## Success Criteria

All acceptance criteria from user stories must pass:

- [ ] `npm test` runs all UI tests
- [ ] Tests execute in under 10 seconds
- [ ] Watch mode available
- [ ] Components testable without engine
- [ ] Mock utilities for IPC hooks
- [ ] Example tests for ≥2 components
- [ ] `cargo xtask test` runs all tests
- [ ] `--ui` and `--engine` flags work
- [ ] CI runs tests on PRs
- [ ] PR blocked if tests fail
