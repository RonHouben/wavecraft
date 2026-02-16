# Coding Standards — Testing, Linting & Quality

## Related Documents

- [Coding Standards Overview](./coding-standards.md) — Quick reference and navigation hub
- [Development Workflows](./development-workflows.md) — Build commands, CI/CD pipelines
- [Agent Development Flow](./agent-development-flow.md) — Testing workflow for agents

---

## Testing

Wavecraft uses Vitest and React Testing Library for UI unit testing.

### Documentation Examples (Rust doctests)

**Rule:** Prefer compiling doctests over ignored ones.

Use the following conventions for Rust doc examples:

- **`rust,no_run`** for examples that should compile but don't need to execute.
- **`text`** for cross-crate or illustrative snippets that cannot compile in the current crate.
- **Avoid `ignore`** unless there's a hard external dependency that can't be represented.

**Do:**

````rust
/// ```rust,no_run
/// use wavecraft_core::prelude::*;
/// ```
````

**Do (non-compiling illustration):**

````text
/// ```text
/// use wavecraft::prelude::*; // via Cargo rename in downstream crate
/// ```
````

### Pre-Push Validation

**Rule:** Always run `cargo xtask ci-check` and `cargo xtask sync-ui-versions --check` before pushing changes.

This command simulates CI checks locally and runs ~26x faster than Docker-based CI:

```bash
# Run standard checks (docs, UI build, lint+typecheck, tests — ~1 minute)
cargo xtask ci-check

# Verify UI workspace dependency/version synchronization
cargo xtask sync-ui-versions --check

# Run with auto-fix for linting issues
cargo xtask ci-check --fix

# Full validation (adds template validation + CD dry-run)
cargo xtask ci-check --full   # or -F

# Skip individual phases
cargo xtask ci-check --skip-docs       # Skip doc link checking
cargo xtask ci-check --skip-lint       # Skip linting + type-checking
cargo xtask ci-check --skip-tests      # Skip automated tests
cargo xtask ci-check -F --skip-template  # Full minus template validation
cargo xtask ci-check -F --skip-cd        # Full minus CD dry-run
```

**What it runs (6 phases):** 0. **Documentation** — Link validation via `scripts/check-links.sh` (skippable: `--skip-docs`)

1. **UI Dist Build** — Rebuilds `ui/dist` to mirror CI (always runs; two-stage: `build:lib` in `ui/`, then full build in `sdk-template/ui/`, dist copied to `ui/dist/`)
2. **Linting + Type-Checking** — ESLint, Prettier, `tsc --noEmit`, cargo fmt, clippy (skippable: `--skip-lint`; fixable: `--fix`)
3. **Automated Tests** — Engine (Rust) + UI (Vitest) tests (skippable: `--skip-tests`)
4. **Template Validation** — Runs `validate-template` to check CLI-generated projects compile (`--full` only; skippable: `--skip-template`)
5. **CD Dry-Run** — Git-based change detection matching CD workflow path filters (`--full` only; skippable: `--skip-cd`)

**Visual Testing:** For UI validation, use `cargo xtask dev` to start dev servers,
then invoke the "playwright-mcp-ui-testing" skill for browser-based testing.

### Running Tests

```bash
# Run all tests (Engine + UI)
cargo xtask test

# Run only UI tests
cargo xtask test --ui

# Run only Engine tests
cargo xtask test --engine

# Run UI tests in watch mode (from ui/ directory)
npm run test:watch

# Run UI tests with coverage
npm run test:coverage
```

### Test File Organization

Tests are co-located with source files in each package:

```
ui/
├── packages/
│   ├── core/src/
│   │   ├── IpcBridge.test.ts      # IPC bridge tests
│   │   ├── environment.test.ts    # Environment detection tests
│   │   └── logger/
│   │       └── Logger.test.ts     # Logger tests
│   └── components/src/
│       ├── Meter.tsx
│       ├── Meter.test.tsx         # Component test
│       ├── ParameterSlider.tsx
│       └── ParameterSlider.test.tsx
├── src/                           # (empty — app code lives in sdk-template/ui/)
└── test/
    ├── setup.ts                   # Global test setup
    └── mocks/
        └── ipc.ts                 # IPC mock module
```

### Mocking IPC for Tests

The `ui/test/mocks/ipc.ts` module provides mock implementations of IPC hooks that allow testing components without the Rust engine.

**Do:**

```typescript
// ✅ Use mock utilities to set up test state
import { setMockParameter, resetMocks } from '../test/mocks/ipc';
import { useParameter } from '../test/mocks/ipc'; // Use mock hook

beforeEach(() => {
  resetMocks();
  setMockParameter('volume', { value: 0.5, name: 'Volume' });
});
```

**Mock API:**

| Function                     | Purpose                                     |
| ---------------------------- | ------------------------------------------- |
| `setMockParameter(id, info)` | Set parameter state for a test              |
| `setMockMeterFrame(frame)`   | Set meter data for a test                   |
| `getMockParameter(id)`       | Get current mock parameter value            |
| `resetMocks()`               | Clear all mock state (call in `beforeEach`) |

### Test Configuration

**Vitest Configuration** (`ui/vitest.config.ts`):

- Environment: `happy-dom` (faster than jsdom)
- Globals: enabled (`describe`, `it`, `expect` without imports)
- Setup: `test/setup.ts` runs before each test file

**TypeScript Support:**

- Types: `vitest/globals`, `@testing-library/jest-dom`
- npm packages: `@wavecraft/core`, `@wavecraft/components` available via workspace

### Testing CLI-Generated Plugins

