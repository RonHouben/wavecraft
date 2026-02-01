# Coding Standards

This document defines the coding standards and conventions for the VstKit project.

---

## TypeScript / JavaScript

### Class-Based Architecture

**Rule:** Use class syntax for all non-React TypeScript code.

React components should use functional components with hooks, but all other TypeScript code (services, clients, utilities, state management) should use classes.

**Rationale:**
- Classes provide clear encapsulation of state and behavior
- Better IDE support (autocomplete, refactoring)
- Explicit instantiation and lifecycle management
- Easier to mock in tests
- Clear separation between static and instance members

**Do:**
```typescript
// ✅ Service classes
class IpcBridge {
  private requestId = 0;
  private pendingRequests = new Map<number, PendingRequest>();

  async invoke<T>(method: string, params?: unknown): Promise<T> {
    // ...
  }

  on<T>(event: string, callback: (data: T) => void): () => void {
    // ...
  }
}

// ✅ Client classes
class ParameterClient {
  constructor(private bridge: IpcBridge) {}

  async getParameter(id: string): Promise<ParameterInfo> {
    return this.bridge.invoke('getParameter', { id });
  }
}

// ✅ Singleton pattern when appropriate
class IpcClient {
  private static instance: IpcClient | null = null;

  static getInstance(): IpcClient {
    if (!IpcClient.instance) {
      IpcClient.instance = new IpcClient();
    }
    return IpcClient.instance;
  }

  private constructor() {
    // Private constructor for singleton
  }
}
```

**Don't:**
```typescript
// ❌ Exported functions with module-level state
let requestId = 0;
const pendingRequests = new Map();

export async function invoke<T>(method: string, params?: unknown): Promise<T> {
  // ...
}

export function on(event: string, callback: Function): () => void {
  // ...
}
```

### React Components

**Rule:** Use functional components with hooks for all React code.

```typescript
// ✅ Functional component with hooks
export function ParameterSlider({ id, name, min, max }: Props) {
  const { param, setValue } = useParameter(id);
  
  return (
    <input
      type="range"
      value={param?.value ?? 0}
      onChange={(e) => setValue(parseFloat(e.target.value))}
    />
  );
}

// ❌ Class components (avoid)
class ParameterSlider extends React.Component<Props> {
  // ...
}
```

### Custom Hooks

Custom React hooks bridge between class-based services and React components:

```typescript
// Hook wraps class-based client for React integration
export function useParameter(id: string) {
  const [param, setParam] = useState<ParameterInfo | null>(null);
  const client = ParameterClient.getInstance();

  useEffect(() => {
    client.getParameter(id).then(setParam);
  }, [id]);

  const setValue = useCallback(async (value: number) => {
    await client.setParameter(id, value);
  }, [id]);

  return { param, setValue };
}
```

### Environment-Aware Hooks

**Rule:** Hooks that depend on runtime environment must detect environment at module scope.

When hooks need different behavior based on runtime environment (e.g., browser vs WKWebView), the environment check must be evaluated once at module load time, not inside the hook body.

**Rationale:**
- React's Rules of Hooks require consistent hook call order across renders
- Conditional hook behavior inside the hook body can violate this rule
- Module-scope evaluation ensures the condition is stable

**Do:**
```typescript
// ✅ Environment detection at module scope
import { isBrowserEnvironment } from './environment';

// Evaluated once when module loads
const IS_BROWSER = isBrowserEnvironment();

export function useParameter(id: string) {
  // IS_BROWSER is stable - same value for all renders
  const [param, setParam] = useState(IS_BROWSER ? mockData : null);
  
  useEffect(() => {
    if (IS_BROWSER) return; // Safe: IS_BROWSER never changes
    // ... fetch real data
  }, [id]);
}
```

**Don't:**
```typescript
// ❌ Environment detection inside hook
export function useParameter(id: string) {
  // BAD: Called on every render, could theoretically change
  if (isBrowserEnvironment()) {
    return { param: mockData };  // Violates Rules of Hooks!
  }
  // ... rest of hook
}
```

### Build-Time Constants

**Rule:** Use Vite's `define` block for compile-time constants that come from the build system.

Values that need to be injected at build time (e.g., version from Cargo.toml) should use Vite's `define` configuration rather than environment variables.

**Configuration (`vite.config.ts`):**
```typescript
export default defineConfig({
  define: {
    // Compile-time replacement, falls back to 'dev' for standalone npm run dev
    '__APP_VERSION__': JSON.stringify(process.env.VITE_APP_VERSION || 'dev'),
  },
});
```

