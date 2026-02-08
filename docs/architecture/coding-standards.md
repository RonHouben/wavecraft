# Coding Standards

This document defines the coding standards and conventions for the Wavecraft project.

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
    // Compile-time replacement, falls back to 'dev' for local npm run dev
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

The UI codebase is organized as an npm workspace with publishable packages:

```
ui/
├── packages/                      # Published npm packages
│   ├── core/                      # @wavecraft/core
│   │   ├── src/
│   │   │   ├── index.ts           # Main entry (re-exports all public API)
│   │   │   ├── meters.ts          # /meters subpath (pure audio math)
│   │   │   ├── hooks/             # React hooks (domain folder)
│   │   │   │   ├── useParameter.ts
│   │   │   │   ├── useAllParameters.ts
│   │   │   │   ├── useParameterGroups.ts
│   │   │   │   ├── useConnectionStatus.ts
│   │   │   │   ├── useLatencyMonitor.ts
│   │   │   │   ├── useMeterFrame.ts
│   │   │   │   ├── useRequestResize.ts
│   │   │   │   └── useWindowResizeSync.ts
│   │   │   ├── ipc/               # IPC classes (domain folder)
│   │   │   │   ├── IpcBridge.ts   # Low-level IPC bridge
│   │   │   │   └── ParameterClient.ts # High-level parameter API
│   │   │   ├── types/             # TypeScript types (domain folder)
│   │   │   │   ├── ipc.ts         # IPC protocol types
│   │   │   │   ├── parameters.ts  # Parameter types
│   │   │   │   └── metering.ts    # Meter types
│   │   │   ├── utils/             # Utilities (domain folder)
│   │   │   │   ├── environment.ts # Runtime detection
│   │   │   │   └── audio-math.ts  # linearToDb, dbToLinear
│   │   │   ├── transports/        # Transport implementations
│   │   │   └── logger/            # Structured logging
│   │   ├── dist/                  # Built ESM bundle + DTS
│   │   └── package.json           # npm package config
│   └── components/                # @wavecraft/components
│       ├── src/
│       │   ├── index.ts           # Component exports
│       │   ├── Meter.tsx          # Audio level meter
│       │   ├── ParameterSlider.tsx# Slider control
│       │   ├── ParameterGroup.tsx # Grouped parameters
│       │   └── VersionBadge.tsx   # Version display
│       ├── dist/                  # Built ESM bundle + DTS
│       └── package.json           # npm package config
├── src/                           # Development app (internal)
│   ├── App.tsx                    # Dev app entry
│   └── main.tsx                   # Dev app bootstrap
├── test/
│   ├── setup.ts                   # Global test setup
│   └── mocks/
│       └── ipc.ts                 # IPC mock module
└── package.json                   # Workspace root (workspaces: ["packages/*"])
```

**Domain folder conventions:**
- **No internal barrel files** — Domain folders do not have `index.ts` barrels
- The main `src/index.ts` imports directly from each file (e.g., `./hooks/useParameter`)
- One file per hook/class/type domain
- External imports use `@wavecraft/core` or `@wavecraft/components`

### Barrel Files

**Rule:** Use barrel files only for published package entry points, not for internal folders.

**Rationale:**
- Internal barrels can defeat tree-shaking in application code
- They can mask circular dependencies
- They slow down TypeScript IDE performance
- Published packages are pre-bundled, so the main entry barrel is acceptable

**Do:**
```typescript
// ✅ Main entry point (src/index.ts) - this IS the public API
export { useParameter } from './hooks/useParameter';
export { IpcBridge } from './ipc/IpcBridge';
export type { ParameterInfo } from './types/parameters';

// ✅ Internal imports use direct paths
import { IpcBridge } from './ipc/IpcBridge';
import type { MeterFrame } from './types/metering';
```

**Don't:**
```typescript
// ❌ Internal barrel file (hooks/index.ts)
export { useParameter } from './useParameter';
export { useAllParameters } from './useAllParameters';
// ...then importing from barrel
import { useParameter } from './hooks';  // Avoid this pattern
```

### Import Aliases

**Rule:** Use npm package imports for the SDK, not relative paths.

The UI SDK is distributed as npm packages. User plugins and the internal dev app both import from these packages:

| Package | Exports | Usage |
|---------|---------|-------|
| `@wavecraft/core` | IPC, hooks, Logger, types | Core SDK functionality |
| `@wavecraft/core/meters` | `linearToDb`, `dbToLinear` | Pure audio math utilities |
| `@wavecraft/components` | `Meter`, `ParameterSlider`, etc. | Pre-built React components |

