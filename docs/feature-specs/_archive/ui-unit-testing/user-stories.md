# UI Unit Testing Framework — User Stories

## Overview

Enable automated unit testing for the React UI layer using Vitest and React Testing Library. This establishes a testing foundation that prevents regressions, enables confident refactoring, and supports test-driven development.

---

## User Stories

### Story 1: Developer Runs UI Tests Locally

**As a** plugin developer  
**I want** to run UI unit tests with a single command  
**So that** I can verify my changes don't break existing functionality before committing

#### Acceptance Criteria
- [ ] `npm test` or `npm run test` runs all UI tests
- [ ] Tests execute in under 10 seconds for the initial test suite
- [ ] Test output clearly shows pass/fail status and failure details
- [ ] Watch mode available (`npm run test:watch`) for development

---

### Story 2: Developer Tests React Components in Isolation

**As a** plugin developer  
**I want** to test React components in isolation  
**So that** I can verify component behavior without running the full plugin

#### Acceptance Criteria
- [ ] React Testing Library configured for component rendering
- [ ] Components can be tested without IPC/engine dependencies
- [ ] Mock utilities available for IPC hooks (`useParameter`, `useMeter`, etc.)
- [ ] Example tests provided for at least 2 existing components

---

### Story 3: CI Pipeline Runs Tests Automatically

**As a** plugin developer  
**I want** UI tests to run automatically on every PR  
**So that** regressions are caught before merging

#### Acceptance Criteria
- [ ] Tests run in GitHub Actions CI workflow
- [ ] PR blocked if tests fail
- [ ] Test results visible in PR status checks

---

### Story 4: Developer Uses xtask for Unified Testing

**As a** plugin developer  
**I want** a unified `cargo xtask test` command  
**So that** I can run all tests (Rust + UI) from a single entry point

#### Acceptance Criteria
- [ ] `cargo xtask test` runs both Rust and UI tests
- [ ] `cargo xtask test --ui` runs only UI tests
- [ ] `cargo xtask test --engine` runs only Rust tests
- [ ] Exit code reflects overall pass/fail status

---

## Out of Scope

- End-to-end testing with real plugin (that's Milestone 6: Browser Testing)
- Visual regression testing (snapshot comparisons)
- Performance/benchmark testing
- Coverage thresholds (can be added later)

---

## Dependencies

- Existing `ui/` workspace with Vite + React + TypeScript
- Existing `cargo xtask` infrastructure

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Test execution time | < 10 seconds for initial suite |
| Component coverage | Example tests for ≥2 components |
| CI integration | Tests run on every PR |

---

## Notes

- Vitest is the recommended test runner (Vite-native, fast, Jest-compatible API)
- React Testing Library encourages testing behavior over implementation
- Focus on testing logic and interactions, not visual appearance (that's for M6)
- Mocking IPC is critical — tests must not require the Rust engine to run