**Type Declaration (`vite-env.d.ts`):**
```typescript
declare const __APP_VERSION__: string;
```

**Usage:**
```typescript
// ✅ Use the constant directly - it's replaced at build time
<span>v{__APP_VERSION__}</span>
```

**Why `define` over `.env` files:**
- Build system (xtask) can inject values via environment variables
- `define` creates compile-time replacement (zero runtime cost)
- No risk of `.env` files drifting from source of truth
- Clear TypeScript typing via declaration file

### Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Classes | PascalCase | `IpcBridge`, `ParameterClient` |
| Interfaces | PascalCase | `ParameterInfo`, `IpcError` |
| Type aliases | PascalCase | `EventCallback`, `RequestId` |
| Methods | camelCase | `getParameter`, `setReceiveCallback` |
| Private members | camelCase (no underscore prefix) | `private requestId` |
| Constants | UPPER_SNAKE_CASE | `DEFAULT_TIMEOUT_MS` |
| React components | PascalCase | `ParameterSlider` |
| React hooks | camelCase with `use` prefix | `useParameter` |

### File Organization

```
src/
├── lib/
│   └── vstkit-ipc/
│       ├── index.ts          # Public exports
│       ├── types.ts          # Type definitions
│       ├── environment.ts    # Environment detection (browser vs WKWebView)
│       ├── IpcBridge.ts      # Class: low-level bridge
│       ├── ParameterClient.ts # Class: high-level API
│       └── hooks.ts          # React hooks (functional)
├── components/
│   ├── ParameterSlider.tsx   # Functional component
│   ├── ParameterSlider.test.tsx # Co-located test
│   ├── VersionBadge.tsx      # Version display component
│   └── LatencyMonitor.tsx    # Functional component
├── test/
│   ├── setup.ts              # Global test setup
│   └── mocks/
│       └── ipc.ts            # IPC mock module
└── App.tsx
```

**Note:** Class files should be named with PascalCase matching the class name.

### Import Aliases

**Rule:** Use configured path aliases instead of relative imports for shared libraries.

The project defines the following import aliases (configured in `tsconfig.json`, `vite.config.ts`, and `vitest.config.ts`):

| Alias | Path | Usage |
|-------|------|-------|
| `@vstkit/ipc` | `./src/lib/vstkit-ipc` | IPC client, hooks, and types |
| `@vstkit/ipc/meters` | `./src/lib/vstkit-ipc/meters` | Pure audio math utilities (no IPC side effects) |

**Do:**
```typescript
// ✅ Use alias for IPC features (in components)
import { getMeterFrame, MeterFrame, useParameter } from '@vstkit/ipc';

// ✅ Use subpath alias for pure utilities (especially in tests)
import { linearToDb, dbToLinear } from '@vstkit/ipc/meters';
```

**Don't:**
```typescript
// ❌ Relative imports to shared libraries
import { getMeterFrame } from '../lib/vstkit-ipc';
import { useParameter } from '../../lib/vstkit-ipc';

// ❌ Relative imports in test files
import { linearToDb } from './vstkit-ipc/meters';
```

**Subpath Aliases:**

The `@vstkit/ipc/meters` subpath provides access to pure utility functions (`linearToDb`, `dbToLinear`, `getMeterFrame`) without triggering IPC initialization side effects. Use this subpath:
- In unit tests for pure functions
- When you only need math utilities, not hooks or clients

**Rationale:**
- Cleaner imports that don't change when files move
- Immediately identifies imports as project-internal
- Consistent import paths across the codebase
- Test files follow the same conventions as production code
- Subpath aliases avoid initialization side effects in tests

### Global Object Access

**Rule:** Use `globalThis` instead of `window` for accessing the global object.

This applies to:
- TypeScript/JavaScript source files
- JavaScript strings embedded in Rust code (e.g., `evaluate_script` calls)

**Do:**
```typescript
// ✅ Use globalThis in TypeScript/JavaScript
globalThis.vstkit?.invoke('getParameter', { id });
globalThis.addEventListener('message', handler);
```

```rust
// ✅ Use globalThis in embedded JavaScript (Rust)
let js = format!(
    "if (globalThis.__VSTKIT_IPC__ && globalThis.__VSTKIT_IPC__._onParamUpdate) {{ \
        globalThis.__VSTKIT_IPC__._onParamUpdate({{ id: '{}', value: {} }}); \
    }}",
    id, value
);
webview.evaluate_script(&js);
```