**Do:**
```typescript
// ✅ Import from npm packages
import { useParameter, useAllParameters, Logger } from '@wavecraft/core';
import { linearToDb, dbToLinear } from '@wavecraft/core/meters';
import { Meter, ParameterSlider } from '@wavecraft/components';
```

**Don't:**
```typescript
// ❌ Relative imports (no longer applicable for SDK usage)
import { useParameter } from '../lib/wavecraft-ipc';
import { Meter } from '../../components/Meter';
```

**Subpath Exports:**

The `@wavecraft/core/meters` subpath provides access to pure utility functions (`linearToDb`, `dbToLinear`, `getMeterFrame`) without triggering IPC initialization side effects. Use this subpath:
- In unit tests for pure functions
- When you only need math utilities, not hooks or clients

**Rationale:**
- Standard npm package consumption pattern
- Version management via package.json
- Clear separation between SDK and user code
- Tree-shaking support via ESM exports
- Subpath exports avoid initialization side effects in tests

### Global Object Access

**Rule:** Use `globalThis` instead of `window` for accessing the global object.

This applies to:
- TypeScript/JavaScript source files
- JavaScript strings embedded in Rust code (e.g., `evaluate_script` calls)

**Do:**
```typescript
// ✅ Use globalThis in TypeScript/JavaScript
globalThis.wavecraft?.invoke('getParameter', { id });
globalThis.addEventListener('message', handler);
```

```rust
// ✅ Use globalThis in embedded JavaScript (Rust)
let js = format!(
    "if (globalThis.__WAVECRAFT_IPC__ && globalThis.__WAVECRAFT_IPC__._onParamUpdate) {{ \
        globalThis.__WAVECRAFT_IPC__._onParamUpdate({{ id: '{}', value: {} }}); \
    }}",
    id, value
);
webview.evaluate_script(&js);
```

**Don't:**
```typescript
// ❌ Using window in TypeScript/JavaScript
window.wavecraft?.invoke('getParameter', { id });
window.addEventListener('message', handler);
```

```rust
// ❌ Using window in embedded JavaScript (Rust)
let js = format!(
    "if (window.__WAVECRAFT_IPC__) {{ window.__WAVECRAFT_IPC__._onParamUpdate(...); }}"
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
- `wavecraft-nih_plug` — nih-plug integration, WebView editor, plugin exports (`publish = false`, git-only)
- `wavecraft-core` — Core SDK types and declarative macros (publishable, no nih_plug dependency)
- `wavecraft-macros` — Procedural macros: `ProcessorParams` derive, `wavecraft_plugin!`
- `wavecraft-protocol` — Shared contracts and types
- `wavecraft-dsp` — Pure DSP code, `Processor` trait, built-in processors
- `wavecraft-bridge` — IPC handling
- `wavecraft-metering` — SPSC ring buffer for audio → UI metering
- `wavecraft-dev-server` — Development server for browser-based UI testing

### Declarative Plugin DSL

**Rule:** Use the declarative DSL macros for new plugin definitions. Manual `Plugin` implementations should be avoided unless necessary for advanced use cases.

**Processor Wrapper Macro:**
```rust
// ✅ Use wavecraft_processor! for named processor wrappers
wavecraft_processor!(InputGain => Gain);
wavecraft_processor!(OutputStage => Passthrough);
```

**Plugin Definition Macro:**
```rust
// ✅ Use wavecraft_plugin! for complete plugin generation
wavecraft_plugin! {
    name: "My Plugin",
    vendor: "My Company",
    url: "https://example.com",      // optional
    email: "info@example.com",       // optional
    signal: InputGain,
}
```

**Parameter Definition:**
```rust
// ✅ Use #[derive(ProcessorParams)] for parameter structs
#[derive(ProcessorParams, Default)]
struct GainParams {
    #[param(range = "-60.0..=24.0", default = 0.0, unit = "dB")]
    gain: f32,
    