When SDK changes affect generated plugins (templates, CLI, or engine crates), validate that `wavecraft create` produces working projects.

**Standard workflow using `--output` flag:**

```bash
# Generate test plugin into SDK's build directory (gitignored)
# Note: --local-sdk is NOT needed when running via `cargo run` — the CLI
# auto-detects SDK development mode and uses path dependencies automatically.
cargo run --manifest-path cli/Cargo.toml -- create TestPlugin \
  --output target/tmp/test-plugin

# Test the generated plugin
cd target/tmp/test-plugin
cargo run --manifest-path /path/to/wavecraft/cli/Cargo.toml -- start --install
```

**Why use `--output`:**

- Test artifacts live in `target/tmp/` (gitignored)
- Easy cleanup: `rm -rf target/tmp/test-plugin`
- Auto-detection uses path dependencies, so local changes are tested without publishing
- `--local-sdk` is still available as a manual override but is no longer needed

**Test checklist for CLI/template changes:**

1. `wavecraft create` completes without errors
2. `wavecraft start` builds without compile errors
3. **`cargo clippy` passes on generated project** — catch unused imports, dead code warnings
4. `cargo xtask bundle` produces valid plugin bundles
5. Plugin loads in a DAW

---

## Linting & Formatting

Wavecraft enforces consistent code quality through automated linting.

### Running Linters

```bash
# Run all linters (UI + Engine)
cargo xtask lint

# Run with auto-fix
cargo xtask lint --fix

# Run only UI linting (ESLint + Prettier)
cargo xtask lint --ui

# Run only Engine linting (cargo fmt + clippy)
cargo xtask lint --engine
```

### UI Linting (TypeScript/React)

**Tools:**

- **ESLint 9** — Static analysis with strict TypeScript and React rules
- **Prettier** — Code formatting

**Key Rules:**

- `@typescript-eslint/no-explicit-any: error` — No `any` types
- `@typescript-eslint/explicit-function-return-type: warn` — Explicit return types preferred
- `react-hooks/rules-of-hooks: error` — Enforce hooks rules
- `react-hooks/exhaustive-deps: error` — Complete dependency arrays

**Configuration:**

- ESLint: `ui/eslint.config.js` (flat config format)
- Prettier: `ui/.prettierrc`

**NPM Scripts:**

```bash
npm run lint        # ESLint check
npm run lint:fix    # ESLint auto-fix
npm run format      # Prettier format
npm run format:check # Prettier check
```

### Engine Linting (Rust)

**Tools:**

- **cargo fmt** — Code formatting
- **cargo clippy** — Lint analysis with `-D warnings` (warnings as errors)

**Rules:**

- All Clippy warnings are treated as errors
- Standard Rust formatting via rustfmt

**Manual Commands:**

```bash
cargo fmt --check   # Check formatting
cargo fmt           # Auto-format
cargo clippy --workspace -- -D warnings
```

### CI Integration

Linting runs automatically on all PRs via `.github/workflows/lint.yml`:

- Engine linting runs on `macos-latest`
- UI linting runs on `ubuntu-latest`
- CI validation includes `cargo xtask sync-ui-versions --check` to enforce UI workspace dependency/version synchronization

---

## Logging

**Rule:** Use structured logging instead of direct console calls or println!.

### UI Logging (TypeScript)

Use the `Logger` class from `@wavecraft/core` for all UI logging. Direct `console.*` calls are prohibited in production code.

```typescript
// ✅ Import and use the logger
import { logger } from '@wavecraft/core';

logger.debug('Verbose tracing info', { requestId: 123 });
logger.info('Connection established', { transport: 'WebSocket' });
logger.warn('Reconnecting...', { attempt: 3 });
logger.error('Request failed', { method: 'setParameter', error });

// ❌ Direct console calls
console.log('Connected');
console.error('Failed:', error);
```

### Log Levels

| Level   | Usage                                     | Production |
| ------- | ----------------------------------------- | ---------- |
| `DEBUG` | Verbose tracing, request/response details | Hidden     |
| `INFO`  | Significant events (connection, init)     | Visible    |
| `WARN`  | Recoverable issues, degraded operation    | Visible    |
| `ERROR` | Failures requiring attention              | Visible    |

### Structured Context

Always pass structured context objects instead of string interpolation:

```typescript
// ✅ Structured context
logger.error('Parameter update failed', {
  parameterId: 'gain',
  value: 0.5,
  error
});

// ❌ String interpolation
logger.error(`Parameter ${id} update failed: ${error.message}`);
```

### Engine Logging (Rust)

Use the `tracing` crate for all engine logging. Direct `println!`/`eprintln!` are prohibited except in CLI output (xtask commands).

```rust
// ✅ Use tracing macros
use tracing::{debug, info, warn, error};

info!("WebSocket server started on port {}", port);
debug!(client_id = %id, "Client connected");
warn!(reconnect_attempt = attempts, "Connection lost, reconnecting...");
error!(?err, "Failed to parse message");

// ❌ Direct println (except in xtask CLI commands)
println!("Server started on port {}", port);
eprintln!("Error: {}", err);
```

### Exceptions (println! allowed)

- `xtask` CLI commands — these are intentional user-facing output
- Benchmark/test output — `println!` is acceptable for test diagnostics

**Rationale:**

- Structured logging enables filtering, searching, and analysis
- Consistent log format across UI and Engine
- Production builds can adjust log levels without code changes
- Context objects preserve machine-parseable data

---

## Error Handling

- TypeScript: Use explicit error types, avoid `any`
- Rust: Use `Result<T, E>` with descriptive error types
- Always handle errors explicitly; avoid silent failures