**Don't:**
```typescript
// ❌ Using window in TypeScript/JavaScript
window.vstkit?.invoke('getParameter', { id });
window.addEventListener('message', handler);
```

```rust
// ❌ Using window in embedded JavaScript (Rust)
let js = format!(
    "if (window.__VSTKIT_IPC__) {{ window.__VSTKIT_IPC__._onParamUpdate(...); }}"
);
```

**Rationale:**
- `globalThis` is the standardized way to access the global object (ES2020+)
- Works consistently across all JavaScript environments (browser, Node.js, Web Workers, etc.)
- Plugin UI may run in different contexts where `window` is not available
- Future-proofs code against environment changes

---

## CSS / Styling (TailwindCSS)

### Utility-First Styling

**Rule:** Use TailwindCSS utility classes for all styling. Do not create separate CSS files for components.

The project uses TailwindCSS with a custom theme defined in `ui/tailwind.config.js`.

**Do:**
```tsx
// ✅ Utility classes in JSX
<div className="flex flex-col gap-2 rounded-lg border border-plugin-border bg-plugin-surface p-4">
  <h3 className="text-base font-semibold text-gray-200">Section Title</h3>
  <p className="text-sm text-gray-400">Description text</p>
</div>
```

**Don't:**
```tsx
// ❌ Separate CSS files
import './MyComponent.css';

<div className="my-component">
  <h3 className="my-component__title">Section Title</h3>
</div>
```

### Theme Tokens

**Rule:** Use semantic theme tokens instead of hardcoded colors.

| Token | Usage | Value |
|-------|-------|-------|
| `bg-plugin-dark` | Main background | `#1a1a1a` |
| `bg-plugin-surface` | Card/section backgrounds | `#2a2a2a` |
| `border-plugin-border` | Borders | `#444444` |
| `text-accent` | Primary accent color | `#4a9eff` |
| `text-accent-light` | Accent hover state | `#6bb0ff` |
| `bg-meter-safe` | Meter safe zone | `#4caf50` |
| `bg-meter-warning` | Meter warning zone | `#ffeb3b` |
| `bg-meter-clip` | Meter clipping | `#ff1744` |

**Do:**
```tsx
// ✅ Semantic theme tokens
<div className="bg-plugin-surface border-plugin-border text-accent">
```

**Don't:**
```tsx
// ❌ Hardcoded colors (except where no token exists)
<div className="bg-[#2a2a2a] border-[#444444] text-[#4a9eff]">
```

### Custom CSS (Exceptions)

**Rule:** Only use `@layer` directives for CSS that cannot be expressed as utility classes.

Valid exceptions:
- Vendor-prefixed pseudo-elements (`::-webkit-slider-thumb`)
- Complex animations requiring `@keyframes`
- Browser-specific hacks

**Example (slider thumb styling):**
```css
@layer components {
  .slider-thumb::-webkit-slider-thumb {
    @apply h-[18px] w-[18px] cursor-pointer rounded-full bg-accent;
  }
}
```

### Class Organization

**Rule:** Order Tailwind classes logically: layout → spacing → colors → typography → effects.

**Do:**
```tsx
// ✅ Logical grouping
<div className="flex flex-col gap-2 p-4 bg-plugin-surface text-gray-200 rounded-lg">
```

**Don't:**
```tsx
// ❌ Random ordering
<div className="text-gray-200 rounded-lg flex p-4 bg-plugin-surface flex-col gap-2">
```

**Note:** The Prettier Tailwind plugin automatically sorts classes. Run `npm run format` to apply.

### WebView Background Color

**Rule:** Both `body` and `#root` must have explicit background colors matching the theme.

The WebView shows a default white background when the user scrolls beyond the content boundaries (over-scroll/rubber-band scrolling). To prevent this visual inconsistency:

1. Apply `bg-plugin-dark` to both `body` and `#root` in `index.css`
2. Add an inline `style="background-color: #1a1a1a;"` to the `<html>` element in `index.html` as a pre-CSS fallback

**Why both?**
- `body` background covers the document area
- `#root` background covers the React app container
- `<html>` inline style prevents white flash before CSS loads

**Example (`index.css`):**
```css
@layer base {
  body {
    @apply m-0 bg-plugin-dark p-0;
  }

  #root {
    @apply h-screen w-full overflow-y-auto bg-plugin-dark;
  }
}
```

**Example (`index.html`):**
```html
<html lang="en" style="background-color: #1a1a1a;">
```

### File Structure