    #[param(range = "0.0..=1.0", default = 1.0, unit = "%", group = "Output")]
    mix: f32,
}
```

**Param Attribute Options:**
| Attribute | Required | Description | Example |
|-----------|----------|-------------|---------|
| `range` | Yes | Value range as `"MIN..=MAX"` | `range = "-60.0..=24.0"` |
| `default` | No | Default value (defaults to midpoint) | `default = 0.0` |
| `unit` | No | Unit string for display | `unit = "dB"` |
| `factor` | No | Skew factor (>1 = log, <1 = exp) | `factor = 2.5` |
| `group` | No | UI grouping name | `group = "Input"` |

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

Wavecraft is primarily developed for macOS, with the editor/WebView components being platform-specific. Code that only runs on certain platforms should be properly gated.

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

Wavecraft uses Vitest and React Testing Library for UI unit testing.

### Documentation Examples (Rust doctests)

**Rule:** Prefer compiling doctests over ignored ones.

Use the following conventions for Rust doc examples:

- **`rust,no_run`** for examples that should compile but don’t need to execute.
- **`text`** for cross-crate or illustrative snippets that cannot compile in the current crate.
- **Avoid `ignore`** unless there’s a hard external dependency that can’t be represented.

**Do:**
```rust
/// ```rust,no_run
/// use wavecraft_core::prelude::*;
/// ```
```

**Do (non-compiling illustration):**
```text
/// ```text
/// use wavecraft::prelude::*; // via Cargo rename in downstream crate
/// ```
```

### Pre-Push Validation

**Rule:** Always run `cargo xtask ci-check` before pushing changes.

This command simulates CI checks locally and runs ~26x faster than Docker-based CI:

```bash
# Run all checks (lint + tests, ~1 minute)
cargo xtask ci-check

# Run with auto-fix for linting issues
cargo xtask ci-check --fix

# Skip certain phases
cargo xtask ci-check --skip-lint
cargo xtask ci-check --skip-tests
```

**What it runs:**
1. **Linting**: ESLint, Prettier, cargo fmt, clippy (with optional --fix)
2. **Automated Tests**: Engine (Rust) + UI (Vitest) tests

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
├── src/                           # Dev app (limited tests)
└── test/
    ├── setup.ts                   # Global test setup
    └── mocks/
        └── ipc.ts                 # IPC mock module
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
3. `cargo xtask bundle` produces valid plugin bundles
4. Plugin loads in a DAW

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

### SDK Distribution Versioning (CI Auto-Bump)

**Rule:** Distribution packages (CLI, npm) are version-bumped automatically by CI. Do not manually bump these versions for publish-only changes.

The CD pipeline (`continuous-deploy.yml`) automatically patches and publishes distribution packages when any SDK component changes. This ensures users always get the latest SDK via `cargo install wavecraft` or `npm install @wavecraft/core`.

**Two version domains exist:**

| Domain | Packages | Owner | Bumped By |
|--------|----------|-------|-----------|
| **Product Version** | `engine/Cargo.toml` workspace version | PO (decides), Coder (implements) | Manual — during feature development |
| **Distribution Version** | CLI (`cli/Cargo.toml`), `@wavecraft/core`, `@wavecraft/components` | CI | Automatic — patch bump on any SDK change |

**How it works:**
1. A push to `main` triggers the CD pipeline
2. `detect-changes` identifies which SDK components changed
3. If **any** component changed, the CLI is also published (cascade trigger)
4. For each package, CI compares the local version against the published registry version
5. If the local version is not ahead, CI auto-bumps the patch version, commits as `github-actions[bot]`, and publishes
6. The `github-actions[bot]` author is detected by the pipeline to prevent infinite re-triggering

**What developers should do:**
- Bump `engine/Cargo.toml` workspace version for product milestones (as before)
- Do **not** manually bump CLI or npm package versions unless making a deliberate breaking change
- If you need a specific version (e.g., minor bump for a breaking CLI change), bump it in your PR — CI will respect the manual bump

**What CI does:**
- Auto-patches CLI, `@wavecraft/core`, `@wavecraft/components` when their local version ≤ registry version
- Commits version bumps as `github-actions[bot]` author
- Uses `git pull --rebase` before each push to handle parallel job conflicts

**Infinite loop prevention:**
- Commits authored by `github-actions[bot]` are skipped by the `detect-changes` job
- This is more robust than commit message markers (which can false-match on squash merge bodies)
- Preferred over `[skip ci]` because other workflows (CI, template validation) should still run on auto-bump commits

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

### Logging

**Rule:** Use structured logging instead of direct console calls or println!.

**UI Logging (TypeScript):**

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

**Log Levels:**

| Level | Usage | Production |
|-------|-------|------------|
| `DEBUG` | Verbose tracing, request/response details | Hidden |
| `INFO` | Significant events (connection, init) | Visible |
| `WARN` | Recoverable issues, degraded operation | Visible |
| `ERROR` | Failures requiring attention | Visible |

**Structured Context:**

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

**Engine Logging (Rust):**

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

**Exceptions (println! allowed):**

- `xtask` CLI commands — these are intentional user-facing output
- Benchmark/test output — `println!` is acceptable for test diagnostics

**Rationale:**
- Structured logging enables filtering, searching, and analysis
- Consistent log format across UI and Engine
- Production builds can adjust log levels without code changes
- Context objects preserve machine-parseable data

### Error Handling

- TypeScript: Use explicit error types, avoid `any`
- Rust: Use `Result<T, E>` with descriptive error types
- Always handle errors explicitly; avoid silent failures

### Validation Against Language Specifications

**Rule:** When validating identifiers, keywords, or language constructs, use the language's own parser/lexer libraries instead of maintaining custom lists.

**Rationale:**
- **Future-proof**: Automatically stays current with language updates (new keywords, editions)
- **Authoritative**: Uses the language's official rules as source of truth
- **Comprehensive**: Covers all cases including strict keywords, reserved words, and edition-specific additions
- **Maintainable**: No manual lists to keep in sync

**Do (Rust keyword validation):**
```rust
use syn;

