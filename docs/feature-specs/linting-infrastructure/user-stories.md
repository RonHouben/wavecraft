# Linting Infrastructure — User Stories

> **Epic:** Add comprehensive linting for both UI (TypeScript/React) and Engine (Rust) with unified xtask commands and CI integration.

---

## US-1: ESLint Setup for UI

**As a** plugin developer working on the React UI  
**I want** ESLint configured with strict TypeScript/React rules  
**So that** I catch bugs early and maintain consistent code quality

### Acceptance Criteria

- [ ] ESLint installed and configured in `ui/` directory
- [ ] TypeScript ESLint parser configured (`@typescript-eslint/parser`)
- [ ] Recommended rules enabled:
  - `eslint:recommended`
  - `@typescript-eslint/recommended`
  - `@typescript-eslint/strict` (no `any`, strict type checking)
  - `react-hooks/recommended` (exhaustive deps)
  - `react/recommended`
- [ ] `npm run lint` script added to `package.json`
- [ ] `npm run lint:fix` script for auto-fixing issues
- [ ] All existing UI code passes linting (or issues documented)

### Notes

- ESLint 9 uses flat config (`eslint.config.js`) — use this format
- Consider `eslint-plugin-react-refresh` for Vite HMR compatibility

---

## US-2: Prettier Setup for UI

**As a** plugin developer working on the React UI  
**I want** Prettier configured for consistent code formatting  
**So that** formatting is automated and consistent (like `cargo fmt` for Rust)

### Acceptance Criteria

- [ ] Prettier installed in `ui/` directory
- [ ] `.prettierrc` config file with sensible defaults:
  - Single quotes
  - Semicolons
  - 2-space indentation
  - 100 char line width (matches most editors)
- [ ] `npm run format` script for formatting all files
- [ ] `npm run format:check` script for CI (exits non-zero if unformatted)
- [ ] ESLint configured to not conflict with Prettier (`eslint-config-prettier`)
- [ ] All existing UI code is formatted

### Notes

- Prettier and ESLint can conflict on formatting rules — use `eslint-config-prettier` to disable ESLint formatting rules

---

## US-3: xtask lint --ui Command

**As a** developer  
**I want** a `cargo xtask lint --ui` command  
**So that** I can run UI linting from the workspace root without switching to the `ui/` directory

### Acceptance Criteria

- [ ] `cargo xtask lint --ui` runs ESLint on the UI codebase
- [ ] Exits with non-zero code if linting fails
- [ ] Output is colored and readable
- [ ] Includes Prettier format check
- [ ] Works from any directory in the workspace

### Notes

- Should run `npm run lint` and `npm run format:check` in the `ui/` directory
- Consider running `npm ci` first if `node_modules` doesn't exist (or error with helpful message)

---

## US-4: xtask lint --engine Command

**As a** developer  
**I want** a `cargo xtask lint --engine` command  
**So that** I can run Rust linting (clippy + fmt) with a single command

### Acceptance Criteria

- [ ] `cargo xtask lint --engine` runs:
  - `cargo fmt --check` (format check)
  - `cargo clippy --workspace -- -D warnings` (strict clippy)
- [ ] Exits with non-zero code if any check fails
- [ ] Output clearly shows which check failed
- [ ] Works from any directory in the workspace

### Notes

- This formalizes what the QA agent already does manually
- Consider `--fix` flag to auto-fix what's possible (`cargo fmt` + `cargo clippy --fix`)

---

## US-5: xtask lint Command (All)

**As a** developer  
**I want** a `cargo xtask lint` command (no flags)  
**So that** I can run all linting checks at once

### Acceptance Criteria

- [ ] `cargo xtask lint` runs both `--ui` and `--engine` checks
- [ ] Runs both even if one fails (reports all issues)
- [ ] Summary at end shows pass/fail status for each
- [ ] Exits with non-zero code if any check fails

### Notes

- Useful for pre-commit checks and CI

---

## US-6: QA Agent Includes Linting

**As a** project maintainer  
**I want** the QA agent to include linting output in its checks  
**So that** QA reports cover both code quality (lint) and architecture

### Acceptance Criteria

- [ ] QA agent runs `cargo xtask lint` as part of automated checks
- [ ] Linting results included in QA report under "Automated Checks"
- [ ] Both UI and Engine linting status documented
- [ ] ESLint errors/warnings counted and reported

### Notes

- Update `.github/agents/QA.agent.md` to include lint commands
- QA report template should have a section for linting results

---

## US-7: CI Fails on Linting Issues

**As a** project maintainer  
**I want** CI to fail if linting issues are found  
**So that** code quality is enforced before merge

### Acceptance Criteria

- [ ] GitHub Actions workflow includes `cargo xtask lint`
- [ ] PR checks fail if linting fails
- [ ] Clear error messages in CI logs
- [ ] Linting runs early in pipeline (fast feedback)

### Notes

- Depends on CI/CD pipeline (Milestone 5 task)
- This user story defines the requirement; CI implementation is separate

---

## Priority Order

1. **US-1** (ESLint) — Core value, enables everything else
2. **US-2** (Prettier) — Completes UI tooling
3. **US-4** (xtask lint --engine) — Formalizes existing practice
4. **US-3** (xtask lint --ui) — Integrates UI linting
5. **US-5** (xtask lint all) — Convenience command
6. **US-6** (QA Agent) — Process integration
7. **US-7** (CI) — Enforcement (depends on CI pipeline)

---

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Documentation updated (README or guide)
- [ ] Existing code passes linting (or issues tracked)
- [ ] QA agent updated
- [ ] Tested manually