```
ui/
├── tailwind.config.js    # Theme configuration (colors, fonts, animations)
├── postcss.config.js     # PostCSS plugins
└── src/
    ├── index.css         # Tailwind directives + @layer overrides only
    └── components/
        └── *.tsx         # All styling via className utilities
```

No `.css` files should exist in `src/components/`.

---

## Rust

### Module Organization

Follow the existing crate structure:
- `protocol` — Shared contracts and types
- `dsp` — Pure DSP code (no framework dependencies)
- `plugin` — nih-plug integration
- `bridge` — IPC handling
- `desktop` — Standalone desktop app for testing
- `metering` — SPSC ring buffer for audio → UI metering

### xtask Commands

The `xtask` crate provides build system commands. Each command is a module under `commands/`:

```
engine/xtask/src/
├── lib.rs           # Shared utilities (paths, platform, output)
├── main.rs          # CLI definition (clap)
└── commands/
    ├── mod.rs       # Command exports and run_all()
    ├── bundle.rs    # Build VST3/CLAP bundles
    ├── lint.rs      # Unified linting (UI + Engine)
    ├── sign.rs      # macOS code signing
    ├── notarize.rs  # Apple notarization
    ├── release.rs   # Complete release workflow
    └── ...
```

**Command conventions:**
- Each command module exposes a `run()` function as entry point
- Use `anyhow::Result` for error propagation
- Use `xtask::output::*` helpers for colored terminal output
- Platform checks: `if Platform::current() != Platform::MacOS { bail!(...) }`
- Configuration from environment: `Config::from_env()` pattern
- Unit tests in `#[cfg(test)] mod tests { }` at bottom of file

**Adding a new command:**
1. Create `commands/mycommand.rs` with `pub fn run(...) -> Result<()>`
2. Register in `commands/mod.rs`: `pub mod mycommand;`
3. Add CLI variant in `main.rs`: `enum Commands { MyCommand { ... } }`
4. Wire up in `main()` match: `Some(Commands::MyCommand { .. }) => commands::mycommand::run(...)`

### Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Structs | PascalCase | `IpcHandler`, `AppState` |
| Traits | PascalCase | `ParameterHost` |
| Functions | snake_case | `handle_request`, `get_parameter` |
| Methods | snake_case | `fn set_sample_rate(&mut self)` |
| Constants | UPPER_SNAKE_CASE | `const WINDOW_WIDTH: u32` |
| Modules | snake_case | `mod params`, `mod handler` |

### Platform-Specific Code

**Rule:** Use `#[cfg(target_os = "...")]` attributes for platform-specific code. Do not use `#[allow(dead_code)]` to suppress warnings for platform-gated items.

VstKit is primarily developed for macOS, with the editor/WebView components being platform-specific. Code that only runs on certain platforms should be properly gated.

**Patterns:**

```rust
// ✅ Platform-gate the entire item (imports, functions, statics)
#[cfg(any(target_os = "macos", target_os = "windows"))]
use include_dir::{Dir, include_dir};

#[cfg(any(target_os = "macos", target_os = "windows"))]
static UI_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../../ui/dist");

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn get_asset(path: &str) -> Option<(&'static [u8], &'static str)> {
    // ...
}

// ✅ Platform-gate tests that use platform-specific functions
#[cfg(test)]
mod tests {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    use super::*;

    #[test]
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn test_platform_specific_function() {
        // Test only runs on macOS/Windows
    }
}

// ✅ Use #[allow(dead_code)] ONLY for trait methods called by platform implementations
// (Rust's analysis can't see calls from platform-specific code)
pub trait WebViewHandle: Any + Send {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn evaluate_script(&self, script: &str) -> Result<(), String>;

    /// Note: Called by platform implementations, not from trait consumers.
    #[allow(dead_code)]
    fn resize(&self, width: u32, height: u32);
}
```

**Don't:**
```rust
// ❌ Using #[allow(dead_code)] instead of proper platform-gating
#[allow(dead_code)]
pub fn get_asset(path: &str) -> Option<...> {
    // This compiles everywhere but is only used on macOS
}

// ❌ Using `test` in cfg to make code compile for tests on all platforms
#[cfg(any(target_os = "macos", target_os = "windows", test))]
static UI_ASSETS: Dir = ...;  // Compiles on Linux CI but isn't used
```

**Rationale:**
- Platform-gated code should only compile on platforms where it's used
- This catches real dead code (lint checks work correctly)
- Linux CI doesn't need to compile macOS/Windows GUI code
- `#[allow(dead_code)]` should be reserved for legitimate false positives (e.g., trait methods called by platform impls)

### Real-Time Safety

