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
│       ├── IpcBridge.ts      # Class: low-level bridge
│       ├── ParameterClient.ts # Class: high-level API
│       └── hooks.ts          # React hooks (functional)
├── components/
│   ├── ParameterSlider.tsx   # Functional component
│   └── LatencyMonitor.tsx    # Functional component
└── App.tsx
```

**Note:** Class files should be named with PascalCase matching the class name.

### Import Aliases

**Rule:** Use configured path aliases instead of relative imports for shared libraries.

The project defines the following import aliases (configured in `tsconfig.json` and `vite.config.ts`):

| Alias | Path | Usage |
|-------|------|-------|
| `@vstkit/ipc` | `./src/lib/vstkit-ipc` | IPC client and types |

**Do:**
```typescript
// ✅ Use alias for shared libraries
import { getMeterFrame, MeterFrame } from '@vstkit/ipc';
import { useParameter } from '@vstkit/ipc';
```

**Don't:**
```typescript
// ❌ Relative imports to shared libraries
import { getMeterFrame } from '../lib/vstkit-ipc';
import { useParameter } from '../../lib/vstkit-ipc';
```

**Rationale:**
- Cleaner imports that don't change when files move
- Immediately identifies imports as project-internal
- Consistent import paths across the codebase

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

### Real-Time Safety

Code running on the audio thread must:
- Never allocate (`Box::new`, `Vec::push`, `String::from`)
- Never lock (`Mutex`, `RwLock`)
- Never make system calls that can block
- Use atomic types for shared state
- Use SPSC ring buffers for data transfer

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