/// Validates that a name is not a Rust keyword.
/// Uses syn's parser - the same rules Rust itself uses.
pub fn validate_not_keyword(name: &str) -> Result<()> {
    // Convert hyphens to underscores (crate names allow hyphens)
    let ident_name = name.replace('-', "_");
    
    // syn::parse_str::<syn::Ident>() fails for keywords
    if syn::parse_str::<syn::Ident>(&ident_name).is_err() {
        bail!("'{}' is a reserved Rust keyword", name);
    }
    Ok(())
}
```

**Don't (hardcoded keyword list):**
```rust
// ❌ Hardcoded list becomes stale as language evolves
const KEYWORDS: &[&str] = &[
    "fn", "let", "if", "else", "match", // incomplete...
    // Missing: async, await, try, dyn, etc.
];

fn validate_not_keyword(name: &str) -> Result<()> {
    if KEYWORDS.contains(&name) {
        bail!("Reserved keyword");
    }
    Ok(())
}
```

**Why syn for Rust:**
- `syn` is the de-facto standard Rust parser, used by proc-macros
- `syn::Ident` parsing uses Rust's official keyword list
- Automatically includes edition-specific keywords (e.g., `async`/`await` in 2018+)
- Zero maintenance burden for keyword list updates

**Similar patterns for other languages:**
- **TypeScript**: Use TypeScript compiler API for identifier validation
- **JavaScript**: Use `acorn` or `esprima` parser libraries

### Rust `unwrap()` and `expect()` Usage

**Rule:** Avoid `unwrap()` in production code. Use `expect()` with descriptive messages or proper error handling.

**Rationale:**
- `unwrap()` panics without context, making debugging difficult
- `expect()` provides a message explaining why the operation should succeed
- Proper error handling with `?` is preferred when errors are recoverable

**Production Code:**

```rust
// ✅ Use expect() with justification for infallible operations
// Serialization of well-typed Response structs cannot fail because:
// - All fields are simple types (strings, numbers, Options)
// - No custom serializers that could error
// - serde_json always succeeds for #[derive(Serialize)] types
serde_json::to_string(&response).expect("Response serialization is infallible")

// ✅ Use ? operator for fallible operations
let config = SigningConfig::from_env()?;
let data = serde_json::from_str::<Request>(json)?;

// ✅ Use if-let or match for optional handling
if let Some(param) = params.get(id) {
    // use param
}

// ❌ Avoid bare unwrap() in production
let value = some_option.unwrap();  // No context if it fails
let data = serde_json::from_str(json).unwrap();  // Hides parse errors
```

**Test Code:**

```rust
// ✅ Prefer expect() with descriptive messages in tests
let result: GetParameterResult = serde_json::from_value(response.result.clone())
    .expect("response should contain valid GetParameterResult");

// ✅ Use assert! macros for test assertions
assert!(response.result.is_some(), "expected successful response");
assert_eq!(result.value, 0.5, "parameter value should be 0.5");

// ⚠️ unwrap() is acceptable in tests when the intent is obvious
// but expect() is preferred for better failure messages
let error = response.error.unwrap();  // Acceptable but not ideal
```

**When `unwrap()` is Acceptable:**

1. **Infallible operations with documentation**: When an operation mathematically cannot fail and this is documented in a comment
2. **Test setup code**: Where failure indicates a test bug, not a product bug
3. **Compile-time constants**: `NonZeroU32::new(2).unwrap()` in const contexts

**Pattern for IPC Response Serialization:**

The `IpcHandler::handle_json()` method uses `unwrap()` for serializing responses. This is acceptable because:

```rust
// IpcResponse derives Serialize with simple field types:
// - RequestId (enum of u32/String)
// - Option<Value> (serde_json::Value, always serializable)
// - Option<IpcError> (simple struct with String fields)
//
// serde_json::to_string() cannot fail for these types.
serde_json::to_string(&response).expect("IpcResponse serialization is infallible")
```