Code running on the audio thread must:
- Never allocate (`Box::new`, `Vec::push`, `String::from`)
- Never lock (`Mutex`, `RwLock`)
- Never make system calls that can block
- Use atomic types for shared state
- Use SPSC ring buffers for data transfer

---

## Testing

VstKit uses Vitest and React Testing Library for UI unit testing.

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

Tests are co-located with source files:

```
ui/src/
├── components/
│   ├── Meter.tsx
│   ├── Meter.test.tsx           # Component test
│   ├── ParameterSlider.tsx
│   └── ParameterSlider.test.tsx # Component test
├── lib/
│   ├── audio-math.ts
│   └── audio-math.test.ts       # Pure function tests
└── test/
    ├── setup.ts                 # Global test setup
    └── mocks/
        └── ipc.ts               # IPC mock module
```

### Mocking IPC for Tests

The `ui/src/test/mocks/ipc.ts` module provides mock implementations of IPC hooks that allow testing components without the Rust engine.

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

| Function | Purpose |
|----------|----------|
| `setMockParameter(id, info)` | Set parameter state for a test |
| `setMockMeterFrame(frame)` | Set meter data for a test |
| `getMockParameter(id)` | Get current mock parameter value |
| `resetMocks()` | Clear all mock state (call in `beforeEach`) |

### Test Configuration

**Vitest Configuration** (`ui/vitest.config.ts`):
- Environment: `happy-dom` (faster than jsdom)
- Globals: enabled (`describe`, `it`, `expect` without imports)
- Setup: `src/test/setup.ts` runs before each test file

**TypeScript Support:**
- Types: `vitest/globals`, `@testing-library/jest-dom`
- Aliases: Same as production (`@vstkit/ipc`, `@vstkit/ipc/meters`)

---

## Linting & Formatting

VstKit enforces consistent code quality through automated linting.

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

---

## General

### Versioning

**Rule:** The Product Owner decides the target version in the user stories. The Coder implements the version bump during the coding phase.

The version is defined in `engine/Cargo.toml` under `[workspace.package]` and is the **single source of truth**. It gets injected into the UI at build time via Vite's `define` configuration.

**Workflow:**
1. **PO** specifies the target version in `user-stories.md` with rationale
2. **Coder** implements the version bump in `engine/Cargo.toml` during coding
3. **Tester** verifies the correct version is displayed in the UI

**Version bump criteria:**
- **Minor version** (0.X.0): Significant features, architectural changes, milestone completions
- **Patch version** (0.0.X): Small features, bug fixes, polish items, documentation updates

**Examples:**
- Adding WebSocket IPC bridge → Minor bump (0.2.0 → 0.3.0)
- Removing a feature flag → Patch bump (0.2.0 → 0.2.1)
- Performance optimization → Patch bump
- New UI component → Patch bump (unless it's a major feature)

**Why PO decides:**
- Version communicates product significance to users — a product decision
- Ensures consistent versioning across features
- Separates "what version" (product) from "when to bump" (engineering)

**Why Coder implements during coding:**
- Allows testers to verify the correct version is rendered in the UI
- Creates clear traceability between builds and features
- Ensures version is updated before testing, not as an afterthought

**Example:**
```toml
# engine/Cargo.toml
[workspace.package]
version = "0.2.0"  # Bump this when implementing a new feature
```

The VersionBadge component in the UI displays this version (e.g., "v0.2.0").

---

### Comments and Documentation

- Use `///` doc comments for public APIs
- Include examples in doc comments where helpful
- Keep comments up-to-date with code changes

### Documentation References

**Rule:** Always link to relevant documentation in the `docs/` folder.

All project documentation (README, specs, design docs) must include links to related architecture documents. This ensures discoverability and keeps documentation interconnected.

**Required links:**
- [High-Level Design](./high-level-design.md) — Architecture overview, component design, data flows
- [Coding Standards](./coding-standards.md) — Code conventions and patterns
- [Roadmap](../roadmap.md) — Milestone tracking and progress

**Do:**
```markdown
## Documentation

- [High-Level Design](docs/architecture/high-level-design.md) — Architecture overview
- [Coding Standards](docs/architecture/coding-standards.md) — Code conventions
- [Roadmap](docs/roadmap.md) — Implementation progress
```

**Don't:**
```markdown
## Documentation

See the docs folder for more information.
```

### Error Handling

- TypeScript: Use explicit error types, avoid `any`
- Rust: Use `Result<T, E>` with descriptive error types
- Always handle errors explicitly; avoid silent failures
